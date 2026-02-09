use serde::{Deserialize, Serialize};

/// Player statistics data stored in the score database.
///
/// Corresponds to Java `PlayerData` — all fields are i64 (Java long).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PlayerData {
    pub date: i64,
    pub playcount: i64,
    pub clear: i64,
    pub epg: i64,
    pub lpg: i64,
    pub egr: i64,
    pub lgr: i64,
    pub egd: i64,
    pub lgd: i64,
    pub ebd: i64,
    pub lbd: i64,
    pub epr: i64,
    pub lpr: i64,
    pub ems: i64,
    pub lms: i64,
    pub playtime: i64,
    pub maxcombo: i64,
}

impl PlayerData {
    pub fn from_row(row: &rusqlite::Row) -> rusqlite::Result<Self> {
        Ok(Self {
            date: row.get::<_, Option<i64>>("date")?.unwrap_or(0),
            playcount: row.get::<_, Option<i64>>("playcount")?.unwrap_or(0),
            clear: row.get::<_, Option<i64>>("clear")?.unwrap_or(0),
            epg: row.get::<_, Option<i64>>("epg")?.unwrap_or(0),
            lpg: row.get::<_, Option<i64>>("lpg")?.unwrap_or(0),
            egr: row.get::<_, Option<i64>>("egr")?.unwrap_or(0),
            lgr: row.get::<_, Option<i64>>("lgr")?.unwrap_or(0),
            egd: row.get::<_, Option<i64>>("egd")?.unwrap_or(0),
            lgd: row.get::<_, Option<i64>>("lgd")?.unwrap_or(0),
            ebd: row.get::<_, Option<i64>>("ebd")?.unwrap_or(0),
            lbd: row.get::<_, Option<i64>>("lbd")?.unwrap_or(0),
            epr: row.get::<_, Option<i64>>("epr")?.unwrap_or(0),
            lpr: row.get::<_, Option<i64>>("lpr")?.unwrap_or(0),
            ems: row.get::<_, Option<i64>>("ems")?.unwrap_or(0),
            lms: row.get::<_, Option<i64>>("lms")?.unwrap_or(0),
            playtime: row.get::<_, Option<i64>>("playtime")?.unwrap_or(0),
            maxcombo: row.get::<_, Option<i64>>("maxcombo")?.unwrap_or(0),
        })
    }
}
