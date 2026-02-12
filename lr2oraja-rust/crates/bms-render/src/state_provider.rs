// SkinStateProvider trait and StaticStateProvider for testing/demo.
//
// Phase 11's MainState will implement SkinStateProvider with real game state.
// StaticStateProvider serves as a HashMap-backed implementation for Phase 10
// testing and demo purposes.

use std::collections::HashMap;

use bms_skin::property_id::{BooleanId, FloatId, IntegerId, StringId, TimerId};
use bms_skin::skin_object::SkinOffset;
use serde::{Deserialize, Serialize};

/// Interface for providing runtime state to the skin renderer.
///
/// Phase 11's MainState will implement this trait. In Phase 10,
/// [`StaticStateProvider`] serves as a testing/demo implementation.
pub trait SkinStateProvider: Send + Sync {
    /// Returns the elapsed time for a timer in milliseconds.
    /// Returns `None` if the timer is not active (object should be hidden).
    fn timer_value(&self, timer: TimerId) -> Option<i64>;

    /// Returns the integer property value. Returns 0 for unknown IDs.
    fn integer_value(&self, id: IntegerId) -> i32;

    /// Returns true if the integer property ID has an explicit value.
    fn has_integer_value(&self, _id: IntegerId) -> bool {
        true
    }

    /// Returns the float property value. Returns 0.0 for unknown IDs.
    fn float_value(&self, id: FloatId) -> f32;

    /// Returns true if the float property ID has an explicit value.
    fn has_float_value(&self, _id: FloatId) -> bool {
        true
    }

    /// Returns the string property value. Returns `None` for unknown IDs.
    fn string_value(&self, id: StringId) -> Option<String>;

    /// Returns the boolean property value. Returns `false` for unknown IDs.
    /// Handles [`BooleanId`] negation: if `id.is_negated()`, returns `!lookup(abs_id)`.
    fn boolean_value(&self, id: BooleanId) -> bool;

    /// Returns true if the boolean property ID has an explicit value.
    fn has_boolean_value(&self, _id: BooleanId) -> bool {
        true
    }

    /// Returns the current time in milliseconds.
    fn now_time_ms(&self) -> i64;

    /// Returns the offset for the given ID. Returns default (all zeros) for unknown IDs.
    fn offset_value(&self, id: i32) -> SkinOffset;
}

/// A static state provider for testing and demo purposes.
/// All values are set via `HashMap`s.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct StaticStateProvider {
    pub timers: HashMap<i32, i64>,
    pub integers: HashMap<i32, i32>,
    pub floats: HashMap<i32, f32>,
    pub strings: HashMap<i32, String>,
    pub booleans: HashMap<i32, bool>,
    pub offsets: HashMap<i32, SkinOffset>,
    pub time_ms: i64,
}

impl SkinStateProvider for StaticStateProvider {
    fn timer_value(&self, timer: TimerId) -> Option<i64> {
        self.timers.get(&timer.0).copied()
    }

    fn integer_value(&self, id: IntegerId) -> i32 {
        self.integers.get(&id.0).copied().unwrap_or(0)
    }

    fn has_integer_value(&self, id: IntegerId) -> bool {
        self.integers.contains_key(&id.0)
    }

    fn float_value(&self, id: FloatId) -> f32 {
        self.floats.get(&id.0).copied().unwrap_or(0.0)
    }

    fn has_float_value(&self, id: FloatId) -> bool {
        self.floats.contains_key(&id.0)
    }

    fn string_value(&self, id: StringId) -> Option<String> {
        self.strings.get(&id.0).cloned()
    }

    fn boolean_value(&self, id: BooleanId) -> bool {
        let raw = self.booleans.get(&id.abs_id()).copied().unwrap_or(false);
        if id.is_negated() { !raw } else { raw }
    }

    fn has_boolean_value(&self, id: BooleanId) -> bool {
        self.booleans.contains_key(&id.abs_id())
    }

    fn now_time_ms(&self) -> i64 {
        self.time_ms
    }

    fn offset_value(&self, id: i32) -> SkinOffset {
        self.offsets.get(&id).copied().unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_provider() -> StaticStateProvider {
        StaticStateProvider::default()
    }

    #[test]
    fn timer_value_returns_some_when_set() {
        let mut p = make_provider();
        p.timers.insert(10, 5000);
        assert_eq!(p.timer_value(TimerId(10)), Some(5000));
    }

    #[test]
    fn timer_value_returns_none_when_not_set() {
        let p = make_provider();
        assert_eq!(p.timer_value(TimerId(99)), None);
    }

    #[test]
    fn integer_value_returns_value_when_set() {
        let mut p = make_provider();
        p.integers.insert(42, 123);
        assert_eq!(p.integer_value(IntegerId(42)), 123);
    }

    #[test]
    fn integer_value_returns_zero_when_not_set() {
        let p = make_provider();
        assert_eq!(p.integer_value(IntegerId(999)), 0);
    }

    #[test]
    fn float_value_returns_value_when_set() {
        let mut p = make_provider();
        p.floats.insert(7, 0.75);
        assert_eq!(p.float_value(FloatId(7)), 0.75);
    }

    #[test]
    fn float_value_returns_zero_when_not_set() {
        let p = make_provider();
        assert_eq!(p.float_value(FloatId(888)), 0.0);
    }

    #[test]
    fn string_value_returns_some_when_set() {
        let mut p = make_provider();
        p.strings.insert(1, "hello".to_string());
        assert_eq!(p.string_value(StringId(1)), Some("hello".to_string()));
    }

    #[test]
    fn string_value_returns_none_when_not_set() {
        let p = make_provider();
        assert_eq!(p.string_value(StringId(500)), None);
    }

    #[test]
    fn boolean_value_returns_true_when_set() {
        let mut p = make_provider();
        p.booleans.insert(5, true);
        assert!(p.boolean_value(BooleanId(5)));
    }

    #[test]
    fn boolean_value_returns_false_when_not_set() {
        let p = make_provider();
        assert!(!p.boolean_value(BooleanId(77)));
    }

    #[test]
    fn boolean_value_negation_inverts_lookup() {
        let mut p = make_provider();
        p.booleans.insert(5, true);
        // BooleanId(-5) should return !true = false
        assert!(!p.boolean_value(BooleanId(-5)));
    }

    #[test]
    fn boolean_value_negation_with_default() {
        let p = make_provider();
        // BooleanId(-99) → abs_id=99 → default false → negated = true
        assert!(p.boolean_value(BooleanId(-99)));
    }

    #[test]
    fn boolean_value_negation_inverts_false() {
        let mut p = make_provider();
        p.booleans.insert(3, false);
        // BooleanId(-3) should return !false = true
        assert!(p.boolean_value(BooleanId(-3)));
    }

    #[test]
    fn offset_value_returns_value_when_set() {
        let mut p = make_provider();
        let offset = SkinOffset {
            x: 10.0,
            y: 20.0,
            w: 30.0,
            h: 40.0,
            r: 45.0,
            a: 128.0,
        };
        p.offsets.insert(2, offset);
        let result = p.offset_value(2);
        assert_eq!(result.x, 10.0);
        assert_eq!(result.y, 20.0);
        assert_eq!(result.w, 30.0);
        assert_eq!(result.h, 40.0);
        assert_eq!(result.r, 45.0);
        assert_eq!(result.a, 128.0);
    }

    #[test]
    fn offset_value_returns_default_when_not_set() {
        let p = make_provider();
        let result = p.offset_value(999);
        assert_eq!(result.x, 0.0);
        assert_eq!(result.y, 0.0);
        assert_eq!(result.w, 0.0);
        assert_eq!(result.h, 0.0);
        assert_eq!(result.r, 0.0);
        assert_eq!(result.a, 0.0);
    }

    #[test]
    fn now_time_ms_returns_set_value() {
        let mut p = make_provider();
        p.time_ms = 12345;
        assert_eq!(p.now_time_ms(), 12345);
    }

    #[test]
    fn multiple_values_set_simultaneously() {
        let mut p = make_provider();
        p.timers.insert(1, 100);
        p.timers.insert(2, 200);
        p.integers.insert(10, 42);
        p.integers.insert(20, -7);
        p.floats.insert(1, 1.5);
        p.booleans.insert(1, true);
        p.booleans.insert(2, false);
        p.strings.insert(1, "test".to_string());
        p.time_ms = 9999;

        assert_eq!(p.timer_value(TimerId(1)), Some(100));
        assert_eq!(p.timer_value(TimerId(2)), Some(200));
        assert_eq!(p.integer_value(IntegerId(10)), 42);
        assert_eq!(p.integer_value(IntegerId(20)), -7);
        assert_eq!(p.float_value(FloatId(1)), 1.5);
        assert!(p.boolean_value(BooleanId(1)));
        assert!(!p.boolean_value(BooleanId(2)));
        assert_eq!(p.string_value(StringId(1)), Some("test".to_string()));
        assert_eq!(p.now_time_ms(), 9999);
    }

    #[test]
    fn default_provider_has_zero_time() {
        let p = make_provider();
        assert_eq!(p.now_time_ms(), 0);
    }
}
