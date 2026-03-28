// SkinBar wrapper for SkinObject enum (Phase 32b)
// Minimal SkinBar type in beatoraja-skin to avoid circular dependency with beatoraja-select.
// The full SkinBar implementation lives in beatoraja-select::skin_bar.

use crate::skin::reexports::MainState;
use crate::skin::types::skin_object::{DestinationParams, SkinObjectData, SkinObjectRenderer};

/// SkinBar skin object — minimal wrapper with SkinObjectData for the skin pipeline.
/// The full bar rendering logic lives in beatoraja-select::skin_bar::SkinBar.
pub struct SkinBarObject {
    pub data: SkinObjectData,
    /// Position mode (0 = normal, 1 = reverse)
    pub position: i32,
}

impl SkinBarObject {
    pub fn new(position: i32) -> Self {
        let mut data = SkinObjectData::new();
        // Java: this.setDestination(0, 0, 0, 0, 0, 0, 0, 255, 255, 255, 0, 0, 0, 0, 0, 0, new int[0]);
        data.set_destination_with_int_timer_ops(
            &DestinationParams {
                time: 0,
                x: 0.0,
                y: 0.0,
                w: 0.0,
                h: 0.0,
                acc: 0,
                a: 0,
                r: 255,
                g: 255,
                b: 255,
                blend: 0,
                filter: 0,
                angle: 0,
                center: 0,
                loop_val: 0,
            },
            0,
            &[],
        );
        Self { data, position }
    }

    pub fn prepare(&mut self, time: i64, state: &dyn MainState) {
        self.data.prepare(time, state);
    }

    pub fn draw_impl(&mut self, _sprite: &mut SkinObjectRenderer) {
        // Intentionally empty: actual bar rendering is delegated to
        // BarRenderer in rubato-select, which owns the bar layout logic
        // and calls SkinObjectData methods directly on each bar element.
    }

    pub fn dispose(&mut self) {
        self.data.set_disposed();
    }
}

impl crate::skin::types::skin_node::SkinNode for SkinBarObject {
    fn data(&self) -> &SkinObjectData {
        &self.data
    }
    fn data_mut(&mut self) -> &mut SkinObjectData {
        &mut self.data
    }
    fn prepare(&mut self, time: i64, state: &dyn MainState) {
        SkinBarObject::prepare(self, time, state)
    }
    fn draw(&mut self, sprite: &mut SkinObjectRenderer, _state: &dyn MainState) {
        self.draw_impl(sprite)
    }
    fn dispose(&mut self) {
        SkinBarObject::dispose(self)
    }
    fn type_name(&self) -> &'static str {
        "SkinBar"
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
    use crate::skin::test_helpers::MockMainState;

    #[test]
    fn test_skin_bar_object_has_default_destination() {
        let bar = SkinBarObject::new(0);
        assert!(
            !bar.data.dst.is_empty(),
            "SkinBarObject must have default DST entry"
        );
    }

    #[test]
    fn test_skin_bar_object_two_phase_prepare_draw() {
        let mut bar = SkinBarObject::new(0);

        let state = MockMainState::default();

        // Phase 1: prepare — default destination ensures draw=true
        bar.prepare(0, &state);
        assert!(bar.data.draw);

        // Phase 2: draw — reads pre-computed state (stub does nothing but verifies signature)
        let mut renderer = SkinObjectRenderer::new();
        bar.draw_impl(&mut renderer);
    }

    #[test]
    fn test_skin_bar_object_position_preserved() {
        let bar = SkinBarObject::new(1);
        assert_eq!(bar.position, 1);
    }
}
