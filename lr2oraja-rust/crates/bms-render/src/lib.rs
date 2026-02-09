// Rendering: Bevy setup, sprite batch, blend modes, fonts.
//
// Uses Bevy 0.15 for window, camera, and 2D sprite rendering.
// Skin objects are iterated in Vec order each frame (procedural render loop).

pub mod blend;
pub mod coord;
pub mod draw;
pub mod image_loader_bevy;
pub mod plugin;
pub mod skin_renderer;
pub mod state_provider;
pub mod texture_map;
