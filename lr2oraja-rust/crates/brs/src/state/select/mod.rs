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

use bar_manager::{Bar, BarManager, SortMode};

/// Default input delay in milliseconds.
const DEFAULT_INPUT_DELAY_MS: i64 = 500;
/// Default fadeout duration in milliseconds.
const DEFAULT_FADEOUT_DURATION_MS: i64 = 500;

/// Music select state — song browser and selection.
pub struct MusicSelectState {
    bar_manager: BarManager,
    fadeout_started: bool,
    sort_mode: SortMode,
    mode_filter: Option<i32>,
    #[allow(dead_code)] // Reserved for text search UI integration
    search_mode: bool,
    #[allow(dead_code)] // Reserved for text search UI integration
    search_text: String,
}

impl MusicSelectState {
    pub fn new() -> Self {
        Self {
            bar_manager: BarManager::new(),
            fadeout_started: false,
            sort_mode: SortMode::Default,
            mode_filter: None,
            search_mode: false,
            search_text: String::new(),
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
                    ControlKeys::Num2 => {
                        // Cycle sort mode
                        self.sort_mode = self.sort_mode.next();
                        self.bar_manager.sort(self.sort_mode);
                        info!(sort = ?self.sort_mode, "MusicSelect: sort changed");
                        return;
                    }
                    ControlKeys::Num1 => {
                        // Cycle mode filter
                        self.mode_filter = match self.mode_filter {
                            None => Some(7),      // Beat7K
                            Some(7) => Some(14),  // Beat14K
                            Some(14) => Some(9),  // PopN9K
                            Some(9) => Some(5),   // Beat5K
                            Some(5) => Some(10),  // Beat10K
                            Some(10) => Some(25), // 24K
                            _ => None,            // All
                        };
                        if let Some(db) = ctx.database {
                            self.bar_manager.load_root(&db.song_db);
                            if let Some(mode_id) = self.mode_filter {
                                self.bar_manager.filter_by_mode(Some(mode_id));
                            }
                            self.bar_manager.sort(self.sort_mode);
                        }
                        info!(filter = ?self.mode_filter, "MusicSelect: mode filter changed");
                        return;
                    }
                    ControlKeys::Num3 => {
                        // Cycle gauge type
                        ctx.player_config.gauge = (ctx.player_config.gauge + 1) % 6;
                        info!(
                            gauge = ctx.player_config.gauge,
                            "MusicSelect: gauge changed"
                        );
                        return;
                    }
                    ControlKeys::Num4 => {
                        // Cycle random type
                        ctx.player_config.random = (ctx.player_config.random + 1) % 10;
                        info!(
                            random = ctx.player_config.random,
                            "MusicSelect: random changed"
                        );
                        return;
                    }
                    ControlKeys::Num5 => {
                        // Cycle DP option
                        ctx.player_config.doubleoption = (ctx.player_config.doubleoption + 1) % 4;
                        info!(
                            dp = ctx.player_config.doubleoption,
                            "MusicSelect: DP option changed"
                        );
                        return;
                    }
                    ControlKeys::Num6 => {
                        // Cycle hi-speed (placeholder for future integration)
                        return;
                    }
                    ControlKeys::Del => {
                        // Transition to KeyConfig
                        *ctx.transition = Some(AppStateType::KeyConfig);
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

    pub(crate) fn sort_mode(&self) -> SortMode {
        self.sort_mode
    }

    pub(crate) fn mode_filter(&self) -> Option<i32> {
        self.mode_filter
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input_mapper::InputState;
    use crate::player_resource::PlayerResource;
    use crate::timer_manager::TimerManager;
    use bms_config::{Config, PlayerConfig};
    use bms_model::BmsModel;

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

    /// Create a context with input enabled (TIMER_STARTINPUT on) and a key pressed.
    fn make_input_ctx<'a>(
        timer: &'a mut TimerManager,
        resource: &'a mut PlayerResource,
        config: &'a Config,
        player_config: &'a mut PlayerConfig,
        transition: &'a mut Option<AppStateType>,
        input_state: &'a InputState,
    ) -> StateContext<'a> {
        StateContext {
            timer,
            resource,
            config,
            player_config,
            transition,
            keyboard_backend: None,
            database: None,
            input_state: Some(input_state),
            skin_manager: None,
            sound_manager: None,
        }
    }

    fn setup_input_ready(timer: &mut TimerManager) {
        timer.set_now_micro_time(1_000_000);
        timer.switch_timer(TIMER_STARTINPUT, true);
    }

    #[test]
    fn create_with_bms_loaded_transitions_to_decide() {
        let mut state = MusicSelectState::new();
        let mut timer = TimerManager::new();
        let mut resource = PlayerResource::default();
        resource.bms_model = Some(BmsModel::default());
        let config = Config::default();
        let mut player_config = PlayerConfig::default();
        let mut transition = None;

        let mut ctx = make_ctx(
            &mut timer,
            &mut resource,
            &config,
            &mut player_config,
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
        let mut player_config = PlayerConfig::default();
        let mut transition = None;

        let mut ctx = make_ctx(
            &mut timer,
            &mut resource,
            &config,
            &mut player_config,
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
        let mut player_config = PlayerConfig::default();
        let mut transition = None;

        // Before delay
        timer.set_now_micro_time(400_000);
        let mut ctx = make_ctx(
            &mut timer,
            &mut resource,
            &config,
            &mut player_config,
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
            &mut player_config,
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
        let mut player_config = PlayerConfig::default();
        let mut transition = None;

        timer.set_now_micro_time(1_000_000);
        timer.set_timer_on(TIMER_FADEOUT);
        timer.set_now_micro_time(1_501_000);

        let mut ctx = make_ctx(
            &mut timer,
            &mut resource,
            &config,
            &mut player_config,
            &mut transition,
        );
        state.render(&mut ctx);
        assert_eq!(transition, Some(AppStateType::Decide));
    }

    #[test]
    fn sort_mode_cycles_on_num2() {
        let mut state = MusicSelectState::new();
        let mut timer = TimerManager::new();
        let mut resource = PlayerResource::default();
        let config = Config::default();
        let mut player_config = PlayerConfig::default();
        let mut transition = None;

        setup_input_ready(&mut timer);
        assert_eq!(state.sort_mode(), SortMode::Default);

        let input = InputState {
            commands: vec![],
            pressed_keys: vec![ControlKeys::Num2],
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
        assert_eq!(state.sort_mode(), SortMode::Title);

        state.input(&mut ctx);
        assert_eq!(state.sort_mode(), SortMode::Artist);

        state.input(&mut ctx);
        assert_eq!(state.sort_mode(), SortMode::Level);

        state.input(&mut ctx);
        assert_eq!(state.sort_mode(), SortMode::Default);
    }

    #[test]
    fn mode_filter_cycles_on_num1() {
        let mut state = MusicSelectState::new();
        let mut timer = TimerManager::new();
        let mut resource = PlayerResource::default();
        let config = Config::default();
        let mut player_config = PlayerConfig::default();
        let mut transition = None;

        setup_input_ready(&mut timer);
        assert_eq!(state.mode_filter(), None);

        let input = InputState {
            commands: vec![],
            pressed_keys: vec![ControlKeys::Num1],
        };
        // Without database, filter changes but no reload happens
        let mut ctx = make_input_ctx(
            &mut timer,
            &mut resource,
            &config,
            &mut player_config,
            &mut transition,
            &input,
        );
        state.input(&mut ctx);
        assert_eq!(state.mode_filter(), Some(7));

        state.input(&mut ctx);
        assert_eq!(state.mode_filter(), Some(14));

        state.input(&mut ctx);
        assert_eq!(state.mode_filter(), Some(9));

        state.input(&mut ctx);
        assert_eq!(state.mode_filter(), Some(5));

        state.input(&mut ctx);
        assert_eq!(state.mode_filter(), Some(10));

        state.input(&mut ctx);
        assert_eq!(state.mode_filter(), Some(25));

        state.input(&mut ctx);
        assert_eq!(state.mode_filter(), None);
    }

    #[test]
    fn gauge_cycles_on_num3() {
        let mut state = MusicSelectState::new();
        let mut timer = TimerManager::new();
        let mut resource = PlayerResource::default();
        let config = Config::default();
        let mut player_config = PlayerConfig::default();
        let mut transition = None;

        setup_input_ready(&mut timer);
        assert_eq!(player_config.gauge, 0);

        let input = InputState {
            commands: vec![],
            pressed_keys: vec![ControlKeys::Num3],
        };

        // Press Num3: gauge 0 -> 1
        {
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
        assert_eq!(player_config.gauge, 1);

        // Press Num3: gauge 1 -> 2
        {
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
        assert_eq!(player_config.gauge, 2);

        // Test wrap-around: 5 -> 0
        player_config.gauge = 5;
        {
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
        assert_eq!(player_config.gauge, 0);
    }

    #[test]
    fn random_cycles_on_num4() {
        let mut state = MusicSelectState::new();
        let mut timer = TimerManager::new();
        let mut resource = PlayerResource::default();
        let config = Config::default();
        let mut player_config = PlayerConfig::default();
        let mut transition = None;

        setup_input_ready(&mut timer);
        assert_eq!(player_config.random, 0);

        let input = InputState {
            commands: vec![],
            pressed_keys: vec![ControlKeys::Num4],
        };

        // Press Num4: random 0 -> 1
        {
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
        assert_eq!(player_config.random, 1);

        // Test wrap-around: 9 -> 0
        player_config.random = 9;
        {
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
        assert_eq!(player_config.random, 0);
    }

    #[test]
    fn dp_option_cycles_on_num5() {
        let mut state = MusicSelectState::new();
        let mut timer = TimerManager::new();
        let mut resource = PlayerResource::default();
        let config = Config::default();
        let mut player_config = PlayerConfig::default();
        let mut transition = None;

        setup_input_ready(&mut timer);
        assert_eq!(player_config.doubleoption, 0);

        let input = InputState {
            commands: vec![],
            pressed_keys: vec![ControlKeys::Num5],
        };

        // Press Num5: doubleoption 0 -> 1
        {
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
        assert_eq!(player_config.doubleoption, 1);

        // Test wrap-around: 3 -> 0
        player_config.doubleoption = 3;
        {
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
        assert_eq!(player_config.doubleoption, 0);
    }

    #[test]
    fn del_transitions_to_key_config() {
        let mut state = MusicSelectState::new();
        let mut timer = TimerManager::new();
        let mut resource = PlayerResource::default();
        let config = Config::default();
        let mut player_config = PlayerConfig::default();
        let mut transition = None;

        setup_input_ready(&mut timer);

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
        assert_eq!(transition, Some(AppStateType::KeyConfig));
    }
}
