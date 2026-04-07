use crate::model::Quote;
use crate::portfolio::model::{Position, PositionView};

pub fn position_views(items: Vec<(Position, Quote)>) -> Vec<PositionView> {
    let total_market_value = items
        .iter()
        .filter_map(|(position, quote)| market_value(position, quote))
        .sum::<f64>();

    items
        .into_iter()
        .map(|(position, quote)| view_for_position(position, quote, total_market_value))
        .collect()
}

pub fn view_for_position(
    position: Position,
    quote: Quote,
    total_market_value: f64,
) -> PositionView {
    let market_value = market_value(&position, &quote);
    let cost_basis = cost_basis(&position);
    let day_pl = position
        .shares
        .and_then(|shares| quote.change.map(|change| shares * change));
    let unrealized_pl = market_value
        .zip(cost_basis)
        .map(|(value, cost)| value - cost);
    let unrealized_pl_percent = unrealized_pl.zip(cost_basis).and_then(|(pl, cost)| {
        if cost.abs() > f64::EPSILON {
            Some(pl / cost * 100.0)
        } else {
            None
        }
    });
    let allocation_percent = market_value.and_then(|value| {
        if total_market_value.abs() > f64::EPSILON {
            Some(value / total_market_value * 100.0)
        } else {
            None
        }
    });

    PositionView {
        position,
        quote,
        market_value,
        cost_basis,
        day_pl,
        unrealized_pl,
        unrealized_pl_percent,
        allocation_percent,
    }
}

fn market_value(position: &Position, quote: &Quote) -> Option<f64> {
    position.shares.map(|shares| shares * quote.price)
}

fn cost_basis(position: &Position) -> Option<f64> {
    position
        .shares
        .zip(position.avg_cost)
        .map(|(shares, avg_cost)| shares * avg_cost)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::portfolio::model::Position;

    fn quote(change: Option<f64>) -> Quote {
        Quote {
            symbol: "AAPL".to_string(),
            price: 200.0,
            change,
            change_percent: None,
            as_of: None,
            currency: Some("USD".to_string()),
            source: "fixture".to_string(),
            stale: false,
        }
    }

    fn position(shares: Option<f64>, avg_cost: Option<f64>) -> Position {
        Position::new("AAPL".to_string(), None, None, shares, avg_cost)
    }

    #[test]
    fn watchlist_only_entry_has_no_derived_values() {
        let view = view_for_position(position(None, None), quote(Some(1.0)), 0.0);

        assert_eq!(view.market_value, None);
        assert_eq!(view.day_pl, None);
        assert_eq!(view.unrealized_pl, None);
    }

    #[test]
    fn shares_without_average_cost_has_market_value_only() {
        let view = view_for_position(position(Some(2.0), None), quote(Some(1.0)), 400.0);

        assert_eq!(view.market_value, Some(400.0));
        assert_eq!(view.cost_basis, None);
        assert_eq!(view.day_pl, Some(2.0));
        assert_eq!(view.unrealized_pl, None);
        assert_eq!(view.allocation_percent, Some(100.0));
    }

    #[test]
    fn full_position_calculates_gain() {
        let view = view_for_position(position(Some(2.0), Some(150.0)), quote(Some(1.0)), 400.0);

        assert_eq!(view.market_value, Some(400.0));
        assert_eq!(view.cost_basis, Some(300.0));
        assert_eq!(view.unrealized_pl, Some(100.0));
        assert!((view.unrealized_pl_percent.unwrap() - 33.333333).abs() < 0.0001);
    }

    #[test]
    fn missing_quote_change_has_no_day_pl() {
        let view = view_for_position(position(Some(2.0), Some(150.0)), quote(None), 400.0);

        assert_eq!(view.day_pl, None);
    }
}
