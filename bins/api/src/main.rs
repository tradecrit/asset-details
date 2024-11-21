mod config;
mod auth_interceptor;

use crate::auth_interceptor::{AuthInterceptor, AuthServiceImpl};
use crate::config::ApiState;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tonic::transport::{Server};
use tonic_middleware::InterceptorFor;
use tower::ServiceBuilder;
use grpc::asset_details::asset_details::asset_details_server::AssetDetailsServer;

async fn start_server() -> Result<(), Box<dyn std::error::Error>> {
    let app_state: ApiState = config::load_state().await?;

    let server_address: String = format!("{}:{}", &app_state.address, &app_state.port);

    tracing::info!("Starting server on {}", server_address);

    let addr: SocketAddr = server_address.parse()?;

    let (_health_reporter, health_service) = tonic_health::server::health_reporter();

    let database_connection = app_state.global_state.database_client.clone();
    let cache_client = app_state.cache_client.clone();

    let asset_details = grpc::asset_details::AssetDetailsService {
        database_connection,
        cache_client
    };

    let asset_details_server = AssetDetailsServer::new(asset_details);

    // Create an instance of the auth interceptor, essentially a gRPC middleware for ensuring
    // that requests are authenticated
    let auth_url = app_state.auth_url;
    let auth_service = AuthServiceImpl::new(auth_url.clone());
    let auth_interceptor = AuthInterceptor {
        auth_service: Arc::new(auth_service),
    };

    // QoS for the server, including load shedding, timeouts, and concurrency limits
    let layered_server = ServiceBuilder::new()
        .load_shed()
        .timeout(Duration::from_secs(10)) // Increase timeout
        .into_inner();

    Server::builder()
        .concurrency_limit_per_connection(128) // Increase concurrency limit
        .timeout(Duration::from_secs(10)) // Increase timeout
        .max_connection_age(Duration::from_secs(60)) // Increase max connection age
        .tcp_keepalive(Some(Duration::from_secs(30))) // Enable TCP keepalive
        .http2_keepalive_interval(Some(Duration::from_secs(30))) // Enable HTTP/2 keepalive
        .http2_keepalive_timeout(Some(Duration::from_secs(10))) // Set HTTP/2 keepalive timeout
        .layer(layered_server)
        .add_service(health_service)
        .add_service(InterceptorFor::new(asset_details_server, auth_interceptor))
        .serve(addr)
        .await?;

    Ok(())
}

#[tokio::main]
async fn main() {
    let start = start_server().await;

    match start {
        Ok(_) => {
            tracing::info!("Server started successfully");
        },
        Err(e) => {
            tracing::error!("Error starting server: {:?}", e);
            std::process::exit(1);
        }
    }
}
