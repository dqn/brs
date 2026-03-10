use std::collections::HashMap;
use std::path::Path;

use crate::json::json_skin_loader::{
    SkinObjectData as LoaderSkinObjectData, SongListBarData, SourceData,
};
use crate::objects::skin_image::SkinImage;
use crate::objects::skin_number::SkinNumber;
use crate::types::skin::SkinObject;

use super::object_converter::{apply_destinations, convert_skin_object};

/// Build SelectBarData from resolved JSON SongList bar sub-objects.
/// Each sub-SkinObjectData is converted to the appropriate skin type
/// (SkinImage, SkinNumber, SkinTextFont) and stored in SelectBarData.
pub(super) fn build_select_bar_data(
    bar_data: &SongListBarData,
    center: i32,
    clickable: &[i32],
    source_map: &mut HashMap<String, SourceData>,
    skin_path: &Path,
    usecim: bool,
    scale_y: f32,
) -> crate::select_bar_data::SelectBarData {
    crate::select_bar_data::SelectBarData {
        barimageon: convert_bar_sub_images(
            &bar_data.liston,
            source_map,
            skin_path,
            usecim,
            scale_y,
        ),
        barimageoff: convert_bar_sub_images(
            &bar_data.listoff,
            source_map,
            skin_path,
            usecim,
            scale_y,
        ),
        center_bar: center,
        clickable_bar: clickable.to_vec(),
        barlevel: convert_bar_sub_numbers(&bar_data.level, source_map, skin_path, usecim, scale_y),
        bartext: convert_bar_sub_text(&bar_data.text, source_map, skin_path, usecim, scale_y),
        barlamp: convert_bar_sub_images(&bar_data.lamp, source_map, skin_path, usecim, scale_y),
        barmylamp: convert_bar_sub_images(
            &bar_data.playerlamp,
            source_map,
            skin_path,
            usecim,
            scale_y,
        ),
        barrivallamp: convert_bar_sub_images(
            &bar_data.rivallamp,
            source_map,
            skin_path,
            usecim,
            scale_y,
        ),
        bartrophy: convert_bar_sub_images(&bar_data.trophy, source_map, skin_path, usecim, scale_y),
        barlabel: convert_bar_sub_images(&bar_data.label, source_map, skin_path, usecim, scale_y),
        // Known rendering gap: songlist.graph from JSON select skins is not propagated.
        // Fix: resolve bar_data.graph into graph_type/graph_images/graph_region here.
        graph_type: None,
        graph_images: None,
        graph_region: crate::stubs::Rectangle::default(),
    }
}

fn convert_bar_sub_images(
    objs: &[Option<LoaderSkinObjectData>],
    source_map: &mut HashMap<String, SourceData>,
    skin_path: &Path,
    usecim: bool,
    scale_y: f32,
) -> Vec<Option<SkinImage>> {
    objs.iter()
        .map(|opt_obj| {
            let obj_data = opt_obj.as_ref()?;
            let skin_obj = convert_skin_object(
                &obj_data.object_type,
                source_map,
                skin_path,
                usecim,
                scale_y,
            )?;
            if let SkinObject::Image(mut img) = skin_obj {
                apply_destinations(&mut img.data, &obj_data.destinations);
                Some(img)
            } else {
                None
            }
        })
        .collect()
}

fn convert_bar_sub_text(
    objs: &[Option<LoaderSkinObjectData>],
    source_map: &mut HashMap<String, SourceData>,
    skin_path: &Path,
    usecim: bool,
    scale_y: f32,
) -> Vec<Option<crate::skin_text::SkinTextEnum>> {
    objs.iter()
        .map(|opt_obj| {
            let obj_data = opt_obj.as_ref()?;
            let skin_obj = convert_skin_object(
                &obj_data.object_type,
                source_map,
                skin_path,
                usecim,
                scale_y,
            )?;
            if let SkinObject::TextFont(mut stf) = skin_obj {
                apply_destinations(&mut stf.text_data.data, &obj_data.destinations);
                Some(crate::skin_text::SkinTextEnum::Font(stf))
            } else {
                None
            }
        })
        .collect()
}

fn convert_bar_sub_numbers(
    objs: &[Option<LoaderSkinObjectData>],
    source_map: &mut HashMap<String, SourceData>,
    skin_path: &Path,
    usecim: bool,
    scale_y: f32,
) -> Vec<Option<SkinNumber>> {
    objs.iter()
        .map(|opt_obj| {
            let obj_data = opt_obj.as_ref()?;
            let skin_obj = convert_skin_object(
                &obj_data.object_type,
                source_map,
                skin_path,
                usecim,
                scale_y,
            )?;
            if let SkinObject::Number(mut num) = skin_obj {
                apply_destinations(&mut num.data, &obj_data.destinations);
                Some(num)
            } else {
                None
            }
        })
        .collect()
}
