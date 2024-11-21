use redis::{Commands, Connection};
use serde::de::DeserializeOwned;
use serde::Serialize;
use crate::error::{Error, ErrorType};
use crate::error::ErrorType::{CacheError, ParseError};

/// Check the cache for a generic key and return the value if it exists
/// Shared fetch, handle and parse logic for all cache fetch operations
pub fn check_cache<T: DeserializeOwned>(connection: &mut Connection, key: &str) -> Result<T, Error> {
    tracing::debug!("{}", format!("Checking cache for {}", key));

    let cached_data: String = connection.get(key).map_err(|error| {
        match error.kind() {
            redis::ErrorKind::TypeError => Error {
                error_type: ErrorType::CacheMiss,
                message: error.to_string(),
            },
            _ => Error {
                error_type: CacheError,
                message: error.to_string(),
            }
        }
    })?;

    let parse_data: T = serde_json::from_str(&cached_data).map_err(|error| {
        tracing::error!("Unable to parse cached data for {}", key);
        Error {
            error_type: ParseError,
            message: error.to_string(),
        }
    })?;

    Ok(parse_data)
}

pub fn set_cache<T>(
    connection: &mut Connection,
    key: &str,
    data: &T,
    expires_in: Option<u64>,
) -> Result<(), Error>
where
    T: Serialize,
{
    tracing::debug!("{}", format!("Setting cache for {}", key));

    let serialized_data: String = serde_json::to_string(data).map_err(|error| {
        tracing::error!("Unable to serialize data for {}", key);
        Error {
            error_type: ParseError,
            message: error.to_string(),
        }
    })?;

    let cache_expiry: u64 = expires_in.unwrap_or(3600);

    // Set the cache with an expiry, can't use ? operator here due to never type fallback issues
    let set_cache: Result<(), Error> = connection.set_ex(key, serialized_data, cache_expiry).map_err(|error| {
        tracing::error!("Unable to set cache for {}", key);
        Error {
            error_type: CacheError,
            message: error.to_string(),
        }
    });

    match set_cache {
        Ok(_) => {
            tracing::debug!("{}", format!("Successfully set cache for {}", key));
            Ok(())
        },
        Err(e) => Err(e),
    }
}
