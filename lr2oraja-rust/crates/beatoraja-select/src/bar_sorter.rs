use std::cmp::Ordering;

use crate::bar::bar::Bar;

/// Bar sorting algorithms
/// Translates: bms.player.beatoraja.select.BarSorter
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BarSorter {
    Title,
    Artist,
    Bpm,
    Length,
    Level,
    Clear,
    Score,
    MissCount,
    Duration,
    LastUpdate,
    RivalCompareClear,
    RivalCompareScore,
}

impl BarSorter {
    pub const DEFAULT_SORTER: &'static [BarSorter] = &[
        BarSorter::Title,
        BarSorter::Artist,
        BarSorter::Bpm,
        BarSorter::Length,
        BarSorter::Level,
        BarSorter::Clear,
        BarSorter::Score,
        BarSorter::MissCount,
    ];

    pub const ALL_SORTER: &'static [BarSorter] = &[
        BarSorter::Title,
        BarSorter::Artist,
        BarSorter::Bpm,
        BarSorter::Length,
        BarSorter::Level,
        BarSorter::Clear,
        BarSorter::Score,
        BarSorter::MissCount,
        BarSorter::Duration,
        BarSorter::LastUpdate,
        BarSorter::RivalCompareClear,
        BarSorter::RivalCompareScore,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            BarSorter::Title => "TITLE",
            BarSorter::Artist => "ARTIST",
            BarSorter::Bpm => "BPM",
            BarSorter::Length => "LENGTH",
            BarSorter::Level => "LEVEL",
            BarSorter::Clear => "CLEAR",
            BarSorter::Score => "SCORE",
            BarSorter::MissCount => "MISSCOUNT",
            BarSorter::Duration => "DURATION",
            BarSorter::LastUpdate => "LASTUPDATE",
            BarSorter::RivalCompareClear => "RIVALCOMPARE_CLEAR",
            BarSorter::RivalCompareScore => "RIVALCOMPARE_SCORE",
        }
    }

    pub fn value_of(name: &str) -> Option<BarSorter> {
        match name {
            "TITLE" => Some(BarSorter::Title),
            "ARTIST" => Some(BarSorter::Artist),
            "BPM" => Some(BarSorter::Bpm),
            "LENGTH" => Some(BarSorter::Length),
            "LEVEL" => Some(BarSorter::Level),
            "CLEAR" => Some(BarSorter::Clear),
            "SCORE" => Some(BarSorter::Score),
            "MISSCOUNT" => Some(BarSorter::MissCount),
            "DURATION" => Some(BarSorter::Duration),
            "LASTUPDATE" => Some(BarSorter::LastUpdate),
            "RIVALCOMPARE_CLEAR" => Some(BarSorter::RivalCompareClear),
            "RIVALCOMPARE_SCORE" => Some(BarSorter::RivalCompareScore),
            _ => None,
        }
    }

    pub fn compare(&self, o1: &Bar, o2: &Bar) -> Ordering {
        match self {
            BarSorter::Title => Self::compare_title(o1, o2),
            BarSorter::Artist => Self::compare_artist(o1, o2),
            BarSorter::Bpm => Self::compare_bpm(o1, o2),
            BarSorter::Length => Self::compare_length(o1, o2),
            BarSorter::Level => Self::compare_level(o1, o2),
            BarSorter::Clear => Self::compare_clear(o1, o2),
            BarSorter::Score => Self::compare_score(o1, o2),
            BarSorter::MissCount => Self::compare_misscount(o1, o2),
            BarSorter::Duration => Self::compare_duration(o1, o2),
            BarSorter::LastUpdate => Self::compare_lastupdate(o1, o2),
            BarSorter::RivalCompareClear => Self::compare_rival_clear(o1, o2),
            BarSorter::RivalCompareScore => Self::compare_rival_score(o1, o2),
        }
    }

    fn is_song_or_folder(bar: &Bar) -> bool {
        bar.as_song_bar().is_some() || bar.as_folder_bar().is_some()
    }

    fn compare_title(o1: &Bar, o2: &Bar) -> Ordering {
        if !Self::is_song_or_folder(o1) && !Self::is_song_or_folder(o2) {
            return Ordering::Equal;
        }
        if !Self::is_song_or_folder(o1) {
            return Ordering::Greater;
        }
        if !Self::is_song_or_folder(o2) {
            return Ordering::Less;
        }

        if let (Some(s1), Some(s2)) = (o1.as_song_bar(), o2.as_song_bar()) {
            let title_cmp = s1
                .song
                .get_title()
                .to_lowercase()
                .cmp(&s2.song.get_title().to_lowercase());
            if title_cmp == Ordering::Equal {
                return s1.song.get_difficulty().cmp(&s2.song.get_difficulty());
            }
            return title_cmp;
        }

        o1.get_title()
            .to_lowercase()
            .cmp(&o2.get_title().to_lowercase())
    }

    fn compare_artist(o1: &Bar, o2: &Bar) -> Ordering {
        let (s1, s2) = match (o1.as_song_bar(), o2.as_song_bar()) {
            (Some(s1), Some(s2)) => (s1, s2),
            _ => return Self::compare_title(o1, o2),
        };
        if !s1.exists_song() && !s2.exists_song() {
            return Ordering::Equal;
        }
        if !s1.exists_song() {
            return Ordering::Greater;
        }
        if !s2.exists_song() {
            return Ordering::Less;
        }
        s1.song
            .get_artist()
            .to_lowercase()
            .cmp(&s2.song.get_artist().to_lowercase())
    }

    fn compare_bpm(o1: &Bar, o2: &Bar) -> Ordering {
        let (s1, s2) = match (o1.as_song_bar(), o2.as_song_bar()) {
            (Some(s1), Some(s2)) => (s1, s2),
            _ => return Self::compare_title(o1, o2),
        };
        if !s1.exists_song() && !s2.exists_song() {
            return Ordering::Equal;
        }
        if !s1.exists_song() {
            return Ordering::Greater;
        }
        if !s2.exists_song() {
            return Ordering::Less;
        }
        s1.song.get_maxbpm().cmp(&s2.song.get_maxbpm())
    }

    fn compare_length(o1: &Bar, o2: &Bar) -> Ordering {
        let (s1, s2) = match (o1.as_song_bar(), o2.as_song_bar()) {
            (Some(s1), Some(s2)) => (s1, s2),
            _ => return Self::compare_title(o1, o2),
        };
        if !s1.exists_song() && !s2.exists_song() {
            return Ordering::Equal;
        }
        if !s1.exists_song() {
            return Ordering::Greater;
        }
        if !s2.exists_song() {
            return Ordering::Less;
        }
        s1.song.get_length().cmp(&s2.song.get_length())
    }

    fn compare_level(o1: &Bar, o2: &Bar) -> Ordering {
        let (s1, s2) = match (o1.as_song_bar(), o2.as_song_bar()) {
            (Some(s1), Some(s2)) => (s1, s2),
            _ => return Self::compare_title(o1, o2),
        };
        if !s1.exists_song() && !s2.exists_song() {
            return Ordering::Equal;
        }
        if !s1.exists_song() {
            return Ordering::Greater;
        }
        if !s2.exists_song() {
            return Ordering::Less;
        }
        let level_sort = s1.song.get_level().cmp(&s2.song.get_level());
        if level_sort == Ordering::Equal {
            return s1.song.get_difficulty().cmp(&s2.song.get_difficulty());
        }
        level_sort
    }

    fn compare_clear(o1: &Bar, o2: &Bar) -> Ordering {
        if o1.as_song_bar().is_none() || o2.as_song_bar().is_none() {
            return Self::compare_title(o1, o2);
        }
        match (o1.get_score(), o2.get_score()) {
            (None, None) => Ordering::Equal,
            (None, _) => Ordering::Greater,
            (_, None) => Ordering::Less,
            (Some(s1), Some(s2)) => s1.get_clear().cmp(&s2.get_clear()),
        }
    }

    fn compare_score(o1: &Bar, o2: &Bar) -> Ordering {
        if o1.as_song_bar().is_none() || o2.as_song_bar().is_none() {
            return Self::compare_title(o1, o2);
        }
        let n1 = o1.get_score().map(|s| s.get_notes()).unwrap_or(0);
        let n2 = o2.get_score().map(|s| s.get_notes()).unwrap_or(0);
        if n1 == 0 && n2 == 0 {
            return Ordering::Equal;
        }
        if n1 == 0 {
            return Ordering::Greater;
        }
        if n2 == 0 {
            return Ordering::Less;
        }
        let r1 = o1.get_score().unwrap().get_exscore() as f32 / n1 as f32;
        let r2 = o2.get_score().unwrap().get_exscore() as f32 / n2 as f32;
        r1.partial_cmp(&r2).unwrap_or(Ordering::Equal)
    }

    fn compare_misscount(o1: &Bar, o2: &Bar) -> Ordering {
        if o1.as_song_bar().is_none() || o2.as_song_bar().is_none() {
            return Self::compare_title(o1, o2);
        }
        match (o1.get_score(), o2.get_score()) {
            (None, None) => Ordering::Equal,
            (None, _) => Ordering::Greater,
            (_, None) => Ordering::Less,
            (Some(s1), Some(s2)) => s1.get_minbp().cmp(&s2.get_minbp()),
        }
    }

    fn compare_duration(o1: &Bar, o2: &Bar) -> Ordering {
        if o1.as_song_bar().is_none() || o2.as_song_bar().is_none() {
            return Self::compare_title(o1, o2);
        }
        let exists1 = o1
            .get_score()
            .map(|s| s.get_avgjudge() != i64::MAX)
            .unwrap_or(false);
        let exists2 = o2
            .get_score()
            .map(|s| s.get_avgjudge() != i64::MAX)
            .unwrap_or(false);
        if !exists1 && !exists2 {
            return Ordering::Equal;
        }
        if !exists1 {
            return Ordering::Greater;
        }
        if !exists2 {
            return Ordering::Less;
        }
        let d = o1.get_score().unwrap().get_avgjudge() - o2.get_score().unwrap().get_avgjudge();
        d.cmp(&0)
    }

    fn compare_lastupdate(o1: &Bar, o2: &Bar) -> Ordering {
        if o1.as_song_bar().is_none() || o2.as_song_bar().is_none() {
            return Self::compare_title(o1, o2);
        }
        match (o1.get_score(), o2.get_score()) {
            (None, None) => Ordering::Equal,
            (None, _) => Ordering::Greater,
            (_, None) => Ordering::Less,
            (Some(s1), Some(s2)) => {
                let d = s1.get_date() - s2.get_date();
                d.cmp(&0)
            }
        }
    }

    fn compare_rival_clear(o1: &Bar, o2: &Bar) -> Ordering {
        if o1.as_song_bar().is_none() || o2.as_song_bar().is_none() {
            return Self::compare_title(o1, o2);
        }
        let has1 = o1.get_score().is_some() && o1.get_rival_score().is_some();
        let has2 = o2.get_score().is_some() && o2.get_rival_score().is_some();
        if !has1 && !has2 {
            return Ordering::Equal;
        }
        if !has1 {
            return Ordering::Greater;
        }
        if !has2 {
            return Ordering::Less;
        }
        let d1 = o1.get_score().unwrap().get_clear() - o1.get_rival_score().unwrap().get_clear();
        let d2 = o2.get_score().unwrap().get_clear() - o2.get_rival_score().unwrap().get_clear();
        d1.cmp(&d2)
    }

    fn compare_rival_score(o1: &Bar, o2: &Bar) -> Ordering {
        if o1.as_song_bar().is_none() || o2.as_song_bar().is_none() {
            return Self::compare_title(o1, o2);
        }
        let n1 = o1.get_score().map(|s| s.get_notes()).unwrap_or(0);
        let n2 = o2.get_score().map(|s| s.get_notes()).unwrap_or(0);
        let r1 = o1.get_rival_score().map(|s| s.get_notes()).unwrap_or(0);
        let r2 = o2.get_rival_score().map(|s| s.get_notes()).unwrap_or(0);
        if (n1 == 0 || r1 == 0) && (n2 == 0 || r2 == 0) {
            return Ordering::Equal;
        }
        if n1 == 0 || r1 == 0 {
            return Ordering::Greater;
        }
        if n2 == 0 || r2 == 0 {
            return Ordering::Less;
        }
        let v1 = o1.get_score().unwrap().get_exscore() as f32 / n1 as f32
            - o1.get_rival_score().unwrap().get_exscore() as f32 / r1 as f32;
        let v2 = o2.get_score().unwrap().get_exscore() as f32 / n2 as f32
            - o2.get_rival_score().unwrap().get_exscore() as f32 / r2 as f32;
        v1.partial_cmp(&v2).unwrap_or(Ordering::Equal)
    }
}
