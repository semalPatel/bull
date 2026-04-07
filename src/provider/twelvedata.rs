use crate::error::{BullError, Result};
use crate::model::Quote;
use crate::provider::QuoteProvider;

#[derive(Debug, Clone)]
pub struct TwelveDataProvider {
    api_key: String,
}

impl TwelveDataProvider {
    pub fn new(api_key: String) -> Self {
        Self { api_key }
    }
}

impl QuoteProvider for TwelveDataProvider {
    fn name(&self) -> &'static str {
        "twelvedata"
    }

    fn quote(&self, symbol: &str) -> Result<Quote> {
        let _ = &self.api_key;
        Err(BullError::Provider {
            provider: self.name().to_string(),
            symbol: symbol.to_string(),
            message: "provider adapter is not implemented yet".to_string(),
        })
    }
}
