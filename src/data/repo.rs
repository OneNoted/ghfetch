use crate::api::client::GhClient;
use crate::cli::RepoOpts;
use crate::data::languages::LanguageBreakdown;
use anyhow::{Context, Result, bail};
use reqwest::Url;
use serde::Serialize;

const REPO_INPUT_HELP: &str = "Repository must be in owner/repo, GitHub URL, or GitHub SSH format";

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

fn parse_repo_path(repo_path: &str) -> Result<(String, String)> {
    let repo_path = repo_path.trim();

    if repo_path.is_empty() {
        bail!("{REPO_INPUT_HELP}");
    }

    if let Some(path) = repo_path
        .strip_prefix("git@github.com:")
        .or_else(|| repo_path.strip_prefix("git@www.github.com:"))
        .or_else(|| repo_path.strip_prefix("github.com/"))
        .or_else(|| repo_path.strip_prefix("www.github.com/"))
    {
        return parse_owner_repo(path);
    }

    if repo_path.contains("://") {
        return parse_repo_url(repo_path);
    }

    parse_owner_repo(repo_path)
}

fn parse_repo_url(repo_url: &str) -> Result<(String, String)> {
    let url = Url::parse(repo_url).context(REPO_INPUT_HELP)?;
    let host = url.host_str().context(REPO_INPUT_HELP)?;

    if host != "github.com" && host != "www.github.com" {
        bail!("Repository URL must point to github.com");
    }

    let mut segments = url
        .path_segments()
        .into_iter()
        .flatten()
        .filter(|segment| !segment.is_empty());
    let owner = segments.next().context(REPO_INPUT_HELP)?;
    let repo = segments.next().context(REPO_INPUT_HELP)?;

    Ok((owner.to_string(), strip_git_suffix(repo).to_string()))
}

fn parse_owner_repo(repo_path: &str) -> Result<(String, String)> {
    let parts = repo_path
        .trim()
        .trim_matches('/')
        .split('/')
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>();

    if parts.len() != 2 {
        bail!("{REPO_INPUT_HELP}");
    }

    let owner = parts[0];
    let repo = strip_git_suffix(parts[1]);

    if owner.is_empty() || repo.is_empty() {
        bail!("{REPO_INPUT_HELP}");
    }

    Ok((owner.to_string(), repo.to_string()))
}

fn strip_git_suffix(repo: &str) -> &str {
    repo.strip_suffix(".git").unwrap_or(repo)
}

pub async fn fetch_repo_profile(
    client: &GhClient,
    repo_path: &str,
    opts: &RepoOpts,
) -> Result<RepoProfile> {
    let (owner, repo) = parse_repo_path(repo_path)?;

    let detail = client.get_repo_detail(&owner, &repo).await?;

    let languages = if opts.show_languages() {
        let lang_bytes = client.get_repo_languages(&owner, &repo).await?;
        let lang_map = std::collections::HashMap::from([(
            repo.to_string(),
            lang_bytes.into_iter().collect::<Vec<_>>(),
        )]);
        let limit = if opts.detailed_languages() { 0 } else { 10 };
        Some(crate::data::languages::aggregate_languages(
            &lang_map, limit,
        ))
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

#[cfg(test)]
mod tests {
    use super::parse_repo_path;

    #[test]
    fn accepts_owner_repo_path() {
        let repo = parse_repo_path("lazyvim/lazyvim").unwrap();
        assert_eq!(repo, ("lazyvim".to_string(), "lazyvim".to_string()));
    }

    #[test]
    fn accepts_github_repo_url() {
        let repo = parse_repo_path("https://github.com/lazyvim/lazyvim").unwrap();
        assert_eq!(repo, ("lazyvim".to_string(), "lazyvim".to_string()));
    }

    #[test]
    fn accepts_github_ssh_remote() {
        let repo = parse_repo_path("git@github.com:LazyVim/LazyVim.git").unwrap();
        assert_eq!(repo, ("LazyVim".to_string(), "LazyVim".to_string()));
    }

    #[test]
    fn accepts_ssh_url_with_extra_path_segments() {
        let repo = parse_repo_path("ssh://git@github.com/lazyvim/lazyvim.git/tree/main").unwrap();
        assert_eq!(repo, ("lazyvim".to_string(), "lazyvim".to_string()));
    }

    #[test]
    fn rejects_non_github_repo_url() {
        let err = parse_repo_path("https://gitlab.com/lazyvim/lazyvim").unwrap_err();
        assert_eq!(err.to_string(), "Repository URL must point to github.com");
    }
}
