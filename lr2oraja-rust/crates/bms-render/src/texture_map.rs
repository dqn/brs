// Texture map: ImageHandle → Bevy Handle<Image> + dimensions.
//
// Maps skin ImageHandle IDs to Bevy texture handles and dimensions,
// used by the skin renderer to look up GPU textures.

use bevy::prelude::*;
use bms_skin::image_handle::ImageHandle;

/// Entry in the texture map storing a Bevy image handle and dimensions.
#[derive(Debug, Clone)]
pub struct TextureEntry {
    pub handle: Handle<Image>,
    pub width: f32,
    pub height: f32,
}

/// Maps skin ImageHandle IDs to Bevy texture handles and dimensions.
#[derive(Debug, Default)]
pub struct TextureMap {
    entries: Vec<Option<TextureEntry>>,
}

impl TextureMap {
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert a texture entry for the given image handle.
    pub fn insert(
        &mut self,
        handle: ImageHandle,
        bevy_handle: Handle<Image>,
        width: f32,
        height: f32,
    ) {
        let idx = handle.0 as usize;
        if idx >= self.entries.len() {
            self.entries.resize_with(idx + 1, || None);
        }
        self.entries[idx] = Some(TextureEntry {
            handle: bevy_handle,
            width,
            height,
        });
    }

    /// Get the texture entry for the given image handle.
    pub fn get(&self, handle: ImageHandle) -> Option<&TextureEntry> {
        if !handle.is_valid() {
            return None;
        }
        self.entries.get(handle.0 as usize)?.as_ref()
    }

    /// Get the dimensions of a texture.
    pub fn dimensions(&self, handle: ImageHandle) -> Option<(f32, f32)> {
        self.get(handle).map(|e| (e.width, e.height))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_handle() -> Handle<Image> {
        Handle::default()
    }

    #[test]
    fn test_insert_and_get() {
        let mut map = TextureMap::new();
        let bh = dummy_handle();
        map.insert(ImageHandle(0), bh.clone(), 100.0, 200.0);
        let entry = map.get(ImageHandle(0)).unwrap();
        assert_eq!(entry.width, 100.0);
        assert_eq!(entry.height, 200.0);
    }

    #[test]
    fn test_get_missing() {
        let map = TextureMap::new();
        assert!(map.get(ImageHandle(5)).is_none());
    }

    #[test]
    fn test_get_none_handle() {
        let mut map = TextureMap::new();
        map.insert(ImageHandle(0), dummy_handle(), 10.0, 10.0);
        assert!(map.get(ImageHandle::NONE).is_none());
    }

    #[test]
    fn test_multiple_entries() {
        let mut map = TextureMap::new();
        map.insert(ImageHandle(0), dummy_handle(), 100.0, 100.0);
        map.insert(ImageHandle(3), dummy_handle(), 200.0, 300.0);
        map.insert(ImageHandle(7), dummy_handle(), 50.0, 75.0);

        assert!(map.get(ImageHandle(0)).is_some());
        assert!(map.get(ImageHandle(1)).is_none());
        assert!(map.get(ImageHandle(3)).is_some());
        assert_eq!(map.get(ImageHandle(7)).unwrap().width, 50.0);
    }

    #[test]
    fn test_dimensions() {
        let mut map = TextureMap::new();
        map.insert(ImageHandle(2), dummy_handle(), 640.0, 480.0);
        assert_eq!(map.dimensions(ImageHandle(2)), Some((640.0, 480.0)));
        assert_eq!(map.dimensions(ImageHandle(99)), None);
    }

    #[test]
    fn test_overwrite() {
        let mut map = TextureMap::new();
        map.insert(ImageHandle(0), dummy_handle(), 100.0, 100.0);
        map.insert(ImageHandle(0), dummy_handle(), 200.0, 300.0);
        let entry = map.get(ImageHandle(0)).unwrap();
        assert_eq!(entry.width, 200.0);
        assert_eq!(entry.height, 300.0);
    }
}
