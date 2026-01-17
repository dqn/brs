mod bga;
mod config;
mod effects;
pub mod font;
mod highway;
mod lane_cover;
mod video;

// Public API for library consumers
pub use bga::BgaManager;
#[allow(unused_imports)]
pub use config::HighwayConfig;
pub use effects::EffectManager;
pub use highway::Highway;
pub use lane_cover::LaneCover;
#[allow(unused_imports)]
pub use video::VideoDecoder;
