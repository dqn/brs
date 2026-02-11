// SkinConfig state — skin selection and customization screen.
//
// Allows users to browse available skin types, select skins,
// and configure custom options per skin.

use tracing::info;

use bms_config::SkinType;
use bms_input::control_keys::ControlKeys;

use crate::app_state::AppStateType;
use crate::state::{GameStateHandler, StateContext};

/// Skin types available for configuration.
/// Subset of SkinType that users can configure (excludes SoundSet, Theme, battle variants).
const CONFIGURABLE_SKIN_TYPES: &[SkinType] = &[
    SkinType::MusicSelect,
    SkinType::Decide,
    SkinType::Result,
    SkinType::CourseResult,
    SkinType::KeyConfig,
    SkinType::SkinSelect,
    SkinType::Play7Keys,
    SkinType::Play5Keys,
    SkinType::Play14Keys,
    SkinType::Play10Keys,
    SkinType::Play9Keys,
    SkinType::Play24Keys,
    SkinType::Play24KeysDouble,
];

/// Skin configuration state — browse and configure skins.
pub struct SkinConfigState {
    skin_type_index: usize,
    cursor: usize,
}

impl SkinConfigState {
    pub fn new() -> Self {
        Self {
            skin_type_index: 0,
            cursor: 0,
        }
    }

    fn current_skin_type(&self) -> SkinType {
        CONFIGURABLE_SKIN_TYPES[self.skin_type_index]
    }
}

impl Default for SkinConfigState {
    fn default() -> Self {
        Self::new()
    }
}

impl GameStateHandler for SkinConfigState {
    fn create(&mut self, _ctx: &mut StateContext) {
        self.cursor = 0;
        info!(skin_type = self.current_skin_type().name(), "SkinConfig: create");
    }

    fn render(&mut self, _ctx: &mut StateContext) {
        // SkinConfig is input-driven, no timer-based transitions.
    }

    fn input(&mut self, ctx: &mut StateContext) {
        if let Some(input_state) = ctx.input_state {
            for key in &input_state.pressed_keys {
                match key {
                    ControlKeys::Left => {
                        // Previous skin type
                        if self.skin_type_index == 0 {
                            self.skin_type_index = CONFIGURABLE_SKIN_TYPES.len() - 1;
                        } else {
                            self.skin_type_index -= 1;
                        }
                        self.cursor = 0;
                        info!(skin_type = self.current_skin_type().name(), "SkinConfig: type changed");
                        return;
                    }
                    ControlKeys::Right => {
                        // Next skin type
                        self.skin_type_index =
                            (self.skin_type_index + 1) % CONFIGURABLE_SKIN_TYPES.len();
                        self.cursor = 0;
                        info!(skin_type = self.current_skin_type().name(), "SkinConfig: type changed");
                        return;
                    }
                    ControlKeys::Up => {
                        // Scroll custom options up
                        let option_count = self.option_count(ctx);
                        if option_count > 0 {
                            if self.cursor == 0 {
                                self.cursor = option_count - 1;
                            } else {
                                self.cursor -= 1;
                            }
                        }
                        return;
                    }
                    ControlKeys::Down => {
                        // Scroll custom options down
                        let option_count = self.option_count(ctx);
                        if option_count > 0 {
                            self.cursor = (self.cursor + 1) % option_count;
                        }
                        return;
                    }
                    ControlKeys::Enter => {
                        // Cycle custom option value forward
                        self.cycle_option_value(ctx, 1);
                        return;
                    }
                    ControlKeys::Del => {
                        // Cycle custom option value backward
                        self.cycle_option_value(ctx, -1);
                        return;
                    }
                    ControlKeys::Escape => {
                        // Save and exit
                        ctx.resource.config_save_requested = true;
                        *ctx.transition = Some(AppStateType::MusicSelect);
                        info!("SkinConfig: save and exit");
                        return;
                    }
                    _ => {}
                }
            }
        }
    }

    fn shutdown(&mut self, _ctx: &mut StateContext) {
        info!("SkinConfig: shutdown");
    }
}

impl SkinConfigState {
    /// Get the number of custom options for the current skin type.
    fn option_count(&self, ctx: &StateContext) -> usize {
        let skin_id = self.current_skin_type().id() as usize;
        ctx.player_config
            .skin
            .get(skin_id)
            .and_then(|sc| sc.properties.as_ref())
            .map(|p| p.option.len())
            .unwrap_or(0)
    }

    /// Cycle the current option value by delta.
    fn cycle_option_value(&self, ctx: &mut StateContext, delta: i32) {
        let skin_id = self.current_skin_type().id() as usize;
        if let Some(skin_config) = ctx.player_config.skin.get_mut(skin_id) {
            if let Some(props) = skin_config.properties.as_mut() {
                if let Some(opt) = props.option.get_mut(self.cursor) {
                    opt.value += delta;
                    info!(name = %opt.name, value = opt.value, "SkinConfig: option changed");
                }
            }
        }
    }
}

#[cfg(test)]
impl SkinConfigState {
    pub(crate) fn skin_type_index(&self) -> usize {
        self.skin_type_index
    }

    pub(crate) fn cursor(&self) -> usize {
        self.cursor
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input_mapper::InputState;
    use crate::player_resource::PlayerResource;
    use crate::timer_manager::TimerManager;
    use bms_config::{Config, PlayerConfig, Property, SkinOption};

    fn make_ctx<'a>(
        timer: &'a mut TimerManager,
        resource: &'a mut PlayerResource,
        config: &'a Config,
        player_config: &'a mut PlayerConfig,
        transition: &'a mut Option<AppStateType>,
    ) -> StateContext<'a> {
        StateContext {
            timer,
            resource,
            config,
            player_config,
            transition,
            keyboard_backend: None,
            database: None,
            input_state: None,
            skin_manager: None,
            sound_manager: None,
        }
    }

    fn make_input_ctx<'a>(
        timer: &'a mut TimerManager,
        resource: &'a mut PlayerResource,
        config: &'a Config,
        player_config: &'a mut PlayerConfig,
        transition: &'a mut Option<AppStateType>,
        input: &'a InputState,
    ) -> StateContext<'a> {
        StateContext {
            timer,
            resource,
            config,
            player_config,
            transition,
            keyboard_backend: None,
            database: None,
            input_state: Some(input),
            skin_manager: None,
            sound_manager: None,
        }
    }

    /// Create a validated PlayerConfig with custom options on MusicSelect skin (index 5).
    fn player_config_with_options(options: Vec<SkinOption>) -> PlayerConfig {
        let mut pc = PlayerConfig::default();
        pc.validate();
        // MusicSelect is the first entry in CONFIGURABLE_SKIN_TYPES with id=5.
        pc.skin[SkinType::MusicSelect.id() as usize].properties = Some(Property {
            option: options,
            file: Vec::new(),
            offset: Vec::new(),
        });
        pc
    }

    #[test]
    fn skin_type_switch_wraps_forward() {
        let mut state = SkinConfigState::new();
        let mut timer = TimerManager::new();
        let mut resource = PlayerResource::default();
        let config = Config::default();
        let mut player_config = PlayerConfig::default();
        player_config.validate();
        let mut transition = None;

        // Press Right repeatedly to go past the last type and wrap to 0.
        for _ in 0..CONFIGURABLE_SKIN_TYPES.len() {
            let input = InputState {
                commands: vec![],
                pressed_keys: vec![ControlKeys::Right],
            };
            let mut ctx = make_input_ctx(
                &mut timer,
                &mut resource,
                &config,
                &mut player_config,
                &mut transition,
                &input,
            );
            state.input(&mut ctx);
        }
        // After cycling through all types, we should be back at 0.
        assert_eq!(state.skin_type_index(), 0);
    }

    #[test]
    fn skin_type_switch_wraps_backward() {
        let mut state = SkinConfigState::new();
        let mut timer = TimerManager::new();
        let mut resource = PlayerResource::default();
        let config = Config::default();
        let mut player_config = PlayerConfig::default();
        player_config.validate();
        let mut transition = None;

        // Press Left once from index 0 should wrap to last.
        let input = InputState {
            commands: vec![],
            pressed_keys: vec![ControlKeys::Left],
        };
        let mut ctx = make_input_ctx(
            &mut timer,
            &mut resource,
            &config,
            &mut player_config,
            &mut transition,
            &input,
        );
        state.input(&mut ctx);
        assert_eq!(state.skin_type_index(), CONFIGURABLE_SKIN_TYPES.len() - 1);
    }

    #[test]
    fn cursor_wraps_down() {
        let mut state = SkinConfigState::new();
        let mut timer = TimerManager::new();
        let mut resource = PlayerResource::default();
        let config = Config::default();
        let mut player_config = player_config_with_options(vec![
            SkinOption {
                name: "opt1".to_string(),
                value: 0,
            },
            SkinOption {
                name: "opt2".to_string(),
                value: 0,
            },
        ]);
        let mut transition = None;

        // Press Down 3 times with 2 options => should wrap: 0->1->0->1
        for _ in 0..3 {
            let input = InputState {
                commands: vec![],
                pressed_keys: vec![ControlKeys::Down],
            };
            let mut ctx = make_input_ctx(
                &mut timer,
                &mut resource,
                &config,
                &mut player_config,
                &mut transition,
                &input,
            );
            state.input(&mut ctx);
        }
        assert_eq!(state.cursor(), 1);
    }

    #[test]
    fn cursor_wraps_up() {
        let mut state = SkinConfigState::new();
        let mut timer = TimerManager::new();
        let mut resource = PlayerResource::default();
        let config = Config::default();
        let mut player_config = player_config_with_options(vec![
            SkinOption {
                name: "opt1".to_string(),
                value: 0,
            },
            SkinOption {
                name: "opt2".to_string(),
                value: 0,
            },
            SkinOption {
                name: "opt3".to_string(),
                value: 0,
            },
        ]);
        let mut transition = None;

        // Press Up from cursor=0 should wrap to last (2).
        let input = InputState {
            commands: vec![],
            pressed_keys: vec![ControlKeys::Up],
        };
        let mut ctx = make_input_ctx(
            &mut timer,
            &mut resource,
            &config,
            &mut player_config,
            &mut transition,
            &input,
        );
        state.input(&mut ctx);
        assert_eq!(state.cursor(), 2);
    }

    #[test]
    fn escape_transitions_to_music_select() {
        let mut state = SkinConfigState::new();
        let mut timer = TimerManager::new();
        let mut resource = PlayerResource::default();
        let config = Config::default();
        let mut player_config = PlayerConfig::default();
        player_config.validate();
        let mut transition = None;

        let input = InputState {
            commands: vec![],
            pressed_keys: vec![ControlKeys::Escape],
        };
        let mut ctx = make_input_ctx(
            &mut timer,
            &mut resource,
            &config,
            &mut player_config,
            &mut transition,
            &input,
        );
        state.input(&mut ctx);

        assert_eq!(transition, Some(AppStateType::MusicSelect));
        assert!(resource.config_save_requested);
    }

    #[test]
    fn option_cycle_increments_value() {
        let mut state = SkinConfigState::new();
        let mut timer = TimerManager::new();
        let mut resource = PlayerResource::default();
        let config = Config::default();
        let mut player_config = player_config_with_options(vec![SkinOption {
            name: "volume".to_string(),
            value: 5,
        }]);
        let mut transition = None;

        // Press Enter to cycle forward (+1).
        let input = InputState {
            commands: vec![],
            pressed_keys: vec![ControlKeys::Enter],
        };
        let mut ctx = make_input_ctx(
            &mut timer,
            &mut resource,
            &config,
            &mut player_config,
            &mut transition,
            &input,
        );
        state.input(&mut ctx);

        let skin_id = SkinType::MusicSelect.id() as usize;
        let opt_value = player_config.skin[skin_id]
            .properties
            .as_ref()
            .unwrap()
            .option[0]
            .value;
        assert_eq!(opt_value, 6);
    }

    #[test]
    fn option_cycle_decrements_value() {
        let mut state = SkinConfigState::new();
        let mut timer = TimerManager::new();
        let mut resource = PlayerResource::default();
        let config = Config::default();
        let mut player_config = player_config_with_options(vec![SkinOption {
            name: "volume".to_string(),
            value: 5,
        }]);
        let mut transition = None;

        // Press Del to cycle backward (-1).
        let input = InputState {
            commands: vec![],
            pressed_keys: vec![ControlKeys::Del],
        };
        let mut ctx = make_input_ctx(
            &mut timer,
            &mut resource,
            &config,
            &mut player_config,
            &mut transition,
            &input,
        );
        state.input(&mut ctx);

        let skin_id = SkinType::MusicSelect.id() as usize;
        let opt_value = player_config.skin[skin_id]
            .properties
            .as_ref()
            .unwrap()
            .option[0]
            .value;
        assert_eq!(opt_value, 4);
    }

    #[test]
    fn create_resets_cursor() {
        let mut state = SkinConfigState::new();
        state.cursor = 5;

        let mut timer = TimerManager::new();
        let mut resource = PlayerResource::default();
        let config = Config::default();
        let mut player_config = PlayerConfig::default();
        player_config.validate();
        let mut transition = None;

        let mut ctx = make_ctx(
            &mut timer,
            &mut resource,
            &config,
            &mut player_config,
            &mut transition,
        );
        state.create(&mut ctx);
        assert_eq!(state.cursor(), 0);
    }

    #[test]
    fn skin_type_change_resets_cursor() {
        let mut state = SkinConfigState::new();
        let mut timer = TimerManager::new();
        let mut resource = PlayerResource::default();
        let config = Config::default();
        let mut player_config = player_config_with_options(vec![
            SkinOption {
                name: "a".to_string(),
                value: 0,
            },
            SkinOption {
                name: "b".to_string(),
                value: 0,
            },
        ]);
        let mut transition = None;

        // Move cursor down first.
        let input = InputState {
            commands: vec![],
            pressed_keys: vec![ControlKeys::Down],
        };
        let mut ctx = make_input_ctx(
            &mut timer,
            &mut resource,
            &config,
            &mut player_config,
            &mut transition,
            &input,
        );
        state.input(&mut ctx);
        assert_eq!(state.cursor(), 1);

        // Switch skin type — cursor should reset.
        let input = InputState {
            commands: vec![],
            pressed_keys: vec![ControlKeys::Right],
        };
        let mut ctx = make_input_ctx(
            &mut timer,
            &mut resource,
            &config,
            &mut player_config,
            &mut transition,
            &input,
        );
        state.input(&mut ctx);
        assert_eq!(state.cursor(), 0);
    }

    #[test]
    fn no_options_cursor_stays_zero() {
        let mut state = SkinConfigState::new();
        let mut timer = TimerManager::new();
        let mut resource = PlayerResource::default();
        let config = Config::default();
        // Default PlayerConfig has empty properties for skins.
        let mut player_config = PlayerConfig::default();
        player_config.validate();
        let mut transition = None;

        // Press Down with no options — cursor should stay 0.
        let input = InputState {
            commands: vec![],
            pressed_keys: vec![ControlKeys::Down],
        };
        let mut ctx = make_input_ctx(
            &mut timer,
            &mut resource,
            &config,
            &mut player_config,
            &mut transition,
            &input,
        );
        state.input(&mut ctx);
        assert_eq!(state.cursor(), 0);
    }
}
