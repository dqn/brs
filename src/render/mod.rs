mod config;
mod effects;
mod highway;
mod lane_cover;

// Public API for library consumers
#[allow(unused_imports)]
pub use config::HighwayConfig;
pub use effects::EffectManager;
pub use highway::Highway;
