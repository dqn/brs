// Configuration structs (system, player, play, skin, audio, IR)

pub mod audio_config;
pub mod config;
pub mod ir_config;
pub mod play_config;
pub mod play_mode_config;
pub mod player_config;
pub mod resolution;
pub mod skin_config;
pub mod skin_type;

pub use audio_config::{AudioConfig, DriverType, FrequencyType};
pub use config::{Config, DisplayMode, SongPreview};
pub use ir_config::IRConfig;
pub use play_config::PlayConfig;
pub use play_mode_config::{
    ControllerConfig, KeyboardConfig, MidiConfig, MidiInput, MidiInputType, MouseScratchConfig,
    PlayModeConfig,
};
pub use player_config::PlayerConfig;
pub use resolution::Resolution;
pub use skin_config::{FilePath, Offset, Property, SkinConfig, SkinOption};
pub use skin_type::SkinType;
