use crate::error::{BullError, Result};
use crate::model::{Resolution, ResolutionCandidate, ResolutionStrategy};
use std::collections::BTreeMap;
use std::fs;
use std::io::{self, IsTerminal, Write};
use std::path::PathBuf;

pub mod matcher;
pub mod sec_index;

#[derive(Debug, Clone)]
pub struct Resolver {
    index: sec_index::SecIndex,
    aliases: BTreeMap<String, ResolutionCandidate>,
    alias_path: Option<PathBuf>,
}

impl Resolver {
    pub fn new() -> Self {
        Self::with_index(sec_index::SecIndex::load())
    }

    pub fn with_index(index: sec_index::SecIndex) -> Self {
        let alias_path = alias_path();
        let aliases = alias_path
            .as_ref()
            .and_then(|path| fs::read(path).ok())
            .and_then(|bytes| serde_json::from_slice(&bytes).ok())
            .unwrap_or_default();

        Self {
            index,
            aliases,
            alias_path,
        }
    }

    pub fn resolve(&mut self, query: &str, force_symbol: bool, yes: bool) -> Result<Resolution> {
        let normalized = query.trim();
        if normalized.is_empty() {
            return Err(BullError::NoQueries);
        }

        if force_symbol || matcher::is_ticker(normalized) {
            return Ok(Resolution {
                query: query.to_string(),
                symbol: normalized.to_uppercase(),
                company_name: None,
                confidence: 1.0,
                strategy: if force_symbol {
                    ResolutionStrategy::ForcedSymbol
                } else {
                    ResolutionStrategy::TickerPattern
                },
            });
        }

        let normalized_query = matcher::normalize(query);
        if let Some(alias) = self.aliases.get(&normalized_query) {
            return Ok(to_resolution(query, alias, ResolutionStrategy::Alias));
        }

        let candidates = matcher::candidates(query, self.index.companies());
        if candidates.is_empty() {
            return Err(BullError::NoMatch {
                query: query.to_string(),
                suggestions: Vec::new(),
            });
        }

        if matcher::auto_resolves(&candidates) {
            return Ok(to_resolution(
                query,
                &candidates[0],
                candidates[0].strategy.clone(),
            ));
        }

        if yes {
            eprintln!(
                "bull: ambiguous query '{query}', picking {} ({}) because --yes was set",
                candidates[0].symbol, candidates[0].company_name
            );
            return Ok(to_resolution(
                query,
                &candidates[0],
                candidates[0].strategy.clone(),
            ));
        }

        if !io::stdin().is_terminal() {
            return Err(BullError::Ambiguous {
                query: query.to_string(),
                candidates,
            });
        }

        let selected = prompt_for_candidate(query, &candidates)?;
        let candidate = candidates
            .get(selected)
            .ok_or_else(|| BullError::InteractionRequired {
                query: query.to_string(),
            })?;
        self.persist_alias(&normalized_query, candidate)?;
        Ok(to_resolution(query, candidate, candidate.strategy.clone()))
    }

    fn persist_alias(
        &mut self,
        normalized_query: &str,
        candidate: &ResolutionCandidate,
    ) -> Result<()> {
        self.aliases
            .insert(normalized_query.to_string(), candidate.clone());
        if let Some(path) = &self.alias_path {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(path, serde_json::to_vec_pretty(&self.aliases)?)?;
        }
        Ok(())
    }
}

impl Default for Resolver {
    fn default() -> Self {
        Self::new()
    }
}

fn to_resolution(
    query: &str,
    candidate: &ResolutionCandidate,
    strategy: ResolutionStrategy,
) -> Resolution {
    Resolution {
        query: query.to_string(),
        symbol: candidate.symbol.clone(),
        company_name: Some(candidate.company_name.clone()),
        confidence: candidate.confidence,
        strategy,
    }
}

fn prompt_for_candidate(query: &str, candidates: &[ResolutionCandidate]) -> Result<usize> {
    eprintln!("Ambiguous query '{query}'. Select one:");
    for (index, candidate) in candidates.iter().enumerate() {
        eprintln!(
            "{}) {} - {}",
            index + 1,
            candidate.symbol,
            candidate.company_name
        );
    }
    eprint!("Selection: ");
    io::stderr().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let selected = input
        .trim()
        .parse::<usize>()
        .ok()
        .and_then(|value| value.checked_sub(1))
        .ok_or_else(|| BullError::InteractionRequired {
            query: query.to_string(),
        })?;
    Ok(selected)
}

fn alias_path() -> Option<PathBuf> {
    directories::ProjectDirs::from("dev", "bull", "bull")
        .map(|dirs| dirs.cache_dir().join("resolver_aliases.json"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolver::sec_index::Company;

    fn resolver() -> Resolver {
        Resolver::with_index(sec_index::SecIndex::from_companies(vec![
            Company {
                symbol: "AAPL".to_string(),
                company_name: "Apple Inc.".to_string(),
                cik: 320193,
                exchange: None,
            },
            Company {
                symbol: "META".to_string(),
                company_name: "Meta Platforms, Inc.".to_string(),
                cik: 1326801,
                exchange: None,
            },
        ]))
    }

    #[test]
    fn resolves_lowercase_company_name() {
        let mut resolver = resolver();
        let resolution = resolver.resolve("apple", false, false).unwrap();
        assert_eq!(resolution.symbol, "AAPL");
        assert_eq!(resolution.strategy, ResolutionStrategy::ExactName);
    }

    #[test]
    fn force_symbol_skips_name_resolution() {
        let mut resolver = resolver();
        let resolution = resolver.resolve("apple", true, false).unwrap();
        assert_eq!(resolution.symbol, "APPLE");
        assert_eq!(resolution.strategy, ResolutionStrategy::ForcedSymbol);
    }
}
