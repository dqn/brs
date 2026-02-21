/// BGA skin object
pub struct SkinBGA {
    bga_expand: i32,
    time: i64,
}

/// BGA expand modes (from Config)
pub const BGAEXPAND_FULL: i32 = 0;
pub const BGAEXPAND_KEEP_ASPECT_RATIO: i32 = 1;
pub const BGAEXPAND_OFF: i32 = 2;

/// Stretch types
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StretchType {
    Stretch,
    KeepAspectRatioFitInner,
    KeepAspectRatioNoExpanding,
}

impl SkinBGA {
    pub fn new(bga_expand: i32) -> Self {
        SkinBGA {
            bga_expand,
            time: 0,
        }
    }

    pub fn get_stretch_type(&self) -> StretchType {
        match self.bga_expand {
            BGAEXPAND_FULL => StretchType::Stretch,
            BGAEXPAND_KEEP_ASPECT_RATIO => StretchType::KeepAspectRatioFitInner,
            BGAEXPAND_OFF => StretchType::KeepAspectRatioNoExpanding,
            _ => StretchType::Stretch,
        }
    }

    pub fn prepare(&mut self, time: i64) {
        self.time = time;
        // TODO: Phase 7+ dependency - requires BMSPlayer state, BGAProcessor
    }

    pub fn draw(&self) {
        // TODO: Phase 7+ dependency - requires SkinObjectRenderer, PlayerResource, BGAManager
    }

    pub fn dispose(&mut self) {
        // no resources to dispose in Rust translation
    }
}
