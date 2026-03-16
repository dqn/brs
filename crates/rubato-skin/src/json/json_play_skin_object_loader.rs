// Mechanical translation of JsonPlaySkinObjectLoader.java
// Play skin object loader (handles note, judge, hidden cover, BGA, etc.)

use std::path::Path;

use crate::json::json_skin;
use crate::json::json_skin_loader::{JSONSkinLoader, SkinData, SkinObjectData, SkinObjectType};
use crate::json::json_skin_object_loader::{self, JsonSkinObjectLoader};

/// Corresponds to JsonPlaySkinObjectLoader extends JsonSkinObjectLoader<PlaySkin>
pub struct JsonPlaySkinObjectLoader;

impl JsonSkinObjectLoader for JsonPlaySkinObjectLoader {
    fn skin(&self, header: &crate::json::json_skin_loader::SkinHeaderData) -> SkinData {
        // Corresponds to Java: new PlaySkin(header)
        let skin_type = crate::skin_type::SkinType::skin_type_by_id(header.skin_type)
            .unwrap_or(crate::skin_type::SkinType::Play7Keys);
        SkinData::from_header(header, skin_type)
    }

    fn load_skin_object(
        &self,
        loader: &mut JSONSkinLoader,
        skin: &SkinData,
        sk: &json_skin::Skin,
        dst: &json_skin::Destination,
        p: &Path,
    ) -> Option<SkinObjectData> {
        // Try base loader first
        let obj = json_skin_object_loader::load_base_skin_object(loader, skin, sk, dst, p);
        if obj.is_some() {
            return obj;
        }

        let dst_id = dst.id.as_deref()?;

        // note (playskin only)
        if let Some(ref note) = sk.note
            && dst_id == note.id.as_deref().unwrap_or("")
        {
            use crate::json::json_skin_object_loader::utilities::note_texture;
            use crate::skin_note_object::SkinNoteObject;

            // Determine lane count from note.dst (per-lane regions) or note.note (image IDs)
            let lane_count = if !note.dst.is_empty() {
                note.dst.len()
            } else {
                note.note.len()
            };

            let mut note_obj = SkinNoteObject::new(lane_count);

            // Set lane regions from note.dst
            for (i, anim) in note.dst.iter().enumerate() {
                note_obj.inner.set_lane_region(
                    i,
                    &rubato_play::skin::note::LaneRegion {
                        x: anim.x as f32,
                        y: anim.y as f32,
                        width: anim.w as f32,
                        height: anim.h as f32,
                        scale: 1.0,
                        dstnote2: i32::MIN,
                    },
                );
            }

            // Resolve note textures (first frame of each lane's animation)
            let note_textures = note_texture(loader, &note.note, p);
            for (i, tex) in note_textures.iter().enumerate() {
                if let Some(regions) = tex
                    && let Some(first) = regions.first()
                    && i < note_obj.note_images.len()
                {
                    // Use note image height as scale (Java: scale = noteImage.getRegionHeight() * dy)
                    if note.size.get(i).copied().unwrap_or(0.0) > 0.0 {
                        note_obj.inner.lanes_mut()[i].scale = note.size[i];
                    } else {
                        note_obj.inner.lanes_mut()[i].scale = first.region_height as f32;
                    }
                    note_obj.note_images[i] = Some(first.clone());
                }
            }

            // Resolve mine textures
            let mine_textures = note_texture(loader, &note.mine, p);
            for (i, tex) in mine_textures.iter().enumerate() {
                if let Some(regions) = tex
                    && let Some(first) = regions.first()
                    && i < note_obj.mine_images.len()
                {
                    note_obj.mine_images[i] = Some(first.clone());
                }
            }

            log::debug!(
                "Note: lane_count={}, note_images_wired={}, mine_images_wired={}",
                lane_count,
                note_obj.note_images.iter().filter(|i| i.is_some()).count(),
                note_obj.mine_images.iter().filter(|i| i.is_some()).count(),
            );

            let obj = SkinObjectData {
                name: note.id.clone(),
                object_type: SkinObjectType::Note,
                resolved_note: Some(note_obj),
                ..Default::default()
            };
            return Some(obj);
        }

        // hidden cover (playskin only)
        for img in &sk.hidden_cover {
            if dst_id == img.id.as_deref().unwrap_or("") {
                let obj = SkinObjectData {
                    name: img.id.clone(),
                    object_type: SkinObjectType::HiddenCover {
                        src: img.src.clone(),
                        x: img.x,
                        y: img.y,
                        w: img.w,
                        h: img.h,
                        divx: img.divx,
                        divy: img.divy,
                        timer: img.timer,
                        cycle: img.cycle,
                        disapear_line: img.disapear_line,
                        is_disapear_line_link_lift: img.is_disapear_line_link_lift,
                    },
                    ..Default::default()
                };
                return Some(obj);
            }
        }

        // lift cover (playskin only)
        for img in &sk.lift_cover {
            if dst_id == img.id.as_deref().unwrap_or("") {
                let obj = SkinObjectData {
                    name: img.id.clone(),
                    object_type: SkinObjectType::LiftCover {
                        src: img.src.clone(),
                        x: img.x,
                        y: img.y,
                        w: img.w,
                        h: img.h,
                        divx: img.divx,
                        divy: img.divy,
                        timer: img.timer,
                        cycle: img.cycle,
                        disapear_line: img.disapear_line,
                        is_disapear_line_link_lift: img.is_disapear_line_link_lift,
                    },
                    ..Default::default()
                };
                return Some(obj);
            }
        }

        // bga (playskin only)
        if let Some(ref bga) = sk.bga
            && dst_id == bga.id.as_deref().unwrap_or("")
        {
            let obj = SkinObjectData {
                name: bga.id.clone(),
                object_type: SkinObjectType::Bga {
                    bga_expand: loader.bga_expand,
                },
                ..Default::default()
            };
            return Some(obj);
        }

        // judge (playskin only)
        for judge in &sk.judge {
            if dst_id == judge.id.as_deref().unwrap_or("") {
                let obj = SkinObjectData {
                    name: judge.id.clone(),
                    object_type: SkinObjectType::Judge {
                        index: judge.index,
                        shift: judge.shift,
                    },
                    ..Default::default()
                };
                return Some(obj);
            }
        }

        // POMYU chara
        for chara in &sk.pmchara {
            if dst_id == chara.id.as_deref().unwrap_or("") {
                let obj = SkinObjectData {
                    name: chara.id.clone(),
                    object_type: SkinObjectType::PmChara {
                        src: chara.src.clone(),
                        color: chara.color,
                        chara_type: chara.chara_type,
                        side: chara.side,
                    },
                    ..Default::default()
                };
                return Some(obj);
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::json::json_skin_loader::SkinHeaderData;
    use crate::json::json_skin_object_loader::JsonSkinObjectLoader;
    use crate::skin_type::SkinType;

    fn make_header(skin_type_id: i32) -> SkinHeaderData {
        SkinHeaderData {
            skin_type: skin_type_id,
            name: "Test Play Skin".to_string(),
            ..Default::default()
        }
    }

    #[test]
    fn test_get_skin_returns_play7keys_for_7key_header() {
        let loader = JsonPlaySkinObjectLoader;
        let header = make_header(SkinType::Play7Keys.id());
        let skin = loader.skin(&header);
        assert_eq!(skin.skin_type, Some(SkinType::Play7Keys));
        assert!(skin.header.is_some());
        assert_eq!(skin.header.unwrap().name, "Test Play Skin");
    }

    #[test]
    fn test_get_skin_returns_play5keys_for_5key_header() {
        let loader = JsonPlaySkinObjectLoader;
        let header = make_header(SkinType::Play5Keys.id());
        let skin = loader.skin(&header);
        assert_eq!(skin.skin_type, Some(SkinType::Play5Keys));
    }

    #[test]
    fn test_get_skin_returns_play14keys_for_14key_header() {
        let loader = JsonPlaySkinObjectLoader;
        let header = make_header(SkinType::Play14Keys.id());
        let skin = loader.skin(&header);
        assert_eq!(skin.skin_type, Some(SkinType::Play14Keys));
    }

    #[test]
    fn test_get_skin_returns_play24keys_for_24key_header() {
        let loader = JsonPlaySkinObjectLoader;
        let header = make_header(SkinType::Play24Keys.id());
        let skin = loader.skin(&header);
        assert_eq!(skin.skin_type, Some(SkinType::Play24Keys));
    }

    #[test]
    fn test_get_skin_fallback_to_play7keys_for_unknown_id() {
        let loader = JsonPlaySkinObjectLoader;
        let header = make_header(-999);
        let skin = loader.skin(&header);
        assert_eq!(skin.skin_type, Some(SkinType::Play7Keys));
    }

    #[test]
    fn test_get_skin_default_fields_are_zero() {
        let loader = JsonPlaySkinObjectLoader;
        let header = make_header(SkinType::Play7Keys.id());
        let skin = loader.skin(&header);
        assert_eq!(skin.fadeout, 0);
        assert_eq!(skin.input, 0);
        assert_eq!(skin.scene, 0);
        assert!(skin.objects.is_empty());
    }
}
