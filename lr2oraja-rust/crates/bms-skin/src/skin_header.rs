// SkinHeader ported from SkinHeader.java.
//
// Stores skin metadata and user-customizable options, files, and offsets.
// Resolution types and SkinType are reused from bms-config.

use std::path::PathBuf;

use bms_config::resolution::Resolution;
use bms_config::skin_type::SkinType;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Skin format type
// ---------------------------------------------------------------------------

/// The format of the skin definition file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum SkinFormat {
    /// LR2 format skin (.lr2skin / CSV).
    #[default]
    Lr2 = 0,
    /// beatoraja JSON format skin (.json).
    Beatoraja = 1,
    /// Lua format skin (.luaskin).
    Lua = 2,
}

// ---------------------------------------------------------------------------
// CustomOption
// ---------------------------------------------------------------------------

/// A user-selectable option group.
///
/// Each option has a set of IDs and display labels. The user selects one,
/// and the corresponding boolean property ID becomes active.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomOption {
    /// Display name for this option group.
    pub name: String,
    /// Option IDs (used as boolean property IDs).
    pub option_ids: Vec<i32>,
    /// Display labels for each option.
    pub contents: Vec<String>,
    /// Default option label (None = first option).
    pub default_label: Option<String>,
    /// Currently selected index (-1 = unset).
    pub selected_index: i32,
}

impl CustomOption {
    pub fn new(name: String, option_ids: Vec<i32>, contents: Vec<String>) -> Self {
        Self {
            name,
            option_ids,
            contents,
            default_label: None,
            selected_index: -1,
        }
    }

    /// Returns the option ID for the default selection.
    pub fn default_option(&self) -> Option<i32> {
        if let Some(def) = &self.default_label {
            for (i, label) in self.contents.iter().enumerate() {
                if label == def {
                    return self.option_ids.get(i).copied();
                }
            }
        }
        self.option_ids.first().copied()
    }

    /// Returns the currently selected option ID, or None if unset.
    pub fn selected_option(&self) -> Option<i32> {
        let idx = self.selected_index as usize;
        if self.selected_index >= 0 && idx < self.option_ids.len() {
            Some(self.option_ids[idx])
        } else {
            None
        }
    }
}

// ---------------------------------------------------------------------------
// CustomFile
// ---------------------------------------------------------------------------

/// A user-selectable file path.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomFile {
    /// Display name for this file selection.
    pub name: String,
    /// File path pattern (may contain wildcards like `*.png`).
    pub path: String,
    /// Default filename.
    pub default_filename: Option<String>,
    /// Currently selected filename.
    pub selected_filename: Option<String>,
}

impl CustomFile {
    pub fn new(name: String, path: String, default_filename: Option<String>) -> Self {
        Self {
            name,
            path,
            default_filename,
            selected_filename: None,
        }
    }
}

// ---------------------------------------------------------------------------
// CustomOffset
// ---------------------------------------------------------------------------

/// A user-editable offset value with per-axis enable flags.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomOffset {
    /// Display name for this offset.
    pub name: String,
    /// Offset ID (maps to SkinProperty OFFSET constants).
    pub id: i32,
    /// Which axes are editable by the user.
    pub editable_x: bool,
    pub editable_y: bool,
    pub editable_w: bool,
    pub editable_h: bool,
    pub editable_r: bool,
    pub editable_a: bool,
}

impl CustomOffset {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        name: String,
        id: i32,
        x: bool,
        y: bool,
        w: bool,
        h: bool,
        r: bool,
        a: bool,
    ) -> Self {
        Self {
            name,
            id,
            editable_x: x,
            editable_y: y,
            editable_w: w,
            editable_h: h,
            editable_r: r,
            editable_a: a,
        }
    }
}

// ---------------------------------------------------------------------------
// CustomCategory
// ---------------------------------------------------------------------------

/// A category that groups custom items (options, files, offsets) for UI display.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomCategory {
    /// Category display name.
    pub name: String,
    /// Indices of items belonging to this category.
    /// Items can be options, files, or offsets — stored as tagged indices.
    pub items: Vec<CustomCategoryItem>,
}

/// A reference to a custom item within a category.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CustomCategoryItem {
    Option(usize),
    File(usize),
    Offset(usize),
}

// ---------------------------------------------------------------------------
// SkinHeader
// ---------------------------------------------------------------------------

/// Skin metadata and customization definitions.
#[derive(Debug, Clone)]
pub struct SkinHeader {
    /// Skin format type.
    pub format: SkinFormat,
    /// Path to the skin definition file.
    pub path: Option<PathBuf>,
    /// Skin type (play, select, result, etc.).
    pub skin_type: Option<SkinType>,
    /// Skin display name.
    pub name: String,
    /// Skin author name.
    pub author: String,
    /// User-customizable options.
    pub options: Vec<CustomOption>,
    /// User-customizable file paths.
    pub files: Vec<CustomFile>,
    /// User-customizable offsets.
    pub offsets: Vec<CustomOffset>,
    /// Item categories for UI grouping.
    pub categories: Vec<CustomCategory>,
    /// Skin design resolution.
    pub resolution: Resolution,
    /// Source resolution (for scaling calculations).
    pub source_resolution: Option<Resolution>,
    /// Destination resolution (for scaling calculations).
    pub destination_resolution: Option<Resolution>,
}

impl Default for SkinHeader {
    fn default() -> Self {
        Self {
            format: SkinFormat::default(),
            path: None,
            skin_type: None,
            name: String::new(),
            author: String::new(),
            options: Vec::new(),
            files: Vec::new(),
            offsets: Vec::new(),
            categories: Vec::new(),
            resolution: Resolution::Sd,
            source_resolution: None,
            destination_resolution: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_header() {
        let h = SkinHeader::default();
        assert_eq!(h.format, SkinFormat::Lr2);
        assert!(h.path.is_none());
        assert!(h.skin_type.is_none());
        assert!(h.name.is_empty());
        assert!(h.options.is_empty());
        assert_eq!(h.resolution, Resolution::Sd);
    }

    #[test]
    fn test_custom_option_default() {
        let opt = CustomOption::new(
            "BGA Size".to_string(),
            vec![900, 901, 902],
            vec![
                "Normal".to_string(),
                "Extend".to_string(),
                "Off".to_string(),
            ],
        );
        assert_eq!(opt.default_option(), Some(900));
        assert_eq!(opt.selected_option(), None);
    }

    #[test]
    fn test_custom_option_with_default() {
        let mut opt = CustomOption::new(
            "Ghost".to_string(),
            vec![910, 911],
            vec!["Type A".to_string(), "Type B".to_string()],
        );
        opt.default_label = Some("Type B".to_string());
        assert_eq!(opt.default_option(), Some(911));
    }

    #[test]
    fn test_custom_option_selected() {
        let mut opt = CustomOption::new(
            "Test".to_string(),
            vec![100, 101, 102],
            vec!["A".to_string(), "B".to_string(), "C".to_string()],
        );
        opt.selected_index = 1;
        assert_eq!(opt.selected_option(), Some(101));

        opt.selected_index = -1;
        assert_eq!(opt.selected_option(), None);

        opt.selected_index = 99;
        assert_eq!(opt.selected_option(), None);
    }

    #[test]
    fn test_custom_file() {
        let f = CustomFile::new(
            "Background".to_string(),
            "bg/*.png".to_string(),
            Some("default.png".to_string()),
        );
        assert_eq!(f.name, "Background");
        assert!(f.selected_filename.is_none());
    }

    #[test]
    fn test_custom_offset() {
        let o = CustomOffset::new(
            "Lift".to_string(),
            3,
            false,
            true,
            false,
            false,
            false,
            false,
        );
        assert_eq!(o.id, 3);
        assert!(!o.editable_x);
        assert!(o.editable_y);
    }

    #[test]
    fn test_custom_category() {
        let cat = CustomCategory {
            name: "Display".to_string(),
            items: vec![
                CustomCategoryItem::Option(0),
                CustomCategoryItem::File(0),
                CustomCategoryItem::Offset(0),
            ],
        };
        assert_eq!(cat.items.len(), 3);
    }
}
