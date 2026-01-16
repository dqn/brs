# Internet Ranking (IR) Score Submission

インターネットランキングシステムによるスコア送信・ランキング表示機能。

## Overview

| 項目 | 内容 |
|------|------|
| 優先度 | 中 |
| 難易度 | 高 |
| 推定工数 | 5-7日 |
| 依存関係 | スコア保存システム（実装済み） |

## Background

IR (Internet Ranking) は BMS コミュニティで広く使われているオンラインスコアランキングシステム。LR2IR、Mocha-IR 等のサーバーが存在する。

### Existing IR Systems

| システム | 特徴 |
|----------|------|
| LR2IR | 最も普及、MD5 ハッシュベース |
| Mocha-IR | beatoraja 向け、SHA256 対応 |
| MinIR | 軽量実装 |

## Dependencies

- スコア保存システム（実装済み）
- ネットワーク通信用クレート

## Files to Modify/Create

| ファイル | 変更内容 |
|----------|----------|
| `src/ir/mod.rs` (新規) | IR モジュールルート |
| `src/ir/client.rs` (新規) | HTTP クライアント |
| `src/ir/protocol.rs` (新規) | スコア送信プロトコル |
| `src/ir/validation.rs` (新規) | スコア検証・ハッシュ |
| `src/config/settings.rs` | IR 設定追加 |
| `src/scene/result.rs` | IR 送信 UI |
| `Cargo.toml` | reqwest, md-5 追加 |

## Implementation Phases

### Phase 1: Dependencies

```toml
# Cargo.toml
[dependencies]
reqwest = { version = "0.12", features = ["json", "rustls-tls"] }
md-5 = "0.10"
```

### Phase 2: IR Protocol Definition

```rust
// src/ir/protocol.rs

use serde::{Deserialize, Serialize};

/// LR2IR compatible score submission
#[derive(Debug, Serialize)]
pub struct ScoreSubmission {
    /// Player identifier
    pub player_id: String,
    /// Chart SHA256 hash
    pub chart_hash: String,
    /// Chart MD5 hash (for LR2IR compatibility)
    pub chart_md5: String,
    /// EX Score (PGREAT*2 + GREAT)
    pub ex_score: u32,
    /// Clear lamp (0-7)
    pub clear_lamp: u8,
    /// Maximum combo
    pub max_combo: u32,
    /// Judgment counts
    pub pgreat_count: u32,
    pub great_count: u32,
    pub good_count: u32,
    pub bad_count: u32,
    pub poor_count: u32,
    /// Play options
    pub play_option: PlayOptionFlags,
    /// Unix timestamp
    pub timestamp: u64,
    /// Client version
    pub client_version: String,
    /// Score hash for validation
    pub score_hash: String,
}

#[derive(Debug, Serialize)]
pub struct PlayOptionFlags {
    /// Random option (0=OFF, 1=MIRROR, 2=RANDOM, etc.)
    pub random_option: u8,
    /// Gauge type (0=NORMAL, 1=EASY, 2=HARD, etc.)
    pub gauge_type: u8,
    /// Assist options bitmap
    pub assist_options: u8,
}

#[derive(Debug, Deserialize)]
pub struct SubmissionResponse {
    pub success: bool,
    pub rank: Option<u32>,
    pub total_players: Option<u32>,
    pub message: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RankingEntry {
    pub rank: u32,
    pub player_name: String,
    pub ex_score: u32,
    pub clear_lamp: u8,
    pub timestamp: u64,
}

#[derive(Debug, Deserialize)]
pub struct ChartRanking {
    pub chart_hash: String,
    pub entries: Vec<RankingEntry>,
    pub total_players: u32,
}
```

### Phase 3: Score Validation

```rust
// src/ir/validation.rs

use hmac::{Hmac, Mac};
use sha2::Sha256;
use md5;

type HmacSha256 = Hmac<Sha256>;

/// Generate score hash for anti-cheat validation
pub fn generate_score_hash(
    chart_hash: &str,
    ex_score: u32,
    clear_lamp: u8,
    pgreat_count: u32,
    timestamp: u64,
    secret_key: &[u8],
) -> String {
    let data = format!(
        "{}:{}:{}:{}:{}",
        chart_hash, ex_score, clear_lamp, pgreat_count, timestamp
    );

    let mut mac = HmacSha256::new_from_slice(secret_key)
        .expect("HMAC key error");
    mac.update(data.as_bytes());

    hex::encode(mac.finalize().into_bytes())
}

/// Compute MD5 hash for LR2IR compatibility
pub fn compute_md5_hash(path: &Path) -> Result<String> {
    let content = std::fs::read(path)?;
    let hash = md5::compute(&content);
    Ok(format!("{:x}", hash))
}

/// Compute SHA256 hash (already used in score.rs)
pub fn compute_sha256_hash(path: &Path) -> Result<String> {
    use sha2::{Sha256, Digest};
    let content = std::fs::read(path)?;
    let mut hasher = Sha256::new();
    hasher.update(&content);
    Ok(format!("{:x}", hasher.finalize()))
}
```

### Phase 4: IR Client

```rust
// src/ir/client.rs

use reqwest::Client;
use super::protocol::*;

pub struct IrClient {
    client: Client,
    base_url: String,
    player_id: String,
    secret_key: Vec<u8>,
}

impl IrClient {
    pub fn new(base_url: &str, player_id: &str) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.to_string(),
            player_id: player_id.to_string(),
            secret_key: Vec::new(), // Set from config
        }
    }

    /// Submit score to IR server
    pub async fn submit_score(
        &self,
        submission: ScoreSubmission,
    ) -> Result<SubmissionResponse> {
        let url = format!("{}/score/submit", self.base_url);

        let response = self.client
            .post(&url)
            .json(&submission)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            Err(anyhow!("IR submission failed: {}", response.status()))
        }
    }

    /// Get ranking for a chart
    pub async fn get_ranking(
        &self,
        chart_hash: &str,
        limit: u32,
    ) -> Result<ChartRanking> {
        let url = format!(
            "{}/ranking/{}?limit={}",
            self.base_url, chart_hash, limit
        );

        let response = self.client
            .get(&url)
            .send()
            .await?;

        Ok(response.json().await?)
    }

    /// Get player's rank for a chart
    pub async fn get_my_rank(&self, chart_hash: &str) -> Result<Option<u32>> {
        let url = format!(
            "{}/ranking/{}/player/{}",
            self.base_url, chart_hash, self.player_id
        );

        let response = self.client
            .get(&url)
            .send()
            .await?;

        if response.status().is_success() {
            let entry: RankingEntry = response.json().await?;
            Ok(Some(entry.rank))
        } else {
            Ok(None)
        }
    }
}
```

### Phase 5: Settings Integration

```rust
// src/config/settings.rs

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IrSettings {
    /// Enable IR score submission
    pub enabled: bool,
    /// IR server URL
    pub server_url: String,
    /// Player ID
    pub player_id: String,
    /// Auto-submit scores after play
    pub auto_submit: bool,
}

// Add to GameSettings
pub struct GameSettings {
    // ... existing fields
    #[serde(default)]
    pub ir: IrSettings,
}
```

### Phase 6: Result Scene Integration

```rust
// src/scene/result.rs

impl ResultScene {
    async fn submit_to_ir(&mut self) {
        if !self.settings.ir.enabled {
            return;
        }

        let client = IrClient::new(
            &self.settings.ir.server_url,
            &self.settings.ir.player_id,
        );

        let submission = ScoreSubmission {
            player_id: self.settings.ir.player_id.clone(),
            chart_hash: self.chart_hash.clone(),
            chart_md5: self.chart_md5.clone(),
            ex_score: self.result.ex_score,
            clear_lamp: self.result.clear_lamp as u8,
            // ... fill other fields
        };

        match client.submit_score(submission).await {
            Ok(response) => {
                if response.success {
                    self.ir_rank = response.rank;
                    self.ir_total = response.total_players;
                }
            }
            Err(e) => {
                eprintln!("IR submission failed: {}", e);
            }
        }
    }
}
```

## API Endpoints (Reference)

### LR2IR Compatible

```
POST /score/submit
  Body: ScoreSubmission JSON
  Response: SubmissionResponse

GET /ranking/{chart_hash}?limit=100
  Response: ChartRanking

GET /ranking/{chart_hash}/player/{player_id}
  Response: RankingEntry
```

## Verification

1. モック IR サーバーをセットアップ
2. スコア送信が正しく動作することを確認
3. ランキング取得が正しく動作することを確認
4. ネットワークエラー時のハンドリングを確認
5. MD5/SHA256 ハッシュの互換性を確認

## Security Considerations

- スコアハッシュによる改ざん検出
- HTTPS 通信必須
- プレイヤー認証（API キー or OAuth）
- レート制限対応
- 不正スコア検出（理論上限チェック等）

## Notes

- LR2IR との互換性のため MD5 ハッシュも計算
- 一部の IR サーバーは独自認証を使用
- オフライン時はローカルキューに保存して後で送信
