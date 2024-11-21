use std::collections::HashMap;
use polygon_sdk::models::{Exchange, Stock};
use utils::error::{Error, ErrorType};

/// Get all stocks from the NYSE, NASDAQ, and AMEX exchanges, wrapper util for the Polygon SDK
/// 
/// # Arguments
/// 
/// * `client` - The Polygon SDK client
/// 
/// # Returns
/// 
/// A hashmap of stock tickers to stock details
/// 
/// # Errors
/// 
/// * If the request fails, returns a custom error
pub async fn get_stocks(client: &polygon_sdk::Client) -> Result<HashMap<String, Stock>, Error> {
    let mut all_stocks: Vec<Stock> = Vec::new();

    let nyse_stocks = client.fetch_exchange_stocks(
        Exchange::Nyse,
    )
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch NYSE stocks: {}", e);
            Error::new(ErrorType::ThirdPartyError, format!("Failed to fetch NYSE stocks: {}", e))
        })?;

    all_stocks.extend(nyse_stocks);

    let nasdaq_stocks = client.fetch_exchange_stocks(
        Exchange::Nasdaq,
    )
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch NASDAQ stocks: {}", e);
            Error::new(ErrorType::ThirdPartyError, format!("Failed to fetch NASDAQ stocks: {}", e))
        })?;

    all_stocks.extend(nasdaq_stocks);

    let amex_stocks = client.fetch_exchange_stocks(
        Exchange::Amex,
    )
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch AMEX stocks: {}", e);
            Error::new(ErrorType::ThirdPartyError, format!("Failed to fetch AMEX stocks: {}", e))
        })?;

    all_stocks.extend(amex_stocks);

    tracing::info!("Found {} NYSE/NASDAQ/AMEX stocks", all_stocks.len());

    let stocks: HashMap<String, Stock> = HashMap::from_iter(all_stocks.into_iter().map(|stock| (stock.ticker.clone(), stock)));

    Ok(stocks)
}
