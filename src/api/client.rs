use anyhow::Result;
use reqwest::header::{ACCEPT, AUTHORIZATION, USER_AGENT};

pub struct GhClient {
    pub http: reqwest::Client,
    pub token: Option<String>,
    pub verbose: bool,
}

impl GhClient {
    pub fn new(token: Option<String>, verbose: bool) -> Result<Self> {
        let http = reqwest::Client::builder().user_agent("ghfetch").build()?;
        Ok(Self {
            http,
            token,
            verbose,
        })
    }

    pub fn is_authenticated(&self) -> bool {
        self.token.is_some()
    }

    pub fn auth_headers(&self) -> Vec<(reqwest::header::HeaderName, String)> {
        let mut headers = vec![
            (ACCEPT, "application/vnd.github+json".to_string()),
            (USER_AGENT, "ghfetch".to_string()),
        ];
        if let Some(ref t) = self.token {
            headers.push((AUTHORIZATION, format!("Bearer {t}")));
        }
        headers
    }

    pub async fn rest_get(&self, url: &str) -> Result<reqwest::Response> {
        let mut req = self.http.get(url);
        for (k, v) in self.auth_headers() {
            req = req.header(k, v);
        }
        let resp = req.send().await?;
        if self.verbose {
            eprintln!("[api] GET {} → {}", url, resp.status());
        }
        Ok(resp)
    }

    pub async fn graphql(
        &self,
        query: &str,
        variables: &serde_json::Value,
    ) -> Result<serde_json::Value> {
        let token = self
            .token
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("GraphQL requires authentication"))?;

        let body = serde_json::json!({
            "query": query,
            "variables": variables,
        });

        let resp = self
            .http
            .post("https://api.github.com/graphql")
            .header(AUTHORIZATION, format!("Bearer {token}"))
            .header(USER_AGENT, "ghfetch")
            .json(&body)
            .send()
            .await?;

        if self.verbose {
            eprintln!("[api] POST /graphql → {}", resp.status());
        }

        let data: serde_json::Value = resp.json().await?;
        if let Some(errors) = data.get("errors") {
            anyhow::bail!("GraphQL errors: {errors}");
        }
        Ok(data)
    }
}
