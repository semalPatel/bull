use crate::cache::DEFAULT_INDEX_CACHE_TTL;
use crate::error::{BullError, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;

const SEC_COMPANY_TICKERS_URL: &str = "https://www.sec.gov/files/company_tickers.json";
const SEC_USER_AGENT: &str = "bull-cli/0.1 (contact: semalpatel2596@gmail.com)";

#[derive(Debug, Clone)]
pub struct SecIndex {
    companies: Vec<Company>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Company {
    pub symbol: String,
    pub company_name: String,
    pub cik: u64,
    pub exchange: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SecCompany {
    cik_str: u64,
    ticker: String,
    title: String,
}

impl SecIndex {
    pub fn load() -> Self {
        Self::load_result().unwrap_or_else(|_| Self::seed())
    }

    pub fn from_companies(companies: Vec<Company>) -> Self {
        Self { companies }
    }

    pub fn companies(&self) -> &[Company] {
        &self.companies
    }

    fn load_result() -> Result<Self> {
        let cache_path = cache_path()?;
        if cache_is_fresh(&cache_path) {
            return Self::from_cache(&cache_path);
        }

        match fetch_sec_companies() {
            Ok(companies) => {
                if let Some(parent) = cache_path.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::write(&cache_path, serde_json::to_vec_pretty(&companies)?)?;
                Ok(Self { companies })
            }
            Err(error) if cache_path.exists() => {
                let _ = error;
                Self::from_cache(&cache_path)
            }
            Err(error) => Err(error),
        }
    }

    fn from_cache(cache_path: &PathBuf) -> Result<Self> {
        let bytes = fs::read(cache_path)?;
        let companies = serde_json::from_slice(&bytes)?;
        Ok(Self { companies })
    }

    fn seed() -> Self {
        Self {
            companies: vec![
                company("AAPL", "Apple Inc.", 320193),
                company("MSFT", "Microsoft Corp.", 789019),
                company("TSLA", "Tesla, Inc.", 1318605),
                company("META", "Meta Platforms, Inc.", 1326801),
                company("AMZN", "Amazon.com, Inc.", 1018724),
                company("NVDA", "NVIDIA Corp.", 1045810),
                company("GOOG", "Alphabet Inc.", 1652044),
                company("GOOGL", "Alphabet Inc.", 1652044),
            ],
        }
    }
}

fn company(symbol: &str, company_name: &str, cik: u64) -> Company {
    Company {
        symbol: symbol.to_string(),
        company_name: company_name.to_string(),
        cik,
        exchange: None,
    }
}

fn fetch_sec_companies() -> Result<Vec<Company>> {
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(8))
        .build()?;
    let response = client
        .get(SEC_COMPANY_TICKERS_URL)
        .header(reqwest::header::USER_AGENT, SEC_USER_AGENT)
        .send()?
        .error_for_status()?;
    parse_company_tickers(&response.text()?)
}

fn parse_company_tickers(payload: &str) -> Result<Vec<Company>> {
    let parsed: BTreeMap<String, SecCompany> = serde_json::from_str(payload)?;
    let mut companies = parsed
        .into_values()
        .map(|entry| Company {
            symbol: entry.ticker,
            company_name: entry.title,
            cik: entry.cik_str,
            exchange: None,
        })
        .collect::<Vec<_>>();
    companies.sort_by(|left, right| left.symbol.cmp(&right.symbol));
    Ok(companies)
}

fn cache_path() -> Result<PathBuf> {
    let dirs = ProjectDirs::from("dev", "bull", "bull")
        .ok_or_else(|| BullError::Cache("could not determine cache directory".to_string()))?;
    Ok(dirs.cache_dir().join("sec_company_tickers.json"))
}

fn cache_is_fresh(cache_path: &PathBuf) -> bool {
    let Ok(metadata) = fs::metadata(cache_path) else {
        return false;
    };
    let Ok(modified) = metadata.modified() else {
        return false;
    };
    let Ok(age) = SystemTime::now().duration_since(modified) else {
        return false;
    };
    age <= DEFAULT_INDEX_CACHE_TTL
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_sec_company_tickers_payload() {
        let payload = r#"{
            "0": {"cik_str": 320193, "ticker": "AAPL", "title": "Apple Inc."},
            "1": {"cik_str": 789019, "ticker": "MSFT", "title": "Microsoft Corp."}
        }"#;

        let companies = parse_company_tickers(payload).unwrap();

        assert_eq!(companies.len(), 2);
        assert_eq!(companies[0].symbol, "AAPL");
        assert_eq!(companies[0].company_name, "Apple Inc.");
    }
}
