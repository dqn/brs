use crate::config::Config;
use crate::main_state_type::MainStateType;
use crate::player_config::PlayerConfig;
use crate::player_resource_access::PlayerResourceAccess;

/// Trait interface for MainController access.
///
/// Downstream crates use `&dyn MainControllerAccess` instead of concrete MainController stubs.
/// The real implementation in beatoraja-core implements this trait.
///
/// Methods that return types not available in beatoraja-types (e.g., BMSPlayerInputProcessor,
/// SystemSoundManager, IRStatus) are NOT included here. Downstream crates that need those
/// methods should keep local extension stubs until the types are unified.
pub trait MainControllerAccess {
    /// Get config reference
    fn get_config(&self) -> &Config;

    /// Get player config reference
    fn get_player_config(&self) -> &PlayerConfig;

    /// Change to a different state
    fn change_state(&mut self, state: MainStateType);

    /// Save config to disk
    fn save_config(&self);

    /// Exit the application
    fn exit(&self);

    /// Save OBS last recording with the given reason tag
    fn save_last_recording(&self, reason: &str);

    /// Update song database for the given path
    fn update_song(&mut self, path: Option<&str>);

    /// Get player resource (immutable)
    fn get_player_resource(&self) -> Option<&dyn PlayerResourceAccess>;

    /// Get player resource (mutable)
    fn get_player_resource_mut(&mut self) -> Option<&mut dyn PlayerResourceAccess>;
}

/// Null implementation of MainControllerAccess for stub contexts.
/// All methods panic with todo!().
pub struct NullMainController;

impl MainControllerAccess for NullMainController {
    fn get_config(&self) -> &Config {
        todo!("NullMainController::get_config")
    }
    fn get_player_config(&self) -> &PlayerConfig {
        todo!("NullMainController::get_player_config")
    }
    fn change_state(&mut self, _state: MainStateType) {
        todo!("NullMainController::change_state")
    }
    fn save_config(&self) {
        todo!("NullMainController::save_config")
    }
    fn exit(&self) {
        todo!("NullMainController::exit")
    }
    fn save_last_recording(&self, _reason: &str) {
        todo!("NullMainController::save_last_recording")
    }
    fn update_song(&mut self, _path: Option<&str>) {
        todo!("NullMainController::update_song")
    }
    fn get_player_resource(&self) -> Option<&dyn PlayerResourceAccess> {
        None
    }
    fn get_player_resource_mut(&mut self) -> Option<&mut dyn PlayerResourceAccess> {
        None
    }
}
