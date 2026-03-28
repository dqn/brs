// Re-exports for backward compatibility.
// All shadow types have been removed; callers use SkinRenderContext methods instead.

// Re-export all rendering types (wgpu-backed LibGDX equivalents)
pub use crate::render_reexports::*;

// MainState trait -- canonical definition in crate::main_state
pub use crate::main_state::MainState;

// Timer -- canonical definition in crate::skin_timer
pub use crate::skin_timer::Timer;

// Resolution -- canonical definition in crate::skin_resolution
pub use crate::skin_resolution::Resolution;

// SkinConfigOffset -- canonical definition in crate::skin_config_offset
pub use crate::skin_config_offset::SkinConfigOffset;

// SkinOffset -- re-exported from rubato-types
pub use crate::skin_offset::SkinOffset;

// TimingDistribution -- re-exported from rubato-types
pub use crate::timing_distribution::TimingDistribution;

// beatoraja.song types (re-exports from rubato-types)
pub use crate::song_data::SongData;
pub use crate::song_information::SongInformation;
