use serde::{Deserialize, Serialize};

/// IR player data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IRPlayerData {
    pub id: String,
    pub name: String,
    /// Rank / 段位
    pub rank: String,
}

impl IRPlayerData {
    pub fn new(id: impl Into<String>, name: impl Into<String>, rank: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            rank: rank.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serde_round_trip() {
        let player = IRPlayerData::new("id1", "TestPlayer", "10dan");
        let json = serde_json::to_string(&player).unwrap();
        let deserialized: IRPlayerData = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, "id1");
        assert_eq!(deserialized.name, "TestPlayer");
        assert_eq!(deserialized.rank, "10dan");
    }
}
