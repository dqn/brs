/// Semantic newtype for skin property value IDs.
///
/// Wraps the raw `i32` used as integer/image-index property IDs throughout the
/// skin property system (`integer_value`, `image_index_value`, etc.).
///
/// Provides `From<i32>` / `Into<i32>` for gradual migration at crate boundaries.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct ValueId(pub i32);

impl ValueId {
    /// Sentinel for "no value ID" / invalid.
    pub const UNDEFINED: ValueId = ValueId(i32::MIN);

    /// Create a new ValueId from a raw i32.
    pub fn new(id: i32) -> Self {
        Self(id)
    }

    /// Extract the raw i32.
    pub fn as_i32(self) -> i32 {
        self.0
    }

    /// Convert to a `usize` index, returning `None` for negative IDs.
    pub fn as_index(self) -> Option<usize> {
        if self.0 >= 0 {
            Some(self.0 as usize)
        } else {
            None
        }
    }
}

impl From<i32> for ValueId {
    fn from(id: i32) -> Self {
        Self(id)
    }
}

impl From<ValueId> for i32 {
    fn from(id: ValueId) -> Self {
        id.0
    }
}

impl std::fmt::Display for ValueId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ValueId({})", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_and_as_i32() {
        let id = ValueId::new(42);
        assert_eq!(id.as_i32(), 42);
        assert_eq!(id.0, 42);
    }

    #[test]
    fn test_from_i32() {
        let id: ValueId = 99.into();
        assert_eq!(id.as_i32(), 99);
    }

    #[test]
    fn test_into_i32() {
        let id = ValueId::new(7);
        let raw: i32 = id.into();
        assert_eq!(raw, 7);
    }

    #[test]
    fn test_as_index() {
        assert_eq!(ValueId::new(0).as_index(), Some(0));
        assert_eq!(ValueId::new(100).as_index(), Some(100));
        assert_eq!(ValueId::new(-1).as_index(), None);
        assert_eq!(ValueId::UNDEFINED.as_index(), None);
    }

    #[test]
    fn test_undefined() {
        assert_eq!(ValueId::UNDEFINED.as_i32(), i32::MIN);
    }

    #[test]
    fn test_default() {
        assert_eq!(ValueId::default(), ValueId(0));
    }

    #[test]
    fn test_display() {
        assert_eq!(format!("{}", ValueId::new(42)), "ValueId(42)");
    }

    #[test]
    fn test_ord() {
        assert!(ValueId::new(1) < ValueId::new(2));
        assert!(ValueId::new(5) > ValueId::new(3));
    }
}
