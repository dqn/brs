use aes::Aes128;
use aes::cipher::{BlockDecryptMut, BlockEncryptMut, KeyInit, block_padding::Pkcs7};
use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use serde::{Deserialize, Serialize};

pub const IR_SEND_ALWAYS: i32 = 0;
pub const IR_SEND_COMPLETE_SONG: i32 = 1;
pub const IR_SEND_UPDATE_SCORE: i32 = 2;

/// Internet Ranking configuration.
///
/// The `cuserid` and `cpassword` fields hold AES-encrypted values.
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

type Aes128EcbEncryptor = ecb::Encryptor<Aes128>;
type Aes128EcbDecryptor = ecb::Decryptor<Aes128>;

const IR_CONFIG_AES_KEY: &[u8; 16] = b"0123456789abcdef";

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
    pub fn validate(&self) -> bool {
        !self.irname.is_empty()
    }

    /// Encrypt plaintext credentials into `cuserid` / `cpassword` and
    /// clear plaintext fields before writing to disk.
    pub fn sanitize_credentials_for_write(&mut self) {
        if !self.userid.is_empty()
            && let Some(encrypted) = encrypt_credential(&self.userid)
        {
            self.cuserid = encrypted;
            self.userid.clear();
        }

        if !self.password.is_empty()
            && let Some(encrypted) = encrypt_credential(&self.password)
        {
            self.cpassword = encrypted;
            self.password.clear();
        }
    }

    /// Decrypt `cuserid` / `cpassword` into plaintext fields for in-memory use.
    ///
    /// Keeps encrypted fields unchanged for backwards compatibility.
    pub fn hydrate_credentials_for_use(&mut self) {
        if self.userid.is_empty()
            && !self.cuserid.is_empty()
            && let Some(decrypted) = decrypt_credential(&self.cuserid)
        {
            self.userid = decrypted;
        }

        if self.password.is_empty()
            && !self.cpassword.is_empty()
            && let Some(decrypted) = decrypt_credential(&self.cpassword)
        {
            self.password = decrypted;
        }
    }
}

fn encrypt_credential(plaintext: &str) -> Option<String> {
    let cipher = Aes128EcbEncryptor::new_from_slice(IR_CONFIG_AES_KEY).ok()?;
    let msg_len = plaintext.len();
    let mut buf = vec![0_u8; msg_len + 16];
    buf[..msg_len].copy_from_slice(plaintext.as_bytes());
    let encrypted = cipher.encrypt_padded_mut::<Pkcs7>(&mut buf, msg_len).ok()?;
    Some(BASE64_STANDARD.encode(encrypted))
}

fn decrypt_credential(ciphertext_b64: &str) -> Option<String> {
    let mut ciphertext = BASE64_STANDARD.decode(ciphertext_b64).ok()?;
    let cipher = Aes128EcbDecryptor::new_from_slice(IR_CONFIG_AES_KEY).ok()?;
    let decrypted = cipher.decrypt_padded_mut::<Pkcs7>(&mut ciphertext).ok()?;
    String::from_utf8(decrypted.to_vec()).ok()
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
    fn test_sanitize_credentials_for_write_encrypts_plaintext() {
        let mut ir = IRConfig {
            irname: "LR2IR".to_string(),
            userid: "alice".to_string(),
            password: "secret".to_string(),
            ..Default::default()
        };

        ir.sanitize_credentials_for_write();

        assert!(ir.userid.is_empty());
        assert!(ir.password.is_empty());
        assert!(!ir.cuserid.is_empty());
        assert!(!ir.cpassword.is_empty());
        assert_ne!(ir.cuserid, "alice");
        assert_ne!(ir.cpassword, "secret");
    }

    #[test]
    fn test_hydrate_credentials_for_use_decrypts_ciphertext() {
        let mut ir = IRConfig {
            irname: "LR2IR".to_string(),
            userid: "alice".to_string(),
            password: "secret".to_string(),
            ..Default::default()
        };
        ir.sanitize_credentials_for_write();

        let mut hydrated = IRConfig {
            irname: ir.irname.clone(),
            cuserid: ir.cuserid.clone(),
            cpassword: ir.cpassword.clone(),
            ..Default::default()
        };
        hydrated.hydrate_credentials_for_use();

        assert_eq!(hydrated.userid, "alice");
        assert_eq!(hydrated.password, "secret");
        assert_eq!(hydrated.cuserid, ir.cuserid);
        assert_eq!(hydrated.cpassword, ir.cpassword);
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
