# bull

A clutter-free stock quote CLI for checking the latest available free quote for US equities.

`bull` is zero-config for common US equities: pass a ticker or company name and it resolves the symbol, fetches a quote, and shows the quote freshness/source so provider latency is visible. It also keeps a local-only watchlist/positions file for quick portfolio checks.

## Usage

```sh
bull AAPL
bull apple
bull apple msft tesla
bull --json apple
bull --watch 2 apple
bull
bull add apple
bull add apple msft
bull add apple --shares 10 --avg-cost 150
bull position AAPL
bull update AAPL --shares 12 --avg-cost 155
bull remove AAPL
bull remove AAPL MSFT
bull details AAPL
bull --json details apple
```

With no arguments, `bull` shows saved watchlist entries and positions. `bull add <query>` creates a watchlist-only entry unless `--shares` and `--avg-cost` are provided. Multiple watchlist entries can be added or removed with space-separated queries or symbols.

## Flags

- `--json` renders stable machine-readable JSON.
- `--watch <seconds>` refreshes the latest available quote. Values below `2` are clamped to `2`.
- `--provider <auto|community|twelvedata|alphavantage>` overrides provider selection.
- `--symbol` treats every query as a ticker.
- `--yes` auto-picks the top resolver match for ambiguous non-interactive use and writes a warning to stderr.
- `--no-color` disables ANSI color in table output.

## Portfolio Data

Portfolio data is local-only user data. `bull` does not connect to brokerage accounts, sync holdings, store transaction lots, or provide financial advice.

The portfolio file is stored with `directories::ProjectDirs` under the platform data directory as `portfolio.json`, for example under `$XDG_DATA_HOME/bull/portfolio.json` on Linux when `XDG_DATA_HOME` is set. Quote cache data remains separate in the cache directory.

Positions allow fractional shares. `--shares` must be greater than `0`; `--avg-cost` must be greater than or equal to `0`. Watchlist-only entries can omit both values, and their value and P/L fields render as unavailable rather than zero.

## Data Sources

- Company-name resolution uses SEC company ticker mappings, cached locally for about 24 hours.
- Default quote retrieval uses a no-key community quote endpoint (`community-stooq`) on a best-effort basis.
- `bull details <query>` uses `community-stooq` for OHLCV quote details when available.
- Optional key-based providers:
  - `BULL_TWELVEDATA_API_KEY`
  - `BULL_ALPHA_VANTAGE_API_KEY`

`bull` does not promise exchange-grade real-time data. It shows the latest available free quote returned by the configured provider and includes the source in table and JSON output.

Google Search and raw Google Finance page scraping are intentionally not data sources. There is no official public Google Finance API for this CLI flow, and scraping result pages is brittle.

## Cache

Quote responses are cached briefly, defaulting to 15 seconds. If a refresh fails and cached data exists, `bull` can return the stale cached quote with a stale marker.

Environment overrides:

- `BULL_PROVIDER`
- `BULL_CACHE_TTL_QUOTES`
- `BULL_CACHE_TTL_INDEX`
