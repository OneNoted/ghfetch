use anyhow::Result;

/// Resolve the GitHub token from multiple sources in priority order:
/// 1. --token flag (passed in)
/// 2. GITHUB_TOKEN env (handled by clap)
/// 3. GH_TOKEN env
/// 4. `gh auth token` subprocess
/// 5. None (unauthenticated)
pub async fn resolve_token(flag_token: Option<String>) -> Option<String> {
    if let Some(t) = flag_token
        && !t.is_empty() {
            return Some(t);
        }

    if let Ok(t) = std::env::var("GH_TOKEN")
        && !t.is_empty() {
            return Some(t);
        }

    if let Ok(output) = gh_auth_token().await
        && !output.is_empty() {
            return Some(output);
        }

    None
}

async fn gh_auth_token() -> Result<String> {
    let output = tokio::process::Command::new("gh")
        .args(["auth", "token"])
        .output()
        .await?;

    if output.status.success() {
        Ok(String::from_utf8(output.stdout)?.trim().to_string())
    } else {
        anyhow::bail!("gh auth token failed")
    }
}
