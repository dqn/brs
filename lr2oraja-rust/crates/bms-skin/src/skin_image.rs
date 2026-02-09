// SkinImage ported from SkinImage.java.
//
// Displays single or multiple images with optional animation based on
// an integer property selector. Actual texture management is deferred to
// Phase 10 (Bevy rendering).

use crate::image_handle::ImageHandle;
use crate::property_id::IntegerId;
use crate::skin_object::SkinObjectBase;

// ---------------------------------------------------------------------------
// SkinImage
// ---------------------------------------------------------------------------

/// A skin image object that displays one or more image sources.
///
/// When `ref_id` is set, the integer property value selects which source
/// to display (index into `sources`). Otherwise, the first source is used.
#[derive(Debug, Clone, Default)]
pub struct SkinImage {
    /// Base animation/destination properties.
    pub base: SkinObjectBase,
    /// Image source references. Each element is a source that produces
    /// a single texture (via animation frames, timer, cycle).
    pub sources: Vec<SkinImageSource>,
    /// Optional integer property that selects which source to display.
    pub ref_id: Option<IntegerId>,
    /// Whether this is a movie source (affects rendering pipeline).
    pub is_movie: bool,
}

/// A single image source entry for SkinImage.
#[derive(Debug, Clone)]
pub enum SkinImageSource {
    /// Reference to a globally loaded image by ID.
    Reference(i32),
    /// Inline animation frames with timer and cycle.
    Frames {
        images: Vec<ImageHandle>,
        timer: Option<i32>,
        cycle: i32,
    },
}

impl SkinImage {
    /// Creates a SkinImage referencing a single global image ID.
    pub fn from_reference(image_id: i32) -> Self {
        Self {
            sources: vec![SkinImageSource::Reference(image_id)],
            ..Default::default()
        }
    }

    /// Creates a SkinImage with inline animation frames.
    pub fn from_frames(images: Vec<ImageHandle>, timer: Option<i32>, cycle: i32) -> Self {
        Self {
            sources: vec![SkinImageSource::Frames {
                images,
                timer,
                cycle,
            }],
            ..Default::default()
        }
    }

    /// Creates a SkinImage with multiple source sets and a ref selector.
    pub fn with_ref(sources: Vec<SkinImageSource>, ref_id: IntegerId) -> Self {
        Self {
            sources,
            ref_id: Some(ref_id),
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_reference() {
        let img = SkinImage::from_reference(42);
        assert_eq!(img.sources.len(), 1);
        assert!(img.ref_id.is_none());
        assert!(!img.is_movie);
    }

    #[test]
    fn test_from_frames() {
        let handles = vec![ImageHandle(1), ImageHandle(2)];
        let img = SkinImage::from_frames(handles, Some(41), 1000);
        assert_eq!(img.sources.len(), 1);
        if let SkinImageSource::Frames {
            images,
            timer,
            cycle,
        } = &img.sources[0]
        {
            assert_eq!(images.len(), 2);
            assert_eq!(*timer, Some(41));
            assert_eq!(*cycle, 1000);
        } else {
            panic!("Expected Frames variant");
        }
    }

    #[test]
    fn test_with_ref() {
        let sources = vec![SkinImageSource::Reference(1), SkinImageSource::Reference(2)];
        let img = SkinImage::with_ref(sources, IntegerId(100));
        assert_eq!(img.sources.len(), 2);
        assert_eq!(img.ref_id, Some(IntegerId(100)));
    }
}
