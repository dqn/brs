use serde::{Deserialize, Serialize};

/// Player profile information stored in the score database.
///
/// Corresponds to Java `PlayerInformation`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PlayerInformation {
    pub id: String,
    pub name: String,
    pub rank: String,
}

impl PlayerInformation {
    pub fn from_row(row: &rusqlite::Row) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get("id")?,
            name: row.get::<_, Option<String>>("name")?.unwrap_or_default(),
            rank: row.get::<_, Option<String>>("rank")?.unwrap_or_default(),
        })
    }
}
