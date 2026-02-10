use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use bms_database::SongData;

/// IR chart data for transmission.
///
/// Corresponds to Java `IRChartData`. Constructed from `SongData`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IRChartData {
    pub md5: String,
    pub sha256: String,
    pub title: String,
    pub subtitle: String,
    pub genre: String,
    pub artist: String,
    pub subartist: String,
    pub url: String,
    pub appendurl: String,
    pub level: i32,
    pub total: i32,
    pub mode: i32,
    /// LN TYPE (-1: unspecified, 0: LN, 1: CN, 2: HCN)
    pub lntype: i32,
    pub judge: i32,
    pub minbpm: i32,
    pub maxbpm: i32,
    pub notes: i32,
    pub has_undefined_ln: bool,
    pub has_ln: bool,
    pub has_cn: bool,
    pub has_hcn: bool,
    pub has_mine: bool,
    pub has_random: bool,
    pub has_stop: bool,
    pub values: HashMap<String, String>,
}

impl IRChartData {
    /// Create from SongData with specified lntype.
    pub fn from_song_data(song: &SongData, lntype: i32) -> Self {
        Self {
            md5: song.md5.clone(),
            sha256: song.sha256.clone(),
            title: song.title.clone(),
            subtitle: song.subtitle.clone(),
            genre: song.genre.clone(),
            artist: song.artist.clone(),
            subartist: song.subartist.clone(),
            // Rust SongData doesn't have url/appendurl fields
            url: String::new(),
            appendurl: String::new(),
            level: song.level,
            total: 0, // model.getTotal() — not available from SongData alone
            mode: song.mode,
            lntype,
            judge: song.judge,
            minbpm: song.minbpm,
            maxbpm: song.maxbpm,
            notes: song.notes,
            has_undefined_ln: song.has_undefined_long_note(),
            has_ln: song.has_long_note(),
            has_cn: song.has_charge_note(),
            has_hcn: song.has_hell_charge_note(),
            has_mine: song.has_mine_note(),
            has_random: song.has_random_sequence(),
            has_stop: song.has_stop_sequence(),
            values: HashMap::new(),
        }
    }
}

impl From<&SongData> for IRChartData {
    fn from(song: &SongData) -> Self {
        Self::from_song_data(song, 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_song_data() -> SongData {
        SongData {
            md5: "deadbeef".to_string(),
            sha256: "abc123def456".to_string(),
            title: "Test Song".to_string(),
            subtitle: "~sub~".to_string(),
            genre: "BMS".to_string(),
            artist: "Composer".to_string(),
            subartist: "obj:Charter".to_string(),
            level: 12,
            mode: 7,
            judge: 100,
            minbpm: 150,
            maxbpm: 180,
            notes: 1200,
            feature: 0b0000_0011, // UNDEFINEDLN | MINENOTE
            ..Default::default()
        }
    }

    #[test]
    fn from_song_data_basic() {
        let song = sample_song_data();
        let chart = IRChartData::from(&song);
        assert_eq!(chart.md5, "deadbeef");
        assert_eq!(chart.sha256, "abc123def456");
        assert_eq!(chart.title, "Test Song");
        assert_eq!(chart.subtitle, "~sub~");
        assert_eq!(chart.level, 12);
        assert_eq!(chart.notes, 1200);
        assert_eq!(chart.lntype, 0);
    }

    #[test]
    fn from_song_data_feature_flags() {
        let song = sample_song_data();
        let chart = IRChartData::from(&song);
        assert!(chart.has_undefined_ln);
        assert!(chart.has_mine);
        assert!(!chart.has_cn);
        assert!(!chart.has_hcn);
        assert!(!chart.has_random);
        assert!(!chart.has_stop);
    }

    #[test]
    fn from_song_data_with_lntype() {
        let song = sample_song_data();
        let chart = IRChartData::from_song_data(&song, 2);
        assert_eq!(chart.lntype, 2);
    }

    #[test]
    fn serde_round_trip() {
        let song = sample_song_data();
        let chart = IRChartData::from(&song);
        let json = serde_json::to_string(&chart).unwrap();
        let deserialized: IRChartData = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.md5, chart.md5);
        assert_eq!(deserialized.sha256, chart.sha256);
        assert_eq!(deserialized.has_undefined_ln, chart.has_undefined_ln);
    }
}
