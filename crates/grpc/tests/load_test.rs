use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};


#[cfg(test)]
mod tests {
    use super::*;

    use tokio::sync::{Barrier};
    use config::load_state;

    #[tokio::test]
    async fn test_verify_token() -> Result<(), Box<dyn std::error::Error>> {
        // Used to load tracing, telemetry config, etc.
        let _app_state = load_state().await;

        let token = utils::env::get_required_env_var("BEARER_TOKEN");
        let url = utils::env::get_required_env_var("API_URL");

        let concurrency = 10;

        let counter = Arc::new(AtomicUsize::new(0));
        let barrier = Arc::new(Barrier::new(concurrency + 1));

        // let mut tasks = vec![];

        for _ in 0..concurrency {
            let counter = Arc::clone(&counter);
            let barrier = Arc::clone(&barrier);

            // tasks.push(tokio::spawn(async move {
            //     let filter: Filter = Filter {
            //         asset_type: Some("stock".to_string()),
            //         event_type: Some("earning".to_string()),
            //         symbol: Some("AAPL".to_string()),
            //     };
            //
            //     let request = AssetEventsRequest {
            //         filter: Some(filter),
            //         start: "2024-07-31T00:00:00Z".to_string(),
            //         end: "2024-08-01T23:59:59Z".to_string(),
            //         limit: 1,
            //         next_item: None
            //     };
            //
            //     let mut client = AssetEventsClient::connect(url).await.map_err(|e| {
            //         eprintln!("Error connecting to server: {:?}", e);
            //     }).unwrap();
            //
            //     let mut request = tonic::Request::new(request.clone());
            //     request.metadata_mut().insert("authorization", token.parse().unwrap());
            //
            //     let response = client.get_events(request).await;
            //
            //     match response {
            //         Ok(_) => {
            //             counter.fetch_add(1, Ordering::SeqCst);
            //         }
            //         Err(e) => eprintln!("Error: {:?}", e),
            //     }

                // barrier.wait().await;
            // }));
        }

        barrier.wait().await;
        // let _ = futures::future::join_all(tasks).await;

        println!("Total successful responses: {}", counter.load(Ordering::SeqCst));

        Ok(())
    }
}
