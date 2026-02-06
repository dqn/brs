use rusqlite::Row;

/// Song data stored in the song database.
/// Matches beatoraja's `song` table schema.
#[derive(Debug, Clone, Default)]
pub struct SongData {
    pub md5: String,
    pub sha256: String,
    pub title: String,
    pub subtitle: String,
    pub genre: String,
    pub artist: String,
    pub subartist: String,
    pub tag: String,
    pub path: String,
    pub folder: String,
    pub stagefile: String,
    pub banner: String,
    pub backbmp: String,
    pub preview: String,
    pub parent: String,
    pub level: i32,
    pub difficulty: i32,
    pub maxbpm: i32,
    pub minbpm: i32,
    pub length: i32,
    pub mode: i32,
    pub judge: i32,
    pub feature: i32,
    pub content: i32,
    pub date: i64,
    pub favorite: i32,
    pub adddate: i64,
    pub notes: i32,
    pub charthash: String,
}

impl SongData {
    /// Full title combining title and subtitle.
    pub fn full_title(&self) -> String {
        if self.subtitle.is_empty() {
            self.title.clone()
        } else {
            format!("{} {}", self.title, self.subtitle)
        }
    }

    /// Full artist combining artist and subartist.
    pub fn full_artist(&self) -> String {
        if self.subartist.is_empty() {
            self.artist.clone()
        } else {
            format!("{} {}", self.artist, self.subartist)
        }
    }

    /// Read a SongData from a rusqlite Row.
    pub fn from_row(row: &Row<'_>) -> rusqlite::Result<Self> {
        Ok(Self {
            md5: row.get("md5")?,
            sha256: row.get("sha256")?,
            title: row.get("title")?,
            subtitle: row.get("subtitle")?,
            genre: row.get("genre")?,
            artist: row.get("artist")?,
            subartist: row.get("subartist")?,
            tag: row.get("tag")?,
            path: row.get("path")?,
            folder: row.get("folder")?,
            stagefile: row.get("stagefile")?,
            banner: row.get("banner")?,
            backbmp: row.get("backbmp")?,
            preview: row.get("preview")?,
            parent: row.get("parent")?,
            level: row.get("level")?,
            difficulty: row.get("difficulty")?,
            maxbpm: row.get("maxbpm")?,
            minbpm: row.get("minbpm")?,
            length: row.get("length")?,
            mode: row.get("mode")?,
            judge: row.get("judge")?,
            feature: row.get("feature")?,
            content: row.get("content")?,
            date: row.get("date")?,
            favorite: row.get("favorite")?,
            adddate: row.get("adddate")?,
            notes: row.get("notes")?,
            charthash: row.get("charthash")?,
        })
    }
}

/// Feature flags for SongData.
pub mod feature {
    pub const UNDEFINED_LN: i32 = 1;
    pub const MINE_NOTE: i32 = 2;
    pub const RANDOM: i32 = 4;
    pub const LONG_NOTE: i32 = 8;
    pub const CHARGE_NOTE: i32 = 16;
    pub const HELL_CHARGE_NOTE: i32 = 32;
    pub const STOP_SEQUENCE: i32 = 64;
    pub const SCROLL: i32 = 128;
}

/// Content flags for SongData.
pub mod content {
    pub const TEXT: i32 = 1;
    pub const BGA: i32 = 2;
    pub const PREVIEW: i32 = 4;
    pub const NO_KEYSOUND: i32 = 128;
}

/// Favorite flags for SongData.
pub mod favorite_flag {
    pub const SONG: i32 = 1;
    pub const CHART: i32 = 2;
    pub const INVISIBLE_SONG: i32 = 4;
    pub const INVISIBLE_CHART: i32 = 8;
}

/// Folder data stored in the song database.
/// Matches beatoraja's `folder` table schema.
#[derive(Debug, Clone, Default)]
pub struct FolderData {
    pub title: String,
    pub subtitle: String,
    pub command: String,
    pub path: String,
    pub banner: String,
    pub parent: String,
    pub folder_type: i32,
    pub date: i64,
    pub adddate: i64,
    pub max: i32,
}

impl FolderData {
    /// Read a FolderData from a rusqlite Row.
    pub fn from_row(row: &Row<'_>) -> rusqlite::Result<Self> {
        Ok(Self {
            title: row.get("title")?,
            subtitle: row.get("subtitle")?,
            command: row.get("command")?,
            path: row.get("path")?,
            banner: row.get("banner")?,
            parent: row.get("parent")?,
            folder_type: row.get("type")?,
            date: row.get("date")?,
            adddate: row.get("adddate")?,
            max: row.get("max")?,
        })
    }
}

/// Score data stored in the score database.
/// Matches beatoraja's `score` table schema.
#[derive(Debug, Clone, Default)]
pub struct ScoreData {
    pub sha256: String,
    pub mode: i32,
    pub clear: i32,
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
    pub notes: i32,
    pub combo: i32,
    pub minbp: i32,
    pub avgjudge: i64,
    pub playcount: i32,
    pub clearcount: i32,
    pub trophy: String,
    pub ghost: String,
    pub option: i32,
    pub seed: i64,
    pub random: i32,
    pub date: i64,
    pub state: i32,
    pub scorehash: String,
}

impl ScoreData {
    /// EX score = (PG * 2) + GR.
    pub fn exscore(&self) -> i32 {
        (self.epg + self.lpg) * 2 + self.egr + self.lgr
    }

    /// Read a ScoreData from a rusqlite Row.
    pub fn from_row(row: &Row<'_>) -> rusqlite::Result<Self> {
        Ok(Self {
            sha256: row.get("sha256")?,
            mode: row.get("mode")?,
            clear: row.get("clear")?,
            epg: row.get("epg")?,
            lpg: row.get("lpg")?,
            egr: row.get("egr")?,
            lgr: row.get("lgr")?,
            egd: row.get("egd")?,
            lgd: row.get("lgd")?,
            ebd: row.get("ebd")?,
            lbd: row.get("lbd")?,
            epr: row.get("epr")?,
            lpr: row.get("lpr")?,
            ems: row.get("ems")?,
            lms: row.get("lms")?,
            notes: row.get("notes")?,
            combo: row.get("combo")?,
            minbp: row.get("minbp")?,
            avgjudge: row.get("avgjudge")?,
            playcount: row.get("playcount")?,
            clearcount: row.get("clearcount")?,
            trophy: row.get("trophy")?,
            ghost: row.get("ghost")?,
            option: row.get("option")?,
            seed: row.get("seed")?,
            random: row.get("random")?,
            date: row.get("date")?,
            state: row.get("state")?,
            scorehash: row.get("scorehash")?,
        })
    }

    /// Update this score with a new score, keeping best values.
    /// Returns true if any field was updated.
    pub fn update(&mut self, new: &ScoreData) -> bool {
        let mut updated = false;
        if self.clear < new.clear {
            self.clear = new.clear;
            self.option = new.option;
            self.seed = new.seed;
            updated = true;
        }
        if self.exscore() < new.exscore() {
            self.epg = new.epg;
            self.lpg = new.lpg;
            self.egr = new.egr;
            self.lgr = new.lgr;
            self.egd = new.egd;
            self.lgd = new.lgd;
            self.ebd = new.ebd;
            self.lbd = new.lbd;
            self.epr = new.epr;
            self.lpr = new.lpr;
            self.ems = new.ems;
            self.lms = new.lms;
            self.option = new.option;
            self.seed = new.seed;
            self.ghost = new.ghost.clone();
            updated = true;
        }
        if self.minbp > new.minbp {
            self.minbp = new.minbp;
            updated = true;
        }
        if self.combo < new.combo {
            self.combo = new.combo;
            updated = true;
        }
        updated
    }
}

/// Clear type constants matching beatoraja's ClearType enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(i32)]
pub enum ClearType {
    NoPlay = 0,
    Failed = 1,
    LightAssistEasy = 2,
    AssistEasy = 3,
    Easy = 4,
    Normal = 5,
    Hard = 6,
    ExHard = 7,
    FullCombo = 8,
    Perfect = 9,
    Max = 10,
}
