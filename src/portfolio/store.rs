use crate::error::{BullError, Result};
use crate::portfolio::model::{Portfolio, Position};
use chrono::Utc;
use directories::ProjectDirs;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct PortfolioStore {
    path: PathBuf,
}

impl PortfolioStore {
    pub fn load_default() -> Result<(Self, Portfolio)> {
        let store = Self {
            path: portfolio_path()?,
        };
        let portfolio = store.load()?;
        Ok((store, portfolio))
    }

    pub fn load(&self) -> Result<Portfolio> {
        match fs::read(&self.path) {
            Ok(bytes) => Ok(serde_json::from_slice(&bytes)?),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(Portfolio::default()),
            Err(error) => Err(error.into()),
        }
    }

    pub fn save(&self, portfolio: &Portfolio) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }
        let tmp_path = self.path.with_extension("json.tmp");
        fs::write(&tmp_path, serde_json::to_vec_pretty(portfolio)?)?;
        fs::rename(tmp_path, &self.path)?;
        Ok(())
    }

    #[cfg(test)]
    pub fn at(path: PathBuf) -> Self {
        Self { path }
    }
}

impl Portfolio {
    pub fn add(&mut self, position: Position) -> Result<()> {
        validate_position(&position)?;
        if self.find(&position.symbol).is_some() {
            return Err(BullError::Portfolio(format!(
                "{} is already saved",
                position.symbol
            )));
        }
        self.positions.push(position);
        self.positions
            .sort_by(|left, right| left.symbol.cmp(&right.symbol));
        Ok(())
    }

    pub fn update(
        &mut self,
        symbol: &str,
        shares: Option<f64>,
        avg_cost: Option<f64>,
    ) -> Result<()> {
        validate_shares(shares)?;
        validate_avg_cost(avg_cost)?;
        let symbol = symbol.to_ascii_uppercase();
        let position = self.find_mut(&symbol).ok_or_else(|| {
            BullError::Portfolio(format!("{symbol} is not saved in the portfolio"))
        })?;
        position.shares = shares;
        position.avg_cost = avg_cost;
        position.updated_at = Utc::now();
        Ok(())
    }

    pub fn remove(&mut self, symbol: &str) -> Result<Position> {
        let symbol = symbol.to_ascii_uppercase();
        let Some(index) = self
            .positions
            .iter()
            .position(|position| position.symbol == symbol)
        else {
            return Err(BullError::Portfolio(format!(
                "{symbol} is not saved in the portfolio"
            )));
        };
        Ok(self.positions.remove(index))
    }

    pub fn find(&self, symbol: &str) -> Option<&Position> {
        let symbol = symbol.to_ascii_uppercase();
        self.positions
            .iter()
            .find(|position| position.symbol == symbol)
    }

    fn find_mut(&mut self, symbol: &str) -> Option<&mut Position> {
        let symbol = symbol.to_ascii_uppercase();
        self.positions
            .iter_mut()
            .find(|position| position.symbol == symbol)
    }
}

fn portfolio_path() -> Result<PathBuf> {
    let dirs = ProjectDirs::from("dev", "bull", "bull")
        .ok_or_else(|| BullError::Portfolio("could not determine data directory".to_string()))?;
    Ok(dirs.data_dir().join("portfolio.json"))
}

fn validate_position(position: &Position) -> Result<()> {
    validate_shares(position.shares)?;
    validate_avg_cost(position.avg_cost)?;
    if position.symbol.trim().is_empty() {
        return Err(BullError::Portfolio("symbol is required".to_string()));
    }
    Ok(())
}

fn validate_shares(shares: Option<f64>) -> Result<()> {
    if let Some(shares) = shares {
        if !shares.is_finite() || shares <= 0.0 {
            return Err(BullError::Portfolio(
                "shares must be greater than 0".to_string(),
            ));
        }
    }
    Ok(())
}

fn validate_avg_cost(avg_cost: Option<f64>) -> Result<()> {
    if let Some(avg_cost) = avg_cost {
        if !avg_cost.is_finite() || avg_cost < 0.0 {
            return Err(BullError::Portfolio(
                "average cost must be greater than or equal to 0".to_string(),
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn position(symbol: &str, shares: Option<f64>, avg_cost: Option<f64>) -> Position {
        Position::new(symbol.to_string(), None, None, shares, avg_cost)
    }

    #[test]
    fn missing_store_loads_empty_portfolio() {
        let store = PortfolioStore::at(
            std::env::temp_dir().join(format!("bull-missing-{}.json", std::process::id())),
        );

        assert_eq!(store.load().unwrap(), Portfolio::default());
    }

    #[test]
    fn persists_and_reloads_portfolio() {
        let path = std::env::temp_dir().join(format!("bull-store-{}.json", std::process::id()));
        let store = PortfolioStore::at(path);
        let mut portfolio = Portfolio::default();
        portfolio
            .add(position("aapl", Some(1.5), Some(100.0)))
            .unwrap();

        store.save(&portfolio).unwrap();
        let loaded = store.load().unwrap();

        assert_eq!(loaded.positions.len(), 1);
        assert_eq!(loaded.positions[0].symbol, "AAPL");
    }

    #[test]
    fn add_update_remove_validates_positions() {
        let mut portfolio = Portfolio::default();

        portfolio
            .add(position("AAPL", Some(1.0), Some(100.0)))
            .unwrap();
        assert!(portfolio.add(position("AAPL", None, None)).is_err());
        assert!(portfolio.add(position("MSFT", Some(0.0), None)).is_err());
        assert!(portfolio
            .add(position("MSFT", Some(1.0), Some(-1.0)))
            .is_err());

        portfolio.update("AAPL", Some(2.0), Some(120.0)).unwrap();
        assert_eq!(portfolio.find("aapl").unwrap().shares, Some(2.0));

        assert!(portfolio.remove("MSFT").is_err());
        assert_eq!(portfolio.remove("AAPL").unwrap().symbol, "AAPL");
    }
}
