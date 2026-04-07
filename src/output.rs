use crate::error::Result;
use crate::model::{QuoteDetails, QuoteResult};
use crate::portfolio::PositionView;
use console::Style;
use prettytable::{Cell, Row, Table};

pub fn render_json(results: &[QuoteResult]) -> Result<String> {
    Ok(serde_json::to_string_pretty(results)?)
}

pub fn render_positions_json(results: &[PositionView]) -> Result<String> {
    Ok(serde_json::to_string_pretty(results)?)
}

pub fn render_position_json(result: &PositionView) -> Result<String> {
    Ok(serde_json::to_string_pretty(result)?)
}

pub fn render_details_json(details: &QuoteDetails) -> Result<String> {
    Ok(serde_json::to_string_pretty(details)?)
}

pub fn render_table(results: &[QuoteResult], no_color: bool) -> String {
    let mut table = Table::new();
    table.add_row(row(&[
        "Query",
        "Symbol",
        "Price",
        "Change",
        "% Change",
        "Timestamp",
        "Source",
    ]));

    for result in results {
        let quote = &result.quote;
        table.add_row(row(&[
            &result.query,
            &quote.symbol,
            &format_price(quote.price, quote.currency.as_deref()),
            &format_signed(quote.change),
            &format_percent(quote.change_percent),
            &quote
                .as_of
                .map(|timestamp| timestamp.to_rfc3339())
                .unwrap_or_else(|| "unknown".to_string()),
            &format_source(&quote.source, quote.stale),
        ]));
    }

    let mut rendered = table.to_string();
    if !no_color {
        rendered = colorize_numeric_cells(rendered);
    }
    rendered
}

pub fn render_positions_table(results: &[PositionView], no_color: bool) -> String {
    if results.is_empty() {
        return "No saved positions or watchlist entries. Add one with: bull add AAPL".to_string();
    }

    let mut table = Table::new();
    table.add_row(row(&[
        "Symbol",
        "Company",
        "Shares",
        "Price",
        "Value",
        "Day P/L",
        "Total P/L",
        "% P/L",
        "Source",
        "Timestamp",
    ]));

    for result in results {
        table.add_row(row(&[
            &result.position.symbol,
            result.position.company_name.as_deref().unwrap_or("-"),
            &format_optional_number(result.position.shares),
            &format_price(result.quote.price, result.quote.currency.as_deref()),
            &format_optional_money(result.market_value, result.quote.currency.as_deref()),
            &format_optional_signed_money(result.day_pl, result.quote.currency.as_deref()),
            &format_optional_signed_money(result.unrealized_pl, result.quote.currency.as_deref()),
            &format_percent_dash(result.unrealized_pl_percent),
            &format_source(&result.quote.source, result.quote.stale),
            &format_timestamp(result.quote.as_of),
        ]));
    }

    let mut rendered = table.to_string();
    if !no_color {
        rendered = colorize_numeric_cells(rendered);
    }
    rendered
}

pub fn render_position_detail(result: &PositionView, no_color: bool) -> String {
    let mut lines = vec![
        format!(
            "{} - {}",
            result.position.symbol,
            result.position.company_name.as_deref().unwrap_or("-")
        ),
        format!(
            "Price: {}",
            format_price(result.quote.price, result.quote.currency.as_deref())
        ),
        format!("Shares: {}", format_optional_number(result.position.shares)),
        format!(
            "Avg cost: {}",
            format_optional_money(result.position.avg_cost, result.quote.currency.as_deref())
        ),
        format!(
            "Market value: {}",
            format_optional_money(result.market_value, result.quote.currency.as_deref())
        ),
        format!(
            "Cost basis: {}",
            format_optional_money(result.cost_basis, result.quote.currency.as_deref())
        ),
        format!(
            "Day P/L: {}",
            format_optional_signed_money(result.day_pl, result.quote.currency.as_deref())
        ),
        format!(
            "Unrealized P/L: {} ({})",
            format_optional_signed_money(result.unrealized_pl, result.quote.currency.as_deref()),
            format_percent_dash(result.unrealized_pl_percent)
        ),
        format!("As of: {}", format_timestamp(result.quote.as_of)),
        format!(
            "Source: {}",
            format_source(&result.quote.source, result.quote.stale)
        ),
    ];

    let rendered = lines.join("\n");
    if no_color {
        rendered
    } else {
        lines = colorize_numeric_cells(rendered)
            .lines()
            .map(str::to_string)
            .collect();
        lines.join("\n")
    }
}

pub fn render_details_table(details: &QuoteDetails, no_color: bool) -> String {
    let mut table = Table::new();
    table.add_row(row(&[
        "Symbol",
        "Price",
        "Open",
        "High",
        "Low",
        "Close",
        "Volume",
        "Timestamp",
        "Source",
    ]));
    table.add_row(row(&[
        &details.symbol,
        &format_price(details.price, details.currency.as_deref()),
        &format_optional_money(details.open, details.currency.as_deref()),
        &format_optional_money(details.high, details.currency.as_deref()),
        &format_optional_money(details.low, details.currency.as_deref()),
        &format_optional_money(details.close, details.currency.as_deref()),
        &details
            .volume
            .map(|volume| volume.to_string())
            .unwrap_or_else(|| "-".to_string()),
        &format_timestamp(details.as_of),
        &format_source(&details.source, details.stale),
    ]));

    let mut rendered = table.to_string();
    if !no_color {
        rendered = colorize_numeric_cells(rendered);
    }
    rendered
}

fn row(values: &[&str]) -> Row {
    Row::new(values.iter().map(|value| Cell::new(value)).collect())
}

fn format_price(price: f64, currency: Option<&str>) -> String {
    match currency {
        Some("USD") | None => format!("${price:.2}"),
        Some(currency) => format!("{price:.2} {currency}"),
    }
}

fn format_optional_money(value: Option<f64>, currency: Option<&str>) -> String {
    value
        .map(|value| format_price(value, currency))
        .unwrap_or_else(|| "-".to_string())
}

fn format_optional_signed_money(value: Option<f64>, currency: Option<&str>) -> String {
    value
        .map(|value| match currency {
            Some("USD") | None => format!("{value:+.2}").replace('+', "+$").replace('-', "-$"),
            Some(currency) => format!("{value:+.2} {currency}"),
        })
        .unwrap_or_else(|| "-".to_string())
}

fn format_optional_number(value: Option<f64>) -> String {
    value
        .map(|value| {
            format!("{value:.4}")
                .trim_end_matches('0')
                .trim_end_matches('.')
                .to_string()
        })
        .unwrap_or_else(|| "-".to_string())
}

fn format_signed(value: Option<f64>) -> String {
    value
        .map(|value| format!("{value:+.2}"))
        .unwrap_or_else(|| "unknown".to_string())
}

fn format_percent(value: Option<f64>) -> String {
    value
        .map(|value| format!("{value:+.2}%"))
        .unwrap_or_else(|| "unknown".to_string())
}

fn format_percent_dash(value: Option<f64>) -> String {
    value
        .map(|value| format!("{value:+.2}%"))
        .unwrap_or_else(|| "-".to_string())
}

fn format_timestamp(value: Option<chrono::DateTime<chrono::Utc>>) -> String {
    value
        .map(|timestamp| timestamp.to_rfc3339())
        .unwrap_or_else(|| "-".to_string())
}

fn format_source(source: &str, stale: bool) -> String {
    if stale {
        format!("{source} (stale cache)")
    } else {
        source.to_string()
    }
}

fn colorize_numeric_cells(rendered: String) -> String {
    let green = Style::new().green();
    let red = Style::new().red();
    rendered
        .lines()
        .map(|line| {
            if line.contains("| -") {
                red.apply_to(line).to_string()
            } else if line.contains("| +") {
                green.apply_to(line).to_string()
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Quote, Resolution, ResolutionStrategy};

    #[test]
    fn renders_default_table_contract() {
        let output = render_table(
            &[QuoteResult {
                query: "apple".to_string(),
                resolution: Resolution {
                    query: "apple".to_string(),
                    symbol: "AAPL".to_string(),
                    company_name: Some("Apple Inc.".to_string()),
                    confidence: 1.0,
                    strategy: ResolutionStrategy::ExactName,
                },
                quote: Quote {
                    symbol: "AAPL".to_string(),
                    price: 182.31,
                    change: Some(1.2),
                    change_percent: Some(0.66),
                    as_of: None,
                    currency: Some("USD".to_string()),
                    source: "fixture".to_string(),
                    stale: false,
                },
            }],
            true,
        );

        assert!(output.contains("Query"));
        assert!(output.contains("AAPL"));
        assert!(output.contains("$182.31"));
        assert!(output.contains("fixture"));
    }
}
