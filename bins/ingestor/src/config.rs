use config::GlobalState;
use utils::env::{get_optional_env_var, get_required_env_var};
use utils::error::{Error};

#[derive(Debug, Clone)]
pub struct IngestorState {
    pub global_state: GlobalState,
    pub polygon_api_key: String,
    pub cloudflare_api_key: String,
    pub cloudflare_account_id: String,
    pub cloudflare_account_hash: Option<String>,
}

pub async fn load_state() -> Result<IngestorState, Error> {
    let global_state: GlobalState = config::load_state().await;

    let polygon_api_key = get_required_env_var("POLYGON_API_KEY");
    let cloudflare_api_key = get_required_env_var("CLOUDFLARE_API_KEY");
    let cloudflare_account_id = get_required_env_var("CLOUDFLARE_ACCOUNT_ID");

    let try_cloudflare_account_hash = get_optional_env_var("CLOUDFLARE_ACCOUNT_HASH", "".to_string());
    let cloudflare_account_hash: Option<String> = match try_cloudflare_account_hash.as_str() {
        "" => None,
        _ => Some(try_cloudflare_account_hash)
    };

    // for each strip all single and double quote from start/end if present
    let app_state: IngestorState = IngestorState {
        global_state,
        polygon_api_key,
        cloudflare_api_key,
        cloudflare_account_id,
        cloudflare_account_hash
    };

    Ok(app_state)
}
