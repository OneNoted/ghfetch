use super::client::GhClient;
use super::types::*;
use anyhow::{Context, Result};

const BASE: &str = "https://api.github.com";

impl GhClient {
    pub async fn get_user(&self, username: &str) -> Result<GitHubUser> {
        let resp = self
            .rest_get(&format!("{BASE}/users/{username}"))
            .await?
            .error_for_status()
            .context("Failed to fetch user profile")?;
        Ok(resp.json().await?)
    }

    pub async fn get_user_repos(&self, username: &str, per_page: u32) -> Result<Vec<GitHubRepo>> {
        let mut all_repos = Vec::new();
        let mut page = 1u32;

        loop {
            // "all" includes private repos when authenticated
            let visibility = if self.is_authenticated() {
                "all"
            } else {
                "owner"
            };
            let url = format!(
                "{BASE}/users/{username}/repos?per_page={per_page}&page={page}&sort=updated&type={visibility}"
            );
            let resp = self
                .rest_get(&url)
                .await?
                .error_for_status()
                .context("Failed to fetch user repos")?;

            let repos: Vec<GitHubRepo> = resp.json().await?;
            let done = repos.len() < per_page as usize;
            all_repos.extend(repos);

            if done {
                break;
            }
            page += 1;
        }

        Ok(all_repos)
    }

    pub async fn get_repo_languages(&self, owner: &str, repo: &str) -> Result<LanguageBytes> {
        let resp = self
            .rest_get(&format!("{BASE}/repos/{owner}/{repo}/languages"))
            .await?
            .error_for_status()
            .context("Failed to fetch repo languages")?;
        Ok(resp.json().await?)
    }

    pub async fn get_repo_detail(&self, owner: &str, repo: &str) -> Result<GitHubRepoDetail> {
        let resp = self
            .rest_get(&format!("{BASE}/repos/{owner}/{repo}"))
            .await?
            .error_for_status()
            .context("Failed to fetch repo detail")?;
        Ok(resp.json().await?)
    }

    pub async fn get_org(&self, orgname: &str) -> Result<GitHubOrg> {
        let resp = self
            .rest_get(&format!("{BASE}/orgs/{orgname}"))
            .await?
            .error_for_status()
            .context("Failed to fetch org profile")?;
        Ok(resp.json().await?)
    }

    pub async fn get_org_repos(&self, orgname: &str, per_page: u32) -> Result<Vec<GitHubRepo>> {
        let mut all_repos = Vec::new();
        let mut page = 1u32;

        loop {
            // "all" includes private repos when authenticated with org access
            let url = format!(
                "{BASE}/orgs/{orgname}/repos?per_page={per_page}&page={page}&sort=updated&type=all"
            );
            let resp = self
                .rest_get(&url)
                .await?
                .error_for_status()
                .context("Failed to fetch org repos")?;

            let repos: Vec<GitHubRepo> = resp.json().await?;
            let done = repos.len() < per_page as usize;
            all_repos.extend(repos);

            if done {
                break;
            }
            page += 1;
        }

        Ok(all_repos)
    }
}
