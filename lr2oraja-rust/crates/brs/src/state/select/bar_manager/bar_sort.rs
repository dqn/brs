// Bar sorting — sort and filter methods for BarManager.
//
// Java parity: all sort modes (except Default) fall back to title_cmp()
// when either bar is not a Song. TITLE sort treats Song and Folder as
// "sortable" bar types; others go to end.

use std::cmp::Ordering;
use std::collections::HashMap;

use bms_rule::ScoreData;

use super::bar_types::Bar;
use super::{BarManager, SortMode};

/// TITLE sort comparator.
///
/// Sorts all bars by their display name (case-insensitive). When both are
/// Song with equal titles, sub-sort by difficulty.
fn title_cmp(a: &Bar, b: &Bar) -> Ordering {
    // Special case: both are Song with equal titles → sub-sort by difficulty
    if let (Bar::Song(sa), Bar::Song(sb)) = (a, b) {
        let title_ord = sa.title.to_lowercase().cmp(&sb.title.to_lowercase());
        if title_ord == Ordering::Equal {
            return sa.difficulty.cmp(&sb.difficulty);
        }
        return title_ord;
    }

    // Otherwise, compare by display name
    a.bar_name()
        .to_lowercase()
        .cmp(&b.bar_name().to_lowercase())
}

/// Fallback comparator for non-TITLE sort modes.
///
/// If either bar is not a Song, sorts non-Songs to the beginning (they have
/// empty/default values for Song-specific fields), then by title.
/// Returns `None` when both bars are Song (caller handles Song-specific logic).
fn title_fallback_cmp(a: &Bar, b: &Bar) -> Option<Ordering> {
    let a_is_song = matches!(a, Bar::Song(_));
    let b_is_song = matches!(b, Bar::Song(_));

    if a_is_song && b_is_song {
        // Both are songs - caller will handle
        None
    } else if !a_is_song && !b_is_song {
        // Neither is a song - sort by title
        Some(title_cmp(a, b))
    } else if a_is_song {
        // a is song, b is not - non-song comes first
        Some(Ordering::Greater)
    } else {
        // b is song, a is not - non-song comes first
        Some(Ordering::Less)
    }
}

impl BarManager {
    /// Sort bars by the given mode.
    ///
    /// Follows Java BarSorter parity: non-Song bars fall back to TITLE sort
    /// for all modes except Default. Score-dependent modes use the
    /// `score_cache` keyed by SHA-256.
    pub fn sort(&mut self, mode: SortMode, score_cache: &HashMap<String, ScoreData>) {
        match mode {
            SortMode::Default => {} // Keep original order
            SortMode::Title => {
                self.bars.sort_by(title_cmp);
            }
            SortMode::Artist => {
                self.bars.sort_by(|a, b| {
                    if let Some(ord) = title_fallback_cmp(a, b) {
                        return ord;
                    }
                    let sa = a.as_song().unwrap();
                    let sb = b.as_song().unwrap();
                    sa.artist.to_lowercase().cmp(&sb.artist.to_lowercase())
                });
            }
            SortMode::Level => {
                self.bars.sort_by(|a, b| {
                    if let Some(ord) = title_fallback_cmp(a, b) {
                        return ord;
                    }
                    let sa = a.as_song().unwrap();
                    let sb = b.as_song().unwrap();
                    let level_ord = sa.level.cmp(&sb.level);
                    if level_ord == Ordering::Equal {
                        return sa.difficulty.cmp(&sb.difficulty);
                    }
                    level_ord
                });
            }
            SortMode::Bpm => {
                self.bars.sort_by(|a, b| {
                    if let Some(ord) = title_fallback_cmp(a, b) {
                        return ord;
                    }
                    let sa = a.as_song().unwrap();
                    let sb = b.as_song().unwrap();
                    sa.maxbpm.cmp(&sb.maxbpm)
                });
            }
            SortMode::Length => {
                self.bars.sort_by(|a, b| {
                    if let Some(ord) = title_fallback_cmp(a, b) {
                        return ord;
                    }
                    let sa = a.as_song().unwrap();
                    let sb = b.as_song().unwrap();
                    sa.length.cmp(&sb.length)
                });
            }
            SortMode::Clear => {
                self.bars.sort_by(|a, b| {
                    if let Some(ord) = title_fallback_cmp(a, b) {
                        return ord;
                    }
                    let sa = a.as_song().unwrap();
                    let sb = b.as_song().unwrap();
                    let score_a = score_cache.get(&sa.sha256);
                    let score_b = score_cache.get(&sb.sha256);
                    match (score_a, score_b) {
                        (None, None) => Ordering::Equal,
                        // Songs without scores come first (treated as NoPlay/0)
                        (None, Some(_)) => Ordering::Less,
                        (Some(_), None) => Ordering::Greater,
                        // Ascending order: lower clear types (NoPlay) first
                        (Some(a), Some(b)) => (a.clear.id() as i32).cmp(&(b.clear.id() as i32)),
                    }
                });
            }
            SortMode::Score => {
                self.bars.sort_by(|a, b| {
                    if let Some(ord) = title_fallback_cmp(a, b) {
                        return ord;
                    }
                    let sa = a.as_song().unwrap();
                    let sb = b.as_song().unwrap();
                    let score_a = score_cache
                        .get(&sa.sha256)
                        .map(|sd| sd.exscore())
                        .unwrap_or(0);
                    let score_b = score_cache
                        .get(&sb.sha256)
                        .map(|sd| sd.exscore())
                        .unwrap_or(0);
                    score_b.cmp(&score_a) // Descending (high scores first)
                });
            }
            SortMode::MissCount => {
                self.bars.sort_by(|a, b| {
                    if let Some(ord) = title_fallback_cmp(a, b) {
                        return ord;
                    }
                    let sa = a.as_song().unwrap();
                    let sb = b.as_song().unwrap();
                    let score_a = score_cache.get(&sa.sha256);
                    let score_b = score_cache.get(&sb.sha256);
                    match (score_a, score_b) {
                        (None, None) => Ordering::Equal,
                        (None, Some(_)) => Ordering::Greater,
                        (Some(_), None) => Ordering::Less,
                        (Some(a), Some(b)) => a.minbp.cmp(&b.minbp),
                    }
                });
            }
            SortMode::Duration => {
                self.bars.sort_by(|a, b| {
                    if let Some(ord) = title_fallback_cmp(a, b) {
                        return ord;
                    }
                    let sa = a.as_song().unwrap();
                    let sb = b.as_song().unwrap();
                    let pc_a = score_cache
                        .get(&sa.sha256)
                        .map(|sd| sd.playcount)
                        .unwrap_or(0);
                    let pc_b = score_cache
                        .get(&sb.sha256)
                        .map(|sd| sd.playcount)
                        .unwrap_or(0);
                    pc_b.cmp(&pc_a) // Descending (most played first)
                });
            }
            SortMode::LastUpdate => {
                self.bars.sort_by(|a, b| {
                    if let Some(ord) = title_fallback_cmp(a, b) {
                        return ord;
                    }
                    let sa = a.as_song().unwrap();
                    let sb = b.as_song().unwrap();
                    let score_a = score_cache.get(&sa.sha256);
                    let score_b = score_cache.get(&sb.sha256);
                    match (score_a, score_b) {
                        (None, None) => Ordering::Equal,
                        (None, Some(_)) => Ordering::Greater,
                        (Some(_), None) => Ordering::Less,
                        // Descending order: most recent first
                        (Some(a), Some(b)) => b.date.cmp(&a.date),
                    }
                });
            }
            SortMode::RivalCompareClear => {
                tracing::info!("RivalCompareClear sort: stub (requires rival score data)");
            }
            SortMode::RivalCompareScore => {
                tracing::info!("RivalCompareScore sort: stub (requires rival score data)");
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
