// Stubs for external dependencies not yet available in the Rust port.
// These will be replaced with real implementations in future phases.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

// =========================================================================
// MainController stub
// =========================================================================

/// Stub for MainController reference used by various menus
pub struct MainController;

impl MainController {
    pub fn get_config(&self) -> Config {
        todo!("MainController::get_config - Phase 8+ dependency")
    }

    pub fn get_player_config(&self) -> PlayerConfig {
        todo!("MainController::get_player_config - Phase 8+ dependency")
    }

    pub fn get_current_state(&self) -> Box<dyn MainState> {
        todo!("MainController::get_current_state - Phase 8+ dependency")
    }

    pub fn get_timer(&self) -> Timer {
        todo!("MainController::get_timer - Phase 8+ dependency")
    }

    pub fn save_config(&self) {
        todo!("MainController::save_config - Phase 8+ dependency")
    }

    pub fn load_new_profile(&self, _config: PlayerConfig) {
        todo!("MainController::load_new_profile - Phase 8+ dependency")
    }

    pub fn get_player_resource(&self) -> PlayerResource {
        todo!("MainController::get_player_resource - Phase 8+ dependency")
    }
}

// =========================================================================
// Config stub
// =========================================================================

#[derive(Clone, Debug, Default)]
pub struct Config {
    pub playername: String,
}

impl Config {
    pub fn get_playername(&self) -> &str {
        &self.playername
    }
}

// =========================================================================
// PlayerConfig stub
// =========================================================================

#[derive(Clone, Debug, Default)]
pub struct PlayerConfig {
    pub skin: Vec<SkinConfig>,
    pub skin_history: Vec<SkinConfig>,
}

impl PlayerConfig {
    pub fn read_all_player_id(dir: &str) -> Vec<String> {
        todo!("PlayerConfig::read_all_player_id - Phase 8+ dependency")
    }

    pub fn read_player_config(dir: &str, player_id: &str) -> PlayerConfig {
        todo!("PlayerConfig::read_player_config - Phase 8+ dependency")
    }

    pub fn get_play_config(&self, _mode: &bms_model::mode::Mode) -> PlayModeConfig {
        todo!("PlayerConfig::get_play_config - Phase 8+ dependency")
    }

    pub fn get_skin(&self) -> &Vec<SkinConfig> {
        &self.skin
    }

    pub fn get_skin_mut(&mut self) -> &mut Vec<SkinConfig> {
        &mut self.skin
    }

    pub fn get_skin_history(&self) -> &Vec<SkinConfig> {
        &self.skin_history
    }

    pub fn set_skin_history(&mut self, history: Vec<SkinConfig>) {
        self.skin_history = history;
    }
}

// =========================================================================
// PlayModeConfig / PlayConfig stubs
// =========================================================================

#[derive(Clone, Debug, Default)]
pub struct PlayModeConfig {
    pub playconfig: PlayConfig,
}

impl PlayModeConfig {
    pub fn get_playconfig(&self) -> &PlayConfig {
        &self.playconfig
    }

    pub fn get_playconfig_mut(&mut self) -> &mut PlayConfig {
        &mut self.playconfig
    }
}

#[derive(Clone, Debug, Default)]
pub struct PlayConfig {
    pub enablelift: bool,
    pub lift: f32,
    pub enablehidden: bool,
    pub hidden: f32,
    pub enablelanecover: bool,
    pub lanecover: f32,
    pub lanecovermarginlow: f32,
    pub lanecovermarginhigh: f32,
    pub lanecoverswitchduration: i32,
    pub enable_constant: bool,
    pub constant_fadein_time: i32,
}

impl PlayConfig {
    pub fn is_enablelift(&self) -> bool {
        self.enablelift
    }
    pub fn set_enablelift(&mut self, v: bool) {
        self.enablelift = v;
    }
    pub fn get_lift(&self) -> f32 {
        self.lift
    }
    pub fn set_lift(&mut self, v: f32) {
        self.lift = v;
    }
    pub fn is_enablehidden(&self) -> bool {
        self.enablehidden
    }
    pub fn set_enablehidden(&mut self, v: bool) {
        self.enablehidden = v;
    }
    pub fn get_hidden(&self) -> f32 {
        self.hidden
    }
    pub fn set_hidden(&mut self, v: f32) {
        self.hidden = v;
    }
    pub fn is_enablelanecover(&self) -> bool {
        self.enablelanecover
    }
    pub fn set_enablelanecover(&mut self, v: bool) {
        self.enablelanecover = v;
    }
    pub fn get_lanecover(&self) -> f32 {
        self.lanecover
    }
    pub fn set_lanecover(&mut self, v: f32) {
        self.lanecover = v;
    }
    pub fn get_lanecovermarginlow(&self) -> f32 {
        self.lanecovermarginlow
    }
    pub fn set_lanecovermarginlow(&mut self, v: f32) {
        self.lanecovermarginlow = v;
    }
    pub fn get_lanecovermarginhigh(&self) -> f32 {
        self.lanecovermarginhigh
    }
    pub fn set_lanecovermarginhigh(&mut self, v: f32) {
        self.lanecovermarginhigh = v;
    }
    pub fn get_lanecoverswitchduration(&self) -> i32 {
        self.lanecoverswitchduration
    }
    pub fn set_lanecoverswitchduration(&mut self, v: i32) {
        self.lanecoverswitchduration = v;
    }
    pub fn is_enable_constant(&self) -> bool {
        self.enable_constant
    }
    pub fn set_enable_constant(&mut self, v: bool) {
        self.enable_constant = v;
    }
    pub fn get_constant_fadein_time(&self) -> i32 {
        self.constant_fadein_time
    }
    pub fn set_constant_fadein_time(&mut self, v: i32) {
        self.constant_fadein_time = v;
    }
}

// =========================================================================
// SkinConfig stub
// =========================================================================

#[derive(Clone, Debug, Default)]
pub struct SkinConfig {
    pub path: String,
    pub properties: SkinConfigProperty,
}

impl SkinConfig {
    pub fn get_path(&self) -> &str {
        &self.path
    }

    pub fn set_path(&mut self, path: String) {
        self.path = path;
    }

    pub fn get_properties(&self) -> &SkinConfigProperty {
        &self.properties
    }

    pub fn set_properties(&mut self, properties: SkinConfigProperty) {
        self.properties = properties;
    }

    pub fn validate(&mut self) {
        // stub
    }
}

pub struct SkinConfigDefault;

impl SkinConfigDefault {
    pub fn get(_skin_type: &SkinType) -> SkinConfigDefaultEntry {
        SkinConfigDefaultEntry {
            path: String::new(),
        }
    }
}

pub struct SkinConfigDefaultEntry {
    pub path: String,
}

#[derive(Clone, Debug, Default)]
pub struct SkinConfigProperty {
    pub option: Vec<SkinConfigOption>,
    pub file: Vec<SkinConfigFilePath>,
    pub offset: Vec<SkinConfigOffset>,
}

impl SkinConfigProperty {
    pub fn get_option(&self) -> &[SkinConfigOption] {
        &self.option
    }

    pub fn set_option(&mut self, option: Vec<SkinConfigOption>) {
        self.option = option;
    }

    pub fn get_file(&self) -> &[SkinConfigFilePath] {
        &self.file
    }

    pub fn set_file(&mut self, file: Vec<SkinConfigFilePath>) {
        self.file = file;
    }

    pub fn get_offset(&self) -> &[SkinConfigOffset] {
        &self.offset
    }

    pub fn set_offset(&mut self, offset: Vec<SkinConfigOffset>) {
        self.offset = offset;
    }
}

#[derive(Clone, Debug, Default)]
pub struct SkinConfigOption {
    pub name: String,
    pub value: i32,
}

#[derive(Clone, Debug, Default)]
pub struct SkinConfigFilePath {
    pub name: String,
    pub path: String,
}

#[derive(Clone, Debug, Default)]
pub struct SkinConfigOffset {
    pub name: String,
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
    pub r: i32,
    pub a: i32,
}

// =========================================================================
// Timer stub
// =========================================================================

pub struct Timer;

impl Timer {
    pub fn set_frozen(&self, _frozen: bool) {
        todo!("Timer::set_frozen - egui integration")
    }
}

// =========================================================================
// MainState trait stub
// =========================================================================

pub trait MainState {
    fn get_skin(&self) -> &Skin;
    fn set_skin(&mut self, skin: Skin);
    fn as_any(&self) -> &dyn std::any::Any;
}

// =========================================================================
// Version stub
// =========================================================================

pub struct Version;

impl Version {
    pub fn get_git_commit_hash() -> &'static str {
        "unknown"
    }

    pub fn get_build_date() -> &'static str {
        "unknown"
    }
}

// =========================================================================
// Skin types stubs
// =========================================================================

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub enum SkinType {
    #[default]
    PLAY_7KEYS,
    PLAY_5KEYS,
    PLAY_14KEYS,
    PLAY_10KEYS,
    PLAY_9KEYS,
    MUSIC_SELECT,
    DECIDE,
    RESULT,
    COURSE_RESULT,
    SKIN_SELECT,
    KEY_CONFIG,
}

impl SkinType {
    pub fn get_id(&self) -> usize {
        match self {
            SkinType::PLAY_7KEYS => 0,
            SkinType::PLAY_5KEYS => 1,
            SkinType::PLAY_14KEYS => 2,
            SkinType::PLAY_10KEYS => 3,
            SkinType::PLAY_9KEYS => 4,
            SkinType::MUSIC_SELECT => 5,
            SkinType::DECIDE => 6,
            SkinType::RESULT => 7,
            SkinType::COURSE_RESULT => 8,
            SkinType::SKIN_SELECT => 9,
            SkinType::KEY_CONFIG => 10,
        }
    }
}

// =========================================================================
// SkinHeader stub
// =========================================================================

pub const TYPE_LR2SKIN: i32 = 0;

#[derive(Clone, Debug)]
pub struct SkinHeader {
    pub name: String,
    pub path: PathBuf,
    pub skin_type: SkinType,
    pub header_type: i32,
    pub custom_options: Vec<CustomOption>,
    pub custom_files: Vec<CustomFile>,
    pub custom_offsets: Vec<CustomOffset>,
    pub custom_categories: Vec<CustomCategory>,
}

impl Default for SkinHeader {
    fn default() -> Self {
        SkinHeader {
            name: String::new(),
            path: PathBuf::new(),
            skin_type: SkinType::default(),
            header_type: 0,
            custom_options: Vec::new(),
            custom_files: Vec::new(),
            custom_offsets: Vec::new(),
            custom_categories: Vec::new(),
        }
    }
}

impl SkinHeader {
    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    pub fn get_path(&self) -> &Path {
        &self.path
    }

    pub fn get_skin_type(&self) -> &SkinType {
        &self.skin_type
    }

    pub fn set_skin_type(&mut self, skin_type: SkinType) {
        self.skin_type = skin_type;
    }

    pub fn get_type(&self) -> i32 {
        self.header_type
    }

    pub fn get_custom_options(&self) -> &[CustomOption] {
        &self.custom_options
    }

    pub fn get_custom_files(&self) -> &[CustomFile] {
        &self.custom_files
    }

    pub fn get_custom_offsets(&self) -> &[CustomOffset] {
        &self.custom_offsets
    }

    pub fn get_custom_categories(&self) -> &[CustomCategory] {
        &self.custom_categories
    }
}

#[derive(Clone, Debug)]
pub struct CustomOption {
    pub name: String,
    pub contents: Vec<String>,
    pub option: Vec<i32>,
    pub default_option: i32,
}

impl CustomOption {
    pub fn get_default_option(&self) -> i32 {
        self.default_option
    }
}

#[derive(Clone, Debug)]
pub struct CustomFile {
    pub name: String,
    pub path: String,
    pub def: Option<String>,
}

#[derive(Clone, Debug)]
pub struct CustomOffset {
    pub name: String,
    pub x: bool,
    pub y: bool,
    pub w: bool,
    pub h: bool,
    pub r: bool,
    pub a: bool,
}

#[derive(Clone, Debug)]
pub struct CustomCategory {
    pub name: String,
    pub items: Vec<CustomCategoryItem>,
}

#[derive(Clone, Debug)]
pub enum CustomCategoryItem {
    Option(CustomOption),
    File(CustomFile),
    Offset(CustomOffset),
}

// =========================================================================
// Skin stub
// =========================================================================

#[derive(Clone, Debug, Default)]
pub struct Skin {
    pub header: SkinHeader,
    objects: Vec<SkinObject>,
}

impl Skin {
    pub fn get_all_skin_objects(&self) -> &[SkinObject] {
        &self.objects
    }

    pub fn prepare(&self, _state: &dyn MainState) {
        todo!("Skin::prepare - rendering dependency")
    }
}

// =========================================================================
// SkinObject stub
// =========================================================================

#[derive(Clone, Debug, Default)]
pub struct SkinObject {
    pub name: Option<String>,
    pub draw: bool,
    pub visible: bool,
    pub destinations: Vec<SkinObjectDestination>,
}

impl SkinObject {
    pub fn get_name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    pub fn get_all_destination(&self) -> &[SkinObjectDestination] {
        &self.destinations
    }
}

#[derive(Clone, Debug, Default)]
pub struct SkinObjectDestination {
    pub time: i32,
    pub region: Rectangle,
    pub color: Option<[f32; 4]>,
    pub angle: f32,
    pub alpha: f32,
}

#[derive(Clone, Debug, Default)]
pub struct Rectangle {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

// =========================================================================
// SkinLoader stub
// =========================================================================

pub struct SkinLoader;

impl SkinLoader {
    pub fn load(
        _state: &dyn MainState,
        _skin_type: &SkinType,
        _config: &SkinConfig,
    ) -> Option<Skin> {
        todo!("SkinLoader::load - rendering dependency")
    }
}

// =========================================================================
// JSONSkinLoader / LR2SkinHeaderLoader / LuaSkinLoader stubs
// =========================================================================

pub struct JSONSkinLoader;

impl Default for JSONSkinLoader {
    fn default() -> Self {
        Self::new()
    }
}

impl JSONSkinLoader {
    pub fn new() -> Self {
        JSONSkinLoader
    }

    pub fn load_header(&self, _path: &Path) -> Option<SkinHeader> {
        todo!("JSONSkinLoader::load_header - skin loader dependency")
    }
}

pub struct LR2SkinHeaderLoader;

impl LR2SkinHeaderLoader {
    pub fn new(_config: &Config) -> Self {
        LR2SkinHeaderLoader
    }

    pub fn load_skin(&self, _path: &Path, _opt: Option<()>) -> std::io::Result<SkinHeader> {
        todo!("LR2SkinHeaderLoader::load_skin - skin loader dependency")
    }
}

pub struct LuaSkinLoader;

impl Default for LuaSkinLoader {
    fn default() -> Self {
        Self::new()
    }
}

impl LuaSkinLoader {
    pub fn new() -> Self {
        LuaSkinLoader
    }

    pub fn load_header(&self, _path: &Path) -> Option<SkinHeader> {
        todo!("LuaSkinLoader::load_header - skin loader dependency")
    }
}

// =========================================================================
// SkinProperty constants stub
// =========================================================================

pub const OPTION_RANDOM_VALUE: i32 = -1;

// =========================================================================
// MusicSelector stub
// =========================================================================

pub struct MusicSelector {
    pub main: MusicSelectorMainRef,
}

impl MusicSelector {
    pub fn get_bar_render(&self) -> &BarRenderer {
        todo!("MusicSelector::get_bar_render - select dependency")
    }

    pub fn get_bar_manager(&self) -> &BarManager {
        todo!("MusicSelector::get_bar_manager - select dependency")
    }

    pub fn get_selected_bar(&self) -> &dyn Bar {
        todo!("MusicSelector::get_selected_bar - select dependency")
    }
}

pub struct MusicSelectorMainRef;

impl MusicSelectorMainRef {
    pub fn get_player_resource(&self) -> &PlayerResource {
        todo!("MusicSelectorMainRef::get_player_resource - select dependency")
    }
}

pub struct BarRenderer;

impl BarRenderer {
    pub fn update_bar_text(&self) {
        todo!("BarRenderer::update_bar_text - select dependency")
    }
}

pub struct BarManager;

impl BarManager {
    pub fn update_bar(&self) {
        todo!("BarManager::update_bar - select dependency")
    }
}

pub trait Bar {
    fn as_any(&self) -> &dyn std::any::Any;
}

pub struct SongBar {
    pub song_data: Option<SongData>,
    pub score: Option<ScoreData>,
}

impl SongBar {
    pub fn get_song_data(&self) -> Option<&SongData> {
        self.song_data.as_ref()
    }

    pub fn get_score(&self) -> Option<&ScoreData> {
        self.score.as_ref()
    }
}

// =========================================================================
// SongData / ScoreData stubs
// =========================================================================

#[derive(Clone, Debug, Default)]
pub struct SongData {
    pub title: String,
    pub path: Option<String>,
}

impl SongData {
    pub fn get_title(&self) -> &str {
        &self.title
    }

    pub fn get_path(&self) -> Option<&str> {
        self.path.as_deref()
    }
}

#[derive(Clone, Debug, Default)]
pub struct ScoreData {
    pub date: i64,
}

impl ScoreData {
    pub fn get_date(&self) -> i64 {
        self.date
    }
}

// =========================================================================
// PlayerResource stub
// =========================================================================

pub struct PlayerResource;

impl PlayerResource {
    pub fn get_reverse_lookup_data(&self) -> Vec<String> {
        todo!("PlayerResource::get_reverse_lookup_data - Phase 8+ dependency")
    }
}

// =========================================================================
// ImGui/egui stub types
// =========================================================================

/// Stub for ImBoolean (Java imgui binding type).
/// In egui this would just be a `bool` with mutable access.
#[derive(Clone, Debug, Default)]
pub struct ImBoolean {
    pub value: bool,
}

impl ImBoolean {
    pub fn new(value: bool) -> Self {
        ImBoolean { value }
    }

    pub fn get(&self) -> bool {
        self.value
    }

    pub fn set(&mut self, value: bool) {
        self.value = value;
    }
}

/// Stub for ImInt (Java imgui binding type).
#[derive(Clone, Debug, Default)]
pub struct ImInt {
    pub value: i32,
}

impl ImInt {
    pub fn new(value: i32) -> Self {
        ImInt { value }
    }

    pub fn get(&self) -> i32 {
        self.value
    }

    pub fn set(&mut self, value: i32) {
        self.value = value;
    }
}

/// Stub for ImFloat (Java imgui binding type).
#[derive(Clone, Debug, Default)]
pub struct ImFloat {
    pub value: f32,
}

impl ImFloat {
    pub fn new(value: f32) -> Self {
        ImFloat { value }
    }

    pub fn get(&self) -> f32 {
        self.value
    }

    pub fn set(&mut self, value: f32) {
        self.value = value;
    }
}

// =========================================================================
// LWJGL3/LibGDX stubs
// =========================================================================

pub struct InputProcessor;

pub struct Lwjgl3ControllerManager;

impl Default for Lwjgl3ControllerManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Lwjgl3ControllerManager {
    pub fn new() -> Self {
        Lwjgl3ControllerManager
    }

    pub fn get_controllers(&self) -> Vec<Controller> {
        Vec::new()
    }
}

pub struct Controller {
    pub name: String,
}

impl Controller {
    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_axis(&self, _index: i32) -> f32 {
        0.0
    }
}

// =========================================================================
// Clipboard stub
// =========================================================================

pub struct Clipboard;

impl Default for Clipboard {
    fn default() -> Self {
        Self::new()
    }
}

impl Clipboard {
    pub fn new() -> Self {
        Clipboard
    }

    pub fn set_contents(&self, _contents: &str) {
        todo!("Clipboard::set_contents - platform dependency")
    }
}
