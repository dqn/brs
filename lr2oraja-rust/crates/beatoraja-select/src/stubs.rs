// External dependency stubs for beatoraja-select
// Types that can be replaced with real implementations are re-exported from beatoraja-core.
// Remaining stubs are for types that cannot be replaced due to API incompatibilities.

use std::collections::HashMap;

// ============================================================
// LibGDX types — re-exported from beatoraja-skin stubs
// ============================================================

pub use beatoraja_skin::stubs::Color;
pub use beatoraja_skin::stubs::Pixmap;
pub use beatoraja_skin::stubs::Rectangle;
pub use beatoraja_skin::stubs::TextureRegion;

// ============================================================
// beatoraja core types — re-exported from real implementations
// ============================================================

pub use beatoraja_core::audio_config::AudioConfig;
pub use beatoraja_core::config::{Config, SongPreview};
pub use beatoraja_core::play_config::PlayConfig;
pub use beatoraja_core::play_mode_config::{
    ControllerConfig, KeyboardConfig, MidiConfig, PlayModeConfig,
};
pub use beatoraja_core::player_config::PlayerConfig;
pub use beatoraja_core::resolution::Resolution;
pub use beatoraja_core::score_data::ScoreData;

// ============================================================
// beatoraja.song types
// ============================================================

/// Stub for beatoraja.song.SongData
/// The actual SongData lives in beatoraja-core, but the select module
/// uses many more fields. We replicate the full API here as stubs.
#[derive(Clone, Debug, Default)]
pub struct SongData {
    pub sha256: String,
    pub md5: String,
    pub title: String,
    pub subtitle: String,
    pub artist: String,
    pub subartist: String,
    pub genre: String,
    pub path: Option<String>,
    pub folder: String,
    pub url: Option<String>,
    pub appendurl: Option<String>,
    pub banner: String,
    pub stagefile: String,
    pub preview: Option<String>,
    pub mode: i32,
    pub level: i32,
    pub difficulty: i32,
    pub maxbpm: i32,
    pub minbpm: i32,
    pub length: i32,
    pub notes: i32,
    pub favorite: i32,
    pub adddate: i64,
    pub feature: i32,
    pub ipfs: Option<String>,
    pub bms_model: Option<BMSModelStub>,
}

impl SongData {
    pub const FAVORITE_CHART: i32 = 1;
    pub const FAVORITE_SONG: i32 = 2;
    pub const INVISIBLE_SONG: i32 = 4;
    pub const INVISIBLE_CHART: i32 = 8;
    pub const FEATURE_UNDEFINEDLN: i32 = 1;
    pub const FEATURE_LONGNOTE: i32 = 2;
    pub const FEATURE_CHARGENOTE: i32 = 4;
    pub const FEATURE_HELLCHARGENOTE: i32 = 8;
    pub const FEATURE_MINENOTE: i32 = 16;
    pub const FEATURE_RANDOM: i32 = 32;

    pub fn get_sha256(&self) -> &str {
        &self.sha256
    }
    pub fn set_sha256(&mut self, s: String) {
        self.sha256 = s;
    }
    pub fn get_md5(&self) -> &str {
        &self.md5
    }
    pub fn set_md5(&mut self, s: String) {
        self.md5 = s;
    }
    pub fn get_title(&self) -> &str {
        &self.title
    }
    pub fn set_title(&mut self, s: String) {
        self.title = s;
    }
    pub fn get_subtitle(&self) -> &str {
        &self.subtitle
    }
    pub fn get_artist(&self) -> &str {
        &self.artist
    }
    pub fn set_artist(&mut self, s: String) {
        self.artist = s;
    }
    pub fn get_genre(&self) -> &str {
        &self.genre
    }
    pub fn set_genre(&mut self, s: String) {
        self.genre = s;
    }
    pub fn get_path(&self) -> Option<&str> {
        self.path.as_deref()
    }
    pub fn set_path(&mut self, p: Option<String>) {
        self.path = p;
    }
    pub fn get_folder(&self) -> &str {
        &self.folder
    }
    pub fn get_url(&self) -> Option<&str> {
        self.url.as_deref()
    }
    pub fn set_url(&mut self, s: String) {
        self.url = Some(s);
    }
    pub fn get_appendurl(&self) -> Option<&str> {
        self.appendurl.as_deref()
    }
    pub fn set_appendurl(&mut self, s: String) {
        self.appendurl = Some(s);
    }
    pub fn get_banner(&self) -> &str {
        &self.banner
    }
    pub fn get_stagefile(&self) -> &str {
        &self.stagefile
    }
    pub fn get_preview(&self) -> Option<&str> {
        self.preview.as_deref()
    }
    pub fn get_mode(&self) -> i32 {
        self.mode
    }
    pub fn set_mode(&mut self, m: i32) {
        self.mode = m;
    }
    pub fn get_level(&self) -> i32 {
        self.level
    }
    pub fn get_difficulty(&self) -> i32 {
        self.difficulty
    }
    pub fn get_maxbpm(&self) -> i32 {
        self.maxbpm
    }
    pub fn get_minbpm(&self) -> i32 {
        self.minbpm
    }
    pub fn get_length(&self) -> i32 {
        self.length
    }
    pub fn get_notes(&self) -> i32 {
        self.notes
    }
    pub fn get_favorite(&self) -> i32 {
        self.favorite
    }
    pub fn set_favorite(&mut self, f: i32) {
        self.favorite = f;
    }
    pub fn get_adddate(&self) -> i64 {
        self.adddate
    }
    pub fn get_feature(&self) -> i32 {
        self.feature
    }
    pub fn get_ipfs(&self) -> Option<&str> {
        self.ipfs.as_deref()
    }

    pub fn get_full_title(&self) -> String {
        if self.subtitle.is_empty() {
            self.title.clone()
        } else {
            format!("{} {}", self.title, self.subtitle)
        }
    }

    pub fn has_undefined_long_note(&self) -> bool {
        (self.feature & Self::FEATURE_UNDEFINEDLN) != 0
    }

    pub fn merge(&mut self, other: &SongData) {
        if !other.title.is_empty() && self.title.is_empty() {
            self.title = other.title.clone();
        }
        if !other.artist.is_empty() && self.artist.is_empty() {
            self.artist = other.artist.clone();
        }
    }

    pub fn get_bms_model(&self) -> Option<&BMSModelStub> {
        self.bms_model.as_ref()
    }

    pub fn set_bms_model(&mut self, model: Option<BMSModelStub>) {
        self.bms_model = model;
    }
}

/// Stub for bms.model.BMSModel
#[derive(Clone, Debug, Default)]
pub struct BMSModelStub {
    pub mode: ModeStub,
}

impl BMSModelStub {
    pub fn get_mode(&self) -> &ModeStub {
        &self.mode
    }
}

/// Stub for bms.model.Mode (used for mode comparison)
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct ModeStub {
    pub id: i32,
}

// ============================================================
// beatoraja.song.FolderData
// ============================================================

/// Stub for beatoraja.song.FolderData
#[derive(Clone, Debug, Default)]
pub struct FolderData {
    pub title: String,
    pub path: String,
    pub adddate: i64,
}

impl FolderData {
    pub fn get_title(&self) -> &str {
        &self.title
    }
    pub fn get_path(&self) -> &str {
        &self.path
    }
    pub fn get_adddate(&self) -> i64 {
        self.adddate
    }
}

// ============================================================
// beatoraja.song.SongDatabaseAccessor
// ============================================================

/// Stub for beatoraja.song.SongDatabaseAccessor
#[derive(Clone, Debug, Default)]
pub struct SongDatabaseAccessor;

impl SongDatabaseAccessor {
    pub fn get_song_datas_by_key(&self, _key: &str, _value: &str) -> Vec<SongData> {
        todo!("SongDatabaseAccessor.getSongDatas")
    }

    pub fn get_song_datas_by_sql(
        &self,
        _sql: &str,
        _score_db: &str,
        _scorelog_db: &str,
        _info_db: Option<&str>,
    ) -> Vec<SongData> {
        todo!("SongDatabaseAccessor.getSongDatas(sql)")
    }

    pub fn get_song_datas_by_hashes(&self, _hashes: &[String]) -> Vec<SongData> {
        todo!("SongDatabaseAccessor.getSongDatas(hashes)")
    }

    pub fn get_song_datas_by_text(&self, _text: &str) -> Vec<SongData> {
        todo!("SongDatabaseAccessor.getSongDatasByText")
    }

    pub fn get_folder_datas(&self, _key: &str, _value: &str) -> Vec<FolderData> {
        todo!("SongDatabaseAccessor.getFolderDatas")
    }

    pub fn set_song_datas(&self, _songs: &[SongData]) {
        todo!("SongDatabaseAccessor.setSongDatas")
    }
}

// ============================================================
// beatoraja.song.SongUtils
// ============================================================

pub struct SongUtils;

impl SongUtils {
    pub fn crc32(_path: &str, _ext: &[&str], _root: &str) -> String {
        todo!("SongUtils.crc32")
    }
}

// ============================================================
// beatoraja.song.SongInformationAccessor
// ============================================================

/// Stub for beatoraja.song.SongInformationAccessor
pub struct SongInformationAccessor;

impl SongInformationAccessor {
    pub fn get_information(&self, _songs: &[SongData]) {
        todo!("SongInformationAccessor.getInformation")
    }
}

// ============================================================
// beatoraja core types (stubbed — cannot be replaced)
// ============================================================

/// Stub for beatoraja.MainController
#[derive(Debug, Default)]
pub struct MainController;

impl MainController {
    pub fn get_config(&self) -> &Config {
        todo!()
    }
    pub fn get_player_config(&self) -> &PlayerConfig {
        todo!()
    }
    pub fn get_song_database(&self) -> &SongDatabaseAccessor {
        todo!()
    }
    pub fn get_info_database(&self) -> Option<&SongInformationAccessor> {
        todo!()
    }
    pub fn get_play_data_accessor(&self) -> &PlayDataAccessor {
        todo!()
    }
    pub fn get_rival_data_accessor(&self) -> &RivalDataAccessor {
        todo!()
    }
    pub fn get_ir_status(&self) -> &[IRStatus] {
        todo!()
    }
    pub fn get_ranking_data_cache(&self) -> &RankingDataCache {
        todo!()
    }
    pub fn get_input_processor(&self) -> &BMSPlayerInputProcessor {
        todo!()
    }
    pub fn get_sound_manager(&self) -> &SystemSoundManager {
        todo!()
    }
    pub fn get_player_resource(&self) -> &PlayerResource {
        todo!()
    }
    pub fn get_current_state(&self) -> &dyn MainState {
        todo!()
    }
    pub fn get_music_download_processor(&self) -> Option<&MusicDownloadProcessor> {
        todo!()
    }
    pub fn get_http_download_processor(&self) -> Option<&HttpDownloadProcessor> {
        todo!()
    }
    pub fn change_state(&self, _state: MainStateType) {
        todo!()
    }
    pub fn update_song(&self, _path: Option<&str>) {
        todo!()
    }
    pub fn exit(&self) {
        todo!()
    }
}

/// Stub for beatoraja.MainState
pub trait MainState {
    fn get_main(&self) -> &MainController;
}

/// Stub for beatoraja.MainStateType
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MainStateType {
    SelectConfig,
    SkinConfig,
    Decide,
}

/// Stub for beatoraja.ScoreDatabaseAccessor.ScoreDataCollector
pub trait ScoreDataCollector: Fn(&SongData, Option<&ScoreData>) {}
impl<F: Fn(&SongData, Option<&ScoreData>)> ScoreDataCollector for F {}

/// Stub for beatoraja.BMSPlayerMode
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BMSPlayerMode {
    pub mode: BMSPlayerModeType,
    pub id: i32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BMSPlayerModeType {
    Play,
    AutoPlay,
    Practice,
    Replay,
}

impl BMSPlayerMode {
    pub const PLAY: BMSPlayerMode = BMSPlayerMode {
        mode: BMSPlayerModeType::Play,
        id: 0,
    };
    pub const AUTOPLAY: BMSPlayerMode = BMSPlayerMode {
        mode: BMSPlayerModeType::AutoPlay,
        id: 1,
    };
    pub const PRACTICE: BMSPlayerMode = BMSPlayerMode {
        mode: BMSPlayerModeType::Practice,
        id: 2,
    };

    pub fn get_replay_mode(index: i32) -> BMSPlayerMode {
        BMSPlayerMode {
            mode: BMSPlayerModeType::Replay,
            id: index + 3,
        }
    }
}

/// Stub for beatoraja.CourseData
/// Cannot be replaced: field name differs (song vs hash), TrophyData types differ
#[derive(Clone, Debug, Default)]
pub struct CourseData {
    pub name: String,
    pub song: Vec<SongData>,
    pub constraint: Vec<CourseDataConstraint>,
    pub trophy: Vec<TrophyData>,
    pub release: bool,
}

impl CourseData {
    pub fn get_name(&self) -> &str {
        &self.name
    }
    pub fn set_name(&mut self, s: String) {
        self.name = s;
    }
    pub fn get_song(&self) -> &[SongData] {
        &self.song
    }
    pub fn set_song(&mut self, s: Vec<SongData>) {
        self.song = s;
    }
    pub fn get_constraint(&self) -> &[CourseDataConstraint] {
        &self.constraint
    }
    pub fn set_constraint(&mut self, c: Vec<CourseDataConstraint>) {
        self.constraint = c;
    }
    pub fn get_trophy(&self) -> &[TrophyData] {
        &self.trophy
    }
    pub fn set_trophy(&mut self, t: Vec<TrophyData>) {
        self.trophy = t;
    }
    pub fn set_release(&mut self, r: bool) {
        self.release = r;
    }
    pub fn set_course_song_models(&mut self, _models: &[SongData]) { /* stub */
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize)]
pub enum CourseDataConstraint {
    Class,
    Mirror,
    Random,
    Ln,
    Cn,
    Hcn,
    NoSpeed,
    NoGood,
    NoGreat,
}

#[derive(Clone, Debug, Default)]
pub struct TrophyData {
    pub name: String,
    pub missrate: f64,
    pub scorerate: f64,
}

impl TrophyData {
    pub fn get_name(&self) -> &str {
        &self.name
    }
    pub fn set_name(&mut self, s: String) {
        self.name = s;
    }
    pub fn get_missrate(&self) -> f64 {
        self.missrate
    }
    pub fn set_missrate(&mut self, v: f64) {
        self.missrate = v;
    }
    pub fn get_scorerate(&self) -> f64 {
        self.scorerate
    }
    pub fn set_scorerate(&mut self, v: f64) {
        self.scorerate = v;
    }
}

/// Stub for beatoraja.RandomCourseData
#[derive(Clone, Debug, Default, serde::Deserialize)]
pub struct RandomCourseData {
    pub name: String,
    pub stage: Vec<RandomStageData>,
    pub constraint: Vec<CourseDataConstraint>,
}

impl RandomCourseData {
    pub const EMPTY: &'static [RandomCourseData] = &[];

    pub fn get_name(&self) -> &str {
        &self.name
    }
    pub fn get_stage(&self) -> &[RandomStageData] {
        &self.stage
    }
    pub fn get_song_datas(&self) -> Vec<SongData> {
        todo!()
    }
    pub fn lottery_song_datas(&self, _main: &MainController) {
        todo!()
    }
    pub fn create_course_data(&self) -> CourseData {
        todo!()
    }
}

/// Stub for beatoraja.RandomStageData
#[derive(Clone, Debug, Default, serde::Deserialize)]
pub struct RandomStageData;

/// Stub for beatoraja.TableData
#[derive(Clone, Debug, Default)]
pub struct TableData {
    pub name: String,
    pub url: Option<String>,
    pub folder: Vec<TableFolder>,
    pub course: Vec<CourseData>,
}

impl TableData {
    pub fn get_name(&self) -> &str {
        &self.name
    }
    pub fn set_name(&mut self, s: String) {
        self.name = s;
    }
    pub fn get_url(&self) -> Option<&str> {
        self.url.as_deref()
    }
    pub fn set_url(&mut self, u: String) {
        self.url = Some(u);
    }
    pub fn get_folder(&self) -> &[TableFolder] {
        &self.folder
    }
    pub fn set_folder(&mut self, f: Vec<TableFolder>) {
        self.folder = f;
    }
    pub fn get_course(&self) -> &[CourseData] {
        &self.course
    }
    pub fn set_course(&mut self, c: Vec<CourseData>) {
        self.course = c;
    }
    pub fn validate(&self) -> bool {
        true
    }
}

#[derive(Clone, Debug, Default)]
pub struct TableFolder {
    pub name: String,
    pub song: Vec<SongData>,
}

impl TableFolder {
    pub fn get_name(&self) -> &str {
        &self.name
    }
    pub fn set_name(&mut self, s: String) {
        self.name = s;
    }
    pub fn get_song(&self) -> &[SongData] {
        &self.song
    }
    pub fn set_song(&mut self, s: Vec<SongData>) {
        self.song = s;
    }
}

/// Stub for beatoraja.TableDataAccessor
#[derive(Clone, Debug, Default)]
pub struct TableDataAccessor {
    pub tablepath: String,
}

impl TableDataAccessor {
    pub fn new(tablepath: &str) -> Self {
        Self {
            tablepath: tablepath.to_string(),
        }
    }
    pub fn read_all(&self) -> Vec<TableData> {
        todo!()
    }
    pub fn write(&self, _td: &TableData) {
        todo!()
    }
}

/// Stub for TableDataAccessor.TableAccessor trait
pub trait TableAccessor: Send + Sync {
    fn name(&self) -> &str;
    fn read(&self) -> TableData;
    fn write(&self, td: &TableData);
}

/// Stub for DifficultyTableAccessor
pub struct DifficultyTableAccessor {
    pub tablepath: String,
    pub url: String,
}

impl DifficultyTableAccessor {
    pub fn new(tablepath: &str, url: &str) -> Self {
        Self {
            tablepath: tablepath.to_string(),
            url: url.to_string(),
        }
    }
}

impl TableAccessor for DifficultyTableAccessor {
    fn name(&self) -> &str {
        &self.url
    }
    fn read(&self) -> TableData {
        todo!()
    }
    fn write(&self, _td: &TableData) { /* stub */
    }
}

/// Stub for beatoraja.CourseDataAccessor
pub struct CourseDataAccessor {
    pub path: String,
}

impl CourseDataAccessor {
    pub fn new(path: &str) -> Self {
        Self {
            path: path.to_string(),
        }
    }
    pub fn read_all(&self) -> Vec<CourseData> {
        todo!()
    }
}

/// Stub for beatoraja.PlayDataAccessor
pub struct PlayDataAccessor;

impl PlayDataAccessor {
    pub fn read_score_data_single(
        &self,
        _hash: &str,
        _ln: bool,
        _lnmode: i32,
    ) -> Option<ScoreData> {
        todo!()
    }
    pub fn read_score_data_multi(
        &self,
        _hashes: &[String],
        _ln: bool,
        _lnmode: i32,
        _mode: i32,
        _constraints: &[CourseDataConstraint],
    ) -> Option<ScoreData> {
        todo!()
    }
    pub fn read_score_datas(
        &self,
        _collector: &dyn Fn(&SongData, Option<&ScoreData>),
        _songs: &[SongData],
        _lnmode: i32,
    ) {
        todo!()
    }
    pub fn exists_replay_data_single(
        &self,
        _hash: &str,
        _ln: bool,
        _lnmode: i32,
        _index: i32,
    ) -> bool {
        todo!()
    }
    pub fn exists_replay_data_multi(
        &self,
        _hashes: &[String],
        _ln: bool,
        _lnmode: i32,
        _index: i32,
        _constraints: &[CourseDataConstraint],
    ) -> bool {
        todo!()
    }
    pub fn read_replay_data(&self, _model: &(), _lnmode: i32, _id: i32) -> Option<ReplayData> {
        todo!()
    }
    pub fn read_player_data(&self) -> PlayerData {
        todo!()
    }
}

/// Stub for beatoraja.ReplayData
/// Cannot be replaced: real type references KeyInputLog/PatternModifyLog stubs
#[derive(Clone, Debug, Default)]
pub struct ReplayData {
    pub randomoption: i32,
    pub randomoptionseed: i64,
    pub randomoption2: i32,
    pub randomoption2seed: i64,
    pub doubleoption: i32,
    pub rand: i32,
}

/// Stub for beatoraja.PlayerData
#[derive(Clone, Debug, Default)]
pub struct PlayerData;

/// Stub for beatoraja.RivalDataAccessor
pub struct RivalDataAccessor;

impl RivalDataAccessor {
    pub fn get_rival_count(&self) -> i32 {
        0
    }
    pub fn get_rival_information(&self, _index: i32) -> Option<&PlayerInformation> {
        None
    }
    pub fn get_rival_score_data_cache(&self, _index: i32) -> Option<&ScoreDataCacheStub> {
        None
    }
}

/// Stub for beatoraja.PlayerInformation
#[derive(Clone, Debug, Default)]
pub struct PlayerInformation {
    pub name: String,
}

impl PlayerInformation {
    pub fn get_name(&self) -> &str {
        &self.name
    }
}

/// Stub for score data cache (abstract in Java)
pub struct ScoreDataCacheStub;

/// Stub for beatoraja.PlayerResource
pub struct PlayerResource;

impl PlayerResource {
    pub fn get_config(&self) -> &Config {
        todo!()
    }
    pub fn get_player_config(&self) -> &PlayerConfig {
        todo!()
    }
}

/// Stub for beatoraja.RankingData
pub struct RankingData {
    pub state: i32,
    pub last_update_time: i64,
    pub total_player: i32,
}

impl Default for RankingData {
    fn default() -> Self {
        Self::new()
    }
}

impl RankingData {
    pub const ACCESS: i32 = 0;
    pub const FINISH: i32 = 1;
    pub const FAIL: i32 = 2;

    pub fn new() -> Self {
        Self {
            state: 0,
            last_update_time: 0,
            total_player: 0,
        }
    }

    pub fn get_state(&self) -> i32 {
        self.state
    }
    pub fn get_last_update_time(&self) -> i64 {
        self.last_update_time
    }
    pub fn get_total_player(&self) -> i32 {
        self.total_player
    }
    pub fn load_song(&self, _selector: &dyn MainState, _song: &SongData) {
        todo!()
    }
    pub fn load_course(&self, _selector: &dyn MainState, _course: &CourseData) {
        todo!()
    }
}

/// Stub for beatoraja.RankingDataCache
pub struct RankingDataCache;

impl RankingDataCache {
    pub fn get_song(&self, _song: &SongData, _lnmode: i32) -> Option<&RankingData> {
        None
    }
    pub fn get_course(&self, _course: &CourseData, _lnmode: i32) -> Option<&RankingData> {
        None
    }
    pub fn put_song(&self, _song: &SongData, _lnmode: i32, _data: RankingData) {
        todo!()
    }
    pub fn put_course(&self, _course: &CourseData, _lnmode: i32, _data: RankingData) {
        todo!()
    }
}

/// Stub for beatoraja.PixmapResourcePool
pub struct PixmapResourcePool;

impl PixmapResourcePool {
    pub fn new(_gen: i32) -> Self {
        Self
    }
    pub fn get(&self, _path: &str) -> Option<Pixmap> {
        todo!()
    }
    pub fn dispose(&self) {}
    pub fn dispose_old(&self) {}
}

// ============================================================
// beatoraja.input types
// ============================================================

/// Stub for beatoraja.input.BMSPlayerInputProcessor
pub struct BMSPlayerInputProcessor;

impl BMSPlayerInputProcessor {
    pub fn get_key_state(&self, _key: i32) -> bool {
        false
    }
    pub fn is_analog_input(&self, _key: i32) -> bool {
        false
    }
    pub fn get_analog_diff_and_reset(&self, _key: i32, _threshold: i32) -> i32 {
        0
    }
    pub fn reset_key_changed_time(&self, _key: i32) -> bool {
        false
    }
    pub fn start_pressed(&self) -> bool {
        false
    }
    pub fn is_select_pressed(&self) -> bool {
        false
    }
    pub fn get_scroll(&self) -> i32 {
        0
    }
    pub fn reset_scroll(&self) {}
    pub fn get_control_key_state(&self, _key: ControlKeys) -> bool {
        false
    }
    pub fn is_control_key_pressed(&self, _key: ControlKeys) -> bool {
        false
    }
    pub fn is_activated(&self, _cmd: KeyCommand) -> bool {
        false
    }
    pub fn set_keyboard_config(&self, _config: &KeyboardConfig) {}
    pub fn set_controller_config(&self, _config: &ControllerConfig) {}
    pub fn set_midi_config(&self, _config: &MidiConfig) {}
    pub fn get_keyboard_input_processor(&self) -> &KeyBoardInputProcessor {
        todo!()
    }
}

/// Stub for ControlKeys
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ControlKeys {
    Num0,
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
    Enter,
    Escape,
}

/// Stub for KeyCommand
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum KeyCommand {
    OpenSkinConfiguration,
    AutoplayFolder,
    OpenIr,
    AddFavoriteSong,
    AddFavoriteChart,
    UpdateFolder,
    OpenExplorer,
    CopySongMd5Hash,
    CopySongSha256Hash,
    CopyHighlightedMenuText,
}

/// Stub for KeyBoardInputProcessor
pub struct KeyBoardInputProcessor;

impl KeyBoardInputProcessor {
    pub fn set_text_input_mode(&self, _mode: bool) {}
}

// ============================================================
// beatoraja.ir types
// ============================================================

/// Stub for beatoraja.MainController.IRStatus
pub struct IRStatus {
    pub connection: IRConnection,
    pub player: IRPlayerData,
}

/// Stub for IR connection
pub struct IRConnection;

impl IRConnection {
    pub fn get_play_data(
        &self,
        _player: &IRPlayerData,
        _chart: &IRChartData,
    ) -> IRResponse<Vec<IRScoreData>> {
        todo!()
    }
    pub fn get_table_datas(&self) -> IRResponse<Vec<IRTableData>> {
        todo!()
    }
}

/// Stub for IRResponse
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

/// Stub for IRPlayerData
pub struct IRPlayerData;

/// Stub for IRChartData
pub struct IRChartData;

impl IRChartData {
    pub fn new(_song: &SongData) -> Self {
        Self
    }
}

/// Stub for IRScoreData
#[derive(Clone, Debug, Default)]
pub struct IRScoreData {
    pub player: String,
    pub clear: ClearType,
    pub exscore: i32,
}

impl IRScoreData {
    pub fn get_exscore(&self) -> i32 {
        self.exscore
    }
    pub fn convert_to_score_data(&self) -> ScoreData {
        todo!()
    }
}

/// Stub ClearType for IR types (NOT the real beatoraja-core ClearType enum)
#[derive(Clone, Debug, Default)]
pub struct ClearType {
    pub id: i32,
}

/// Stub for IRTableData
pub struct IRTableData {
    pub name: String,
    pub folders: Vec<IRTableFolder>,
    pub courses: Vec<IRTableCourse>,
}

pub struct IRTableFolder {
    pub name: String,
    pub charts: Vec<IRTableChart>,
}

pub struct IRTableChart {
    pub sha256: String,
    pub md5: String,
    pub title: String,
    pub artist: String,
    pub genre: String,
    pub url: String,
    pub appendurl: String,
    pub mode: Option<bms_model::Mode>,
}

pub struct IRTableCourse {
    pub name: String,
    pub charts: Vec<IRTableChart>,
    pub constraint: Vec<CourseDataConstraint>,
    pub trophy: Vec<IRTableTrophy>,
}

pub struct IRTableTrophy {
    pub name: String,
    pub smissrate: f64,
    pub scorerate: f64,
}

/// Stub for LeaderboardEntry
#[derive(Clone, Debug, Default)]
pub struct LeaderboardEntry {
    pub ir_score: IRScoreData,
    pub lr2_id: i32,
    pub is_lr2ir: bool,
}

impl LeaderboardEntry {
    pub fn new_entry_primary_ir(score: &IRScoreData) -> Self {
        Self {
            ir_score: score.clone(),
            lr2_id: 0,
            is_lr2ir: false,
        }
    }
    pub fn get_ir_score(&self) -> &IRScoreData {
        &self.ir_score
    }
    pub fn get_lr2_id(&self) -> i32 {
        self.lr2_id
    }
    pub fn is_lr2ir(&self) -> bool {
        self.is_lr2ir
    }
}

/// Stub for LR2IRConnection
pub struct LR2IRConnection;

impl LR2IRConnection {
    pub fn get_score_data(_chart: &IRChartData) -> (Option<IRScoreData>, Vec<LeaderboardEntry>) {
        todo!()
    }
    pub fn get_ghost_data(_md5: &str, _lr2_id: i32) -> Option<LR2GhostData> {
        todo!()
    }
}

/// Stub for LR2GhostData
pub struct LR2GhostData {
    pub judgements: Vec<i32>,
    pub pgreat: i32,
    pub great: i32,
    pub good: i32,
    pub bad: i32,
    pub poor: i32,
    pub random: i32,
    pub lane_order: Vec<i32>,
}

impl LR2GhostData {
    pub fn get_judgements(&self) -> &[i32] {
        &self.judgements
    }
    pub fn get_pgreat(&self) -> i32 {
        self.pgreat
    }
    pub fn get_great(&self) -> i32 {
        self.great
    }
    pub fn get_good(&self) -> i32 {
        self.good
    }
    pub fn get_bad(&self) -> i32 {
        self.bad
    }
    pub fn get_poor(&self) -> i32 {
        self.poor
    }
    pub fn get_random(&self) -> i32 {
        self.random
    }
    pub fn get_lane_order(&self) -> &[i32] {
        &self.lane_order
    }
}

// ============================================================
// beatoraja.play types
// ============================================================

/// Stub for GhostBattlePlay
pub struct GhostBattlePlay;

impl GhostBattlePlay {
    pub fn setup(_random: i32, _lane_order: &[i32]) {
        todo!()
    }
}

// ============================================================
// beatoraja.skin types
// ============================================================

/// Stub for beatoraja.skin.SkinType
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SkinType {
    MusicSelect,
}

/// Stub for beatoraja.skin.SkinObject
#[derive(Clone, Debug, Default)]
pub struct SkinObject {
    pub draw: bool,
    pub region: SkinRegion,
}

#[derive(Clone, Debug, Default)]
pub struct SkinRegion {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

/// Stub for beatoraja.skin.SkinImage
#[derive(Clone, Debug, Default)]
pub struct SkinImage {
    pub draw: bool,
    pub region: SkinRegion,
}

impl SkinImage {
    pub fn draw(
        &self,
        _sprite: &SkinObjectRenderer,
        _time: i64,
        _state: &dyn MainState,
        _value: i32,
        _dx: f32,
        _dy: f32,
    ) {
        todo!()
    }
    pub fn draw_offset(&self, _sprite: &SkinObjectRenderer, _dx: f32, _dy: f32) {
        todo!()
    }
    pub fn prepare(&self, _time: i64, _state: &dyn MainState) {}
    pub fn validate(&self) -> bool {
        true
    }
    pub fn get_destination(&self, _time: i64, _state: &dyn MainState) -> Option<Rectangle> {
        None
    }
}

/// Stub for beatoraja.skin.SkinText
#[derive(Clone, Debug, Default)]
pub struct SkinText;

impl SkinText {
    pub fn set_text(&self, _text: &str) {}
    pub fn draw(&self, _sprite: &SkinObjectRenderer, _x: f32, _y: f32) {
        todo!()
    }
    pub fn prepare(&self, _time: i64, _state: &dyn MainState) {}
    pub fn prepare_font(&self, _chars: &str) {}
    pub fn validate(&self) -> bool {
        true
    }
}

/// Stub for beatoraja.skin.SkinNumber
#[derive(Clone, Debug, Default)]
pub struct SkinNumber;

impl SkinNumber {
    pub fn draw(
        &self,
        _sprite: &SkinObjectRenderer,
        _time: i64,
        _value: i32,
        _state: &dyn MainState,
        _x: f32,
        _y: f32,
    ) {
        todo!()
    }
    pub fn prepare(&self, _time: i64, _state: &dyn MainState) {}
    pub fn validate(&self) -> bool {
        true
    }
}

/// Stub for beatoraja.skin.SkinSource
pub trait SkinSource {
    fn get_image(&self, time: i64, state: &dyn MainState) -> Option<TextureRegion>;
}

/// Stub for beatoraja.skin.SkinSourceImage
pub struct SkinSourceImage;

/// Stub for SkinObjectRenderer
pub struct SkinObjectRenderer;

impl SkinObjectRenderer {
    pub fn draw(&self, _image: &Option<TextureRegion>, _x: f32, _y: f32, _w: f32, _h: f32) {
        todo!()
    }
}

/// Stub for beatoraja.skin.SkinHeader
#[derive(Clone, Debug, Default)]
pub struct SkinHeader;

/// Stub for beatoraja.skin.Skin
pub struct SkinStub {
    pub input: i64,
}

impl SkinStub {
    pub fn get_input(&self) -> i64 {
        self.input
    }
}

// ============================================================
// beatoraja.skin.property types
// ============================================================

/// Stub for beatoraja.skin.property.EventFactory.EventType
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EventType {
    Mode,
    Sort,
    Lnmode,
    Option1p,
    Option2p,
    Optiondp,
    Gauge1p,
    Hsfix,
    Target,
    Bga,
    GaugeAutoShift,
    NotesDisplayTiming,
    NotesDisplayTimingAutoAdjust,
    Duration1p,
    Rival,
    OpenDocument,
    OpenWithExplorer,
    OpenIr,
    FavoriteSong,
    FavoriteChart,
    UpdateFolder,
    OpenDownloadSite,
}

/// Stub for beatoraja.skin.property.StringPropertyFactory
pub struct StringPropertyFactory;

impl StringPropertyFactory {
    pub fn get_string_property(_name: &str) -> Box<dyn StringProperty> {
        todo!()
    }
}

pub trait StringProperty {
    fn get(&self, state: &dyn MainState) -> String;
}

// skin_property constants — re-exported from beatoraja-skin
pub use beatoraja_skin::skin_property;

// ============================================================
// beatoraja.SystemSoundManager
// ============================================================

/// Stub for SystemSoundManager
pub struct SystemSoundManager;

// SoundType — re-exported from beatoraja-core
pub use beatoraja_core::system_sound_manager::SoundType;

// ============================================================
// beatoraja.audio types
// ============================================================

/// Stub for AudioDriver
pub struct AudioDriver;

impl AudioDriver {
    pub fn play(&self, _path: &str, _volume: f32, _looping: bool) {}
    pub fn stop(&self, _path: &str) {}
    pub fn dispose(&self, _path: &str) {}
    pub fn is_playing(&self, _path: &str) -> bool {
        false
    }
    pub fn set_volume(&self, _path: &str, _volume: f32) {}
}

// ============================================================
// beatoraja.external types
// ============================================================

/// Stub for beatoraja.external.BMSSearchAccessor
pub struct BMSSearchAccessor;

impl BMSSearchAccessor {
    pub fn new(_tablepath: &str) -> Self {
        Self
    }
    pub fn read(&self) -> Option<TableData> {
        None
    }
}

impl TableAccessor for BMSSearchAccessor {
    fn name(&self) -> &str {
        "BMS Search"
    }
    fn read(&self) -> TableData {
        todo!()
    }
    fn write(&self, _td: &TableData) {}
}

// ============================================================
// beatoraja.modmenu types
// ============================================================

/// Stub for beatoraja.modmenu.ImGuiNotify
pub struct ImGuiNotify;

impl ImGuiNotify {
    pub fn info(_msg: &str) {
        log::info!("{}", _msg);
    }
    pub fn info_with_duration(_msg: &str, _duration: i32) {
        log::info!("{}", _msg);
    }
    pub fn warning(_msg: &str) {
        log::warn!("{}", _msg);
    }
    pub fn error(_msg: &str) {
        log::error!("{}", _msg);
    }
    pub fn error_with_duration(_msg: &str, _duration: i32) {
        log::error!("{}", _msg);
    }
}

/// Stub for beatoraja.modmenu.SongManagerMenu
pub struct SongManagerMenu;

impl SongManagerMenu {
    pub fn is_last_played_sort_enabled() -> bool {
        false
    }
    pub fn force_disable_last_played_sort() {}
}

/// Stub for beatoraja.modmenu.DownloadTaskState
pub struct DownloadTaskState;

impl DownloadTaskState {
    pub fn get_running_download_tasks() -> HashMap<String, DownloadTask> {
        HashMap::new()
    }
}

/// Stub for bms.tool.mdprocessor.DownloadTask
#[derive(Clone, Debug)]
pub struct DownloadTask {
    pub hash: String,
    pub download_size: i64,
    pub content_length: i64,
    pub status: DownloadTaskStatus,
}

impl DownloadTask {
    pub fn get_hash(&self) -> &str {
        &self.hash
    }
    pub fn get_download_size(&self) -> i64 {
        self.download_size
    }
    pub fn get_content_length(&self) -> i64 {
        self.content_length
    }
    pub fn get_download_task_status(&self) -> &DownloadTaskStatus {
        &self.status
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DownloadTaskStatus {
    Prepare,
    Downloading,
    Downloaded,
    Extracted,
    Error,
    Cancel,
}

/// Stub for beatoraja.MusicDownloadProcessor
pub struct MusicDownloadProcessor;

impl MusicDownloadProcessor {
    pub fn is_alive(&self) -> bool {
        false
    }
    pub fn start(&self, _song: &SongData) {
        todo!()
    }
}

/// Stub for bms.tool.mdprocessor.HttpDownloadProcessor
pub struct HttpDownloadProcessor;

impl HttpDownloadProcessor {
    pub fn submit_md5_task(&self, _md5: &str, _title: &str) {
        todo!()
    }
}

// ============================================================
// beatoraja.ScoreDataProperty
// ============================================================

/// Stub for ScoreDataProperty
pub struct ScoreDataProperty;

impl ScoreDataProperty {
    pub fn update(&self, _score: Option<&ScoreData>, _rival_score: Option<&ScoreData>) {}
}

// ============================================================
// beatoraja.MainLoader
// ============================================================

/// Stub for MainLoader
pub struct MainLoader;

impl MainLoader {
    pub fn get_illegal_song_count() -> i32 {
        0
    }
    pub fn get_illegal_songs() -> Vec<SongData> {
        vec![]
    }
}

// ============================================================
// beatoraja.PerformanceMetrics
// ============================================================

/// Stub for PerformanceMetrics
pub struct PerformanceMetrics;

// ============================================================
// bms.model.Mode — re-exported from real bms-model crate
// ============================================================

pub use ::bms_model::mode as bms_model;

// ============================================================
// bms.tool.util.Pair
// ============================================================

/// Stub for bms.tool.util.Pair
#[derive(Clone, Debug)]
pub struct Pair<A, B> {
    pub first: A,
    pub second: B,
}

impl<A, B> Pair<A, B> {
    pub fn of(first: A, second: B) -> Self {
        Self { first, second }
    }
    pub fn get_first(&self) -> &A {
        &self.first
    }
    pub fn get_second(&self) -> &B {
        &self.second
    }
}

impl<A: Clone, B: Clone> Pair<A, B> {
    pub fn project_first(pairs: &[Self]) -> Vec<A> {
        pairs.iter().map(|p| p.first.clone()).collect()
    }
}

// ============================================================
// Timer stub
// ============================================================

/// Stub for timer used in MainState
pub struct TimerState {
    pub now_time: i64,
}

impl TimerState {
    pub fn get_now_time(&self) -> i64 {
        self.now_time
    }
    pub fn get_timer(&self, _id: i32) -> i64 {
        0
    }
    pub fn set_timer_on(&self, _id: i32) {}
    pub fn set_timer_off(&self, _id: i32) {}
    pub fn switch_timer(&self, _id: i32, _on: bool) {}
}

// ============================================================
// Clipboard stub
// ============================================================

/// Stub for clipboard access
pub struct Clipboard;

impl Clipboard {
    pub fn set_contents(_text: &str) {
        // stub: would copy to system clipboard
    }
}
