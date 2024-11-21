use config::GlobalState;
use utils::cache::init_redis;
use utils::env::{get_optional_env_var, get_required_env_var};
use utils::error::Error;

#[derive(Debug, Clone)]
pub struct ApiState {
    pub global_state: GlobalState,
    pub cache_client: redis::Client,
    pub auth_url: String,
    pub address: String,
    pub port: String,
}


pub async fn load_state() -> Result<ApiState, Error> {
    let global_state: GlobalState = config::load_state().await;

    let auth_url: String = get_required_env_var("AUTH_URL");

    let address: String = get_optional_env_var("ADDRESS", "0.0.0.0".to_string());

    let port: String = get_optional_env_var("PORT", "50051".to_string());

    let cache_uri: String = get_required_env_var("CACHE_URL");

    let cache_client = init_redis(cache_uri, None)?;

    let app_state: ApiState = ApiState {
        global_state,
        cache_client,
        auth_url,
        address,
        port
    };

    Ok(app_state)
}
