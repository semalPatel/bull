use crate::error::{BullError, Result};
use crate::model::{Quote, QuoteDetails};
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
        let url = stooq_url(symbol);
        let response = self.client.get(url).send()?.error_for_status()?.text()?;
        parse_quote(symbol, &response)
    }

    fn quote_details(&self, symbol: &str) -> Result<QuoteDetails> {
        let url = stooq_url(symbol);
        let response = self.client.get(url).send()?.error_for_status()?.text()?;
        parse_quote_details(symbol, &response)
    }
}

fn parse_quote(symbol: &str, payload: &str) -> Result<Quote> {
    let row = parse_stooq_row(symbol, payload)?;
    let price = row.close.ok_or_else(|| BullError::Provider {
        provider: "community-stooq".to_string(),
        symbol: symbol.to_string(),
        message: "provider response did not include a current quote".to_string(),
    })?;
    let change = row.open.map(|open| price - open);
    let change_percent = row.open.and_then(|open| {
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
        as_of: row.as_of,
        currency: Some("USD".to_string()),
        source: "community-stooq".to_string(),
        stale: false,
    })
}

fn parse_quote_details(symbol: &str, payload: &str) -> Result<QuoteDetails> {
    let row = parse_stooq_row(symbol, payload)?;
    let price = row.close.ok_or_else(|| BullError::Provider {
        provider: "community-stooq".to_string(),
        symbol: symbol.to_string(),
        message: "provider response did not include a current quote".to_string(),
    })?;

    Ok(QuoteDetails {
        symbol: symbol.to_ascii_uppercase(),
        price,
        open: row.open,
        high: row.high,
        low: row.low,
        close: row.close,
        volume: row.volume,
        as_of: row.as_of,
        currency: Some("USD".to_string()),
        source: "community-stooq".to_string(),
        stale: false,
    })
}

#[derive(Debug)]
struct StooqRow {
    open: Option<f64>,
    high: Option<f64>,
    low: Option<f64>,
    close: Option<f64>,
    volume: Option<u64>,
    as_of: Option<chrono::DateTime<Utc>>,
}

fn parse_stooq_row(symbol: &str, payload: &str) -> Result<StooqRow> {
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

    Ok(StooqRow {
        open: parse_optional_number(symbol, columns[3], "open")?,
        high: parse_optional_number(symbol, columns[4], "high")?,
        low: parse_optional_number(symbol, columns[5], "low")?,
        close: parse_optional_number(symbol, columns[6], "close")?,
        volume: parse_optional_u64(symbol, columns[7], "volume")?,
        as_of: parse_timestamp(columns[1], columns[2]),
    })
}

fn stooq_url(symbol: &str) -> String {
    let stooq_symbol = format!("{}.us", symbol.to_ascii_lowercase());
    format!(
        "https://stooq.com/q/l/?s={}&f=sd2t2ohlcv&h&e=csv",
        stooq_symbol
    )
}

fn parse_optional_number(symbol: &str, value: &str, field: &str) -> Result<Option<f64>> {
    if value == "N/D" || value.trim().is_empty() {
        return Ok(None);
    }
    parse_number(symbol, value, field).map(Some)
}

fn parse_number(symbol: &str, value: &str, field: &str) -> Result<f64> {
    value.parse::<f64>().map_err(|_| BullError::Provider {
        provider: "community-stooq".to_string(),
        symbol: symbol.to_string(),
        message: format!("provider response did not include numeric {field}"),
    })
}

fn parse_optional_u64(symbol: &str, value: &str, field: &str) -> Result<Option<u64>> {
    if value == "N/D" || value.trim().is_empty() {
        return Ok(None);
    }
    value
        .parse::<u64>()
        .map(Some)
        .map_err(|_| BullError::Provider {
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

    #[test]
    fn parses_stooq_quote_details_payload() {
        let payload = "Symbol,Date,Time,Open,High,Low,Close,Volume\nAAPL.US,2026-04-06,22:00:15,256.51,262.16,256.46,258.86,29329911\n";

        let details = parse_quote_details("AAPL", payload).unwrap();

        assert_eq!(details.symbol, "AAPL");
        assert_eq!(details.price, 258.86);
        assert_eq!(details.open, Some(256.51));
        assert_eq!(details.high, Some(262.16));
        assert_eq!(details.low, Some(256.46));
        assert_eq!(details.close, Some(258.86));
        assert_eq!(details.volume, Some(29329911));
    }

    #[test]
    fn nd_stooq_details_payload_returns_provider_error() {
        let payload =
            "Symbol,Date,Time,Open,High,Low,Close,Volume\nAAPL.US,N/D,N/D,N/D,N/D,N/D,N/D,N/D\n";

        assert!(parse_quote_details("AAPL", payload).is_err());
    }
}
