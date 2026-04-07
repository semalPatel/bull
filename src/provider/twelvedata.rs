use crate::error::{BullError, Result};
use crate::model::Quote;
use crate::provider::QuoteProvider;
use chrono::{DateTime, TimeZone, Utc};
use serde::Deserialize;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct TwelveDataProvider {
    api_key: String,
    client: reqwest::blocking::Client,
}

impl TwelveDataProvider {
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

impl QuoteProvider for TwelveDataProvider {
    fn name(&self) -> &'static str {
        "twelvedata"
    }

    fn quote(&self, symbol: &str) -> Result<Quote> {
        let response = self
            .client
            .get("https://api.twelvedata.com/quote")
            .query(&[("symbol", symbol), ("apikey", self.api_key.as_str())])
            .send()?
            .error_for_status()?
            .text()?;
        parse_quote(symbol, &response)
    }
}

#[derive(Debug, Deserialize)]
struct TwelveDataQuote {
    symbol: Option<String>,
    close: Option<String>,
    change: Option<String>,
    percent_change: Option<String>,
    timestamp: Option<i64>,
    currency: Option<String>,
    status: Option<String>,
    message: Option<String>,
}

fn parse_quote(symbol: &str, payload: &str) -> Result<Quote> {
    let quote: TwelveDataQuote = serde_json::from_str(payload)?;
    if quote.status.as_deref() == Some("error") {
        return Err(BullError::Provider {
            provider: "twelvedata".to_string(),
            symbol: symbol.to_string(),
            message: quote
                .message
                .unwrap_or_else(|| "provider returned error".to_string()),
        });
    }

    let price = parse_number(symbol, quote.close.as_deref(), "close")?;
    Ok(Quote {
        symbol: quote.symbol.unwrap_or_else(|| symbol.to_string()),
        price,
        change: parse_optional_number(quote.change.as_deref()),
        change_percent: parse_optional_number(quote.percent_change.as_deref()),
        as_of: quote.timestamp.and_then(timestamp_to_utc),
        currency: quote.currency,
        source: "twelvedata".to_string(),
        stale: false,
    })
}

fn parse_number(symbol: &str, value: Option<&str>, field: &str) -> Result<f64> {
    value
        .and_then(|value| value.parse::<f64>().ok())
        .ok_or_else(|| BullError::Provider {
            provider: "twelvedata".to_string(),
            symbol: symbol.to_string(),
            message: format!("provider response did not include numeric {field}"),
        })
}

fn parse_optional_number(value: Option<&str>) -> Option<f64> {
    value.and_then(|value| value.parse::<f64>().ok())
}

fn timestamp_to_utc(timestamp: i64) -> Option<DateTime<Utc>> {
    Utc.timestamp_opt(timestamp, 0).single()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_twelvedata_quote_payload() {
        let payload = r#"{
            "symbol": "AAPL",
            "close": "182.31",
            "change": "1.20",
            "percent_change": "0.66",
            "timestamp": 1700000000,
            "currency": "USD"
        }"#;

        let quote = parse_quote("AAPL", payload).unwrap();

        assert_eq!(quote.symbol, "AAPL");
        assert_eq!(quote.price, 182.31);
        assert_eq!(quote.change, Some(1.2));
        assert_eq!(quote.source, "twelvedata");
    }
}
