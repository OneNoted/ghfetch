use serde::Serialize;
use std::cmp::Reverse;
use std::collections::HashMap;

#[derive(Debug, Serialize, Clone)]
pub struct LanguageBreakdown {
    pub entries: Vec<LanguageEntry>,
    pub total_bytes: u64,
}

#[derive(Debug, Serialize, Clone)]
pub struct LanguageEntry {
    pub name: String,
    pub bytes: u64,
    pub percentage: f64,
    pub repo_count: u32,
}

/// Aggregate language bytes from a map of {repo_name: {lang: bytes}}.
/// When `limit` is 0, return all languages without truncation.
pub fn aggregate_languages(
    repo_languages: &HashMap<String, Vec<(String, u64)>>,
    limit: usize,
) -> LanguageBreakdown {
    let mut totals: HashMap<String, u64> = HashMap::new();
    let mut repo_counts: HashMap<String, u32> = HashMap::new();

    for langs in repo_languages.values() {
        for (lang, bytes) in langs {
            *totals.entry(lang.clone()).or_default() += bytes;
            *repo_counts.entry(lang.clone()).or_default() += 1;
        }
    }

    let total_bytes: u64 = totals.values().sum();

    let mut entries: Vec<LanguageEntry> = totals
        .into_iter()
        .map(|(name, bytes)| {
            let percentage = if total_bytes > 0 {
                (bytes as f64 / total_bytes as f64) * 100.0
            } else {
                0.0
            };
            let repo_count = repo_counts.get(&name).copied().unwrap_or(0);
            LanguageEntry {
                name,
                bytes,
                percentage,
                repo_count,
            }
        })
        .collect();

    entries.sort_by_key(|entry| Reverse(entry.bytes));

    if limit > 0 && entries.len() > limit {
        let hidden = entries.split_off(limit);
        let hidden_bytes: u64 = hidden.iter().map(|entry| entry.bytes).sum();

        if hidden_bytes > 0 {
            // Keep the card totals honest by surfacing truncated languages as a single tail bucket.
            entries.push(LanguageEntry {
                name: "Other".to_string(),
                bytes: hidden_bytes,
                percentage: (hidden_bytes as f64 / total_bytes as f64) * 100.0,
                repo_count: hidden.iter().map(|entry| entry.repo_count).sum(),
            });
        }
    }

    LanguageBreakdown {
        entries,
        total_bytes,
    }
}

/// Aggregate from REST LanguageBytes maps.
pub fn aggregate_from_rest(
    repo_languages: &HashMap<String, HashMap<String, u64>>,
    limit: usize,
) -> LanguageBreakdown {
    let converted: HashMap<String, Vec<(String, u64)>> = repo_languages
        .iter()
        .map(|(repo, langs)| {
            let vec: Vec<(String, u64)> = langs.iter().map(|(k, v)| (k.clone(), *v)).collect();
            (repo.clone(), vec)
        })
        .collect();
    aggregate_languages(&converted, limit)
}

/// Format bytes into a human-readable string (B, KB, MB, GB).
pub fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * KB;
    const GB: u64 = 1024 * MB;

    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{bytes} B")
    }
}

/// Estimate lines of code from byte count.
/// Uses ~40 bytes/line as a rough cross-language average.
pub fn estimate_lines(bytes: u64) -> u64 {
    bytes / 40
}

/// Format a line count with commas and "~" prefix.
pub fn format_lines(bytes: u64) -> String {
    let lines = estimate_lines(bytes);
    let s = lines.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    let formatted: String = result.chars().rev().collect();
    format!("~{formatted}")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_languages() -> HashMap<String, Vec<(String, u64)>> {
        HashMap::from([
            (
                "repo-a".to_string(),
                vec![
                    ("Rust".to_string(), 60),
                    ("Python".to_string(), 30),
                    ("C".to_string(), 10),
                ],
            ),
            (
                "repo-b".to_string(),
                vec![("Rust".to_string(), 40), ("Nix".to_string(), 20)],
            ),
        ])
    }

    fn approx_eq(left: f64, right: f64) {
        assert!((left - right).abs() < 0.001, "{left} != {right}");
    }

    #[test]
    fn truncated_breakdown_adds_other_bucket() {
        let breakdown = aggregate_languages(&sample_languages(), 2);

        assert_eq!(breakdown.total_bytes, 160);
        assert_eq!(breakdown.entries.len(), 3);
        assert_eq!(breakdown.entries[0].name, "Rust");
        assert_eq!(breakdown.entries[1].name, "Python");
        assert_eq!(breakdown.entries[2].name, "Other");
        assert_eq!(breakdown.entries[2].bytes, 30);
        approx_eq(breakdown.entries[0].percentage, 62.5);
        approx_eq(breakdown.entries[1].percentage, 18.75);
        approx_eq(breakdown.entries[2].percentage, 18.75);
    }

    #[test]
    fn unbounded_breakdown_keeps_individual_languages() {
        let breakdown = aggregate_languages(&sample_languages(), 0);

        assert_eq!(breakdown.entries.len(), 4);
        assert!(breakdown.entries.iter().all(|entry| entry.name != "Other"));
    }
}
