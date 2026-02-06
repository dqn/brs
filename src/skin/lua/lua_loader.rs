use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::{Result, anyhow};
use mlua::prelude::*;

use crate::skin::destination::{Destination, DestinationSet};
use crate::skin::object::graph::GraphObject;
use crate::skin::object::image::ImageObject;
use crate::skin::object::number::NumberObject;
use crate::skin::object::slider::SliderObject;
use crate::skin::skin_data::{SkinData, SkinObject, SkinSource};
use crate::skin::skin_header::{CustomFile, CustomOffset, CustomOption, SkinHeader, SkinType};

/// Convert mlua::Error to anyhow::Error.
fn lua_err(e: mlua::Error) -> anyhow::Error {
    anyhow!("Lua error: {}", e)
}

/// Loads skin data from a Lua skin file (beatoraja Lua skin format).
pub struct LuaSkinLoader {
    skin_dir: PathBuf,
}

impl LuaSkinLoader {
    pub fn new(skin_dir: PathBuf) -> Self {
        Self { skin_dir }
    }

    /// Load header information from a Lua skin file.
    pub fn load_header(&self, path: &Path) -> Result<SkinHeader> {
        let lua = Lua::new();
        self.setup_lua_env(&lua, path)?;

        let script = std::fs::read_to_string(path)
            .map_err(|e| anyhow!("Failed to read skin file {}: {}", path.display(), e))?;

        let value: LuaTable = lua
            .load(&script)
            .set_name(path.to_string_lossy())
            .eval()
            .map_err(|e| anyhow!("Failed to execute Lua skin {}: {}", path.display(), e))?;

        self.parse_header(&value, path)
    }

    /// Load full skin data from a Lua skin file.
    pub fn load(
        &self,
        path: &Path,
        dst_width: u32,
        dst_height: u32,
        selected_options: &HashMap<String, i32>,
        selected_files: &HashMap<String, String>,
    ) -> Result<SkinData> {
        let lua = Lua::new();
        self.setup_lua_env(&lua, path)?;

        // Export skin_config with selected options
        self.export_skin_config(&lua, selected_options, selected_files)?;

        // Export main_state stub
        self.export_main_state_stub(&lua)?;

        let script = std::fs::read_to_string(path)
            .map_err(|e| anyhow!("Failed to read skin file {}: {}", path.display(), e))?;

        let value: LuaTable = lua
            .load(&script)
            .set_name(path.to_string_lossy())
            .eval()
            .map_err(|e| anyhow!("Failed to execute Lua skin {}: {}", path.display(), e))?;

        let header = self.parse_header(&value, path)?;
        let mut skin_data = SkinData::new(header, dst_width, dst_height);

        // Parse sources
        if let Ok(source_table) = value.get::<LuaTable>("source") {
            self.parse_sources(&source_table, &mut skin_data)?;
        }

        // Parse destinations (skin objects)
        if let Ok(dst_table) = value.get::<LuaTable>("destination") {
            self.parse_destinations(&dst_table, &mut skin_data)?;
        }

        Ok(skin_data)
    }

    fn setup_lua_env(&self, lua: &Lua, path: &Path) -> Result<()> {
        let skin_dir = path.parent().unwrap_or(Path::new("."));

        // Add skin directory to package.path
        lua.load(format!(
            "package.path = '{}' .. '/?.lua;' .. package.path",
            skin_dir.display()
        ))
        .exec()
        .map_err(lua_err)?;

        Ok(())
    }

    fn export_skin_config(
        &self,
        lua: &Lua,
        options: &HashMap<String, i32>,
        files: &HashMap<String, String>,
    ) -> Result<()> {
        let config = lua.create_table().map_err(lua_err)?;
        let option_table = lua.create_table().map_err(lua_err)?;
        let file_table = lua.create_table().map_err(lua_err)?;

        for (name, &value) in options {
            option_table.set(name.as_str(), value).map_err(lua_err)?;
        }

        for (name, value) in files {
            file_table
                .set(name.as_str(), value.as_str())
                .map_err(lua_err)?;
        }

        config.set("option", option_table).map_err(lua_err)?;
        config.set("file", file_table).map_err(lua_err)?;

        lua.globals().set("skin_config", config).map_err(lua_err)?;
        Ok(())
    }

    fn export_main_state_stub(&self, lua: &Lua) -> Result<()> {
        let main_state = lua.create_table().map_err(lua_err)?;

        // Stub functions that return 0/false for header loading
        let number_fn = lua.create_function(|_, _id: i32| Ok(0)).map_err(lua_err)?;
        let option_fn = lua
            .create_function(|_, _id: i32| Ok(false))
            .map_err(lua_err)?;
        let timer_fn = lua
            .create_function(|_, _id: i32| Ok(i64::MIN))
            .map_err(lua_err)?;
        let text_fn = lua
            .create_function(|_, _id: i32| Ok(String::new()))
            .map_err(lua_err)?;
        let float_fn = lua
            .create_function(|_, _id: i32| Ok(0.0f64))
            .map_err(lua_err)?;
        let time_fn = lua.create_function(|_, ()| Ok(0i64)).map_err(lua_err)?;

        main_state.set("number", number_fn).map_err(lua_err)?;
        main_state.set("option", option_fn).map_err(lua_err)?;
        main_state.set("timer", timer_fn).map_err(lua_err)?;
        main_state.set("text", text_fn).map_err(lua_err)?;
        main_state.set("float_number", float_fn).map_err(lua_err)?;
        main_state.set("time", time_fn).map_err(lua_err)?;
        main_state
            .set("timer_off_value", i64::MIN)
            .map_err(lua_err)?;

        // Register as a loaded module
        let loaded: LuaTable = lua.load("return package.loaded").eval().map_err(lua_err)?;
        loaded.set("main_state", main_state).map_err(lua_err)?;

        Ok(())
    }

    fn parse_header(&self, table: &LuaTable, path: &Path) -> Result<SkinHeader> {
        let skin_type_id: i32 = table.get("type").unwrap_or(0);
        let skin_type = SkinType::from_id(skin_type_id)
            .ok_or_else(|| anyhow!("Unknown skin type: {}", skin_type_id))?;

        let mut header = SkinHeader {
            skin_type,
            name: table.get::<String>("name").unwrap_or_default(),
            author: table.get::<String>("author").unwrap_or_default(),
            path: path.to_path_buf(),
            src_width: table.get::<u32>("w").unwrap_or(1920),
            src_height: table.get::<u32>("h").unwrap_or(1080),
            scene: table.get::<i32>("scene").unwrap_or(3_600_000),
            input: table.get::<i32>("input").unwrap_or(0),
            fadeout: table.get::<i32>("fadeout").unwrap_or(0),
            load_end: table.get::<i32>("loadend").unwrap_or(0),
            play_start: table.get::<i32>("playstart").unwrap_or(0),
            close: table.get::<i32>("close").unwrap_or(0),
            ..Default::default()
        };

        // Parse property (custom options)
        if let Ok(prop_table) = table.get::<LuaTable>("property") {
            header.options = self.parse_custom_options(&prop_table)?;
        }

        // Parse filepath (custom files)
        if let Ok(file_table) = table.get::<LuaTable>("filepath") {
            header.files = self.parse_custom_files(&file_table)?;
        }

        // Parse offset (custom offsets)
        if let Ok(offset_table) = table.get::<LuaTable>("offset") {
            header.offsets = self.parse_custom_offsets(&offset_table)?;
        }

        Ok(header)
    }

    fn parse_custom_options(&self, table: &LuaTable) -> Result<Vec<CustomOption>> {
        let mut options = Vec::new();
        for pair in table.sequence_values::<LuaTable>() {
            let entry = pair.map_err(lua_err)?;
            let name: String = entry.get("name").unwrap_or_default();
            let mut option_ids = Vec::new();
            let mut option_names = Vec::new();

            if let Ok(items) = entry.get::<LuaTable>("item") {
                for item_pair in items.sequence_values::<LuaTable>() {
                    let item = item_pair.map_err(lua_err)?;
                    let item_name: String = item.get("name").unwrap_or_default();
                    let op: i32 = item.get("op").unwrap_or(0);
                    option_ids.push(op);
                    option_names.push(item_name);
                }
            }

            let def: Option<String> = entry.get("def").ok();
            options.push(CustomOption {
                name,
                option_ids,
                option_names,
                default_name: def,
                selected_index: -1,
            });
        }
        Ok(options)
    }

    fn parse_custom_files(&self, table: &LuaTable) -> Result<Vec<CustomFile>> {
        let mut files = Vec::new();
        for pair in table.sequence_values::<LuaTable>() {
            let entry = pair.map_err(lua_err)?;
            files.push(CustomFile {
                name: entry.get("name").unwrap_or_default(),
                path: entry.get("path").unwrap_or_default(),
                default_name: entry.get("def").ok(),
                selected_filename: None,
            });
        }
        Ok(files)
    }

    fn parse_custom_offsets(&self, table: &LuaTable) -> Result<Vec<CustomOffset>> {
        let mut offsets = Vec::new();
        for pair in table.sequence_values::<LuaTable>() {
            let entry = pair.map_err(lua_err)?;
            offsets.push(CustomOffset {
                name: entry.get("name").unwrap_or_default(),
                id: entry.get("id").unwrap_or(0),
                x: entry.get("x").unwrap_or(false),
                y: entry.get("y").unwrap_or(false),
                w: entry.get("w").unwrap_or(false),
                h: entry.get("h").unwrap_or(false),
                r: entry.get("r").unwrap_or(false),
                a: entry.get("a").unwrap_or(false),
            });
        }
        Ok(offsets)
    }

    fn parse_sources(&self, table: &LuaTable, skin: &mut SkinData) -> Result<()> {
        for pair in table.sequence_values::<LuaTable>() {
            let entry = pair.map_err(lua_err)?;
            let id: i32 = entry.get("id").unwrap_or(-1);
            let path_str: String = entry.get("path").unwrap_or_default();

            if id < 0 || path_str.is_empty() {
                continue;
            }

            let full_path = self.skin_dir.join(&path_str);
            skin.add_source(SkinSource {
                id,
                path: full_path,
                texture: None,
            });
        }
        Ok(())
    }

    fn parse_destinations(&self, table: &LuaTable, skin: &mut SkinData) -> Result<()> {
        for pair in table.sequence_values::<LuaTable>() {
            let entry = pair.map_err(lua_err)?;
            if let Some(obj) = self.parse_skin_object(&entry, skin)? {
                skin.add_object(obj);
            }
        }
        Ok(())
    }

    fn parse_skin_object(&self, table: &LuaTable, skin: &SkinData) -> Result<Option<SkinObject>> {
        let id: String = table.get("id").unwrap_or_default();

        // Parse destination set
        let dst = self.parse_destination_set(table, skin)?;

        // Check if this is a number, slider, text, etc. based on fields
        let ref_id: Option<i32> = table.get("ref").ok();
        let digit: Option<i32> = table.get("digit").ok();
        let direction: Option<i32> = table.get("direction").ok();
        let range: Option<f32> = table.get("range").ok();

        let src_id: i32 = table.get("src").unwrap_or(-1);

        // Determine type from fields
        if digit.is_some() {
            let mut num = NumberObject {
                id,
                ref_id: ref_id.unwrap_or(0),
                src: src_id,
                div_x: table.get("divx").unwrap_or(10),
                digit: digit.unwrap_or(0),
                padding: table.get("padding").unwrap_or(0),
                align: table.get("align").unwrap_or(0),
                dst,
                ..Default::default()
            };
            Self::apply_source_rect_from_table(
                table,
                &mut num.src_x,
                &mut num.src_y,
                &mut num.src_w,
                &mut num.src_h,
            );
            return Ok(Some(SkinObject::Number(num)));
        }

        if range.is_some() {
            let mut sl = SliderObject {
                id,
                ref_id: ref_id.unwrap_or(0),
                src: src_id,
                range: range.unwrap_or(0.0),
                direction: direction.unwrap_or(0),
                dst,
                ..Default::default()
            };
            Self::apply_source_rect_from_table(
                table,
                &mut sl.src_x,
                &mut sl.src_y,
                &mut sl.src_w,
                &mut sl.src_h,
            );
            return Ok(Some(SkinObject::Slider(sl)));
        }

        if let Ok(graph_type) = table.get::<i32>("graph") {
            let mut g = GraphObject {
                id,
                graph_type,
                src: src_id,
                direction: direction.unwrap_or(0),
                dst,
                ..Default::default()
            };
            Self::apply_source_rect_from_table(
                table,
                &mut g.src_x,
                &mut g.src_y,
                &mut g.src_w,
                &mut g.src_h,
            );
            return Ok(Some(SkinObject::Graph(g)));
        }

        // Default to image object
        let mut img = ImageObject {
            id,
            src: src_id,
            div_x: table.get("divx").unwrap_or(1),
            div_y: table.get("divy").unwrap_or(1),
            cycle: table.get("cycle").unwrap_or(0),
            timer: table.get("timer").unwrap_or(0),
            dst,
            ..Default::default()
        };
        Self::apply_source_rect_from_table(
            table,
            &mut img.src_x,
            &mut img.src_y,
            &mut img.src_w,
            &mut img.src_h,
        );
        Ok(Some(SkinObject::Image(img)))
    }

    fn apply_source_rect_from_table(
        table: &LuaTable,
        x: &mut i32,
        y: &mut i32,
        w: &mut i32,
        h: &mut i32,
    ) {
        *x = table.get("x").unwrap_or(0);
        *y = table.get("y").unwrap_or(0);
        *w = table.get("w").unwrap_or(0);
        *h = table.get("h").unwrap_or(0);
    }

    fn parse_destination_set(&self, table: &LuaTable, skin: &SkinData) -> Result<DestinationSet> {
        let mut dst_set = DestinationSet {
            timer: table.get("timer").unwrap_or(0),
            loop_ms: table.get("loop").unwrap_or(0),
            blend: table.get("blend").unwrap_or(0),
            filter: table.get("filter").unwrap_or(0),
            center: table.get("center").unwrap_or(0),
            stretch: table.get("stretch").unwrap_or(0),
            ..Default::default()
        };

        // Parse options
        if let Ok(op_table) = table.get::<LuaTable>("op") {
            for op in op_table.sequence_values::<i32>().flatten() {
                if op != 0 {
                    dst_set.options.push(op);
                }
            }
        }

        // Parse offsets
        if let Ok(offset) = table.get::<i32>("offset")
            && offset > 0
        {
            dst_set.offsets.push(offset);
        }
        if let Ok(offset_table) = table.get::<LuaTable>("offsets") {
            for off in offset_table.sequence_values::<i32>().flatten() {
                if off > 0 {
                    dst_set.offsets.push(off);
                }
            }
        }

        // Parse destination keyframes from "dst" array
        if let Ok(dst_array) = table.get::<LuaTable>("dst") {
            for pair in dst_array.sequence_values::<LuaTable>() {
                let entry = pair.map_err(lua_err)?;
                let time: i64 = entry.get("time").unwrap_or(0);
                let x: f32 = entry.get("x").unwrap_or(0.0);
                let y: f32 = entry.get("y").unwrap_or(0.0);
                let w: f32 = entry.get("w").unwrap_or(0.0);
                let h: f32 = entry.get("h").unwrap_or(0.0);
                let acc: i32 = entry.get("acc").unwrap_or(0);
                let a: i32 = entry.get("a").unwrap_or(255);
                let r: i32 = entry.get("r").unwrap_or(255);
                let g: i32 = entry.get("g").unwrap_or(255);
                let b: i32 = entry.get("b").unwrap_or(255);
                let angle: i32 = entry.get("angle").unwrap_or(0);

                dst_set.add_destination(Destination::new(
                    time,
                    x * skin.scale_x,
                    y * skin.scale_y,
                    w * skin.scale_x,
                    h * skin.scale_y,
                    acc,
                    a,
                    r,
                    g,
                    b,
                    angle,
                ));
            }
        }

        Ok(dst_set)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_skin_type_from_id() {
        assert_eq!(SkinType::from_id(0), Some(SkinType::Play7Keys));
        assert_eq!(SkinType::from_id(5), Some(SkinType::MusicSelect));
        assert_eq!(SkinType::from_id(99), None);
    }
}
