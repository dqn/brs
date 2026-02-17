// Bar sorting — sort and filter methods for BarManager.

use std::collections::HashMap;

use bms_rule::ScoreData;

use super::bar_types::Bar;
use super::{BarManager, SortMode};

impl BarManager {
    /// Sort bars by the given mode.
    ///
    /// Sort order for non-Song bars: Folders first, then Courses (by name).
    /// Score-dependent modes (Clear, Score, MissCount, Duration, LastUpdate) use
    /// the `score_cache` keyed by SHA-256.
    pub fn sort(&mut self, mode: SortMode, score_cache: &HashMap<String, ScoreData>) {
        match mode {
            SortMode::Default => {} // Keep original order
            SortMode::Title => {
                self.bars.sort_by(|a, b| {
                    a.bar_name()
                        .to_lowercase()
                        .cmp(&b.bar_name().to_lowercase())
                });
            }
            SortMode::Artist => {
                self.bars.sort_by(|a, b| {
                    let artist_a = match a {
                        Bar::Song(s) => s.artist.as_str(),
                        _ => "",
                    };
                    let artist_b = match b {
                        Bar::Song(s) => s.artist.as_str(),
                        _ => "",
                    };
                    artist_a.to_lowercase().cmp(&artist_b.to_lowercase())
                });
            }
            SortMode::Level => {
                self.bars.sort_by(|a, b| {
                    let level_a = match a {
                        Bar::Song(s) => s.level,
                        _ => 0,
                    };
                    let level_b = match b {
                        Bar::Song(s) => s.level,
                        _ => 0,
                    };
                    level_a.cmp(&level_b)
                });
            }
            SortMode::Bpm => {
                self.bars.sort_by(|a, b| {
                    let bpm_a = match a {
                        Bar::Song(s) => s.maxbpm,
                        _ => 0,
                    };
                    let bpm_b = match b {
                        Bar::Song(s) => s.maxbpm,
                        _ => 0,
                    };
                    bpm_a.cmp(&bpm_b)
                });
            }
            SortMode::Length => {
                self.bars.sort_by(|a, b| {
                    let len_a = match a {
                        Bar::Song(s) => s.length,
                        _ => 0,
                    };
                    let len_b = match b {
                        Bar::Song(s) => s.length,
                        _ => 0,
                    };
                    len_a.cmp(&len_b)
                });
            }
            SortMode::Clear => {
                self.bars.sort_by(|a, b| {
                    let clear_a = bar_score_field(a, score_cache, |sd| sd.clear.id() as i32, 0);
                    let clear_b = bar_score_field(b, score_cache, |sd| sd.clear.id() as i32, 0);
                    clear_a.cmp(&clear_b)
                });
            }
            SortMode::Score => {
                self.bars.sort_by(|a, b| {
                    let score_a = bar_score_field(a, score_cache, |sd| sd.exscore(), 0);
                    let score_b = bar_score_field(b, score_cache, |sd| sd.exscore(), 0);
                    score_b.cmp(&score_a) // Descending
                });
            }
            SortMode::MissCount => {
                self.bars.sort_by(|a, b| {
                    let bp_a = bar_score_field(a, score_cache, |sd| sd.minbp, i32::MAX);
                    let bp_b = bar_score_field(b, score_cache, |sd| sd.minbp, i32::MAX);
                    bp_a.cmp(&bp_b) // Ascending (fewer misses first)
                });
            }
            SortMode::Duration => {
                self.bars.sort_by(|a, b| {
                    let pc_a = bar_score_field(a, score_cache, |sd| sd.playcount, 0);
                    let pc_b = bar_score_field(b, score_cache, |sd| sd.playcount, 0);
                    pc_b.cmp(&pc_a) // Descending (most played first)
                });
            }
            SortMode::LastUpdate => {
                self.bars.sort_by(|a, b| {
                    let date_a = bar_score_field_i64(a, score_cache, |sd| sd.date, 0);
                    let date_b = bar_score_field_i64(b, score_cache, |sd| sd.date, 0);
                    date_b.cmp(&date_a) // Descending (most recent first)
                });
            }
        }
        self.cursor = 0;
    }

    /// Filter bars to retain only songs matching the given mode ID.
    /// Non-Song bars are always retained.
    pub fn filter_by_mode(&mut self, mode: Option<i32>) {
        if let Some(mode_id) = mode {
            self.bars.retain(|bar| match bar {
                Bar::Song(s) => s.mode == mode_id,
                _ => true,
            });
            self.cursor = 0;
        }
    }
}

/// Extract an i32 field from a score associated with a Bar::Song.
fn bar_score_field(
    bar: &Bar,
    cache: &HashMap<String, ScoreData>,
    extract: impl Fn(&ScoreData) -> i32,
    default: i32,
) -> i32 {
    match bar {
        Bar::Song(s) => cache.get(&s.sha256).map(&extract).unwrap_or(default),
        _ => default,
    }
}

/// Extract an i64 field from a score associated with a Bar::Song.
fn bar_score_field_i64(
    bar: &Bar,
    cache: &HashMap<String, ScoreData>,
    extract: impl Fn(&ScoreData) -> i64,
    default: i64,
) -> i64 {
    match bar {
        Bar::Song(s) => cache.get(&s.sha256).map(&extract).unwrap_or(default),
        _ => default,
    }
}
