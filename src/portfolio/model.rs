use crate::model::Quote;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Portfolio {
    pub schema_version: u32,
    pub positions: Vec<Position>,
}

impl Default for Portfolio {
    fn default() -> Self {
        Self {
            schema_version: 1,
            positions: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Position {
    pub symbol: String,
    pub company_name: Option<String>,
    pub original_query: Option<String>,
    pub shares: Option<f64>,
    pub avg_cost: Option<f64>,
    pub currency: String,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Position {
    pub fn new(
        symbol: String,
        company_name: Option<String>,
        original_query: Option<String>,
        shares: Option<f64>,
        avg_cost: Option<f64>,
    ) -> Self {
        let now = Utc::now();
        Self {
            symbol: symbol.to_ascii_uppercase(),
            company_name,
            original_query,
            shares,
            avg_cost,
            currency: "USD".to_string(),
            notes: None,
            created_at: now,
            updated_at: now,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PositionView {
    pub position: Position,
    pub quote: Quote,
    pub market_value: Option<f64>,
    pub cost_basis: Option<f64>,
    pub day_pl: Option<f64>,
    pub unrealized_pl: Option<f64>,
    pub unrealized_pl_percent: Option<f64>,
    pub allocation_percent: Option<f64>,
}
