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
