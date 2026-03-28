use std::collections::HashMap;

use crate::skin::audio_config::DEFAULT_AUDIO_VOLUME;
use crate::skin::config::Config;
use crate::skin::distribution_data::DistributionData;
use crate::skin::main_state_type::MainStateType;
use crate::skin::play_config::PlayConfig;
use crate::skin::player_config::PlayerConfig;
use crate::skin::player_data::PlayerData;
use crate::skin::replay_data::ReplayData;
use crate::skin::score_data::ScoreData;
use crate::skin::score_data_property::ScoreDataProperty;
use crate::skin::skin_action_queue::SkinActionQueue;
use crate::skin::skin_offset::SkinOffset;
use crate::skin::song_data::SongData;
use crate::skin::timer_access::TimerAccess;
use crate::skin::timer_id::TimerId;
use crate::skin::timing_distribution::TimingDistribution;

/// Read-only snapshot of game state for skin rendering.
///
/// Each frame, the active screen builds a `PropertySnapshot` from its live state.
/// The skin rendering pipeline reads exclusively from this snapshot, never from
/// the game state directly. This eliminates the adapter-delegate pattern
/// (`PlayRenderContext`, `ResultRenderContext`, etc.) that was the #1 source of
/// bugs (24% of all fix commits).
///
/// Write-back actions from mouse clicks and Lua scripts are collected in the
/// embedded `SkinActionQueue`. The owning screen drains the queue after the
/// skin render pass.
pub struct PropertySnapshot {
    // ================================================================
    // Timing
    // ================================================================
    /// Current time in milliseconds.
    pub now_time: i64,
    /// Current time in microseconds.
    pub now_micro_time: i64,
    /// Timer values: timer ID → start timestamp in microseconds.
    /// `i64::MIN` means the timer is off.
    pub timers: HashMap<TimerId, i64>,

    // ================================================================
    // State identity
    // ================================================================
    /// Current screen state type.
    pub state_type: Option<MainStateType>,
    /// Milliseconds since application boot.
    pub boot_time_millis: i64,

    // ================================================================
    // Property values (keyed by ID)
    // ================================================================
    /// Integer property values (skin property ID → value).
    pub integers: HashMap<i32, i32>,
    /// Image-index property values (separate namespace from integers).
    pub image_indices: HashMap<i32, i32>,
    /// Boolean property values.
    pub booleans: HashMap<i32, bool>,
    /// Float property values.
    pub floats: HashMap<i32, f32>,
    /// String property values.
    pub strings: HashMap<i32, String>,

    // ================================================================
    // Gameplay state
    // ================================================================
    /// Judge counts: (judge_index, is_fast) → count.
    pub judge_counts: HashMap<(i32, bool), i32>,
    /// Gauge value (0.0 - 1.0).
    pub gauge_value: f32,
    /// Gauge type ID.
    pub gauge_type: i32,
    /// Whether the gauge reached max value.
    pub is_gauge_max: bool,
    /// Gauge element borders: (border, max) per gauge type.
    pub gauge_element_borders: Vec<(f32, f32)>,
    /// Whether the chart mode was changed from original.
    pub is_mode_changed: bool,
    /// Current judge type per player.
    pub now_judges: Vec<i32>,
    /// Current combo count per player.
    pub now_combos: Vec<i32>,
    /// Recent judge timing offsets (circular buffer).
    pub recent_judges: Vec<i64>,
    /// Current write index into the recent judges buffer.
    pub recent_judges_index: usize,

    // ================================================================
    // Config (owned copies for rendering)
    // ================================================================
    /// Player config snapshot. Mutable access returns `&mut` to this copy;
    /// the caller is responsible for propagating changes back.
    pub player_config: Option<Box<PlayerConfig>>,
    /// Global config snapshot.
    pub config: Option<Box<Config>>,
    /// Play config for the current mode.
    pub play_config: Option<Box<PlayConfig>>,

    // ================================================================
    // Song / score data
    // ================================================================
    /// Current song data.
    pub song_data: Option<Box<SongData>>,
    /// Player's score data.
    pub score_data: Option<Box<ScoreData>>,
    /// Rival's score data.
    pub rival_score_data: Option<Box<ScoreData>>,
    /// Target score data.
    pub target_score_data: Option<Box<ScoreData>>,
    /// Replay option data (random/gauge/etc. settings from replay).
    pub replay_option_data: Option<Box<ReplayData>>,
    /// Computed score data property (rate, exscore, etc.).
    pub score_data_property: ScoreDataProperty,

    // ================================================================
    // Skin offsets
    // ================================================================
    /// Skin offset values by ID.
    pub offsets: HashMap<i32, SkinOffset>,

    // ================================================================
    // Mouse
    // ================================================================
    pub mouse_x: f32,
    pub mouse_y: f32,

    // ================================================================
    // Display
    // ================================================================
    pub is_debug: bool,

    // ================================================================
    // Timing distribution (result screens)
    // ================================================================
    pub timing_distribution: Option<TimingDistribution>,
    pub judge_area: Option<Vec<Vec<i32>>>,

    // ================================================================
    // Gauge history (result screens)
    // ================================================================
    pub gauge_history: Option<Vec<Vec<f32>>>,
    pub course_gauge_history: Vec<Vec<Vec<f32>>>,
    pub gauge_border_max: Option<(f32, f32)>,
    pub gauge_min: f32,
    pub gauge_transition_last_values: HashMap<i32, f32>,
    pub result_gauge_type: i32,

    // ================================================================
    // Media / practice
    // ================================================================
    pub is_media_load_finished: bool,
    pub is_practice_mode: bool,

    // ================================================================
    // Select screen data
    // ================================================================
    pub distribution_data: Option<DistributionData>,
    pub mode_image_index: Option<i32>,
    pub sort_image_index: Option<i32>,
    pub ranking_offset: i32,
    /// Clear types for ranking positions (slot 0-9).
    pub ranking_clear_types: Vec<i32>,
    /// Lane shuffle patterns per player: [player][lane] → pattern index.
    pub lane_shuffle_patterns: Option<Vec<Vec<i32>>>,

    // ================================================================
    // Player / course data (for shared property computations)
    // ================================================================
    /// Player statistics (playcount, clear, judge counts, playtime).
    pub player_data: Option<PlayerData>,
    /// Current course stage index (0-based).
    pub course_index: usize,
    /// Number of songs in the current course.
    pub course_song_count: usize,
    /// Whether a course is active.
    pub is_course_mode: bool,
    /// Whether score saving is enabled.
    pub is_update_score: bool,

    // ================================================================
    // Write-back action queue
    // ================================================================
    /// Actions collected during skin rendering (mouse clicks, Lua writes).
    pub actions: SkinActionQueue,
}

impl Default for PropertySnapshot {
    fn default() -> Self {
        Self {
            now_time: 0,
            now_micro_time: 0,
            timers: HashMap::new(),
            state_type: None,
            boot_time_millis: 0,
            integers: HashMap::new(),
            image_indices: HashMap::new(),
            booleans: HashMap::new(),
            floats: HashMap::new(),
            strings: HashMap::new(),
            judge_counts: HashMap::new(),
            gauge_value: 0.0,
            gauge_type: 0,
            is_gauge_max: false,
            gauge_element_borders: Vec::new(),
            is_mode_changed: false,
            now_judges: Vec::new(),
            now_combos: Vec::new(),
            recent_judges: Vec::new(),
            recent_judges_index: 0,
            player_config: None,
            config: None,
            play_config: None,
            song_data: None,
            score_data: None,
            rival_score_data: None,
            target_score_data: None,
            replay_option_data: None,
            score_data_property: ScoreDataProperty::default(),
            offsets: HashMap::new(),
            mouse_x: 0.0,
            mouse_y: 0.0,
            is_debug: false,
            timing_distribution: None,
            judge_area: None,
            gauge_history: None,
            course_gauge_history: Vec::new(),
            gauge_border_max: None,
            gauge_min: 0.0,
            gauge_transition_last_values: HashMap::new(),
            result_gauge_type: 0,
            is_media_load_finished: false,
            is_practice_mode: false,
            distribution_data: None,
            mode_image_index: None,
            sort_image_index: None,
            ranking_offset: 0,
            ranking_clear_types: Vec::new(),
            lane_shuffle_patterns: None,
            player_data: None,
            course_index: 0,
            course_song_count: 0,
            is_course_mode: false,
            is_update_score: false,
            actions: SkinActionQueue::default(),
        }
    }
}

impl PropertySnapshot {
    pub fn new() -> Self {
        Self::default()
    }

    // ================================================================
    // Shared property computations
    //
    // These compute property values from the raw data stored in the
    // snapshot, covering ~80 IDs shared across most screens. Returns
    // `None` when the ID is not a shared property, signaling the
    // caller to fall through to default_*_value.
    // ================================================================

    fn shared_integer_value(&self, id: i32) -> Option<i32> {
        self.player_data_integer(id)
            .or_else(|| self.volume_integer(id))
            .or_else(|| self.song_data_integer(id))
            .or_else(|| self.score_property_integer(id))
            .or_else(|| self.play_config_integer(id))
    }

    fn player_data_integer(&self, id: i32) -> Option<i32> {
        let pd = self.player_data.as_ref()?;
        let val = match id {
            // Cumulative playtime (hours/minutes/seconds)
            17 => (pd.playtime / 3600) as i32,
            18 => ((pd.playtime / 60) % 60) as i32,
            19 => (pd.playtime % 60) as i32,
            // Player profile stats
            30 => pd.playcount as i32,
            31 => pd.clear as i32,
            32 => (pd.playcount - pd.clear) as i32,
            33 => pd.judge_count(0) as i32,
            34 => pd.judge_count(1) as i32,
            35 => pd.judge_count(2) as i32,
            36 => pd.judge_count(3) as i32,
            37 => pd.judge_count(4) as i32,
            333 => {
                let total: i64 = (0..=3).map(|j| pd.judge_count(j)).sum();
                total.min(i32::MAX as i64) as i32
            }
            _ => return None,
        };
        Some(val)
    }

    fn volume_integer(&self, id: i32) -> Option<i32> {
        let audio = self.config.as_ref()?.audio.as_ref()?;
        let val = match id {
            57 => (audio.systemvolume * 100.0) as i32,
            58 => (audio.keyvolume * 100.0) as i32,
            59 => (audio.bgvolume * 100.0) as i32,
            _ => return None,
        };
        Some(val)
    }

    fn song_data_integer(&self, id: i32) -> Option<i32> {
        let song = self.song_data.as_ref()?;
        let val = match id {
            90 => song.chart.maxbpm,
            91 => song.chart.minbpm,
            92 => song
                .info
                .as_ref()
                .map(|i| i.mainbpm as i32)
                .unwrap_or(i32::MIN),
            96 => song.chart.level,
            312 => song.chart.length,
            350 => song.chart.notes,
            351 => song.info.as_ref().map_or(i32::MIN, |i| i.n),
            352 => song.info.as_ref().map_or(i32::MIN, |i| i.ln),
            353 => song.info.as_ref().map_or(i32::MIN, |i| i.s),
            360 => song
                .info
                .as_ref()
                .map_or(i32::MIN, |i| i.peakdensity as i32),
            361 => song
                .info
                .as_ref()
                .map_or(i32::MIN, |i| ((i.peakdensity * 100.0) as i32) % 100),
            362 => song.info.as_ref().map_or(i32::MIN, |i| i.enddensity as i32),
            363 => song
                .info
                .as_ref()
                .map_or(i32::MIN, |i| ((i.enddensity * 100.0) as i32) % 100),
            364 => song.info.as_ref().map_or(i32::MIN, |i| i.density as i32),
            365 => song
                .info
                .as_ref()
                .map_or(i32::MIN, |i| ((i.density * 100.0) as i32) % 100),
            368 => song.info.as_ref().map_or(i32::MIN, |i| i.total as i32),
            400 => song.chart.judge,
            1163 => (song.chart.length.max(0) / 60000) % 60,
            1164 => (song.chart.length.max(0) / 1000) % 60,
            _ => return None,
        };
        Some(val)
    }

    fn score_property_integer(&self, id: i32) -> Option<i32> {
        let sp = &self.score_data_property;
        let has_score = self.score_data.is_some();
        let val = match id {
            // EX score
            71 | 101 | 171 => self.score_data.as_ref().map_or(i32::MIN, |s| s.exscore()),
            // Max score (notes * 2)
            72 => self.score_data.as_ref().map_or(i32::MIN, |s| s.notes * 2),
            // Max combo
            75 => self.score_data.as_ref().map_or(i32::MIN, |s| s.maxcombo),
            // Miss count / minbp
            76 => self.score_data.as_ref().map_or(i32::MIN, |s| s.minbp),
            // Judge counts (total)
            80..=84 => {
                let index = id - 80;
                self.score_data
                    .as_ref()
                    .map_or(i32::MIN, |s| s.judge_count_total(index))
            }
            // Judge count rates (count * 100 / notes)
            85..=89 => {
                let index = id - 85;
                self.score_data.as_ref().map_or(i32::MIN, |s| {
                    if s.notes > 0 {
                        s.judge_count_total(index) * 100 / s.notes
                    } else {
                        i32::MIN
                    }
                })
            }
            // Score data property values
            100 => sp.now_score(),
            102 => {
                if has_score {
                    sp.nowrate_int
                } else {
                    i32::MIN
                }
            }
            103 => {
                if has_score {
                    sp.nowrate_after_dot
                } else {
                    i32::MIN
                }
            }
            108 | 128 | 153 => sp.nowscore - sp.nowrivalscore,
            115 | 155 => {
                if has_score {
                    sp.rate_int
                } else {
                    i32::MIN
                }
            }
            116 | 156 => {
                if has_score {
                    sp.rate_after_dot
                } else {
                    i32::MIN
                }
            }
            121 | 151 => sp.rivalscore,
            122 | 157 => sp.rivalrate_int,
            123 | 158 => sp.rivalrate_after_dot,
            152 | 172 => sp.nowscore - sp.nowbestscore,
            154 => sp.nextrank,
            183 => sp.bestrate_int,
            184 => sp.bestrate_after_dot,
            _ => return None,
        };
        Some(val)
    }

    fn play_config_integer(&self, id: i32) -> Option<i32> {
        let pc = self.play_config.as_ref()?;
        let val = match id {
            10 => (pc.hispeed * 100.0) as i32,
            12 => self
                .player_config
                .as_ref()
                .map_or(i32::MIN, |p| p.judge_settings.judgetiming),
            310 => pc.hispeed as i32,
            311 => ((pc.hispeed * 100.0) as i32) % 100,
            313 => pc.duration,
            _ => return None,
        };
        Some(val)
    }

    fn shared_boolean_value(&self, id: i32) -> Option<bool> {
        let val = match id {
            // BGA on/off from config
            40 => self
                .config
                .as_ref()
                .is_some_and(|c| c.render.bga == crate::skin::config::BgaMode::Off),
            41 => self
                .config
                .as_ref()
                .is_none_or(|c| c.render.bga != crate::skin::config::BgaMode::Off),
            // Save score on/off
            60 => !self.is_update_score,
            61 => self.is_update_score,
            // Stagefile/banner/backbmp existence
            190 => self
                .song_data
                .as_ref()
                .is_none_or(|s| s.file.stagefile.is_empty()),
            191 => self
                .song_data
                .as_ref()
                .is_some_and(|s| !s.file.stagefile.is_empty()),
            192 => self
                .song_data
                .as_ref()
                .is_none_or(|s| s.file.banner.is_empty()),
            193 => self
                .song_data
                .as_ref()
                .is_some_and(|s| !s.file.banner.is_empty()),
            194 => self
                .song_data
                .as_ref()
                .is_none_or(|s| s.file.backbmp.is_empty()),
            195 => self
                .song_data
                .as_ref()
                .is_some_and(|s| !s.file.backbmp.is_empty()),
            // Course stage
            280 => self.is_course_mode && self.course_index == 0,
            281 => self.is_course_mode && self.course_index == 1,
            282 => self.is_course_mode && self.course_index == 2,
            283 => self.is_course_mode && self.course_index == 3,
            // Course final stage
            289 => {
                self.is_course_mode
                    && self.course_song_count > 0
                    && self.course_index == self.course_song_count - 1
            }
            // Course mode
            290 => self.is_course_mode,
            _ => return None,
        };
        Some(val)
    }

    fn shared_float_value(&self, id: i32) -> Option<f32> {
        let audio = self.config.as_ref()?.audio.as_ref();
        let val = match id {
            17 => audio.map_or(DEFAULT_AUDIO_VOLUME, |a| a.systemvolume),
            18 => audio.map_or(DEFAULT_AUDIO_VOLUME, |a| a.keyvolume),
            19 => audio.map_or(DEFAULT_AUDIO_VOLUME, |a| a.bgvolume),
            _ => return None,
        };
        Some(val)
    }

    fn shared_string_value(&self, id: i32) -> Option<String> {
        let song = self.song_data.as_ref()?;
        let val = match id {
            10 => song.metadata.title.clone(),
            11 => song.metadata.subtitle.clone(),
            12 => {
                if song.metadata.subtitle.is_empty() {
                    song.metadata.title.clone()
                } else {
                    format!("{} {}", song.metadata.title, song.metadata.subtitle)
                }
            }
            13 => song.metadata.genre.clone(),
            14 => song.metadata.artist.clone(),
            15 => song.metadata.subartist.clone(),
            16 => {
                if song.metadata.subartist.is_empty() {
                    song.metadata.artist.clone()
                } else {
                    format!("{} {}", song.metadata.artist, song.metadata.subartist)
                }
            }
            _ => return None,
        };
        Some(val)
    }

    fn shared_image_index_value(&self, id: i32) -> Option<i32> {
        match id {
            308 => {
                if let Some(song) = self.song_data.as_ref()
                    && let Some(override_val) =
                        crate::skin::skin_render_context::compute_lnmode_from_chart(&song.chart)
                {
                    Some(override_val)
                } else {
                    None // fall through to default_image_index_value
                }
            }
            _ => None,
        }
    }
}

// ================================================================
// TimerAccess implementation
// ================================================================
impl TimerAccess for PropertySnapshot {
    fn now_time(&self) -> i64 {
        self.now_time
    }

    fn now_micro_time(&self) -> i64 {
        self.now_micro_time
    }

    fn micro_timer(&self, timer_id: TimerId) -> i64 {
        self.timers.get(&timer_id).copied().unwrap_or(i64::MIN)
    }

    fn timer(&self, timer_id: TimerId) -> i64 {
        let micro = self.micro_timer(timer_id);
        if micro == i64::MIN {
            i64::MIN / 1000
        } else {
            micro / 1000
        }
    }

    fn now_time_for(&self, timer_id: TimerId) -> i64 {
        let micro = self.micro_timer(timer_id);
        if micro == i64::MIN {
            0
        } else {
            (self.now_micro_time - micro) / 1000
        }
    }

    fn is_timer_on(&self, timer_id: TimerId) -> bool {
        self.timers.get(&timer_id).is_some_and(|&v| v != i64::MIN)
    }
}

// ================================================================
// SkinRenderContext implementation
// ================================================================
impl crate::skin::skin_render_context::SkinRenderContext for PropertySnapshot {
    fn execute_event(&mut self, id: i32, arg1: i32, arg2: i32) {
        self.actions.custom_events.push((id, arg1, arg2));
    }

    fn change_state(&mut self, state: MainStateType) {
        self.actions.state_changes.push(state);
    }

    fn set_timer_micro(&mut self, timer_id: TimerId, micro_time: i64) {
        self.actions.timer_sets.push((timer_id, micro_time));
    }

    fn audio_play(&mut self, path: &str, volume: f32, is_loop: bool) {
        self.actions
            .audio_plays
            .push((path.to_owned(), volume, is_loop));
    }

    fn audio_stop(&mut self, path: &str) {
        self.actions.audio_stops.push(path.to_owned());
    }

    fn current_state_type(&self) -> Option<MainStateType> {
        self.state_type
    }

    fn recent_judges(&self) -> &[i64] {
        &self.recent_judges
    }

    fn recent_judges_index(&self) -> usize {
        self.recent_judges_index
    }

    fn boot_time_millis(&self) -> i64 {
        self.boot_time_millis
    }

    fn integer_value(&self, id: i32) -> i32 {
        self.integers
            .get(&id)
            .copied()
            .or_else(|| self.shared_integer_value(id))
            .unwrap_or_else(|| self.default_integer_value(id))
    }

    fn image_index_value(&self, id: i32) -> i32 {
        self.image_indices
            .get(&id)
            .copied()
            .or_else(|| self.shared_image_index_value(id))
            .unwrap_or_else(|| self.default_image_index_value(id))
    }

    fn boolean_value(&self, id: i32) -> bool {
        self.booleans
            .get(&id)
            .copied()
            .or_else(|| self.shared_boolean_value(id))
            .unwrap_or_else(|| self.default_boolean_value(id))
    }

    fn float_value(&self, id: i32) -> f32 {
        self.floats
            .get(&id)
            .copied()
            .or_else(|| self.shared_float_value(id))
            .unwrap_or_else(|| self.default_float_value(id))
    }

    fn string_value(&self, id: i32) -> String {
        self.strings
            .get(&id)
            .cloned()
            .or_else(|| self.shared_string_value(id))
            .unwrap_or_default()
    }

    fn set_float_value(&mut self, id: i32, value: f32) {
        // Update the snapshot copy so subsequent reads in the same frame see it.
        self.floats.insert(id, value);
        // Also queue for write-back to game state.
        self.actions.float_writes.push((id, value));
    }

    fn replay_option_data(&self) -> Option<&ReplayData> {
        self.replay_option_data.as_deref()
    }

    fn target_score_data(&self) -> Option<&ScoreData> {
        self.target_score_data.as_deref()
    }

    fn score_data_ref(&self) -> Option<&ScoreData> {
        self.score_data.as_deref()
    }

    fn rival_score_data_ref(&self) -> Option<&ScoreData> {
        self.rival_score_data.as_deref()
    }

    fn ranking_score_clear_type(&self, slot: i32) -> i32 {
        let index = (self.ranking_offset + slot) as usize;
        self.ranking_clear_types.get(index).copied().unwrap_or(-1)
    }

    fn ranking_offset(&self) -> i32 {
        self.ranking_offset
    }

    fn current_play_config_ref(&self) -> Option<&PlayConfig> {
        self.play_config.as_deref()
    }

    fn song_data_ref(&self) -> Option<&SongData> {
        self.song_data.as_deref()
    }

    fn lane_shuffle_pattern_value(&self, player: usize, lane: usize) -> i32 {
        self.lane_shuffle_patterns
            .as_ref()
            .and_then(|patterns| patterns.get(player))
            .and_then(|lanes| lanes.get(lane))
            .copied()
            .unwrap_or(-1)
    }

    fn mode_image_index(&self) -> Option<i32> {
        self.mode_image_index
    }

    fn sort_image_index(&self) -> Option<i32> {
        self.sort_image_index
    }

    fn judge_count(&self, judge: i32, fast: bool) -> i32 {
        self.judge_counts.get(&(judge, fast)).copied().unwrap_or(0)
    }

    fn gauge_value(&self) -> f32 {
        self.gauge_value
    }

    fn gauge_type(&self) -> i32 {
        self.gauge_type
    }

    fn is_mode_changed(&self) -> bool {
        self.is_mode_changed
    }

    fn gauge_element_borders(&self) -> Vec<(f32, f32)> {
        self.gauge_element_borders.clone()
    }

    fn now_judge(&self, player: i32) -> i32 {
        self.now_judges.get(player as usize).copied().unwrap_or(0)
    }

    fn now_combo(&self, player: i32) -> i32 {
        self.now_combos.get(player as usize).copied().unwrap_or(0)
    }

    fn player_config_ref(&self) -> Option<&PlayerConfig> {
        self.player_config.as_deref()
    }

    fn player_config_mut(&mut self) -> Option<&mut PlayerConfig> {
        self.player_config.as_deref_mut()
    }

    fn config_ref(&self) -> Option<&Config> {
        self.config.as_deref()
    }

    fn config_mut(&mut self) -> Option<&mut Config> {
        self.config.as_deref_mut()
    }

    fn selected_play_config_mut(&mut self) -> Option<&mut PlayConfig> {
        self.play_config.as_deref_mut()
    }

    fn notify_audio_config_changed(&mut self) {
        self.actions.audio_config_changed = true;
    }

    fn play_option_change_sound(&mut self) {
        self.actions.option_change_sound = true;
    }

    fn update_bar_after_change(&mut self) {
        self.actions.update_bar_after_change = true;
    }

    fn select_song_mode(&mut self, event_id: i32) {
        self.actions.select_song_mode_requests.push(event_id);
    }

    fn get_offset_value(&self, id: i32) -> Option<&SkinOffset> {
        self.offsets.get(&id)
    }

    fn mouse_x(&self) -> f32 {
        self.mouse_x
    }

    fn mouse_y(&self) -> f32 {
        self.mouse_y
    }

    fn is_debug(&self) -> bool {
        self.is_debug
    }

    fn get_timing_distribution(&self) -> Option<&TimingDistribution> {
        self.timing_distribution.as_ref()
    }

    fn judge_area(&self) -> Option<Vec<Vec<i32>>> {
        self.judge_area.clone()
    }

    fn score_data_property(&self) -> &ScoreDataProperty {
        &self.score_data_property
    }

    fn gauge_history(&self) -> Option<&Vec<Vec<f32>>> {
        self.gauge_history.as_ref()
    }

    fn course_gauge_history(&self) -> &[Vec<Vec<f32>>] {
        &self.course_gauge_history
    }

    fn gauge_border_max(&self) -> Option<(f32, f32)> {
        self.gauge_border_max
    }

    fn gauge_min(&self) -> f32 {
        self.gauge_min
    }

    fn gauge_transition_last_value(&self, gauge_type: i32) -> Option<f32> {
        self.gauge_transition_last_values.get(&gauge_type).copied()
    }

    fn result_gauge_type(&self) -> i32 {
        self.result_gauge_type
    }

    fn is_gauge_max(&self) -> bool {
        self.is_gauge_max
    }

    fn is_media_load_finished(&self) -> bool {
        self.is_media_load_finished
    }

    fn is_practice_mode(&self) -> bool {
        self.is_practice_mode
    }

    fn get_distribution_data(&self) -> Option<DistributionData> {
        self.distribution_data.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::skin::skin_render_context::SkinRenderContext;

    #[test]
    fn default_snapshot_returns_safe_defaults() {
        let snapshot = PropertySnapshot::new();
        assert_eq!(snapshot.now_time(), 0);
        assert_eq!(snapshot.now_micro_time(), 0);
        assert_eq!(snapshot.micro_timer(TimerId::new(0)), i64::MIN);
        assert!(!snapshot.is_timer_on(TimerId::new(0)));
        assert_eq!(snapshot.now_time_for(TimerId::new(0)), 0);
        assert_eq!(snapshot.integer_value(999), i32::MIN);
        assert!(!snapshot.boolean_value(999));
        assert_eq!(snapshot.float_value(999), f32::MIN);
        assert_eq!(snapshot.string_value(999), "");
        assert_eq!(snapshot.gauge_value(), 0.0);
        assert_eq!(snapshot.gauge_type(), 0);
        assert_eq!(snapshot.judge_count(0, false), 0);
        assert_eq!(snapshot.now_judge(0), 0);
        assert_eq!(snapshot.now_combo(0), 0);
        assert!(snapshot.player_config_ref().is_none());
        assert!(snapshot.config_ref().is_none());
        assert!(snapshot.song_data_ref().is_none());
        assert!(snapshot.score_data_ref().is_none());
        assert!(snapshot.get_offset_value(0).is_none());
        assert_eq!(snapshot.mouse_x(), 0.0);
        assert_eq!(snapshot.mouse_y(), 0.0);
        assert!(!snapshot.is_debug());
        assert!(!snapshot.is_media_load_finished());
        assert!(!snapshot.is_practice_mode());
        assert!(snapshot.current_state_type().is_none());
    }

    #[test]
    fn timer_access_with_populated_timers() {
        let mut snapshot = PropertySnapshot::new();
        snapshot.now_time = 5000;
        snapshot.now_micro_time = 5_000_000;
        snapshot.timers.insert(TimerId::new(1), 2_000_000);
        snapshot.timers.insert(TimerId::new(2), i64::MIN);

        // Timer 1 is on, started at 2s
        assert!(snapshot.is_timer_on(TimerId::new(1)));
        assert_eq!(snapshot.micro_timer(TimerId::new(1)), 2_000_000);
        assert_eq!(snapshot.timer(TimerId::new(1)), 2000);
        assert_eq!(snapshot.now_time_for(TimerId::new(1)), 3000); // 5s - 2s = 3s

        // Timer 2 is off (i64::MIN)
        assert!(!snapshot.is_timer_on(TimerId::new(2)));
        assert_eq!(snapshot.now_time_for(TimerId::new(2)), 0);

        // Timer 3 is not set
        assert!(!snapshot.is_timer_on(TimerId::new(3)));
        assert_eq!(snapshot.micro_timer(TimerId::new(3)), i64::MIN);
    }

    #[test]
    fn integer_value_falls_through_to_default() {
        let snapshot = PropertySnapshot::new();

        // Default integer values: date/time IDs should be non-zero
        let year = snapshot.integer_value(21);
        assert!(year >= 2024, "year should be current: {}", year);

        // Unhandled ID returns i32::MIN
        assert_eq!(snapshot.integer_value(9999), i32::MIN);
    }

    #[test]
    fn integer_value_prefers_stored_value() {
        let mut snapshot = PropertySnapshot::new();
        snapshot.integers.insert(21, 42);

        // Should return stored value, not live date
        assert_eq!(snapshot.integer_value(21), 42);
    }

    #[test]
    fn boolean_value_with_song_data() {
        let mut snapshot = PropertySnapshot::new();
        let mut song = SongData::default();
        song.chart.difficulty = 3;
        song.chart.mode = 7;
        snapshot.song_data = Some(Box::new(song));

        // ID 153 = difficulty 3
        assert!(snapshot.boolean_value(153));
        // ID 152 = difficulty 2
        assert!(!snapshot.boolean_value(152));
        // ID 160 = 7key
        assert!(snapshot.boolean_value(160));
        // ID 161 = 5key
        assert!(!snapshot.boolean_value(161));
    }

    #[test]
    fn boolean_value_prefers_stored_value() {
        let mut snapshot = PropertySnapshot::new();
        // Override default: pretend difficulty check says true even with no song data
        snapshot.booleans.insert(153, true);
        assert!(snapshot.boolean_value(153));
    }

    #[test]
    fn float_value_with_play_config() {
        let mut snapshot = PropertySnapshot::new();
        let mut pc = PlayConfig::default();
        pc.hispeed = 3.5;
        snapshot.play_config = Some(Box::new(pc));

        // ID 310 = hispeed
        assert_eq!(snapshot.float_value(310), 3.5);
    }

    #[test]
    fn set_float_value_updates_snapshot_and_queues_action() {
        let mut snapshot = PropertySnapshot::new();
        snapshot.set_float_value(1, 0.75);

        // Reads reflect the new value
        assert_eq!(snapshot.float_value(1), 0.75);
        // Action is queued
        assert_eq!(snapshot.actions.float_writes.len(), 1);
        assert_eq!(snapshot.actions.float_writes[0], (1, 0.75));
    }

    #[test]
    fn write_methods_queue_actions() {
        let mut snapshot = PropertySnapshot::new();
        snapshot.execute_event(100, 1, 2);
        snapshot.change_state(MainStateType::Play);
        snapshot.set_timer_micro(TimerId::new(5), 1_000_000);
        snapshot.audio_play("test.wav", 0.8, true);
        snapshot.audio_stop("test.wav");
        snapshot.notify_audio_config_changed();
        snapshot.play_option_change_sound();
        snapshot.update_bar_after_change();
        snapshot.select_song_mode(42);

        assert!(!snapshot.actions.is_empty());
        assert_eq!(snapshot.actions.custom_events, vec![(100, 1, 2)]);
        assert_eq!(snapshot.actions.state_changes, vec![MainStateType::Play]);
        assert_eq!(
            snapshot.actions.timer_sets,
            vec![(TimerId::new(5), 1_000_000)]
        );
        assert_eq!(
            snapshot.actions.audio_plays,
            vec![("test.wav".to_owned(), 0.8, true)]
        );
        assert_eq!(snapshot.actions.audio_stops, vec!["test.wav".to_owned()]);
        assert!(snapshot.actions.audio_config_changed);
        assert!(snapshot.actions.option_change_sound);
        assert!(snapshot.actions.update_bar_after_change);
        assert_eq!(snapshot.actions.select_song_mode_requests, vec![42]);
    }

    #[test]
    fn judge_count_lookup() {
        let mut snapshot = PropertySnapshot::new();
        snapshot.judge_counts.insert((0, false), 100);
        snapshot.judge_counts.insert((0, true), 30);
        snapshot.judge_counts.insert((1, false), 50);

        assert_eq!(snapshot.judge_count(0, false), 100);
        assert_eq!(snapshot.judge_count(0, true), 30);
        assert_eq!(snapshot.judge_count(1, false), 50);
        assert_eq!(snapshot.judge_count(2, false), 0); // not set
    }

    #[test]
    fn now_judge_and_combo_per_player() {
        let mut snapshot = PropertySnapshot::new();
        snapshot.now_judges = vec![3, 1];
        snapshot.now_combos = vec![42, 10];

        assert_eq!(snapshot.now_judge(0), 3);
        assert_eq!(snapshot.now_judge(1), 1);
        assert_eq!(snapshot.now_judge(2), 0); // out of range
        assert_eq!(snapshot.now_combo(0), 42);
        assert_eq!(snapshot.now_combo(1), 10);
    }

    #[test]
    fn ranking_score_clear_type_with_offset() {
        let mut snapshot = PropertySnapshot::new();
        snapshot.ranking_clear_types = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        snapshot.ranking_offset = 3;

        // slot 0 → index 3
        assert_eq!(snapshot.ranking_score_clear_type(0), 3);
        // slot 5 → index 8
        assert_eq!(snapshot.ranking_score_clear_type(5), 8);
        // slot 8 → index 11 → out of range
        assert_eq!(snapshot.ranking_score_clear_type(8), -1);
    }

    #[test]
    fn lane_shuffle_pattern_lookup() {
        let mut snapshot = PropertySnapshot::new();
        snapshot.lane_shuffle_patterns =
            Some(vec![vec![1, 2, 3, 4, 5, 6, 7], vec![7, 6, 5, 4, 3, 2, 1]]);

        assert_eq!(snapshot.lane_shuffle_pattern_value(0, 0), 1);
        assert_eq!(snapshot.lane_shuffle_pattern_value(1, 0), 7);
        assert_eq!(snapshot.lane_shuffle_pattern_value(0, 6), 7);
        assert_eq!(snapshot.lane_shuffle_pattern_value(2, 0), -1); // no player 2
    }

    #[test]
    fn gauge_history_access() {
        let mut snapshot = PropertySnapshot::new();
        let history = vec![vec![0.0, 0.5, 1.0], vec![0.1, 0.3, 0.8]];
        snapshot.gauge_history = Some(history.clone());
        snapshot.course_gauge_history = vec![history.clone()];
        snapshot.gauge_border_max = Some((0.8, 1.0));
        snapshot.gauge_min = 0.02;
        snapshot.gauge_transition_last_values.insert(0, 0.95);

        assert_eq!(snapshot.gauge_history(), Some(&history));
        assert_eq!(snapshot.course_gauge_history().len(), 1);
        assert_eq!(snapshot.gauge_border_max(), Some((0.8, 1.0)));
        assert_eq!(snapshot.gauge_min(), 0.02);
        assert_eq!(snapshot.gauge_transition_last_value(0), Some(0.95));
        assert_eq!(snapshot.gauge_transition_last_value(1), None);
    }

    #[test]
    fn config_mut_returns_owned_copy() {
        let mut snapshot = PropertySnapshot::new();
        snapshot.player_config = Some(Box::new(PlayerConfig::default()));
        snapshot.config = Some(Box::new(Config::default()));
        snapshot.play_config = Some(Box::new(PlayConfig::default()));

        // Mutable access works
        assert!(snapshot.player_config_mut().is_some());
        assert!(snapshot.config_mut().is_some());
        assert!(snapshot.selected_play_config_mut().is_some());
    }

    #[test]
    fn state_type_helpers() {
        let mut snapshot = PropertySnapshot::new();

        snapshot.state_type = Some(MainStateType::MusicSelect);
        assert!(snapshot.is_music_selector());
        assert!(!snapshot.is_result_state());
        assert!(!snapshot.is_bms_player());

        snapshot.state_type = Some(MainStateType::Play);
        assert!(!snapshot.is_music_selector());
        assert!(snapshot.is_bms_player());

        snapshot.state_type = Some(MainStateType::Result);
        assert!(snapshot.is_result_state());
    }

    // ================================================================
    // Shared property computation tests
    // ================================================================

    #[test]
    fn shared_player_data_integers() {
        let mut snapshot = PropertySnapshot::new();
        let mut pd = PlayerData::default();
        pd.playtime = 3723; // 1h 2m 3s
        pd.playcount = 500;
        pd.clear = 300;
        pd.epg = 100;
        pd.lpg = 50;
        pd.egr = 80;
        pd.lgr = 40;
        pd.egd = 30;
        pd.lgd = 20;
        pd.ebd = 10;
        pd.lbd = 5;
        pd.epr = 8;
        pd.lpr = 3;
        pd.ems = 2;
        pd.lms = 1;
        snapshot.player_data = Some(pd);

        // Playtime: 3723s = 1h 2m 3s
        assert_eq!(snapshot.integer_value(17), 1); // hours
        assert_eq!(snapshot.integer_value(18), 2); // minutes
        assert_eq!(snapshot.integer_value(19), 3); // seconds
        // Player stats
        assert_eq!(snapshot.integer_value(30), 500); // playcount
        assert_eq!(snapshot.integer_value(31), 300); // clear
        assert_eq!(snapshot.integer_value(32), 200); // playcount - clear
    }

    #[test]
    fn shared_player_data_none_falls_through() {
        let snapshot = PropertySnapshot::new();
        // Without player_data, IDs 17-19 should fall through to default (FPS/date)
        // ID 17 is in player_data_integer range but player_data is None,
        // so it falls through to default_integer_value which returns i32::MIN for 17
        assert_eq!(snapshot.integer_value(17), i32::MIN);
    }

    #[test]
    fn shared_volume_integers() {
        let mut snapshot = PropertySnapshot::new();
        let mut config = Config::default();
        let mut audio = crate::skin::audio_config::AudioConfig::default();
        audio.systemvolume = 0.75;
        audio.keyvolume = 0.5;
        audio.bgvolume = 0.25;
        config.audio = Some(audio);
        snapshot.config = Some(Box::new(config));

        assert_eq!(snapshot.integer_value(57), 75);
        assert_eq!(snapshot.integer_value(58), 50);
        assert_eq!(snapshot.integer_value(59), 25);
    }

    #[test]
    fn shared_song_data_integers() {
        let mut snapshot = PropertySnapshot::new();
        let mut song = SongData::default();
        song.chart.maxbpm = 200;
        song.chart.minbpm = 100;
        song.chart.level = 12;
        song.chart.notes = 1500;
        song.chart.length = 125000; // 2m 5s
        song.chart.judge = 50;
        snapshot.song_data = Some(Box::new(song));

        assert_eq!(snapshot.integer_value(90), 200); // maxbpm
        assert_eq!(snapshot.integer_value(91), 100); // minbpm
        assert_eq!(snapshot.integer_value(96), 12); // level
        assert_eq!(snapshot.integer_value(312), 125000); // length
        assert_eq!(snapshot.integer_value(350), 1500); // notes
        assert_eq!(snapshot.integer_value(400), 50); // judge
        assert_eq!(snapshot.integer_value(1163), 2); // duration minutes
        assert_eq!(snapshot.integer_value(1164), 5); // duration seconds
    }

    #[test]
    fn shared_score_property_integers() {
        let mut snapshot = PropertySnapshot::new();
        let mut score = ScoreData::default();
        score.judge_counts.epg = 100;
        score.judge_counts.lpg = 50;
        score.notes = 200;
        score.maxcombo = 180;
        score.minbp = 10;
        snapshot.score_data = Some(Box::new(score));
        snapshot.score_data_property.nowpoint = 300;
        snapshot.score_data_property.nowscore = 350;
        snapshot.score_data_property.nowrivalscore = 250;
        snapshot.score_data_property.nowbestscore = 280;
        snapshot.score_data_property.nowrate_int = 85;
        snapshot.score_data_property.nowrate_after_dot = 50;

        // EX score = (epg + lpg) * 2 + egr + lgr = (100+50)*2 = 300
        assert_eq!(snapshot.integer_value(71), 300);
        // Max score = notes * 2
        assert_eq!(snapshot.integer_value(72), 400);
        // Max combo
        assert_eq!(snapshot.integer_value(75), 180);
        // Miss count
        assert_eq!(snapshot.integer_value(76), 10);
        // Score property (now_score() reads nowpoint)
        assert_eq!(snapshot.integer_value(100), 300); // nowpoint
        assert_eq!(snapshot.integer_value(102), 85); // nowrate_int
        // Diff vs target (nowscore - nowrivalscore)
        assert_eq!(snapshot.integer_value(108), 100); // 350 - 250
        // Diff vs best (nowscore - nowbestscore)
        assert_eq!(snapshot.integer_value(152), 70); // 350 - 280
    }

    #[test]
    fn shared_play_config_integers() {
        let mut snapshot = PropertySnapshot::new();
        let mut pc = PlayConfig::default();
        pc.hispeed = 3.75;
        pc.duration = 500;
        snapshot.play_config = Some(Box::new(pc));
        snapshot.player_config = Some(Box::new(PlayerConfig::default()));

        assert_eq!(snapshot.integer_value(10), 375); // hispeed * 100
        assert_eq!(snapshot.integer_value(310), 3); // hispeed integer part
        assert_eq!(snapshot.integer_value(311), 75); // hispeed afterdot
        assert_eq!(snapshot.integer_value(313), 500); // duration
    }

    #[test]
    fn shared_boolean_bga() {
        let mut snapshot = PropertySnapshot::new();
        let mut config = Config::default();
        config.render.bga = crate::skin::config::BgaMode::Off;
        snapshot.config = Some(Box::new(config));

        assert!(snapshot.boolean_value(40)); // BGA off
        assert!(!snapshot.boolean_value(41)); // BGA on
    }

    #[test]
    fn shared_boolean_save_score() {
        let mut snapshot = PropertySnapshot::new();
        snapshot.is_update_score = true;

        assert!(!snapshot.boolean_value(60)); // disable save = false
        assert!(snapshot.boolean_value(61)); // enable save = true
    }

    #[test]
    fn shared_boolean_stagefile() {
        let mut snapshot = PropertySnapshot::new();
        let mut song = SongData::default();
        song.file.stagefile = "stage.bmp".to_string();
        snapshot.song_data = Some(Box::new(song));

        assert!(!snapshot.boolean_value(190)); // no stagefile = false
        assert!(snapshot.boolean_value(191)); // has stagefile = true
    }

    #[test]
    fn shared_boolean_course() {
        let mut snapshot = PropertySnapshot::new();
        snapshot.is_course_mode = true;
        snapshot.course_index = 2;
        snapshot.course_song_count = 4;

        assert!(!snapshot.boolean_value(280)); // stage 0
        assert!(!snapshot.boolean_value(281)); // stage 1
        assert!(snapshot.boolean_value(282)); // stage 2 (current)
        assert!(!snapshot.boolean_value(283)); // stage 3
        assert!(!snapshot.boolean_value(289)); // final stage (index 2 != 3)
        assert!(snapshot.boolean_value(290)); // is course mode
    }

    #[test]
    fn shared_boolean_course_final_stage() {
        let mut snapshot = PropertySnapshot::new();
        snapshot.is_course_mode = true;
        snapshot.course_index = 3;
        snapshot.course_song_count = 4;

        assert!(snapshot.boolean_value(289)); // final stage
    }

    #[test]
    fn shared_float_volume() {
        let mut snapshot = PropertySnapshot::new();
        let mut config = Config::default();
        let mut audio = crate::skin::audio_config::AudioConfig::default();
        audio.systemvolume = 0.8;
        audio.keyvolume = 0.6;
        audio.bgvolume = 0.4;
        config.audio = Some(audio);
        snapshot.config = Some(Box::new(config));

        assert_eq!(snapshot.float_value(17), 0.8);
        assert_eq!(snapshot.float_value(18), 0.6);
        assert_eq!(snapshot.float_value(19), 0.4);
    }

    #[test]
    fn shared_string_song_metadata() {
        let mut snapshot = PropertySnapshot::new();
        let mut song = SongData::default();
        song.metadata.title = "Test Song".to_string();
        song.metadata.subtitle = "~remix~".to_string();
        song.metadata.artist = "DJ Test".to_string();
        song.metadata.subartist = "feat. Vocal".to_string();
        song.metadata.genre = "Techno".to_string();
        snapshot.song_data = Some(Box::new(song));

        assert_eq!(snapshot.string_value(10), "Test Song");
        assert_eq!(snapshot.string_value(11), "~remix~");
        assert_eq!(snapshot.string_value(12), "Test Song ~remix~");
        assert_eq!(snapshot.string_value(13), "Techno");
        assert_eq!(snapshot.string_value(14), "DJ Test");
        assert_eq!(snapshot.string_value(15), "feat. Vocal");
        assert_eq!(snapshot.string_value(16), "DJ Test feat. Vocal");
    }

    #[test]
    fn shared_string_no_subtitle() {
        let mut snapshot = PropertySnapshot::new();
        let mut song = SongData::default();
        song.metadata.title = "Test Song".to_string();
        snapshot.song_data = Some(Box::new(song));

        // ID 12 (full title) should be just title when subtitle is empty
        assert_eq!(snapshot.string_value(12), "Test Song");
    }

    #[test]
    fn shared_image_index_lnmode() {
        let mut snapshot = PropertySnapshot::new();
        let mut song = SongData::default();
        song.chart.feature = crate::skin::song_data::FEATURE_LONGNOTE;
        snapshot.song_data = Some(Box::new(song));

        // LN type = 0 (LN) because has_long_note && has_any_long_note && !has_undefined
        assert_eq!(snapshot.image_index_value(308), 0);
    }

    #[test]
    fn hashmap_overrides_shared_value() {
        let mut snapshot = PropertySnapshot::new();
        let mut pd = PlayerData::default();
        pd.playcount = 500;
        snapshot.player_data = Some(pd);

        // Shared value: ID 30 = playcount = 500
        assert_eq!(snapshot.integer_value(30), 500);

        // HashMap override takes priority
        snapshot.integers.insert(30, 999);
        assert_eq!(snapshot.integer_value(30), 999);
    }
}
