use crate::api::client::GhClient;
use crate::cli::UserOpts;
use crate::data::contributions::StreakInfo;
use crate::data::languages::LanguageBreakdown;
use crate::data::stats::AggregateStats;
use anyhow::Result;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Serialize)]
pub struct UserProfile {
    pub login: String,
    pub name: Option<String>,
    pub bio: Option<String>,
    pub location: Option<String>,
    pub company: Option<String>,
    pub blog: Option<String>,
    pub twitter: Option<String>,
    pub followers: u32,
    pub following: u32,
    pub public_repos: u32,
    pub public_gists: u32,
    pub joined: String,
    pub stats: AggregateStats,
    pub contributions: Option<ContributionStats>,
    pub languages: Option<LanguageBreakdown>,
    pub streaks: Option<StreakInfo>,
    pub top_repos: Option<Vec<RepoSummary>>,
    pub is_authenticated: bool,
}

#[derive(Debug, Serialize)]
pub struct ContributionStats {
    pub total_commits: u32,
    pub total_prs: u32,
    pub total_issues: u32,
    pub total_reviews: u32,
    pub total_contributions: u32,
}

#[derive(Debug, Serialize)]
pub struct RepoSummary {
    pub name: String,
    pub stars: u32,
    pub forks: u32,
    pub language: Option<String>,
    pub description: Option<String>,
    pub is_private: bool,
}

pub async fn fetch_user_profile(
    client: &GhClient,
    username: &str,
    opts: &UserOpts,
) -> Result<UserProfile> {
    let is_auth = client.is_authenticated();

    // Fetch user profile (always needed)
    let user = client.get_user(username).await?;

    // Parallel fetch: contributions + repos/languages
    let (contributions, repos_result) = if is_auth {
        let contribs_fut = client.get_contributions(username);
        let repos_fut = client.get_repos_with_languages(username);
        let (c, r) = tokio::try_join!(contribs_fut, repos_fut)?;
        (Some(c), ReposResult::GraphQL(r))
    } else {
        eprintln!("Warning: No authentication token found. Contributions and streaks unavailable.");
        eprintln!("         Rate limit: 60 requests/hour. Use --token or set GITHUB_TOKEN.");
        let repos = client.get_user_repos(username, 100).await?;
        (None, ReposResult::Rest(repos))
    };

    // Process repos
    let (stats, languages, top_repos) = match &repos_result {
        ReposResult::GraphQL(repos) => {
            let filtered: Vec<_> = if opts.no_forks {
                repos.iter().filter(|r| !r.is_fork).cloned().collect()
            } else {
                repos.clone()
            };

            let stats = crate::data::stats::aggregate_from_graphql(&filtered);

            let lang_map: HashMap<String, Vec<(String, u64)>> = filtered
                .iter()
                .map(|r| (r.name.clone(), r.languages.clone()))
                .collect();
            let languages =
                crate::data::languages::aggregate_languages(&lang_map, opts.effective_lang_limit());

            let top: Vec<RepoSummary> = {
                let mut sorted = filtered;
                sort_graphql_repos(&mut sorted, opts.sort_by);
                sorted
                    .into_iter()
                    .take(opts.repo_limit)
                    .map(|r| RepoSummary {
                        name: r.name,
                        stars: r.stargazer_count,
                        forks: r.fork_count,
                        language: r.languages.first().map(|(n, _)| n.clone()),
                        description: r.description,
                        is_private: r.is_private,
                    })
                    .collect()
            };

            (stats, Some(languages), Some(top))
        }
        ReposResult::Rest(repos) => {
            let filtered: Vec<_> = if opts.no_forks {
                repos.iter().filter(|r| !r.fork).cloned().collect()
            } else {
                repos.to_vec()
            };

            let stats = crate::data::stats::aggregate_from_rest(&filtered);

            // For REST, we need individual language calls — do a limited batch
            let languages = if opts.show_languages() {
                let mut lang_map: HashMap<String, HashMap<String, u64>> = HashMap::new();
                // Only fetch languages for top repos to avoid rate limits
                let to_fetch: Vec<_> = filtered.iter().take(30).collect();
                for repo in to_fetch {
                    if let Ok(langs) = client.get_repo_languages(username, &repo.name).await {
                        lang_map.insert(repo.name.clone(), langs);
                    }
                }
                Some(crate::data::languages::aggregate_from_rest(
                    &lang_map,
                    opts.effective_lang_limit(),
                ))
            } else {
                None
            };

            let top: Vec<RepoSummary> = {
                let mut sorted = filtered;
                sort_rest_repos(&mut sorted, opts.sort_by);
                sorted
                    .into_iter()
                    .take(opts.repo_limit)
                    .map(|r| RepoSummary {
                        name: r.name,
                        stars: r.stargazers_count,
                        forks: r.forks_count,
                        language: r.language,
                        description: r.description,
                        is_private: false, // REST /users/:user/repos doesn't include private for other users
                    })
                    .collect()
            };

            (stats, languages, Some(top))
        }
    };

    // Process contributions
    let (contrib_stats, streaks) = match contributions {
        Some(c) => {
            let streaks = crate::data::contributions::calculate_streaks(&c.days);
            let stats = ContributionStats {
                total_commits: c.total_commits,
                total_prs: c.total_prs,
                total_issues: c.total_issues,
                total_reviews: c.total_reviews,
                total_contributions: c.total_contributions,
            };
            (Some(stats), Some(streaks))
        }
        None => (None, None),
    };

    Ok(UserProfile {
        login: user.login,
        name: user.name,
        bio: user.bio,
        location: user.location,
        company: user.company,
        blog: user.blog,
        twitter: user.twitter_username,
        followers: user.followers,
        following: user.following,
        public_repos: user.public_repos,
        public_gists: user.public_gists,
        joined: user.created_at.format("%b %Y").to_string(),
        stats,
        contributions: contrib_stats,
        languages,
        streaks,
        top_repos: if opts.show_repos() { top_repos } else { None },
        is_authenticated: is_auth,
    })
}

enum ReposResult {
    GraphQL(Vec<crate::api::graphql::GraphQLRepo>),
    Rest(Vec<crate::api::types::GitHubRepo>),
}

fn sort_graphql_repos(repos: &mut [crate::api::graphql::GraphQLRepo], sort_by: crate::cli::SortBy) {
    use crate::cli::SortBy;
    match sort_by {
        SortBy::Stars => repos.sort_by(|a, b| b.stargazer_count.cmp(&a.stargazer_count)),
        SortBy::Forks => repos.sort_by(|a, b| b.fork_count.cmp(&a.fork_count)),
        SortBy::Size => {
            repos.sort_by(|a, b| b.disk_usage.unwrap_or(0).cmp(&a.disk_usage.unwrap_or(0)))
        }
        SortBy::Name => repos.sort_by(|a, b| a.name.cmp(&b.name)),
        SortBy::Updated => {} // Already sorted by GitHub
    }
}

fn sort_rest_repos(repos: &mut [crate::api::types::GitHubRepo], sort_by: crate::cli::SortBy) {
    use crate::cli::SortBy;
    match sort_by {
        SortBy::Stars => repos.sort_by(|a, b| b.stargazers_count.cmp(&a.stargazers_count)),
        SortBy::Forks => repos.sort_by(|a, b| b.forks_count.cmp(&a.forks_count)),
        SortBy::Size => repos.sort_by(|a, b| b.size.cmp(&a.size)),
        SortBy::Name => repos.sort_by(|a, b| a.name.cmp(&b.name)),
        SortBy::Updated => {} // Already sorted by GitHub
    }
}
