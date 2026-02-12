// On-screen message renderer.
//
// Manages a queue of timed messages with TTL and alpha pulsing effect.
//
// Ported from Java MessageRenderer.java.

/// On-screen message with TTL and alpha pulsing.
#[derive(Debug, Clone)]
pub struct Message {
    pub text: String,
    pub color: [f32; 4],
    pub expiry_ms: i64,
    pub message_type: i32,
}

impl Message {
    pub fn new(text: String, ttl_ms: i64, color: [f32; 4], message_type: i32, now_ms: i64) -> Self {
        Self {
            text,
            color,
            expiry_ms: now_ms + ttl_ms,
            message_type,
        }
    }

    pub fn is_expired(&self, now_ms: i64) -> bool {
        now_ms >= self.expiry_ms
    }

    pub fn stop(&mut self) {
        self.expiry_ms = -1;
    }

    /// Compute alpha with pulsing effect.
    pub fn pulsing_alpha(&self, time_ms: i64) -> f32 {
        let phase = (time_ms % 1440) as f32 / 4.0;
        let sin_val = (phase * std::f32::consts::PI / 180.0).sin();
        sin_val * 0.3 + 0.7
    }
}

/// Default TTL: 24 hours in milliseconds.
pub const DEFAULT_TTL_MS: i64 = 24 * 60 * 60 * 1000;

/// Vertical spacing between messages in pixels.
pub const MESSAGE_SPACING: f32 = 24.0;

/// Message renderer managing a queue of on-screen messages.
#[derive(Debug, Default)]
pub struct MessageRenderer {
    messages: Vec<Message>,
}

impl MessageRenderer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_message(
        &mut self,
        text: String,
        color: [f32; 4],
        message_type: i32,
        now_ms: i64,
    ) -> usize {
        self.add_message_with_ttl(text, DEFAULT_TTL_MS, color, message_type, now_ms)
    }

    pub fn add_message_with_ttl(
        &mut self,
        text: String,
        ttl_ms: i64,
        color: [f32; 4],
        message_type: i32,
        now_ms: i64,
    ) -> usize {
        let msg = Message::new(text, ttl_ms, color, message_type, now_ms);
        self.messages.push(msg);
        self.messages.len() - 1
    }

    pub fn remove_expired(&mut self, now_ms: i64) {
        self.messages.retain(|m| !m.is_expired(now_ms));
    }

    pub fn active_messages(&self) -> &[Message] {
        &self.messages
    }

    pub fn clear(&mut self) {
        self.messages.clear();
    }

    pub fn message_count(&self) -> usize {
        self.messages.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_and_count() {
        let mut renderer = MessageRenderer::new();
        assert_eq!(renderer.message_count(), 0);

        renderer.add_message("Hello".to_string(), [1.0, 1.0, 1.0, 1.0], 0, 0);
        assert_eq!(renderer.message_count(), 1);

        renderer.add_message("World".to_string(), [1.0, 0.0, 0.0, 1.0], 1, 0);
        assert_eq!(renderer.message_count(), 2);
    }

    #[test]
    fn message_expiry() {
        let mut renderer = MessageRenderer::new();
        // TTL = 5000ms, now = 0
        renderer.add_message_with_ttl("Short".to_string(), 5000, [1.0; 4], 0, 0);
        renderer.add_message_with_ttl("Long".to_string(), 10000, [1.0; 4], 0, 0);

        // At 4999ms, neither expired
        renderer.remove_expired(4999);
        assert_eq!(renderer.message_count(), 2);

        // At 5000ms, first expires
        renderer.remove_expired(5000);
        assert_eq!(renderer.message_count(), 1);
        assert_eq!(renderer.active_messages()[0].text, "Long");

        // At 10000ms, second expires
        renderer.remove_expired(10000);
        assert_eq!(renderer.message_count(), 0);
    }

    #[test]
    fn clear_removes_all() {
        let mut renderer = MessageRenderer::new();
        renderer.add_message("A".to_string(), [1.0; 4], 0, 0);
        renderer.add_message("B".to_string(), [1.0; 4], 0, 0);
        renderer.clear();
        assert_eq!(renderer.message_count(), 0);
    }

    #[test]
    fn stop_expires_immediately() {
        let mut msg = Message::new("Test".to_string(), 10000, [1.0; 4], 0, 0);
        assert!(!msg.is_expired(5000));
        msg.stop();
        assert!(msg.is_expired(0));
    }

    #[test]
    fn pulsing_alpha_range() {
        let msg = Message::new("Test".to_string(), 10000, [1.0; 4], 0, 0);
        // Pulsing alpha should be in range [0.4, 1.0]
        // sin range is [-1, 1], so 0.3*(-1)+0.7 = 0.4 and 0.3*(1)+0.7 = 1.0
        for t in 0..1440 {
            let alpha = msg.pulsing_alpha(t);
            assert!(
                (0.39..=1.01).contains(&alpha),
                "alpha {alpha} out of range at t={t}"
            );
        }
    }

    #[test]
    fn default_ttl_is_24_hours() {
        assert_eq!(DEFAULT_TTL_MS, 86_400_000);
    }

    #[test]
    fn active_messages_returns_slice() {
        let mut renderer = MessageRenderer::new();
        renderer.add_message("A".to_string(), [1.0; 4], 0, 0);
        renderer.add_message("B".to_string(), [1.0; 4], 1, 0);
        let msgs = renderer.active_messages();
        assert_eq!(msgs.len(), 2);
        assert_eq!(msgs[0].text, "A");
        assert_eq!(msgs[1].text, "B");
    }
}
