/// BGA expand modes (from Config)
///
/// Moved from rubato-play::skin::bga to break the rubato-skin -> rubato-play dependency.
pub const BGAEXPAND_FULL: i32 = 0;
pub const BGAEXPAND_KEEP_ASPECT_RATIO: i32 = 1;
pub const BGAEXPAND_OFF: i32 = 2;

/// BGA stretch types for aspect-ratio correction.
/// Subset of the full StretchType, covering only the 3 modes
/// used by BGA expand config.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StretchType {
    Stretch,
    KeepAspectRatioFitInner,
    KeepAspectRatioNoExpanding,
}

/// Renderer type hint for BGA drawing.
/// Corresponds to SkinObjectRenderer.TYPE_* constants.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BgaRenderType {
    /// Linear filtering (for static images and miss layer)
    Linear,
    /// FFmpeg shader (for movie frames)
    Ffmpeg,
    /// Layer blending (for static image layers)
    Layer,
}
