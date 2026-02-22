use std::fmt;

/// Corresponds to Java's `bms.player.beatoraja.exceptions.PlayerConfigException`
/// which extends `RuntimeException`.
#[derive(Debug, Clone)]
pub struct PlayerConfigException {
    message: String,
}

impl PlayerConfigException {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for PlayerConfigException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for PlayerConfigException {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_with_message() {
        let ex = PlayerConfigException::new("test error");
        assert_eq!(ex.message(), "test error");
    }

    #[test]
    fn test_display() {
        let ex = PlayerConfigException::new("config is invalid");
        assert_eq!(format!("{}", ex), "config is invalid");
    }

    #[test]
    fn test_error_trait() {
        let ex = PlayerConfigException::new("some error");
        // Verify it implements std::error::Error
        let err: &dyn std::error::Error = &ex;
        assert_eq!(err.to_string(), "some error");
    }

    #[test]
    fn test_into_anyhow() {
        let ex = PlayerConfigException::new("anyhow test");
        let err: anyhow::Error = ex.into();
        assert_eq!(err.to_string(), "anyhow test");
    }
}
