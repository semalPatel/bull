use crate::error::{BullError, Result};
use crate::model::Quote;
use crate::provider::QuoteProvider;
use chrono::{NaiveDate, TimeZone, Utc};
use serde::Deserialize;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct AlphaVantageProvider {
    api_key: String,
    client: reqwest::blocking::Client,
}

impl AlphaVantageProvider {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            client: reqwest::blocking::Client::builder()
                .timeout(Duration::from_secs(6))
                .user_agent("bull-cli/0.1")
                .build()
                .expect("reqwest client configuration should be valid"),
        }
    }
}

impl QuoteProvider for AlphaVantageProvider {
    fn name(&self) -> &'static str {
        "alphavantage"
    }

    fn quote(&self, symbol: &str) -> Result<Quote> {
        let response = self
            .client
            .get("https://www.alphavantage.co/query")
            .query(&[
                ("function", "GLOBAL_QUOTE"),
                ("symbol", symbol),
                ("apikey", self.api_key.as_str()),
            ])
            .send()?
            .error_for_status()?
            .text()?;
        parse_quote(symbol, &response)
    }
}

#[derive(Debug, Deserialize)]
struct AlphaVantageResponse {
    #[serde(rename = "Global Quote")]
    global_quote: Option<AlphaVantageQuote>,
    #[serde(rename = "Note")]
    note: Option<String>,
    #[serde(rename = "Error Message")]
    error_message: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AlphaVantageQuote {
    #[serde(rename = "01. symbol")]
    symbol: String,
    #[serde(rename = "05. price")]
    price: String,
    #[serde(rename = "07. latest trading day")]
    latest_trading_day: Option<String>,
    #[serde(rename = "09. change")]
    change: Option<String>,
    #[serde(rename = "10. change percent")]
    change_percent: Option<String>,
}

fn parse_quote(symbol: &str, payload: &str) -> Result<Quote> {
    let response: AlphaVantageResponse = serde_json::from_str(payload)?;
    let missing_quote_message = response
        .error_message
        .or(response.note)
        .unwrap_or_else(|| "provider response did not include Global Quote".to_string());
    let quote = response.global_quote.ok_or_else(|| BullError::Provider {
        provider: "alphavantage".to_string(),
        symbol: symbol.to_string(),
        message: missing_quote_message,
    })?;

    let price = quote.price.parse::<f64>().map_err(|_| BullError::Provider {
        provider: "alphavantage".to_string(),
        symbol: symbol.to_string(),
        message: "provider response did not include numeric price".to_string(),
    })?;

    Ok(Quote {
        symbol: quote.symbol,
        price,
        change: quote.change.as_deref().and_then(parse_optional_number),
        change_percent: quote
            .change_percent
            .as_deref()
            .map(|value| value.trim_end_matches('%'))
            .and_then(parse_optional_number),
        as_of: quote.latest_trading_day.as_deref().and_then(parse_day),
        currency: Some("USD".to_string()),
        source: "alphavantage".to_string(),
        stale: false,
    })
}

fn parse_optional_number(value: &str) -> Option<f64> {
    value.parse::<f64>().ok()
}

fn parse_day(value: &str) -> Option<chrono::DateTime<Utc>> {
    NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .ok()
        .and_then(|date| date.and_hms_opt(0, 0, 0))
        .map(|datetime| Utc.from_utc_datetime(&datetime))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_alphavantage_quote_payload() {
        let payload = r#"{
            "Global Quote": {
                "01. symbol": "AAPL",
                "05. price": "182.3100",
                "07. latest trading day": "2026-04-06",
                "09. change": "1.2000",
                "10. change percent": "0.6600%"
            }
        }"#;

        let quote = parse_quote("AAPL", payload).unwrap();

        assert_eq!(quote.symbol, "AAPL");
        assert_eq!(quote.price, 182.31);
        assert_eq!(quote.change_percent, Some(0.66));
        assert_eq!(quote.source, "alphavantage");
    }
}
