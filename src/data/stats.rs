use crate::api::graphql::GraphQLRepo;
use crate::api::types::GitHubRepo;
use serde::Serialize;

#[derive(Debug, Serialize, Clone, Default)]
pub struct AggregateStats {
    pub total_stars: u32,
    pub total_forks: u32,
    pub total_repos: u32,
    pub total_size_kb: u64,
}

pub fn aggregate_from_graphql(repos: &[GraphQLRepo]) -> AggregateStats {
    let mut stats = AggregateStats {
        total_repos: repos.len() as u32,
        ..Default::default()
    };

    for repo in repos {
        stats.total_stars += repo.stargazer_count;
        stats.total_forks += repo.fork_count;
        stats.total_size_kb += repo.disk_usage.unwrap_or(0) as u64;
    }

    stats
}

pub fn aggregate_from_rest(repos: &[GitHubRepo]) -> AggregateStats {
    let mut stats = AggregateStats {
        total_repos: repos.len() as u32,
        ..Default::default()
    };

    for repo in repos {
        stats.total_stars += repo.stargazers_count;
        stats.total_forks += repo.forks_count;
        stats.total_size_kb += repo.size as u64;
    }

    stats
}
