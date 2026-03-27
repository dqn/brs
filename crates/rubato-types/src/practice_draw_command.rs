/// Colors used in practice configuration UI drawing.
///
/// Moved from rubato-play::practice_configuration::constants to break
/// the rubato-skin -> rubato-play dependency.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PracticeColor {
    Yellow,
    Cyan,
    Orange,
    White,
}

/// Draw commands emitted by PracticeConfiguration.draw().
/// The skin layer executes these using SkinObjectRenderer.
#[derive(Clone, Debug)]
pub enum PracticeDrawCommand {
    /// Draw text at position with color
    DrawText {
        text: String,
        x: f32,
        y: f32,
        color: PracticeColor,
    },
    /// Draw note distribution graph
    DrawGraph {
        graph_type: i32,
        region: (f32, f32, f32, f32),
        start_time: i32,
        end_time: i32,
        freq: f32,
    },
}
