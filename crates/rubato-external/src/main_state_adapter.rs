// MainState concrete struct bridging external code to skin's property system.

use rubato_core::config::Config;
use rubato_types::screen_type::ScreenType;

use crate::player_resource_adapter::PlayerResource;

/// Legacy MainState wrapper for external code that accesses `state.resource`.
/// Implements MainStateAccess and provides direct field access for compatibility.
pub struct MainState {
    pub resource: PlayerResource,
    pub screen_type: ScreenType,
}

impl rubato_types::main_state_access::MainStateAccess for MainState {
    fn screen_type(&self) -> ScreenType {
        self.screen_type
    }

    fn resource(&self) -> Option<&dyn rubato_types::player_resource_access::PlayerResourceAccess> {
        Some(&*self.resource.inner)
    }

    fn config(&self) -> &Config {
        self.resource.config()
    }
}

impl Default for MainState {
    fn default() -> Self {
        Self {
            resource: PlayerResource::default(),
            screen_type: ScreenType::Other,
        }
    }
}

// skin::MainState trait impl — bridges external's concrete MainState
// to skin's property system (resolves type mismatch, not a circular dep)

impl rubato_types::timer_access::TimerAccess for MainState {
    fn now_time(&self) -> i64 {
        0
    }
    fn now_micro_time(&self) -> i64 {
        0
    }
    fn micro_timer(&self, _: rubato_types::timer_id::TimerId) -> i64 {
        i64::MIN
    }
    fn timer(&self, _: rubato_types::timer_id::TimerId) -> i64 {
        i64::MIN
    }
    fn now_time_for(&self, _: rubato_types::timer_id::TimerId) -> i64 {
        0
    }
    fn is_timer_on(&self, _: rubato_types::timer_id::TimerId) -> bool {
        false
    }
}

impl rubato_types::skin_render_context::SkinRenderContext for MainState {}

impl rubato_skin::stubs::MainState for MainState {}

#[cfg(test)]
mod tests {
    use super::*;
    use rubato_types::main_state_access::MainStateAccess;

    #[test]
    fn main_state_default_screen_type_is_other() {
        let state = MainState::default();
        assert_eq!(state.screen_type(), ScreenType::Other);
    }

    #[test]
    fn main_state_with_screen_type_returns_correct_type() {
        let state = MainState {
            resource: PlayerResource::default(),
            screen_type: ScreenType::MusicSelector,
        };
        assert_eq!(state.screen_type(), ScreenType::MusicSelector);
    }

    #[test]
    fn main_state_with_each_screen_type_variant() {
        let variants = vec![
            ScreenType::MusicSelector,
            ScreenType::MusicDecide,
            ScreenType::BMSPlayer,
            ScreenType::MusicResult,
            ScreenType::CourseResult,
            ScreenType::KeyConfiguration,
            ScreenType::Other,
        ];
        for variant in variants {
            let state = MainState {
                resource: PlayerResource::default(),
                screen_type: variant,
            };
            assert_eq!(state.screen_type(), variant);
        }
    }
}
