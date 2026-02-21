// Phase 11: stubs replaced with real imports where possible.
// Remaining stubs are for types from crates that beatoraja-play cannot depend on
// (beatoraja-skin circular dep).

// Re-export from beatoraja-core
pub use beatoraja_core::main_controller::MainController;

// Re-export from beatoraja-render (was a stub, now uses real type)
pub use beatoraja_render::Texture;
