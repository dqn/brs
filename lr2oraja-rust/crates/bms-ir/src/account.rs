use serde::{Deserialize, Serialize};

/// IR account credentials.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IRAccount {
    pub id: String,
    pub password: String,
    pub name: String,
}

impl IRAccount {
    pub fn new(
        id: impl Into<String>,
        password: impl Into<String>,
        name: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            password: password.into(),
            name: name.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serde_round_trip() {
        let account = IRAccount::new("user1", "pass123", "Player1");
        let json = serde_json::to_string(&account).unwrap();
        let deserialized: IRAccount = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, "user1");
        assert_eq!(deserialized.password, "pass123");
        assert_eq!(deserialized.name, "Player1");
    }
}
