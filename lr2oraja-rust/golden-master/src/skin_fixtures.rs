// Skin snapshot infrastructure for golden-master testing.
//
// Converts a Skin into a lightweight summary (SkinSnapshot) for
// structural comparison without requiring Serialize on all skin types.

use std::collections::BTreeMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

use bms_skin::skin::Skin;
use bms_skin::skin_object_type::SkinObjectType;

// ---------------------------------------------------------------------------
// Snapshot types
// ---------------------------------------------------------------------------

/// Lightweight snapshot of a Skin for structural comparison.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SkinSnapshot {
    // header
    pub name: String,
    pub skin_type_id: Option<i32>,
    pub resolution_w: i32,
    pub resolution_h: i32,
    pub option_count: usize,
    pub file_count: usize,
    pub offset_count: usize,
    // skin
    pub width: f32,
    pub height: f32,
    pub scale_x: f32,
    pub scale_y: f32,
    pub input: i32,
    pub scene: i32,
    pub fadeout: i32,
    pub object_count: usize,
    /// Object counts by type name (e.g. "Image" -> 147).
    pub objects_by_type: BTreeMap<String, usize>,
    pub custom_event_count: usize,
    pub custom_timer_count: usize,
    /// Per-object summaries.
    pub objects: Vec<ObjectSnapshot>,
}

/// Summary of a single SkinObject.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ObjectSnapshot {
    pub kind: String,
    pub destination_count: usize,
    pub timer_id: Option<i32>,
    pub blend: i32,
    pub first_dst: Option<DstSnapshot>,
}

/// Summary of a Destination keyframe.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DstSnapshot {
    pub time: i64,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub a: f32,
    pub angle: i32,
}

// ---------------------------------------------------------------------------
// Conversion
// ---------------------------------------------------------------------------

fn object_type_name(obj: &SkinObjectType) -> &'static str {
    match obj {
        SkinObjectType::Image(_) => "Image",
        SkinObjectType::Number(_) => "Number",
        SkinObjectType::Text(_) => "Text",
        SkinObjectType::Slider(_) => "Slider",
        SkinObjectType::Graph(_) => "Graph",
        SkinObjectType::Gauge(_) => "Gauge",
        SkinObjectType::BpmGraph(_) => "BpmGraph",
        SkinObjectType::HitErrorVisualizer(_) => "HitErrorVisualizer",
        SkinObjectType::NoteDistributionGraph(_) => "NoteDistributionGraph",
        SkinObjectType::TimingDistributionGraph(_) => "TimingDistributionGraph",
        SkinObjectType::TimingVisualizer(_) => "TimingVisualizer",
    }
}

fn object_snapshot(obj: &SkinObjectType) -> ObjectSnapshot {
    let base = obj.base();
    let first_dst = base.destinations.first().map(|d| DstSnapshot {
        time: d.time,
        x: d.region.x,
        y: d.region.y,
        w: d.region.w,
        h: d.region.h,
        a: d.color.a,
        angle: d.angle,
    });
    ObjectSnapshot {
        kind: object_type_name(obj).to_string(),
        destination_count: base.destinations.len(),
        timer_id: base.timer.map(|t| t.0),
        blend: base.blend,
        first_dst,
    }
}

/// Converts a Skin into a SkinSnapshot for comparison.
pub fn snapshot_from_skin(skin: &Skin) -> SkinSnapshot {
    let mut objects_by_type = BTreeMap::new();
    for obj in &skin.objects {
        *objects_by_type
            .entry(object_type_name(obj).to_string())
            .or_insert(0) += 1;
    }

    let objects: Vec<ObjectSnapshot> = skin.objects.iter().map(object_snapshot).collect();

    SkinSnapshot {
        name: skin.header.name.clone(),
        skin_type_id: skin.header.skin_type.map(|t| t.id()),
        resolution_w: skin.header.resolution.width(),
        resolution_h: skin.header.resolution.height(),
        option_count: skin.header.options.len(),
        file_count: skin.header.files.len(),
        offset_count: skin.header.offsets.len(),
        width: skin.width,
        height: skin.height,
        scale_x: skin.scale_x,
        scale_y: skin.scale_y,
        input: skin.input,
        scene: skin.scene,
        fadeout: skin.fadeout,
        object_count: skin.objects.len(),
        objects_by_type,
        custom_event_count: skin.custom_events.len(),
        custom_timer_count: skin.custom_timers.len(),
        objects,
    }
}

// ---------------------------------------------------------------------------
// Snapshot comparison helpers
// ---------------------------------------------------------------------------

/// Loads a snapshot fixture from a JSON file.
pub fn load_snapshot(path: &Path) -> anyhow::Result<SkinSnapshot> {
    let content = std::fs::read_to_string(path)?;
    Ok(serde_json::from_str(&content)?)
}

/// Saves a snapshot fixture to a JSON file.
pub fn save_snapshot(snapshot: &SkinSnapshot, path: &Path) -> anyhow::Result<()> {
    let json = serde_json::to_string_pretty(snapshot)?;
    std::fs::write(path, json)?;
    Ok(())
}

/// Returns true if the UPDATE_SNAPSHOTS env var is set.
pub fn should_update_snapshots() -> bool {
    std::env::var("UPDATE_SNAPSHOTS").is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use bms_skin::skin_header::SkinHeader;

    #[test]
    fn test_empty_skin_snapshot() {
        let skin = Skin::new(SkinHeader::default());
        let snap = snapshot_from_skin(&skin);
        assert_eq!(snap.object_count, 0);
        assert!(snap.objects_by_type.is_empty());
        assert!(snap.objects.is_empty());
    }

    #[test]
    fn test_snapshot_with_objects() {
        use bms_skin::skin_image::SkinImage;
        use bms_skin::skin_number::SkinNumber;

        let mut skin = Skin::new(SkinHeader::default());
        skin.add(SkinImage::default().into());
        skin.add(SkinImage::from_reference(1).into());
        skin.add(SkinNumber::default().into());

        let snap = snapshot_from_skin(&skin);
        assert_eq!(snap.object_count, 3);
        assert_eq!(snap.objects_by_type.get("Image"), Some(&2));
        assert_eq!(snap.objects_by_type.get("Number"), Some(&1));
        assert_eq!(snap.objects.len(), 3);
        assert_eq!(snap.objects[0].kind, "Image");
        assert_eq!(snap.objects[2].kind, "Number");
    }

    #[test]
    fn test_snapshot_serde_round_trip() {
        let skin = Skin::new(SkinHeader::default());
        let snap = snapshot_from_skin(&skin);
        let json = serde_json::to_string(&snap).unwrap();
        let back: SkinSnapshot = serde_json::from_str(&json).unwrap();
        assert_eq!(snap, back);
    }
}
