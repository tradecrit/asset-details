use tonic::{Status};
use tonic::transport::{Channel, Error};

use crate::authentication::authentication::authentication_client::AuthenticationClient;
use crate::authentication::authentication::VerifyRequest;

pub mod authentication {
    tonic::include_proto!("authentication");
}

/// Check the authentication server to verify a token
/// 
/// # Arguments
/// 
/// * `auth_url` - The URL of the authentication server
/// * `token` - The token to verify
/// 
/// # Returns
/// 
/// * If the token is verified, returns Ok(())
/// 
/// # Errors
/// 
/// * If the token is not verified, returns a Status error
pub async fn check_auth(auth_url: String, token: &str) -> Result<(), Status> {
    tracing::debug!("Calling auth server at {} to verify token", auth_url);

    let open_authentication_client: Result<AuthenticationClient<Channel>, Error> = AuthenticationClient::connect(auth_url).await;

    let mut authentication_client: AuthenticationClient<Channel> = match open_authentication_client {
        Ok(client) => client,
        Err(e) => {
            return Err(Status::unauthenticated(format!("Error connecting to auth server: {:?}", e)));
        }
    };

    tracing::debug!("Connected to auth server, checking token...");

    // Notice how the token is passed in the body not a header, this is for the ability to layer
    // authentication mechanisms, so that client_credentials from the service call the
    // authentication service to verify the user or other token it received.
    let verify_response = authentication_client.verify_token(VerifyRequest {
        token: token.to_string(),
    }).await;

    match verify_response {
        Ok(data) => {
            if data.get_ref().authenticated {
                tracing::debug!("Token verified");

                let response = data.get_ref().clone();

                let authenticated_status = response.authenticated;

                if authenticated_status {
                    Ok(())
                } else {
                    Err(Status::unauthenticated("Unauthenticated"))
                }
            } else {
                Err(Status::unauthenticated(data.get_ref().message.clone()))
            }
        }
        Err(e) => {
            Err(Status::unauthenticated(format!("Error verifying token: {:?}", e)))
        }
    }
}
