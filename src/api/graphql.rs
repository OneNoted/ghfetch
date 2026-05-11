use super::client::GhClient;
use anyhow::Result;
use serde::Deserialize;

const CONTRIBUTIONS_QUERY: &str = r#"
query($login: String!) {
  user(login: $login) {
    contributionsCollection {
      totalCommitContributions
      totalPullRequestContributions
      totalIssueContributions
      totalPullRequestReviewContributions
      contributionCalendar {
        totalContributions
        weeks {
          contributionDays {
            date
            contributionCount
          }
        }
      }
    }
  }
}
"#;

const REPOS_WITH_LANGUAGES_QUERY: &str = r#"
query($login: String!, $first: Int!, $after: String) {
  user(login: $login) {
    repositories(first: $first, after: $after, ownerAffiliations: OWNER, orderBy: {field: STARGAZERS, direction: DESC}) {
      totalCount
      pageInfo {
        hasNextPage
        endCursor
      }
      nodes {
        name
        nameWithOwner
        description
        isFork
        isPrivate
        stargazerCount
        forkCount
        updatedAt
        diskUsage
        primaryLanguage {
          name
        }
        languages(first: 10, orderBy: {field: SIZE, direction: DESC}) {
          edges {
            size
            node {
              name
            }
          }
        }
      }
    }
  }
}
"#;

const ORG_REPOS_QUERY: &str = r#"
query($login: String!, $first: Int!, $after: String) {
  organization(login: $login) {
    repositories(first: $first, after: $after, orderBy: {field: STARGAZERS, direction: DESC}) {
      totalCount
      pageInfo {
        hasNextPage
        endCursor
      }
      nodes {
        name
        nameWithOwner
        description
        isFork
        isPrivate
        stargazerCount
        forkCount
        updatedAt
        diskUsage
        primaryLanguage {
          name
        }
        languages(first: 10, orderBy: {field: SIZE, direction: DESC}) {
          edges {
            size
            node {
              name
            }
          }
        }
      }
    }
  }
}
"#;

#[derive(Debug, Deserialize)]
pub struct ContributionsResponse {
    pub total_commits: u32,
    pub total_prs: u32,
    pub total_issues: u32,
    pub total_reviews: u32,
    pub total_contributions: u32,
    pub days: Vec<ContributionDay>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ContributionDay {
    pub date: String,
    pub count: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct GraphQLRepo {
    pub name: String,
    pub name_with_owner: String,
    pub description: Option<String>,
    pub is_fork: bool,
    pub is_private: bool,
    pub stargazer_count: u32,
    pub fork_count: u32,
    pub disk_usage: Option<u32>,
    pub languages: Vec<(String, u64)>,
}

fn parse_repo_nodes(nodes: &serde_json::Value) -> Vec<GraphQLRepo> {
    let mut repos = Vec::new();
    if let Some(nodes) = nodes.as_array() {
        for node in nodes {
            let mut languages = Vec::new();
            if let Some(edges) = node["languages"]["edges"].as_array() {
                for edge in edges {
                    let name = edge["node"]["name"]
                        .as_str()
                        .unwrap_or_default()
                        .to_string();
                    let size = edge["size"].as_u64().unwrap_or(0);
                    languages.push((name, size));
                }
            }

            repos.push(GraphQLRepo {
                name: node["name"].as_str().unwrap_or_default().to_string(),
                name_with_owner: node["nameWithOwner"]
                    .as_str()
                    .unwrap_or_default()
                    .to_string(),
                description: node["description"].as_str().map(|s| s.to_string()),
                is_fork: node["isFork"].as_bool().unwrap_or(false),
                is_private: node["isPrivate"].as_bool().unwrap_or(false),
                stargazer_count: node["stargazerCount"].as_u64().unwrap_or(0) as u32,
                fork_count: node["forkCount"].as_u64().unwrap_or(0) as u32,
                disk_usage: node["diskUsage"].as_u64().map(|v| v as u32),
                languages,
            });
        }
    }
    repos
}

impl GhClient {
    pub async fn get_contributions(&self, login: &str) -> Result<ContributionsResponse> {
        let variables = serde_json::json!({ "login": login });
        let data = self.graphql(CONTRIBUTIONS_QUERY, &variables).await?;

        let collection = &data["data"]["user"]["contributionsCollection"];
        let calendar = &collection["contributionCalendar"];

        let mut days = Vec::new();
        if let Some(weeks) = calendar["weeks"].as_array() {
            for week in weeks {
                if let Some(contribution_days) = week["contributionDays"].as_array() {
                    for day in contribution_days {
                        days.push(ContributionDay {
                            date: day["date"].as_str().unwrap_or_default().to_string(),
                            count: day["contributionCount"].as_u64().unwrap_or(0) as u32,
                        });
                    }
                }
            }
        }

        Ok(ContributionsResponse {
            total_commits: collection["totalCommitContributions"].as_u64().unwrap_or(0) as u32,
            total_prs: collection["totalPullRequestContributions"]
                .as_u64()
                .unwrap_or(0) as u32,
            total_issues: collection["totalIssueContributions"].as_u64().unwrap_or(0) as u32,
            total_reviews: collection["totalPullRequestReviewContributions"]
                .as_u64()
                .unwrap_or(0) as u32,
            total_contributions: calendar["totalContributions"].as_u64().unwrap_or(0) as u32,
            days,
        })
    }

    pub async fn get_repos_with_languages(&self, login: &str) -> Result<Vec<GraphQLRepo>> {
        let mut all_repos = Vec::new();
        let mut cursor: Option<String> = None;

        loop {
            let variables = serde_json::json!({
                "login": login,
                "first": 100,
                "after": cursor,
            });

            let data = self.graphql(REPOS_WITH_LANGUAGES_QUERY, &variables).await?;

            let repos_data = &data["data"]["user"]["repositories"];
            let page_info = &repos_data["pageInfo"];
            let has_next = page_info["hasNextPage"].as_bool().unwrap_or(false);

            all_repos.extend(parse_repo_nodes(&repos_data["nodes"]));

            if has_next {
                cursor = page_info["endCursor"].as_str().map(|s| s.to_string());
            } else {
                break;
            }
        }

        Ok(all_repos)
    }

    pub async fn get_org_repos_graphql(&self, login: &str) -> Result<Vec<GraphQLRepo>> {
        let mut all_repos = Vec::new();
        let mut cursor: Option<String> = None;

        loop {
            let variables = serde_json::json!({
                "login": login,
                "first": 100,
                "after": cursor,
            });

            let data = self.graphql(ORG_REPOS_QUERY, &variables).await?;

            let repos_data = &data["data"]["organization"]["repositories"];
            let page_info = &repos_data["pageInfo"];
            let has_next = page_info["hasNextPage"].as_bool().unwrap_or(false);

            all_repos.extend(parse_repo_nodes(&repos_data["nodes"]));

            if has_next {
                cursor = page_info["endCursor"].as_str().map(|s| s.to_string());
            } else {
                break;
            }
        }

        Ok(all_repos)
    }
}
