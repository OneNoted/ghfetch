use crate::api::client::GhClient;
use crate::cli::OrgOpts;
use crate::data::languages::LanguageBreakdown;
use crate::data::stats::AggregateStats;
use crate::data::user::RepoSummary;
use anyhow::Result;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Serialize)]
pub struct OrgProfile {
    pub login: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub location: Option<String>,
    pub blog: Option<String>,
    pub twitter: Option<String>,
    pub public_repos: u32,
    pub followers: u32,
    pub created: String,
    pub url: String,
    pub stats: AggregateStats,
    pub languages: Option<LanguageBreakdown>,
    pub top_repos: Option<Vec<RepoSummary>>,
}

pub async fn fetch_org_profile(
    client: &GhClient,
    orgname: &str,
    opts: &OrgOpts,
) -> Result<OrgProfile> {
    let org = client.get_org(orgname).await?;
    let repos = client.get_org_repos(orgname, 100).await?;

    let stats = crate::data::stats::aggregate_from_rest(&repos);

    let languages = if opts.show_languages() {
        let mut lang_map: HashMap<String, HashMap<String, u64>> = HashMap::new();
        let to_fetch: Vec<_> = repos.iter().take(30).collect();
        for repo in to_fetch {
            if let Ok(langs) = client.get_repo_languages(orgname, &repo.name).await {
                lang_map.insert(repo.name.clone(), langs);
            }
        }
        Some(crate::data::languages::aggregate_from_rest(&lang_map, 10))
    } else {
        None
    };

    let top_repos = if opts.show_repos() || opts.show_full_card() {
        let mut sorted = repos.clone();
        sorted.sort_by(|a, b| b.stargazers_count.cmp(&a.stargazers_count));
        Some(
            sorted
                .into_iter()
                .take(opts.repo_limit)
                .map(|r| RepoSummary {
                    name: r.name,
                    stars: r.stargazers_count,
                    forks: r.forks_count,
                    language: r.language,
                    description: r.description,
                })
                .collect(),
        )
    } else {
        None
    };

    Ok(OrgProfile {
        login: org.login,
        name: org.name,
        description: org.description,
        location: org.location,
        blog: org.blog,
        twitter: org.twitter_username,
        public_repos: org.public_repos,
        followers: org.followers,
        created: org.created_at.format("%b %Y").to_string(),
        url: org.html_url,
        stats,
        languages,
        top_repos,
    })
}
