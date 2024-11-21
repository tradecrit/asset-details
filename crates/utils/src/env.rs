use std::env;
use crate::StripQuotes;

/// Get a required environment variable, and panic if not set
/// 
/// # Arguments
/// 
/// * `key` - The environment variable key
/// 
/// # Returns
/// 
/// The value of the environment variable
pub fn get_required_env_var(key: &str) -> String {
    let env_var = env::var(key)
        .unwrap_or_else(|_| {
            tracing::error!("{} not set", key);
            panic!("{} not set", key);
        });

    env_var.strip_quotes()
}

/// Get an optional environment variable, with a default value if not set
/// 
/// # Arguments
/// 
/// * `key` - The environment variable key
/// * `default` - The default value to use if the environment variable is not set
/// 
/// # Returns
/// 
/// The value of the environment variable if set, otherwise the default value
pub fn get_optional_env_var(key: &str, default: String) -> String {
    match env::var(key) {
        Ok(val) => val.strip_quotes(),
        Err(_) => default,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_required_env_var() {
        env::set_var("REQ_TEST_ENV_VAR", "test_value");
        assert_eq!(get_required_env_var("REQ_TEST_ENV_VAR"), "test_value");
    }
    
    #[test]
    #[should_panic]
    fn test_get_required_env_var_panic() {
        get_required_env_var("REQ_TEST_ENV_VAR_NOT_SET");
    }

    #[test]
    fn test_get_optional_env_var() {
        env::set_var("OPT_TEST_ENV_VAR", "test_value");
        assert_eq!(get_optional_env_var("OPT_TEST_ENV_VAR", "default_value".to_string()), "test_value");
    }

    #[test]
    fn test_get_optional_env_var_default() {
        assert_eq!(get_optional_env_var("OPT_TEST_ENV_VAR_DEFAULT", "default_value".to_string()), "default_value");
    }
}
