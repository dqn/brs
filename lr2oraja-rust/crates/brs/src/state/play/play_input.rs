// Play input — key input processing, replay injection, and key beam timers.

use bms_model::{LaneProperty, Note};
use bms_rule::JUDGE_BD;
use bms_rule::judge_manager::JudgeEvent;
use bms_skin::property_id::{
    TIMER_COMBO_1P, TIMER_COMBO_2P, TIMER_JUDGE_1P, TIMER_JUDGE_2P, TIMER_PLAY,
};
use bms_skin::property_mapper;

use crate::state::StateContext;

use super::{NOT_SET, PlayPhase, PlayState};

impl PlayState {
    /// Inject replay events into key state up to the current time.
    pub(super) fn inject_replay_events(&mut self, ptime_us: i64) {
        let phys_count = self.key_states.len();
        while self.replay_cursor < self.replay_log.len() {
            let event = &self.replay_log[self.replay_cursor];
            if event.get_time() > ptime_us {
                break;
            }
            let key = event.keycode as usize;
            if key < phys_count {
                self.key_states[key] = event.pressed;
                self.key_changed_times[key] = event.get_time();
            }
            self.replay_cursor += 1;
        }
    }

    /// Process input during the Playing phase.
    pub(super) fn process_playing_input(&mut self, ctx: &mut StateContext) {
        if self.phase != PlayPhase::Playing || self.key_beam_stop {
            return;
        }

        let ptime_us = ctx.timer.now_time_of(TIMER_PLAY) * 1000;

        // Poll keyboard via InputProcessor (manual play mode)
        if let (Some(ip), Some(backend)) = (&mut self.input_processor, ctx.keyboard_backend) {
            ip.poll_keyboard(ptime_us, backend);
            // Copy key states from InputProcessor
            let phys_count = self.key_states.len();
            for i in 0..phys_count {
                self.key_states[i] = ip.get_key_state(i);
                self.key_changed_times[i] = ip.get_key_changed_time(i);
            }
        }

        // Inject replay events
        if self.is_replay {
            self.inject_replay_events(ptime_us);
        }

        // Update JudgeManager
        if let (Some(jm), Some(gauge)) = (&mut self.judge_manager, &mut self.gauge) {
            let events = jm.update(
                ptime_us,
                &self.judge_notes,
                &self.key_states,
                &self.key_changed_times,
                gauge,
            );

            // Process events inline (mine damage, audio)
            for event in &events {
                match event {
                    JudgeEvent::MineDamage { damage, .. } => {
                        gauge.add_value(-(*damage as f32));
                    }
                    JudgeEvent::KeySound { wav_id } => {
                        if let Some(driver) = &mut self.audio_driver {
                            let note = Note::keysound(*wav_id);
                            driver.play_note(&note, 1.0, 0);
                        }
                    }
                    JudgeEvent::Judge { lane, judge, .. } => {
                        // Trigger miss layer on BD/PR/MS judgments
                        if *judge >= JUDGE_BD
                            && let Some(bga) = &mut self.bga_processor
                        {
                            bga.set_miss_triggered(ptime_us);
                        }
                        // Per-player judge/combo timers
                        let player = self.lane_property.lane_player(*lane);
                        let judge_timer = if player == 0 {
                            TIMER_JUDGE_1P
                        } else {
                            TIMER_JUDGE_2P
                        };
                        let combo_timer = if player == 0 {
                            TIMER_COMBO_1P
                        } else {
                            TIMER_COMBO_2P
                        };
                        ctx.timer.set_timer_on(judge_timer);
                        ctx.timer.set_timer_on(combo_timer);
                    }
                    JudgeEvent::HcnGauge { .. } => {
                        // Already handled internally by JudgeManager
                    }
                }
            }

            // Track whether any notes have been judged (for key beam behavior)
            if !self.is_judge_started && jm.past_notes() > 0 {
                self.is_judge_started = true;
            }

            // Update key beam timers
            update_key_beam_timers(
                &self.lane_property,
                &self.key_states,
                jm.auto_presstime(),
                self.key_beam_stop,
                self.is_autoplay,
                self.is_judge_started,
                ctx.timer,
            );

            // Reset key changed times for next frame
            self.key_changed_times.fill(NOT_SET);

            // Reset InputProcessor's key changed times
            if let Some(ip) = &mut self.input_processor {
                ip.reset_all_key_changed_time();
            }

            // Update score in resource
            ctx.resource.score_data = jm.score().clone();
            ctx.resource.maxcombo = ctx.resource.maxcombo.max(jm.max_combo());
        }
    }
}

/// Update key beam timers based on key states and autoplay press times.
///
/// Ported from Java `KeyInputProccessor.input()` — toggles TIMER_KEYON/TIMER_KEYOFF
/// per lane for skin key beam animation.
pub(super) fn update_key_beam_timers(
    lane_property: &LaneProperty,
    key_states: &[bool],
    auto_presstime: &[i64],
    key_beam_stop: bool,
    _is_autoplay: bool,
    _is_judge_started: bool,
    timer: &mut crate::timer_manager::TimerManager,
) {
    for lane in 0..lane_property.lane_count() {
        let offset = lane_property.lane_skin_offset(lane);
        let player = lane_property.lane_player(lane);
        let is_scratch = lane_property.scratch_index(lane).is_some();

        let mut pressed = false;
        if !key_beam_stop {
            for &key in lane_property.lane_to_keys(lane) {
                if key_states.get(key).copied().unwrap_or(false)
                    || auto_presstime.get(key).copied().unwrap_or(NOT_SET) != NOT_SET
                {
                    pressed = true;
                    break;
                }
            }
        }

        let timer_on = property_mapper::key_on_timer_id(player, offset);
        let timer_off = property_mapper::key_off_timer_id(player, offset);
        if timer_on < 0 || timer_off < 0 {
            continue;
        }

        if pressed {
            // Activate key-on timer. For scratch lanes, always re-trigger
            // (scratch can toggle direction rapidly).
            if !timer.is_timer_on(timer_on) || is_scratch {
                timer.set_timer_on(timer_on);
                timer.set_timer_off(timer_off);
            }
        } else if timer.is_timer_on(timer_on) {
            timer.set_timer_on(timer_off);
            timer.set_timer_off(timer_on);
        }
    }
}
