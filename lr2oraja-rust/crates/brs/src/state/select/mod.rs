// MusicSelect state — song selection browser.
//
// Loads the song list from the database, allows cursor navigation,
// and transitions to Decide when a song is selected.

pub mod bar_manager;

use tracing::info;

use bms_input::control_keys::ControlKeys;
use bms_skin::property_id::{TIMER_FADEOUT, TIMER_STARTINPUT};

use crate::app_state::AppStateType;
use crate::state::{GameStateHandler, StateContext};

use bar_manager::{Bar, BarManager};

/// Default input delay in milliseconds.
const DEFAULT_INPUT_DELAY_MS: i64 = 500;
/// Default fadeout duration in milliseconds.
const DEFAULT_FADEOUT_DURATION_MS: i64 = 500;

/// Music select state — song browser and selection.
pub struct MusicSelectState {
    bar_manager: BarManager,
    fadeout_started: bool,
}

impl MusicSelectState {
    pub fn new() -> Self {
        Self {
            bar_manager: BarManager::new(),
            fadeout_started: false,
        }
    }
}

impl Default for MusicSelectState {
    fn default() -> Self {
        Self::new()
    }
}

impl GameStateHandler for MusicSelectState {
    fn create(&mut self, ctx: &mut StateContext) {
        self.fadeout_started = false;
        info!("MusicSelect: create");

        // If a BMS model is already loaded (via CLI --bms), skip to Decide immediately
        if ctx.resource.bms_model.is_some() {
            info!("MusicSelect: BMS already loaded, transitioning to Decide");
            *ctx.transition = Some(AppStateType::Decide);
            return;
        }

        // Load song list from database
        if let Some(db) = ctx.database {
            self.bar_manager.load_root(&db.song_db);
            info!(
                songs = self.bar_manager.bar_count(),
                "MusicSelect: loaded song list"
            );
        }
    }

    fn render(&mut self, ctx: &mut StateContext) {
        let now = ctx.timer.now_time();

        // Enable input after initial delay
        if now > DEFAULT_INPUT_DELAY_MS {
            ctx.timer.switch_timer(TIMER_STARTINPUT, true);
        }

        // Check fadeout -> transition
        if ctx.timer.is_timer_on(TIMER_FADEOUT)
            && ctx.timer.now_time_of(TIMER_FADEOUT) > DEFAULT_FADEOUT_DURATION_MS
        {
            info!("MusicSelect: transition to Decide");
            *ctx.transition = Some(AppStateType::Decide);
        }
    }

    fn input(&mut self, ctx: &mut StateContext) {
        if ctx.timer.is_timer_on(TIMER_FADEOUT) || !ctx.timer.is_timer_on(TIMER_STARTINPUT) {
            return;
        }

        if let Some(input_state) = ctx.input_state {
            for key in &input_state.pressed_keys {
                match key {
                    ControlKeys::Up => {
                        self.bar_manager.move_cursor(-1);
                        return;
                    }
                    ControlKeys::Down => {
                        self.bar_manager.move_cursor(1);
                        return;
                    }
                    ControlKeys::Enter => {
                        self.select_current(ctx);
                        return;
                    }
                    ControlKeys::Escape => {
                        if self.bar_manager.is_in_folder() {
                            self.bar_manager.leave_folder();
                        }
                        return;
                    }
                    ControlKeys::Insert => {
                        *ctx.transition = Some(AppStateType::SkinConfig);
                        return;
                    }
                    _ => {}
                }
            }
        }
    }

    fn shutdown(&mut self, _ctx: &mut StateContext) {
        info!("MusicSelect: shutdown");
    }
}

impl MusicSelectState {
    fn select_current(&mut self, ctx: &mut StateContext) {
        match self.bar_manager.current() {
            Some(Bar::Song(song_data)) => {
                // Load BMS file
                let path = std::path::PathBuf::from(&song_data.path);
                match bms_model::BmsDecoder::decode(&path) {
                    Ok(model) => {
                        ctx.resource.play_mode = model.mode;
                        ctx.resource.bms_dir = path.parent().map(|p| p.to_path_buf());
                        ctx.resource.bms_model = Some(model);
                        // Start fadeout -> Decide
                        self.fadeout_started = true;
                        ctx.timer.set_timer_on(TIMER_FADEOUT);
                    }
                    Err(e) => {
                        tracing::warn!(path = %path.display(), "MusicSelect: failed to load BMS: {e}");
                    }
                }
            }
            Some(Bar::Folder { .. }) => {
                if let Some(db) = ctx.database {
                    self.bar_manager.enter_folder(&db.song_db);
                }
            }
            None => {}
        }
    }
}

#[cfg(test)]
impl MusicSelectState {
    pub(crate) fn bar_manager(&self) -> &BarManager {
        &self.bar_manager
    }

    pub(crate) fn is_fadeout_started(&self) -> bool {
        self.fadeout_started
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::player_resource::PlayerResource;
    use crate::timer_manager::TimerManager;
    use bms_config::{Config, PlayerConfig};
    use bms_model::BmsModel;

    fn make_ctx<'a>(
        timer: &'a mut TimerManager,
        resource: &'a mut PlayerResource,
        config: &'a Config,
        player_config: &'a PlayerConfig,
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
        }
    }

    #[test]
    fn create_with_bms_loaded_transitions_to_decide() {
        let mut state = MusicSelectState::new();
        let mut timer = TimerManager::new();
        let mut resource = PlayerResource::default();
        resource.bms_model = Some(BmsModel::default());
        let config = Config::default();
        let player_config = PlayerConfig::default();
        let mut transition = None;

        let mut ctx = make_ctx(
            &mut timer,
            &mut resource,
            &config,
            &player_config,
            &mut transition,
        );
        state.create(&mut ctx);
        assert_eq!(transition, Some(AppStateType::Decide));
    }

    #[test]
    fn create_without_bms_stays() {
        let mut state = MusicSelectState::new();
        let mut timer = TimerManager::new();
        let mut resource = PlayerResource::default();
        let config = Config::default();
        let player_config = PlayerConfig::default();
        let mut transition = None;

        let mut ctx = make_ctx(
            &mut timer,
            &mut resource,
            &config,
            &player_config,
            &mut transition,
        );
        state.create(&mut ctx);
        assert_eq!(transition, None);
    }

    #[test]
    fn render_enables_input_after_delay() {
        let mut state = MusicSelectState::new();
        let mut timer = TimerManager::new();
        let mut resource = PlayerResource::default();
        let config = Config::default();
        let player_config = PlayerConfig::default();
        let mut transition = None;

        // Before delay
        timer.set_now_micro_time(400_000);
        let mut ctx = make_ctx(
            &mut timer,
            &mut resource,
            &config,
            &player_config,
            &mut transition,
        );
        state.render(&mut ctx);
        assert!(!timer.is_timer_on(TIMER_STARTINPUT));

        // After delay
        timer.set_now_micro_time(501_000);
        let mut ctx = make_ctx(
            &mut timer,
            &mut resource,
            &config,
            &player_config,
            &mut transition,
        );
        state.render(&mut ctx);
        assert!(timer.is_timer_on(TIMER_STARTINPUT));
    }

    #[test]
    fn render_fadeout_transitions_to_decide() {
        let mut state = MusicSelectState::new();
        let mut timer = TimerManager::new();
        let mut resource = PlayerResource::default();
        let config = Config::default();
        let player_config = PlayerConfig::default();
        let mut transition = None;

        timer.set_now_micro_time(1_000_000);
        timer.set_timer_on(TIMER_FADEOUT);
        timer.set_now_micro_time(1_501_000);

        let mut ctx = make_ctx(
            &mut timer,
            &mut resource,
            &config,
            &player_config,
            &mut transition,
        );
        state.render(&mut ctx);
        assert_eq!(transition, Some(AppStateType::Decide));
    }
}
