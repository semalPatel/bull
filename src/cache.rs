use std::time::Duration;

pub const DEFAULT_QUOTE_CACHE_TTL: Duration = Duration::from_secs(15);
pub const DEFAULT_INDEX_CACHE_TTL: Duration = Duration::from_secs(24 * 60 * 60);
