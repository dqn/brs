/// IR response data
///
/// Translated from: IRResponse.java (generic interface)
///
/// In Java this is a generic interface `IRResponse<T>` with methods:
/// - isSucceeded() -> boolean
/// - getMessage() -> String
/// - getData() -> T
///
/// In Rust, we implement this as a concrete struct with a generic type parameter.
#[derive(Clone, Debug)]
pub struct IRResponse<T> {
    pub succeeded: bool,
    pub message: String,
    pub data: Option<T>,
}

impl<T> IRResponse<T> {
    pub fn new(succeeded: bool, message: String, data: Option<T>) -> Self {
        Self {
            succeeded,
            message,
            data,
        }
    }

    pub fn success(message: String, data: T) -> Self {
        Self {
            succeeded: true,
            message,
            data: Some(data),
        }
    }

    pub fn failure(message: String) -> Self {
        Self {
            succeeded: false,
            message,
            data: None,
        }
    }

    /// Whether the IR operation succeeded
    pub fn is_succeeded(&self) -> bool {
        self.succeeded
    }

    /// Get the message from IR
    pub fn get_message(&self) -> &str {
        &self.message
    }

    /// Get the data from IR
    pub fn get_data(&self) -> Option<&T> {
        self.data.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ir_response_success_constructor() {
        let resp = IRResponse::success("OK".to_string(), 42);
        assert!(resp.is_succeeded());
        assert_eq!(resp.get_message(), "OK");
        assert_eq!(resp.get_data(), Some(&42));
    }

    #[test]
    fn test_ir_response_failure_constructor() {
        let resp: IRResponse<i32> = IRResponse::failure("Error occurred".to_string());
        assert!(!resp.is_succeeded());
        assert_eq!(resp.get_message(), "Error occurred");
        assert!(resp.get_data().is_none());
    }

    #[test]
    fn test_ir_response_new_custom() {
        let resp = IRResponse::new(true, "partial".to_string(), Some(vec![1, 2, 3]));
        assert!(resp.is_succeeded());
        assert_eq!(resp.get_data(), Some(&vec![1, 2, 3]));
    }
}
