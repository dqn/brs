// SkinBga ported from SkinBGA.java.
//
// BGA display object that renders BGA/layer/poor images with configurable
// aspect ratio handling.

use crate::skin_object::SkinObjectBase;
use crate::stretch_type::StretchType;

/// BGA expand mode (matches Java Config.BGAEXPAND_* constants).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BgaExpand {
    /// Stretch to fill destination.
    #[default]
    Full,
    /// Keep aspect ratio, fit inside destination.
    KeepAspectRatio,
    /// No expanding, keep original size.
    Off,
}

impl BgaExpand {
    /// Convert to the corresponding StretchType for rendering.
    pub fn to_stretch_type(self) -> StretchType {
        match self {
            Self::Full => StretchType::Stretch,
            Self::KeepAspectRatio => StretchType::KeepAspectRatioFitInner,
            Self::Off => StretchType::KeepAspectRatioNoExpanding,
        }
    }
}

/// BGA skin object.
#[derive(Debug, Clone, Default)]
pub struct SkinBga {
    /// Base animation/destination properties.
    pub base: SkinObjectBase,
    /// BGA expand mode.
    pub expand: BgaExpand,
}

impl SkinBga {
    /// Create a new SkinBga with the given expand mode.
    pub fn new(expand: BgaExpand) -> Self {
        let mut bga = Self {
            expand,
            ..Default::default()
        };
        bga.base.stretch = expand.to_stretch_type();
        bga
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_expand_is_full() {
        let bga = SkinBga::default();
        assert_eq!(bga.expand, BgaExpand::Full);
    }

    #[test]
    fn new_sets_stretch_type() {
        let bga = SkinBga::new(BgaExpand::KeepAspectRatio);
        assert_eq!(bga.base.stretch, StretchType::KeepAspectRatioFitInner);
    }

    #[test]
    fn expand_to_stretch_type() {
        assert_eq!(BgaExpand::Full.to_stretch_type(), StretchType::Stretch);
        assert_eq!(
            BgaExpand::KeepAspectRatio.to_stretch_type(),
            StretchType::KeepAspectRatioFitInner
        );
        assert_eq!(
            BgaExpand::Off.to_stretch_type(),
            StretchType::KeepAspectRatioNoExpanding
        );
    }

    #[test]
    fn new_with_off() {
        let bga = SkinBga::new(BgaExpand::Off);
        assert_eq!(bga.expand, BgaExpand::Off);
        assert_eq!(bga.base.stretch, StretchType::KeepAspectRatioNoExpanding);
    }
}
