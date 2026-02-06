use anyhow::{Result, anyhow};

use super::score_data::{IrRankingResponse, IrScoreData, IrSubmitResponse};

/// IR (Internet Ranking) client for score submission and ranking retrieval.
pub struct IrConnection {
    /// Base URL of the IR server.
    base_url: String,
    /// HTTP client.
    client: reqwest::Client,
    /// Player authentication token.
    token: Option<String>,
}

impl IrConnection {
    /// Create a new IR connection.
    pub fn new(base_url: String) -> Self {
        Self {
            base_url,
            client: reqwest::Client::new(),
            token: None,
        }
    }

    /// Set the authentication token.
    pub fn set_token(&mut self, token: String) {
        self.token = Some(token);
    }

    /// Submit a score to the IR server.
    pub async fn submit_score(&self, score: &IrScoreData) -> Result<IrSubmitResponse> {
        let url = format!("{}/api/score", self.base_url);
        let mut req = self.client.post(&url).json(score);
        if let Some(token) = &self.token {
            req = req.bearer_auth(token);
        }
        let resp = req
            .send()
            .await
            .map_err(|e| anyhow!("IR submit request failed: {e}"))?;

        if !resp.status().is_success() {
            return Err(anyhow!("IR submit failed with status: {}", resp.status()));
        }

        resp.json::<IrSubmitResponse>()
            .await
            .map_err(|e| anyhow!("IR submit response parse failed: {e}"))
    }

    /// Retrieve rankings for a chart.
    pub async fn get_ranking(&self, sha256: &str) -> Result<IrRankingResponse> {
        let url = format!("{}/api/ranking/{sha256}", self.base_url);
        let mut req = self.client.get(&url);
        if let Some(token) = &self.token {
            req = req.bearer_auth(token);
        }
        let resp = req
            .send()
            .await
            .map_err(|e| anyhow!("IR ranking request failed: {e}"))?;

        if !resp.status().is_success() {
            return Err(anyhow!("IR ranking failed with status: {}", resp.status()));
        }

        resp.json::<IrRankingResponse>()
            .await
            .map_err(|e| anyhow!("IR ranking response parse failed: {e}"))
    }

    /// Get the base URL.
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Whether a token is set.
    pub fn is_authenticated(&self) -> bool {
        self.token.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_connection() {
        let conn = IrConnection::new("https://ir.example.com".to_string());
        assert_eq!(conn.base_url(), "https://ir.example.com");
        assert!(!conn.is_authenticated());
    }

    #[test]
    fn set_token() {
        let mut conn = IrConnection::new("https://ir.example.com".to_string());
        conn.set_token("my_token".to_string());
        assert!(conn.is_authenticated());
    }
}
