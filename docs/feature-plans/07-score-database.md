# Score Database Implementation

## Overview

Persistent storage of scores, clear lamps, and replays is essential for tracking player progress. This document covers the database schema and implementation.

## Storage Options

### SQLite (Recommended)

Used by beatoraja. Benefits:
- Single file, portable
- No server required
- Good query performance
- Rust support via `rusqlite` or `sqlx`

### File-based

Alternative for simpler implementation:
- JSON/MessagePack files per song
- Easy to backup/transfer
- Less efficient for queries

## Database Schema

### Songs Table

```sql
CREATE TABLE songs (
    sha256 TEXT PRIMARY KEY,     -- Chart hash (unique identifier)
    md5 TEXT,                    -- Legacy hash (for LR2 compatibility)
    title TEXT NOT NULL,
    subtitle TEXT,
    artist TEXT,
    subartist TEXT,
    genre TEXT,
    path TEXT NOT NULL,          -- File path
    folder TEXT,                 -- Parent folder
    total_notes INTEGER NOT NULL,
    level INTEGER,               -- Difficulty level (☆)
    difficulty INTEGER,          -- 0=BEGINNER, 1=NORMAL, etc.
    total REAL,                  -- #TOTAL value
    bpm_min REAL,
    bpm_max REAL,
    play_length_ms INTEGER,      -- Song duration
    last_scan_time INTEGER,      -- Unix timestamp
    mode INTEGER DEFAULT 7       -- Key mode (5, 7, 9, 14, etc.)
);

CREATE INDEX idx_songs_path ON songs(path);
CREATE INDEX idx_songs_folder ON songs(folder);
CREATE INDEX idx_songs_title ON songs(title);
```

### Scores Table

```sql
CREATE TABLE scores (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    sha256 TEXT NOT NULL,
    mode INTEGER NOT NULL,       -- Key mode
    clear INTEGER NOT NULL,      -- Clear lamp type

    -- Judgment counts (Early/Late separated)
    epg INTEGER DEFAULT 0,       -- Early PGREAT
    lpg INTEGER DEFAULT 0,       -- Late PGREAT
    egr INTEGER DEFAULT 0,       -- Early GREAT
    lgr INTEGER DEFAULT 0,       -- Late GREAT
    egd INTEGER DEFAULT 0,       -- Early GOOD
    lgd INTEGER DEFAULT 0,       -- Late GOOD
    ebd INTEGER DEFAULT 0,       -- Early BAD
    lbd INTEGER DEFAULT 0,       -- Late BAD
    epr INTEGER DEFAULT 0,       -- Early POOR
    lpr INTEGER DEFAULT 0,       -- Late POOR
    ems INTEGER DEFAULT 0,       -- Empty POOR (early miss)
    lms INTEGER DEFAULT 0,       -- Late miss

    notes INTEGER NOT NULL,      -- Total notes in chart
    combo INTEGER DEFAULT 0,     -- Max combo achieved
    minbp INTEGER,               -- Minimum bad/poor count

    playcount INTEGER DEFAULT 1,
    clearcount INTEGER DEFAULT 0,

    trophy TEXT,                 -- Achievement badges
    ghost BLOB,                  -- Ghost data (score graph)
    scorehash TEXT,              -- Integrity verification

    option INTEGER DEFAULT 0,    -- Play options bitmask
    random INTEGER DEFAULT 0,    -- Random option used

    date INTEGER NOT NULL,       -- Unix timestamp
    state INTEGER DEFAULT 0,     -- Play state flags

    FOREIGN KEY (sha256) REFERENCES songs(sha256),
    UNIQUE(sha256, mode)
);

CREATE INDEX idx_scores_sha256 ON scores(sha256);
CREATE INDEX idx_scores_clear ON scores(clear);
CREATE INDEX idx_scores_date ON scores(date);
```

### Replays Table

```sql
CREATE TABLE replays (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    sha256 TEXT NOT NULL,
    mode INTEGER NOT NULL,
    slot INTEGER NOT NULL,       -- Replay slot (1-4)

    data BLOB NOT NULL,          -- Compressed replay data

    ex_score INTEGER NOT NULL,
    clear INTEGER NOT NULL,
    combo INTEGER NOT NULL,

    option INTEGER,
    random INTEGER,
    random_seed INTEGER,

    date INTEGER NOT NULL,

    FOREIGN KEY (sha256) REFERENCES songs(sha256),
    UNIQUE(sha256, mode, slot)
);
```

## Clear Lamp Types

```rust
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum ClearLamp {
    NoPlay = 0,
    Failed = 1,
    AssistEasy = 2,
    LightAssistEasy = 3,
    Easy = 4,
    Normal = 5,
    Hard = 6,
    ExHard = 7,
    FullCombo = 8,
    Perfect = 9,
    Max = 10,
}

impl ClearLamp {
    pub fn from_gauge_and_stats(
        gauge_type: GaugeType,
        cleared: bool,
        has_miss: bool,  // BAD or POOR
        has_good: bool,
        has_great: bool,
    ) -> Self {
        if !cleared {
            return Self::Failed;
        }

        // Check for special lamps first
        if !has_great && !has_good && !has_miss {
            return Self::Max;  // 100% PGREAT
        }
        if !has_good && !has_miss {
            return Self::Perfect;  // All PGREAT/GREAT
        }
        if !has_miss {
            return Self::FullCombo;
        }

        // Standard clear lamps
        match gauge_type {
            GaugeType::AssistEasy => Self::AssistEasy,
            GaugeType::Easy => Self::Easy,
            GaugeType::Normal => Self::Normal,
            GaugeType::Hard => Self::Hard,
            GaugeType::ExHard => Self::ExHard,
            _ => Self::Normal,
        }
    }
}
```

## Score Calculation

### EX Score

```rust
pub fn calculate_ex_score(stats: &JudgmentStats) -> u32 {
    stats.pgreat * 2 + stats.great
}

pub fn calculate_max_ex_score(total_notes: u32) -> u32 {
    total_notes * 2
}

pub fn calculate_score_rate(ex_score: u32, max_ex_score: u32) -> f32 {
    if max_ex_score == 0 {
        0.0
    } else {
        ex_score as f32 / max_ex_score as f32 * 100.0
    }
}
```

### Grade/Rank

```rust
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Grade {
    F,    // < 22.22%
    E,    // 22.22% - 33.33%
    D,    // 33.33% - 44.44%
    C,    // 44.44% - 55.55%
    B,    // 55.55% - 66.66%
    A,    // 66.66% - 77.77%
    AA,   // 77.77% - 88.88%
    AAA,  // 88.88% - 100%
    Max,  // 100%
}

impl Grade {
    pub fn from_score_rate(rate: f32) -> Self {
        match rate {
            r if r >= 100.0 => Self::Max,
            r if r >= 88.89 => Self::AAA,
            r if r >= 77.78 => Self::AA,
            r if r >= 66.67 => Self::A,
            r if r >= 55.56 => Self::B,
            r if r >= 44.45 => Self::C,
            r if r >= 33.34 => Self::D,
            r if r >= 22.23 => Self::E,
            _ => Self::F,
        }
    }
}
```

## Score Manager

```rust
use rusqlite::{Connection, params};

pub struct ScoreManager {
    conn: Connection,
}

impl ScoreManager {
    pub fn new(db_path: &str) -> anyhow::Result<Self> {
        let conn = Connection::open(db_path)?;
        let manager = Self { conn };
        manager.init_tables()?;
        Ok(manager)
    }

    fn init_tables(&self) -> anyhow::Result<()> {
        self.conn.execute_batch(include_str!("schema.sql"))?;
        Ok(())
    }

    pub fn get_score(&self, sha256: &str, mode: u8) -> anyhow::Result<Option<Score>> {
        let mut stmt = self.conn.prepare(
            "SELECT * FROM scores WHERE sha256 = ? AND mode = ?"
        )?;

        let score = stmt.query_row(params![sha256, mode], |row| {
            Ok(Score {
                sha256: row.get("sha256")?,
                mode: row.get("mode")?,
                clear: ClearLamp::from_u8(row.get("clear")?),
                ex_score: self.calculate_ex_score_from_row(row),
                combo: row.get("combo")?,
                minbp: row.get("minbp")?,
                playcount: row.get("playcount")?,
                date: row.get("date")?,
            })
        }).optional()?;

        Ok(score)
    }

    pub fn save_score(&self, score: &PlayResult) -> anyhow::Result<bool> {
        // Check if we should update (new best)
        if let Some(existing) = self.get_score(&score.sha256, score.mode)? {
            if !self.is_better_score(score, &existing) {
                // Still increment playcount
                self.increment_playcount(&score.sha256, score.mode)?;
                return Ok(false);
            }
        }

        self.conn.execute(
            "INSERT OR REPLACE INTO scores (
                sha256, mode, clear, epg, lpg, egr, lgr, egd, lgd,
                ebd, lbd, epr, lpr, ems, lms, notes, combo, minbp,
                playcount, clearcount, option, random, date, state
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?,
                COALESCE((SELECT playcount FROM scores WHERE sha256 = ? AND mode = ?), 0) + 1,
                COALESCE((SELECT clearcount FROM scores WHERE sha256 = ? AND mode = ?), 0) + ?,
                ?, ?, ?, ?)",
            params![
                score.sha256, score.mode, score.clear as u8,
                score.stats.early_pgreat, score.stats.late_pgreat,
                score.stats.early_great, score.stats.late_great,
                score.stats.early_good, score.stats.late_good,
                score.stats.early_bad, score.stats.late_bad,
                score.stats.early_poor, score.stats.late_poor,
                score.stats.early_miss, score.stats.late_miss,
                score.total_notes, score.max_combo, score.min_bp,
                score.sha256, score.mode,
                score.sha256, score.mode, if score.cleared { 1 } else { 0 },
                score.option, score.random, score.timestamp, 0,
            ],
        )?;

        Ok(true)
    }

    fn is_better_score(&self, new: &PlayResult, existing: &Score) -> bool {
        // Clear lamp takes priority
        if new.clear > existing.clear {
            return true;
        }
        if new.clear < existing.clear {
            return false;
        }

        // Then EX score
        new.ex_score() > existing.ex_score
    }

    pub fn get_folder_stats(&self, folder: &str) -> anyhow::Result<FolderStats> {
        let mut stmt = self.conn.prepare(
            "SELECT
                COUNT(*) as total,
                SUM(CASE WHEN s.clear >= 5 THEN 1 ELSE 0 END) as cleared,
                SUM(CASE WHEN s.clear >= 8 THEN 1 ELSE 0 END) as fc,
                AVG(CASE WHEN s.clear >= 1 THEN s.clear ELSE NULL END) as avg_clear
             FROM songs
             LEFT JOIN scores s ON songs.sha256 = s.sha256
             WHERE songs.folder = ?"
        )?;

        stmt.query_row(params![folder], |row| {
            Ok(FolderStats {
                total: row.get("total")?,
                cleared: row.get("cleared")?,
                full_combo: row.get("fc")?,
                avg_clear: row.get("avg_clear")?,
            })
        }).map_err(Into::into)
    }
}
```

## Replay Data Format

```rust
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct ReplayData {
    pub version: u32,
    pub sha256: String,
    pub mode: u8,
    pub random_seed: u64,
    pub random_option: u8,
    pub judge_system: u8,
    pub gauge_type: u8,
    pub inputs: Vec<ReplayInput>,
}

#[derive(Serialize, Deserialize)]
pub struct ReplayInput {
    pub time_ms: i64,
    pub lane: u8,
    pub pressed: bool,  // true = press, false = release
}

impl ReplayData {
    pub fn compress(&self) -> anyhow::Result<Vec<u8>> {
        let json = serde_json::to_vec(self)?;
        let mut encoder = flate2::write::GzEncoder::new(
            Vec::new(),
            flate2::Compression::default(),
        );
        std::io::Write::write_all(&mut encoder, &json)?;
        Ok(encoder.finish()?)
    }

    pub fn decompress(data: &[u8]) -> anyhow::Result<Self> {
        let mut decoder = flate2::read::GzDecoder::new(data);
        let mut json = Vec::new();
        std::io::Read::read_to_end(&mut decoder, &mut json)?;
        Ok(serde_json::from_slice(&json)?)
    }
}
```

## Auto-Save Replay Conditions

beatoraja supports automatic replay saving based on conditions:

| Slot | Condition |
|------|-----------|
| 1 | Always save (latest play) |
| 2 | Better or same EX score |
| 3 | Better or same Clear lamp |
| 4 | Better or same Max combo |

## Chart Hash Calculation

```rust
use sha2::{Sha256, Digest};

pub fn calculate_chart_hash(chart_data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(chart_data);
    let result = hasher.finalize();
    hex::encode(result)
}
```

## Reference Links

- [beatoraja Score Database](https://github.com/exch-bms2/beatoraja)
- [beatoraja Scores and Clears Guide](https://github.com/wcko87/beatoraja-english-guide/wiki/Scores-and-Clears)
- [rusqlite Documentation](https://docs.rs/rusqlite/)
