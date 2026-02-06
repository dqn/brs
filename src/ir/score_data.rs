use serde::{Deserialize, Serialize};

/// Score data for IR submission.
/// Matches the beatoraja IR score format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IrScoreData {
    /// Chart SHA-256 hash.
    pub sha256: String,
    /// EX score (PG * 2 + GR).
    pub exscore: u32,
    /// Maximum combo.
    pub max_combo: u32,
    /// Miss count (BD + PR + MS).
    pub min_bp: u32,
    /// Clear type index.
    pub clear: u32,
    /// Judge counts [PG, GR, GD, BD, PR, MS].
    pub judges: [u32; 6],
    /// Gauge type index.
    pub gauge: u32,
    /// Random option.
    pub random: i32,
    /// Play mode (lane count).
    pub mode: u32,
    /// Player name.
    pub player: String,
}

/// A ranking entry from the IR server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IrRankingEntry {
    /// Rank position (1-based).
    pub rank: u32,
    /// Player name.
    pub player: String,
    /// EX score.
    pub exscore: u32,
    /// Clear type index.
    pub clear: u32,
    /// Maximum combo.
    pub max_combo: u32,
    /// Miss count.
    pub min_bp: u32,
}

/// Response from IR server for score submission.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IrSubmitResponse {
    /// Whether submission was successful.
    pub success: bool,
    /// Server message.
    #[serde(default)]
    pub message: String,
    /// New ranking position (if changed).
    pub rank: Option<u32>,
}

/// Response from IR server for ranking retrieval.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IrRankingResponse {
    /// Whether retrieval was successful.
    pub success: bool,
    /// Ranking entries.
    #[serde(default)]
    pub rankings: Vec<IrRankingEntry>,
    /// Total number of players who have played this chart.
    #[serde(default)]
    pub total_players: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn score_data_serialization() {
        let data = IrScoreData {
            sha256: "abc123".to_string(),
            exscore: 1500,
            max_combo: 300,
            min_bp: 5,
            clear: 3,
            judges: [200, 100, 50, 3, 1, 1],
            gauge: 0,
            random: 0,
            mode: 7,
            player: "testplayer".to_string(),
        };
        let json = serde_json::to_string(&data).unwrap();
        let restored: IrScoreData = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.exscore, 1500);
        assert_eq!(restored.judges[0], 200);
        assert_eq!(restored.player, "testplayer");
    }

    #[test]
    fn ranking_entry_serialization() {
        let entry = IrRankingEntry {
            rank: 1,
            player: "topplayer".to_string(),
            exscore: 2000,
            clear: 5,
            max_combo: 500,
            min_bp: 0,
        };
        let json = serde_json::to_string(&entry).unwrap();
        let restored: IrRankingEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.rank, 1);
        assert_eq!(restored.exscore, 2000);
    }

    #[test]
    fn submit_response_serialization() {
        let resp = IrSubmitResponse {
            success: true,
            message: "Score accepted".to_string(),
            rank: Some(5),
        };
        let json = serde_json::to_string(&resp).unwrap();
        let restored: IrSubmitResponse = serde_json::from_str(&json).unwrap();
        assert!(restored.success);
        assert_eq!(restored.rank, Some(5));
    }

    #[test]
    fn ranking_response_serialization() {
        let resp = IrRankingResponse {
            success: true,
            rankings: vec![IrRankingEntry {
                rank: 1,
                player: "top".to_string(),
                exscore: 2000,
                clear: 5,
                max_combo: 500,
                min_bp: 0,
            }],
            total_players: 100,
        };
        let json = serde_json::to_string(&resp).unwrap();
        let restored: IrRankingResponse = serde_json::from_str(&json).unwrap();
        assert!(restored.success);
        assert_eq!(restored.rankings.len(), 1);
        assert_eq!(restored.total_players, 100);
    }

    #[test]
    fn submit_response_defaults() {
        let json = r#"{"success": false}"#;
        let resp: IrSubmitResponse = serde_json::from_str(json).unwrap();
        assert!(!resp.success);
        assert!(resp.message.is_empty());
        assert!(resp.rank.is_none());
    }
}
