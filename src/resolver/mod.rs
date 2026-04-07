use crate::error::{BullError, Result};
use crate::model::{Resolution, ResolutionStrategy};

pub mod matcher;
pub mod sec_index;

#[derive(Debug, Clone)]
pub struct Resolver;

impl Resolver {
    pub fn new() -> Self {
        Self
    }

    pub fn resolve(&self, query: &str, force_symbol: bool, _yes: bool) -> Result<Resolution> {
        let normalized = query.trim();
        if normalized.is_empty() {
            return Err(BullError::NoQueries);
        }

        if force_symbol || matcher::is_ticker(normalized) {
            return Ok(Resolution {
                query: query.to_string(),
                symbol: normalized.to_uppercase(),
                company_name: None,
                confidence: 1.0,
                strategy: if force_symbol {
                    ResolutionStrategy::ForcedSymbol
                } else {
                    ResolutionStrategy::TickerPattern
                },
            });
        }

        Err(BullError::NoMatch {
            query: query.to_string(),
            suggestions: Vec::new(),
        })
    }
}

impl Default for Resolver {
    fn default() -> Self {
        Self::new()
    }
}
