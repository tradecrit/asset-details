use std::time::Duration;
use dotenvy::dotenv;
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use tracing::log;
use utils::env::{get_optional_env_var, get_required_env_var};
use migration;

#[derive(Debug, Clone)]
pub struct GlobalState {
    pub environment: String,
    pub database_client: DatabaseConnection,
}

fn init_observability(log_level: tracing::Level) {
    tracing_subscriber::fmt()
        .with_level(true)
        .with_max_level(log_level)
        .event_format(
            tracing_subscriber::fmt::format()
                .with_file(true)
                .with_line_number(true)
        )
        .json()
        .init();
}


async fn init_postgres(database_url: &str) -> DatabaseConnection {
    let mut opt = ConnectOptions::new(database_url);
    opt.max_connections(1024)
        .min_connections(5)
        .connect_timeout(Duration::from_secs(8))
        .acquire_timeout(Duration::from_secs(8))
        .idle_timeout(Duration::from_secs(8))
        .max_lifetime(Duration::from_secs(8))
        .sqlx_logging(false)
        .sqlx_logging_level(log::LevelFilter::Info)
        .set_schema_search_path("public");

    let database_pool: Result<DatabaseConnection, sea_orm::error::DbErr> = Database::connect(opt.clone()).await;

    match database_pool {
        Ok(pool) => {
            tracing::info!("Connected to Postgres");

            migration::run_migrations(&pool).await;

            pool
        }
        Err(e) => {
            tracing::error!("Error connecting to Postgres: {:?}", e);
            std::process::exit(1);
        }
    }
}

pub async fn load_state() -> GlobalState {
    // Log configuration and bootstrap
    let load_env = dotenv();
    if load_env.is_err() {
        tracing::warn!("No .env file found");
    }

    let raw_log_level: String = get_optional_env_var("RUST_LOG", "INFO".to_string());
    let uppercased_log_level: String = raw_log_level.to_uppercase();

    let tracing_level: tracing::Level = match uppercased_log_level.as_str() {
        "DEBUG" => tracing::Level::DEBUG,
        "INFO" => tracing::Level::INFO,
        "WARN" => tracing::Level::WARN,
        "ERROR" => tracing::Level::ERROR,
        _ => tracing::Level::INFO,
    };

    init_observability(tracing_level);

    tracing::info!("Starting application with tracing level: {}", tracing_level);

    // Core environment variables
    let environment = get_optional_env_var("ENVIRONMENT", "development".to_string());

    let database_url = get_required_env_var("DATABASE_URL");

    let database_client: DatabaseConnection = init_postgres(&database_url).await;

    let app_state: GlobalState = GlobalState {
        environment,
        database_client,
    };

    app_state
}
