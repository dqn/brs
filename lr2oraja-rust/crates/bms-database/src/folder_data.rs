use serde::{Deserialize, Serialize};

/// Folder data stored in the song database.
///
/// Corresponds to Java `FolderData` with 10 DB columns.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FolderData {
    pub title: String,
    pub subtitle: String,
    pub command: String,
    pub path: String,
    pub banner: String,
    pub parent: String,
    pub r#type: i32,
    pub date: i32,
    pub adddate: i32,
    pub max: i32,
}

impl FolderData {
    pub fn from_row(row: &rusqlite::Row) -> rusqlite::Result<Self> {
        Ok(Self {
            title: row.get::<_, Option<String>>("title")?.unwrap_or_default(),
            subtitle: row
                .get::<_, Option<String>>("subtitle")?
                .unwrap_or_default(),
            command: row.get::<_, Option<String>>("command")?.unwrap_or_default(),
            path: row.get("path")?,
            banner: row.get::<_, Option<String>>("banner")?.unwrap_or_default(),
            parent: row.get::<_, Option<String>>("parent")?.unwrap_or_default(),
            r#type: row.get::<_, Option<i32>>("type")?.unwrap_or(0),
            date: row.get::<_, Option<i32>>("date")?.unwrap_or(0),
            adddate: row.get::<_, Option<i32>>("adddate")?.unwrap_or(0),
            max: row.get::<_, Option<i32>>("max")?.unwrap_or(0),
        })
    }
}
