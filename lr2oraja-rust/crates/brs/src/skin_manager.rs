// Skin loading state manager.

/// Skin types matching Java SkinType enum.
#[allow(dead_code)] // Reserved for future skin loading system
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkinType {
    MusicSelect,
    Decide,
    Play5,
    Play7,
    Play9,
    Play10,
    Play14,
    Play24,
    Result,
    CourseResult,
    KeyConfig,
    SkinConfig,
}

/// Manages skin loading requests and state.
#[allow(dead_code)] // Reserved for future skin loading system
#[derive(Default)]
pub struct SkinManager {
    /// Pending skin load request (set by states, consumed by system).
    request: Option<SkinType>,
    /// Whether the current skin is fully loaded.
    loaded: bool,
    /// Currently active skin type.
    current: Option<SkinType>,
}

#[allow(dead_code)] // Reserved for future skin loading system
impl SkinManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// Request a skin to be loaded.
    pub fn request_load(&mut self, skin_type: SkinType) {
        self.request = Some(skin_type);
        self.loaded = false;
    }

    /// Take the pending request (consumed by skin loading system).
    pub fn take_request(&mut self) -> Option<SkinType> {
        self.request.take()
    }

    /// Mark the current skin as loaded.
    pub fn mark_loaded(&mut self, skin_type: SkinType) {
        self.current = Some(skin_type);
        self.loaded = true;
    }

    pub fn is_loaded(&self) -> bool {
        self.loaded
    }

    pub fn current_type(&self) -> Option<SkinType> {
        self.current
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_starts_unloaded() {
        let mgr = SkinManager::new();
        assert!(!mgr.is_loaded());
        assert_eq!(mgr.current_type(), None);
    }

    #[test]
    fn request_load_sets_request() {
        let mut mgr = SkinManager::new();
        mgr.request_load(SkinType::Play7);
        assert_eq!(mgr.take_request(), Some(SkinType::Play7));
        assert!(!mgr.is_loaded());
    }

    #[test]
    fn take_request_clears_request() {
        let mut mgr = SkinManager::new();
        mgr.request_load(SkinType::MusicSelect);
        assert_eq!(mgr.take_request(), Some(SkinType::MusicSelect));
        assert_eq!(mgr.take_request(), None);
    }

    #[test]
    fn mark_loaded_sets_loaded_and_current() {
        let mut mgr = SkinManager::new();
        mgr.request_load(SkinType::Result);
        mgr.mark_loaded(SkinType::Result);
        assert!(mgr.is_loaded());
        assert_eq!(mgr.current_type(), Some(SkinType::Result));
    }
}
