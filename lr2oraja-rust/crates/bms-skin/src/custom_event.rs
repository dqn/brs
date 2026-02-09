// Custom event and timer definitions ported from CustomEvent.java / CustomTimer.java.
//
// These are user-defined scripted behaviors that execute during skin updates.
// Actual execution logic depends on Phase 11 (State Machine); this module
// stores only the definition data.

use crate::property_id::{BooleanId, EventId, TimerId};

// ---------------------------------------------------------------------------
// CustomEventDef
// ---------------------------------------------------------------------------

/// Definition of a custom event that can be triggered by the skin.
///
/// Events have an action (EventId), an optional condition (BooleanId),
/// and a minimum interval between executions.
#[derive(Debug, Clone)]
pub struct CustomEventDef {
    /// Event identifier.
    pub id: EventId,
    /// Optional condition that must be true for auto-execution.
    pub condition: Option<BooleanId>,
    /// Minimum interval between auto-executions (milliseconds).
    pub min_interval: i32,
}

impl CustomEventDef {
    pub fn new(id: EventId, condition: Option<BooleanId>, min_interval: i32) -> Self {
        Self {
            id,
            condition,
            min_interval,
        }
    }
}

// ---------------------------------------------------------------------------
// CustomTimerDef
// ---------------------------------------------------------------------------

/// Definition of a custom timer that can be read/written by the skin.
///
/// Timers can be "active" (driven by a timer function) or "passive"
/// (set externally via `setMicroCustomTimer`).
#[derive(Debug, Clone)]
pub struct CustomTimerDef {
    /// Timer identifier.
    pub id: TimerId,
    /// If Some, this timer is driven by the referenced timer property.
    /// If None, this is a passive timer (externally controlled).
    pub timer_func: Option<TimerId>,
}

impl CustomTimerDef {
    /// Creates an active timer driven by a timer function.
    pub fn active(id: TimerId, func: TimerId) -> Self {
        Self {
            id,
            timer_func: Some(func),
        }
    }

    /// Creates a passive timer (externally controlled).
    pub fn passive(id: TimerId) -> Self {
        Self {
            id,
            timer_func: None,
        }
    }

    /// Returns true if this is a passive (externally controlled) timer.
    pub fn is_passive(&self) -> bool {
        self.timer_func.is_none()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_custom_event() {
        let evt = CustomEventDef::new(EventId(1000), Some(BooleanId(100)), 500);
        assert_eq!(evt.id, EventId(1000));
        assert_eq!(evt.condition, Some(BooleanId(100)));
        assert_eq!(evt.min_interval, 500);
    }

    #[test]
    fn test_custom_timer_active() {
        let timer = CustomTimerDef::active(TimerId(10000), TimerId(41));
        assert!(!timer.is_passive());
        assert_eq!(timer.timer_func, Some(TimerId(41)));
    }

    #[test]
    fn test_custom_timer_passive() {
        let timer = CustomTimerDef::passive(TimerId(10001));
        assert!(timer.is_passive());
        assert_eq!(timer.timer_func, None);
    }
}
