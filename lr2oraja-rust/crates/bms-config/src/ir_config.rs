use serde::{Deserialize, Serialize};

pub const IR_SEND_ALWAYS: i32 = 0;
pub const IR_SEND_COMPLETE_SONG: i32 = 1;
pub const IR_SEND_UPDATE_SCORE: i32 = 2;

/// Internet Ranking configuration.
///
/// Encryption of userid/password is deferred to Phase 12.
/// The `cuserid` and `cpassword` fields hold encrypted values as-is.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct IRConfig {
    pub irname: String,
    pub userid: String,
    pub cuserid: String,
    pub password: String,
    pub cpassword: String,
    pub irsend: i32,
    pub importscore: bool,
    pub importrival: bool,
}

impl Default for IRConfig {
    fn default() -> Self {
        Self {
            irname: String::new(),
            userid: String::new(),
            cuserid: String::new(),
            password: String::new(),
            cpassword: String::new(),
            irsend: 0,
            importscore: false,
            importrival: true,
        }
    }
}

impl IRConfig {
    /// Validates this IR config. Returns false if irname is empty.
    ///
    /// Note: Encryption of userid/password is deferred to Phase 12.
    pub fn validate(&self) -> bool {
        !self.irname.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let ir = IRConfig::default();
        assert!(ir.irname.is_empty());
        assert!(ir.userid.is_empty());
        assert!(ir.cuserid.is_empty());
        assert!(ir.password.is_empty());
        assert!(ir.cpassword.is_empty());
        assert_eq!(ir.irsend, 0);
        assert!(!ir.importscore);
        assert!(ir.importrival);
    }

    #[test]
    fn test_validate_empty_irname() {
        let ir = IRConfig::default();
        assert!(!ir.validate());
    }

    #[test]
    fn test_validate_with_irname() {
        let ir = IRConfig {
            irname: "LR2IR".to_string(),
            ..Default::default()
        };
        assert!(ir.validate());
    }

    #[test]
    fn test_serde_round_trip() {
        let ir = IRConfig {
            irname: "LR2IR".to_string(),
            cuserid: "encrypted_user".to_string(),
            cpassword: "encrypted_pass".to_string(),
            irsend: IR_SEND_COMPLETE_SONG,
            importscore: true,
            importrival: false,
            ..Default::default()
        };
        let json = serde_json::to_string(&ir).unwrap();
        let back: IRConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(back.irname, "LR2IR");
        assert_eq!(back.cuserid, "encrypted_user");
        assert_eq!(back.cpassword, "encrypted_pass");
        assert_eq!(back.irsend, IR_SEND_COMPLETE_SONG);
        assert!(back.importscore);
        assert!(!back.importrival);
    }

    #[test]
    fn test_deserialize_from_empty() {
        let ir: IRConfig = serde_json::from_str("{}").unwrap();
        assert!(ir.irname.is_empty());
        assert!(ir.importrival); // default is true
    }

    #[test]
    fn test_constants() {
        assert_eq!(IR_SEND_ALWAYS, 0);
        assert_eq!(IR_SEND_COMPLETE_SONG, 1);
        assert_eq!(IR_SEND_UPDATE_SCORE, 2);
    }
}
