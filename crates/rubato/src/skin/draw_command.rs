/// Draw command types emitted by the lane renderer.
/// These represent the rendering operations that the caller must execute
/// using whatever rendering backend is available (SkinObjectRenderer, etc.).
///
/// Moved from rubato-play::lane_renderer to break the rubato-skin -> rubato-play dependency.
#[derive(Clone, Debug, PartialEq)]
pub enum DrawCommand {
    /// Set color (RGBA)
    SetColor { r: f32, g: f32, b: f32, a: f32 },
    /// Set blend mode
    SetBlend(i32),
    /// Set renderer type
    SetType(i32),
    /// Draw a note image at position
    DrawNote {
        lane: usize,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        /// Which image to draw: Normal, Processed, Mine, Hidden
        image_type: NoteImageType,
    },
    /// Draw a long note body/start/end
    DrawLongNote {
        lane: usize,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        /// Index into the longImage array (0-9)
        image_index: usize,
    },
    /// Draw section line (delegates to skin line images)
    DrawSectionLine { y_offset: i32 },
    /// Draw timeline display (practice mode)
    DrawTimeLine { y_offset: i32 },
    /// Draw BPM change line
    DrawBpmLine { y_offset: i32, bpm: f64 },
    /// Draw stop line
    DrawStopLine { y_offset: i32, stop_ms: i64 },
    /// Draw timeline text (time display in practice mode)
    DrawTimeText { text: String, x: f32, y: f32 },
    /// Draw BPM text
    DrawBpmText { text: String, x: f32, y: f32 },
    /// Draw stop text
    DrawStopText { text: String, x: f32, y: f32 },
    /// Draw judge area (colored rectangles)
    DrawJudgeArea {
        lane: usize,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        color_index: usize,
    },
}

/// Note image types for DrawNote command
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NoteImageType {
    Normal,
    Processed,
    Mine,
    Hidden,
}
