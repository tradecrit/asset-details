use rust_decimal::prelude::ToPrimitive;
use sea_orm::{DatabaseConnection, EntityTrait};
use tonic::{Response, Status};
use sea_orm::QueryFilter;
use sea_orm::ColumnTrait;
use entities::company;
use crate::asset_details::asset_details::{AssetDetailsRequest, AssetDetailsCompanyResponse};
use crate::asset_details::asset_details::asset_details_server::AssetDetails;
use entities::company::{Model};
use utils::error::ErrorType;

pub mod asset_details {
    tonic::include_proto!("asset_details");
}

#[derive(Debug)]
pub struct AssetDetailsService {
    pub database_connection: DatabaseConnection,
    pub cache_client: redis::Client,
}

#[tonic::async_trait]
impl AssetDetails for AssetDetailsService {
    async fn get_company(
        &self,
        request: tonic::Request<AssetDetailsRequest>,
    ) -> Result<Response<AssetDetailsCompanyResponse>, Status> {
        let incoming_request = request.into_inner();

        let symbol_to_find = incoming_request.symbol;

        tracing::info!("Fetching company details for symbol: {}", symbol_to_find);

        // First check cache, if missing then query DB
        let cache_key = format!("company_details:{}", symbol_to_find);

        let mut check_cache_connection = self.cache_client.get_connection().map_err(|e| {
            tracing::error!("Failed to get cache connection: {}", e);
            Status::internal("Failed to get cache connection")
        });

        let cached_company_details: Option<Model>  = match check_cache_connection {
            Ok(ref mut connection) => {
                let cached_entry = utils::cache::check_cache(connection, &cache_key);

                match cached_entry {
                    Ok(value) => Some(value),
                    Err(e) => {
                        match e.error_type {
                            ErrorType::CacheMiss => {
                                tracing::debug!("Cache miss for symbol: {}", symbol_to_find);
                            },
                            _ => {
                                tracing::error!("Failed to check cache: {}", e);
                            }
                        }
                        None
                    }
                }
            },
            Err(e) => {
                tracing::error!("Failed to get cache connection: {}", e);
                None
            }
        };

        if cached_company_details.is_some() {
            let cached_company: Model = cached_company_details.unwrap();

            tracing::info!("Cached company details found for symbol: {}", symbol_to_find);
            
            let response = AssetDetailsCompanyResponse {
                id: cached_company.id.to_string(),
                symbol: cached_company.symbol,
                name: cached_company.name,
                description: cached_company.description,
                address: cached_company.address,
                city: cached_company.city,
                state: cached_company.state,
                zip: cached_company.zip,
                logo_url: cached_company.logo_url,
                icon_url: cached_company.icon_url,
                cik: cached_company.cik,
                homepage_url: cached_company.homepage_url,
                list_date: cached_company.list_date.map(|value| value.to_string()),
                market_cap: cached_company.market_cap.map(|value| value.to_f64().unwrap_or_default()),
                phone_number: cached_company.phone_number,
                primary_exchange_id: cached_company.primary_exchange_id,
                primary_exchange_name: cached_company.primary_exchange_name,
                sic_code: cached_company.sic_code,
                sic_description: cached_company.sic_description,
                total_employees: cached_company.total_employees,
                weighted_shares_outstanding: cached_company.weighted_shares_outstanding,
            };

            return Ok(Response::new(response));
        }

        let query_result = company::Entity::find()
            .filter(company::Column::Symbol.eq(symbol_to_find.clone()))
            .all(&self.database_connection)
            .await
            .map_err(|e| {
                tracing::error!("Failed to execute query: {}", e);
                Status::internal("Failed to execute query")
            })?;

        let raw_company = match query_result.first() {
            Some(company) => company.clone(),
            None => {
                tracing::error!("Company details not found for symbol: {}", symbol_to_find);
                return Err(Status::not_found(format!("{} Company details not found", symbol_to_find)));
            }
        };

        // Cache the result
        tracing::info!("Caching company details for symbol: {}", symbol_to_find);
        let mut set_cache_connection = self.cache_client.get_connection().map_err(|e| {
            tracing::error!("Failed to get cache connection: {}", e);
            Status::internal("Failed to get cache connection")
        });

        match set_cache_connection {
            Ok(ref mut connection) => {
                tracing::info!("Cache connection established");
                let one_month: u64 = 60 * 60 * 24 * 30;
                utils::cache::set_cache(connection, &cache_key, &raw_company, Some(one_month)).map_err(|e| {
                    tracing::error!("Failed to cache result: {}", e);
                    Status::internal("Failed to cache result")
                })?;
            },
            Err(e) => {
                tracing::error!("Failed to get cache connection: {}", e);
            }
        }

        let parsed_list_date = raw_company.list_date.map(|value| value.to_string());

        let parsed_market_cap: Option<f64> = match raw_company.market_cap {
            Some(value) => value.to_f64(),
            None => None,
        };

        let response = AssetDetailsCompanyResponse {
            id: raw_company.id.to_string(),
            symbol: raw_company.symbol,
            name: raw_company.name,
            description: raw_company.description,
            address: raw_company.address,
            city: raw_company.city,
            state: raw_company.state,
            zip: raw_company.zip,
            logo_url: raw_company.logo_url,
            icon_url: raw_company.icon_url,
            cik: raw_company.cik,
            homepage_url: raw_company.homepage_url,
            list_date: parsed_list_date,
            market_cap: parsed_market_cap,
            phone_number: raw_company.phone_number,
            primary_exchange_id: raw_company.primary_exchange_id,
            primary_exchange_name: raw_company.primary_exchange_name,
            sic_code: raw_company.sic_code,
            sic_description: raw_company.sic_description,
            total_employees: raw_company.total_employees,
            weighted_shares_outstanding: raw_company.weighted_shares_outstanding,
        };

        tracing::info!("Company details found for symbol: {}", symbol_to_find);
        tracing::debug!("Company details: {:?}", response);

        Ok(Response::new(response))
    }
}
