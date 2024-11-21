use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Company::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Company::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Company::Symbol).string().unique_key().not_null())
                    .col(ColumnDef::new(Company::Address).string())
                    .col(ColumnDef::new(Company::City).string())
                    .col(ColumnDef::new(Company::State).string())
                    .col(ColumnDef::new(Company::Zip).string())
                    .col(ColumnDef::new(Company::IconUrl).string())
                    .col(ColumnDef::new(Company::LogoUrl).string())
                    .col(ColumnDef::new(Company::Cik).string())
                    .col(ColumnDef::new(Company::Description).string())
                    .col(ColumnDef::new(Company::HomepageUrl).string())
                    .col(ColumnDef::new(Company::ListDate).date())
                    .col(ColumnDef::new(Company::MarketCap).decimal())
                    .col(ColumnDef::new(Company::Name).string().not_null())
                    .col(ColumnDef::new(Company::PhoneNumber).string())
                    .col(ColumnDef::new(Company::PrimaryExchangeId).string())
                    .col(ColumnDef::new(Company::PrimaryExchangeName).string())
                    .col(ColumnDef::new(Company::SicCode).string())
                    .col(ColumnDef::new(Company::SicDescription).string())
                    .col(ColumnDef::new(Company::TotalEmployees).big_integer())
                    .col(ColumnDef::new(Company::WeightedSharesOutstanding).big_integer())
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(Index::create()
                .name("idx-detail-symbol")
                .table(Company::Table)
                .col(Company::Symbol)
                .to_owned())
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Company::Table).to_owned())
            .await
            .expect("cant drop table");

        Ok(())
    }
}

#[derive(DeriveIden)]
pub(crate) enum Company {
    Table,
    Id,
    Address,
    City,
    State,
    Zip,
    IconUrl,
    LogoUrl,
    Cik,
    Description,
    HomepageUrl,
    ListDate,
    MarketCap,
    Name,
    PhoneNumber,
    PrimaryExchangeId,
    PrimaryExchangeName,
    SicCode,
    SicDescription,
    Symbol,
    TotalEmployees,
    WeightedSharesOutstanding,
}
