/// Trait for beatoraja.MainState
///
/// Extends `SkinRenderContext` (which extends `TimerAccess`) so that all
/// property value, config access, event, gauge, judge, audio, and timer
/// methods are inherited from `SkinRenderContext`.
///
/// Only methods that depend on skin-crate-local types (`MainController`,
/// `PlayerResource`, `TextureRegion`) remain here.
pub trait MainState: rubato_types::skin_render_context::SkinRenderContext {
    fn timer(&self) -> &dyn rubato_types::timer_access::TimerAccess;
    fn get_main(&self) -> &super::stubs::MainController;
    fn get_image(&self, id: i32) -> Option<super::rendering_stubs::TextureRegion>;
    fn get_resource(&self) -> &super::stubs::PlayerResource;

    /// Select a song with the given play mode.
    /// Only meaningful for MusicSelector.
    /// Note: SkinRenderContext has `select_song_mode(event_id: i32)` with a different signature.
    fn select_song(&mut self, _mode: rubato_core::bms_player_mode::BMSPlayerMode) {
        // default no-op
    }

    // ============================================================
    // Backward-compatibility shims (Phase 3b)
    // These delegate to the renamed SkinRenderContext methods so that
    // existing callers continue to compile until Phase 3c migrates them.
    // ============================================================

    /// Deprecated: use `SkinRenderContext::gauge_value()` instead.
    fn get_gauge_value(&self) -> f32 {
        self.gauge_value()
    }

    /// Deprecated: use `SkinRenderContext::now_judge()` instead.
    fn get_now_judge(&self, player: i32) -> i32 {
        self.now_judge(player)
    }

    /// Deprecated: use `SkinRenderContext::now_combo()` instead.
    fn get_now_combo(&self, player: i32) -> i32 {
        self.now_combo(player)
    }

    /// Deprecated: use `SkinRenderContext::player_config_ref()` instead.
    fn get_player_config_ref(&self) -> Option<&rubato_types::player_config::PlayerConfig> {
        self.player_config_ref()
    }

    /// Deprecated: use `SkinRenderContext::config_ref()` instead.
    fn get_config_ref(&self) -> Option<&rubato_types::config::Config> {
        self.config_ref()
    }

    /// Deprecated: use `SkinRenderContext::config_mut()` instead.
    fn get_config_mut(&mut self) -> Option<&mut rubato_types::config::Config> {
        self.config_mut()
    }

    /// Deprecated: use `SkinRenderContext::selected_play_config_mut()` instead.
    fn get_selected_play_config_mut(
        &mut self,
    ) -> Option<&mut rubato_types::play_config::PlayConfig> {
        self.selected_play_config_mut()
    }

    /// Deprecated: use `SkinRenderContext::current_play_config_ref()` instead.
    fn get_selected_play_config_ref(&self) -> Option<&rubato_types::play_config::PlayConfig> {
        self.current_play_config_ref()
    }
}
