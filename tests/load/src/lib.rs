

#[cfg(test)]
mod tests {
    use std::io::{Error, ErrorKind};
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use kinde_sdk::JwtRequestOptions;
    use tokio::sync::Barrier;
    use grpc::asset_details::asset_details;
    use grpc::asset_details::asset_details::asset_details_client::AssetDetailsClient;
    use grpc::asset_details::asset_details::asset_details_server::AssetDetailsServer;


    // API Integration test, ensure all components run together and can handle laod
    #[tokio::test]
    async fn test_query_company_details() -> Result<(), Error> {
        let global_state = config::load_state().await;

        let cache_url = utils::env::get_required_env_var("CACHE_URL");
        let cache_client = utils::cache::init_redis(cache_url, None)
            .map_err(|e| {
                tracing::error!("Error: {:?}", e);
                Error::new(ErrorKind::Other, "Failed to initialize cache client")
            })?;

        let oauth_domain = utils::env::get_required_env_var("OAUTH_DOMAIN");
        let client_id = utils::env::get_required_env_var("OAUTH_CLIENT_ID");
        let client_secret = utils::env::get_required_env_var("OAUTH_CLIENT_SECRET");

        let auth_client = kinde_sdk::Client::new(oauth_domain);
        let jwt_options = JwtRequestOptions {
            client_id,
            client_secret,
            audience: None,
        };

        // We have to generate client credentials for service authentication
        // This is due to how the grpc auth interceptor works, it must have a valid token
        let client_credentials = auth_client.get_client_credentials(jwt_options)
            .await
            .map_err(|e| {
                tracing::error!("Error: {:?}", e);
                Error::new(ErrorKind::Other, "Failed to get client credentials")
            })?;

        let concurrency = 100;
        let counter = Arc::new(AtomicUsize::new(0));
        let barrier = Arc::new(Barrier::new(concurrency + 1));

        let mut tasks = vec![];

        for _ in 0..concurrency {
            let counter = Arc::clone(&counter);
            let barrier = Arc::clone(&barrier);

            let api_url: String = utils::env::get_required_env_var("API_URL");
            let access_token: String = client_credentials.access_token.clone();

            tasks.push(tokio::spawn(async move {
                let try_client = AssetDetailsClient::connect(api_url)
                    .await
                    .map_err(|e| {
                        tracing::error!("Error: {:?}", e);
                        Error::new(ErrorKind::Other, "Failed to connect to API")
                    });

                let mut client = match try_client {
                    Ok(client) => client,
                    Err(e) => {
                        println!("Error: {:?}", e);
                        return;
                    }
                };

                let mut request = tonic::Request::new(asset_details::AssetDetailsRequest {
                    symbol: "AAPL".to_string(),
                });

                request.metadata_mut().insert("authorization", access_token.parse().unwrap());
                

                let response = client.get_company(request).await;

                match response {
                    Ok(_) => {
                        counter.fetch_add(1, Ordering::SeqCst);
                    }
                    Err(e) => tracing::error!("Error: {:?}", e),
                }

                barrier.wait().await;
            }));
        }

        barrier.wait().await;

        let _ = futures::future::join_all(tasks).await;

        tracing::info!("Total successful responses: {}", counter.load(Ordering::SeqCst));

        Ok(())
    }
}
