use crate::cache::index_cache_ttl;
use crate::error::{BullError, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;

const SEC_COMPANY_TICKERS_URL: &str = "https://www.sec.gov/files/company_tickers.json";
const DEFAULT_SEC_CONTACT: &str = "9878547+semalPatel@users.noreply.github.com";

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

    #[cfg(test)]
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
        .header(reqwest::header::USER_AGENT, sec_user_agent())
        .send()?
        .error_for_status()?;
    parse_company_tickers(&response.text()?)
}

fn sec_user_agent() -> String {
    let contact = std::env::var("BULL_SEC_CONTACT")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| DEFAULT_SEC_CONTACT.to_string());
    format!("bull-cli/0.1 (contact: {contact})")
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
    age <= index_cache_ttl()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, OnceLock};

    fn env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

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

    #[test]
    fn default_sec_user_agent_uses_project_contact() {
        let _guard = env_lock().lock().unwrap();
        std::env::remove_var("BULL_SEC_CONTACT");

        assert_eq!(
            sec_user_agent(),
            "bull-cli/0.1 (contact: 9878547+semalPatel@users.noreply.github.com)"
        );
    }

    #[test]
    fn sec_user_agent_uses_contact_override_when_present() {
        let _guard = env_lock().lock().unwrap();
        std::env::set_var("BULL_SEC_CONTACT", "oss@example.com");

        assert_eq!(sec_user_agent(), "bull-cli/0.1 (contact: oss@example.com)");

        std::env::remove_var("BULL_SEC_CONTACT");
    }
}
