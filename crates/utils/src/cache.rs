use redis::{Client, Commands, Connection, ConnectionAddr, ConnectionInfo, ProtocolVersion, RedisConnectionInfo};
use serde::de::DeserializeOwned;
use serde::Serialize;
use crate::error::{Error, ErrorType};
use crate::error::ErrorType::{CacheError, ParseError};
use crate::StripQuotes;

pub fn init_redis(uri: String, password: Option<String>) -> Result<redis::Client, Error>{
    let cache_connection_data = uri.split(":").collect::<Vec<&str>>();

    let conn_address = cache_connection_data[0];

    let raw_port = cache_connection_data[1];
    let conn_port = match raw_port.parse::<u16>() {
        Ok(p) => p,
        Err(e) => {
            tracing::error!("Error parsing Redis port: {:?}", e);
            return Err(Error {
                error_type: CacheError,
                message: e.to_string(),
            });
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

    Ok(redis_client)
}


/// Check the cache for a generic key and return the value if it exists
/// Shared fetch, handle and parse logic for all cache fetch operations
///
/// # Arguments
///
/// * `connection` - The Redis connection
/// * `key` - The cache key
///
/// # Returns
///
/// The deserialized value of the cache key, of generic type T
///
/// # Errors
///
/// * If the cache key does not exist, returns a CacheMiss error which is a custom error. This
/// is due to the Redis crate returning a TypeError when a key does not exist, which is not
/// a useful error message for the caller. This maps the type error for Redis type nil to a
/// custom error type Miss.
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

/// Set the cache for a generic key with a value
/// Shared set logic for all cache set operations
///
/// # Arguments
///
/// * `connection` - The Redis connection
/// * `key` - The cache key
/// * `data` - The data to cache
/// * `expires_in` - The expiry time for the cache, in seconds
///
/// # Returns
///
/// A Result with an empty tuple if successful
///
/// # Errors
///
/// * If the cache set operation fails, returns a CacheError which is a custom error with details
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
