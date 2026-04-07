mod cache;
mod cli;
mod error;
mod model;
mod output;
mod provider;
mod resolver;

use clap::Parser;
use cache::QuoteCache;
use cli::Cli;
use error::{BullError, Result};
use model::{QuoteResult, Resolution};
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
    if args.queries.is_empty() {
        return Err(BullError::NoQueries);
    }

    let mut resolver = resolver::Resolver::new();
    let resolutions = resolve_queries(&args, &mut resolver)?;
    let provider_policy = ProviderPolicy::from_env(args.provider);
    let mut quote_cache = QuoteCache::load()?;

    if args.watch_was_clamped() {
        eprintln!("bull: watch interval below 2s; clamping to 2s");
    }

    if let Some(interval) = args.watch_interval() {
        loop {
            render_once(&args, &resolutions, &provider_policy, &mut quote_cache)?;
            thread::sleep(interval);
        }
    }

    render_once(&args, &resolutions, &provider_policy, &mut quote_cache)
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
