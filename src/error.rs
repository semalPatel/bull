use crate::model::ResolutionCandidate;

#[derive(Debug, thiserror::Error)]
pub enum BullError {
    #[error("no queries provided")]
    NoQueries,
    #[error("unknown company or symbol: {query}")]
    NoMatch {
        query: String,
        suggestions: Vec<ResolutionCandidate>,
    },
    #[error("ambiguous query: {query}")]
    Ambiguous {
        query: String,
        candidates: Vec<ResolutionCandidate>,
    },
    #[error("interactive selection is required for {query}; rerun with --yes or --symbol")]
    InteractionRequired { query: String },
    #[error("quote provider {provider} failed for {symbol}: {message}")]
    Provider {
        provider: String,
        symbol: String,
        message: String,
    },
    #[error("no quote provider could return a quote for {symbol}")]
    ProviderUnavailable { symbol: String },
    #[error("cache error: {0}")]
    Cache(String),
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, BullError>;
