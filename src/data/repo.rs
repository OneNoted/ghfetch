use crate::api::client::GhClient;
use crate::cli::RepoOpts;
use crate::data::languages::LanguageBreakdown;
use anyhow::{Context, Result};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct RepoProfile {
    pub name: String,
    pub full_name: String,
    pub owner: String,
    pub description: Option<String>,
    pub stars: u32,
    pub forks: u32,
    pub watchers: u32,
    pub open_issues: u32,
    pub size_kb: u32,
    pub default_branch: String,
    pub license: Option<String>,
    pub topics: Vec<String>,
    pub archived: bool,
    pub created: String,
    pub updated: String,
    pub pushed: Option<String>,
    pub url: String,
    pub languages: Option<LanguageBreakdown>,
}

pub async fn fetch_repo_profile(
    client: &GhClient,
    repo_path: &str,
    opts: &RepoOpts,
) -> Result<RepoProfile> {
    let (owner, repo) = repo_path
        .split_once('/')
        .context("Repository must be in owner/repo format")?;

    let detail = client.get_repo_detail(owner, repo).await?;

    let languages = if opts.show_languages() {
        let lang_bytes = client.get_repo_languages(owner, repo).await?;
        let lang_map = std::collections::HashMap::from([(
            repo.to_string(),
            lang_bytes.into_iter().collect::<Vec<_>>(),
        )]);
        let limit = if opts.detailed_languages() { 0 } else { 10 };
        Some(crate::data::languages::aggregate_languages(&lang_map, limit))
    } else {
        None
    };

    Ok(RepoProfile {
        name: detail.name,
        full_name: detail.full_name,
        owner: detail.owner.login,
        description: detail.description,
        stars: detail.stargazers_count,
        forks: detail.forks_count,
        watchers: detail.subscribers_count,
        open_issues: detail.open_issues_count,
        size_kb: detail.size,
        default_branch: detail.default_branch,
        license: detail.license.map(|l| l.name),
        topics: detail.topics.unwrap_or_default(),
        archived: detail.archived,
        created: detail.created_at.format("%b %Y").to_string(),
        updated: detail.updated_at.format("%b %-d, %Y").to_string(),
        pushed: detail.pushed_at.map(|d| d.format("%b %-d, %Y").to_string()),
        url: detail.html_url,
        languages,
    })
}
