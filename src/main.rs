mod cache;
mod cli;
mod error;
mod model;
mod output;
mod provider;
mod resolver;

use clap::Parser;
use cli::Cli;
use error::{BullError, Result};

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

    let resolver = resolver::Resolver::new();
    for query in &args.queries {
        let resolution = resolver.resolve(query, args.symbol, args.yes)?;
        println!(
            "{} -> {} ({:.2}, {:?})",
            resolution.query, resolution.symbol, resolution.confidence, resolution.strategy
        );
    }

    Ok(())
}
