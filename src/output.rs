use crate::error::Result;
use crate::model::QuoteResult;

pub fn render_json(results: &[QuoteResult]) -> Result<String> {
    Ok(serde_json::to_string_pretty(results)?)
}
