use crate::error::{BullError, Result};
use crate::model::Quote;
use crate::provider::QuoteProvider;

#[derive(Debug, Default)]
pub struct CommunityProvider;

impl QuoteProvider for CommunityProvider {
    fn name(&self) -> &'static str {
        "community"
    }

    fn quote(&self, symbol: &str) -> Result<Quote> {
        Err(BullError::Provider {
            provider: self.name().to_string(),
            symbol: symbol.to_string(),
            message: "provider adapter is not implemented yet".to_string(),
        })
    }
}
