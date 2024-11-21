mod config;
mod images;

use crate::config::IngestorState;
use polygon_sdk::models::{CompanyDetails, Stock};
use sea_orm::DatabaseConnection;
use services::stocks::get_stocks;
use std::collections::HashMap;
use utils::error::Error;


#[tokio::main]
async fn main() -> Result<(), Error> {
    let app_state: IngestorState = config::load_state().await?;

    let database_connection: DatabaseConnection = app_state.global_state.database_client;

    let cloudflare_api_key = app_state.cloudflare_api_key.clone();
    let cloudflare_account_id = app_state.cloudflare_account_id.clone();

    let cloudflare_client = cloudflare_sdk::Client::new(
        cloudflare_api_key,
        cloudflare_account_id
    );

    let polygon_client = polygon_sdk::Client::new(&app_state.polygon_api_key);

    let stocks: HashMap<String, Stock> = get_stocks(&polygon_client).await?;

    let total_length = stocks.len();
    let mut progress = 0;

    for (ticker, stock) in stocks.iter() {
        let percentage = (progress as f64 / total_length as f64) * 100.0;
        tracing::info!("Stock: {} - {} ({:.2}%)", ticker, stock.name, percentage);

        let fetch_company_details = polygon_client.fetch_company_details(ticker)
            .await
            .map_err(|e| {
                tracing::error!("Failed to fetch company details: {}", e);
                Error::new(utils::error::ErrorType::ThirdPartyError, format!("Failed to fetch company details: {}", e))
            });

        let mut company_details: CompanyDetails = match fetch_company_details {
            Ok(details) => details,
            Err(e) => {
                tracing::error!("Failed to fetch company details for {}: {}", ticker, e);
                continue;
            }
        };

        // Hackish but it's to make sure we don't store the private url
        let company_branding = company_details.branding;
        company_details.branding = None;

        match company_branding {
            Some(branding) => {
                let branding_result = images::process_branding_images(
                    &cloudflare_client,
                    ticker.to_string(),
                    branding,
                    app_state.polygon_api_key.clone()
                ).await;

                match branding_result {
                    Ok(new_branding) => {
                        tracing::info!("Successfully processed branding images for {}", ticker);
                        company_details.branding = Some(new_branding);
                    }
                    Err(e) => {
                        tracing::error!("Failed to process branding images for {}: {}", ticker, e);
                    }
                }
            }
            None => {
                tracing::info!("No branding images found for {}", ticker);
            }
        }

        let insert_result = services::companies::find_existing_or_create(&database_connection, company_details).await;

        match insert_result {
            Ok(_) => {
                tracing::info!("Successfully inserted company details for {}", ticker);
            }
            Err(e) => {
                tracing::error!("Failed to insert company details for {}: {}", ticker, e);
            }
        }

        progress += 1;
    }

    Ok(())
}
