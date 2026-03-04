use crate::skin_image::SkinImage;

/// Bar data extracted from select skin loaders (LR2, JSON).
/// Transferred to MusicSelector after skin loading so BarRenderer can use it.
pub struct SelectBarData {
    /// Bar body images for the selected (focused) bar
    pub barimageon: Vec<Option<SkinImage>>,
    /// Bar body images for non-selected bars
    pub barimageoff: Vec<Option<SkinImage>>,
    /// Center bar index (which bar slot is the cursor)
    pub center_bar: i32,
    /// Clickable bar indices
    pub clickable_bar: Vec<i32>,
}
