# Bull Portfolio and Details Execution Plan

## Status
- Phase: Implemented
- Code changes: Portfolio store, CLI subcommands, Stooq details, renderers, tests, and README updates are complete
- Goal: Add local positions/watchlist and Stooq-backed stock details without changing the existing quote-first CLI behavior
- Validation: `cargo test` and `cargo clippy --all-targets --all-features -- -D warnings` pass as of 2026-04-07

## Scope
### In Scope
- Local, user-managed watchlist entries
- Local, user-managed positions with optional shares and average cost
- Default no-argument command that shows saved positions/watchlist
- Explicit position add/update/remove/detail/list commands
- Stooq-backed quote details view with OHLCV fields
- JSON output for portfolio and details flows
- Fixture-backed tests that do not require live network calls

### Out of Scope
- Brokerage account integration
- Transaction lots, tax lots, realized gains, and performance history
- Alerts and notifications
- Historical charts
- Raw Google Search or Google Finance scraping
- Portfolio advice, recommendations, or financial planning

## Product Decisions
- `bull <query>...` remains the direct quote flow.
- `bull` with no arguments should show saved positions/watchlist.
- `bull add <query>` creates a watchlist-only entry when `--shares` and `--avg-cost` are omitted.
- Fractional shares are allowed.
- Position data is local-only user data and must not be stored in the disposable quote cache directory.
- Use `f64` for v1 calculations to avoid expanding scope with a decimal math dependency.
- The default no-key details source is `community-stooq`.
- Google Search should not be used as a stock provider because there is no official public Google Finance API and scraping search/finance result pages is brittle and policy-risky.

## CLI Contract
### Existing Quote Flow To Preserve
```sh
bull apple
bull AAPL
bull apple msft
bull --json apple
bull --watch 2 apple
```

### New Portfolio Commands
```sh
bull
bull positions
bull position AAPL
bull add apple
bull add apple --shares 10 --avg-cost 150
bull update AAPL --shares 12 --avg-cost 155
bull remove AAPL
```

### New Details Commands
```sh
bull details AAPL
bull --json details apple
```

### Flag Behavior
- `--json`: applies to quote, positions, single-position, and details output.
- `--provider`: applies to quote/details fetching when the selected provider supports the requested operation.
- `--symbol`: skips name resolution and treats query inputs as tickers.
- `--yes`: keeps existing ambiguity behavior; for portfolio mutations it may auto-pick the top resolver candidate and warn on stderr.
- `--no-color`: applies to all table/detail output.

## Data Model
### Portfolio Store
```rust
Portfolio {
    schema_version: u32,
    positions: Vec<Position>,
}
```

### Position
```rust
Position {
    symbol: String,
    company_name: Option<String>,
    original_query: Option<String>,
    shares: Option<f64>,
    avg_cost: Option<f64>,
    currency: String,
    notes: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}
```

### Position View
```rust
PositionView {
    position: Position,
    quote: Quote,
    market_value: Option<f64>,
    cost_basis: Option<f64>,
    day_pl: Option<f64>,
    unrealized_pl: Option<f64>,
    unrealized_pl_percent: Option<f64>,
    allocation_percent: Option<f64>,
}
```

### Quote Details
```rust
QuoteDetails {
    symbol: String,
    price: f64,
    open: Option<f64>,
    high: Option<f64>,
    low: Option<f64>,
    close: Option<f64>,
    volume: Option<u64>,
    as_of: Option<DateTime<Utc>>,
    currency: Option<String>,
    source: String,
    stale: bool,
}
```

## Storage Plan
- Use `directories::ProjectDirs`.
- Store user-owned portfolio data under `data_dir()` or `config_dir()`, not `cache_dir()`.
- Suggested file name: `portfolio.json`.
- Use JSON for v1 because the project already uses `serde_json` and the data shape is small.
- Write safely by serializing to a temporary file and renaming over the destination.
- Keep quote cache behavior separate from portfolio storage.

## Validation Rules
- Symbols are stored uppercase.
- `shares` must be greater than `0` when provided.
- `avg_cost` must be greater than or equal to `0` when provided.
- `currency` defaults to `USD`.
- Duplicate symbols are rejected on `add`.
- `update` requires an existing symbol.
- `remove` should return a clear not-found error when the symbol is not saved.
- Portfolio calculations must avoid divide-by-zero and return `None` for unavailable fields.

## Calculation Rules
```text
market_value = shares * price
cost_basis = shares * avg_cost
day_pl = shares * quote.change
unrealized_pl = market_value - cost_basis
unrealized_pl_percent = unrealized_pl / cost_basis * 100
allocation_percent = market_value / total_market_value * 100
```

Null behavior:
- Watchlist-only entry: shares/value/P&L fields are `None`.
- Shares without average cost: market value can be calculated, total P&L fields are `None`.
- Missing quote change: day P&L is `None`.
- Zero total market value: allocation percent is `None`.

## Output Contract
### Positions Table
Columns:
- Symbol
- Company
- Shares
- Price
- Value
- Day P/L
- Total P/L
- % P/L
- Source
- Timestamp

Use `-` for unavailable table values.

### Single Position Detail
Example:
```text
AAPL - Apple Inc.
Price: $258.86
Shares: 10
Avg cost: $150.00
Market value: $2,588.60
Cost basis: $1,500.00
Day P/L: +$23.50
Unrealized P/L: +$1,088.60 (+72.57%)
As of: 2026-04-06T22:00:15Z
Source: community-stooq
```

### Details Table
Columns:
- Symbol
- Price
- Open
- High
- Low
- Close
- Volume
- Timestamp
- Source

### JSON Rules
- Include raw quote or quote details fields.
- Include derived position fields.
- Use `null` for missing values.
- Always include source and stale/freshness metadata.

## Provider Plan
### Stooq Details
The current `community-stooq` CSV response includes:
```text
Symbol,Date,Time,Open,High,Low,Close,Volume
```

Add provider support for quote details:
```rust
trait QuoteProvider {
    fn name(&self) -> &'static str;
    fn quote(&self, symbol: &str) -> Result<Quote>;
    fn quote_details(&self, symbol: &str) -> Result<QuoteDetails> {
        Err(BullError::ProviderUnsupported { ... })
    }
}
```

Implement `quote_details` for `community-stooq`.

Provider policy should expose:
```rust
ProviderPolicy::quote(symbol)
ProviderPolicy::quote_details(symbol)
```

For providers that do not support details yet, return a typed unsupported-provider error. Do not silently invent details from basic quote data.

### Google Search Decision
Do not implement raw Google Search scraping in this feature set.

If Google Finance-style data is ever added, implement it as a separate optional third-party provider, for example:
```text
--provider serpapi-google-finance
```

That provider would require its own API key and docs, and it should be clearly labeled as third-party rather than Google-owned official API access.

## Module Plan
Add:
```text
src/portfolio/mod.rs
src/portfolio/model.rs
src/portfolio/store.rs
src/portfolio/calculator.rs
```

Modify:
```text
src/cli.rs
src/main.rs
src/model.rs
src/output.rs
src/provider/mod.rs
src/provider/community.rs
src/error.rs
tests/integration_cli.rs
README.md
docs/PROJECT_EXECUTION_PLAN.md or this document's status section
```

## Phase 0: Contract Lock
Inputs:
- This document
- Existing v1 quote CLI behavior

Tasks:
- Confirm CLI command names and defaults.
- Confirm local-only storage behavior.
- Confirm Google Search is out of scope.

Outputs:
- Updated planning status if product decisions change.

Gate:
- No code changes.
- Any changed product decision is recorded in this document.

## Phase 1: Portfolio Store
Tasks:
- Add portfolio modules.
- Implement `Portfolio` and `Position`.
- Implement load/save from JSON.
- Implement add/update/remove/list helpers.
- Implement validation.
- Use `ProjectDirs::data_dir()` or `config_dir()`.
- Use safe write pattern: temp file then rename.

Tests:
- Empty/missing store loads as empty portfolio.
- Portfolio persists and reloads.
- Add/update/remove works.
- Duplicate add fails.
- Invalid shares fail.
- Invalid average cost fails.
- Unknown remove fails with typed error.

Gate:
```sh
cargo test portfolio
cargo clippy --all-targets --all-features -- -D warnings
```

Commit:
```text
portfolio: add local position store
```

## Phase 2: CLI Subcommands
Tasks:
- Refactor `Cli` to support subcommands.
- Preserve existing quote varargs flow.
- Make bare `bull` map to positions mode.
- Add `add`, `update`, `remove`, `positions`, `position`, and `details`.
- Ensure `--json`, `--symbol`, `--yes`, `--provider`, and `--no-color` are available where needed.

Tests:
- `bull apple` still means quote query.
- `bull` maps to positions mode.
- `bull add apple --shares 10 --avg-cost 150` parses.
- `bull details AAPL` parses.
- Help text documents the new commands.

Gate:
```sh
cargo test cli
cargo run -- --help
cargo run -- add --help
```

Commit:
```text
cli: add portfolio and details commands
```

## Phase 3: Position Calculations
Tasks:
- Add `PositionView`.
- Implement market value, cost basis, day P/L, total P/L, total P/L percent, and allocation percent.
- Ensure calculation outputs are optional when inputs are incomplete.

Tests:
- Watchlist-only entry has no value/P&L fields.
- Shares without average cost has market value but no total P&L.
- Full position with gain.
- Full position with loss.
- Missing quote change.
- Zero allocation denominator.

Gate:
```sh
cargo test portfolio::calculator
```

Commit:
```text
portfolio: calculate position views
```

## Phase 4: Portfolio Command Flows
Tasks:
- Wire `add`, `update`, `remove`, `positions`, and `position` in `main.rs`.
- Resolve names through existing resolver for mutations.
- Fetch quotes through existing provider policy and quote cache.
- Use stale quote fallback when available.
- Support partial failure for multi-position views.
- Return actionable error messages.

Behavior:
- Empty portfolio table prints an actionable message and exits `0`.
- Empty portfolio JSON prints `[]` and exits `0`.
- Partial quote failure displays available rows and exits non-zero.
- Ambiguous add/update follows existing resolver ambiguity behavior.

Integration tests:
- Add watchlist entry.
- Add position entry.
- Update position.
- Remove position.
- Bare `bull` shows saved entries.
- `bull positions` shows saved entries.
- `bull position AAPL` shows one entry.
- JSON output works.
- Tests isolate data with `XDG_DATA_HOME` or `XDG_CONFIG_HOME`.
- Tests seed quote cache and avoid live network.

Gate:
```sh
cargo test
```

Commit:
```text
portfolio: wire command flows
```

## Phase 5: Stooq Quote Details
Tasks:
- Add `QuoteDetails` model.
- Add default `quote_details` provider trait method returning typed unsupported error.
- Implement `quote_details` in `community-stooq`.
- Parse Stooq CSV fields: symbol, date, time, open, high, low, close, volume.
- Add provider policy `quote_details`.
- Add details command flow.

Tests:
- Parses valid Stooq CSV.
- `N/D` row returns typed provider error.
- Missing row returns typed provider error.
- Unsupported provider returns typed provider error.
- `bull details AAPL` table output.
- `bull --json details apple` JSON output.

Gate:
```sh
cargo test provider::community
cargo test --test integration_cli details
```

Commit:
```text
provider: add stooq quote details
```

## Phase 6: Output Rendering
Tasks:
- Add positions table renderer.
- Add single-position detail renderer.
- Add quote-details renderer.
- Add JSON renderers for position and details outputs.
- Preserve `--no-color`.
- Always show source and timestamp/freshness.
- Use `-` in tables and `null` in JSON for missing values.

Tests:
- Positions table includes expected columns.
- Single position detail includes calculated fields.
- Watchlist-only output does not show misleading zero P&L.
- Details table includes OHLCV.
- JSON output includes raw and derived fields.

Gate:
```sh
cargo test output
cargo test --test integration_cli
```

Commit:
```text
output: render portfolio and details views
```

## Phase 7: Docs And Release Validation
Tasks:
- Update `README.md`.
- Update this document status and decision log.
- Document local-only storage and no brokerage sync.
- Document data location.
- Document watchlist-only vs position examples.
- Document Stooq best-effort caveat.
- Document Google Search non-goal.

Required quality gates:
```sh
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo build --release
test -x target/release/bull
```

Smoke tests:
```sh
target/release/bull --help
target/release/bull details AAPL
target/release/bull --json details apple
target/release/bull add apple
target/release/bull
target/release/bull position AAPL
target/release/bull update AAPL --shares 10 --avg-cost 150
target/release/bull positions
target/release/bull remove AAPL
```

Use isolated data/cache dirs for smoke testing when possible:
```sh
XDG_DATA_HOME=/tmp/bull-smoke-data XDG_CACHE_HOME=/tmp/bull-smoke-cache target/release/bull add apple
```

Commit:
```text
docs: document portfolio and details usage
```

## Final Definition Of Done
- Existing quote flows still work.
- Bare `bull` shows saved watchlist/positions.
- Position add/update/remove/list/detail flows work.
- Watchlist-only entries do not require shares or average cost.
- Position calculations are correct and tested.
- `bull details <query>` shows Stooq OHLCV details.
- JSON output is stable for positions and details.
- No raw Google Search scraping is implemented.
- All required quality gates pass.
- Release binary exists at `target/release/bull`.
