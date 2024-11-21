use sea_orm_migration::prelude::*;

pub struct Migrator;

use sea_orm::DatabaseConnection;

mod m20240913_000001_company_table;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20240913_000001_company_table::Migration),
        ]
    }
}

pub async fn run_migrations(database: &DatabaseConnection) {
    let outcome = Migrator::up(database, None).await;

    match outcome {
        Ok(_) => {
            log::info!("Migrations applied successfully");
        }
        Err(e) => {
            log::error!("Error applying migrations: {:?}", e);
        }
    }
}
