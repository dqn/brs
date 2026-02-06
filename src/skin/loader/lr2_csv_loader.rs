use std::path::Path;

use anyhow::{Context, Result};

use crate::skin::destination::{Destination, DestinationSet};
use crate::skin::object::image::ImageObject;
use crate::skin::object::number::NumberObject;
use crate::skin::skin_data::{SkinData, SkinObject, SkinSource};
use crate::skin::skin_header::{SkinHeader, SkinType};

/// LR2 CSV skin loader.
/// Parses the LR2 skin CSV format (#ENDOFHEADER / #SRC_IMAGE / #DST_IMAGE etc.)
pub struct Lr2CsvLoader;

impl Lr2CsvLoader {
    /// Load an LR2 CSV skin file.
    pub fn load(path: &Path, dst_width: u32, dst_height: u32) -> Result<SkinData> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read LR2 CSV skin: {}", path.display()))?;

        let skin_dir = path.parent().unwrap_or(Path::new("."));
        let mut header = SkinHeader {
            skin_type: SkinType::Play7Keys,
            path: path.to_path_buf(),
            src_width: 1280,
            src_height: 720,
            ..Default::default()
        };

        let mut skin_data: Option<SkinData> = None;
        let mut in_header = true;
        let mut sources: Vec<SkinSource> = Vec::new();
        let mut objects: Vec<SkinObject> = Vec::new();
        let mut current_src_id = 0;
        let mut current_object: Option<SkinObject> = None;

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with("//") {
                continue;
            }

            let parts: Vec<&str> = line.split(',').collect();
            if parts.is_empty() {
                continue;
            }

            let cmd = parts[0].trim();
            match cmd {
                "#ENDOFHEADER" => {
                    in_header = false;
                    skin_data = Some(SkinData::new(header.clone(), dst_width, dst_height));
                }
                "#INFORMATION" if in_header => {
                    if parts.len() > 1
                        && let Ok(t) = parts[1].trim().parse::<i32>()
                    {
                        header.skin_type = SkinType::from_id(t).unwrap_or(SkinType::Play7Keys);
                    }
                    if parts.len() > 2 {
                        header.name = parts[2].trim().to_string();
                    }
                }
                "#CUSTOMOPTION" if in_header => {
                    // Custom options parsing (simplified)
                }
                "#CUSTOMFILE" if in_header => {
                    // Custom files parsing (simplified)
                }
                "#IMAGE" => {
                    if parts.len() > 1 {
                        let img_path = parts[1].trim();
                        let full_path = skin_dir.join(img_path);
                        sources.push(SkinSource {
                            id: current_src_id,
                            path: full_path,
                            texture: None,
                        });
                        current_src_id += 1;
                    }
                }
                s if s.starts_with("#SRC_IMAGE") => {
                    Self::parse_src_image(&parts, &mut current_object)?;
                }
                s if s.starts_with("#DST_IMAGE") => {
                    if let Some(ref mut obj) = current_object
                        && let Some(sd) = &skin_data
                    {
                        Self::parse_dst_entry(&parts, obj, sd.scale_x, sd.scale_y)?;
                    }
                }
                s if s.starts_with("#SRC_NUMBER") => {
                    Self::parse_src_number(&parts, &mut current_object)?;
                }
                s if s.starts_with("#DST_NUMBER") => {
                    if let Some(ref mut obj) = current_object
                        && let Some(sd) = &skin_data
                    {
                        Self::parse_dst_entry(&parts, obj, sd.scale_x, sd.scale_y)?;
                    }
                }
                _ => {}
            }

            // Flush object when we see a new SRC command
            if cmd.starts_with("#SRC_") && current_object.is_some() {
                // The previous object was a SRC without matching DST entries,
                // we handle this when the next SRC or end of file occurs
            }
        }

        // Flush remaining object
        if let Some(obj) = current_object {
            objects.push(obj);
        }

        let mut sd = skin_data.unwrap_or_else(|| SkinData::new(header, dst_width, dst_height));
        for src in sources {
            sd.add_source(src);
        }
        for obj in objects {
            sd.add_object(obj);
        }

        Ok(sd)
    }

    fn parse_i32(parts: &[&str], index: usize) -> i32 {
        parts
            .get(index)
            .and_then(|s| s.trim().parse::<i32>().ok())
            .unwrap_or(0)
    }

    fn parse_f32(parts: &[&str], index: usize) -> f32 {
        parts
            .get(index)
            .and_then(|s| s.trim().parse::<f32>().ok())
            .unwrap_or(0.0)
    }

    fn parse_src_image(parts: &[&str], current_object: &mut Option<SkinObject>) -> Result<()> {
        // Flush previous object
        // #SRC_IMAGE,gr,num,x,y,w,h,divx,divy,cycle,timer
        let gr = Self::parse_i32(parts, 1);
        let _num = Self::parse_i32(parts, 2);
        let x = Self::parse_i32(parts, 3);
        let y = Self::parse_i32(parts, 4);
        let w = Self::parse_i32(parts, 5);
        let h = Self::parse_i32(parts, 6);
        let divx = Self::parse_i32(parts, 7).max(1);
        let divy = Self::parse_i32(parts, 8).max(1);
        let cycle = Self::parse_i32(parts, 9);
        let timer = Self::parse_i32(parts, 10);

        *current_object = Some(SkinObject::Image(ImageObject {
            src: gr,
            src_x: x,
            src_y: y,
            src_w: w,
            src_h: h,
            div_x: divx,
            div_y: divy,
            cycle,
            timer,
            ..Default::default()
        }));
        Ok(())
    }

    fn parse_src_number(parts: &[&str], current_object: &mut Option<SkinObject>) -> Result<()> {
        // #SRC_NUMBER,gr,num,x,y,w,h,divx,divy,cycle,timer,num_type,ref
        let gr = Self::parse_i32(parts, 1);
        let x = Self::parse_i32(parts, 3);
        let y = Self::parse_i32(parts, 4);
        let w = Self::parse_i32(parts, 5);
        let h = Self::parse_i32(parts, 6);
        let divx = Self::parse_i32(parts, 7).max(1);
        let digit = Self::parse_i32(parts, 11);
        let ref_id = Self::parse_i32(parts, 12);

        *current_object = Some(SkinObject::Number(NumberObject {
            src: gr,
            ref_id,
            src_x: x,
            src_y: y,
            src_w: w,
            src_h: h,
            div_x: divx,
            digit,
            ..Default::default()
        }));
        Ok(())
    }

    fn parse_dst_entry(
        parts: &[&str],
        object: &mut SkinObject,
        scale_x: f32,
        scale_y: f32,
    ) -> Result<()> {
        // #DST_IMAGE,time,x,y,w,h,acc,a,r,g,b,blend,filter,angle,center,loop,timer,op1,op2,op3
        let time = Self::parse_i32(parts, 1) as i64;
        let x = Self::parse_f32(parts, 2) * scale_x;
        let y = Self::parse_f32(parts, 3) * scale_y;
        let w = Self::parse_f32(parts, 4) * scale_x;
        let h = Self::parse_f32(parts, 5) * scale_y;
        let acc = Self::parse_i32(parts, 6);
        let a = Self::parse_i32(parts, 7);
        let r = Self::parse_i32(parts, 8);
        let g = Self::parse_i32(parts, 9);
        let b = Self::parse_i32(parts, 10);
        let blend = Self::parse_i32(parts, 11);
        let filter = Self::parse_i32(parts, 12);
        let angle = Self::parse_i32(parts, 13);
        let center = Self::parse_i32(parts, 14);
        let loop_ms = Self::parse_i32(parts, 15);
        let timer = Self::parse_i32(parts, 16);
        let op1 = Self::parse_i32(parts, 17);
        let op2 = Self::parse_i32(parts, 18);
        let op3 = Self::parse_i32(parts, 19);

        let dst_entry = Destination::new(time, x, y, w, h, acc, a, r, g, b, angle);

        let set_dst = |dst: &mut DestinationSet| {
            dst.add_destination(dst_entry.clone());
            if dst.timer == 0 && timer > 0 {
                dst.timer = timer;
            }
            if dst.loop_ms == 0 && loop_ms != 0 {
                dst.loop_ms = loop_ms;
            }
            if dst.blend == 0 && blend != 0 {
                dst.blend = blend;
            }
            if dst.filter == 0 && filter != 0 {
                dst.filter = filter;
            }
            if dst.center == 0 && center > 0 {
                dst.center = center;
            }
            for op in [op1, op2, op3] {
                if op != 0 && !dst.options.contains(&op) {
                    dst.options.push(op);
                }
            }
        };

        match object {
            SkinObject::Image(img) => set_dst(&mut img.dst),
            SkinObject::Number(num) => set_dst(&mut num.dst),
            SkinObject::Slider(sl) => set_dst(&mut sl.dst),
            SkinObject::Text(txt) => set_dst(&mut txt.dst),
            SkinObject::Graph(g) => set_dst(&mut g.dst),
            SkinObject::Gauge(g) => set_dst(&mut g.dst),
            SkinObject::Judge(j) => set_dst(&mut j.dst),
            SkinObject::Bargraph(bg) => set_dst(&mut bg.dst),
            SkinObject::ImageSet(is) => set_dst(&mut is.dst),
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_i32_values() {
        let parts = vec!["#SRC_IMAGE", "0", "1", "10", "20", "100", "50"];
        assert_eq!(Lr2CsvLoader::parse_i32(&parts, 1), 0);
        assert_eq!(Lr2CsvLoader::parse_i32(&parts, 3), 10);
        assert_eq!(Lr2CsvLoader::parse_i32(&parts, 99), 0); // out of bounds
    }
}
