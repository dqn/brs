use crate::skin::reexports::Rectangle;

/// Parsed play skin properties from the LR2 skin loader.
/// This is a type-erased data struct that allows the caller to apply
/// values to the concrete PlaySkin type without rubato-skin needing
/// to know about it.
pub struct PlaySkinProperties {
    pub close: Option<i32>,
    pub playstart: Option<i32>,
    pub loadstart: Option<i32>,
    pub loadend: Option<i32>,
    pub finish_margin: Option<i32>,
    pub judgetimer: Option<i32>,
    pub note_expansion_rate: Option<[i32; 2]>,
    pub judgeregion: Option<i32>,
    pub laneregion: Option<Vec<Rectangle>>,
    pub lanegroupregion: Option<Vec<Rectangle>>,
    pub line_count: usize,
}
