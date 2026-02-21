// External dependency stubs for beatoraja-result crate
// These will be replaced with actual implementations when corresponding phases are translated.

use std::sync::Arc;

// ============================================================
// MainController stub
// ============================================================

/// Stub for bms.player.beatoraja.MainController
pub struct MainController;

impl MainController {
    pub fn get_input_processor(&mut self) -> &mut BMSPlayerInputProcessor {
        todo!("Phase 8+ dependency: MainController.getInputProcessor")
    }

    pub fn get_config(&self) -> &beatoraja_core::config::Config {
        todo!("Phase 8+ dependency: MainController.getConfig")
    }

    pub fn get_player_config(&self) -> &beatoraja_core::player_config::PlayerConfig {
        todo!("Phase 8+ dependency: MainController.getPlayerConfig")
    }

    pub fn get_ir_status(&self) -> &[IRStatus] {
        todo!("Phase 8+ dependency: MainController.getIRStatus")
    }

    pub fn get_play_data_accessor(&self) -> &PlayDataAccessor {
        todo!("Phase 8+ dependency: MainController.getPlayDataAccessor")
    }

    pub fn get_audio_processor(&self) -> &AudioProcessor {
        todo!("Phase 8+ dependency: MainController.getAudioProcessor")
    }

    pub fn change_state(&mut self, _state_type: beatoraja_core::main_state::MainStateType) {
        todo!("Phase 8+ dependency: MainController.changeState")
    }

    pub fn save_last_recording(&self, _tag: &str) {
        todo!("Phase 8+ dependency: MainController.saveLastRecording")
    }

    pub fn get_ranking_data_cache(&self) -> &RankingDataCache {
        todo!("Phase 8+ dependency: MainController.getRankingDataCache")
    }

    pub fn ir_send_status(&self) -> &Vec<IRSendStatusMain> {
        todo!("Phase 8+ dependency: MainController.irSendStatus")
    }

    pub fn ir_send_status_mut(&mut self) -> &mut Vec<IRSendStatusMain> {
        todo!("Phase 8+ dependency: MainController.irSendStatus")
    }
}

// ============================================================
// IR (Internet Ranking) stubs
// ============================================================

/// Stub for bms.player.beatoraja.MainController.IRStatus
pub struct IRStatus {
    pub connection: IRConnection,
    pub config: IRConfig,
}

/// Stub for bms.player.beatoraja.ir.IRConnection
#[derive(Clone)]
pub struct IRConnection;

impl IRConnection {
    pub fn get_course_play_data(
        &self,
        _player: Option<()>,
        _course_data: &IRCourseData,
    ) -> IRResponse<Vec<IRScoreData>> {
        todo!("IR dependency: IRConnection.getCoursePlayData")
    }

    pub fn get_play_data(
        &self,
        _player: Option<()>,
        _chart_data: &IRChartData,
    ) -> IRResponse<Vec<IRScoreData>> {
        todo!("IR dependency: IRConnection.getPlayData")
    }

    pub fn send_course_play_data(
        &self,
        _course_data: &IRCourseData,
        _score_data: &IRScoreData,
    ) -> IRResponse<()> {
        todo!("IR dependency: IRConnection.sendCoursePlayData")
    }

    pub fn send_play_data(
        &self,
        _chart_data: &IRChartData,
        _score_data: &IRScoreData,
    ) -> IRResponse<()> {
        todo!("IR dependency: IRConnection.sendPlayData")
    }
}

/// Stub for bms.player.beatoraja.ir.IRConfig
pub struct IRConfig;

impl IRConfig {
    pub const IR_SEND_ALWAYS: i32 = 0;
    pub const IR_SEND_COMPLETE_SONG: i32 = 1;
    pub const IR_SEND_UPDATE_SCORE: i32 = 2;

    pub fn get_irsend(&self) -> i32 {
        0
    }
}

/// Stub for bms.player.beatoraja.ir.IRResponse
pub struct IRResponse<T> {
    pub data: Option<T>,
    pub message: String,
    pub succeeded: bool,
}

impl<T> IRResponse<T> {
    pub fn is_succeeded(&self) -> bool {
        self.succeeded
    }

    pub fn get_data(&self) -> Option<&T> {
        self.data.as_ref()
    }

    pub fn get_message(&self) -> &str {
        &self.message
    }
}

/// Stub for bms.player.beatoraja.ir.IRScoreData
pub struct IRScoreData {
    pub score: beatoraja_core::score_data::ScoreData,
}

impl IRScoreData {
    pub fn new(score: &beatoraja_core::score_data::ScoreData) -> Self {
        Self {
            score: score.clone(),
        }
    }
}

/// Stub for bms.player.beatoraja.ir.IRCourseData
pub struct IRCourseData {
    pub course: beatoraja_core::course_data::CourseData,
    pub lnmode: i32,
}

impl IRCourseData {
    pub fn new(course: &beatoraja_core::course_data::CourseData, lnmode: i32) -> Self {
        Self {
            course: course.clone(),
            lnmode,
        }
    }
}

/// Stub for bms.player.beatoraja.ir.IRChartData
pub struct IRChartData;

impl IRChartData {
    pub fn new(_songdata: &SongData) -> Self {
        Self
    }
}

/// Stub for bms.player.beatoraja.ir.RankingData
#[derive(Clone, Debug)]
pub struct RankingData {
    pub rank: i32,
    pub previous_rank: i32,
    pub total_player: i32,
}

impl Default for RankingData {
    fn default() -> Self {
        Self::new()
    }
}

impl RankingData {
    pub fn new() -> Self {
        Self {
            rank: 0,
            previous_rank: 0,
            total_player: 0,
        }
    }

    pub fn get_rank(&self) -> i32 {
        self.rank
    }

    pub fn get_previous_rank(&self) -> i32 {
        self.previous_rank
    }

    pub fn get_total_player(&self) -> i32 {
        self.total_player
    }

    pub fn update_score(
        &mut self,
        _data: Option<&Vec<IRScoreData>>,
        _score: &beatoraja_core::score_data::ScoreData,
    ) {
        todo!("IR dependency: RankingData.updateScore")
    }
}

/// Stub for RankingDataCache
pub struct RankingDataCache;

impl RankingDataCache {
    pub fn get(&self, _songdata: &SongData, _lnmode: i32) -> Option<RankingData> {
        None
    }

    pub fn put(&self, _songdata: &SongData, _lnmode: i32, _data: RankingData) {
        // stub
    }
}

// ============================================================
// MainController.IRSendStatus (for MusicResult)
// ============================================================

/// Stub for bms.player.beatoraja.MainController.IRSendStatus
pub struct IRSendStatusMain {
    pub connection: IRConnection,
    pub songdata: SongData,
    pub score: beatoraja_core::score_data::ScoreData,
    pub retry: i32,
}

impl IRSendStatusMain {
    pub fn new(
        connection: IRConnection,
        songdata: &SongData,
        score: &beatoraja_core::score_data::ScoreData,
    ) -> Self {
        Self {
            connection,
            songdata: songdata.clone(),
            score: score.clone(),
            retry: 0,
        }
    }

    pub fn send(&mut self) -> bool {
        todo!("IR dependency: IRSendStatus.send")
    }
}

// ============================================================
// Input stubs
// ============================================================

/// Stub for bms.player.beatoraja.input.BMSPlayerInputProcessor
pub struct BMSPlayerInputProcessor;

impl BMSPlayerInputProcessor {
    pub fn get_scroll(&self) -> i32 {
        0
    }

    pub fn reset_scroll(&mut self) {
        // stub
    }

    pub fn get_key_state(&self, _index: i32) -> bool {
        false
    }

    pub fn reset_key_changed_time(&mut self, _index: i32) -> bool {
        false
    }

    pub fn reset_all_key_changed_time(&mut self) {
        // stub
    }

    pub fn is_control_key_pressed(&self, _key: ControlKeys) -> bool {
        false
    }

    pub fn is_activated(&self, _command: KeyCommand) -> bool {
        false
    }
}

/// Stub for bms.player.beatoraja.input.KeyBoardInputProcesseor.ControlKeys
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ControlKeys {
    Escape,
    Enter,
    Num1,
    Num2,
    Num3,
    Num4,
    Num5,
    Num6,
    Num7,
    Num8,
    Num9,
    Up,
    Down,
    Left,
    Right,
}

/// Stub for bms.player.beatoraja.input.KeyCommand
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyCommand {
    OpenIr,
}

// ============================================================
// PlayerResource stub
// ============================================================

/// Stub for bms.player.beatoraja.PlayerResource
pub struct PlayerResource {
    pub play_mode: BMSPlayerMode,
}

impl PlayerResource {
    pub fn get_bms_model(&self) -> &bms_model::bms_model::BMSModel {
        todo!("Phase 8+ dependency: PlayerResource.getBMSModel")
    }

    pub fn get_player_config(&self) -> &beatoraja_core::player_config::PlayerConfig {
        todo!("Phase 8+ dependency: PlayerResource.getPlayerConfig")
    }

    pub fn get_config(&self) -> &beatoraja_core::config::Config {
        todo!("Phase 8+ dependency: PlayerResource.getConfig")
    }

    pub fn get_course_bms_models(&self) -> Option<&[bms_model::bms_model::BMSModel]> {
        None
    }

    pub fn get_play_mode(&self) -> &BMSPlayerMode {
        &self.play_mode
    }

    pub fn get_groove_gauge(&self) -> &GrooveGaugeStub {
        todo!("Phase 8+ dependency: PlayerResource.getGrooveGauge")
    }

    pub fn get_gauge(&self) -> &[FloatArray] {
        todo!("Phase 8+ dependency: PlayerResource.getGauge")
    }

    pub fn get_course_gauge(&self) -> &GdxArray<Vec<FloatArray>> {
        todo!("Phase 8+ dependency: PlayerResource.getCourseGauge")
    }

    pub fn get_course_gauge_mut(&mut self) -> &mut GdxArray<Vec<FloatArray>> {
        todo!("Phase 8+ dependency: PlayerResource.getCourseGauge")
    }

    pub fn get_score_data(&self) -> Option<&beatoraja_core::score_data::ScoreData> {
        None
    }

    pub fn get_score_data_mut(&mut self) -> Option<&mut beatoraja_core::score_data::ScoreData> {
        None
    }

    pub fn get_course_score_data(&self) -> Option<&beatoraja_core::score_data::ScoreData> {
        None
    }

    pub fn get_course_score_data_mut(
        &mut self,
    ) -> Option<&mut beatoraja_core::score_data::ScoreData> {
        None
    }

    pub fn set_course_score_data(&mut self, _score: beatoraja_core::score_data::ScoreData) {
        // stub
    }

    pub fn get_ranking_data(&self) -> Option<&RankingData> {
        None
    }

    pub fn set_ranking_data(&mut self, _data: Option<RankingData>) {
        // stub
    }

    pub fn get_replay_data(&self) -> &beatoraja_core::replay_data::ReplayData {
        todo!("Phase 8+ dependency: PlayerResource.getReplayData")
    }

    pub fn get_replay_data_mut(&mut self) -> &mut beatoraja_core::replay_data::ReplayData {
        todo!("Phase 8+ dependency: PlayerResource.getReplayData")
    }

    pub fn get_course_replay(&self) -> &[beatoraja_core::replay_data::ReplayData] {
        todo!("Phase 8+ dependency: PlayerResource.getCourseReplay")
    }

    pub fn get_course_replay_mut(&mut self) -> &mut Vec<beatoraja_core::replay_data::ReplayData> {
        todo!("Phase 8+ dependency: PlayerResource.getCourseReplay")
    }

    pub fn add_course_replay(&mut self, _replay: &beatoraja_core::replay_data::ReplayData) {
        // stub
    }

    pub fn add_course_gauge(&mut self, _gauge: &[FloatArray]) {
        // stub
    }

    pub fn get_maxcombo(&self) -> i32 {
        0
    }

    pub fn get_target_score_data(&self) -> Option<&beatoraja_core::score_data::ScoreData> {
        None
    }

    pub fn is_update_score(&self) -> bool {
        false
    }

    pub fn is_update_course_score(&self) -> bool {
        false
    }

    pub fn is_force_no_ir_send(&self) -> bool {
        false
    }

    pub fn get_course_data(&self) -> &beatoraja_core::course_data::CourseData {
        todo!("Phase 8+ dependency: PlayerResource.getCourseData")
    }

    pub fn get_songdata(&self) -> &SongData {
        todo!("Phase 8+ dependency: PlayerResource.getSongdata")
    }

    pub fn get_org_gauge_option(&self) -> i32 {
        0
    }

    pub fn get_constraint(&self) -> Vec<beatoraja_core::course_data::CourseDataConstraint> {
        vec![]
    }

    pub fn get_course_index(&self) -> usize {
        0
    }

    pub fn get_assist(&self) -> i32 {
        0
    }

    pub fn next_course(&mut self) -> bool {
        false
    }

    pub fn reload_bms_file(&mut self) {
        // stub
    }

    pub fn is_freq_on(&self) -> bool {
        false
    }
}

// ============================================================
// BMSPlayerMode stub
// ============================================================

/// Stub for bms.player.beatoraja.BMSPlayerMode
#[derive(Clone, Debug)]
pub struct BMSPlayerMode {
    pub mode: BMSPlayerModeType,
}

/// Stub for bms.player.beatoraja.BMSPlayerMode.Mode
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BMSPlayerModeType {
    Play,
    Practice,
    Replay,
    ReplayDifferent,
}

// ============================================================
// PlayDataAccessor stub
// ============================================================

/// Stub for bms.player.beatoraja.PlayDataAccessor
pub struct PlayDataAccessor;

impl PlayDataAccessor {
    pub fn exists_replay_data_model(
        &self,
        _model: &bms_model::bms_model::BMSModel,
        _lnmode: i32,
        _index: i32,
    ) -> bool {
        false
    }

    pub fn exists_replay_data_course(
        &self,
        _models: &[bms_model::bms_model::BMSModel],
        _lnmode: i32,
        _index: i32,
        _constraint: &[beatoraja_core::course_data::CourseDataConstraint],
    ) -> bool {
        false
    }

    pub fn read_score_data(
        &self,
        _model: &bms_model::bms_model::BMSModel,
        _lnmode: i32,
    ) -> Option<beatoraja_core::score_data::ScoreData> {
        None
    }

    pub fn read_score_data_course(
        &self,
        _models: &[bms_model::bms_model::BMSModel],
        _lnmode: i32,
        _random: i32,
        _constraint: &[beatoraja_core::course_data::CourseDataConstraint],
    ) -> Option<beatoraja_core::score_data::ScoreData> {
        None
    }

    pub fn write_score_data(
        &self,
        _score: &beatoraja_core::score_data::ScoreData,
        _model: &bms_model::bms_model::BMSModel,
        _lnmode: i32,
        _update: bool,
    ) {
        // stub
    }

    pub fn write_score_data_course(
        &self,
        _score: &beatoraja_core::score_data::ScoreData,
        _models: &[bms_model::bms_model::BMSModel],
        _lnmode: i32,
        _random: i32,
        _constraint: &[beatoraja_core::course_data::CourseDataConstraint],
        _update: bool,
    ) {
        // stub
    }

    pub fn write_replay_data(
        &self,
        _replay: &beatoraja_core::replay_data::ReplayData,
        _model: &bms_model::bms_model::BMSModel,
        _lnmode: i32,
        _index: i32,
    ) {
        // stub
    }

    pub fn write_replay_data_course(
        &self,
        _replays: &[beatoraja_core::replay_data::ReplayData],
        _models: &[bms_model::bms_model::BMSModel],
        _lnmode: i32,
        _index: i32,
        _constraint: &[beatoraja_core::course_data::CourseDataConstraint],
    ) {
        // stub
    }
}

// ============================================================
// AudioProcessor stub
// ============================================================

/// Stub for AudioProcessor
pub struct AudioProcessor;

impl AudioProcessor {
    pub fn stop_note(&self, _note: Option<()>) {
        // stub
    }
}

// ============================================================
// SongData stub
// ============================================================

/// Stub for bms.player.beatoraja.song.SongData
#[derive(Clone, Debug, Default)]
pub struct SongData {
    pub notes: i32,
}

impl SongData {
    pub fn get_notes(&self) -> i32 {
        self.notes
    }
}

// ============================================================
// GrooveGauge stub (partial)
// ============================================================

/// Stub for bms.player.beatoraja.play.GrooveGauge
pub struct GrooveGaugeStub {
    pub gauge_type: i32,
}

impl GrooveGaugeStub {
    pub fn get_type(&self) -> i32 {
        self.gauge_type
    }

    pub fn get_gauge_type_length(&self) -> usize {
        9
    }

    pub fn get_gauge(&self, _gauge_type: i32) -> &beatoraja_play::groove_gauge::Gauge {
        todo!("Phase 8+ dependency: GrooveGauge.getGauge")
    }

    pub fn get_clear_type(&self) -> beatoraja_core::clear_type::ClearType {
        todo!("Phase 8+ dependency: GrooveGauge.getClearType")
    }
}

// ============================================================
// FloatArray (LibGDX) stub
// ============================================================

/// Stub for com.badlogic.gdx.utils.FloatArray
#[derive(Clone, Debug, Default)]
pub struct FloatArray {
    pub items: Vec<f32>,
    pub size: usize,
}

impl FloatArray {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            size: 0,
        }
    }

    pub fn add(&mut self, value: f32) {
        self.items.push(value);
        self.size = self.items.len();
    }

    pub fn get(&self, index: usize) -> f32 {
        self.items[index]
    }

    pub fn add_all(&mut self, other: &FloatArray) {
        self.items.extend_from_slice(&other.items);
        self.size = self.items.len();
    }
}

/// Stub for com.badlogic.gdx.utils.IntArray
#[derive(Clone, Debug, Default)]
pub struct IntArray {
    pub items: Vec<i32>,
    pub size: usize,
}

impl IntArray {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            size: 0,
        }
    }

    pub fn add(&mut self, value: i32) {
        self.items.push(value);
        self.size = self.items.len();
    }

    pub fn get(&self, index: usize) -> i32 {
        self.items[index]
    }

    pub fn contains(&self, value: i32) -> bool {
        self.items.contains(&value)
    }
}

/// Stub for com.badlogic.gdx.utils.Array<T>
#[derive(Clone, Debug, Default)]
pub struct GdxArray<T> {
    pub items: Vec<T>,
    pub size: usize,
}

impl<T> GdxArray<T> {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            size: 0,
        }
    }

    pub fn add(&mut self, value: T) {
        self.items.push(value);
        self.size = self.items.len();
    }

    pub fn get(&self, index: usize) -> &T {
        &self.items[index]
    }
}

impl<T> IntoIterator for GdxArray<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter()
    }
}

impl<'a, T> IntoIterator for &'a GdxArray<T> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.iter()
    }
}

// ============================================================
// Skin-related stubs
// ============================================================

/// Stub for beatoraja.skin.Skin (base class for result skins)
pub struct Skin {
    pub header: SkinHeader,
    pub input: i32,
    pub scene: i32,
    pub fadeout: i32,
}

impl Skin {
    pub fn new(_header: SkinHeader) -> Self {
        Self {
            header: _header,
            input: 0,
            scene: 0,
            fadeout: 0,
        }
    }

    pub fn get_input(&self) -> i32 {
        self.input
    }

    pub fn get_scene(&self) -> i32 {
        self.scene
    }

    pub fn get_fadeout(&self) -> i32 {
        self.fadeout
    }
}

/// Stub for SkinHeader
#[derive(Clone, Debug, Default)]
pub struct SkinHeader;

// ============================================================
// SkinObject stubs (for SkinGaugeGraphObject)
// ============================================================

/// Stub for com.badlogic.gdx.graphics.Color
#[derive(Clone, Debug)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Default for Color {
    fn default() -> Self {
        Self {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        }
    }
}

impl Color {
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub fn value_of(hex: &str) -> Self {
        let hex = hex.trim();
        let len = hex.len();
        if len < 6 {
            return Color::new(1.0, 0.0, 0.0, 1.0);
        }
        let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(255) as f32 / 255.0;
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0) as f32 / 255.0;
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0) as f32 / 255.0;
        let a = if len >= 8 {
            u8::from_str_radix(&hex[6..8], 16).unwrap_or(255) as f32 / 255.0
        } else {
            1.0
        };
        Color::new(r, g, b, a)
    }
}

/// Stub for com.badlogic.gdx.graphics.g2d.TextureRegion
#[derive(Clone, Debug, Default)]
pub struct TextureRegion {
    pub texture: Option<Texture>,
    pub region_width: i32,
    pub region_height: i32,
}

impl TextureRegion {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_texture(texture: Texture) -> Self {
        Self {
            region_width: texture.width,
            region_height: texture.height,
            texture: Some(texture),
        }
    }

    pub fn get_texture(&self) -> Option<&Texture> {
        self.texture.as_ref()
    }

    pub fn set_region(&mut self, _x: i32, _y: i32, _width: i32, _height: i32) {
        // stub
    }
}

/// Stub for com.badlogic.gdx.graphics.Texture
#[derive(Clone, Debug, Default)]
pub struct Texture {
    pub width: i32,
    pub height: i32,
}

impl Texture {
    pub fn from_pixmap(pixmap: &Pixmap) -> Self {
        Self {
            width: pixmap.width,
            height: pixmap.height,
        }
    }

    pub fn get_width(&self) -> i32 {
        self.width
    }

    pub fn get_height(&self) -> i32 {
        self.height
    }

    pub fn dispose(&mut self) {
        // stub
    }
}

/// Stub for com.badlogic.gdx.graphics.Pixmap
#[derive(Clone, Debug, Default)]
pub struct Pixmap {
    pub width: i32,
    pub height: i32,
}

impl Pixmap {
    pub fn new(width: i32, height: i32) -> Self {
        Self { width, height }
    }

    pub fn set_color(&mut self, _color: &Color) {
        // stub
    }

    pub fn fill(&mut self) {
        // stub
    }

    pub fn fill_rectangle(&mut self, _x: i32, _y: i32, _width: i32, _height: i32) {
        // stub
    }

    pub fn draw_line(&mut self, _x1: i32, _y1: i32, _x2: i32, _y2: i32) {
        // stub
    }

    pub fn dispose(&mut self) {
        // stub
    }
}

/// Stub for SkinObject base (partial — only what SkinGaugeGraphObject needs)
pub struct SkinObjectData {
    pub region: Rectangle,
}

/// Stub for com.badlogic.gdx.math.Rectangle
#[derive(Clone, Debug, Default)]
pub struct Rectangle {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

/// Stub for SkinObjectRenderer
pub struct SkinObjectRenderer;

impl SkinObjectRenderer {
    pub fn draw(&mut self, _tex: &TextureRegion, _x: f32, _y: f32, _w: f32, _h: f32) {
        // stub
    }
}

// ============================================================
// MainState shared data stubs (needed by AbstractResult)
// ============================================================

/// Stub for TimerManager (used by AbstractResult via MainState)
pub struct TimerManager;

impl TimerManager {
    pub fn get_now_time(&self) -> i64 {
        0
    }

    pub fn get_now_time_for(&self, _timer_id: i32) -> i64 {
        0
    }

    pub fn switch_timer(&mut self, _timer_id: i32, _on: bool) {
        // stub
    }

    pub fn is_timer_on(&self, _timer_id: i32) -> bool {
        false
    }

    pub fn get_timer(&self, _timer_id: i32) -> i64 {
        0
    }
}

// ============================================================
// EventFactory stub
// ============================================================

/// Stub for bms.player.beatoraja.skin.property.EventFactory.EventType
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EventType {
    #[allow(non_camel_case_types)]
    open_ir,
}

// ============================================================
// FreqTrainerMenu stub
// ============================================================

pub fn is_freq_trainer_enabled() -> bool {
    false
}

pub fn is_freq_negative() -> bool {
    false
}
