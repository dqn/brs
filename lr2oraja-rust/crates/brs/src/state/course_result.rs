// CourseResult state — displays aggregated course results.
//
// Shows combined score across all course stages, determines overall clear type
// (worst of individual clears), and optionally saves to the database.

use tracing::info;

use bms_input::control_keys::ControlKeys;
use bms_skin::property_id::{TIMER_FADEOUT, TIMER_STARTINPUT};

use crate::app_state::AppStateType;
use crate::state::{GameStateHandler, StateContext};

/// Default input delay in milliseconds.
const DEFAULT_INPUT_DELAY_MS: i64 = 500;
/// Default scene duration in milliseconds.
const DEFAULT_SCENE_DURATION_MS: i64 = 5000;
/// Default fadeout duration in milliseconds.
const DEFAULT_FADEOUT_DURATION_MS: i64 = 500;

/// Course result state — aggregates and displays results for a course play session.
pub struct CourseResultState {
    fadeout_started: bool,
}

impl CourseResultState {
    pub fn new() -> Self {
        Self {
            fadeout_started: false,
        }
    }

    fn start_fadeout(&mut self, ctx: &mut StateContext) {
        if !ctx.timer.is_timer_on(TIMER_FADEOUT) && ctx.timer.is_timer_on(TIMER_STARTINPUT) {
            self.fadeout_started = true;
            ctx.timer.set_timer_on(TIMER_FADEOUT);
        }
    }
}

impl Default for CourseResultState {
    fn default() -> Self {
        Self::new()
    }
}

impl GameStateHandler for CourseResultState {
    fn create(&mut self, ctx: &mut StateContext) {
        self.fadeout_started = false;
        info!("CourseResult: create");

        // Aggregate course scores if available
        if let Some(course_scores) = &ctx.resource.course_score_data
            && !course_scores.is_empty()
        {
            let mut aggregated = bms_rule::ScoreData::default();
            let mut worst_clear = bms_rule::ClearType::Max;

            for score in course_scores {
                aggregated.epg += score.epg;
                aggregated.lpg += score.lpg;
                aggregated.egr += score.egr;
                aggregated.lgr += score.lgr;
                aggregated.egd += score.egd;
                aggregated.lgd += score.lgd;
                aggregated.ebd += score.ebd;
                aggregated.lbd += score.lbd;
                aggregated.epr += score.epr;
                aggregated.lpr += score.lpr;
                aggregated.ems += score.ems;
                aggregated.lms += score.lms;
                aggregated.maxcombo += score.maxcombo;
                aggregated.notes += score.notes;

                if score.clear < worst_clear {
                    worst_clear = score.clear;
                }
            }

            aggregated.clear = worst_clear;
            aggregated.minbp = aggregated.ebd
                + aggregated.lbd
                + aggregated.epr
                + aggregated.lpr
                + aggregated.ems
                + aggregated.lms;

            info!(
                exscore = aggregated.exscore(),
                clear = ?aggregated.clear,
                stages = course_scores.len(),
                "CourseResult: aggregated scores"
            );

            ctx.resource.score_data = aggregated;
        }
    }

    fn render(&mut self, ctx: &mut StateContext) {
        let now = ctx.timer.now_time();

        // Enable input after initial delay
        if now > DEFAULT_INPUT_DELAY_MS {
            ctx.timer.switch_timer(TIMER_STARTINPUT, true);
        }

        // Check fadeout -> transition
        if ctx.timer.is_timer_on(TIMER_FADEOUT) {
            if ctx.timer.now_time_of(TIMER_FADEOUT) > DEFAULT_FADEOUT_DURATION_MS {
                info!("CourseResult: transition to MusicSelect");
                *ctx.transition = Some(AppStateType::MusicSelect);
            }
        } else if now > DEFAULT_SCENE_DURATION_MS {
            info!("CourseResult: scene timer expired, starting fadeout");
            self.fadeout_started = true;
            ctx.timer.set_timer_on(TIMER_FADEOUT);
        }
    }

    fn input(&mut self, ctx: &mut StateContext) {
        if ctx.timer.is_timer_on(TIMER_FADEOUT) || !ctx.timer.is_timer_on(TIMER_STARTINPUT) {
            return;
        }

        if let Some(input_state) = ctx.input_state {
            for key in &input_state.pressed_keys {
                match key {
                    ControlKeys::Enter | ControlKeys::Escape => {
                        self.start_fadeout(ctx);
                        return;
                    }
                    _ => {}
                }
            }
        }
    }

    fn shutdown(&mut self, _ctx: &mut StateContext) {
        info!("CourseResult: shutdown");
    }
}

#[cfg(test)]
impl CourseResultState {
    pub(crate) fn is_fadeout_started(&self) -> bool {
        self.fadeout_started
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input_mapper::InputState;
    use crate::player_resource::PlayerResource;
    use crate::timer_manager::TimerManager;
    use bms_config::{Config, PlayerConfig};
    use bms_rule::ClearType;
    use bms_rule::ScoreData;

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

    fn make_score(epg: i32, lpg: i32, egr: i32, lgr: i32, clear: ClearType) -> ScoreData {
        let mut sd = ScoreData::default();
        sd.epg = epg;
        sd.lpg = lpg;
        sd.egr = egr;
        sd.lgr = lgr;
        sd.clear = clear;
        sd
    }

    #[test]
    fn create_aggregates_course_scores() {
        let mut state = CourseResultState::new();
        let mut timer = TimerManager::new();
        let mut resource = PlayerResource::default();
        resource.course_score_data = Some(vec![
            make_score(100, 50, 30, 20, ClearType::Hard),
            make_score(80, 40, 20, 10, ClearType::Normal),
        ]);

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

        // Aggregated: epg=180, lpg=90, egr=50, lgr=30
        assert_eq!(resource.score_data.epg, 180);
        assert_eq!(resource.score_data.lpg, 90);
        assert_eq!(resource.score_data.egr, 50);
        assert_eq!(resource.score_data.lgr, 30);
        // Worst clear: Normal < Hard
        assert_eq!(resource.score_data.clear, ClearType::Normal);
    }

    #[test]
    fn render_enables_input_after_delay() {
        let mut state = CourseResultState::new();
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
    fn render_fadeout_transitions_to_music_select() {
        let mut state = CourseResultState::new();
        let mut timer = TimerManager::new();
        let mut resource = PlayerResource::default();
        let config = Config::default();
        let player_config = PlayerConfig::default();
        let mut transition = None;

        // Set up: FADEOUT timer on at time 1000ms
        timer.set_now_micro_time(1_000_000);
        timer.set_timer_on(TIMER_FADEOUT);

        // Advance past fadeout duration
        timer.set_now_micro_time(1_501_000);
        let mut ctx = make_ctx(
            &mut timer,
            &mut resource,
            &config,
            &player_config,
            &mut transition,
        );
        state.render(&mut ctx);
        assert_eq!(transition, Some(AppStateType::MusicSelect));
    }

    #[test]
    fn input_confirm_starts_fadeout() {
        let mut state = CourseResultState::new();
        let mut timer = TimerManager::new();
        let mut resource = PlayerResource::default();
        let config = Config::default();
        let player_config = PlayerConfig::default();
        let mut transition = None;

        // Enable input
        timer.set_now_micro_time(600_000);
        timer.switch_timer(TIMER_STARTINPUT, true);

        let input_state = InputState {
            commands: vec![],
            pressed_keys: vec![ControlKeys::Enter],
        };

        let mut ctx = StateContext {
            timer: &mut timer,
            resource: &mut resource,
            config: &config,
            player_config: &player_config,
            transition: &mut transition,
            keyboard_backend: None,
            database: None,
            input_state: Some(&input_state),
        };
        state.input(&mut ctx);
        assert!(timer.is_timer_on(TIMER_FADEOUT));
        assert!(state.is_fadeout_started());
    }
}
