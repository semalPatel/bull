use crate::error::Result;
use crate::model::QuoteResult;
use console::Style;
use prettytable::{Cell, Row, Table};

pub fn render_json(results: &[QuoteResult]) -> Result<String> {
    Ok(serde_json::to_string_pretty(results)?)
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

fn row(values: &[&str]) -> Row {
    Row::new(values.iter().map(|value| Cell::new(value)).collect())
}

fn format_price(price: f64, currency: Option<&str>) -> String {
    match currency {
        Some("USD") | None => format!("${price:.2}"),
        Some(currency) => format!("{price:.2} {currency}"),
    }
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
