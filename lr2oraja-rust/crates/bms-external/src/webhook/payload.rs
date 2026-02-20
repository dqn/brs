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

/// Emoji decoration per rank grade.
fn rank_emoji(rank: &str) -> &'static str {
    match rank {
        "AAA" => "\u{1F451}", // crown
        "AA" => "\u{2B50}",  // star
        "A" => "\u{1F7E2}",  // green circle
        "B" => "\u{1F535}",  // blue circle
        "C" => "\u{1F7E1}",  // yellow circle
        "D" => "\u{1F7E0}",  // orange circle
        "E" => "\u{1F534}",  // red circle
        _ => "\u{26AB}",     // black circle (F)
    }
}

/// Create a Discord embed from score info and song data.
pub fn create_embed(score_info: &ScreenshotScoreInfo, song_title: &str) -> Embed {
    let clear_name = screenshot::clear_type_name(score_info.clear_type_id);
    let rank = screenshot::rank_name(score_info.exscore, score_info.max_notes);
    let max_ex = score_info.max_notes * 2;
    let emoji = rank_emoji(rank);

    let score_rate = if max_ex > 0 {
        format!("{:.2}%", score_info.exscore as f64 / max_ex as f64 * 100.0)
    } else {
        "0.00%".to_string()
    };

    let mut fields = vec![
        EmbedField {
            name: "Clear".to_string(),
            value: clear_name.to_string(),
            inline: Some(true),
        },
        EmbedField {
            name: "Rank".to_string(),
            value: format!("{emoji} {rank}"),
            inline: Some(true),
        },
        EmbedField {
            name: "EX Score".to_string(),
            value: format!("{} / {} ({score_rate})", score_info.exscore, max_ex),
            inline: Some(true),
        },
    ];

    if let Some(combo) = score_info.maxcombo {
        fields.push(EmbedField {
            name: "Max Combo".to_string(),
            value: combo.to_string(),
            inline: Some(true),
        });
    }

    if let Some(ref pattern) = score_info.pattern_name {
        if !pattern.is_empty() {
            fields.push(EmbedField {
                name: "Option".to_string(),
                value: pattern.clone(),
                inline: Some(true),
            });
        }
    }

    // Rank delta vs previous best
    let description = score_info.prev_exscore.map(|prev| {
        let delta = score_info.exscore - prev;
        if delta > 0 {
            format!("\u{25B3}+{delta}")
        } else if delta < 0 {
            format!("\u{25BD}{delta}")
        } else {
            "\u{00B1}0".to_string()
        }
    });

    Embed {
        title: Some(song_title.to_string()),
        description,
        color: clear_type_color(score_info.clear_type_id),
        fields,
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
            ..Default::default()
        };
        let embed = create_embed(&info, "Test Song");
        assert_eq!(embed.title.as_deref(), Some("Test Song"));
        assert_eq!(embed.color, 0x0000FF); // NORMAL
        assert_eq!(embed.fields.len(), 3);
        assert_eq!(embed.fields[0].value, "NORMAL");
        assert!(embed.fields[2].value.contains("1500 / 2000"));
        assert!(embed.fields[2].value.contains("75.00%"));
        assert!(embed.description.is_none());
    }

    #[test]
    fn create_embed_full_combo() {
        let info = ScreenshotScoreInfo {
            clear_type_id: 8,
            exscore: 1800,
            max_notes: 1000,
            ..Default::default()
        };
        let embed = create_embed(&info, "FC Song");
        assert_eq!(embed.color, 0x00FFFF);
        assert_eq!(embed.fields[0].value, "FULL COMBO");
    }

    #[test]
    fn create_embed_with_optional_fields() {
        let info = ScreenshotScoreInfo {
            clear_type_id: 6,
            exscore: 1600,
            max_notes: 1000,
            maxcombo: Some(500),
            pattern_name: Some("MIRROR".to_string()),
            prev_exscore: Some(1550),
        };
        let embed = create_embed(&info, "Hard Song");
        assert_eq!(embed.fields.len(), 5);
        assert_eq!(embed.fields[3].name, "Max Combo");
        assert_eq!(embed.fields[3].value, "500");
        assert_eq!(embed.fields[4].name, "Option");
        assert_eq!(embed.fields[4].value, "MIRROR");
        assert_eq!(embed.description.as_deref(), Some("\u{25B3}+50"));
    }

    #[test]
    fn create_embed_rank_delta_negative() {
        let info = ScreenshotScoreInfo {
            clear_type_id: 5,
            exscore: 1400,
            max_notes: 1000,
            prev_exscore: Some(1500),
            ..Default::default()
        };
        let embed = create_embed(&info, "Worse");
        assert_eq!(embed.description.as_deref(), Some("\u{25BD}-100"));
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
