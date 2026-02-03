/// BGA change event.
#[derive(Debug, Clone)]
pub struct BgaEvent {
    pub time_ms: f64,
    pub bga_id: u16,
    pub layer: BgaLayer,
}

/// BGA layer type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BgaLayer {
    /// Main BGA layer.
    Base,
    /// Layer that appears on top of base.
    Layer,
    /// Second overlay layer (on top of Layer).
    Layer2,
    /// Layer that appears on poor judgment.
    Poor,
}
