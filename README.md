# bull

A clutter-free stock quote CLI for checking the latest available free quote for US equities.

`bull` is zero-config for common US equities: pass a ticker or company name and it resolves the symbol, fetches a quote, and shows the quote freshness/source so provider latency is visible.

## Usage

```sh
bull AAPL
bull apple
bull apple msft tesla
bull --json apple
bull --watch 2 apple
```

## Flags

- `--json` renders stable machine-readable JSON.
- `--watch <seconds>` refreshes the latest available quote. Values below `2` are clamped to `2`.
- `--provider <auto|community|twelvedata|alphavantage>` overrides provider selection.
- `--symbol` treats every query as a ticker.
- `--yes` auto-picks the top resolver match for ambiguous non-interactive use and writes a warning to stderr.
- `--no-color` disables ANSI color in table output.

## Data Sources

- Company-name resolution uses SEC company ticker mappings, cached locally for about 24 hours.
- Default quote retrieval uses a no-key community quote endpoint (`community-stooq`) on a best-effort basis.
- Optional key-based providers:
  - `BULL_TWELVEDATA_API_KEY`
  - `BULL_ALPHA_VANTAGE_API_KEY`

`bull` does not promise exchange-grade real-time data. It shows the latest available free quote returned by the configured provider and includes the source in table and JSON output.

## Cache

Quote responses are cached briefly, defaulting to 15 seconds. If a refresh fails and cached data exists, `bull` can return the stale cached quote with a stale marker.

Environment overrides:

- `BULL_PROVIDER`
- `BULL_CACHE_TTL_QUOTES`
- `BULL_CACHE_TTL_INDEX`
