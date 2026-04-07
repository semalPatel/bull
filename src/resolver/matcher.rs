use crate::model::{ResolutionCandidate, ResolutionStrategy};
use crate::resolver::sec_index::Company;

const AUTO_RESOLVE_CONFIDENCE: f64 = 0.90;
const AUTO_RESOLVE_MARGIN: f64 = 0.15;
const AMBIGUOUS_CONFIDENCE: f64 = 0.70;
pub const MAX_CANDIDATES: usize = 5;

pub fn is_ticker(input: &str) -> bool {
    let trimmed = input.trim();
    !trimmed.is_empty()
        && trimmed.len() <= 6
        && trimmed
            .chars()
            .all(|character| character.is_ascii_uppercase() || character == '.' || character == '-')
        && trimmed.chars().any(|character| character.is_ascii_uppercase())
}

pub fn normalize(input: &str) -> String {
    let mut normalized = String::with_capacity(input.len());
    for character in input.chars().flat_map(|character| character.to_lowercase()) {
        if character.is_ascii_alphanumeric() {
            normalized.push(character);
        } else {
            normalized.push(' ');
        }
    }

    normalized
        .split_whitespace()
        .filter(|token| !is_company_suffix(token))
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn candidates(query: &str, companies: &[Company]) -> Vec<ResolutionCandidate> {
    let normalized_query = normalize(query);
    if normalized_query.is_empty() {
        return Vec::new();
    }

    let query_tokens = tokenize(&normalized_query);
    let mut candidates = companies
        .iter()
        .filter_map(|company| {
            let symbol_normalized = company.symbol.to_ascii_lowercase();
            let name_normalized = normalize(&company.company_name);

            let (confidence, strategy) = if name_normalized == normalized_query {
                (1.0, ResolutionStrategy::ExactName)
            } else if symbol_normalized == normalized_query {
                (1.0, ResolutionStrategy::ExactSymbol)
            } else if name_normalized.starts_with(&normalized_query) {
                (0.92, ResolutionStrategy::Prefix)
            } else {
                let company_tokens = tokenize(&name_normalized);
                let confidence = token_confidence(&query_tokens, &company_tokens);
                (confidence, ResolutionStrategy::TokenFuzzy)
            };

            (confidence >= AMBIGUOUS_CONFIDENCE).then(|| ResolutionCandidate {
                symbol: company.symbol.clone(),
                company_name: company.company_name.clone(),
                confidence,
                strategy,
            })
        })
        .collect::<Vec<_>>();

    candidates.sort_by(|left, right| {
        right
            .confidence
            .partial_cmp(&left.confidence)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| left.symbol.cmp(&right.symbol))
    });
    candidates.truncate(MAX_CANDIDATES);
    candidates
}

pub fn auto_resolves(candidates: &[ResolutionCandidate]) -> bool {
    if let Some(top) = candidates.first() {
        let margin = candidates
            .get(1)
            .map(|runner_up| top.confidence - runner_up.confidence)
            .unwrap_or(1.0);
        top.confidence >= AUTO_RESOLVE_CONFIDENCE && margin >= AUTO_RESOLVE_MARGIN
    } else {
        false
    }
}

fn tokenize(input: &str) -> Vec<&str> {
    input.split_whitespace().collect()
}

fn token_confidence(query_tokens: &[&str], company_tokens: &[&str]) -> f64 {
    if query_tokens.is_empty() || company_tokens.is_empty() {
        return 0.0;
    }

    let matches = query_tokens
        .iter()
        .filter(|query_token| {
            company_tokens.iter().any(|company_token| {
                company_token == *query_token
                    || company_token.starts_with(**query_token)
                    || query_token.starts_with(*company_token)
            })
        })
        .count();

    let coverage = matches as f64 / query_tokens.len() as f64;
    if coverage >= 1.0 {
        0.86
    } else if coverage >= 0.5 {
        0.72
    } else {
        0.0
    }
}

fn is_company_suffix(token: &str) -> bool {
    matches!(
        token,
        "inc"
            | "incorporated"
            | "corp"
            | "corporation"
            | "co"
            | "company"
            | "ltd"
            | "limited"
            | "plc"
            | "class"
            | "common"
            | "stock"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_only_uppercase_ticker_inputs() {
        assert!(is_ticker("AAPL"));
        assert!(is_ticker("BRK.B"));
        assert!(!is_ticker("apple"));
        assert!(!is_ticker("Apple"));
    }

    #[test]
    fn normalizes_company_suffixes_and_punctuation() {
        assert_eq!(normalize("Apple, Inc."), "apple");
        assert_eq!(normalize("Meta Platforms, Inc."), "meta platforms");
    }

    #[test]
    fn scores_prefix_candidate_high_enough_to_auto_resolve() {
        let companies = vec![Company {
            symbol: "AAPL".to_string(),
            company_name: "Apple Inc.".to_string(),
            cik: 320193,
            exchange: None,
        }];
        let result = candidates("apple", &companies);
        assert_eq!(result[0].symbol, "AAPL");
        assert!(auto_resolves(&result));
    }
}
