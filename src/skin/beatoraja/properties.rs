//! Custom property system for beatoraja skins
//!
//! Manages user-configurable skin properties.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::types::{CustomProperty, PropertyOption};

/// Property type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PropertyType {
    /// Item selection (dropdown/list)
    Item,
    /// Slider (numeric range)
    Slider,
    /// File path selection
    File,
}

impl From<i32> for PropertyType {
    fn from(value: i32) -> Self {
        match value {
            0 => PropertyType::Item,
            1 => PropertyType::Slider,
            2 => PropertyType::File,
            _ => PropertyType::Item,
        }
    }
}

/// Parsed property definition
#[derive(Debug, Clone)]
pub struct ParsedProperty {
    /// Property name (display)
    pub name: String,
    /// Property type
    pub property_type: PropertyType,
    /// Options for item type
    pub options: Vec<ParsedOption>,
    /// Min value for slider type
    pub min: i32,
    /// Max value for slider type
    pub max: i32,
    /// Default value
    pub default: i32,
    /// Operation code to set
    pub operation: i32,
}

/// Parsed option for item properties
#[derive(Debug, Clone)]
pub struct ParsedOption {
    /// Option name (display)
    pub name: String,
    /// Option value (op code)
    pub value: i32,
}

/// Property manager for skin customization
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PropertyManager {
    /// Property values by operation code
    #[serde(default)]
    values: HashMap<i32, i32>,
    /// Custom file paths by operation code
    #[serde(default)]
    file_paths: HashMap<i32, String>,
}

impl PropertyManager {
    /// Create a new property manager
    pub fn new() -> Self {
        Self::default()
    }

    /// Initialize from property definitions
    pub fn from_definitions(properties: &[CustomProperty]) -> Self {
        let mut manager = Self::new();

        for prop in properties {
            // Set default values
            manager.values.insert(prop.operation, prop.def);
        }

        manager
    }

    /// Get property value
    pub fn get_value(&self, operation: i32) -> i32 {
        self.values.get(&operation).copied().unwrap_or(0)
    }

    /// Set property value
    pub fn set_value(&mut self, operation: i32, value: i32) {
        self.values.insert(operation, value);
    }

    /// Get file path
    pub fn get_file_path(&self, operation: i32) -> Option<&str> {
        self.file_paths.get(&operation).map(|s| s.as_str())
    }

    /// Set file path
    pub fn set_file_path(&mut self, operation: i32, path: String) {
        self.file_paths.insert(operation, path);
    }

    /// Check if an option is selected (for condition evaluation)
    pub fn is_option_selected(&self, operation: i32, option_value: i32) -> bool {
        self.get_value(operation) == option_value
    }

    /// Get all property values
    pub fn all_values(&self) -> &HashMap<i32, i32> {
        &self.values
    }

    /// Apply property values to game state custom options
    pub fn apply_to_conditions(&self, custom_options: &mut HashMap<i32, bool>) {
        for &value in self.values.values() {
            // For item properties, only the selected option's value is true
            // For slider properties, the value is stored but not directly used as bool
            custom_options.insert(value, true);
        }
    }
}

/// Parse property definitions from skin
pub fn parse_properties(properties: &[CustomProperty]) -> Vec<ParsedProperty> {
    properties
        .iter()
        .map(|prop| ParsedProperty {
            name: prop.name.clone(),
            property_type: PropertyType::from(prop.property_type),
            options: prop
                .options
                .iter()
                .map(|opt| ParsedOption {
                    name: opt.name.clone(),
                    value: opt.value,
                })
                .collect(),
            min: prop.min,
            max: prop.max,
            default: prop.def,
            operation: prop.operation,
        })
        .collect()
}

/// Build a property definition for testing/serialization
pub fn build_property(
    name: &str,
    options: &[(&str, i32)],
    default: i32,
    operation: i32,
) -> CustomProperty {
    CustomProperty {
        name: name.to_string(),
        property_type: 0, // Item
        options: options
            .iter()
            .map(|(n, v)| PropertyOption {
                name: n.to_string(),
                value: *v,
            })
            .collect(),
        min: 0,
        max: 0,
        def: default,
        operation,
    }
}

/// Build a slider property definition
pub fn build_slider_property(
    name: &str,
    min: i32,
    max: i32,
    default: i32,
    operation: i32,
) -> CustomProperty {
    CustomProperty {
        name: name.to_string(),
        property_type: 1, // Slider
        options: vec![],
        min,
        max,
        def: default,
        operation,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_property_type_from() {
        assert_eq!(PropertyType::from(0), PropertyType::Item);
        assert_eq!(PropertyType::from(1), PropertyType::Slider);
        assert_eq!(PropertyType::from(2), PropertyType::File);
        assert_eq!(PropertyType::from(99), PropertyType::Item); // Unknown defaults to Item
    }

    #[test]
    fn test_property_manager_basic() {
        let mut manager = PropertyManager::new();

        manager.set_value(900, 42);
        assert_eq!(manager.get_value(900), 42);
        assert_eq!(manager.get_value(901), 0); // Not set

        manager.set_file_path(950, "/path/to/file.png".to_string());
        assert_eq!(manager.get_file_path(950), Some("/path/to/file.png"));
        assert_eq!(manager.get_file_path(951), None);
    }

    #[test]
    fn test_property_manager_from_definitions() {
        let properties = vec![
            build_property("Lane Type", &[("Type A", 900), ("Type B", 901)], 900, 900),
            build_slider_property("Brightness", 0, 100, 50, 910),
        ];

        let manager = PropertyManager::from_definitions(&properties);
        assert_eq!(manager.get_value(900), 900); // Default
        assert_eq!(manager.get_value(910), 50); // Default
    }

    #[test]
    fn test_option_selection() {
        let mut manager = PropertyManager::new();
        manager.set_value(900, 901); // Select "Type B"

        assert!(!manager.is_option_selected(900, 900)); // Type A not selected
        assert!(manager.is_option_selected(900, 901)); // Type B selected
    }

    #[test]
    fn test_apply_to_conditions() {
        let mut manager = PropertyManager::new();
        manager.set_value(900, 901);
        manager.set_value(910, 50);

        let mut conditions = HashMap::new();
        manager.apply_to_conditions(&mut conditions);

        assert!(conditions.get(&901).copied().unwrap_or(false));
        assert!(conditions.get(&50).copied().unwrap_or(false));
    }

    #[test]
    fn test_parse_properties() {
        let properties = vec![build_property(
            "Test Prop",
            &[("Option 1", 900), ("Option 2", 901)],
            900,
            900,
        )];

        let parsed = parse_properties(&properties);
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].name, "Test Prop");
        assert_eq!(parsed[0].property_type, PropertyType::Item);
        assert_eq!(parsed[0].options.len(), 2);
        assert_eq!(parsed[0].default, 900);
    }
}
