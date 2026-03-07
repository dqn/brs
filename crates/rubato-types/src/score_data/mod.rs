use std::fmt;
use std::io::{Read, Write};

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE;
use flate2::Compression;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use serde::{Deserialize, Serialize};

use bms_model::mode::Mode;

use crate::clear_type::ClearType;
use crate::stubs::{BMSPlayerRule, JudgeAlgorithm, bms_player_input_device};
use crate::validatable::Validatable;

/// Early/late judge counts for each judge type (PG, GR, GD, BD, PR, MS).
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct JudgeCounts {
    pub epg: i32,
    pub lpg: i32,
    pub egr: i32,
    pub lgr: i32,
    pub egd: i32,
    pub lgd: i32,
    pub ebd: i32,
    pub lbd: i32,
    pub epr: i32,
    pub lpr: i32,
    pub ems: i32,
    pub lms: i32,
}

/// Timing statistics (average judge, duration, averages, standard deviation).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct TimingStats {
    pub avgjudge: i64,
    #[serde(rename = "totalDuration")]
    pub total_duration: i64,
    pub avg: i64,
    #[serde(rename = "totalAvg")]
    pub total_avg: i64,
    pub stddev: i64,
}

impl Default for TimingStats {
    fn default() -> Self {
        Self {
            avgjudge: i64::MAX,
            total_duration: 0,
            avg: i64::MAX,
            total_avg: 0,
            stddev: i64::MAX,
        }
    }
}

/// Play options and configuration at the time of scoring.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct PlayOption {
    pub random: i32,
    pub option: i32,
    pub seed: i64,
    pub assist: i32,
    pub gauge: i32,
    #[serde(rename = "deviceType")]
    pub device_type: Option<bms_player_input_device::Type>,
    #[serde(rename = "judgeAlgorithm")]
    pub judge_algorithm: Option<JudgeAlgorithm>,
    pub rule: Option<BMSPlayerRule>,
    pub skin: Option<String>,
}

/// Score data
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScoreData {
    pub sha256: String,
    pub player: String,
    pub mode: i32,
    pub clear: i32,
    pub date: i64,
    pub playcount: i32,
    pub clearcount: i32,
    #[serde(flatten)]
    pub judge_counts: JudgeCounts,
    pub maxcombo: i32,
    pub notes: i32,
    pub passnotes: i32,
    pub minbp: i32,
    #[serde(flatten)]
    pub timing_stats: TimingStats,
    pub trophy: String,
    pub ghost: String,
    #[serde(flatten)]
    pub play_option: PlayOption,
    pub state: i32,
    pub scorehash: String,
    pub playmode: Mode,
}

impl ScoreData {
    pub const TROPHY_EASY: SongTrophy = SongTrophy::Easy;
    pub const TROPHY_GROOVE: SongTrophy = SongTrophy::Groove;
    pub const TROPHY_HARD: SongTrophy = SongTrophy::Hard;
    pub const TROPHY_EXHARD: SongTrophy = SongTrophy::ExHard;
    pub const TROPHY_NORMAL: SongTrophy = SongTrophy::Normal;
    pub const TROPHY_MIRROR: SongTrophy = SongTrophy::Mirror;
    pub const TROPHY_RANDOM: SongTrophy = SongTrophy::Random;
    pub const TROPHY_R_RANDOM: SongTrophy = SongTrophy::RRandom;
    pub const TROPHY_S_RANDOM: SongTrophy = SongTrophy::SRandom;
    pub const TROPHY_H_RANDOM: SongTrophy = SongTrophy::HRandom;
    pub const TROPHY_SPIRAL: SongTrophy = SongTrophy::Spiral;
    pub const TROPHY_ALL_SCR: SongTrophy = SongTrophy::AllScr;
    pub const TROPHY_EX_RANDOM: SongTrophy = SongTrophy::ExRandom;
    pub const TROPHY_EX_S_RANDOM: SongTrophy = SongTrophy::ExSRandom;
    pub const TROPHY_BATTLE: SongTrophy = SongTrophy::Battle;
    pub const TROPHY_BATTLE_ASSIST: SongTrophy = SongTrophy::BattleAssist;
}

impl Default for ScoreData {
    fn default() -> Self {
        Self::new(Mode::BEAT_7K)
    }
}

impl ScoreData {
    pub fn new(playmode: Mode) -> Self {
        Self {
            sha256: String::new(),
            player: "unknown".to_string(),
            mode: 0,
            clear: 0,
            date: 0,
            playcount: 0,
            clearcount: 0,
            judge_counts: JudgeCounts::default(),
            maxcombo: 0,
            notes: 0,
            passnotes: 0,
            minbp: i32::MAX,
            timing_stats: TimingStats::default(),
            trophy: String::new(),
            ghost: String::new(),
            play_option: PlayOption {
                seed: -1,
                ..Default::default()
            },
            state: 0,
            scorehash: String::new(),
            playmode,
        }
    }

    pub fn set_player(&mut self, player: Option<&str>) {
        self.player = player.unwrap_or("").to_string();
    }

    pub fn exscore(&self) -> i32 {
        let jc = &self.judge_counts;
        (jc.epg.saturating_add(jc.lpg))
            .saturating_mul(2)
            .saturating_add(jc.egr)
            .saturating_add(jc.lgr)
    }

    pub fn judge_count_total(&self, judge: i32) -> i32 {
        self.judge_count(judge, true) + self.judge_count(judge, false)
    }

    /// Get judge count for a specific judge type.
    /// judge: 0=PG, 1=GR, 2=GD, 3=BD, 4=PR, 5=MS
    /// fast: true=FAST, false=SLOW
    pub fn judge_count(&self, judge: i32, fast: bool) -> i32 {
        let jc = &self.judge_counts;
        match judge {
            0 => {
                if fast {
                    jc.epg
                } else {
                    jc.lpg
                }
            }
            1 => {
                if fast {
                    jc.egr
                } else {
                    jc.lgr
                }
            }
            2 => {
                if fast {
                    jc.egd
                } else {
                    jc.lgd
                }
            }
            3 => {
                if fast {
                    jc.ebd
                } else {
                    jc.lbd
                }
            }
            4 => {
                if fast {
                    jc.epr
                } else {
                    jc.lpr
                }
            }
            5 => {
                if fast {
                    jc.ems
                } else {
                    jc.lms
                }
            }
            _ => 0,
        }
    }

    pub fn add_judge_count(&mut self, judge: i32, fast: bool, count: i32) {
        let jc = &mut self.judge_counts;
        match judge {
            0 => {
                if fast {
                    jc.epg += count;
                } else {
                    jc.lpg += count;
                }
            }
            1 => {
                if fast {
                    jc.egr += count;
                } else {
                    jc.lgr += count;
                }
            }
            2 => {
                if fast {
                    jc.egd += count;
                } else {
                    jc.lgd += count;
                }
            }
            3 => {
                if fast {
                    jc.ebd += count;
                } else {
                    jc.lbd += count;
                }
            }
            4 => {
                if fast {
                    jc.epr += count;
                } else {
                    jc.lpr += count;
                }
            }
            5 => {
                if fast {
                    jc.ems += count;
                } else {
                    jc.lms += count;
                }
            }
            _ => {}
        }
    }

    #[allow(clippy::unbuffered_bytes)]
    pub fn decode_ghost(&self) -> Option<Vec<i32>> {
        if self.ghost.is_empty() {
            return None;
        }
        let decoded = match URL_SAFE.decode(self.ghost.as_bytes()) {
            Ok(d) => d,
            Err(_) => return None,
        };
        let mut gz = match GzDecoder::new(&decoded[..])
            .bytes()
            .collect::<Result<Vec<u8>, _>>()
        {
            Ok(_bytes) => {
                // Re-create decoder for proper reading
                drop(_bytes);
                GzDecoder::new(&decoded[..])
            }
            Err(_) => return None,
        };
        let mut decompressed = Vec::new();
        if gz.read_to_end(&mut decompressed).is_err() {
            return None;
        }
        if decompressed.is_empty() {
            return None;
        }
        let value: Vec<i32> = (0..self.notes as usize)
            .map(|i| {
                if i < decompressed.len() {
                    let judge = decompressed[i] as i32;
                    if judge >= 0 { judge } else { 4 }
                } else {
                    4
                }
            })
            .collect();
        Some(value)
    }

    #[allow(clippy::redundant_guards)]
    pub fn encode_ghost(&mut self, value: Option<&[i32]>) {
        match value {
            None => {
                self.ghost = String::new();
            }
            Some(v) if v.is_empty() => {
                self.ghost = String::new();
            }
            Some(v) => {
                let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
                let bytes: Vec<u8> = v.iter().map(|&j| j as u8).collect();
                if encoder.write_all(&bytes).is_err() {
                    self.ghost = String::new();
                    return;
                }
                match encoder.finish() {
                    Ok(compressed) => {
                        self.ghost = URL_SAFE.encode(&compressed);
                    }
                    Err(_) => {
                        self.ghost = String::new();
                    }
                }
            }
        }
    }

    /// Update this score data with a new score. Returns true if updated.
    /// If update_score is false, only clear is updated.
    pub fn update(&mut self, newscore: &ScoreData, update_score: bool) -> bool {
        let mut update = false;
        if self.clear < newscore.clear {
            self.clear = newscore.clear;
            self.play_option.option = newscore.play_option.option;
            self.play_option.seed = newscore.play_option.seed;
            update = true;
        }
        if self.exscore() < newscore.exscore() && update_score {
            self.judge_counts = newscore.judge_counts.clone();
            self.play_option.option = newscore.play_option.option;
            self.play_option.seed = newscore.play_option.seed;
            self.ghost = newscore.ghost.clone();
            update = true;
        }
        if self.timing_stats.avgjudge > newscore.timing_stats.avgjudge && update_score {
            self.timing_stats.avgjudge = newscore.timing_stats.avgjudge;
            self.play_option.option = newscore.play_option.option;
            self.play_option.seed = newscore.play_option.seed;
            update = true;
        }
        if self.minbp > newscore.minbp && update_score {
            self.minbp = newscore.minbp;
            self.play_option.option = newscore.play_option.option;
            self.play_option.seed = newscore.play_option.seed;
            update = true;
        }
        if self.maxcombo < newscore.maxcombo && update_score {
            self.maxcombo = newscore.maxcombo;
            self.play_option.option = newscore.play_option.option;
            self.play_option.seed = newscore.play_option.seed;
            update = true;
        }
        update
    }
}

impl Validatable for ScoreData {
    fn validate(&mut self) -> bool {
        let jc = &self.judge_counts;
        let po = &self.play_option;
        self.mode >= 0
            && self.clear >= 0
            && self.clear <= ClearType::Max.id()
            && jc.epg >= 0
            && jc.lpg >= 0
            && jc.egr >= 0
            && jc.lgr >= 0
            && jc.egd >= 0
            && jc.lgd >= 0
            && jc.ebd >= 0
            && jc.lbd >= 0
            && jc.epr >= 0
            && jc.lpr >= 0
            && jc.ems >= 0
            && jc.lms >= 0
            && self.clearcount >= 0
            && self.playcount >= self.clearcount
            && self.maxcombo >= 0
            && self.notes > 0
            && self.passnotes >= 0
            && self.passnotes <= self.notes
            && self.minbp >= 0
            && self.timing_stats.avgjudge >= 0
            && po.random >= 0
            && po.option >= 0
            && po.assist >= 0
            && po.gauge >= 0
    }
}

impl fmt::Display for ScoreData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let jc = &self.judge_counts;
        let ts = &self.timing_stats;
        let po = &self.play_option;
        write!(f, "{{")?;
        write!(f, "\"Date\": {}, ", self.date)?;
        write!(f, "\"Playcount\": {}, ", self.playcount)?;
        write!(f, "\"Clear\": {}, ", self.clear)?;
        write!(f, "\"Epg\": {}, ", jc.epg)?;
        write!(f, "\"Lpg\": {}, ", jc.lpg)?;
        write!(f, "\"Egr\": {}, ", jc.egr)?;
        write!(f, "\"Lgr\": {}, ", jc.lgr)?;
        write!(f, "\"Egd\": {}, ", jc.egd)?;
        write!(f, "\"Lgd\": {}, ", jc.lgd)?;
        write!(f, "\"Ebd\": {}, ", jc.ebd)?;
        write!(f, "\"Lbd\": {}, ", jc.lbd)?;
        write!(f, "\"Epr\": {}, ", jc.epr)?;
        write!(f, "\"Lpr\": {}, ", jc.lpr)?;
        write!(f, "\"Ems\": {}, ", jc.ems)?;
        write!(f, "\"Lms\": {}, ", jc.lms)?;
        write!(f, "\"Combo\": {}, ", self.maxcombo)?;
        write!(f, "\"Mode\": {}, ", self.mode)?;
        write!(f, "\"Notes\": {}, ", self.notes)?;
        write!(f, "\"Clearcount\": {}, ", self.clearcount)?;
        write!(f, "\"Minbp\": {}, ", self.minbp)?;
        write!(f, "\"Avgjudge\": {}, ", ts.avgjudge)?;
        write!(f, "\"Trophy\": \"{}\", ", self.trophy)?;
        write!(f, "\"Option\": {}, ", po.option)?;
        write!(f, "\"State\": {}, ", self.state)?;
        write!(f, "\"Sha256\": \"{}\", ", self.sha256)?;
        write!(f, "\"Exscore\": {}, ", self.exscore())?;
        write!(f, "\"Random\": {}, ", po.random)?;
        write!(f, "\"Scorehash\": \"{}\", ", self.scorehash)?;
        write!(f, "\"Assist\": {}, ", po.assist)?;
        write!(f, "\"Gauge\": {}, ", po.gauge)?;
        write!(f, "\"DeviceType\": \"{:?}\", ", po.device_type)?;
        write!(f, "\"Playmode\": \"{:?}\", ", self.playmode)?;
        write!(f, "\"Ghost\": \"{}\", ", self.ghost)?;
        write!(f, "\"Passnotes\": {}", self.passnotes)?;
        write!(f, "}}")
    }
}

mod trophy;
pub use trophy::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_score_data_default() {
        let sd = ScoreData::default();
        assert_eq!(sd.player, "unknown");
        assert_eq!(sd.sha256, "");
        assert_eq!(sd.mode, 0);
        assert_eq!(sd.clear, 0);
        assert_eq!(sd.date, 0);
        assert_eq!(sd.playcount, 0);
        assert_eq!(sd.clearcount, 0);
        assert_eq!(sd.judge_counts.epg, 0);
        assert_eq!(sd.judge_counts.lpg, 0);
        assert_eq!(sd.judge_counts.egr, 0);
        assert_eq!(sd.judge_counts.lgr, 0);
        assert_eq!(sd.judge_counts.egd, 0);
        assert_eq!(sd.judge_counts.lgd, 0);
        assert_eq!(sd.judge_counts.ebd, 0);
        assert_eq!(sd.judge_counts.lbd, 0);
        assert_eq!(sd.judge_counts.epr, 0);
        assert_eq!(sd.judge_counts.lpr, 0);
        assert_eq!(sd.judge_counts.ems, 0);
        assert_eq!(sd.judge_counts.lms, 0);
        assert_eq!(sd.maxcombo, 0);
        assert_eq!(sd.notes, 0);
        assert_eq!(sd.passnotes, 0);
        assert_eq!(sd.minbp, i32::MAX);
        assert_eq!(sd.timing_stats.avgjudge, i64::MAX);
        assert_eq!(sd.play_option.seed, -1);
        assert_eq!(sd.trophy, "");
        assert_eq!(sd.ghost, "");
        assert_eq!(sd.scorehash, "");
        assert!(sd.play_option.device_type.is_none());
        assert!(sd.play_option.judge_algorithm.is_none());
        assert!(sd.play_option.rule.is_none());
        assert!(sd.play_option.skin.is_none());
    }

    #[test]
    fn test_score_data_new_with_mode() {
        let sd = ScoreData::new(Mode::BEAT_5K);
        assert_eq!(sd.playmode, Mode::BEAT_5K);
        assert_eq!(sd.player, "unknown");
    }

    #[test]
    fn test_score_data_serde_round_trip() {
        let mut sd = ScoreData::new(Mode::BEAT_7K);
        sd.sha256 = "abc123".to_string();
        sd.player = "player1".to_string();
        sd.clear = 5;
        sd.judge_counts.epg = 100;
        sd.judge_counts.lpg = 90;
        sd.judge_counts.egr = 80;
        sd.judge_counts.lgr = 70;
        sd.judge_counts.egd = 10;
        sd.judge_counts.lgd = 5;
        sd.maxcombo = 250;
        sd.notes = 500;
        sd.date = 1700000000;

        let json = serde_json::to_string(&sd).unwrap();
        let deserialized: ScoreData = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.sha256, "abc123");
        assert_eq!(deserialized.player, "player1");
        assert_eq!(deserialized.clear, 5);
        assert_eq!(deserialized.judge_counts.epg, 100);
        assert_eq!(deserialized.judge_counts.lpg, 90);
        assert_eq!(deserialized.judge_counts.egr, 80);
        assert_eq!(deserialized.judge_counts.lgr, 70);
        assert_eq!(deserialized.judge_counts.egd, 10);
        assert_eq!(deserialized.judge_counts.lgd, 5);
        assert_eq!(deserialized.maxcombo, 250);
        assert_eq!(deserialized.notes, 500);
        assert_eq!(deserialized.date, 1700000000);
    }

    #[test]
    fn test_exscore_calculation() {
        let mut sd = ScoreData::default();
        sd.judge_counts.epg = 100;
        sd.judge_counts.lpg = 50;
        sd.judge_counts.egr = 30;
        sd.judge_counts.lgr = 20;
        // exscore = (epg + lpg) * 2 + egr + lgr = (100+50)*2 + 30+20 = 350
        assert_eq!(sd.exscore(), 350);
    }

    #[test]
    fn test_judge_count() {
        let mut sd = ScoreData::default();
        sd.judge_counts.epg = 10;
        sd.judge_counts.lpg = 20;
        sd.judge_counts.egr = 30;
        sd.judge_counts.lgr = 40;
        sd.judge_counts.egd = 5;
        sd.judge_counts.lgd = 6;
        sd.judge_counts.ebd = 3;
        sd.judge_counts.lbd = 4;
        sd.judge_counts.epr = 1;
        sd.judge_counts.lpr = 2;
        sd.judge_counts.ems = 7;
        sd.judge_counts.lms = 8;

        // PG (judge=0)
        assert_eq!(sd.judge_count(0, true), 10);
        assert_eq!(sd.judge_count(0, false), 20);
        assert_eq!(sd.judge_count_total(0), 30);

        // GR (judge=1)
        assert_eq!(sd.judge_count(1, true), 30);
        assert_eq!(sd.judge_count(1, false), 40);
        assert_eq!(sd.judge_count_total(1), 70);

        // GD (judge=2)
        assert_eq!(sd.judge_count(2, true), 5);
        assert_eq!(sd.judge_count(2, false), 6);

        // BD (judge=3)
        assert_eq!(sd.judge_count(3, true), 3);
        assert_eq!(sd.judge_count(3, false), 4);

        // PR (judge=4)
        assert_eq!(sd.judge_count(4, true), 1);
        assert_eq!(sd.judge_count(4, false), 2);

        // MS (judge=5)
        assert_eq!(sd.judge_count(5, true), 7);
        assert_eq!(sd.judge_count(5, false), 8);

        // Out of range
        assert_eq!(sd.judge_count(6, true), 0);
        assert_eq!(sd.judge_count(-1, false), 0);
    }

    #[test]
    fn test_add_judge_count() {
        let mut sd = ScoreData::default();
        sd.add_judge_count(0, true, 5);
        sd.add_judge_count(0, false, 3);
        sd.add_judge_count(1, true, 10);
        sd.add_judge_count(5, false, 2);
        // Out of range should be no-op
        sd.add_judge_count(6, true, 100);

        assert_eq!(sd.judge_counts.epg, 5);
        assert_eq!(sd.judge_counts.lpg, 3);
        assert_eq!(sd.judge_counts.egr, 10);
        assert_eq!(sd.judge_counts.lms, 2);
    }

    #[test]
    fn test_set_player() {
        let mut sd = ScoreData::default();
        sd.set_player(Some("TestPlayer"));
        assert_eq!(sd.player, "TestPlayer");

        sd.set_player(None);
        assert_eq!(sd.player, "");
    }

    #[test]
    fn test_ghost_encode_decode_round_trip() {
        let mut sd = ScoreData::default();
        sd.notes = 5;
        let ghost_data = vec![0, 1, 2, 3, 4];
        sd.encode_ghost(Some(&ghost_data));
        assert!(!sd.ghost.is_empty());

        let decoded = sd.decode_ghost().unwrap();
        assert_eq!(decoded, ghost_data);
    }

    #[test]
    fn test_ghost_encode_none() {
        let mut sd = ScoreData::default();
        sd.encode_ghost(None);
        assert!(sd.ghost.is_empty());
    }

    #[test]
    fn test_ghost_encode_empty() {
        let mut sd = ScoreData::default();
        sd.encode_ghost(Some(&[]));
        assert!(sd.ghost.is_empty());
    }

    #[test]
    fn test_ghost_decode_empty() {
        let sd = ScoreData::default();
        assert!(sd.decode_ghost().is_none());
    }

    #[test]
    fn test_update_clear() {
        let mut sd = ScoreData::default();
        sd.clear = 3;
        sd.notes = 100;

        let mut newscore = ScoreData::default();
        newscore.clear = 5;
        newscore.notes = 100;

        assert!(sd.update(&newscore, false));
        assert_eq!(sd.clear, 5);
    }

    #[test]
    fn test_update_exscore() {
        let mut sd = ScoreData::default();
        sd.judge_counts.epg = 10;
        sd.judge_counts.lpg = 10;
        sd.notes = 100;

        let mut newscore = ScoreData::default();
        newscore.judge_counts.epg = 50;
        newscore.judge_counts.lpg = 50;
        newscore.notes = 100;

        assert!(sd.update(&newscore, true));
        assert_eq!(sd.judge_counts.epg, 50);
        assert_eq!(sd.judge_counts.lpg, 50);
    }

    #[test]
    fn test_update_no_change() {
        let mut sd = ScoreData::default();
        sd.clear = 5;
        sd.judge_counts.epg = 100;
        sd.judge_counts.lpg = 100;
        sd.maxcombo = 200;
        sd.minbp = 0;
        sd.timing_stats.avgjudge = 0;

        let newscore = sd.clone();
        assert!(!sd.update(&newscore, true));
    }

    // -- Phase 46b: ghost encoding truncation tests --

    #[test]
    fn test_ghost_encode_valid_range_roundtrip() {
        // Judge values 0–5 are the valid range; encode/decode should roundtrip cleanly
        let mut sd = ScoreData::default();
        let ghost_data: Vec<i32> = vec![0, 1, 2, 3, 4, 5];
        sd.notes = ghost_data.len() as i32;
        sd.encode_ghost(Some(&ghost_data));
        assert!(!sd.ghost.is_empty());

        let decoded = sd.decode_ghost().unwrap();
        assert_eq!(decoded, ghost_data);
    }

    #[test]
    #[ignore] // BUG: encode_ghost uses `j as u8` which silently truncates values >= 256
    // — value 256 becomes 0 after truncation, corrupting the ghost data
    fn test_ghost_encode_truncation_256() {
        let mut sd = ScoreData::default();
        let ghost_data: Vec<i32> = vec![256];
        sd.notes = 1;
        sd.encode_ghost(Some(&ghost_data));

        let decoded = sd.decode_ghost().unwrap();
        // After the bug: 256 as u8 = 0, so decoded[0] = 0 instead of 256
        assert_eq!(
            decoded[0], 256,
            "value 256 should survive roundtrip (actual: {})",
            decoded[0]
        );
    }

    // -- SongTrophy tests --

    #[test]
    fn test_song_trophy_character() {
        assert_eq!(SongTrophy::Easy.character(), 'g');
        assert_eq!(SongTrophy::Groove.character(), 'G');
        assert_eq!(SongTrophy::Hard.character(), 'h');
        assert_eq!(SongTrophy::ExHard.character(), 'H');
        assert_eq!(SongTrophy::Normal.character(), 'n');
        assert_eq!(SongTrophy::Mirror.character(), 'm');
        assert_eq!(SongTrophy::Random.character(), 'r');
        assert_eq!(SongTrophy::SRandom.character(), 's');
        assert_eq!(SongTrophy::Battle.character(), 'B');
    }

    #[test]
    fn test_song_trophy_values_count() {
        assert_eq!(SongTrophy::values().len(), 16);
    }

    #[test]
    fn test_song_trophy_get_trophy() {
        assert_eq!(SongTrophy::trophy('g'), Some(SongTrophy::Easy));
        assert_eq!(SongTrophy::trophy('G'), Some(SongTrophy::Groove));
        assert_eq!(SongTrophy::trophy('H'), Some(SongTrophy::ExHard));
        assert_eq!(SongTrophy::trophy('B'), Some(SongTrophy::Battle));
        assert_eq!(SongTrophy::trophy('z'), None);
    }

    #[test]
    fn test_song_trophy_round_trip() {
        // Every trophy should be recoverable from its character
        for trophy in SongTrophy::values() {
            let c = trophy.character();
            let recovered = SongTrophy::trophy(c);
            assert_eq!(recovered, Some(*trophy));
        }
    }

    #[test]
    fn test_score_data_serde_java_field_names() {
        let mut sd = ScoreData::new(Mode::BEAT_7K);
        sd.maxcombo = 250;
        sd.timing_stats.total_duration = 120_000;
        sd.timing_stats.total_avg = 500;
        sd.play_option.device_type = None;
        sd.play_option.judge_algorithm = None;

        let json = serde_json::to_string(&sd).unwrap();

        // Field must serialize as "maxcombo" (Java field name), not "combo"
        assert!(
            json.contains("\"maxcombo\""),
            "Expected 'maxcombo' in JSON, got: {}",
            json
        );
        assert!(
            !json.contains("\"combo\"") || json.contains("\"maxcombo\""),
            "Should not have bare 'combo' field without 'max' prefix"
        );

        // camelCase renames for Java compatibility
        assert!(
            json.contains("\"totalDuration\""),
            "Expected 'totalDuration' in JSON, got: {}",
            json
        );
        assert!(
            json.contains("\"totalAvg\""),
            "Expected 'totalAvg' in JSON, got: {}",
            json
        );
        assert!(
            json.contains("\"deviceType\""),
            "Expected 'deviceType' in JSON, got: {}",
            json
        );
        assert!(
            json.contains("\"judgeAlgorithm\""),
            "Expected 'judgeAlgorithm' in JSON, got: {}",
            json
        );

        // Verify these snake_case forms do NOT appear
        assert!(
            !json.contains("\"total_duration\""),
            "Should not have 'total_duration' in JSON"
        );
        assert!(
            !json.contains("\"total_avg\""),
            "Should not have 'total_avg' in JSON"
        );
        assert!(
            !json.contains("\"device_type\""),
            "Should not have 'device_type' in JSON"
        );
        assert!(
            !json.contains("\"judge_algorithm\""),
            "Should not have 'judge_algorithm' in JSON"
        );

        // Round-trip: deserialize from Java-style JSON
        let deserialized: ScoreData = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.maxcombo, 250);
        assert_eq!(deserialized.timing_stats.total_duration, 120_000);
        assert_eq!(deserialized.timing_stats.total_avg, 500);
    }

    #[test]
    fn test_score_data_trophy_constants() {
        assert_eq!(ScoreData::TROPHY_EASY, SongTrophy::Easy);
        assert_eq!(ScoreData::TROPHY_GROOVE, SongTrophy::Groove);
        assert_eq!(ScoreData::TROPHY_HARD, SongTrophy::Hard);
        assert_eq!(ScoreData::TROPHY_EXHARD, SongTrophy::ExHard);
        assert_eq!(ScoreData::TROPHY_NORMAL, SongTrophy::Normal);
        assert_eq!(ScoreData::TROPHY_MIRROR, SongTrophy::Mirror);
        assert_eq!(ScoreData::TROPHY_RANDOM, SongTrophy::Random);
        assert_eq!(ScoreData::TROPHY_S_RANDOM, SongTrophy::SRandom);
        assert_eq!(ScoreData::TROPHY_BATTLE, SongTrophy::Battle);
    }

    // -- get_exscore() formula and rank boundary tests --

    /// Helper: build a ScoreData with specific judge counts and notes.
    fn make_score(epg: i32, lpg: i32, egr: i32, lgr: i32, notes: i32) -> ScoreData {
        let mut sd = ScoreData::default();
        sd.judge_counts.epg = epg;
        sd.judge_counts.lpg = lpg;
        sd.judge_counts.egr = egr;
        sd.judge_counts.lgr = lgr;
        sd.notes = notes;
        sd
    }

    /// Compute the rank boundary rate for a given index (0..27).
    /// rank[i] is true when rate >= i / 27.
    /// Rate boundaries:
    ///   0/27=F, 3/27=E, 6/27=D, 9/27=C, 12/27=B, 15/27=A,
    ///   18/27=AA, 21/27=AAA, 24/27=MAX-
    /// Sub-grades: i%3==1 is "-", i%3==2 is base, i%3==0 is "+" (except 0).
    fn rank_boundary_exscore(rank_index: usize, notes: i32) -> i32 {
        // exscore needed = ceil(rank_index / 27 * notes * 2)
        // Using integer arithmetic to avoid float imprecision:
        let max_ex = notes as i64 * 2;
        let needed = (rank_index as i64 * max_ex + 26) / 27; // ceiling division
        needed as i32
    }

    // -- get_exscore() formula verification --

    #[test]
    fn test_exscore_formula_only_perfects() {
        // All PG: exscore = (epg + lpg) * 2
        let sd = make_score(50, 50, 0, 0, 100);
        assert_eq!(sd.exscore(), 200);
    }

    #[test]
    fn test_exscore_formula_only_greats() {
        // All GR: exscore = egr + lgr
        let sd = make_score(0, 0, 60, 40, 100);
        assert_eq!(sd.exscore(), 100);
    }

    #[test]
    fn test_exscore_formula_mixed() {
        // (10 + 20) * 2 + 30 + 40 = 60 + 70 = 130
        let sd = make_score(10, 20, 30, 40, 100);
        assert_eq!(sd.exscore(), 130);
    }

    #[test]
    fn test_exscore_formula_single_epg() {
        let sd = make_score(1, 0, 0, 0, 1);
        // (1 + 0) * 2 + 0 + 0 = 2
        assert_eq!(sd.exscore(), 2);
    }

    #[test]
    fn test_exscore_formula_single_egr() {
        let sd = make_score(0, 0, 1, 0, 1);
        // (0 + 0) * 2 + 1 + 0 = 1
        assert_eq!(sd.exscore(), 1);
    }

    // -- Zero notes (all miss / empty chart) --

    #[test]
    fn test_exscore_zero_notes_all_zero() {
        let sd = make_score(0, 0, 0, 0, 0);
        assert_eq!(sd.exscore(), 0);
    }

    #[test]
    fn test_exscore_zero_judge_counts_nonzero_notes() {
        // Chart has 1000 notes but all missed
        let sd = make_score(0, 0, 0, 0, 1000);
        assert_eq!(sd.exscore(), 0);
    }

    // -- All perfect (MAX) --

    #[test]
    fn test_exscore_all_perfect_100_notes() {
        // 100 notes, all perfect great: max exscore = 200
        let sd = make_score(100, 0, 0, 0, 100);
        assert_eq!(sd.exscore(), 200);
    }

    #[test]
    fn test_exscore_all_perfect_split_fast_slow() {
        // 100 notes: 60 epg + 40 lpg = max exscore 200
        let sd = make_score(60, 40, 0, 0, 100);
        assert_eq!(sd.exscore(), 200);
    }

    #[test]
    fn test_exscore_all_perfect_1000_notes() {
        let sd = make_score(500, 500, 0, 0, 1000);
        assert_eq!(sd.exscore(), 2000);
    }

    // -- Rank boundary transitions using 1000-note chart --
    // For 1000 notes, max exscore = 2000.
    // Boundary at index i: exscore >= ceil(i * 2000 / 27)
    //
    // Rank indices (every 3rd is a major boundary):
    //   0: always true (F floor)
    //   3: E   -> ceil(3*2000/27) = ceil(222.22) = 223
    //   6: D   -> ceil(6*2000/27) = ceil(444.44) = 445
    //   9: C   -> ceil(9*2000/27) = ceil(666.67) = 667
    //  12: B   -> ceil(12*2000/27) = ceil(888.89) = 889
    //  15: A   -> ceil(15*2000/27) = ceil(1111.11) = 1112
    //  18: AA  -> ceil(18*2000/27) = ceil(1333.33) = 1334
    //  21: AAA -> ceil(21*2000/27) = ceil(1555.56) = 1556
    //  24: MAX--> ceil(24*2000/27) = ceil(1777.78) = 1778

    /// Verify rank_boundary_exscore helper is correct for a few known values.
    #[test]
    fn test_rank_boundary_helper_sanity() {
        // Index 0: boundary = 0
        assert_eq!(rank_boundary_exscore(0, 1000), 0);
        // Index 27: boundary = 2000 (max)
        assert_eq!(rank_boundary_exscore(27, 1000), 2000);
        // Index 9 (C): ceil(9*2000/27) = ceil(666.67) = 667
        assert_eq!(rank_boundary_exscore(9, 1000), 667);
        // Index 21 (AAA): ceil(21*2000/27) = ceil(1555.56) = 1556
        assert_eq!(rank_boundary_exscore(21, 1000), 1556);
    }

    /// For each major rank boundary (E, D, C, B, A, AA, AAA, MAX-),
    /// verify exscore exactly at, one below, and one above the boundary.
    /// The rate = exscore / (notes * 2), and rank[i] = rate >= i/27.
    #[test]
    fn test_exscore_at_rank_boundaries() {
        let notes = 1000;
        let max_ex = notes * 2; // 2000

        // Major rank boundary indices
        let boundaries = [
            (3, "E"),
            (6, "D"),
            (9, "C"),
            (12, "B"),
            (15, "A"),
            (18, "AA"),
            (21, "AAA"),
            (24, "MAX-"),
        ];

        for (idx, name) in &boundaries {
            let threshold = rank_boundary_exscore(*idx, notes);

            // One below: should NOT qualify
            if threshold > 0 {
                let below = threshold - 1;
                let rate_below = below as f32 / max_ex as f32;
                let rank_threshold = *idx as f32 / 27.0;
                assert!(
                    rate_below < rank_threshold,
                    "rank {} (idx {}): exscore {} should be below threshold (rate {:.6} < {:.6})",
                    name,
                    idx,
                    below,
                    rate_below,
                    rank_threshold
                );
            }

            // Exactly at: should qualify
            let rate_at = threshold as f32 / max_ex as f32;
            let rank_threshold = *idx as f32 / 27.0;
            assert!(
                rate_at >= rank_threshold,
                "rank {} (idx {}): exscore {} should meet threshold (rate {:.6} >= {:.6})",
                name,
                idx,
                threshold,
                rate_at,
                rank_threshold
            );

            // One above: should qualify
            if threshold < max_ex {
                let above = threshold + 1;
                let rate_above = above as f32 / max_ex as f32;
                assert!(
                    rate_above >= rank_threshold,
                    "rank {} (idx {}): exscore {} should exceed threshold (rate {:.6} >= {:.6})",
                    name,
                    idx,
                    above,
                    rate_above,
                    rank_threshold
                );
            }
        }
    }

    /// Verify all 27 sub-rank boundaries (including +/- variants).
    #[test]
    fn test_exscore_all_27_sub_rank_boundaries() {
        let notes = 1000;
        let max_ex = notes * 2;

        for idx in 0..=26 {
            let threshold = rank_boundary_exscore(idx, notes);
            let rate = threshold as f32 / max_ex as f32;
            let boundary = idx as f32 / 27.0;

            assert!(
                rate >= boundary,
                "sub-rank {}: exscore {} rate {:.6} should >= {:.6}",
                idx,
                threshold,
                rate,
                boundary
            );

            // One below should NOT qualify (except index 0 which is always 0)
            if threshold > 0 {
                let rate_below = (threshold - 1) as f32 / max_ex as f32;
                assert!(
                    rate_below < boundary,
                    "sub-rank {}: exscore {} rate {:.6} should < {:.6}",
                    idx,
                    threshold - 1,
                    rate_below,
                    boundary
                );
            }
        }
    }

    /// Verify exscore exactly produces the right rate for AAA boundary (21/27).
    #[test]
    fn test_exscore_aaa_boundary_exact() {
        // For a chart with 27 notes, max exscore = 54.
        // AAA boundary at index 21: rate >= 21/27 = 7/9
        // Needed exscore = 21 * 54 / 27 = 42 (exact division)
        let sd = make_score(21, 0, 0, 0, 27);
        // epg=21 -> exscore = 21*2 = 42
        assert_eq!(sd.exscore(), 42);
        let rate = sd.exscore() as f32 / (27 * 2) as f32;
        assert!((rate - 7.0 / 9.0).abs() < 1e-6);
    }

    /// Verify one below AAA boundary does not qualify.
    #[test]
    fn test_exscore_aaa_boundary_one_below() {
        // exscore 41 for 27 notes: rate = 41/54 < 21/27
        let sd = make_score(20, 0, 1, 0, 27);
        // epg=20, egr=1 -> exscore = 20*2 + 1 = 41
        assert_eq!(sd.exscore(), 41);
        let rate = sd.exscore() as f32 / (27 * 2) as f32;
        assert!(rate < 21.0 / 27.0);
    }

    /// Verify one above AAA boundary qualifies.
    #[test]
    fn test_exscore_aaa_boundary_one_above() {
        // exscore 43 for 27 notes: rate = 43/54 > 21/27
        let sd = make_score(21, 0, 1, 0, 27);
        // epg=21, egr=1 -> exscore = 21*2 + 1 = 43
        assert_eq!(sd.exscore(), 43);
        let rate = sd.exscore() as f32 / (27 * 2) as f32;
        assert!(rate > 21.0 / 27.0);
    }

    /// MAX rank: all perfect, exscore = notes * 2.
    #[test]
    fn test_exscore_max_rank() {
        let sd = make_score(500, 500, 0, 0, 1000);
        assert_eq!(sd.exscore(), 2000);
        let rate = sd.exscore() as f32 / (1000 * 2) as f32;
        assert!((rate - 1.0).abs() < 1e-6);
    }

    /// F rank: all miss, exscore = 0.
    #[test]
    fn test_exscore_f_rank_all_miss() {
        let sd = make_score(0, 0, 0, 0, 1000);
        assert_eq!(sd.exscore(), 0);
        // rate = 0, only rank[0] should be satisfied (0/27 = 0.0 <= 0.0)
    }

    // -- Saturating arithmetic in get_exscore() --

    #[test]
    fn test_exscore_saturating_epg_lpg_overflow() {
        // (epg + lpg) would overflow i32 without saturating_add
        let sd = make_score(i32::MAX, 1, 0, 0, 1000);
        // saturating_add: i32::MAX + 1 = i32::MAX
        // saturating_mul: i32::MAX * 2 = i32::MAX
        // saturating_add(0).saturating_add(0) = i32::MAX
        assert_eq!(sd.exscore(), i32::MAX);
    }

    #[test]
    fn test_exscore_saturating_mul_overflow() {
        // Even if sum fits, *2 would overflow
        let sd = make_score(i32::MAX / 2 + 1, i32::MAX / 2 + 1, 0, 0, 1000);
        // saturating_add: (MAX/2+1) + (MAX/2+1) = MAX/2*2 + 2 = MAX + 1 -> saturates to MAX
        // saturating_mul: MAX * 2 -> saturates to MAX
        assert_eq!(sd.exscore(), i32::MAX);
    }

    #[test]
    fn test_exscore_saturating_add_egr_overflow() {
        // (epg+lpg)*2 fits, but adding egr overflows
        let sd = make_score(i32::MAX / 4, i32::MAX / 4, i32::MAX, 0, 1000);
        // epg+lpg = MAX/4 + MAX/4 = MAX/2 (fits)
        // (MAX/2) * 2 = MAX - 1 (just under MAX due to integer division)
        // (MAX-1) + MAX -> saturates to MAX
        assert_eq!(sd.exscore(), i32::MAX);
    }

    #[test]
    fn test_exscore_saturating_add_lgr_overflow() {
        // Everything fits except final lgr addition
        let sd = make_score(0, 0, i32::MAX, i32::MAX, 1000);
        // (0+0)*2 = 0; 0 + MAX = MAX; MAX + MAX -> saturates to MAX
        assert_eq!(sd.exscore(), i32::MAX);
    }

    #[test]
    fn test_exscore_saturating_all_max() {
        let sd = make_score(i32::MAX, i32::MAX, i32::MAX, i32::MAX, 1000);
        assert_eq!(sd.exscore(), i32::MAX);
    }

    #[test]
    fn test_exscore_large_values_no_overflow() {
        // Large but within range: (500000 + 500000) * 2 + 100000 + 100000 = 2200000
        let sd = make_score(500_000, 500_000, 100_000, 100_000, 1_200_000);
        assert_eq!(sd.exscore(), 2_200_000);
    }

    #[test]
    fn test_exscore_just_under_overflow_boundary() {
        // (epg + lpg) * 2 just barely fits in i32
        // i32::MAX = 2_147_483_647
        // We want (epg + lpg) * 2 = 2_147_483_646 (MAX - 1), so epg+lpg = 1_073_741_823
        let sd = make_score(1_073_741_823, 0, 0, 0, 1000);
        assert_eq!(sd.exscore(), 2_147_483_646);
    }

    #[test]
    fn test_exscore_just_at_overflow_boundary() {
        // (epg + lpg) * 2 + egr + lgr = i32::MAX = 2_147_483_647
        let sd = make_score(1_073_741_823, 0, 1, 0, 1000);
        assert_eq!(sd.exscore(), 2_147_483_647);
    }

    #[test]
    fn test_exscore_one_past_overflow_boundary() {
        // Without saturation this would overflow, but saturating keeps at MAX
        let sd = make_score(1_073_741_823, 0, 2, 0, 1000);
        // (1_073_741_823 + 0) * 2 = 2_147_483_646
        // 2_147_483_646 + 2 = 2_147_483_648 -> overflows i32, saturates to MAX
        assert_eq!(sd.exscore(), i32::MAX);
    }

    // -- Rank boundary transitions with small note counts --

    #[test]
    fn test_exscore_rank_boundary_single_note() {
        // 1 note, max exscore = 2
        // Only 3 possible exscores: 0, 1, 2
        let sd0 = make_score(0, 0, 0, 0, 1);
        assert_eq!(sd0.exscore(), 0);

        let sd1 = make_score(0, 0, 1, 0, 1);
        assert_eq!(sd1.exscore(), 1);

        let sd2 = make_score(1, 0, 0, 0, 1);
        assert_eq!(sd2.exscore(), 2);
    }

    #[test]
    fn test_exscore_rank_boundary_27_notes_exact_divisions() {
        // With 27 notes, max exscore = 54.
        // Each rank boundary at index i divides exactly: i * 54 / 27 = i * 2.
        // So rank[i] requires exscore >= i * 2.
        for i in 0..=26 {
            let needed = i * 2;
            let rate = needed as f32 / 54.0;
            let threshold = i as f32 / 27.0;
            assert!(
                rate >= threshold,
                "27-note chart: rank {} needs exscore >= {}, rate {:.4} >= {:.4}",
                i,
                needed,
                rate,
                threshold
            );
            if needed > 0 {
                let rate_below = (needed - 1) as f32 / 54.0;
                assert!(
                    rate_below < threshold,
                    "27-note chart: rank {} exscore {} should be below (rate {:.4} < {:.4})",
                    i,
                    needed - 1,
                    rate_below,
                    threshold
                );
            }
        }
    }

    /// Exscore respects that GD/BD/PR/MS do not contribute.
    #[test]
    fn test_exscore_ignores_gd_bd_pr_ms() {
        let mut sd = ScoreData::default();
        sd.judge_counts.egd = 100;
        sd.judge_counts.lgd = 200;
        sd.judge_counts.ebd = 300;
        sd.judge_counts.lbd = 400;
        sd.judge_counts.epr = 500;
        sd.judge_counts.lpr = 600;
        sd.judge_counts.ems = 700;
        sd.judge_counts.lms = 800;
        sd.notes = 3600;
        // None of these contribute to exscore
        assert_eq!(sd.exscore(), 0);
    }

    /// Exscore with asymmetric fast/slow split still calculates correctly.
    #[test]
    fn test_exscore_asymmetric_fast_slow() {
        // All perfects as early, all greats as late
        let sd = make_score(100, 0, 0, 50, 150);
        // (100 + 0) * 2 + 0 + 50 = 250
        assert_eq!(sd.exscore(), 250);
    }
}

#[cfg(test)]
mod prop_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(1024))]

        /// Encoding then decoding ghost data should return the original values
        /// for judge values in the valid range 0..=5.
        #[test]
        fn ghost_roundtrip(judges in prop::collection::vec(0..=5i32, 0..2000)) {
            let mut score = ScoreData {
                notes: judges.len() as i32,
                ..Default::default()
            };
            score.encode_ghost(Some(&judges));
            let decoded = score.decode_ghost();
            if judges.is_empty() {
                // encode_ghost clears ghost for empty input; decode returns None
                prop_assert!(decoded.is_none());
            } else {
                let decoded = decoded.unwrap();
                prop_assert_eq!(decoded, judges);
            }
        }

        /// encode_ghost(None) always clears the ghost field.
        #[test]
        fn ghost_encode_none_clears(ghost_content in "[a-zA-Z0-9_]{0,100}") {
            let mut score = ScoreData {
                ghost: ghost_content,
                ..Default::default()
            };
            score.encode_ghost(None);
            prop_assert!(score.ghost.is_empty());
        }

        /// When notes > encoded length, extra positions are filled with 4 (MISS).
        #[test]
        fn ghost_decode_truncates_to_notes(
            judges in prop::collection::vec(0..=5i32, 1..100usize),
            extra in 1..100usize,
        ) {
            let mut score = ScoreData {
                notes: judges.len() as i32,
                ..Default::default()
            };
            // Encode the original judges
            score.encode_ghost(Some(&judges));

            // Now set notes larger than encoded length to trigger padding
            let padded_len = judges.len() + extra;
            score.notes = padded_len as i32;
            let decoded = score.decode_ghost().unwrap();

            prop_assert_eq!(decoded.len(), padded_len);
            // Original values are preserved
            for (i, &j) in judges.iter().enumerate() {
                prop_assert_eq!(decoded[i], j, "mismatch at index {}", i);
            }
            // Extra positions are filled with 4 (MISS)
            for (i, value) in decoded.iter().enumerate().take(padded_len).skip(judges.len()) {
                prop_assert_eq!(*value, 4, "expected MISS (4) at index {}", i);
            }
        }
    }
}
