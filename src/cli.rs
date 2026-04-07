use clap::{Parser, ValueEnum};
use std::time::Duration;

pub const WATCH_INTERVAL_FLOOR_SECONDS: u64 = 2;

#[derive(Debug, Clone, Parser)]
#[command(
    name = "bull",
    version,
    about = "Check the latest available free quote for US equities.",
    long_about = "Check the latest available free quote for US equities. Quote freshness and source are shown so provider latency is visible."
)]
pub struct Cli {
    #[arg(long, help = "Render stable machine-readable JSON output")]
    pub json: bool,

    #[arg(long, value_name = "SECONDS", num_args = 0..=1, default_missing_value = "5", help = "Refresh the latest available quote every N seconds")]
    pub watch: Option<u64>,

    #[arg(long, value_enum, help = "Override quote provider selection")]
    pub provider: Option<ProviderChoice>,

    #[arg(long, help = "Treat every query as a ticker symbol")]
    pub symbol: bool,

    #[arg(long, help = "Disable ANSI color in table output")]
    pub no_color: bool,

    #[arg(long, help = "For ambiguous non-interactive use, pick the top resolver candidate and warn on stderr")]
    pub yes: bool,

    #[arg(value_name = "QUERY", required = true, num_args = 1.., help = "Ticker symbol or company name")]
    pub queries: Vec<String>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, ValueEnum)]
pub enum ProviderChoice {
    Auto,
    Community,
    Twelvedata,
    Alphavantage,
}

impl Cli {
    pub fn watch_interval(&self) -> Option<Duration> {
        self.watch.map(|seconds| {
            let clamped = seconds.max(WATCH_INTERVAL_FLOOR_SECONDS);
            Duration::from_secs(clamped)
        })
    }

    pub fn watch_was_clamped(&self) -> bool {
        self.watch
            .map(|seconds| seconds < WATCH_INTERVAL_FLOOR_SECONDS)
            .unwrap_or(false)
    }
}
