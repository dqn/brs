// Stubs for external dependencies not yet available as proper imports.
// These will be replaced with real imports as the corresponding crates are translated.

use beatoraja_core::main_state::MainStateType;

// MainControllerAccess — re-exported from beatoraja-types (Phase 15d)
pub use beatoraja_types::main_controller_access::MainControllerAccess;
pub use beatoraja_types::main_state_type::MainStateType as TypesMainStateType;
pub use beatoraja_types::player_resource_access::PlayerResourceAccess;

/// Stub for MainController reference.
/// In Java, MainController.getStateType(MainState) returns the MainStateType.
pub struct MainControllerRef;

impl MainControllerAccess for MainControllerRef {
    fn get_config(&self) -> &beatoraja_types::config::Config {
        todo!()
    }
    fn get_player_config(&self) -> &beatoraja_types::player_config::PlayerConfig {
        todo!()
    }
    fn change_state(&mut self, _state: TypesMainStateType) {
        todo!()
    }
    fn save_config(&self) {
        todo!()
    }
    fn exit(&self) {
        todo!()
    }
    fn save_last_recording(&self, _reason: &str) {
        todo!()
    }
    fn update_song(&mut self, _path: Option<&str>) {
        todo!()
    }
    fn get_player_resource(&self) -> Option<&dyn PlayerResourceAccess> {
        None
    }
    fn get_player_resource_mut(&mut self) -> Option<&mut dyn PlayerResourceAccess> {
        None
    }
}

impl MainControllerRef {
    pub fn get_state_type(
        _state: &dyn beatoraja_core::main_state::MainState,
    ) -> Option<MainStateType> {
        todo!("Phase 8+ dependency: MainController.getStateType")
    }
}

/// Stub for ImGuiNotify (from beatoraja-modmenu, not yet available as cross-dep)
pub struct ImGuiNotify;

impl ImGuiNotify {
    pub fn info(message: &str) {
        log::info!("ImGuiNotify: {}", message);
    }
}

/// Constants from ObsConfigurationView (from beatoraja-launcher, not yet available)
pub const SCENE_NONE: &str = "(No Change)";
pub const ACTION_NONE: &str = "(Do Nothing)";
