//! Custom skin system for visual customization
//!
//! This module provides configurable visual themes including:
//! - Lane and note colors
//! - Judge line appearance
//! - Effect colors and timing
//! - UI element positioning

mod definition;
mod layout;
mod loader;
mod theme;

// Re-export commonly used types
pub use layout::{IidxLayout, InfoAreaLayout, LayoutConfig, PlayAreaLayout, Rect};
pub use loader::SkinLoader;
pub use theme::{EffectConfig, SkinTheme};
