use serde::Serialize;
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

    entries.sort_by(|a, b| b.bytes.cmp(&a.bytes));

    if limit > 0 {
        entries.truncate(limit);

        // Recalculate percentages for the truncated set
        let shown_bytes: u64 = entries.iter().map(|e| e.bytes).sum();
        if shown_bytes > 0 {
            for entry in &mut entries {
                entry.percentage = (entry.bytes as f64 / shown_bytes as f64) * 100.0;
            }
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
