use std::env;
use crate::StripQuotes;

pub fn get_required_env_var(key: &str) -> String {
    let env_var = env::var(key)
        .unwrap_or_else(|_| {
            tracing::error!("{} not set", key);
            std::process::exit(1);
        });

    env_var.strip_quotes()
}

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
        env::set_var("TEST_ENV_VAR", "test_value");
        assert_eq!(get_required_env_var("TEST_ENV_VAR"), "test_value");
    }

    #[test]
    #[should_panic]
    fn test_get_required_env_var_panic() {
        get_required_env_var("TEST_ENV_VAR");
    }

    #[test]
    fn test_get_optional_env_var() {
        env::set_var("TEST_ENV_VAR", "test_value");
        assert_eq!(get_optional_env_var("TEST_ENV_VAR", "default_value".to_string()), "test_value");
    }

    #[test]
    fn test_get_optional_env_var_default() {
        assert_eq!(get_optional_env_var("TEST_ENV_VAR", "default_value".to_string()), "default_value");
    }
}