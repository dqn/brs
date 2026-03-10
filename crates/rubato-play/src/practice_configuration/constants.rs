pub(super) static GAUGE: &[&str] = &[
    "ASSIST EASY",
    "EASY",
    "NORMAL",
    "HARD",
    "EX-HARD",
    "HAZARD",
    "GRADE",
    "EX GRADE",
    "EXHARD GRADE",
];
pub(super) static RANDOM: &[&str] = &[
    "NORMAL",
    "MIRROR",
    "RANDOM",
    "R-RANDOM",
    "S-RANDOM",
    "SPIRAL",
    "H-RANDOM",
    "ALL-SCR",
    "RANDOM-EX",
    "S-RANDOM-EX",
];
pub(super) static DPRANDOM: &[&str] = &["NORMAL", "FLIP"];
pub(super) static GRAPHTYPESTR: &[&str] = &["NOTETYPE", "JUDGE", "EARLYLATE"];

/// Colors used in practice configuration UI drawing.
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
