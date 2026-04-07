use crate::error::{BullError, Result};
use crate::model::Quote;
use crate::provider::QuoteProvider;
use chrono::{DateTime, TimeZone, Utc};
use serde::Deserialize;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct CommunityProvider {
    client: reqwest::blocking::Client,
}

impl CommunityProvider {
    pub fn new() -> Self {
        Self {
            client: reqwest::blocking::Client::builder()
                .timeout(Duration::from_secs(6))
                .user_agent("bull-cli/0.1")
                .build()
                .expect("reqwest client configuration should be valid"),
        }
    }
}

impl Default for CommunityProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl QuoteProvider for CommunityProvider {
    fn name(&self) -> &'static str {
        "community-yahoo"
    }

    fn quote(&self, symbol: &str) -> Result<Quote> {
        let url = format!(
            "https://query1.finance.yahoo.com/v7/finance/quote?symbols={}",
            symbol
        );
        let response = self.client.get(url).send()?.error_for_status()?.text()?;
        parse_quote(symbol, &response)
    }
}

#[derive(Debug, Deserialize)]
struct YahooResponse {
    #[serde(rename = "quoteResponse")]
    quote_response: YahooQuoteResponse,
}

#[derive(Debug, Deserialize)]
struct YahooQuoteResponse {
    result: Vec<YahooQuote>,
}

#[derive(Debug, Deserialize)]
struct YahooQuote {
    symbol: String,
    #[serde(rename = "regularMarketPrice")]
    regular_market_price: Option<f64>,
    #[serde(rename = "regularMarketChange")]
    regular_market_change: Option<f64>,
    #[serde(rename = "regularMarketChangePercent")]
    regular_market_change_percent: Option<f64>,
    #[serde(rename = "regularMarketTime")]
    regular_market_time: Option<i64>,
    currency: Option<String>,
}

fn parse_quote(symbol: &str, payload: &str) -> Result<Quote> {
    let response: YahooResponse = serde_json::from_str(payload)?;
    let quote = response
        .quote_response
        .result
        .into_iter()
        .find(|quote| quote.symbol.eq_ignore_ascii_case(symbol))
        .ok_or_else(|| BullError::Provider {
            provider: "community-yahoo".to_string(),
            symbol: symbol.to_string(),
            message: "symbol was not present in provider response".to_string(),
        })?;
    let price = quote.regular_market_price.ok_or_else(|| BullError::Provider {
        provider: "community-yahoo".to_string(),
        symbol: symbol.to_string(),
        message: "provider response did not include price".to_string(),
    })?;

    Ok(Quote {
        symbol: quote.symbol,
        price,
        change: quote.regular_market_change,
        change_percent: quote.regular_market_change_percent,
        as_of: quote.regular_market_time.and_then(timestamp_to_utc),
        currency: quote.currency,
        source: "community-yahoo".to_string(),
        stale: false,
    })
}

fn timestamp_to_utc(timestamp: i64) -> Option<DateTime<Utc>> {
    Utc.timestamp_opt(timestamp, 0).single()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_yahoo_quote_payload() {
        let payload = r#"{
            "quoteResponse": {
                "result": [{
                    "symbol": "AAPL",
                    "regularMarketPrice": 182.31,
                    "regularMarketChange": 1.2,
                    "regularMarketChangePercent": 0.66,
                    "regularMarketTime": 1700000000,
                    "currency": "USD"
                }],
                "error": null
            }
        }"#;

        let quote = parse_quote("AAPL", payload).unwrap();

        assert_eq!(quote.symbol, "AAPL");
        assert_eq!(quote.price, 182.31);
        assert_eq!(quote.source, "community-yahoo");
        assert_eq!(quote.currency.as_deref(), Some("USD"));
    }
}
