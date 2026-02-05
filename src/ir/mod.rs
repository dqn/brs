use std::collections::HashMap;
use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::{fs, thread};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tracing::{error, info, warn};

use crate::config::AppConfig;
use crate::database::{ClearType, SongData};
use crate::state::play::{GaugeType, PlayResult};

const IR_CONFIG_FILE: &str = "ir.json";

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum IrFormat {
    #[default]
    Form,
    Json,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct IrConfig {
    pub enabled: bool,
    pub endpoint: String,
    pub method: String,
    pub format: IrFormat,
    pub headers: HashMap<String, String>,
    pub fields: HashMap<String, String>,
    pub timeout_ms: u64,
}

impl Default for IrConfig {
    fn default() -> Self {
        let mut fields = HashMap::new();
        fields.insert("player".to_string(), "${player_name}".to_string());
        fields.insert("sha256".to_string(), "${sha256}".to_string());
        fields.insert("md5".to_string(), "${md5}".to_string());
        fields.insert("title".to_string(), "${title}".to_string());
        fields.insert("artist".to_string(), "${artist}".to_string());
        fields.insert("genre".to_string(), "${genre}".to_string());
        fields.insert("level".to_string(), "${level}".to_string());
        fields.insert("notes".to_string(), "${notes}".to_string());
        fields.insert("ex_score".to_string(), "${ex_score}".to_string());
        fields.insert("max_combo".to_string(), "${max_combo}".to_string());
        fields.insert("min_bp".to_string(), "${min_bp}".to_string());
        fields.insert("clear".to_string(), "${clear}".to_string());
        fields.insert("rank".to_string(), "${rank}".to_string());
        fields.insert("gauge".to_string(), "${gauge}".to_string());
        fields.insert("gauge_value".to_string(), "${gauge_value}".to_string());
        fields.insert("score_rate".to_string(), "${score_rate}".to_string());
        fields.insert("hi_speed".to_string(), "${hi_speed}".to_string());
        fields.insert("fast".to_string(), "${fast}".to_string());
        fields.insert("slow".to_string(), "${slow}".to_string());
        fields.insert("mode".to_string(), "${mode}".to_string());
        fields.insert("ln_mode".to_string(), "${ln_mode}".to_string());
        fields.insert("timestamp".to_string(), "${timestamp}".to_string());

        Self {
            enabled: false,
            endpoint: String::new(),
            method: "POST".to_string(),
            format: IrFormat::Form,
            headers: HashMap::new(),
            fields,
            timeout_ms: 5000,
        }
    }
}

impl IrConfig {
    pub fn load() -> Result<Self> {
        Self::load_from(IR_CONFIG_FILE)
    }

    pub fn load_from<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = fs::read_to_string(path)?;
        let config = serde_json::from_str(&content)?;
        Ok(config)
    }

    pub fn save(&self) -> Result<()> {
        self.save_to(IR_CONFIG_FILE)
    }

    pub fn save_to<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }
}

pub fn submit_score(
    play_result: &PlayResult,
    song: &SongData,
    clear_type: ClearType,
    hi_speed: f32,
) {
    let config = match IrConfig::load() {
        Ok(config) => config,
        Err(e) => {
            warn!("Failed to load IR config / IR設定の読み込みに失敗: {}", e);
            return;
        }
    };

    if !config.enabled || config.endpoint.trim().is_empty() {
        return;
    }

    let values = build_values(play_result, song, clear_type, hi_speed);
    let config = config.clone();

    thread::spawn(move || {
        if let Err(e) = send_request(&config, &values) {
            error!("IR submit failed / IR送信に失敗: {}", e);
        } else {
            info!("IR submit succeeded / IR送信に成功");
        }
    });
}

fn send_request(config: &IrConfig, values: &HashMap<String, String>) -> Result<()> {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_millis(config.timeout_ms))
        .build()?;

    let mut fields = HashMap::new();
    for (key, template) in &config.fields {
        fields.insert(key.clone(), render_template(template, values));
    }

    let method = config.method.to_uppercase();
    let is_get = method == "GET";
    let mut request = if is_get {
        client.get(&config.endpoint).query(&fields)
    } else {
        client.post(&config.endpoint)
    };

    for (key, value) in &config.headers {
        request = request.header(key, render_template(value, values));
    }

    if !is_get {
        request = match config.format {
            IrFormat::Json => request.json(&fields),
            IrFormat::Form => request.form(&fields),
        };
    }

    let response = request.send()?;
    if !response.status().is_success() {
        anyhow::bail!("IR returned status {}", response.status());
    }
    Ok(())
}

fn build_values(
    play_result: &PlayResult,
    song: &SongData,
    clear_type: ClearType,
    hi_speed: f32,
) -> HashMap<String, String> {
    let mut values = HashMap::new();
    let player_name = AppConfig::load()
        .unwrap_or_else(|_| AppConfig::default())
        .player_name;

    values.insert("player_name".to_string(), player_name);
    values.insert("sha256".to_string(), song.sha256.clone());
    values.insert("md5".to_string(), song.md5.clone());
    values.insert("title".to_string(), song.title.clone());
    values.insert("artist".to_string(), song.artist.clone());
    values.insert("genre".to_string(), song.genre.clone());
    values.insert("level".to_string(), song.level.to_string());
    values.insert("notes".to_string(), song.notes.to_string());
    values.insert("ex_score".to_string(), play_result.ex_score().to_string());
    values.insert("max_combo".to_string(), play_result.max_combo().to_string());
    values.insert("min_bp".to_string(), play_result.bp().to_string());
    values.insert("clear".to_string(), clear_type.as_i32().to_string());
    values.insert("rank".to_string(), play_result.rank().as_str().to_string());
    values.insert(
        "gauge".to_string(),
        gauge_type_label(play_result.gauge_type).to_string(),
    );
    values.insert(
        "gauge_value".to_string(),
        format!("{:.2}", play_result.gauge_value),
    );
    values.insert(
        "score_rate".to_string(),
        format!("{:.2}", play_result.score.clear_rate()),
    );
    values.insert("hi_speed".to_string(), format!("{:.2}", hi_speed));
    values.insert("fast".to_string(), play_result.fast_count.to_string());
    values.insert("slow".to_string(), play_result.slow_count.to_string());
    values.insert("mode".to_string(), song.mode.as_i32().to_string());
    values.insert(
        "ln_mode".to_string(),
        format!("{:?}", play_result.long_note_mode),
    );
    values.insert(
        "timestamp".to_string(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
            .to_string(),
    );

    values
}

fn gauge_type_label(gauge: GaugeType) -> &'static str {
    match gauge {
        GaugeType::AssistEasy => "ASSIST_EASY",
        GaugeType::LightAssistEasy => "LIGHT_ASSIST_EASY",
        GaugeType::Easy => "EASY",
        GaugeType::Normal => "NORMAL",
        GaugeType::Hard => "HARD",
        GaugeType::ExHard => "EX_HARD",
        GaugeType::Hazard => "HAZARD",
        GaugeType::Class => "CLASS",
    }
}

fn render_template(template: &str, values: &HashMap<String, String>) -> String {
    let mut rendered = template.to_string();
    for (key, value) in values {
        let placeholder = format!("${{{}}}", key);
        rendered = rendered.replace(&placeholder, value);
    }
    rendered
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_template() {
        let mut values = HashMap::new();
        values.insert("title".to_string(), "Example".to_string());
        values.insert("score".to_string(), "1234".to_string());
        let rendered = render_template("Song ${title} score ${score}", &values);
        assert_eq!(rendered, "Song Example score 1234");
    }
}
