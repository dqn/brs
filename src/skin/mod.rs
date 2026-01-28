//! Custom skin system for visual customization
//!
//! This module provides configurable visual themes including:
//! - Lane and note colors
//! - Judge line appearance
//! - Effect colors and timing
//! - UI element positioning
//!
//! ## beatoraja Skin Support
//!
//! The `beatoraja` submodule provides compatibility with beatoraja skins:
//! - JSON format (`.json`)
//! - Lua format (`.luaskin` + `.lua`)

#![allow(dead_code)]
#![allow(unused_imports)]

pub mod assets;
pub mod beatoraja;
mod definition;
mod layout;
mod loader;
mod theme;

// Re-export commonly used types
pub use definition::{Resolution, SkinDefinition, SkinInfo};
pub use layout::{
    BpmDisplayLayout, GaugeDisplayLayout, GraphAreaLayout, IidxLayout, InfoAreaLayout,
    JudgeStatsLayout, LayoutConfig, PlayAreaLayout, Rect, ScoreAreaLayout,
};
pub use loader::{SkinFormatType, SkinLoader};
pub use theme::{EffectConfig, SkinTheme};
