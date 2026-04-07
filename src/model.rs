use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Quote {
    pub symbol: String,
    pub price: f64,
    pub change: Option<f64>,
    pub change_percent: Option<f64>,
    pub as_of: Option<DateTime<Utc>>,
    pub currency: Option<String>,
    pub source: String,
    pub stale: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Resolution {
    pub query: String,
    pub symbol: String,
    pub company_name: Option<String>,
    pub confidence: f64,
    pub strategy: ResolutionStrategy,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ResolutionStrategy {
    ForcedSymbol,
    TickerPattern,
    ExactName,
    ExactSymbol,
    Prefix,
    TokenFuzzy,
    Alias,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuoteResult {
    pub query: String,
    pub resolution: Resolution,
    pub quote: Quote,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResolutionCandidate {
    pub symbol: String,
    pub company_name: String,
    pub confidence: f64,
    pub strategy: ResolutionStrategy,
}
