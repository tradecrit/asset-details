use std::sync::Arc;
use tonic::{async_trait, Status};
use tonic_middleware::RequestInterceptor;

use tonic::body::BoxBody;
use tonic::codegen::http::{Request};
use grpc::authentication::check_auth;


#[async_trait]
pub trait AuthService: Send + Sync {
    async fn verify_token(&self, token: &str) -> Result<(), String>;
}

#[derive(Clone)]
pub struct AuthServiceImpl {
    auth_url: String,
}

impl AuthServiceImpl {
    pub fn new(auth_url: String) -> Self {
        Self { auth_url }
    }
}

#[async_trait]
impl AuthService for AuthServiceImpl {
    async fn verify_token(&self, token: &str) -> Result<(), String> {
        let auth_url = self.auth_url.clone();
        check_auth(auth_url, token).await.map_err(|e| e.to_string())
    }
}

#[derive(Clone)]
pub struct AuthInterceptor<A: AuthService> {
    pub auth_service: Arc<A>,
}

#[async_trait]
impl<A: AuthService> RequestInterceptor for AuthInterceptor<A> {
    async fn intercept(&self, req: Request<BoxBody>) -> Result<Request<BoxBody>, Status> {
        match req.headers().get("authorization").map(|v| v.to_str()) {
            Some(Ok(header_data)) => {
                let parse_token = header_data.split_whitespace().collect::<Vec<&str>>();

                if parse_token.len() != 2 {
                    return Err(Status::unauthenticated("Unauthenticated"));
                }

                let token = parse_token[1];

                tracing::info!("Verifying token");

                // Verify the token using the auth service
                self.auth_service.verify_token(token).await.map_err(|e| {
                    tracing::error!("Error verifying token: {}", e);
                    Status::unauthenticated("Unauthenticated")
                })?;

                tracing::info!("Token verified");

                Ok(req)
            }
            _ => Err(Status::unauthenticated("Unauthenticated")),
        }
    }
}
