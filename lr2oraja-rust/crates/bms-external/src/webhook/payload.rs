use serde::{Deserialize, Serialize};

use crate::screenshot::{self, ScreenshotScoreInfo};

/// Discord webhook payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookPayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,
    pub embeds: Vec<Embed>,
}

/// Discord embed object.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Embed {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub color: u32,
    pub fields: Vec<EmbedField>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail: Option<EmbedThumbnail>,
}

/// Discord embed field.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbedField {
    pub name: String,
    pub value: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inline: Option<bool>,
}

/// Discord embed thumbnail.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbedThumbnail {
    pub url: String,
}

/// Color mapping by clear type ID.
fn clear_type_color(clear_type_id: u8) -> u32 {
    match clear_type_id {
        0 => 0x808080,  // NO PLAY - gray
        1 => 0xFF0000,  // FAILED - red
        2 => 0x9932CC,  // ASSIST EASY - purple
        3 => 0xDA70D6,  // LIGHT ASSIST EASY - orchid
        4 => 0x00FF00,  // EASY - green
        5 => 0x0000FF,  // NORMAL - blue
        6 => 0xFF8C00,  // HARD - orange
        7 => 0xFFD700,  // EX HARD - gold
        8 => 0x00FFFF,  // FULL COMBO - cyan
        9 => 0xFFFFFF,  // PERFECT - white
        10 => 0xFFFF00, // MAX - yellow
        _ => 0x808080,
    }
}

/// Create a Discord embed from score info and song data.
pub fn create_embed(score_info: &ScreenshotScoreInfo, song_title: &str) -> Embed {
    let clear_name = screenshot::clear_type_name(score_info.clear_type_id);
    let rank = screenshot::rank_name(score_info.exscore, score_info.max_notes);
    let max_ex = score_info.max_notes * 2;

    Embed {
        title: Some(song_title.to_string()),
        description: None,
        color: clear_type_color(score_info.clear_type_id),
        fields: vec![
            EmbedField {
                name: "Clear".to_string(),
                value: clear_name.to_string(),
                inline: Some(true),
            },
            EmbedField {
                name: "Rank".to_string(),
                value: rank.to_string(),
                inline: Some(true),
            },
            EmbedField {
                name: "EX Score".to_string(),
                value: format!("{} / {}", score_info.exscore, max_ex),
                inline: Some(true),
            },
        ],
        thumbnail: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clear_type_color_all_variants() {
        assert_eq!(clear_type_color(0), 0x808080);
        assert_eq!(clear_type_color(1), 0xFF0000);
        assert_eq!(clear_type_color(6), 0xFF8C00);
        assert_eq!(clear_type_color(10), 0xFFFF00);
        assert_eq!(clear_type_color(255), 0x808080);
    }

    #[test]
    fn create_embed_basic() {
        let info = ScreenshotScoreInfo {
            clear_type_id: 5,
            exscore: 1500,
            max_notes: 1000,
        };
        let embed = create_embed(&info, "Test Song");
        assert_eq!(embed.title.as_deref(), Some("Test Song"));
        assert_eq!(embed.color, 0x0000FF); // NORMAL
        assert_eq!(embed.fields.len(), 3);
        assert_eq!(embed.fields[0].value, "NORMAL");
        assert_eq!(embed.fields[2].value, "1500 / 2000");
    }

    #[test]
    fn create_embed_full_combo() {
        let info = ScreenshotScoreInfo {
            clear_type_id: 8,
            exscore: 1800,
            max_notes: 1000,
        };
        let embed = create_embed(&info, "FC Song");
        assert_eq!(embed.color, 0x00FFFF);
        assert_eq!(embed.fields[0].value, "FULL COMBO");
    }

    #[test]
    fn webhook_payload_serde() {
        let payload = WebhookPayload {
            username: Some("brs".to_string()),
            avatar_url: None,
            embeds: vec![Embed {
                title: Some("Song".to_string()),
                description: None,
                color: 0xFF0000,
                fields: vec![],
                thumbnail: None,
            }],
        };
        let json = serde_json::to_string(&payload).unwrap();
        let deserialized: WebhookPayload = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.username, Some("brs".to_string()));
        assert!(deserialized.avatar_url.is_none());
        assert_eq!(deserialized.embeds.len(), 1);
    }

    #[test]
    fn webhook_payload_skips_none() {
        let payload = WebhookPayload {
            username: None,
            avatar_url: None,
            embeds: vec![],
        };
        let json = serde_json::to_string(&payload).unwrap();
        assert!(!json.contains("username"));
        assert!(!json.contains("avatar_url"));
    }
}
