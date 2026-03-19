use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// REST: GET /users/{username}
#[derive(Debug, Deserialize, Serialize)]
pub struct GitHubUser {
    pub login: String,
    pub name: Option<String>,
    pub bio: Option<String>,
    pub location: Option<String>,
    pub company: Option<String>,
    pub blog: Option<String>,
    pub twitter_username: Option<String>,
    pub public_repos: u32,
    pub public_gists: u32,
    pub followers: u32,
    pub following: u32,
    pub created_at: DateTime<Utc>,
    pub avatar_url: String,
}

/// REST: GET /users/{username}/repos
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GitHubRepo {
    pub name: String,
    pub full_name: String,
    pub description: Option<String>,
    pub fork: bool,
    pub stargazers_count: u32,
    pub forks_count: u32,
    pub open_issues_count: u32,
    pub language: Option<String>,
    pub size: u32,
    pub updated_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub pushed_at: Option<DateTime<Utc>>,
    pub license: Option<RepoLicense>,
    pub topics: Option<Vec<String>>,
    pub archived: bool,
    pub html_url: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RepoLicense {
    pub spdx_id: Option<String>,
    pub name: String,
}

/// REST: GET /repos/{owner}/{repo} — single repo detail
#[derive(Debug, Deserialize, Serialize)]
pub struct GitHubRepoDetail {
    pub name: String,
    pub full_name: String,
    pub description: Option<String>,
    pub fork: bool,
    pub stargazers_count: u32,
    pub forks_count: u32,
    pub open_issues_count: u32,
    pub watchers_count: u32,
    pub subscribers_count: u32,
    pub language: Option<String>,
    pub size: u32,
    pub default_branch: String,
    pub updated_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub pushed_at: Option<DateTime<Utc>>,
    pub license: Option<RepoLicense>,
    pub topics: Option<Vec<String>>,
    pub archived: bool,
    pub html_url: String,
    pub owner: RepoOwner,
    pub network_count: Option<u32>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RepoOwner {
    pub login: String,
    pub avatar_url: String,
}

/// REST: GET /orgs/{org}
#[derive(Debug, Deserialize, Serialize)]
pub struct GitHubOrg {
    pub login: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub location: Option<String>,
    pub blog: Option<String>,
    pub twitter_username: Option<String>,
    pub public_repos: u32,
    pub public_gists: u32,
    pub followers: u32,
    pub following: u32,
    pub created_at: DateTime<Utc>,
    pub avatar_url: String,
    pub html_url: String,
}

/// REST: GET /repos/{owner}/{repo}/languages → { "Rust": 123456, "Python": 789 }
pub type LanguageBytes = HashMap<String, u64>;
