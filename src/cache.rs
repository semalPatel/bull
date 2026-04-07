use crate::error::{BullError, Result};
use crate::model::Quote;
use crate::provider::ProviderPolicy;
use chrono::{DateTime, Utc};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::time::Duration;

pub const DEFAULT_QUOTE_CACHE_TTL: Duration = Duration::from_secs(15);
pub const DEFAULT_INDEX_CACHE_TTL: Duration = Duration::from_secs(24 * 60 * 60);

#[derive(Debug, Clone)]
pub struct QuoteCache {
    path: PathBuf,
    ttl: Duration,
    entries: BTreeMap<String, CachedQuote>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CachedQuote {
    quote: Quote,
    cached_at: DateTime<Utc>,
}

impl QuoteCache {
    pub fn load() -> Result<Self> {
        let path = quote_cache_path()?;
        let ttl = std::env::var("BULL_CACHE_TTL_QUOTES")
            .ok()
            .and_then(|value| value.parse::<u64>().ok())
            .map(Duration::from_secs)
            .unwrap_or(DEFAULT_QUOTE_CACHE_TTL);
        let entries = fs::read(&path)
            .ok()
            .and_then(|bytes| serde_json::from_slice(&bytes).ok())
            .unwrap_or_default();
        Ok(Self { path, ttl, entries })
    }

    pub fn quote(&mut self, policy: &ProviderPolicy, symbol: &str) -> Result<Quote> {
        let key = symbol.to_ascii_uppercase();
        if let Some(cached) = self.entries.get(&key) {
            if is_fresh(cached.cached_at, self.ttl) {
                let mut quote = cached.quote.clone();
                quote.stale = false;
                return Ok(quote);
            }
        }

        match policy.quote(symbol) {
            Ok(quote) => {
                self.entries.insert(
                    key,
                    CachedQuote {
                        quote: quote.clone(),
                        cached_at: Utc::now(),
                    },
                );
                self.persist()?;
                Ok(quote)
            }
            Err(error) => {
                if let Some(cached) = self.entries.get(&key) {
                    let mut quote = cached.quote.clone();
                    quote.stale = true;
                    return Ok(quote);
                }
                Err(error)
            }
        }
    }

    fn persist(&self) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&self.path, serde_json::to_vec_pretty(&self.entries)?)?;
        Ok(())
    }
}

fn quote_cache_path() -> Result<PathBuf> {
    let dirs = ProjectDirs::from("dev", "bull", "bull")
        .ok_or_else(|| BullError::Cache("could not determine cache directory".to_string()))?;
    Ok(dirs.cache_dir().join("quotes.json"))
}

fn is_fresh(cached_at: DateTime<Utc>, ttl: Duration) -> bool {
    let Ok(age) = Utc::now().signed_duration_since(cached_at).to_std() else {
        return false;
    };
    age <= ttl
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration as ChronoDuration;

    #[test]
    fn detects_fresh_cache_entries() {
        assert!(is_fresh(Utc::now(), Duration::from_secs(15)));
        assert!(!is_fresh(
            Utc::now() - ChronoDuration::seconds(30),
            Duration::from_secs(15)
        ));
    }
}
