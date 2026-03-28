// SkinBGA.java -> skin_bga_object.rs
// Translated from Java SkinBGA, connecting BGA display to the skin rendering system.

use std::sync::{Arc, Mutex};

use crate::skin::sync_utils::lock_or_recover;

use crate::render::color::Color;
use crate::skin::practice_draw_command::{PracticeColor, PracticeDrawCommand};

use crate::skin::graphs::skin_note_distribution_graph::{
    NoteDistributionDrawParams, SkinNoteDistributionGraph, TYPE_EARLYLATE, TYPE_JUDGE, TYPE_NORMAL,
};
use crate::skin::reexports::{MainState, Rectangle};
use crate::skin::skin_object::{SkinObjectData, SkinObjectRenderer};
use crate::skin::skin_property;

/// Trait for BGA drawing, bridging BGAProcessor to the skin rendering system.
/// This trait allows the skin to call BGA operations without knowing BGAProcessor internals.
/// It also enables mock-based testing of the skin BGA object.
pub trait BgaDraw: Send {
    /// Update BGA timeline to the given time (milliseconds).
    /// Pass -1 for states where BGA should not be displayed (preload, practice, ready).
    /// Corresponds to Java BGAProcessor.prepareBGA(time).
    fn prepare_bga(&mut self, time_ms: i64);

    /// Draw BGA content using the skin's renderer.
    /// The bga_expand parameter determines aspect-ratio handling.
    /// Corresponds to Java BGAProcessor.drawBGA(SkinBGA, SkinObjectRenderer, Rectangle).
    fn draw_bga(&mut self, sprite: &mut SkinObjectRenderer, region: &Rectangle, bga_expand: i32);
}

// =========================================================================
// SkinBgaObject -- BGA skin object for the rendering pipeline
// =========================================================================

/// BGA skin object for the rendering pipeline.
/// Translated from: SkinBGA.java
///
/// In practice mode, draws PracticeConfiguration UI instead of BGA.
/// The caller sets practice draw commands via `set_practice_draw_commands()`.
pub struct SkinBgaObject {
    pub data: SkinObjectData,
    bga_expand: i32,
    /// Shared reference to the BGA drawing implementation (BGAProcessor).
    /// Set by BMSPlayer when the skin is loaded.
    bga_draw: Option<Arc<Mutex<dyn BgaDraw>>>,
    /// Practice mode draw commands (set by caller when in practice mode).
    practice_commands: Vec<PracticeDrawCommand>,
    /// Whether this BGA is currently in practice mode.
    practice_mode: bool,
    /// Reusable font for practice mode text rendering.
    /// Stored as a field to avoid per-frame allocation.
    practice_font: crate::render::font::BitmapFont,
    /// Note distribution graphs for practice mode (TYPE_NORMAL, TYPE_JUDGE, TYPE_EARLYLATE).
    practice_graphs: [SkinNoteDistributionGraph; 3],
}

/// Practice mode font size matching Java's default BitmapFont (15px).
const PRACTICE_FONT_SIZE: f32 = 15.0;

/// Try to load a system TrueType font for practice mode text overlay.
/// Java's `new BitmapFont()` uses LibGDX's built-in Arial. We try common system paths.
fn try_load_practice_font() -> crate::render::font::BitmapFont {
    #[cfg(target_os = "macos")]
    const FONT_PATHS: &[&str] = &[
        "/System/Library/Fonts/Helvetica.ttc",
        "/System/Library/Fonts/Supplemental/Arial.ttf",
    ];
    #[cfg(target_os = "linux")]
    const FONT_PATHS: &[&str] = &[
        "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
        "/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf",
    ];
    #[cfg(target_os = "windows")]
    const FONT_PATHS: &[&str] = &[
        "C:\\Windows\\Fonts\\arial.ttf",
        "C:\\Windows\\Fonts\\msgothic.ttc",
    ];
    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    const FONT_PATHS: &[&str] = &[];

    for path in FONT_PATHS {
        let font = crate::render::font::BitmapFont::from_file(path, PRACTICE_FONT_SIZE);
        if font.font().is_some() {
            return font;
        }
    }
    crate::render::font::BitmapFont::new()
}

impl SkinBgaObject {
    pub fn new(bga_expand: i32) -> Self {
        SkinBgaObject {
            data: SkinObjectData::default(),
            bga_expand,
            bga_draw: None,
            practice_commands: Vec::new(),
            practice_mode: false,
            practice_font: try_load_practice_font(),
            practice_graphs: [
                SkinNoteDistributionGraph::new(TYPE_NORMAL, 500, 0, 0, 0, 0),
                SkinNoteDistributionGraph::new(TYPE_JUDGE, 500, 0, 0, 0, 0),
                SkinNoteDistributionGraph::new(TYPE_EARLYLATE, 500, 0, 0, 0, 0),
            ],
        }
    }

    /// Set the BGA drawing implementation.
    /// Called by BMSPlayer to connect the BGAProcessor to the skin system.
    pub fn set_bga_draw(&mut self, bga_draw: Arc<Mutex<dyn BgaDraw>>) {
        self.bga_draw = Some(bga_draw);
    }

    /// Get the BGA expand mode.
    pub fn bga_expand(&self) -> i32 {
        self.bga_expand
    }

    /// Check if this object has a BGA drawing implementation connected.
    pub fn has_bga_draw(&self) -> bool {
        self.bga_draw.is_some()
    }

    /// Set practice mode draw commands.
    /// Called by the game loop when in practice mode.
    pub fn set_practice_draw_commands(&mut self, commands: Vec<PracticeDrawCommand>) {
        self.practice_commands = commands;
        self.practice_mode = true;
    }

    /// Set whether this BGA is in practice mode.
    pub fn set_practice_mode(&mut self, practice: bool) {
        self.practice_mode = practice;
        if !practice {
            self.practice_commands.clear();
        }
    }

    /// Check if this BGA is in practice mode.
    pub fn is_practice_mode(&self) -> bool {
        self.practice_mode
    }

    /// Set the font used for practice mode text rendering.
    /// Call this to provide a properly loaded font so practice text actually renders.
    pub fn set_practice_font(&mut self, font: crate::render::font::BitmapFont) {
        self.practice_font = font;
    }

    /// Prepare BGA for rendering.
    /// Translated from: Java SkinBGA.prepare(long time, MainState state)
    ///
    /// In Java, this:
    /// 1. Sets the player reference from state
    /// 2. Calls super.prepare() to update draw/region/color
    /// 3. If draw is true, calls BGAProcessor.prepareBGA() with appropriate time:
    ///    - -1 for PRELOAD/PRACTICE/READY states
    ///    - timer.getNowTime(TIMER_PLAY) for other states
    pub fn prepare(&mut self, time: i64, state: &dyn MainState) {
        self.data.prepare(time, state);

        if self.data.draw
            && let Some(ref bga_draw) = self.bga_draw
        {
            // Determine BGA time:
            // In Java: s == STATE_PRELOAD || s == STATE_PRACTICE || s == STATE_READY ? -1
            //          : player.timer.getNowTime(TIMER_PLAY)
            let play_time = state.now_time_for(skin_property::TIMER_PLAY);
            // If play timer is not active (returns Long.MIN_VALUE in Java, which
            // we represent as i64::MIN or a negative value), pass -1
            let bga_time = if play_time < 0 { -1 } else { play_time };

            let mut draw = lock_or_recover(bga_draw);
            draw.prepare_bga(bga_time);
        }
    }

    /// Draw BGA content or practice configuration UI.
    ///
    /// Translated from: Java SkinBGA.draw(SkinObjectRenderer sprite)
    /// In Java:
    ///   if (PRACTICE) { player.getPracticeConfiguration().draw(...) }
    ///   else { resource.getBGAManager().drawBGA(...) }
    pub fn draw_impl(&mut self, sprite: &mut SkinObjectRenderer, state: &dyn MainState) {
        if self.practice_mode {
            self.draw_practice(sprite, state);
        } else if let Some(ref bga_draw) = self.bga_draw {
            let region = self.data.region;
            let mut draw = lock_or_recover(bga_draw);
            draw.draw_bga(sprite, &region, self.bga_expand);
        }
    }

    /// Execute practice mode draw commands.
    /// Translated from: Java PracticeConfiguration.draw(Rectangle, SkinObjectRenderer, long, MainState)
    fn draw_practice(&mut self, sprite: &mut SkinObjectRenderer, state: &dyn MainState) {
        // Clone commands to avoid borrow conflict with &mut self (practice_font + practice_graphs).
        let commands: Vec<_> = self.practice_commands.clone();
        for cmd in &commands {
            match cmd {
                PracticeDrawCommand::DrawText { text, x, y, color } => {
                    let c = match color {
                        PracticeColor::Yellow => Color::new(1.0, 1.0, 0.0, 1.0),
                        PracticeColor::Cyan => Color::new(0.0, 1.0, 1.0, 1.0),
                        PracticeColor::Orange => Color::new(1.0, 0.65, 0.0, 1.0),
                        PracticeColor::White => Color::new(1.0, 1.0, 1.0, 1.0),
                    };
                    sprite.draw_font(&mut self.practice_font, text, *x, *y, &c);
                }
                PracticeDrawCommand::DrawGraph {
                    graph_type,
                    region,
                    start_time,
                    end_time,
                    freq,
                } => {
                    let idx = (*graph_type as usize).min(2);
                    let rect = Rectangle::new(region.0, region.1, region.2, region.3);
                    let time = state.now_time_for(skin_property::TIMER_PLAY);
                    self.practice_graphs[idx].draw_with_params(
                        sprite,
                        NoteDistributionDrawParams {
                            time,
                            state,
                            region: &rect,
                            starttime: *start_time,
                            endtime: *end_time,
                            freq: *freq,
                        },
                    );
                }
            }
        }
    }

    pub fn dispose(&mut self) {
        // No resources to dispose in Rust translation
    }

    pub fn validate(&mut self) -> bool {
        self.data.validate()
    }
}

impl crate::skin::types::skin_node::SkinNode for SkinBgaObject {
    fn data(&self) -> &SkinObjectData {
        &self.data
    }
    fn data_mut(&mut self) -> &mut SkinObjectData {
        &mut self.data
    }
    fn prepare(&mut self, time: i64, state: &dyn MainState) {
        SkinBgaObject::prepare(self, time, state)
    }
    fn draw(&mut self, sprite: &mut SkinObjectRenderer, state: &dyn MainState) {
        self.draw_impl(sprite, state)
    }
    fn dispose(&mut self) {
        SkinBgaObject::dispose(self)
    }
    fn type_name(&self) -> &'static str {
        "SkinBGA"
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
    fn into_any_box(self: Box<Self>) -> Box<dyn std::any::Any> {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::skin::bga_types::{BGAEXPAND_FULL, BGAEXPAND_KEEP_ASPECT_RATIO, BGAEXPAND_OFF};

    // =========================================================================
    // Mock BgaDraw for testing
    // =========================================================================

    #[derive(Default)]
    struct MockBgaDraw {
        prepare_calls: Vec<i64>,
        draw_calls: Vec<(f32, f32, f32, f32, i32)>,
    }

    impl BgaDraw for MockBgaDraw {
        fn prepare_bga(&mut self, time_ms: i64) {
            self.prepare_calls.push(time_ms);
        }

        fn draw_bga(
            &mut self,
            _sprite: &mut SkinObjectRenderer,
            region: &Rectangle,
            bga_expand: i32,
        ) {
            self.draw_calls
                .push((region.x, region.y, region.width, region.height, bga_expand));
        }
    }

    // =========================================================================
    // SkinBgaObject tests
    // =========================================================================

    #[test]
    fn test_new_skin_bga_object() {
        let bga = SkinBgaObject::new(BGAEXPAND_FULL);
        assert_eq!(bga.bga_expand(), BGAEXPAND_FULL);
        assert!(!bga.has_bga_draw());
    }

    #[test]
    fn test_new_with_different_expand_modes() {
        assert_eq!(
            SkinBgaObject::new(BGAEXPAND_FULL).bga_expand(),
            BGAEXPAND_FULL
        );
        assert_eq!(
            SkinBgaObject::new(BGAEXPAND_KEEP_ASPECT_RATIO).bga_expand(),
            BGAEXPAND_KEEP_ASPECT_RATIO
        );
        assert_eq!(
            SkinBgaObject::new(BGAEXPAND_OFF).bga_expand(),
            BGAEXPAND_OFF
        );
    }

    #[test]
    fn test_set_bga_draw() {
        let mut bga = SkinBgaObject::new(BGAEXPAND_FULL);
        assert!(!bga.has_bga_draw());

        let mock = Arc::new(Mutex::new(MockBgaDraw::default()));
        bga.set_bga_draw(mock);
        assert!(bga.has_bga_draw());
    }

    #[test]
    fn test_draw_delegates_to_bga_draw() {
        let mut bga = SkinBgaObject::new(BGAEXPAND_KEEP_ASPECT_RATIO);
        let mock = Arc::new(Mutex::new(MockBgaDraw::default()));
        bga.set_bga_draw(mock.clone());

        // Set up region on data
        bga.data.region = Rectangle::new(10.0, 20.0, 300.0, 200.0);

        let mut sprite = SkinObjectRenderer::new();
        let state = crate::skin::test_helpers::MockMainState::default();
        bga.draw_impl(&mut sprite, &state);

        let mock_locked = mock.lock().expect("mutex poisoned");
        assert_eq!(mock_locked.draw_calls.len(), 1);
        assert_eq!(
            mock_locked.draw_calls[0],
            (10.0, 20.0, 300.0, 200.0, BGAEXPAND_KEEP_ASPECT_RATIO)
        );
    }

    #[test]
    fn test_draw_no_bga_draw_is_noop() {
        let mut bga = SkinBgaObject::new(BGAEXPAND_FULL);
        let mut sprite = SkinObjectRenderer::new();
        let state = crate::skin::test_helpers::MockMainState::default();
        // Should not panic
        bga.draw_impl(&mut sprite, &state);
    }

    #[test]
    fn test_dispose_is_noop() {
        let mut bga = SkinBgaObject::new(BGAEXPAND_FULL);
        bga.dispose(); // Should not panic
    }

    // BGAProcessor integration tests and BgaRenderer adapter tests
    // have been moved to rubato-play (the crate that owns BGAProcessor).

    #[test]
    fn test_practice_font_is_reused_across_draw_calls() {
        let mut bga = SkinBgaObject::new(BGAEXPAND_FULL);
        bga.set_practice_draw_commands(vec![PracticeDrawCommand::DrawText {
            text: "test".to_string(),
            x: 10.0,
            y: 20.0,
            color: PracticeColor::White,
        }]);

        let mut sprite = SkinObjectRenderer::new();
        let state = crate::skin::test_helpers::MockMainState::default();
        // Draw multiple times -- should not panic and should reuse the stored font.
        bga.draw_impl(&mut sprite, &state);
        bga.draw_impl(&mut sprite, &state);

        // The practice_font field exists and is not re-created per frame.
        // Verify the font is still the same object (scale matches PRACTICE_FONT_SIZE).
        assert_eq!(bga.practice_font.scale, PRACTICE_FONT_SIZE);
    }

    #[test]
    fn test_set_practice_font_replaces_default() {
        let mut bga = SkinBgaObject::new(BGAEXPAND_FULL);
        assert_eq!(bga.practice_font.scale, PRACTICE_FONT_SIZE);

        let mut custom_font = crate::render::font::BitmapFont::new();
        custom_font.scale = 24.0;
        bga.set_practice_font(custom_font);

        assert_eq!(bga.practice_font.scale, 24.0);
    }
}
