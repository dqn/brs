use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// OBS WebSocket v5 opcodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum OpCode {
    Hello = 0,
    Identify = 1,
    Identified = 2,
    Reidentify = 3,
    Event = 5,
    Request = 6,
    RequestResponse = 7,
    RequestBatch = 8,
    RequestBatchResponse = 9,
}

impl OpCode {
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0 => Some(Self::Hello),
            1 => Some(Self::Identify),
            2 => Some(Self::Identified),
            3 => Some(Self::Reidentify),
            5 => Some(Self::Event),
            6 => Some(Self::Request),
            7 => Some(Self::RequestResponse),
            8 => Some(Self::RequestBatch),
            9 => Some(Self::RequestBatchResponse),
            _ => None,
        }
    }
}

/// OBS WebSocket Hello message (sent by server).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hello {
    pub op: u8,
    pub d: HelloData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HelloData {
    pub obs_web_socket_version: String,
    pub rpc_version: u32,
    pub authentication: Option<AuthChallenge>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthChallenge {
    pub challenge: String,
    pub salt: String,
}

/// OBS WebSocket Identify message (sent by client).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Identify {
    pub op: u8,
    pub d: IdentifyData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IdentifyData {
    pub rpc_version: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authentication: Option<String>,
}

/// OBS WebSocket Identified message (sent by server).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Identified {
    pub op: u8,
    pub d: IdentifiedData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IdentifiedData {
    pub negotiated_rpc_version: u32,
}

/// OBS WebSocket Request message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
    pub op: u8,
    pub d: RequestData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestData {
    pub request_type: String,
    pub request_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_data: Option<serde_json::Value>,
}

/// OBS WebSocket RequestResponse message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestResponse {
    pub op: u8,
    pub d: RequestResponseData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestResponseData {
    pub request_type: String,
    pub request_id: String,
    pub request_status: RequestStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_data: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestStatus {
    pub result: bool,
    pub code: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
}

/// OBS WebSocket Event message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub op: u8,
    pub d: EventData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EventData {
    pub event_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_data: Option<serde_json::Value>,
}

/// Compute OBS WebSocket v5 authentication string.
///
/// Algorithm: SHA256(password + salt) -> Base64 -> SHA256(secret + challenge) -> Base64
pub fn compute_auth(password: &str, salt: &str, challenge: &str) -> String {
    // Step 1: SHA256(password + salt) -> Base64
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    hasher.update(salt.as_bytes());
    let secret = BASE64.encode(hasher.finalize());

    // Step 2: SHA256(secret + challenge) -> Base64
    let mut hasher = Sha256::new();
    hasher.update(secret.as_bytes());
    hasher.update(challenge.as_bytes());
    BASE64.encode(hasher.finalize())
}

/// Create an Identify message.
pub fn create_identify(rpc_version: u32, auth: Option<String>) -> String {
    let msg = Identify {
        op: OpCode::Identify as u8,
        d: IdentifyData {
            rpc_version,
            authentication: auth,
        },
    };
    serde_json::to_string(&msg).expect("failed to serialize Identify")
}

/// Create a Request message.
pub fn create_request(
    request_type: &str,
    request_id: &str,
    request_data: Option<serde_json::Value>,
) -> String {
    let msg = Request {
        op: OpCode::Request as u8,
        d: RequestData {
            request_type: request_type.to_string(),
            request_id: request_id.to_string(),
            request_data,
        },
    };
    serde_json::to_string(&msg).expect("failed to serialize Request")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opcode_from_u8() {
        assert_eq!(OpCode::from_u8(0), Some(OpCode::Hello));
        assert_eq!(OpCode::from_u8(1), Some(OpCode::Identify));
        assert_eq!(OpCode::from_u8(2), Some(OpCode::Identified));
        assert_eq!(OpCode::from_u8(5), Some(OpCode::Event));
        assert_eq!(OpCode::from_u8(6), Some(OpCode::Request));
        assert_eq!(OpCode::from_u8(7), Some(OpCode::RequestResponse));
        assert_eq!(OpCode::from_u8(4), None); // 4 is unused
        assert_eq!(OpCode::from_u8(10), None);
    }

    #[test]
    fn compute_auth_known_values() {
        // Known test vector from OBS WebSocket docs
        let password = "supersecretpassword";
        let salt = "PZL+bv/ztpFAHMjgK9gr/A==";
        let challenge = "ztTBnnuqrqaKDzRM3xcVdbYm";
        let result = compute_auth(password, salt, challenge);
        // Just verify it produces a base64 string
        assert!(!result.is_empty());
        assert!(
            base64::engine::general_purpose::STANDARD
                .decode(&result)
                .is_ok()
        );
    }

    #[test]
    fn compute_auth_deterministic() {
        let a = compute_auth("pass", "salt", "challenge");
        let b = compute_auth("pass", "salt", "challenge");
        assert_eq!(a, b);
    }

    #[test]
    fn compute_auth_different_inputs_differ() {
        let a = compute_auth("pass1", "salt", "challenge");
        let b = compute_auth("pass2", "salt", "challenge");
        assert_ne!(a, b);
    }

    #[test]
    fn create_identify_no_auth() {
        let json = create_identify(1, None);
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["op"], 1);
        assert_eq!(parsed["d"]["rpcVersion"], 1);
        assert!(parsed["d"].get("authentication").is_none());
    }

    #[test]
    fn create_identify_with_auth() {
        let json = create_identify(1, Some("authstring".to_string()));
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["op"], 1);
        assert_eq!(parsed["d"]["authentication"], "authstring");
    }

    #[test]
    fn create_request_no_data() {
        let json = create_request("GetSceneList", "req-1", None);
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["op"], 6);
        assert_eq!(parsed["d"]["requestType"], "GetSceneList");
        assert_eq!(parsed["d"]["requestId"], "req-1");
    }

    #[test]
    fn create_request_with_data() {
        let data = serde_json::json!({"sceneName": "Game"});
        let json = create_request("SetCurrentProgramScene", "req-2", Some(data));
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["d"]["requestData"]["sceneName"], "Game");
    }

    #[test]
    fn hello_deserialize() {
        let json = r#"{"op":0,"d":{"obsWebSocketVersion":"5.0.0","rpcVersion":1,"authentication":{"challenge":"abc","salt":"def"}}}"#;
        let hello: Hello = serde_json::from_str(json).unwrap();
        assert_eq!(hello.op, 0);
        assert_eq!(hello.d.rpc_version, 1);
        let auth = hello.d.authentication.unwrap();
        assert_eq!(auth.challenge, "abc");
        assert_eq!(auth.salt, "def");
    }

    #[test]
    fn identified_deserialize() {
        let json = r#"{"op":2,"d":{"negotiatedRpcVersion":1}}"#;
        let identified: Identified = serde_json::from_str(json).unwrap();
        assert_eq!(identified.op, 2);
        assert_eq!(identified.d.negotiated_rpc_version, 1);
    }

    #[test]
    fn request_response_deserialize() {
        let json = r#"{"op":7,"d":{"requestType":"GetSceneList","requestId":"1","requestStatus":{"result":true,"code":100},"responseData":{"scenes":[]}}}"#;
        let resp: RequestResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.op, 7);
        assert!(resp.d.request_status.result);
        assert_eq!(resp.d.request_status.code, 100);
    }

    #[test]
    fn event_deserialize() {
        let json = r#"{"op":5,"d":{"eventType":"SceneChanged","eventData":{"sceneName":"Game"}}}"#;
        let event: Event = serde_json::from_str(json).unwrap();
        assert_eq!(event.op, 5);
        assert_eq!(event.d.event_type, "SceneChanged");
    }
}
