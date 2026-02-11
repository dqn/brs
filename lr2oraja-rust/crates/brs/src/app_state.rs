// AppStateType and StateRegistry — state machine for game states.
//
// Manages state transitions, calling shutdown/create/prepare on handlers.

use std::collections::HashMap;

use tracing::info;

use crate::player_resource::PlayerResource;
use crate::state::{GameStateHandler, StateContext};
use crate::timer_manager::TimerManager;
use bms_config::{Config, PlayerConfig};

/// Identifies which game state is active.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AppStateType {
    MusicSelect,
    Decide,
    Play,
    Result,
    CourseResult,
    KeyConfig,
    SkinConfig,
}

impl std::fmt::Display for AppStateType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MusicSelect => write!(f, "MusicSelect"),
            Self::Decide => write!(f, "Decide"),
            Self::Play => write!(f, "Play"),
            Self::Result => write!(f, "Result"),
            Self::CourseResult => write!(f, "CourseResult"),
            Self::KeyConfig => write!(f, "KeyConfig"),
            Self::SkinConfig => write!(f, "SkinConfig"),
        }
    }
}

/// Registry of all state handlers with transition logic.
pub struct StateRegistry {
    current: AppStateType,
    handlers: HashMap<AppStateType, Box<dyn GameStateHandler>>,
    initialized: bool,
}

impl StateRegistry {
    /// Creates a new registry with the given initial state.
    /// Handlers must be registered via `register` before `tick`.
    pub fn new(initial: AppStateType) -> Self {
        Self {
            current: initial,
            handlers: HashMap::new(),
            initialized: false,
        }
    }

    /// Registers a handler for a state type.
    pub fn register(&mut self, state_type: AppStateType, handler: Box<dyn GameStateHandler>) {
        self.handlers.insert(state_type, handler);
    }

    /// Returns the current active state type.
    #[allow(dead_code)]
    pub fn current(&self) -> AppStateType {
        self.current
    }

    /// Runs one frame: initializes if needed, processes render + input,
    /// then handles any pending transition.
    pub fn tick(
        &mut self,
        timer: &mut TimerManager,
        resource: &mut PlayerResource,
        config: &Config,
        player_config: &PlayerConfig,
        keyboard_backend: Option<&dyn bms_input::keyboard::KeyboardBackend>,
    ) {
        let mut transition: Option<AppStateType> = None;

        // First-time initialization
        if !self.initialized {
            self.initialized = true;
            info!(state = %self.current, "Initializing state");
            timer.reset();
            if let Some(handler) = self.handlers.get_mut(&self.current) {
                let mut ctx = StateContext {
                    timer,
                    resource,
                    config,
                    player_config,
                    transition: &mut transition,
                    keyboard_backend,
                };
                handler.create(&mut ctx);
                handler.prepare(&mut ctx);
            }
        }

        // Run current state's render and input
        if let Some(handler) = self.handlers.get_mut(&self.current) {
            let mut ctx = StateContext {
                timer,
                resource,
                config,
                player_config,
                transition: &mut transition,
                keyboard_backend,
            };
            handler.render(&mut ctx);
            handler.input(&mut ctx);
        }

        // Handle pending transition
        if let Some(next) = transition {
            self.change_state(
                next,
                timer,
                resource,
                config,
                player_config,
                keyboard_backend,
            );
        }
    }

    /// Performs a state transition: shutdown current -> reset timer -> create+prepare next.
    fn change_state(
        &mut self,
        next: AppStateType,
        timer: &mut TimerManager,
        resource: &mut PlayerResource,
        config: &Config,
        player_config: &PlayerConfig,
        keyboard_backend: Option<&dyn bms_input::keyboard::KeyboardBackend>,
    ) {
        info!(from = %self.current, to = %next, "State transition");

        let mut dummy_transition: Option<AppStateType> = None;

        // Shutdown current state
        if let Some(handler) = self.handlers.get_mut(&self.current) {
            let mut ctx = StateContext {
                timer,
                resource,
                config,
                player_config,
                transition: &mut dummy_transition,
                keyboard_backend,
            };
            handler.shutdown(&mut ctx);
        }

        // Reset timer for new state
        timer.reset();

        self.current = next;

        // Create and prepare new state
        if let Some(handler) = self.handlers.get_mut(&self.current) {
            let mut ctx = StateContext {
                timer,
                resource,
                config,
                player_config,
                transition: &mut dummy_transition,
                keyboard_backend,
            };
            handler.create(&mut ctx);
            handler.prepare(&mut ctx);
        }

        // If the new state's create requested another transition, handle it
        // (e.g., MusicSelect immediately transitions to Decide)
        if let Some(chained) = dummy_transition {
            self.change_state(
                chained,
                timer,
                resource,
                config,
                player_config,
                keyboard_backend,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::GameStateHandler;
    use std::sync::{Arc, Mutex};

    /// Test helper that records lifecycle calls.
    #[derive(Default)]
    struct RecordingHandler {
        log: Arc<Mutex<Vec<String>>>,
        transition_on_create: Option<AppStateType>,
    }

    impl GameStateHandler for RecordingHandler {
        fn create(&mut self, ctx: &mut StateContext) {
            self.log.lock().unwrap().push("create".to_string());
            if let Some(next) = self.transition_on_create {
                *ctx.transition = Some(next);
            }
        }
        fn prepare(&mut self, _ctx: &mut StateContext) {
            self.log.lock().unwrap().push("prepare".to_string());
        }
        fn render(&mut self, _ctx: &mut StateContext) {
            self.log.lock().unwrap().push("render".to_string());
        }
        fn input(&mut self, _ctx: &mut StateContext) {
            self.log.lock().unwrap().push("input".to_string());
        }
        fn shutdown(&mut self, _ctx: &mut StateContext) {
            self.log.lock().unwrap().push("shutdown".to_string());
        }
    }

    fn make_deps() -> (TimerManager, PlayerResource, Config, PlayerConfig) {
        (
            TimerManager::new(),
            PlayerResource::default(),
            Config::default(),
            PlayerConfig::default(),
        )
    }

    #[test]
    fn initial_state_is_set() {
        let reg = StateRegistry::new(AppStateType::MusicSelect);
        assert_eq!(reg.current(), AppStateType::MusicSelect);
    }

    #[test]
    fn tick_calls_create_prepare_render_input() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let handler = RecordingHandler {
            log: log.clone(),
            ..Default::default()
        };

        let mut reg = StateRegistry::new(AppStateType::MusicSelect);
        reg.register(AppStateType::MusicSelect, Box::new(handler));

        let (mut timer, mut resource, config, player_config) = make_deps();
        reg.tick(&mut timer, &mut resource, &config, &player_config, None);

        let calls = log.lock().unwrap();
        assert_eq!(*calls, vec!["create", "prepare", "render", "input"]);
    }

    #[test]
    fn transition_calls_shutdown_and_create() {
        let select_log = Arc::new(Mutex::new(Vec::new()));
        let decide_log = Arc::new(Mutex::new(Vec::new()));

        let select_handler = RecordingHandler {
            log: select_log.clone(),
            ..Default::default()
        };
        let decide_handler = RecordingHandler {
            log: decide_log.clone(),
            ..Default::default()
        };

        let mut reg = StateRegistry::new(AppStateType::MusicSelect);
        reg.register(AppStateType::MusicSelect, Box::new(select_handler));
        reg.register(AppStateType::Decide, Box::new(decide_handler));

        let (mut timer, mut resource, config, player_config) = make_deps();

        // Initialize
        reg.tick(&mut timer, &mut resource, &config, &player_config, None);

        // Clear logs
        select_log.lock().unwrap().clear();
        decide_log.lock().unwrap().clear();

        // Manually trigger transition
        reg.change_state(
            AppStateType::Decide,
            &mut timer,
            &mut resource,
            &config,
            &player_config,
            None,
        );

        assert!(select_log.lock().unwrap().contains(&"shutdown".to_string()));
        let decide_calls = decide_log.lock().unwrap();
        assert!(decide_calls.contains(&"create".to_string()));
        assert!(decide_calls.contains(&"prepare".to_string()));
        assert_eq!(reg.current(), AppStateType::Decide);
    }

    #[test]
    fn timer_resets_on_transition() {
        let mut reg = StateRegistry::new(AppStateType::MusicSelect);

        let handler1: Box<dyn GameStateHandler> = Box::new(RecordingHandler::default());
        let handler2: Box<dyn GameStateHandler> = Box::new(RecordingHandler::default());
        reg.register(AppStateType::MusicSelect, handler1);
        reg.register(AppStateType::Decide, handler2);

        let (mut timer, mut resource, config, player_config) = make_deps();

        // Set a timer in MusicSelect
        timer.set_now_micro_time(5000);
        timer.set_timer_on(bms_skin::property_id::TIMER_STARTINPUT);
        assert!(timer.is_timer_on(bms_skin::property_id::TIMER_STARTINPUT));

        // Transition should reset all timers
        reg.change_state(
            AppStateType::Decide,
            &mut timer,
            &mut resource,
            &config,
            &player_config,
            None,
        );
        assert!(!timer.is_timer_on(bms_skin::property_id::TIMER_STARTINPUT));
    }

    #[test]
    fn chained_transition_in_create() {
        // MusicSelect's create immediately transitions to Decide
        let select_handler = RecordingHandler {
            transition_on_create: Some(AppStateType::Decide),
            ..Default::default()
        };
        let decide_handler = RecordingHandler::default();

        let mut reg = StateRegistry::new(AppStateType::MusicSelect);
        reg.register(AppStateType::MusicSelect, Box::new(select_handler));
        reg.register(AppStateType::Decide, Box::new(decide_handler));

        let (mut timer, mut resource, config, player_config) = make_deps();

        // First tick should initialize MusicSelect, which chains to Decide
        reg.tick(&mut timer, &mut resource, &config, &player_config, None);
        assert_eq!(reg.current(), AppStateType::Decide);
    }
}
