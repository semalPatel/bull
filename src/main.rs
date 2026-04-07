mod cache;
mod cli;
mod error;
mod model;
mod output;
mod portfolio;
mod provider;
mod resolver;

use cache::QuoteCache;
use clap::Parser;
use cli::{Cli, Command};
use error::{BullError, Result};
use model::{Quote, QuoteResult, Resolution};
use portfolio::{Portfolio, PortfolioStore, Position, PositionView};
use provider::ProviderPolicy;
use std::thread;

fn main() {
    if let Err(error) = run() {
        eprintln!("bull: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let args = Cli::parse();
    let provider_policy = ProviderPolicy::from_env(args.provider);

    if args.watch_was_clamped() {
        eprintln!("bull: watch interval below 2s; clamping to 2s");
    }

    match &args.command {
        Some(command) => return run_command(command, &args, &provider_policy),
        None if args.queries.is_empty() => return run_positions(&args, &provider_policy),
        None => {}
    }

    let mut resolver = resolver::Resolver::new();
    let resolutions = resolve_queries(&args, &mut resolver)?;
    let mut quote_cache = QuoteCache::load()?;

    if let Some(interval) = args.watch_interval() {
        loop {
            render_once(&args, &resolutions, &provider_policy, &mut quote_cache)?;
            thread::sleep(interval);
        }
    }

    render_once(&args, &resolutions, &provider_policy, &mut quote_cache)
}

fn run_command(command: &Command, args: &Cli, provider_policy: &ProviderPolicy) -> Result<()> {
    if args.watch.is_some() {
        return Err(BullError::Portfolio(
            "--watch is only supported for quote queries".to_string(),
        ));
    }

    match command {
        Command::Positions => run_positions(args, provider_policy),
        Command::Position { query } => run_single_position(args, provider_policy, query),
        Command::Add {
            queries,
            shares,
            avg_cost,
        } => run_add(args, queries, *shares, *avg_cost),
        Command::Update {
            symbol,
            shares,
            avg_cost,
        } => run_update(symbol, *shares, *avg_cost),
        Command::Remove { symbols } => run_remove(symbols),
        Command::Details { query } => run_details(args, provider_policy, query),
    }
}

fn resolve_queries(args: &Cli, resolver: &mut resolver::Resolver) -> Result<Vec<Resolution>> {
    args.queries
        .iter()
        .map(|query| resolver.resolve(query, args.symbol, args.yes))
        .collect()
}

fn render_once(
    args: &Cli,
    resolutions: &[Resolution],
    provider_policy: &ProviderPolicy,
    quote_cache: &mut QuoteCache,
) -> Result<()> {
    let mut results = Vec::with_capacity(resolutions.len());
    let mut failures = 0;

    for resolution in resolutions {
        match quote_cache.quote(provider_policy, &resolution.symbol) {
            Ok(quote) => results.push(QuoteResult {
                query: resolution.query.clone(),
                resolution: resolution.clone(),
                quote,
            }),
            Err(error) => {
                failures += 1;
                eprintln!("bull: {}: {error}", resolution.query);
            }
        }
    }

    if args.json {
        println!("{}", output::render_json(&results)?);
    } else {
        println!("{}", output::render_table(&results, args.no_color));
    }

    if failures > 0 {
        Err(BullError::ProviderUnavailable {
            symbol: format!("{failures} quote request(s) failed"),
        })
    } else {
        Ok(())
    }
}

fn run_add(
    args: &Cli,
    queries: &[String],
    shares: Option<f64>,
    avg_cost: Option<f64>,
) -> Result<()> {
    if queries.len() > 1 && (shares.is_some() || avg_cost.is_some()) {
        return Err(BullError::Portfolio(
            "--shares and --avg-cost can only be used when adding one position".to_string(),
        ));
    }

    let mut resolver = resolver::Resolver::new();
    let (store, mut portfolio) = PortfolioStore::load_default()?;

    let mut added = Vec::with_capacity(queries.len());
    for query in queries {
        let resolution = resolver.resolve(query, args.symbol, args.yes)?;
        let position = Position::new(
            resolution.symbol,
            resolution.company_name,
            Some(resolution.query),
            shares,
            avg_cost,
        );
        portfolio.add(position.clone())?;
        added.push(position);
    }

    store.save(&portfolio)?;

    if args.json {
        if added.len() == 1 {
            println!("{}", serde_json::to_string_pretty(&added[0])?);
        } else {
            println!("{}", serde_json::to_string_pretty(&added)?);
        }
    } else if added.len() == 1 && added[0].shares.is_some() {
        println!("Added position {}", added[0].symbol);
    } else {
        println!(
            "Added watchlist entries {}",
            added
                .iter()
                .map(|position| position.symbol.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        );
    }
    Ok(())
}

fn run_update(symbol: &str, shares: Option<f64>, avg_cost: Option<f64>) -> Result<()> {
    let (store, mut portfolio) = PortfolioStore::load_default()?;
    portfolio.update(symbol, shares, avg_cost)?;
    store.save(&portfolio)?;
    println!("Updated {}", symbol.to_ascii_uppercase());
    Ok(())
}

fn run_remove(symbols: &[String]) -> Result<()> {
    let (store, mut portfolio) = PortfolioStore::load_default()?;
    let mut removed = Vec::with_capacity(symbols.len());
    for symbol in symbols {
        removed.push(portfolio.remove(symbol)?);
    }
    store.save(&portfolio)?;
    println!(
        "Removed {}",
        removed
            .iter()
            .map(|position| position.symbol.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    );
    Ok(())
}

fn run_positions(args: &Cli, provider_policy: &ProviderPolicy) -> Result<()> {
    let (_store, portfolio) = PortfolioStore::load_default()?;
    if portfolio.positions.is_empty() {
        if args.json {
            println!("[]");
        } else {
            println!("{}", output::render_positions_table(&[], args.no_color));
        }
        return Ok(());
    }

    let (views, failures) = position_views(&portfolio, provider_policy)?;
    if args.json {
        println!("{}", output::render_positions_json(&views)?);
    } else {
        println!("{}", output::render_positions_table(&views, args.no_color));
    }

    if failures > 0 {
        Err(BullError::ProviderUnavailable {
            symbol: format!("{failures} portfolio quote request(s) failed"),
        })
    } else {
        Ok(())
    }
}

fn run_single_position(args: &Cli, provider_policy: &ProviderPolicy, query: &str) -> Result<()> {
    let mut resolver = resolver::Resolver::new();
    let resolution = resolver.resolve(query, args.symbol, args.yes)?;
    let (_store, portfolio) = PortfolioStore::load_default()?;
    let position = portfolio
        .find(&resolution.symbol)
        .cloned()
        .ok_or_else(|| BullError::Portfolio(format!("{} is not saved", resolution.symbol)))?;
    let mut quote_cache = QuoteCache::load()?;
    let quote = quote_cache.quote(provider_policy, &position.symbol)?;
    let view = portfolio::view_for_position(
        position,
        quote.clone(),
        quote.price
            * portfolio
                .find(&resolution.symbol)
                .and_then(|p| p.shares)
                .unwrap_or(0.0),
    );

    if args.json {
        println!("{}", output::render_position_json(&view)?);
    } else {
        println!("{}", output::render_position_detail(&view, args.no_color));
    }
    Ok(())
}

fn position_views(
    portfolio: &Portfolio,
    provider_policy: &ProviderPolicy,
) -> Result<(Vec<PositionView>, usize)> {
    let mut quote_cache = QuoteCache::load()?;
    let mut items: Vec<(Position, Quote)> = Vec::new();
    let mut failures = 0;

    for position in &portfolio.positions {
        match quote_cache.quote(provider_policy, &position.symbol) {
            Ok(quote) => items.push((position.clone(), quote)),
            Err(error) => {
                failures += 1;
                eprintln!("bull: {}: {error}", position.symbol);
            }
        }
    }

    Ok((portfolio::position_views(items), failures))
}

fn run_details(args: &Cli, provider_policy: &ProviderPolicy, query: &str) -> Result<()> {
    let mut resolver = resolver::Resolver::new();
    let resolution = resolver.resolve(query, args.symbol, args.yes)?;
    let details = provider_policy.quote_details(&resolution.symbol)?;

    if args.json {
        println!("{}", output::render_details_json(&details)?);
    } else {
        println!("{}", output::render_details_table(&details, args.no_color));
    }
    Ok(())
}
