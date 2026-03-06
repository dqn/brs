/// Semantic newtype for timer IDs used throughout the skin/state system.
///
/// Wraps the raw `i32` timer ID to prevent accidental misuse of unrelated
/// integer values as timer identifiers.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct TimerId(pub i32);

impl TimerId {
    pub const UNDEFINED: TimerId = TimerId(i32::MIN);

    pub fn new(id: i32) -> Self {
        Self(id)
    }

    pub fn as_i32(self) -> i32 {
        self.0
    }

    pub fn as_index(self) -> Option<usize> {
        if self.0 >= 0 {
            Some(self.0 as usize)
        } else {
            None
        }
    }
}

impl From<i32> for TimerId {
    fn from(v: i32) -> Self {
        Self(v)
    }
}

impl From<TimerId> for i32 {
    fn from(v: TimerId) -> Self {
        v.0
    }
}

impl std::fmt::Display for TimerId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TimerId({})", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_and_as_i32() {
        let id = TimerId::new(42);
        assert_eq!(id.as_i32(), 42);
        assert_eq!(id.0, 42);
    }

    #[test]
    fn test_undefined() {
        assert_eq!(TimerId::UNDEFINED.as_i32(), i32::MIN);
    }

    #[test]
    fn test_default_is_zero() {
        assert_eq!(TimerId::default().as_i32(), 0);
    }

    #[test]
    fn test_as_index() {
        assert_eq!(TimerId::new(5).as_index(), Some(5));
        assert_eq!(TimerId::new(0).as_index(), Some(0));
        assert_eq!(TimerId::new(-1).as_index(), None);
        assert_eq!(TimerId::UNDEFINED.as_index(), None);
    }

    #[test]
    fn test_from_i32() {
        let id: TimerId = 10.into();
        assert_eq!(id.as_i32(), 10);
    }

    #[test]
    fn test_into_i32() {
        let id = TimerId::new(7);
        let v: i32 = id.into();
        assert_eq!(v, 7);
    }

    #[test]
    fn test_eq_and_ord() {
        assert!(TimerId::new(1) < TimerId::new(2));
        assert_eq!(TimerId::new(3), TimerId::new(3));
    }

    #[test]
    fn test_hash() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(TimerId::new(1));
        set.insert(TimerId::new(2));
        set.insert(TimerId::new(1));
        assert_eq!(set.len(), 2);
    }

    #[test]
    fn test_display() {
        assert_eq!(format!("{}", TimerId::new(42)), "TimerId(42)");
    }
}
