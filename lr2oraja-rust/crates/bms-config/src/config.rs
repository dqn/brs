use std::collections::HashMap;
use std::path::Path;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::audio_config::AudioConfig;
use crate::resolution::Resolution;

/// Display mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "UPPERCASE")]
pub enum DisplayMode {
    Fullscreen,
    Borderless,
    #[default]
    Window,
}

/// Song preview mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "UPPERCASE")]
pub enum SongPreview {
    None,
    Once,
    #[default]
    Loop,
}

/// Default difficulty table URLs.
pub const DEFAULT_TABLEURL: &[&str] = &[
    "https://mqppppp.neocities.org/StardustTable.html",
    "https://djkuroakari.github.io/starlighttable.html",
    "https://stellabms.xyz/sl/table.html",
    "https://stellabms.xyz/st/table.html",
    "https://darksabun.club/table/archive/normal1/",
    "https://darksabun.club/table/archive/insane1/",
    "http://rattoto10.jounin.jp/table.html",
    "http://rattoto10.jounin.jp/table_insane.html",
    "https://rattoto10.jounin.jp/table_overjoy.html",
];

fn default_table_url() -> Vec<String> {
    DEFAULT_TABLEURL.iter().map(|s| s.to_string()).collect()
}

/// System-wide configuration (config_sys.json).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct Config {
    pub playername: Option<String>,
    pub last_booted_version: String,
    pub displaymode: DisplayMode,
    pub vsync: bool,
    pub resolution: Resolution,
    pub use_resolution: bool,
    pub window_width: i32,
    pub window_height: i32,
    pub folderlamp: bool,
    pub audio: AudioConfig,
    pub max_frame_per_second: i32,
    pub prepare_frame_per_second: i32,
    pub max_search_bar_count: i32,
    pub skip_decide_screen: bool,
    pub show_no_song_existing_bar: bool,
    pub scrolldurationlow: i32,
    pub scrolldurationhigh: i32,
    pub analog_scroll: bool,
    pub analog_ticks_per_scroll: i32,
    pub song_preview: SongPreview,
    pub cache_skin_image: bool,
    pub use_song_info: bool,
    pub songpath: String,
    pub songinfopath: String,
    pub tablepath: String,
    pub playerpath: String,
    pub skinpath: String,
    pub bgmpath: String,
    pub soundpath: String,
    pub systemfontpath: String,
    pub messagefontpath: String,
    pub bmsroot: Vec<String>,
    #[serde(rename = "tableURL")]
    pub table_url: Vec<String>,
    #[serde(rename = "availableURL")]
    pub available_url: Vec<String>,
    pub bga: i32,
    pub bga_expand: i32,
    pub frameskip: i32,
    pub updatesong: bool,
    pub skin_pixmap_gen: i32,
    pub stagefile_pixmap_gen: i32,
    pub banner_pixmap_gen: i32,
    pub song_resource_gen: i32,
    pub enable_ipfs: bool,
    pub ipfsurl: String,
    pub enable_http: bool,
    pub download_source: String,
    pub default_download_url: String,
    pub override_download_url: String,
    pub download_directory: String,
    pub ir_send_count: i32,
    pub use_discord_rpc: bool,
    pub set_clipboard_screenshot: bool,
    pub monitor_name: String,
    pub webhook_option: i32,
    pub webhook_name: String,
    pub webhook_avatar: String,
    pub webhook_url: Vec<String>,
    pub use_obs_ws: bool,
    pub obs_ws_host: String,
    pub obs_ws_port: i32,
    pub obs_ws_pass: String,
    pub obs_ws_rec_stop_wait: i32,
    pub obs_ws_rec_mode: i32,
    pub obs_scenes: HashMap<String, String>,
    pub obs_actions: HashMap<String, String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            playername: None,
            last_booted_version: String::new(),
            displaymode: DisplayMode::Window,
            vsync: false,
            resolution: Resolution::Hd,
            use_resolution: true,
            window_width: 1280,
            window_height: 720,
            folderlamp: true,
            audio: AudioConfig::default(),
            max_frame_per_second: 240,
            prepare_frame_per_second: 0,
            max_search_bar_count: 10,
            skip_decide_screen: false,
            show_no_song_existing_bar: true,
            scrolldurationlow: 300,
            scrolldurationhigh: 50,
            analog_scroll: true,
            analog_ticks_per_scroll: 3,
            song_preview: SongPreview::Loop,
            cache_skin_image: false,
            use_song_info: true,
            songpath: "songdata.db".to_string(),
            songinfopath: "songinfo.db".to_string(),
            tablepath: "table".to_string(),
            playerpath: "player".to_string(),
            skinpath: "skin".to_string(),
            bgmpath: "bgm".to_string(),
            soundpath: "sound".to_string(),
            systemfontpath: "font/VL-Gothic-Regular.ttf".to_string(),
            messagefontpath: "font/VL-Gothic-Regular.ttf".to_string(),
            bmsroot: Vec::new(),
            table_url: default_table_url(),
            available_url: Vec::new(),
            bga: 0,
            bga_expand: 1,
            frameskip: 1,
            updatesong: false,
            skin_pixmap_gen: 4,
            stagefile_pixmap_gen: 2,
            banner_pixmap_gen: 2,
            song_resource_gen: 1,
            enable_ipfs: true,
            ipfsurl: "https://gateway.ipfs.io/".to_string(),
            enable_http: true,
            download_source: String::new(),
            default_download_url: String::new(),
            override_download_url: String::new(),
            download_directory: "http_download".to_string(),
            ir_send_count: 5,
            use_discord_rpc: false,
            set_clipboard_screenshot: false,
            monitor_name: String::new(),
            webhook_option: 0,
            webhook_name: String::new(),
            webhook_avatar: String::new(),
            webhook_url: Vec::new(),
            use_obs_ws: false,
            obs_ws_host: "localhost".to_string(),
            obs_ws_port: 4455,
            obs_ws_pass: String::new(),
            obs_ws_rec_stop_wait: 5000,
            obs_ws_rec_mode: 0,
            obs_scenes: HashMap::new(),
            obs_actions: HashMap::new(),
        }
    }
}

impl Config {
    pub fn validate(&mut self) {
        // Window dimensions clamp
        self.window_width = self
            .window_width
            .clamp(Resolution::Sd.width(), Resolution::Ultrahd.width());
        self.window_height = self
            .window_height
            .clamp(Resolution::Sd.height(), Resolution::Ultrahd.height());

        self.audio.validate();

        self.max_frame_per_second = self.max_frame_per_second.clamp(0, 50000);
        self.prepare_frame_per_second = self.prepare_frame_per_second.clamp(0, 100000);
        self.max_search_bar_count = self.max_search_bar_count.clamp(1, 100);

        self.scrolldurationlow = self.scrolldurationlow.clamp(2, 1000);
        self.scrolldurationhigh = self.scrolldurationhigh.clamp(1, 1000);
        self.ir_send_count = self.ir_send_count.clamp(1, 100);

        self.skin_pixmap_gen = self.skin_pixmap_gen.clamp(0, 100);
        self.stagefile_pixmap_gen = self.stagefile_pixmap_gen.clamp(0, 100);
        self.banner_pixmap_gen = self.banner_pixmap_gen.clamp(0, 100);
        self.song_resource_gen = self.song_resource_gen.clamp(0, 100);

        // Remove empty bmsroot entries
        self.bmsroot.retain(|s| !s.is_empty());

        if self.table_url.is_empty() {
            self.table_url = default_table_url();
        }
        self.table_url.retain(|s| !s.is_empty());

        self.bga = self.bga.clamp(0, 2);
        self.bga_expand = self.bga_expand.clamp(0, 2);

        if self.ipfsurl.is_empty() {
            self.ipfsurl = "https://gateway.ipfs.io/".to_string();
        }

        // Path defaults
        if self.songpath.is_empty() {
            self.songpath = "songdata.db".to_string();
        }
        if self.songinfopath.is_empty() {
            self.songinfopath = "songinfo.db".to_string();
        }
        if self.tablepath.is_empty() {
            self.tablepath = "table".to_string();
        }
        if self.playerpath.is_empty() {
            self.playerpath = "player".to_string();
        }
        if self.skinpath.is_empty() {
            self.skinpath = "skin".to_string();
        }
        if self.download_directory.is_empty() {
            self.download_directory = "http_download".to_string();
        }
    }

    /// Read config from a JSON file.
    pub fn read(path: &Path) -> Result<Self> {
        let data = std::fs::read_to_string(path)?;
        let mut config: Config = serde_json::from_str(&data)?;
        config.validate();
        Ok(config)
    }

    /// Write config to a JSON file.
    pub fn write(&self, path: &Path) -> Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let c = Config::default();
        assert_eq!(c.displaymode, DisplayMode::Window);
        assert_eq!(c.resolution, Resolution::Hd);
        assert_eq!(c.window_width, 1280);
        assert_eq!(c.window_height, 720);
        assert_eq!(c.max_frame_per_second, 240);
        assert_eq!(c.scrolldurationlow, 300);
        assert_eq!(c.scrolldurationhigh, 50);
        assert_eq!(c.song_preview, SongPreview::Loop);
        assert_eq!(c.bga, 0);
        assert_eq!(c.bga_expand, 1);
        assert!(!c.table_url.is_empty());
        assert_eq!(c.obs_ws_port, 4455);
    }

    #[test]
    fn test_validate_clamps() {
        let mut c = Config {
            window_width: 100,
            window_height: 50000,
            max_frame_per_second: -1,
            scrolldurationlow: 0,
            scrolldurationhigh: 0,
            bga: 5,
            bga_expand: -1,
            ir_send_count: 0,
            ..Default::default()
        };
        c.validate();
        assert_eq!(c.window_width, Resolution::Sd.width());
        assert_eq!(c.window_height, Resolution::Ultrahd.height());
        assert_eq!(c.max_frame_per_second, 0);
        assert_eq!(c.scrolldurationlow, 2);
        assert_eq!(c.scrolldurationhigh, 1);
        assert_eq!(c.bga, 2);
        assert_eq!(c.bga_expand, 0);
        assert_eq!(c.ir_send_count, 1);
    }

    #[test]
    fn test_validate_empty_paths_get_defaults() {
        let mut c = Config {
            songpath: String::new(),
            playerpath: String::new(),
            download_directory: String::new(),
            ipfsurl: String::new(),
            ..Default::default()
        };
        c.validate();
        assert_eq!(c.songpath, "songdata.db");
        assert_eq!(c.playerpath, "player");
        assert_eq!(c.download_directory, "http_download");
        assert_eq!(c.ipfsurl, "https://gateway.ipfs.io/");
    }

    #[test]
    fn test_validate_empty_table_url_gets_default() {
        let mut c = Config {
            table_url: Vec::new(),
            ..Default::default()
        };
        c.validate();
        assert!(!c.table_url.is_empty());
    }

    #[test]
    fn test_serde_round_trip() {
        let c = Config::default();
        let json = serde_json::to_string(&c).unwrap();
        let back: Config = serde_json::from_str(&json).unwrap();
        assert_eq!(back.displaymode, c.displaymode);
        assert_eq!(back.resolution, c.resolution);
        assert_eq!(back.max_frame_per_second, c.max_frame_per_second);
    }

    #[test]
    fn test_display_mode_serde() {
        let json = serde_json::to_string(&DisplayMode::Fullscreen).unwrap();
        assert_eq!(json, "\"FULLSCREEN\"");
        let json = serde_json::to_string(&DisplayMode::Borderless).unwrap();
        assert_eq!(json, "\"BORDERLESS\"");
        let json = serde_json::to_string(&DisplayMode::Window).unwrap();
        assert_eq!(json, "\"WINDOW\"");
    }

    #[test]
    fn test_song_preview_serde() {
        let json = serde_json::to_string(&SongPreview::None).unwrap();
        assert_eq!(json, "\"NONE\"");
        let json = serde_json::to_string(&SongPreview::Once).unwrap();
        assert_eq!(json, "\"ONCE\"");
        let json = serde_json::to_string(&SongPreview::Loop).unwrap();
        assert_eq!(json, "\"LOOP\"");
    }

    #[test]
    fn test_deserialize_from_empty() {
        let c: Config = serde_json::from_str("{}").unwrap();
        assert_eq!(c.displaymode, DisplayMode::Window);
        assert_eq!(c.resolution, Resolution::Hd);
    }

    #[test]
    fn test_read_write_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config_sys.json");

        let config = Config::default();
        config.write(&path).unwrap();

        let loaded = Config::read(&path).unwrap();
        assert_eq!(loaded.displaymode, config.displaymode);
        assert_eq!(loaded.resolution, config.resolution);
        assert_eq!(loaded.max_frame_per_second, config.max_frame_per_second);
    }

    #[test]
    fn test_table_url_serde_name() {
        let c = Config::default();
        let json = serde_json::to_string(&c).unwrap();
        // Verify the serde rename
        assert!(json.contains("\"tableURL\""));
        assert!(json.contains("\"availableURL\""));
    }
}
