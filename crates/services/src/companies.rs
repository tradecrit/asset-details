use chrono::NaiveDate;
use polygon_sdk::models::CompanyDetails;
use sea_orm::{sea_query, ActiveValue, DatabaseConnection, EntityTrait, InsertResult};
use uuid::Uuid;
use utils::error::{Error, ErrorType};
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;
use entities::company;
use entities::company::{ActiveModel};

enum ExchangeCode {
    XNYS,
    XNAS,
    ARCX,
    BATS,
    XASE
}

impl ExchangeCode {
    pub fn name(&self) -> &str {
        match self {
            ExchangeCode::XNYS => "New York Stock Exchange",
            ExchangeCode::XNAS => "Nasdaq",
            ExchangeCode::ARCX => "NYSE Arca",
            ExchangeCode::BATS => "Cboe BZX Exchange",
            ExchangeCode::XASE => "NYSE American",
        }
    }

    pub fn iso_code(&self) -> &str {
        match self {
            ExchangeCode::XNYS => "XNYS",
            ExchangeCode::XNAS => "XNAS",
            ExchangeCode::ARCX => "ARCX",
            ExchangeCode::BATS => "BATS",
            ExchangeCode::XASE => "XASE",
        }
    }
}

impl From<String> for ExchangeCode {
    fn from(code: String) -> Self {
        match code.as_str() {
            "XNYS" => ExchangeCode::XNYS,
            "XNAS" => ExchangeCode::XNAS,
            "ARCX" => ExchangeCode::ARCX,
            "BATS" => ExchangeCode::BATS,
            "XASE" => ExchangeCode::XASE,
            _ => ExchangeCode::XNYS,
        }
    }
}

pub async fn find_existing_or_create(database_connection: &DatabaseConnection, company_details: CompanyDetails) -> Result<(), Error> {
    let new_id = Uuid::now_v7().into();

    let address = company_details.address.clone();
    let branding = company_details.branding.clone();

    let parsed_exchange_id: Option<String> = match company_details.primary_exchange.clone() {
        Some(value) => Some(ExchangeCode::from(value).iso_code().to_owned()),
        None => None,
    };

    let parsed_exchange_name: Option<String> = match company_details.primary_exchange.clone() {
        Some(value) => Some(ExchangeCode::from(value).name().to_owned()),
        None => None,
    };

    let parsed_list_date: Option<NaiveDate> = match company_details.list_date {
        Some(value) => {
            match NaiveDate::parse_from_str(&value, "%Y-%m-%d") {
                Ok(date) => Some(date),
                Err(e) => {
                    tracing::error!("Failed to parse list date: {}", e);
                    return Err(Error::new(ErrorType::ParseError, format!("Failed to parse list date: {}", e)));
                }
            }
        }
        None => None,
    };

    let parsed_market_cap: Option<Decimal> = company_details.market_cap.and_then(Decimal::from_f64);

    let address_value: Option<String> = match &address {
        Some(value) => value.address1.clone(),
        None => None,
    };

    let city_value: Option<String> = match &address {
        Some(value) => value.city.clone(),
        None => None,
    };

    let state_value: Option<String> = match &address {
        Some(value) => value.state.clone(),
        None => None,
    };

    let zip_value: Option<String> = match &address {
        Some(value) => value.postal_code.clone(),
        None => None,
    };

    let icon_url_value: Option<String> = match &branding {
        Some(value) => value.icon_url.clone(),
        None => None,
    };

    let logo_url_value: Option<String> = match &branding {
        Some(value) => value.logo_url.clone(),
        None => None,
    };

    let entry = company::ActiveModel {
        id: ActiveValue::Set(new_id),
        symbol: ActiveValue::Set(company_details.ticker.clone()),
        name: ActiveValue::Set(company_details.name.clone()),
        description: ActiveValue::Set(company_details.description.clone()),
        address: ActiveValue::Set(address_value),
        city: ActiveValue::Set(city_value),
        state: ActiveValue::Set(state_value),
        zip: ActiveValue::Set(zip_value),
        icon_url: ActiveValue::Set(icon_url_value),
        logo_url: ActiveValue::Set(logo_url_value),
        cik: ActiveValue::Set(company_details.cik.clone()),
        homepage_url: ActiveValue::Set(company_details.homepage_url.clone()),
        list_date: ActiveValue::Set(parsed_list_date),
        market_cap: ActiveValue::Set(parsed_market_cap),
        phone_number: ActiveValue::Set(company_details.phone_number.clone()),
        primary_exchange_id: ActiveValue::Set(parsed_exchange_id),
        primary_exchange_name: ActiveValue::Set(parsed_exchange_name),
        sic_code: ActiveValue::Set(company_details.sic_code.clone()),
        sic_description: ActiveValue::Set(company_details.sic_description.clone()),
        total_employees: ActiveValue::Set(company_details.total_employees.clone()),
        weighted_shares_outstanding: ActiveValue::Set(company_details.weighted_shares_outstanding),
    };

    let conflict_statement = sea_query::OnConflict::column(company::Column::Symbol)
        .update_columns(vec![
            company::Column::WeightedSharesOutstanding,
            company::Column::MarketCap,
            company::Column::TotalEmployees,
            company::Column::LogoUrl,
            company::Column::IconUrl,
        ])
        .to_owned();

    let model: InsertResult<ActiveModel> = company::Entity::insert(entry)
        .on_conflict(conflict_statement)
        .exec(database_connection)
        .await
        .map_err(|e| {
            tracing::error!("Failed to insert company details: {}", e);
            Error::new(ErrorType::DatabaseError, format!("Failed to insert company details: {}", e))
        })?;

    tracing::debug!("Inserted company details: {:?}", model);

    Ok(())
}
