use serde::{Deserialize, Serialize};

/// Generic IR response wrapper.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IRResponse<T> {
    pub succeeded: bool,
    pub message: String,
    pub data: Option<T>,
}

impl<T> IRResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            succeeded: true,
            message: String::new(),
            data: Some(data),
        }
    }

    pub fn failure(message: impl Into<String>) -> Self {
        Self {
            succeeded: false,
            message: message.into(),
            data: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn success_response() {
        let resp = IRResponse::success(42);
        assert!(resp.succeeded);
        assert_eq!(resp.data, Some(42));
        assert!(resp.message.is_empty());
    }

    #[test]
    fn failure_response() {
        let resp: IRResponse<i32> = IRResponse::failure("error occurred");
        assert!(!resp.succeeded);
        assert!(resp.data.is_none());
        assert_eq!(resp.message, "error occurred");
    }

    #[test]
    fn serde_round_trip() {
        let resp = IRResponse::success("hello".to_string());
        let json = serde_json::to_string(&resp).unwrap();
        let deserialized: IRResponse<String> = serde_json::from_str(&json).unwrap();
        assert!(deserialized.succeeded);
        assert_eq!(deserialized.data, Some("hello".to_string()));
    }
}
