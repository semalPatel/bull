use crate::error::{BullError, Result};
use crate::model::Quote;
use crate::provider::QuoteProvider;
use chrono::{NaiveDateTime, TimeZone, Utc};
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
        "community-stooq"
    }

    fn quote(&self, symbol: &str) -> Result<Quote> {
        let stooq_symbol = format!("{}.us", symbol.to_ascii_lowercase());
        let url = format!(
            "https://stooq.com/q/l/?s={}&f=sd2t2ohlcv&h&e=csv",
            stooq_symbol
        );
        let response = self.client.get(url).send()?.error_for_status()?.text()?;
        parse_quote(symbol, &response)
    }
}

fn parse_quote(symbol: &str, payload: &str) -> Result<Quote> {
    let mut lines = payload.lines();
    let _header = lines.next();
    let row = lines.next().ok_or_else(|| BullError::Provider {
        provider: "community-stooq".to_string(),
        symbol: symbol.to_string(),
        message: "provider response did not include a quote row".to_string(),
    })?;
    let columns = row.split(',').collect::<Vec<_>>();
    if columns.len() < 8 || columns[3] == "N/D" || columns[6] == "N/D" {
        return Err(BullError::Provider {
            provider: "community-stooq".to_string(),
            symbol: symbol.to_string(),
            message: "provider response did not include a current quote".to_string(),
        });
    }

    let price = parse_number(symbol, columns[6], "close")?;
    let open = parse_number(symbol, columns[3], "open").ok();
    let change = open.map(|open| price - open);
    let change_percent = open.and_then(|open| {
        if open.abs() > f64::EPSILON {
            Some((price - open) / open * 100.0)
        } else {
            None
        }
    });

    Ok(Quote {
        symbol: symbol.to_ascii_uppercase(),
        price,
        change,
        change_percent,
        as_of: parse_timestamp(columns[1], columns[2]),
        currency: Some("USD".to_string()),
        source: "community-stooq".to_string(),
        stale: false,
    })
}

fn parse_number(symbol: &str, value: &str, field: &str) -> Result<f64> {
    value.parse::<f64>().map_err(|_| BullError::Provider {
        provider: "community-stooq".to_string(),
        symbol: symbol.to_string(),
        message: format!("provider response did not include numeric {field}"),
    })
}

fn parse_timestamp(date: &str, time: &str) -> Option<chrono::DateTime<Utc>> {
    NaiveDateTime::parse_from_str(&format!("{date} {time}"), "%Y-%m-%d %H:%M:%S")
        .ok()
        .map(|datetime| Utc.from_utc_datetime(&datetime))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_stooq_quote_payload() {
        let payload = "Symbol,Date,Time,Open,High,Low,Close,Volume\nAAPL.US,2026-04-06,22:00:15,256.51,262.16,256.46,258.86,29329911\n";

        let quote = parse_quote("AAPL", payload).unwrap();

        assert_eq!(quote.symbol, "AAPL");
        assert_eq!(quote.price, 258.86);
        assert_eq!(quote.source, "community-stooq");
        assert_eq!(quote.currency.as_deref(), Some("USD"));
        assert!(quote.change.unwrap() > 0.0);
    }
}
