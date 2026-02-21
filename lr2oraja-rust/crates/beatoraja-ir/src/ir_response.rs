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
