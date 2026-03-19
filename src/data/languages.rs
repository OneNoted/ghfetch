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
}

/// Aggregate language bytes from a map of {repo_name: {lang: bytes}}.
pub fn aggregate_languages(
    repo_languages: &HashMap<String, Vec<(String, u64)>>,
    limit: usize,
) -> LanguageBreakdown {
    let mut totals: HashMap<String, u64> = HashMap::new();

    for langs in repo_languages.values() {
        for (lang, bytes) in langs {
            *totals.entry(lang.clone()).or_default() += bytes;
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
            LanguageEntry {
                name,
                bytes,
                percentage,
            }
        })
        .collect();

    entries.sort_by(|a, b| b.bytes.cmp(&a.bytes));
    entries.truncate(limit);

    // Recalculate percentages for the truncated set
    let shown_bytes: u64 = entries.iter().map(|e| e.bytes).sum();
    if shown_bytes > 0 {
        for entry in &mut entries {
            entry.percentage = (entry.bytes as f64 / shown_bytes as f64) * 100.0;
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
