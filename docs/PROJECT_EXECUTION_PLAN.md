# Bull CLI Execution Plan

## Status
- Phase: Planning only
- Code changes: Not started
- Goal: Iterate on this document until scope and behavior are stable, then implement

## Indexed Execution Map
1. `Map-01`: Product promise and scope guardrails
   - Read: `Product Positioning (v1)`, `Success Criteria`, `Scope (v1)`
   - Implementer outcome: lock behavior to "latest available free quote" for US equities
2. `Map-02`: Data source and provider policy
   - Read: `Data Source Strategy (Free-First)`, `Provider Evaluation`, `SEC Direct Data Plan`
   - Implementer outcome: SEC for resolver truth, separate quote provider policy with fallback
3. `Map-03`: Resolver behavior and confidence rules
   - Read: `Query Resolution Plan` and threshold decisions
   - Implementer outcome: deterministic symbol resolution and ambiguity UX
4. `Map-04`: Output and UX contract
   - Read: `User Experience Design`, `Output Design`
   - Implementer outcome: stable CLI behavior for table, JSON, watch mode
5. `Map-05`: Performance and resilience constraints
   - Read: `Performance and Caching`, `Reliability and Error Handling`
   - Implementer outcome: bounded retries, cache policy, stale fallback, interval floor
6. `Map-06`: Build architecture and file ownership
   - Read: `Execution Handoff (Agent-Ready)` planned file/module layout
   - Implementer outcome: clear module boundaries and implementation write targets
7. `Map-07`: Implementation order and gate progression
   - Read: `End-to-End Implementation Blueprint`, `Phase Exit Checklist (Gate Table)`
   - Implementer outcome: phase-by-phase delivery with objective pass/fail gates
8. `Map-08`: Validation and release proof
   - Read: `Smoke Test Plan`, `Definition of done (v1)`
   - Implementer outcome: proven release binary and smoke-tested behavior

## Agent Startup Checklist
1. Read `Map-01` through `Map-08` in order before writing code.
2. Confirm planned module layout and create missing files only when entering relevant phase.
3. Implement one phase at a time and satisfy that phase gate before proceeding.
4. Record any scope deviation in `Decision Log` with date and reason.
5. Do not claim "real-time"; preserve "latest available free quote" wording in UX/help text.

## Product Goal
Build a dead-simple CLI to check stock prices quickly from the terminal, with first-run success using free data sources.

Primary command should stay:
- `bull <query>`

Where `<query>` can be:
- ticker symbol (`AAPL`)
- natural company name (`apple`, `apple inc`, `tesla`)

## Product Positioning (v1)
- Promise: "latest available free quote" for US equities
- Non-promise: guaranteed exchange-grade real-time quotes for every symbol/provider
- UX requirement: always show quote freshness and source so users can judge latency
- Messaging requirement: describe watch mode as "refreshing latest available quote", not "real-time stream"

## Free Cost Feasibility
### Short answer
- Yes, a free-of-cost CLI is feasible for users and maintainers if the architecture stays client-side and uses open/public data responsibly.

### Cost model assumptions
- No backend servers operated by this project
- No paid API key required for baseline usage
- Local caching to reduce repeated upstream calls
- Optional user-supplied API keys for improved reliability

### What is free in v1
- Company name resolution via SEC data files
- Local resolver index/cache
- CLI usage itself (open-source distribution)

### Where free can break
- Community no-key quote endpoints can degrade or change
- Free-tier API providers can tighten limits over time
- Heavy watch usage may exceed provider constraints

### Mitigations to stay free
- Multi-provider abstraction with runtime failover
- Quote cache + watch interval floor
- Explicit best-effort mode messaging
- Optional bring-your-own-key path for reliability
- No project-hosted proxy to avoid infrastructure bills

## Success Criteria
- Zero-config first run works for common US equities
- Name input resolves to ticker with high confidence
- Single-command flow for most users (`bull apple`)
- Fast response on warm cache
- Clear errors and actionable recovery paths
- `cargo build --release` succeeds and generates runnable binary
- Smoke test command set passes on a clean machine profile

## Constraints
- Prefer free data sources
- Avoid mandatory API keys for baseline behavior
- Keep CLI surface area small and discoverable
- Keep output readable by default, scriptable when needed

## Scope (v1)
- Quote retrieval for US equities
- Name-to-symbol resolution
- Mixed input support (tickers + names in same command)
- Basic watch mode
- JSON output mode
- Caching for speed and reliability

Out of scope for v1:
- Portfolio tracking
- Alerts/notifications
- International markets beyond initial support
- Historical charts and advanced analytics

## User Experience Design
### Core flows
- `bull AAPL` -> immediate quote table row
- `bull apple` -> resolve to `AAPL`, then quote
- `bull apple msft tesla` -> resolve mixed inputs and print multi-row table
- `bull --json apple` -> machine-readable output
- `bull --watch 2 apple` -> refresh every 2 seconds

### Simplicity principles
- One default command path
- Sensible defaults without configuration
- Optional flags only for power users
- Minimal prompts, only when ambiguity is real

### Suggested CLI flags (minimal set)
- `--json` output as JSON
- `--watch <seconds>` refresh interval
- `--provider <name>` provider override
- `--symbol` force ticker interpretation
- `--no-color` disable ANSI color

## Data Source Strategy (Free-First)
### 1) Default quote provider (no API key)
- Use a no-key quote endpoint for baseline operation (best-effort community path)
- Pros: zero onboarding friction
- Risks: delayed data, symbol formatting quirks

### 2) Company-name resolution source (no API key)
- Use SEC official ticker/company mapping for US company resolution
- Store local normalized index for fast search

### 3) Optional fallback provider (free-tier API key)
- Alpha Vantage only when key is configured
- Used for resilience or manual provider selection

### Provider policy
- Default path: no-key providers
- If unavailable: fallback provider if configured
- Always surface which provider supplied each quote

## Provider Evaluation (April 7, 2026)
### Decision criteria
- Free tier availability and longevity
- First-run simplicity (no setup preferred)
- Quote freshness and reliability
- Official docs quality and stability
- Legal clarity for redistribution and caching

### Candidate matrix
- `Community no-key quote endpoint` (example: public CSV-style quote URL)
  - Value: best first-run UX and zero signup
  - Tradeoff: weakest SLA/legal certainty, symbol quirks likely
  - Plan: support as default best-effort path with explicit "community source" label
- `Twelve Data` (official free tier with API key)
  - Value: broad coverage and clean API model
  - Tradeoff: key required and credit-based limits
  - Plan: first recommended key-based provider
- `Alpha Vantage` (official free tier with API key)
  - Value: mature ecosystem and existing project alignment
  - Tradeoff: free-tier constraints and endpoint-specific freshness
  - Plan: secondary key-based provider and fallback
- `Finnhub` (official API with free retail-focused messaging)
  - Value: strong developer positioning and broad datasets
  - Tradeoff: implementation should treat plan limits as dynamic and validate per account tier
  - Plan: post-v1 plugin target unless it clearly beats Twelve Data for v1

### v1 provider decision
- Default mode: no-key community provider for zero-friction onboarding
- Recommended reliable mode: `Twelve Data` when user adds a free API key
- Built-in fallback: `Alpha Vantage` when configured
- Future mode: pluggable providers behind one trait so users can bring their own free/open source

### Source notes
- SEC Data Resources: https://www.sec.gov/sec-data-resources
- Alpha Vantage docs: https://www.alphavantage.co/documentation/
- Twelve Data support/docs intro: https://support.twelvedata.com/en/articles/5609168-introduction-to-twelve-data
- Finnhub public product page: https://finnhubio.github.io/

## SEC Direct Data Plan
### What SEC data can power directly
- Company-name to ticker resolution (`company_tickers.json`, `company_tickers_exchange.json`)
- CIK lookup and canonical company naming
- Filing metadata and fundamentals context via `data.sec.gov` APIs

### What SEC data cannot replace
- Intraday/live quote feed for listed equities
- Exchange-grade real-time price stream

### SEC endpoints to integrate for v1 planning
- Mapping files:
  - `https://www.sec.gov/files/company_tickers.json`
  - `https://www.sec.gov/files/company_tickers_exchange.json`
- API docs:
  - `https://www.sec.gov/edgar/sec-api-documentation`
- Company submissions/facts (post-v1 enrichment):
  - `https://data.sec.gov/submissions/CIK##########.json`
  - `https://data.sec.gov/api/xbrl/companyfacts/CIK##########.json`

### SEC compliance requirements (must implement)
- Set a descriptive `User-Agent` on all SEC requests (app name + contact email)
- Respect SEC fair-access limits (current guideline: at most 10 requests/second)
- Prefer cached files and conditional refresh over repeated live fetches
- Treat SEC mapping accuracy as best-effort and periodically refreshed

### v1 decision using SEC directly
- SEC is the authoritative source for name/ticker resolution in v1
- SEC is not the quote source; quote provider remains separate
- Resolver quality should be measured against SEC mappings before any fuzzy expansions

## Query Resolution Plan
### Input classification
1. Normalize query (trim, lowercase, punctuation handling)
2. If query matches ticker pattern, treat as ticker unless user overrides
3. Otherwise perform name lookup against local index

### Matching strategy
Priority order:
1. exact normalized company-name match
2. exact symbol alias match
3. prefix match
4. token-based fuzzy match

### Confidence behavior
- Single high-confidence candidate -> auto-resolve
- Multiple similar candidates -> show short numbered choices
- No good candidate -> show top suggestions and exit non-zero

### Threshold decisions (v1)
- Auto-resolve only when confidence >= `0.90` and lead margin >= `0.15`
- If top result confidence is `0.70-0.89`, show candidates and require user pick
- If top result confidence < `0.70`, treat as no-match and show suggestions
- Persist manual selection in local alias cache so repeated queries auto-resolve

### Ambiguity UX
Example behavior:
- Input: `bull meta`
- If ambiguous, show:
  - `1) META - Meta Platforms, Inc.`
  - `2) ...`
- Prompt user to select one item (or allow `--symbol` to skip resolution path)

### Ambiguity decisions (v1)
- Prompt by default when ambiguous (no silent auto-pick below threshold)
- Candidate list size: maximum `5`
- Add `--yes` mode for non-interactive pipelines; auto-pick top and write warning to stderr

## Output Design
### Default table columns
- Query
- Symbol
- Price
- Change
- % Change
- Timestamp (if available)
- Source

### JSON mode
- Stable schema for scripting
- Include:
  - original query
  - resolved symbol
  - quote fields
  - provider name
  - freshness metadata

## Performance and Caching
### Caches
- Company index cache (TTL ~24h)
- Quote cache (TTL ~10-15s default, lower in watch mode)

### Watch mode decisions (v1)
- Minimum watch interval: `2s` (hard floor)
- Default watch interval: `5s`
- Inputs below floor are clamped with one-line notice

### Performance targets
- Warm-cache single query under 100ms local overhead
- Cold-start acceptable under network latency constraints

### Concurrency
- Resolve and fetch multiple queries with bounded parallelism
- Preserve deterministic output order matching user input

## Reliability and Error Handling
- Network timeout and retry policy with capped backoff
- Rate limit handling with clear user messaging
- Stale quote fallback when offline (with stale indicator)
- Graceful partial failure in multi-query mode
- Never panic on malformed provider payloads

## Architecture Plan
Proposed modules:
- `cli`: argument parsing and mode selection
- `resolver`: query normalization, lookup, confidence scoring
- `provider`: quote provider trait + implementations
- `cache`: disk cache and TTL logic
- `model`: quote and resolution domain types
- `output`: table and JSON rendering
- `error`: typed errors and user-facing formatting

### Data model (conceptual)
- `Quote { symbol, price, change, change_percent, as_of, currency, source }`
- `Resolution { query, symbol, company_name, confidence, strategy }`

## Configuration Plan
### Environment variables
- `BULL_PROVIDER`
- `BULL_TWELVEDATA_API_KEY`
- `BULL_ALPHA_VANTAGE_API_KEY`
- `BULL_CACHE_TTL_QUOTES`
- `BULL_CACHE_TTL_INDEX`

### Config file (optional)
- Location: user config dir
- Format: TOML
- Rule: CLI flags override env, env overrides config defaults

## Testing Strategy (Before Implementation Lock)
### Unit tests
- normalization and tokenization
- ticker pattern detection
- confidence scoring
- parsing provider fixtures

### Integration tests
- `bull apple` happy path
- mixed queries (`bull apple msft`)
- ambiguous name resolution flow
- no-match flow
- JSON output contract

### Non-functional tests
- cache behavior across TTL boundaries
- watch mode loop behavior
- resilience to network/provider errors

## Milestones
1. Confirm final UX and flag set in this doc
2. Finalize provider and resolution sources
3. Implement provider abstraction and quote retrieval
4. Implement resolver and local company index cache
5. Implement ambiguity and suggestion UX
6. Implement output modes and watch mode
7. Add tests and fixture coverage
8. Update README and usage examples
9. Stabilization and release checklist

## End-to-End Implementation Blueprint
### Phase 0: Contract and scaffolding
- Finalize data contracts (`Quote`, `Resolution`, error surface)
- Define `Provider` trait and resolver interfaces
- Lock CLI command contract (`bull <query>`, flags)

### Phase 1: SEC-powered resolver
- Build SEC downloader/cache with conditional refresh
- Parse SEC ticker mapping files into local index
- Implement normalization, exact/prefix/fuzzy matching, confidence scoring
- Implement ambiguity prompt and alias persistence

### Phase 2: Quote provider layer
- Implement default no-key provider adapter
- Implement Twelve Data adapter (env key)
- Implement Alpha Vantage adapter (env key)
- Add provider policy: priority, fallback, and source labeling

### Phase 3: CLI UX and output
- Mixed query batching (names + symbols)
- Default table output with source/timestamp
- JSON output contract and stable fields
- `--watch` loop with interval clamp and graceful interruption

### Phase 4: Resilience and performance
- Add short-lived quote cache
- Add retry/backoff and partial failure handling
- Add stale-data indicators when offline/unavailable
- Enforce SEC fair-access and provider rate protections

### Phase 5: Validation and release
- Unit tests for resolver scoring and provider parsers
- Integration tests for main command flows
- Fixture-based tests to avoid live network dependence
- README updates: install, examples, limits, provider caveats

### Phase Exit Checklist (Gate Table)
| Phase | Required Inputs | Required Outputs | Pass/Fail Gate |
| --- | --- | --- | --- |
| Phase 0: Contract and scaffolding | Finalized v1 scope in this doc; selected crate set; CLI contract draft | Compiling core types/traits; documented CLI flags/defaults; no TODO blockers in core interfaces | `cargo check` passes and interfaces compile without placeholder panics |
| Phase 1: SEC-powered resolver | SEC endpoint list; cache policy; confidence thresholds | Working SEC index loader; resolver returns symbol + confidence; ambiguity behavior implemented | Resolver unit tests pass; fixture-based SEC parse tests pass; offline resolver tests deterministic |
| Phase 2: Quote provider layer | Provider trait; provider priority order; source labeling rules | At least one no-key provider adapter; optional key-based adapter wiring; provider fallback policy | Provider fixture tests pass; unknown/failed provider path returns typed errors; source labeling visible |
| Phase 3: CLI UX and output | Output schema; non-interactive behavior rules; watch defaults | `bull <query>` flow works for ticker/name/mixed inputs; JSON mode stable; watch mode runs and interrupts cleanly | Integration tests for core flows pass; CLI exits non-zero on failure paths; help text reflects behavior |
| Phase 4: Resilience and performance | Retry/backoff policy; cache TTL defaults; rate-limit rules | Quote cache with freshness metadata; stale-data fallback path; bounded concurrency | Non-functional tests pass for TTL and partial failures; watch interval clamp enforced; no panic on bad payloads |
| Phase 5: Validation and release | Full test suite; smoke test command list; release build target | Updated README; passing smoke checks; release binary generated | `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`, `cargo build --release`, and `test -x target/release/bull` all pass |

### Runtime flow (single query)
1. Parse CLI args and query tokens
2. Resolve each token to symbol (SEC index first)
3. Fetch quote(s) via provider policy
4. Apply cache/fallback if needed
5. Render table or JSON and return code

### Runtime flow (watch mode)
1. Resolve once and cache symbol mapping
2. Loop on interval floor (>=2s)
3. Refresh quotes using cache-aware strategy
4. Render updated output with freshness/source info
5. Stop cleanly on user interrupt

## Execution Handoff (Agent-Ready)
### Planned file/module layout
- `src/main.rs`: CLI entrypoint and exit codes
- `src/cli.rs`: args, defaults, and non-interactive flags
- `src/model.rs`: `Quote`, `Resolution`, provider/freshness metadata
- `src/resolver/mod.rs`: resolver orchestration
- `src/resolver/sec_index.rs`: SEC download, parse, and local index cache
- `src/resolver/matcher.rs`: normalization and scoring
- `src/provider/mod.rs`: provider trait and policy
- `src/provider/community.rs`: no-key quote adapter
- `src/provider/twelvedata.rs`: Twelve Data adapter
- `src/provider/alphavantage.rs`: Alpha Vantage adapter
- `src/cache.rs`: quote + index cache and TTL
- `src/output.rs`: table and JSON renderers
- `src/error.rs`: typed errors and user-facing mapping
- `tests/integration_cli.rs`: CLI flow integration tests
- `tests/fixtures/*`: provider and SEC sample payloads

### Dependency plan (Rust crates)
- CLI: `clap` (derive)
- HTTP: `reqwest` (blocking or async chosen once)
- Serialization: `serde`, `serde_json`, `csv`
- Time: `chrono`
- Caching/config paths: `directories`
- Errors: `thiserror` + `anyhow`
- Terminal formatting: existing `console`/table crate or equivalent
- Tests: `assert_cmd`, `predicates`, fixture loading utilities

### Implementation sequence with gates
1. Build core models and trait contracts; compile gate
2. Add SEC resolver and fixture tests; resolver gate
3. Add one quote provider and fallback policy; quote gate
4. Add CLI output/watch mode; UX gate
5. Add second provider + reliability handling; resilience gate
6. Finalize docs, smoke tests, and release binary gate

### Definition of done (v1)
- All planned unit and integration tests pass
- Smoke tests pass with and without API keys
- `cargo build --release` produces binary at `target/release/bull`
- `bull --help` documents "latest available free quote" positioning
- Freshness/source fields visible in default output and JSON

## Smoke Test Plan
### Build verification
- `cargo fmt --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test`
- `cargo build --release`
- Verify binary exists: `test -x target/release/bull`

### CLI smoke tests (no key)
- `target/release/bull --help`
- `target/release/bull AAPL`
- `target/release/bull apple`
- `target/release/bull apple msft`
- `target/release/bull --json apple`
- `target/release/bull --watch 2 apple` (manual short run, then interrupt)

### CLI smoke tests (with optional keys)
- `BULL_TWELVEDATA_API_KEY=... target/release/bull --provider twelvedata AAPL`
- `BULL_ALPHA_VANTAGE_API_KEY=... target/release/bull --provider alphavantage AAPL`

### Failure-path smoke tests
- Unknown company name returns non-zero and suggestions
- Ambiguous company prompts in interactive mode
- `--yes` mode auto-picks top match and writes warning to stderr
- Watch interval below floor clamps to `2s` with one-line notice

## Democratization Strategy
- Keep core usage free without mandatory account creation
- Keep provider layer open and documented so community can add adapters
- Publish provider contract and fixtures so alternative providers are easy to test
- Never hard-wire vendor-specific logic into CLI UX paths
- Keep telemetry disabled by default; if ever added, require explicit opt-in
- Document data-source caveats plainly (delay, licensing, reliability)
- Use government open data (SEC) as the default source of truth for symbol identity

## Open Ecosystem Plan
- Add a public `Provider` interface contract and adapter guide
- Support runtime provider selection via `--provider` and config/env
- Maintain compatibility matrix in docs (freshness, auth, market scope, limits)
- Encourage community providers in separate crates/modules to avoid core bloat

## Risks and Mitigations
- Free providers may be unstable
  - Mitigation: provider fallback and explicit source labeling
- Name resolution false positives
  - Mitigation: confidence threshold and ambiguity prompt
- Symbol format differences across providers
  - Mitigation: provider-specific symbol normalization layer
- Rate limits in watch mode
  - Mitigation: minimum refresh interval and cache-aware fetches

## Resolved Open Questions (v1)
- Default quote provider: no-key community source with explicit best-effort labeling
- Ambiguity policy: prompt user unless confidence is very high
- Maximum candidate list size on ambiguity: 5
- Minimum watch interval: 2 seconds
- Market scope: US equities only for v1 name-resolution guarantees

## Open Questions for Iteration (Post-v1)
- Add first-class non-US exchange support or keep provider-specific suffix flow
- Add optional persistent alias commands (`bull alias add "google" GOOGL`)
- Introduce a dedicated `search` subcommand or keep single-command philosophy
- Select third provider for redundancy (likely Twelve Data vs Finnhub priority)
- Decide whether to support user-supplied custom provider endpoints in config

## Decision Log
Use this section as decisions are made.

- 2026-04-06: Planning doc created; implementation intentionally deferred.
- 2026-04-07: Resolved v1 defaults for ambiguity behavior, watch interval, and market scope.
- 2026-04-07: Added provider evaluation matrix and democratization strategy.
- 2026-04-07: Confirmed SEC data is primary source for name-to-ticker resolution, not for live quotes.
- 2026-04-07: Confirmed free-of-cost feasibility with client-side architecture and no hosted backend.
- 2026-04-07: v1 messaging locked to "latest available free quote" instead of guaranteed real-time.
