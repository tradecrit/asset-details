use sea_orm_migration::prelude::*;


#[tokio::main]
pub async fn main() {
    cli::run_cli(migration::Migrator).await;
}
