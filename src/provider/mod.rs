use crate::error::Result;
use crate::model::Quote;

pub mod alphavantage;
pub mod community;
pub mod twelvedata;

pub trait QuoteProvider {
    fn name(&self) -> &'static str;
    fn quote(&self, symbol: &str) -> Result<Quote>;
}
