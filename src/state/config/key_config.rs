use serde::{Deserialize, Serialize};

/// Key binding for a single input action.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyBinding {
    /// Name of the action (e.g., "1P_KEY1", "1P_SCRATCH").
    pub action: String,
    /// Primary key code.
    pub primary: Option<u32>,
    /// Secondary key code (alternate binding).
    pub secondary: Option<u32>,
    /// Controller button index.
    pub controller_button: Option<u32>,
    /// Controller axis index and direction.
    pub controller_axis: Option<(u32, AxisDirection)>,
}

/// Axis direction for controller bindings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AxisDirection {
    Positive,
    Negative,
}

impl KeyBinding {
    /// Create a new key binding with only a primary key.
    pub fn new(action: String, primary: u32) -> Self {
        Self {
            action,
            primary: Some(primary),
            secondary: None,
            controller_button: None,
            controller_axis: None,
        }
    }

    /// Create an empty binding for an action.
    pub fn empty(action: String) -> Self {
        Self {
            action,
            primary: None,
            secondary: None,
            controller_button: None,
            controller_axis: None,
        }
    }

    /// Whether this binding has any key assigned.
    pub fn is_bound(&self) -> bool {
        self.primary.is_some()
            || self.secondary.is_some()
            || self.controller_button.is_some()
            || self.controller_axis.is_some()
    }
}

/// Key configuration for a play mode.
///
/// Corresponds to beatoraja's KeyConfiguration.
/// Maps physical keys/buttons to game actions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyConfig {
    /// Bindings for all actions.
    pub bindings: Vec<KeyBinding>,
}

impl KeyConfig {
    /// Create a default key configuration for 7-key mode.
    pub fn default_7k() -> Self {
        let actions = [
            "1P_SCRATCH",
            "1P_KEY1",
            "1P_KEY2",
            "1P_KEY3",
            "1P_KEY4",
            "1P_KEY5",
            "1P_KEY6",
            "1P_KEY7",
        ];

        // Default QWERTY bindings (matching common BMS player defaults)
        let keys: [u32; 8] = [
            16, // Left Shift -> Scratch
            90, // Z -> KEY1
            83, // S -> KEY2
            88, // X -> KEY3
            68, // D -> KEY4
            67, // C -> KEY5
            70, // F -> KEY6
            86, // V -> KEY7
        ];

        let bindings = actions
            .iter()
            .zip(keys.iter())
            .map(|(action, key)| KeyBinding::new(action.to_string(), *key))
            .collect();

        Self { bindings }
    }

    /// Get binding for an action by name.
    pub fn get_binding(&self, action: &str) -> Option<&KeyBinding> {
        self.bindings.iter().find(|b| b.action == action)
    }

    /// Set the primary key for an action.
    pub fn set_primary(&mut self, action: &str, key: u32) {
        if let Some(binding) = self.bindings.iter_mut().find(|b| b.action == action) {
            binding.primary = Some(key);
        }
    }

    /// Set the secondary key for an action.
    pub fn set_secondary(&mut self, action: &str, key: u32) {
        if let Some(binding) = self.bindings.iter_mut().find(|b| b.action == action) {
            binding.secondary = Some(key);
        }
    }

    /// Clear all bindings for an action.
    pub fn clear_binding(&mut self, action: &str) {
        if let Some(binding) = self.bindings.iter_mut().find(|b| b.action == action) {
            binding.primary = None;
            binding.secondary = None;
            binding.controller_button = None;
            binding.controller_axis = None;
        }
    }
}

impl Default for KeyConfig {
    fn default() -> Self {
        Self::default_7k()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_7k_has_8_bindings() {
        let config = KeyConfig::default_7k();
        assert_eq!(config.bindings.len(), 8);
    }

    #[test]
    fn get_binding() {
        let config = KeyConfig::default_7k();
        let scratch = config.get_binding("1P_SCRATCH");
        assert!(scratch.is_some());
        assert_eq!(scratch.unwrap().primary, Some(16));
    }

    #[test]
    fn set_primary() {
        let mut config = KeyConfig::default_7k();
        config.set_primary("1P_KEY1", 65); // A
        assert_eq!(config.get_binding("1P_KEY1").unwrap().primary, Some(65));
    }

    #[test]
    fn set_secondary() {
        let mut config = KeyConfig::default_7k();
        config.set_secondary("1P_KEY1", 75);
        let binding = config.get_binding("1P_KEY1").unwrap();
        assert_eq!(binding.secondary, Some(75));
    }

    #[test]
    fn clear_binding() {
        let mut config = KeyConfig::default_7k();
        config.clear_binding("1P_KEY1");
        let binding = config.get_binding("1P_KEY1").unwrap();
        assert!(!binding.is_bound());
    }

    #[test]
    fn empty_binding() {
        let binding = KeyBinding::empty("TEST".to_string());
        assert!(!binding.is_bound());
    }

    #[test]
    fn binding_is_bound() {
        let mut binding = KeyBinding::empty("TEST".to_string());
        assert!(!binding.is_bound());

        binding.controller_button = Some(5);
        assert!(binding.is_bound());
    }

    #[test]
    fn serialization_round_trip() {
        let config = KeyConfig::default_7k();
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: KeyConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.bindings.len(), 8);
        assert_eq!(
            deserialized.get_binding("1P_SCRATCH").unwrap().primary,
            Some(16)
        );
    }
}
