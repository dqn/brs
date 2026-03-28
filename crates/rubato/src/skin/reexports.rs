// Re-exports for backward compatibility.
// All shadow types have been removed; callers use SkinRenderContext methods instead.

// Re-export all rendering types (wgpu-backed LibGDX equivalents)
pub use crate::skin::render_reexports::*;

// MainState trait -- canonical definition in crate::skin::main_state
pub use crate::skin::main_state::MainState;

// Timer -- canonical definition in crate::skin::skin_timer
pub use crate::skin::skin_timer::Timer;

// Resolution -- canonical definition in crate::skin::skin_resolution
pub use crate::skin::skin_resolution::Resolution;

// SkinConfigOffset -- canonical definition in crate::skin::skin_config_offset
pub use crate::skin::skin_config_offset::SkinConfigOffset;

// SkinOffset -- re-exported from rubato-types
pub use crate::skin::skin_offset::SkinOffset;

// TimingDistribution -- re-exported from rubato-types
pub use crate::skin::timing_distribution::TimingDistribution;

// beatoraja.song types (re-exports from rubato-types)
pub use crate::skin::song_data::SongData;
pub use crate::skin::song_information::SongInformation;
