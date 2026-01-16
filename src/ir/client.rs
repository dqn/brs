use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::protocol::{ChartRanking, IrServerType, ScoreSubmission, SubmissionResponse};

const USER_AGENT: &str = concat!("brs/", env!("CARGO_PKG_VERSION"));

/// IR client for score submission and ranking retrieval
pub struct IrClient {
    client: Client,
    base_url: String,
    player_id: String,
    secret_key: String,
    server_type: IrServerType,
}

impl IrClient {
    /// Create a new IR client
    pub fn new(
        base_url: String,
        player_id: String,
        secret_key: String,
        server_type: IrServerType,
    ) -> Result<Self> {
        let client = Client::builder()
            .user_agent(USER_AGENT)
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self {
            client,
            base_url,
            player_id,
            secret_key,
            server_type,
        })
    }

    /// Get the player ID
    pub fn player_id(&self) -> &str {
        &self.player_id
    }

    /// Get the secret key
    pub fn secret_key(&self) -> &str {
        &self.secret_key
    }

    /// Submit a score to the IR server
    pub async fn submit_score(&self, submission: ScoreSubmission) -> Result<SubmissionResponse> {
        match self.server_type {
            IrServerType::Lr2Ir => self.submit_score_lr2ir(submission).await,
            IrServerType::MochaIr => self.submit_score_mocha(submission).await,
            IrServerType::MinIr => self.submit_score_minir(submission).await,
            IrServerType::Custom => self.submit_score_generic(submission).await,
        }
    }

    /// Get ranking for a chart
    pub async fn get_ranking(&self, chart_hash: &str, limit: u32) -> Result<ChartRanking> {
        match self.server_type {
            IrServerType::Lr2Ir => self.get_ranking_lr2ir(chart_hash, limit).await,
            IrServerType::MochaIr => self.get_ranking_mocha(chart_hash, limit).await,
            IrServerType::MinIr => self.get_ranking_minir(chart_hash, limit).await,
            IrServerType::Custom => self.get_ranking_generic(chart_hash, limit).await,
        }
    }

    /// Get player's rank for a chart
    pub async fn get_my_rank(&self, chart_hash: &str) -> Result<Option<u32>> {
        let url = format!(
            "{}/ranking/{}/player/{}",
            self.base_url, chart_hash, self.player_id
        );

        #[derive(Deserialize)]
        struct RankResponse {
            rank: Option<u32>,
        }

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to send request")?;

        if response.status().is_success() {
            let data: RankResponse = response.json().await.context("Failed to parse response")?;
            Ok(data.rank)
        } else {
            Ok(None)
        }
    }

    /// Test connection to the IR server
    pub async fn test_connection(&self) -> Result<bool> {
        let url = format!("{}/ping", self.base_url);

        match self.client.get(&url).send().await {
            Ok(response) => Ok(response.status().is_success()),
            Err(_) => Ok(false),
        }
    }

    // LR2IR specific implementation
    async fn submit_score_lr2ir(&self, submission: ScoreSubmission) -> Result<SubmissionResponse> {
        let url = format!("{}/score/submit", self.base_url);

        #[derive(Serialize)]
        struct Lr2IrSubmission {
            md5: String,
            playerid: String,
            exscore: u32,
            clear: u8,
            maxcombo: u32,
            perfect: u32,
            great: u32,
            good: u32,
            bad: u32,
            poor: u32,
            totalnotes: u32,
            option: u32,
            scorehash: String,
        }

        let lr2_submission = Lr2IrSubmission {
            md5: submission.chart_md5,
            playerid: submission.player_id,
            exscore: submission.ex_score,
            clear: submission.clear_lamp.as_u8(),
            maxcombo: submission.max_combo,
            perfect: submission.pgreat_count,
            great: submission.great_count,
            good: submission.good_count,
            bad: submission.bad_count,
            poor: submission.poor_count,
            totalnotes: submission.total_notes,
            option: submission.play_option.to_lr2ir_option(),
            scorehash: submission.score_hash,
        };

        let response = self
            .client
            .post(&url)
            .form(&lr2_submission)
            .send()
            .await
            .context("Failed to submit score")?;

        if response.status().is_success() {
            let data: SubmissionResponse =
                response.json().await.context("Failed to parse response")?;
            Ok(data)
        } else {
            Ok(SubmissionResponse {
                success: false,
                rank: None,
                total_players: None,
                message: Some(format!("HTTP error: {}", response.status())),
            })
        }
    }

    async fn get_ranking_lr2ir(&self, chart_hash: &str, limit: u32) -> Result<ChartRanking> {
        let url = format!("{}/ranking/{}?limit={}", self.base_url, chart_hash, limit);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to get ranking")?;

        if response.status().is_success() {
            let data: ChartRanking = response.json().await.context("Failed to parse ranking")?;
            Ok(data)
        } else {
            Ok(ChartRanking {
                chart_hash: chart_hash.to_string(),
                entries: Vec::new(),
                total_players: 0,
            })
        }
    }

    // Mocha-IR specific implementation
    async fn submit_score_mocha(&self, submission: ScoreSubmission) -> Result<SubmissionResponse> {
        // Mocha-IR uses similar format to LR2IR
        self.submit_score_generic(submission).await
    }

    async fn get_ranking_mocha(&self, chart_hash: &str, limit: u32) -> Result<ChartRanking> {
        self.get_ranking_generic(chart_hash, limit).await
    }

    // MinIR specific implementation
    async fn submit_score_minir(&self, submission: ScoreSubmission) -> Result<SubmissionResponse> {
        self.submit_score_generic(submission).await
    }

    async fn get_ranking_minir(&self, chart_hash: &str, limit: u32) -> Result<ChartRanking> {
        self.get_ranking_generic(chart_hash, limit).await
    }

    // Generic implementation for custom servers
    async fn submit_score_generic(
        &self,
        submission: ScoreSubmission,
    ) -> Result<SubmissionResponse> {
        let url = format!("{}/score/submit", self.base_url);

        let response = self
            .client
            .post(&url)
            .json(&submission)
            .send()
            .await
            .context("Failed to submit score")?;

        if response.status().is_success() {
            let data: SubmissionResponse =
                response.json().await.context("Failed to parse response")?;
            Ok(data)
        } else {
            Ok(SubmissionResponse {
                success: false,
                rank: None,
                total_players: None,
                message: Some(format!("HTTP error: {}", response.status())),
            })
        }
    }

    async fn get_ranking_generic(&self, chart_hash: &str, limit: u32) -> Result<ChartRanking> {
        let url = format!("{}/ranking/{}?limit={}", self.base_url, chart_hash, limit);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to get ranking")?;

        if response.status().is_success() {
            let data: ChartRanking = response.json().await.context("Failed to parse ranking")?;
            Ok(data)
        } else {
            Ok(ChartRanking {
                chart_hash: chart_hash.to_string(),
                entries: Vec::new(),
                total_players: 0,
            })
        }
    }
}
