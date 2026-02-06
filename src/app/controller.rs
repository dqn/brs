use anyhow::Result;

use crate::state::game_state::StateTransition;

/// Application state type in the main state machine.
///
/// Corresponds to beatoraja's MainStateType.
/// Defines the screens the application can be in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppStateType {
    /// Music selection screen.
    MusicSelect,
    /// Decide screen (loading/preview before play).
    Decide,
    /// Play screen.
    Play,
    /// Music result screen.
    Result,
    /// Course result screen.
    CourseResult,
    /// Key/skin configuration screen.
    Config,
}

/// Transition request from a state to the controller.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppTransition {
    /// No transition; stay in the current state.
    None,
    /// Transition to a specific state.
    ChangeTo(AppStateType),
    /// Exit the application.
    Exit,
}

/// Main application controller managing the state machine.
///
/// Corresponds to beatoraja's MainController.
/// Manages the SELECT -> DECIDE -> PLAY -> RESULT -> SELECT loop,
/// with optional course mode and config screen.
pub struct AppController {
    /// Current state type.
    current_state: AppStateType,
    /// Whether a course is in progress.
    course_active: bool,
    /// Whether the app should exit.
    should_exit: bool,
}

impl AppController {
    /// Create a new app controller starting at the select screen.
    pub fn new() -> Self {
        Self {
            current_state: AppStateType::MusicSelect,
            course_active: false,
            should_exit: false,
        }
    }

    /// Get the current state type.
    pub fn current_state(&self) -> AppStateType {
        self.current_state
    }

    /// Whether a course is active.
    pub fn is_course_active(&self) -> bool {
        self.course_active
    }

    /// Set course mode active/inactive.
    pub fn set_course_active(&mut self, active: bool) {
        self.course_active = active;
    }

    /// Whether the app should exit.
    pub fn should_exit(&self) -> bool {
        self.should_exit
    }

    /// Request the app to exit.
    pub fn request_exit(&mut self) {
        self.should_exit = true;
    }

    /// Resolve a state transition from the current state.
    ///
    /// Given a `StateTransition` from the currently active `GameState`,
    /// determines what `AppTransition` should be performed.
    pub fn resolve_transition(&self, transition: StateTransition) -> AppTransition {
        match transition {
            StateTransition::None => AppTransition::None,
            StateTransition::Next => self.resolve_next(),
            StateTransition::Back => self.resolve_back(),
        }
    }

    /// Apply a transition, changing the current state.
    pub fn apply_transition(&mut self, transition: &AppTransition) {
        match transition {
            AppTransition::None => {}
            AppTransition::ChangeTo(new_state) => {
                self.current_state = *new_state;
            }
            AppTransition::Exit => {
                self.should_exit = true;
            }
        }
    }

    /// Process a state transition in one step (resolve + apply).
    pub fn process_transition(&mut self, transition: StateTransition) -> Result<AppTransition> {
        let app_transition = self.resolve_transition(transition);
        self.apply_transition(&app_transition);
        Ok(app_transition)
    }

    /// Resolve "Next" transition based on current state.
    fn resolve_next(&self) -> AppTransition {
        match self.current_state {
            AppStateType::MusicSelect => AppTransition::ChangeTo(AppStateType::Decide),
            AppStateType::Decide => AppTransition::ChangeTo(AppStateType::Play),
            AppStateType::Play => {
                if self.course_active {
                    // In course mode, result decides whether to continue or show course result
                    AppTransition::ChangeTo(AppStateType::Result)
                } else {
                    AppTransition::ChangeTo(AppStateType::Result)
                }
            }
            AppStateType::Result => {
                if self.course_active {
                    // Course mode: go to course result or next song
                    // (CoursePlayer determines this externally)
                    AppTransition::ChangeTo(AppStateType::CourseResult)
                } else {
                    AppTransition::ChangeTo(AppStateType::MusicSelect)
                }
            }
            AppStateType::CourseResult => AppTransition::ChangeTo(AppStateType::MusicSelect),
            AppStateType::Config => AppTransition::ChangeTo(AppStateType::MusicSelect),
        }
    }

    /// Resolve "Back" transition based on current state.
    fn resolve_back(&self) -> AppTransition {
        match self.current_state {
            AppStateType::MusicSelect => AppTransition::Exit,
            AppStateType::Decide => AppTransition::ChangeTo(AppStateType::MusicSelect),
            AppStateType::Play => AppTransition::ChangeTo(AppStateType::Result),
            AppStateType::Result => AppTransition::ChangeTo(AppStateType::MusicSelect),
            AppStateType::CourseResult => AppTransition::ChangeTo(AppStateType::MusicSelect),
            AppStateType::Config => AppTransition::ChangeTo(AppStateType::MusicSelect),
        }
    }
}

impl Default for AppController {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_state_is_select() {
        let ctrl = AppController::new();
        assert_eq!(ctrl.current_state(), AppStateType::MusicSelect);
        assert!(!ctrl.should_exit());
        assert!(!ctrl.is_course_active());
    }

    // =======================================================================
    // Normal flow: SELECT -> DECIDE -> PLAY -> RESULT -> SELECT
    // =======================================================================

    #[test]
    fn normal_flow_select_to_decide() {
        let mut ctrl = AppController::new();
        let t = ctrl.process_transition(StateTransition::Next).unwrap();
        assert_eq!(t, AppTransition::ChangeTo(AppStateType::Decide));
        assert_eq!(ctrl.current_state(), AppStateType::Decide);
    }

    #[test]
    fn normal_flow_decide_to_play() {
        let mut ctrl = AppController::new();
        ctrl.apply_transition(&AppTransition::ChangeTo(AppStateType::Decide));

        let t = ctrl.process_transition(StateTransition::Next).unwrap();
        assert_eq!(t, AppTransition::ChangeTo(AppStateType::Play));
        assert_eq!(ctrl.current_state(), AppStateType::Play);
    }

    #[test]
    fn normal_flow_play_to_result() {
        let mut ctrl = AppController::new();
        ctrl.apply_transition(&AppTransition::ChangeTo(AppStateType::Play));

        let t = ctrl.process_transition(StateTransition::Next).unwrap();
        assert_eq!(t, AppTransition::ChangeTo(AppStateType::Result));
        assert_eq!(ctrl.current_state(), AppStateType::Result);
    }

    #[test]
    fn normal_flow_result_to_select() {
        let mut ctrl = AppController::new();
        ctrl.apply_transition(&AppTransition::ChangeTo(AppStateType::Result));

        let t = ctrl.process_transition(StateTransition::Next).unwrap();
        assert_eq!(t, AppTransition::ChangeTo(AppStateType::MusicSelect));
        assert_eq!(ctrl.current_state(), AppStateType::MusicSelect);
    }

    #[test]
    fn full_normal_loop() {
        let mut ctrl = AppController::new();

        // SELECT -> DECIDE
        ctrl.process_transition(StateTransition::Next).unwrap();
        assert_eq!(ctrl.current_state(), AppStateType::Decide);

        // DECIDE -> PLAY
        ctrl.process_transition(StateTransition::Next).unwrap();
        assert_eq!(ctrl.current_state(), AppStateType::Play);

        // PLAY -> RESULT
        ctrl.process_transition(StateTransition::Next).unwrap();
        assert_eq!(ctrl.current_state(), AppStateType::Result);

        // RESULT -> SELECT
        ctrl.process_transition(StateTransition::Next).unwrap();
        assert_eq!(ctrl.current_state(), AppStateType::MusicSelect);
    }

    // =======================================================================
    // Back transitions
    // =======================================================================

    #[test]
    fn back_from_select_exits() {
        let mut ctrl = AppController::new();
        let t = ctrl.process_transition(StateTransition::Back).unwrap();
        assert_eq!(t, AppTransition::Exit);
        assert!(ctrl.should_exit());
    }

    #[test]
    fn back_from_decide_returns_to_select() {
        let mut ctrl = AppController::new();
        ctrl.apply_transition(&AppTransition::ChangeTo(AppStateType::Decide));

        let t = ctrl.process_transition(StateTransition::Back).unwrap();
        assert_eq!(t, AppTransition::ChangeTo(AppStateType::MusicSelect));
    }

    #[test]
    fn back_from_play_goes_to_result() {
        let mut ctrl = AppController::new();
        ctrl.apply_transition(&AppTransition::ChangeTo(AppStateType::Play));

        let t = ctrl.process_transition(StateTransition::Back).unwrap();
        assert_eq!(t, AppTransition::ChangeTo(AppStateType::Result));
    }

    #[test]
    fn back_from_config_returns_to_select() {
        let mut ctrl = AppController::new();
        ctrl.apply_transition(&AppTransition::ChangeTo(AppStateType::Config));

        let t = ctrl.process_transition(StateTransition::Back).unwrap();
        assert_eq!(t, AppTransition::ChangeTo(AppStateType::MusicSelect));
    }

    // =======================================================================
    // Course flow
    // =======================================================================

    #[test]
    fn course_flow_result_to_course_result() {
        let mut ctrl = AppController::new();
        ctrl.set_course_active(true);
        ctrl.apply_transition(&AppTransition::ChangeTo(AppStateType::Result));

        let t = ctrl.process_transition(StateTransition::Next).unwrap();
        assert_eq!(t, AppTransition::ChangeTo(AppStateType::CourseResult));
    }

    #[test]
    fn course_result_returns_to_select() {
        let mut ctrl = AppController::new();
        ctrl.set_course_active(true);
        ctrl.apply_transition(&AppTransition::ChangeTo(AppStateType::CourseResult));

        let t = ctrl.process_transition(StateTransition::Next).unwrap();
        assert_eq!(t, AppTransition::ChangeTo(AppStateType::MusicSelect));
    }

    // =======================================================================
    // None transition
    // =======================================================================

    #[test]
    fn none_transition_stays() {
        let mut ctrl = AppController::new();
        let t = ctrl.process_transition(StateTransition::None).unwrap();
        assert_eq!(t, AppTransition::None);
        assert_eq!(ctrl.current_state(), AppStateType::MusicSelect);
    }

    // =======================================================================
    // Exit
    // =======================================================================

    #[test]
    fn request_exit() {
        let mut ctrl = AppController::new();
        assert!(!ctrl.should_exit());
        ctrl.request_exit();
        assert!(ctrl.should_exit());
    }
}
