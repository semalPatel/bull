use std::fs;
use std::process::Command;

fn seeded_cache_home() -> std::path::PathBuf {
    let root = std::env::temp_dir().join(format!(
        "bull-cli-test-{}-{}",
        std::process::id(),
        std::thread::current().name().unwrap_or("unnamed")
    ));
    let cache_dir = root.join("bull");
    fs::create_dir_all(&cache_dir).unwrap();
    fs::write(
        cache_dir.join("quotes.json"),
        r#"{
  "AAPL": {
    "quote": {
      "symbol": "AAPL",
      "price": 182.31,
      "change": 1.2,
      "change_percent": 0.66,
      "as_of": "2026-04-06T20:00:00Z",
      "currency": "USD",
      "source": "fixture-cache",
      "stale": false
    },
    "cached_at": "2026-04-06T20:00:00Z"
  },
  "MSFT": {
    "quote": {
      "symbol": "MSFT",
      "price": 420.50,
      "change": -2.1,
      "change_percent": -0.50,
      "as_of": "2026-04-06T20:00:00Z",
      "currency": "USD",
      "source": "fixture-cache",
      "stale": false
    },
    "cached_at": "2026-04-06T20:00:00Z"
  }
}"#,
    )
    .unwrap();
    root
}

fn isolated_data_home() -> std::path::PathBuf {
    let root = std::env::temp_dir().join(format!(
        "bull-cli-data-test-{}-{}",
        std::process::id(),
        std::thread::current().name().unwrap_or("unnamed")
    ));
    fs::create_dir_all(&root).unwrap();
    root
}

#[test]
fn help_documents_free_quote_positioning() {
    let output = Command::new(env!("CARGO_BIN_EXE_bull"))
        .arg("--help")
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("latest available free quote"));
}

#[test]
fn resolves_name_and_renders_json_from_cache() {
    let cache_home = seeded_cache_home();
    let output = Command::new(env!("CARGO_BIN_EXE_bull"))
        .env("XDG_CACHE_HOME", cache_home)
        .env("BULL_CACHE_TTL_QUOTES", "315360000")
        .args(["--json", "apple"])
        .output()
        .unwrap();

    assert!(output.status.success(), "{:?}", output);
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("\"query\": \"apple\""));
    assert!(stdout.contains("\"symbol\": \"AAPL\""));
    assert!(stdout.contains("\"source\": \"fixture-cache\""));
}

#[test]
fn renders_mixed_query_table_from_cache() {
    let cache_home = seeded_cache_home();
    let output = Command::new(env!("CARGO_BIN_EXE_bull"))
        .env("XDG_CACHE_HOME", cache_home)
        .env("BULL_CACHE_TTL_QUOTES", "315360000")
        .args(["--no-color", "apple", "MSFT"])
        .output()
        .unwrap();

    assert!(output.status.success(), "{:?}", output);
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Query"));
    assert!(stdout.contains("AAPL"));
    assert!(stdout.contains("MSFT"));
    assert!(stdout.contains("fixture-cache"));
}

#[test]
fn portfolio_add_update_position_remove_flow_uses_isolated_data() {
    let data_home = isolated_data_home();
    let cache_home = seeded_cache_home();

    let add = Command::new(env!("CARGO_BIN_EXE_bull"))
        .env("XDG_DATA_HOME", &data_home)
        .args(["add", "apple", "--shares", "10", "--avg-cost", "150"])
        .output()
        .unwrap();
    assert!(add.status.success(), "{:?}", add);

    let list = Command::new(env!("CARGO_BIN_EXE_bull"))
        .env("XDG_DATA_HOME", &data_home)
        .env("XDG_CACHE_HOME", &cache_home)
        .env("BULL_CACHE_TTL_QUOTES", "315360000")
        .arg("--no-color")
        .output()
        .unwrap();
    assert!(list.status.success(), "{:?}", list);
    let stdout = String::from_utf8(list.stdout).unwrap();
    assert!(stdout.contains("AAPL"));
    assert!(stdout.contains("$1,823.10") || stdout.contains("$1823.10"));
    assert!(stdout.contains("fixture-cache"));

    let update = Command::new(env!("CARGO_BIN_EXE_bull"))
        .env("XDG_DATA_HOME", &data_home)
        .args(["update", "AAPL", "--shares", "12", "--avg-cost", "155"])
        .output()
        .unwrap();
    assert!(update.status.success(), "{:?}", update);

    let position = Command::new(env!("CARGO_BIN_EXE_bull"))
        .env("XDG_DATA_HOME", &data_home)
        .env("XDG_CACHE_HOME", &cache_home)
        .env("BULL_CACHE_TTL_QUOTES", "315360000")
        .args(["--no-color", "position", "AAPL"])
        .output()
        .unwrap();
    assert!(position.status.success(), "{:?}", position);
    let stdout = String::from_utf8(position.stdout).unwrap();
    assert!(stdout.contains("Shares: 12"));
    assert!(stdout.contains("Avg cost: $155.00"));

    let remove = Command::new(env!("CARGO_BIN_EXE_bull"))
        .env("XDG_DATA_HOME", &data_home)
        .args(["remove", "AAPL"])
        .output()
        .unwrap();
    assert!(remove.status.success(), "{:?}", remove);
}

#[test]
fn portfolio_add_and_remove_accept_multiple_watchlist_entries() {
    let data_home = isolated_data_home();
    let cache_home = seeded_cache_home();

    let add = Command::new(env!("CARGO_BIN_EXE_bull"))
        .env("XDG_DATA_HOME", &data_home)
        .args(["add", "apple", "MSFT"])
        .output()
        .unwrap();
    assert!(add.status.success(), "{:?}", add);
    let stdout = String::from_utf8(add.stdout).unwrap();
    assert!(stdout.contains("AAPL"));
    assert!(stdout.contains("MSFT"));

    let list = Command::new(env!("CARGO_BIN_EXE_bull"))
        .env("XDG_DATA_HOME", &data_home)
        .env("XDG_CACHE_HOME", &cache_home)
        .env("BULL_CACHE_TTL_QUOTES", "315360000")
        .arg("--no-color")
        .output()
        .unwrap();
    assert!(list.status.success(), "{:?}", list);
    let stdout = String::from_utf8(list.stdout).unwrap();
    assert!(stdout.contains("AAPL"));
    assert!(stdout.contains("MSFT"));

    let remove = Command::new(env!("CARGO_BIN_EXE_bull"))
        .env("XDG_DATA_HOME", &data_home)
        .args(["remove", "AAPL", "MSFT"])
        .output()
        .unwrap();
    assert!(remove.status.success(), "{:?}", remove);
    let stdout = String::from_utf8(remove.stdout).unwrap();
    assert!(stdout.contains("AAPL"));
    assert!(stdout.contains("MSFT"));
}

#[test]
fn portfolio_multi_add_rejects_position_fields() {
    let data_home = isolated_data_home();
    let output = Command::new(env!("CARGO_BIN_EXE_bull"))
        .env("XDG_DATA_HOME", data_home)
        .args(["add", "apple", "MSFT", "--shares", "10"])
        .output()
        .unwrap();

    assert!(!output.status.success(), "{:?}", output);
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("--shares and --avg-cost can only be used when adding one position"));
}

#[test]
fn empty_portfolio_json_is_empty_array() {
    let data_home = isolated_data_home();
    let output = Command::new(env!("CARGO_BIN_EXE_bull"))
        .env("XDG_DATA_HOME", data_home)
        .arg("--json")
        .output()
        .unwrap();

    assert!(output.status.success(), "{:?}", output);
    assert_eq!(String::from_utf8(output.stdout).unwrap().trim(), "[]");
}

#[test]
fn help_documents_new_commands() {
    let output = Command::new(env!("CARGO_BIN_EXE_bull"))
        .arg("--help")
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("positions"));
    assert!(stdout.contains("details"));
    assert!(stdout.contains("add"));
}

#[test]
fn global_flags_are_accepted_after_subcommands() {
    let output = Command::new(env!("CARGO_BIN_EXE_bull"))
        .env_remove("BULL_ALPHA_VANTAGE_API_KEY")
        .args(["details", "AAPL", "--provider", "alphavantage"])
        .output()
        .unwrap();

    assert_ne!(output.status.code(), Some(2), "{:?}", output);
}
