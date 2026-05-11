use crate::api::client::GhClient;
use crate::api::graphql::GraphQLRepo;
use crate::api::types::GitHubRepo;
use crate::cli::{BreakdownBy, BreakdownOpts};
use anyhow::Result;
use serde::Serialize;
use std::cmp::Reverse;

#[derive(Debug, Serialize)]
pub struct BreakdownProfile {
    pub login: String,
    pub by: String,
    pub total_bytes: u64,
    pub repo_count: usize,
    pub is_authenticated: bool,
    pub language_groups: Vec<LanguageGroup>,
    pub repo_groups: Vec<RepoGroup>,
}

#[derive(Debug, Serialize)]
pub struct LanguageGroup {
    pub name: String,
    pub bytes: u64,
    pub percentage: f64,
    pub estimated_lines: u64,
    pub repo_count: usize,
    pub repos: Vec<RepoContribution>,
}

#[derive(Debug, Serialize)]
pub struct RepoContribution {
    pub name: String,
    pub bytes: u64,
    pub percentage_of_language: f64,
    pub estimated_lines: u64,
    pub stars: u32,
    pub forks: u32,
    pub is_fork: bool,
    pub is_private: bool,
}

#[derive(Debug, Serialize)]
pub struct RepoGroup {
    pub name: String,
    pub bytes: u64,
    pub percentage: f64,
    pub estimated_lines: u64,
    pub stars: u32,
    pub forks: u32,
    pub is_fork: bool,
    pub is_private: bool,
    pub languages: Vec<LanguageContribution>,
}

#[derive(Debug, Serialize)]
pub struct LanguageContribution {
    pub name: String,
    pub bytes: u64,
    pub percentage_of_repo: f64,
    pub estimated_lines: u64,
}

#[derive(Debug, Clone)]
struct RepoLanguageData {
    name: String,
    languages: Vec<(String, u64)>,
    stars: u32,
    forks: u32,
    is_fork: bool,
    is_private: bool,
}

pub async fn fetch_breakdown_profile(
    client: &GhClient,
    username: &str,
    opts: &BreakdownOpts,
) -> Result<BreakdownProfile> {
    let is_authenticated = client.is_authenticated();
    let repos = if is_authenticated {
        let repos = client.get_repos_with_languages(username).await?;
        repos
            .into_iter()
            .filter(|repo| !opts.no_forks || !repo.is_fork)
            .map(repo_data_from_graphql)
            .collect()
    } else {
        eprintln!(
            "Warning: No authentication token found. Breakdown uses REST calls and public data only."
        );
        eprintln!("         Rate limit: 60 requests/hour. Use --token or set GITHUB_TOKEN.");
        let repos = client.get_user_repos(username, 100).await?;
        let filtered: Vec<_> = repos
            .into_iter()
            .filter(|repo| !opts.no_forks || !repo.fork)
            .collect();
        let mut data = Vec::with_capacity(filtered.len());
        for repo in filtered {
            let languages = client
                .get_repo_languages(username, &repo.name)
                .await
                .map(|langs| langs.into_iter().collect())
                .unwrap_or_default();
            data.push(repo_data_from_rest(repo, languages));
        }
        data
    };

    Ok(build_breakdown(username, opts, repos, is_authenticated))
}

fn build_breakdown(
    username: &str,
    opts: &BreakdownOpts,
    repos: Vec<RepoLanguageData>,
    is_authenticated: bool,
) -> BreakdownProfile {
    let filtered_repos: Vec<_> = repos
        .into_iter()
        .filter(|repo| matches_repo_filter(repo, opts.repo.as_deref()))
        .map(|mut repo| {
            if let Some(language) = opts.language.as_deref() {
                repo.languages
                    .retain(|(name, _)| name.eq_ignore_ascii_case(language));
            }
            repo
        })
        .filter(|repo| !repo.languages.is_empty())
        .collect();

    let total_bytes = filtered_repos
        .iter()
        .flat_map(|repo| repo.languages.iter().map(|(_, bytes)| bytes))
        .sum();

    let mut language_groups = build_language_groups(&filtered_repos, total_bytes, opts.repo_limit);
    let mut repo_groups = build_repo_groups(&filtered_repos, total_bytes, opts.repo_limit);

    apply_limit(&mut language_groups, opts.limit);
    apply_limit(&mut repo_groups, opts.limit);

    BreakdownProfile {
        login: username.to_string(),
        by: match opts.by {
            BreakdownBy::Language => "language".to_string(),
            BreakdownBy::Repo => "repo".to_string(),
        },
        total_bytes,
        repo_count: filtered_repos.len(),
        is_authenticated,
        language_groups,
        repo_groups,
    }
}

fn build_language_groups(
    repos: &[RepoLanguageData],
    total_bytes: u64,
    repo_limit: usize,
) -> Vec<LanguageGroup> {
    let mut groups: Vec<LanguageGroup> = Vec::new();

    for repo in repos {
        for (language, bytes) in &repo.languages {
            if let Some(group) = groups.iter_mut().find(|group| group.name == *language) {
                group.bytes += bytes;
                group.repo_count += 1;
                group.repos.push(repo_contribution(repo, *bytes, 0.0));
            } else {
                groups.push(LanguageGroup {
                    name: language.clone(),
                    bytes: *bytes,
                    percentage: 0.0,
                    estimated_lines: 0,
                    repo_count: 1,
                    repos: vec![repo_contribution(repo, *bytes, 0.0)],
                });
            }
        }
    }

    for group in &mut groups {
        group.percentage = percentage(group.bytes, total_bytes);
        group.estimated_lines = crate::data::languages::estimate_lines(group.bytes);
        group.repos.sort_by_key(|repo| Reverse(repo.bytes));
        for repo in &mut group.repos {
            repo.percentage_of_language = percentage(repo.bytes, group.bytes);
        }
        apply_limit(&mut group.repos, repo_limit);
    }

    groups.sort_by(|a, b| b.bytes.cmp(&a.bytes).then_with(|| a.name.cmp(&b.name)));
    groups
}

fn build_repo_groups(
    repos: &[RepoLanguageData],
    total_bytes: u64,
    language_limit: usize,
) -> Vec<RepoGroup> {
    let mut groups: Vec<RepoGroup> = repos
        .iter()
        .map(|repo| {
            let repo_bytes: u64 = repo.languages.iter().map(|(_, bytes)| bytes).sum();
            let mut languages: Vec<LanguageContribution> = repo
                .languages
                .iter()
                .map(|(name, bytes)| LanguageContribution {
                    name: name.clone(),
                    bytes: *bytes,
                    percentage_of_repo: percentage(*bytes, repo_bytes),
                    estimated_lines: crate::data::languages::estimate_lines(*bytes),
                })
                .collect();
            languages.sort_by(|a, b| b.bytes.cmp(&a.bytes).then_with(|| a.name.cmp(&b.name)));
            apply_limit(&mut languages, language_limit);

            RepoGroup {
                name: repo.name.clone(),
                bytes: repo_bytes,
                percentage: percentage(repo_bytes, total_bytes),
                estimated_lines: crate::data::languages::estimate_lines(repo_bytes),
                stars: repo.stars,
                forks: repo.forks,
                is_fork: repo.is_fork,
                is_private: repo.is_private,
                languages,
            }
        })
        .collect();

    groups.sort_by(|a, b| b.bytes.cmp(&a.bytes).then_with(|| a.name.cmp(&b.name)));
    groups
}

fn repo_contribution(
    repo: &RepoLanguageData,
    bytes: u64,
    percentage_of_language: f64,
) -> RepoContribution {
    RepoContribution {
        name: repo.name.clone(),
        bytes,
        percentage_of_language,
        estimated_lines: crate::data::languages::estimate_lines(bytes),
        stars: repo.stars,
        forks: repo.forks,
        is_fork: repo.is_fork,
        is_private: repo.is_private,
    }
}

fn repo_data_from_graphql(repo: GraphQLRepo) -> RepoLanguageData {
    RepoLanguageData {
        name: if repo.name_with_owner.is_empty() {
            repo.name
        } else {
            repo.name_with_owner
        },
        languages: repo.languages,
        stars: repo.stargazer_count,
        forks: repo.fork_count,
        is_fork: repo.is_fork,
        is_private: repo.is_private,
    }
}

fn repo_data_from_rest(repo: GitHubRepo, languages: Vec<(String, u64)>) -> RepoLanguageData {
    RepoLanguageData {
        name: repo.full_name,
        languages,
        stars: repo.stargazers_count,
        forks: repo.forks_count,
        is_fork: repo.fork,
        is_private: false,
    }
}

fn matches_repo_filter(repo: &RepoLanguageData, filter: Option<&str>) -> bool {
    match filter {
        Some(filter) => {
            repo.name.eq_ignore_ascii_case(filter)
                || repo
                    .name
                    .rsplit('/')
                    .next()
                    .is_some_and(|name| name.eq_ignore_ascii_case(filter))
        }
        None => true,
    }
}

fn apply_limit<T>(items: &mut Vec<T>, limit: usize) {
    if limit > 0 && items.len() > limit {
        items.truncate(limit);
    }
}

fn percentage(bytes: u64, total: u64) -> f64 {
    if total > 0 {
        (bytes as f64 / total as f64) * 100.0
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn opts() -> BreakdownOpts {
        BreakdownOpts {
            by: BreakdownBy::Language,
            language: None,
            repo: None,
            limit: 10,
            repo_limit: 10,
            no_forks: false,
        }
    }

    fn repos() -> Vec<RepoLanguageData> {
        vec![
            RepoLanguageData {
                name: "owner/repo-a".to_string(),
                languages: vec![("Rust".to_string(), 400), ("TypeScript".to_string(), 100)],
                stars: 10,
                forks: 2,
                is_fork: false,
                is_private: false,
            },
            RepoLanguageData {
                name: "owner/repo-b".to_string(),
                languages: vec![("Rust".to_string(), 200), ("Python".to_string(), 300)],
                stars: 5,
                forks: 1,
                is_fork: false,
                is_private: false,
            },
        ]
    }

    #[test]
    fn groups_repositories_under_languages() {
        let profile = build_breakdown("owner", &opts(), repos(), true);

        assert_eq!(profile.total_bytes, 1000);
        assert_eq!(profile.language_groups[0].name, "Rust");
        assert_eq!(profile.language_groups[0].bytes, 600);
        assert_eq!(profile.language_groups[0].repo_count, 2);
        assert_eq!(profile.language_groups[0].repos[0].name, "owner/repo-a");
        assert_eq!(
            profile.language_groups[0].repos[0].percentage_of_language,
            400.0 / 600.0 * 100.0
        );
    }

    #[test]
    fn filters_to_single_language_and_repo_name() {
        let mut opts = opts();
        opts.language = Some("rust".to_string());
        opts.repo = Some("repo-b".to_string());

        let profile = build_breakdown("owner", &opts, repos(), true);

        assert_eq!(profile.total_bytes, 200);
        assert_eq!(profile.repo_count, 1);
        assert_eq!(profile.language_groups.len(), 1);
        assert_eq!(profile.repo_groups[0].name, "owner/repo-b");
    }
}
