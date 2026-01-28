//! Skin asset management
//!
//! Handles loading, caching, and management of skin textures and resources.

#![allow(dead_code)]

pub mod cache;
pub mod region;

pub use cache::{TextureCache, TextureId};
pub use region::ImageRegion;
