use redis::{Client, ConnectionAddr, ConnectionInfo, ProtocolVersion, RedisConnectionInfo};
use config::GlobalState;
use utils::env::{get_optional_env_var, get_required_env_var};
use utils::error::Error;
use utils::StripQuotes;

#[derive(Debug, Clone)]
pub struct ApiState {
    pub global_state: GlobalState,
    pub cache_client: redis::Client,
    pub auth_url: String,
    pub address: String,
    pub port: String,
}

fn init_redis(uri: String, password: Option<String>) -> redis::Client {
    let cache_connection_data = uri.split(":").collect::<Vec<&str>>();

    let conn_address = cache_connection_data[0];

    let raw_port = cache_connection_data[1];
    let conn_port = match raw_port.parse::<u16>() {
        Ok(p) => p,
        Err(e) => {
            tracing::error!("Error parsing Redis port: {:?}", e);
            std::process::exit(1);
        }
    };

    tracing::info!("Connecting to Redis at {}:{}", conn_address, conn_port);

    let connection_address = ConnectionAddr::Tcp(conn_address.to_string(), conn_port);

    let sanitized_password: Option<String> = match password {
        Some(p) => Some(p.strip_quotes()),
        None => None
    };

    let redis_connection_info: RedisConnectionInfo = RedisConnectionInfo {
        db: 0,
        username: None,
        password: sanitized_password,
        protocol: ProtocolVersion::RESP3
    };

    let connection_info = ConnectionInfo {
        addr: connection_address,
        redis: redis_connection_info
    };

    let client = Client::open(connection_info);

    let redis_client = match client {
        Ok(c) => c,
        Err(e) => {
            tracing::error!("Error connecting to Redis: {:?}", e);
            std::process::exit(1);
        }
    };

    redis_client
}

pub async fn load_state() -> Result<ApiState, Error> {
    let global_state: GlobalState = config::load_state().await;

    let auth_url: String = get_required_env_var("AUTH_URL");

    let address: String = get_optional_env_var("ADDRESS", "0.0.0.0".to_string());

    let port: String = get_optional_env_var("PORT", "50051".to_string());

    let cache_uri: String = get_required_env_var("CACHE_URL");

    let cache_client = init_redis(cache_uri, None);

    // for each strip all single and double quote from start/end if present
    let app_state: ApiState = ApiState {
        global_state,
        cache_client,
        auth_url,
        address,
        port
    };

    Ok(app_state)
}
