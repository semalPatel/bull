use crate::cli::ProviderChoice;
use crate::error::{BullError, Result};
use crate::model::Quote;

pub mod alphavantage;
pub mod community;
pub mod twelvedata;

pub trait QuoteProvider {
    fn name(&self) -> &'static str;
    fn quote(&self, symbol: &str) -> Result<Quote>;
}

pub struct ProviderPolicy {
    providers: Vec<Box<dyn QuoteProvider>>,
}

impl ProviderPolicy {
    pub fn from_env(choice: Option<ProviderChoice>) -> Self {
        let providers: Vec<Box<dyn QuoteProvider>> = match choice.unwrap_or(ProviderChoice::Auto) {
            ProviderChoice::Auto => {
                let mut providers: Vec<Box<dyn QuoteProvider>> =
                    vec![Box::new(community::CommunityProvider::new())];
                if let Ok(api_key) = std::env::var("BULL_TWELVEDATA_API_KEY") {
                    if !api_key.trim().is_empty() {
                        providers.push(Box::new(twelvedata::TwelveDataProvider::new(api_key)));
                    }
                }
                if let Ok(api_key) = std::env::var("BULL_ALPHA_VANTAGE_API_KEY") {
                    if !api_key.trim().is_empty() {
                        providers.push(Box::new(alphavantage::AlphaVantageProvider::new(api_key)));
                    }
                }
                providers
            }
            ProviderChoice::Community => vec![Box::new(community::CommunityProvider::new())],
            ProviderChoice::Twelvedata => std::env::var("BULL_TWELVEDATA_API_KEY")
                .ok()
                .filter(|api_key| !api_key.trim().is_empty())
                .map(|api_key| {
                    vec![Box::new(twelvedata::TwelveDataProvider::new(api_key))
                        as Box<dyn QuoteProvider>]
                })
                .unwrap_or_default(),
            ProviderChoice::Alphavantage => std::env::var("BULL_ALPHA_VANTAGE_API_KEY")
                .ok()
                .filter(|api_key| !api_key.trim().is_empty())
                .map(|api_key| {
                    vec![Box::new(alphavantage::AlphaVantageProvider::new(api_key))
                        as Box<dyn QuoteProvider>]
                })
                .unwrap_or_default(),
        };

        Self { providers }
    }

    pub fn quote(&self, symbol: &str) -> Result<Quote> {
        if self.providers.is_empty() {
            return Err(BullError::ProviderUnavailable {
                symbol: symbol.to_string(),
            });
        }

        let mut last_error = None;
        for provider in &self.providers {
            let provider_name = provider.name();
            match provider.quote(symbol) {
                Ok(quote) => return Ok(quote),
                Err(error) => {
                    let _ = provider_name;
                    last_error = Some(error);
                }
            }
        }

        Err(last_error.unwrap_or_else(|| BullError::ProviderUnavailable {
            symbol: symbol.to_string(),
        }))
    }
}
