use std::collections::HashMap;
use std::path::Path;

use anyhow::{Context, Result, anyhow};
use mlua::{Lua, Table, Value};

use crate::skin::skin_property::*;
use crate::skin::{
    Destination, FontDef, ImageDef, ImageSetDef, NumberDef, Skin, SkinHeader, SkinObjectData,
    SkinObjectType, SkinSource, SkinType, TextDef,
};

/// Helper trait to convert mlua::Result to anyhow::Result.
trait LuaResultExt<T> {
    fn to_anyhow(self) -> Result<T>;
}

impl<T> LuaResultExt<T> for mlua::Result<T> {
    fn to_anyhow(self) -> Result<T> {
        self.map_err(|e| anyhow!("Lua error: {}", e))
    }
}

/// Temporary holder for judge sub-objects parsed from skin.judge.
struct JudgeSubObjects {
    /// Image destination entries (PG, GR, GD, BD, PR, MS).
    images: Vec<Table>,
    /// Number destination entries (PG, GR, GD, BD, PR, MS).
    numbers: Vec<Table>,
}

/// Lua skin loader for .luaskin files.
pub struct LuaSkinLoader {
    lua: Lua,
}

impl LuaSkinLoader {
    /// Create a new Lua skin loader.
    pub fn new() -> Result<Self> {
        let lua = Lua::new();
        Ok(Self { lua })
    }

    /// Load skin header only (for skin selection).
    pub fn load_header(&self, path: &Path) -> Result<SkinHeader> {
        let skin_dir = path.parent().context("Invalid skin path")?;

        // Set up package.path for require
        self.setup_lua_path(skin_dir)?;

        // Execute the skin file without skin_config
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read skin file: {}", path.display()))?;

        let result: Table = self
            .lua
            .load(&content)
            .set_name(path.to_string_lossy())
            .eval()
            .to_anyhow()?;

        self.parse_header(&result)
    }

    /// Load complete skin with configuration.
    pub fn load(&self, path: &Path, options: &HashMap<String, i32>) -> Result<Skin> {
        let skin_dir = path.parent().context("Invalid skin path")?;

        // Set up package.path for require
        self.setup_lua_path(skin_dir)?;

        let mut resolved_options = options.clone();
        if let Ok(defaults) = Self::default_options_from_file(path) {
            for (key, value) in defaults {
                resolved_options.entry(key).or_insert(value);
            }
        }

        // Set up skin_config global
        self.setup_skin_config(&resolved_options)?;

        // Execute the skin file
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read skin file: {}", path.display()))?;

        let result: Table = self
            .lua
            .load(&content)
            .set_name(path.to_string_lossy())
            .eval()
            .to_anyhow()?;

        let header_table = self.header_table(&result);
        let header = self.parse_header(&header_table)?;
        let file_map = self.collect_file_map(&header_table);

        // Check if the result has a 'main' function (beatoraja-style skin)
        // If so, call it to get the actual skin data
        let skin_table = if let Ok(main_fn) = result.get::<mlua::Function>("main") {
            main_fn.call::<Table>(()).to_anyhow()?
        } else {
            result
        };

        let mut skin = self.parse_skin(&skin_table, skin_dir, header)?;
        skin.file_map = file_map;
        Ok(skin)
    }

    pub(crate) fn default_options_from_file(path: &Path) -> Result<HashMap<String, i32>> {
        let loader = LuaSkinLoader::new()?;
        loader.load_default_options(path)
    }

    fn load_default_options(&self, path: &Path) -> Result<HashMap<String, i32>> {
        let skin_dir = path.parent().context("Invalid skin path")?;

        self.setup_lua_path(skin_dir)?;
        self.setup_main_state()?;
        self.lua
            .globals()
            .set("skin_config", Value::Nil)
            .to_anyhow()?;

        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read skin file: {}", path.display()))?;

        let result: Table = self
            .lua
            .load(&content)
            .set_name(path.to_string_lossy())
            .eval()
            .to_anyhow()?;

        self.collect_default_options(&result)
    }

    fn collect_default_options(&self, table: &Table) -> Result<HashMap<String, i32>> {
        let mut options = HashMap::new();

        // Try to get property directly, or from header (beatoraja-style skin)
        let property_table = if let Ok(value) = table.get::<Table>("property") {
            value
        } else if let Ok(header) = table.get::<Table>("header") {
            match header.get::<Table>("property") {
                Ok(value) => value,
                Err(_) => return Ok(options),
            }
        } else {
            return Ok(options);
        };

        for prop in property_table.sequence_values::<Table>() {
            let prop = prop.to_anyhow()?;
            let name = match prop.get::<String>("name") {
                Ok(value) => value,
                Err(_) => continue,
            };
            let items = match prop.get::<Table>("item") {
                Ok(value) => value,
                Err(_) => continue,
            };

            let mut selected = None;
            for item in items.sequence_values::<Table>() {
                let item = item.to_anyhow()?;
                if let Ok(op) = item.get::<i32>("op") {
                    selected = Some(op);
                    break;
                }
            }

            if let Some(op) = selected {
                options.insert(name, op);
            }
        }

        Ok(options)
    }

    fn get_i32(table: &Table, key: &str) -> Option<i32> {
        if let Ok(value) = table.get::<i32>(key) {
            return Some(value);
        }
        if let Ok(value) = table.get::<f64>(key) {
            return Some(value as i32);
        }
        None
    }

    fn get_f32(table: &Table, key: &str) -> Option<f32> {
        if let Ok(value) = table.get::<f32>(key) {
            return Some(value);
        }
        if let Ok(value) = table.get::<f64>(key) {
            return Some(value as f32);
        }
        None
    }

    fn setup_lua_path(&self, skin_dir: &Path) -> Result<()> {
        let package: Table = self.lua.globals().get("package").to_anyhow()?;
        let skin_dir_str = skin_dir.to_string_lossy();

        // Add skin directory to package.path
        let path: String = package.get("path").to_anyhow()?;
        let new_path = format!("{}/?.lua;{}", skin_dir_str, path);
        package.set("path", new_path).to_anyhow()?;

        Ok(())
    }

    fn setup_skin_config(&self, options: &HashMap<String, i32>) -> Result<()> {
        let skin_config = self.lua.create_table().to_anyhow()?;
        let option_table = self.lua.create_table().to_anyhow()?;

        for (key, value) in options {
            option_table.set(key.as_str(), *value).to_anyhow()?;
        }

        skin_config.set("option", option_table).to_anyhow()?;
        self.lua
            .globals()
            .set("skin_config", skin_config)
            .to_anyhow()?;

        // Set up empty main_state module
        self.setup_main_state()?;

        Ok(())
    }

    fn setup_main_state(&self) -> Result<()> {
        // Create a stub main_state module that returns default values
        let main_state = self.lua.create_table().to_anyhow()?;

        // number(id) -> returns 0
        let number_fn = self.lua.create_function(|_, _id: i32| Ok(0)).to_anyhow()?;
        main_state.set("number", number_fn).to_anyhow()?;

        // option(id) -> returns false
        let option_fn = self
            .lua
            .create_function(|_, _id: i32| Ok(false))
            .to_anyhow()?;
        main_state.set("option", option_fn).to_anyhow()?;

        // timer(id) -> returns timer_off_value
        let timer_fn = self
            .lua
            .create_function(|_, _id: i32| Ok(i64::MIN))
            .to_anyhow()?;
        main_state.set("timer", timer_fn).to_anyhow()?;

        // text(id) -> returns empty string
        let text_fn = self.lua.create_function(|_, _id: i32| Ok("")).to_anyhow()?;
        main_state.set("text", text_fn).to_anyhow()?;

        // gauge() -> returns 0
        let gauge_fn = self.lua.create_function(|_, ()| Ok(0.0)).to_anyhow()?;
        main_state.set("gauge", gauge_fn).to_anyhow()?;

        // gauge_type() -> returns 0
        let gauge_type_fn = self.lua.create_function(|_, ()| Ok(0)).to_anyhow()?;
        main_state.set("gauge_type", gauge_type_fn).to_anyhow()?;

        // Register as a preloaded module
        let package: Table = self.lua.globals().get("package").to_anyhow()?;
        let preload: Table = package.get("preload").to_anyhow()?;
        let main_state_loader = self
            .lua
            .create_function(move |lua, ()| lua.globals().get::<Table>("_main_state_stub"))
            .to_anyhow()?;
        preload.set("main_state", main_state_loader).to_anyhow()?;

        self.lua
            .globals()
            .set("_main_state_stub", main_state)
            .to_anyhow()?;

        Ok(())
    }

    fn parse_header(&self, table: &Table) -> Result<SkinHeader> {
        let mut header = SkinHeader::default();

        let header_table = self.header_table(table);

        if let Ok(name) = header_table.get::<String>("name") {
            header.name = name;
        }

        if let Ok(author) = header_table.get::<String>("author") {
            header.author = author;
        }

        if let Ok(skin_type) = header_table.get::<i32>("type") {
            if let Some(st) = SkinType::from_i32(skin_type) {
                header.skin_type = st;
            }
        }

        if let Ok(w) = header_table.get::<u32>("w") {
            header.width = w;
        }

        if let Ok(h) = header_table.get::<u32>("h") {
            header.height = h;
        }

        if let Ok(loadend) = header_table.get::<i32>("loadend") {
            header.loadend = loadend;
        }

        if let Ok(playstart) = header_table.get::<i32>("playstart") {
            header.playstart = playstart;
        }

        if let Ok(scene) = header_table.get::<i32>("scene") {
            header.scene = scene;
        }

        if let Ok(input) = header_table.get::<i32>("input") {
            header.input = input;
        }

        if let Ok(close) = header_table.get::<i32>("close") {
            header.close = close;
        }

        if let Ok(fadeout) = header_table.get::<i32>("fadeout") {
            header.fadeout = fadeout;
        }

        Ok(header)
    }

    fn parse_skin(&self, table: &Table, skin_dir: &Path, header: SkinHeader) -> Result<Skin> {
        let mut skin = Skin::new(header);
        let mut source_name_map = HashMap::new();

        // Parse source array
        if let Ok(source_table) = table.get::<Table>("source") {
            source_name_map = self.parse_sources(&source_table, skin_dir, &mut skin)?;
        }

        // Parse image definitions
        if let Ok(image_table) = table.get::<Table>("image") {
            self.parse_images(&image_table, &source_name_map, &mut skin)?;
        }

        // Parse slider definitions
        if let Ok(slider_table) = table.get::<Table>("slider") {
            self.parse_sliders(&slider_table, &source_name_map, &mut skin)?;
        }

        // Parse graph definitions (treated as static images for now)
        if let Ok(graph_table) = table.get::<Table>("graph") {
            self.parse_graphs(&graph_table, &source_name_map, &mut skin)?;
        }

        // Parse imageset definitions
        if let Ok(imageset_table) = table.get::<Table>("imageset") {
            self.parse_imagesets(&imageset_table, &mut skin)?;
        }

        // Parse value (number) definitions
        if let Ok(value_table) = table.get::<Table>("value") {
            self.parse_values(&value_table, &source_name_map, &mut skin)?;
        }

        // Parse font definitions
        if let Ok(font_table) = table.get::<Table>("font") {
            self.parse_fonts(&font_table, &mut skin)?;
        }

        // Parse text definitions
        if let Ok(text_table) = table.get::<Table>("text") {
            self.parse_texts(&text_table, &mut skin)?;
        }

        // Parse BGA definitions
        if let Ok(bga_table) = table.get::<Table>("bga") {
            self.parse_bga(&bga_table, &mut skin)?;
        }

        // Parse judge definitions (composite judge objects)
        let judge_map = if let Ok(judge_table) = table.get::<Table>("judge") {
            self.parse_judge(&judge_table)?
        } else {
            HashMap::new()
        };

        // Parse destination (objects)
        if let Ok(dst_table) = table.get::<Table>("destination") {
            self.parse_destinations(&dst_table, &mut skin, &judge_map)?;
        }

        Ok(skin)
    }

    fn header_table(&self, table: &Table) -> Table {
        table
            .get::<Table>("header")
            .unwrap_or_else(|_| table.clone())
    }

    fn collect_file_map(&self, table: &Table) -> HashMap<String, String> {
        let mut file_map = HashMap::new();
        let header_table = self.header_table(table);
        let Ok(filepath_table) = header_table.get::<Table>("filepath") else {
            return file_map;
        };

        for entry in filepath_table.sequence_values::<Table>() {
            let Ok(entry) = entry else {
                continue;
            };
            let Ok(path) = entry.get::<String>("path") else {
                continue;
            };
            let Ok(def_value) = entry.get::<String>("def") else {
                continue;
            };
            if !def_value.is_empty() {
                file_map.insert(path, def_value);
            }
        }

        file_map
    }

    fn parse_sources(
        &self,
        table: &Table,
        _skin_dir: &Path,
        skin: &mut Skin,
    ) -> Result<HashMap<String, u32>> {
        let mut numeric_entries = Vec::new();
        let mut named_entries: Vec<(String, String)> = Vec::new();
        let mut max_numeric_id = 0u32;

        for pair in table.pairs::<Value, Table>() {
            let (_, src_table) = pair.to_anyhow()?;
            let path: String = src_table.get("path").unwrap_or_default();
            if path.is_empty() {
                continue;
            }

            if let Ok(id) = src_table.get::<u32>("id") {
                max_numeric_id = max_numeric_id.max(id);
                numeric_entries.push((id, path));
            } else if let Ok(name) = src_table.get::<String>("id") {
                named_entries.push((name, path));
            }
        }

        for (id, path) in numeric_entries {
            skin.sources.insert(id, SkinSource { id, path });
        }

        named_entries.sort_by(|left, right| left.0.cmp(&right.0));

        let mut source_name_map = HashMap::new();
        let mut next_id = max_numeric_id.saturating_add(1);
        for (name, path) in named_entries {
            let id = next_id;
            next_id = next_id.saturating_add(1);
            source_name_map.insert(name, id);
            skin.sources.insert(id, SkinSource { id, path });
        }

        Ok(source_name_map)
    }

    fn parse_images(
        &self,
        table: &Table,
        source_name_map: &HashMap<String, u32>,
        skin: &mut Skin,
    ) -> Result<()> {
        for pair in table.pairs::<Value, Table>() {
            let (_, img_table) = pair.to_anyhow()?;

            let mut image_def = ImageDef::default();

            if let Ok(id) = img_table.get::<String>("id") {
                image_def.id = id;
            } else if let Ok(id_num) = img_table.get::<i32>("id") {
                image_def.id = id_num.to_string();
            }

            if let Ok(src) = img_table.get::<u32>("src") {
                image_def.src = src;
            } else if let Ok(src_name) = img_table.get::<String>("src") {
                if let Some(id) = source_name_map.get(&src_name) {
                    image_def.src = *id;
                } else if let Ok(id) = src_name.parse::<u32>() {
                    image_def.src = id;
                }
            }

            if let Ok(x) = img_table.get::<i32>("x") {
                image_def.x = x;
            }

            if let Ok(y) = img_table.get::<i32>("y") {
                image_def.y = y;
            }

            if let Ok(w) = img_table.get::<i32>("w") {
                image_def.w = w;
            }

            if let Ok(h) = img_table.get::<i32>("h") {
                image_def.h = h;
            }

            if let Ok(divx) = img_table.get::<i32>("divx") {
                image_def.divx = divx;
            }

            if let Ok(divy) = img_table.get::<i32>("divy") {
                image_def.divy = divy;
            }

            if let Ok(timer) = img_table.get::<i32>("timer") {
                image_def.timer = timer;
            }

            if let Ok(cycle) = img_table.get::<i32>("cycle") {
                image_def.cycle = cycle;
            }

            if !image_def.id.is_empty() {
                skin.images.insert(image_def.id.clone(), image_def);
            }
        }

        Ok(())
    }

    fn parse_sliders(
        &self,
        table: &Table,
        source_name_map: &HashMap<String, u32>,
        skin: &mut Skin,
    ) -> Result<()> {
        use crate::skin::SliderDef;

        for pair in table.pairs::<Value, Table>() {
            let (_, slider_table) = pair.to_anyhow()?;

            let mut slider_def = SliderDef::default();

            if let Ok(id) = slider_table.get::<String>("id") {
                slider_def.id = id;
            } else if let Ok(id_num) = slider_table.get::<i32>("id") {
                slider_def.id = id_num.to_string();
            }

            if let Ok(src) = slider_table.get::<u32>("src") {
                slider_def.src = src;
            } else if let Ok(src_name) = slider_table.get::<String>("src") {
                if let Some(id) = source_name_map.get(&src_name) {
                    slider_def.src = *id;
                } else if let Ok(id) = src_name.parse::<u32>() {
                    slider_def.src = id;
                }
            }

            if let Ok(x) = slider_table.get::<i32>("x") {
                slider_def.x = x;
            }
            if let Ok(y) = slider_table.get::<i32>("y") {
                slider_def.y = y;
            }
            if let Ok(w) = slider_table.get::<i32>("w") {
                slider_def.w = w;
            }
            if let Ok(h) = slider_table.get::<i32>("h") {
                slider_def.h = h;
            }
            if let Ok(divx) = slider_table.get::<i32>("divx") {
                slider_def.divx = divx;
            }
            if let Ok(divy) = slider_table.get::<i32>("divy") {
                slider_def.divy = divy;
            }
            if let Ok(timer) = slider_table.get::<i32>("timer") {
                slider_def.timer = timer;
            }
            if let Ok(cycle) = slider_table.get::<i32>("cycle") {
                slider_def.cycle = cycle;
            }
            if let Ok(angle) = slider_table.get::<i32>("angle") {
                slider_def.angle = angle;
            }
            if let Ok(range) = slider_table.get::<i32>("range") {
                slider_def.range = range;
            }
            if let Ok(slider_type) = slider_table.get::<i32>("type") {
                slider_def.slider_type = slider_type;
            }
            if let Ok(min) = slider_table.get::<i32>("min") {
                slider_def.min = Some(min);
            }
            if let Ok(max) = slider_table.get::<i32>("max") {
                slider_def.max = Some(max);
            }

            if !slider_def.id.is_empty() {
                skin.sliders.insert(slider_def.id.clone(), slider_def);
            }
        }

        Ok(())
    }

    fn parse_graphs(
        &self,
        table: &Table,
        source_name_map: &HashMap<String, u32>,
        skin: &mut Skin,
    ) -> Result<()> {
        for pair in table.pairs::<Value, Table>() {
            let (_, graph_table) = pair.to_anyhow()?;

            let mut image_def = ImageDef::default();

            if let Ok(id) = graph_table.get::<String>("id") {
                image_def.id = id;
            } else if let Ok(id_num) = graph_table.get::<i32>("id") {
                image_def.id = id_num.to_string();
            }

            if let Ok(src) = graph_table.get::<u32>("src") {
                image_def.src = src;
            } else if let Ok(src_name) = graph_table.get::<String>("src") {
                if let Some(id) = source_name_map.get(&src_name) {
                    image_def.src = *id;
                } else if let Ok(id) = src_name.parse::<u32>() {
                    image_def.src = id;
                }
            }

            if let Ok(x) = graph_table.get::<i32>("x") {
                image_def.x = x;
            }
            if let Ok(y) = graph_table.get::<i32>("y") {
                image_def.y = y;
            }
            if let Ok(w) = graph_table.get::<i32>("w") {
                image_def.w = w;
            }
            if let Ok(h) = graph_table.get::<i32>("h") {
                image_def.h = h;
            }
            if let Ok(timer) = graph_table.get::<i32>("timer") {
                image_def.timer = timer;
            }

            // Graph definitions are treated as static images to avoid unintended animation.
            image_def.divx = 1;
            image_def.divy = 1;
            image_def.cycle = 0;

            if !image_def.id.is_empty() {
                skin.images.insert(image_def.id.clone(), image_def);
            }
        }

        Ok(())
    }

    fn parse_imagesets(&self, table: &Table, skin: &mut Skin) -> Result<()> {
        for pair in table.pairs::<Value, Table>() {
            let (_, set_table) = pair.to_anyhow()?;

            let mut imageset_def = ImageSetDef::default();

            if let Ok(id) = set_table.get::<String>("id") {
                imageset_def.id = id;
            } else if let Ok(id_num) = set_table.get::<i32>("id") {
                imageset_def.id = id_num.to_string();
            }

            if let Ok(mode) = set_table.get::<i32>("mode") {
                imageset_def.mode = mode;
            }

            if let Ok(ref_id) = set_table.get::<i32>("ref") {
                imageset_def.ref_id = ref_id;
            }

            if let Ok(images) = set_table.get::<Table>("images") {
                for pair in images.pairs::<Value, Value>() {
                    let (_, value) = pair.to_anyhow()?;
                    if let Value::String(s) = value {
                        imageset_def
                            .images
                            .push(s.to_str().to_anyhow()?.to_string());
                    }
                }
            }

            if !imageset_def.id.is_empty() {
                skin.image_sets
                    .insert(imageset_def.id.clone(), imageset_def);
            }
        }

        Ok(())
    }

    fn parse_values(
        &self,
        table: &Table,
        source_name_map: &HashMap<String, u32>,
        skin: &mut Skin,
    ) -> Result<()> {
        for pair in table.pairs::<Value, Table>() {
            let (_, val_table) = pair.to_anyhow()?;

            let mut number_def = NumberDef::default();

            if let Ok(id) = val_table.get::<String>("id") {
                number_def.id = id;
            } else if let Ok(id_num) = val_table.get::<i32>("id") {
                number_def.id = id_num.to_string();
            }

            if let Ok(src) = val_table.get::<u32>("src") {
                number_def.src = src;
            } else if let Ok(src_name) = val_table.get::<String>("src") {
                if let Some(id) = source_name_map.get(&src_name) {
                    number_def.src = *id;
                } else if let Ok(id) = src_name.parse::<u32>() {
                    number_def.src = id;
                }
            }

            if let Ok(x) = val_table.get::<i32>("x") {
                number_def.x = x;
            }

            if let Ok(y) = val_table.get::<i32>("y") {
                number_def.y = y;
            }

            if let Ok(w) = val_table.get::<i32>("w") {
                number_def.w = w;
            }

            if let Ok(h) = val_table.get::<i32>("h") {
                number_def.h = h;
            }

            if let Ok(divx) = val_table.get::<i32>("divx") {
                number_def.divx = divx;
            }

            if let Ok(divy) = val_table.get::<i32>("divy") {
                number_def.divy = divy;
            }

            if let Ok(digit) = val_table.get::<i32>("digit") {
                number_def.digit = digit;
            }

            if let Ok(ref_id) = val_table.get::<i32>("ref") {
                number_def.ref_id = ref_id;
            }

            if let Ok(align) = val_table.get::<i32>("align") {
                number_def.align = align;
            }

            if let Ok(padding) = val_table.get::<i32>("padding") {
                number_def.zeropadding = padding;
            }
            if let Ok(zeropadding) = val_table.get::<i32>("zeropadding") {
                number_def.zeropadding = zeropadding;
            }

            if let Ok(space) = val_table.get::<i32>("space") {
                number_def.space = space;
            }

            if let Ok(cycle) = val_table.get::<i32>("cycle") {
                number_def.cycle = cycle;
            }

            if !number_def.id.is_empty() {
                skin.numbers.insert(number_def.id.clone(), number_def);
            }
        }

        Ok(())
    }

    fn parse_fonts(&self, table: &Table, skin: &mut Skin) -> Result<()> {
        for pair in table.pairs::<Value, Table>() {
            let (_, font_table) = pair.to_anyhow()?;

            let mut font_def = FontDef::default();

            if let Ok(id) = font_table.get::<u32>("id") {
                font_def.id = id;
            }

            if let Ok(path) = font_table.get::<String>("path") {
                font_def.path = path;
            }

            if font_def.id > 0 && !font_def.path.is_empty() {
                skin.fonts.insert(font_def.id, font_def);
            }
        }

        Ok(())
    }

    fn parse_texts(&self, table: &Table, skin: &mut Skin) -> Result<()> {
        for pair in table.pairs::<Value, Table>() {
            let (_, text_table) = pair.to_anyhow()?;

            let mut text_def = TextDef::default();

            if let Ok(id) = text_table.get::<String>("id") {
                text_def.id = id;
            } else if let Ok(id_num) = text_table.get::<i32>("id") {
                text_def.id = id_num.to_string();
            }

            if let Ok(font) = text_table.get::<u32>("font") {
                text_def.font = font;
            }

            if let Ok(size) = text_table.get::<i32>("size") {
                text_def.size = size;
            }

            if let Ok(align) = text_table.get::<i32>("align") {
                text_def.align = align;
            }

            if let Ok(overflow) = text_table.get::<i32>("overflow") {
                text_def.overflow = overflow;
            }

            if let Ok(ref_id) = text_table.get::<i32>("ref") {
                text_def.ref_id = ref_id;
            }

            if !text_def.id.is_empty() {
                skin.texts.insert(text_def.id.clone(), text_def);
            }
        }

        Ok(())
    }

    fn parse_bga(&self, table: &Table, skin: &mut Skin) -> Result<()> {
        if let Ok(id) = table.get::<String>("id") {
            if !id.is_empty() {
                skin.bga_ids.insert(id);
                return Ok(());
            }
        }

        if let Ok(id_num) = table.get::<i32>("id") {
            skin.bga_ids.insert(id_num.to_string());
            return Ok(());
        }

        for entry in table.sequence_values::<Table>() {
            let entry = entry.to_anyhow()?;
            if let Ok(id) = entry.get::<String>("id") {
                if !id.is_empty() {
                    skin.bga_ids.insert(id);
                }
                continue;
            }
            if let Ok(id_num) = entry.get::<i32>("id") {
                skin.bga_ids.insert(id_num.to_string());
            }
        }

        Ok(())
    }

    fn parse_judge(&self, table: &Table) -> Result<HashMap<String, JudgeSubObjects>> {
        let mut judge_map = HashMap::new();

        for entry in table.sequence_values::<Table>() {
            let entry = entry.to_anyhow()?;

            let id = if let Ok(id) = entry.get::<String>("id") {
                id
            } else if let Ok(id_num) = entry.get::<i32>("id") {
                id_num.to_string()
            } else {
                continue;
            };

            let mut sub = JudgeSubObjects {
                images: Vec::new(),
                numbers: Vec::new(),
            };

            if let Ok(images_table) = entry.get::<Table>("images") {
                for img in images_table.sequence_values::<Table>() {
                    sub.images.push(img.to_anyhow()?);
                }
            }

            if let Ok(numbers_table) = entry.get::<Table>("numbers") {
                for num in numbers_table.sequence_values::<Table>() {
                    sub.numbers.push(num.to_anyhow()?);
                }
            }

            judge_map.insert(id, sub);
        }

        Ok(judge_map)
    }

    fn parse_destinations(
        &self,
        table: &Table,
        skin: &mut Skin,
        judge_map: &HashMap<String, JudgeSubObjects>,
    ) -> Result<()> {
        for dst_entry in table.sequence_values::<Table>() {
            let dst_table = dst_entry.to_anyhow()?;

            let mut obj_data = SkinObjectData::default();

            if let Ok(id) = dst_table.get::<String>("id") {
                obj_data.id = id.clone();
            } else if let Ok(id_num) = dst_table.get::<i32>("id") {
                obj_data.id = id_num.to_string();
            }

            if !obj_data.id.is_empty() {
                // Determine object type based on definition type
                if skin.bga_ids.contains(&obj_data.id) {
                    obj_data.object_type = SkinObjectType::Bga;
                } else if skin.numbers.contains_key(&obj_data.id) {
                    obj_data.object_type = SkinObjectType::Number;
                } else if skin.texts.contains_key(&obj_data.id) {
                    obj_data.object_type = SkinObjectType::Text;
                } else if skin.image_sets.contains_key(&obj_data.id) {
                    obj_data.object_type = SkinObjectType::ImageSet;
                } else if skin.sliders.contains_key(&obj_data.id) {
                    obj_data.object_type = SkinObjectType::Slider;
                } else {
                    obj_data.object_type = SkinObjectType::Image;
                }
            }

            // Parse op (option conditions)
            if let Ok(op_table) = dst_table.get::<Table>("op") {
                for pair in op_table.pairs::<Value, Value>() {
                    let (_, op_value) = pair.to_anyhow()?;
                    match op_value {
                        Value::Integer(op_id) => obj_data.op.push(op_id as i32),
                        Value::Number(op_id) => obj_data.op.push(op_id as i32),
                        _ => {}
                    }
                }
            }

            if let Some(timer) = Self::get_i32(&dst_table, "timer") {
                obj_data.timer = timer;
            }

            if let Some(loop_count) = Self::get_i32(&dst_table, "loop") {
                obj_data.loop_count = loop_count;
            }

            if let Some(offset) = Self::get_i32(&dst_table, "offset") {
                obj_data.offset = offset;
            }

            if let Ok(offsets_table) = dst_table.get::<Table>("offsets") {
                for value in offsets_table.sequence_values::<Value>() {
                    let value = value.to_anyhow()?;
                    let offset_id = match value {
                        Value::Integer(id) => Some(id as i32),
                        Value::Number(id) => Some(id as i32),
                        _ => None,
                    };
                    if let Some(offset_id) = offset_id {
                        obj_data.offsets.push(offset_id);
                    }
                }
            }

            if let Some(blend) = Self::get_i32(&dst_table, "blend") {
                obj_data.blend = blend;
            }

            if let Some(filter) = Self::get_i32(&dst_table, "filter") {
                obj_data.filter = filter;
            }

            if let Some(stretch) = Self::get_i32(&dst_table, "stretch") {
                obj_data.stretch = stretch;
            }

            // Parse dst array (keyframes)
            if let Ok(dst_array) = dst_table.get::<Table>("dst") {
                for keyframe_entry in dst_array.sequence_values::<Table>() {
                    let keyframe_table = keyframe_entry.to_anyhow()?;
                    let dst = self.parse_destination_keyframe(&keyframe_table)?;
                    obj_data.dst.push(dst);
                }
            }

            if !obj_data.id.is_empty() && obj_data.dst.is_empty() {
                // Expand judge sub-objects if this is a judge placeholder
                if let Some(judge_sub) = judge_map.get(&obj_data.id) {
                    self.expand_judge_sub_objects(judge_sub, skin)?;
                }
                continue;
            }

            if !obj_data.id.is_empty() && !obj_data.dst.is_empty() {
                skin.objects.push(obj_data);
            }
        }

        Ok(())
    }

    /// Option IDs for each judge rank (PG, GR, GD, BD, PR, MS).
    const JUDGE_OP_IDS: [i32; 6] = [
        OPTION_1P_PERFECT,
        OPTION_1P_GREAT,
        OPTION_1P_GOOD,
        OPTION_1P_BAD,
        OPTION_1P_POOR,
        OPTION_1P_MISS,
    ];

    fn expand_judge_sub_objects(&self, judge_sub: &JudgeSubObjects, skin: &mut Skin) -> Result<()> {
        // Expand image sub-objects (one per judge rank)
        for (i, sub_table) in judge_sub.images.iter().enumerate() {
            let op_id = Self::JUDGE_OP_IDS.get(i).copied().unwrap_or(0);
            let mut obj_data = self.parse_sub_object_entry(sub_table, skin)?;
            obj_data.op.push(op_id);
            if !obj_data.id.is_empty() && !obj_data.dst.is_empty() {
                skin.objects.push(obj_data);
            }
        }

        // Expand number sub-objects (one per judge rank)
        for (i, sub_table) in judge_sub.numbers.iter().enumerate() {
            let op_id = Self::JUDGE_OP_IDS.get(i).copied().unwrap_or(0);
            let mut obj_data = self.parse_sub_object_entry(sub_table, skin)?;
            obj_data.op.push(op_id);

            // Override ref to NUMBER_COMBO for judge number display
            if let Some(num_def) = skin.numbers.get_mut(&obj_data.id) {
                num_def.ref_id = NUMBER_COMBO;
            }

            if !obj_data.id.is_empty() && !obj_data.dst.is_empty() {
                skin.objects.push(obj_data);
            }
        }

        Ok(())
    }

    /// Parse a judge sub-object entry (from skin.judge images/numbers arrays)
    /// into a SkinObjectData, determining object type from the skin definitions.
    fn parse_sub_object_entry(&self, sub_table: &Table, skin: &Skin) -> Result<SkinObjectData> {
        let mut obj_data = SkinObjectData::default();

        if let Ok(id) = sub_table.get::<String>("id") {
            obj_data.id = id;
        } else if let Ok(id_num) = sub_table.get::<i32>("id") {
            obj_data.id = id_num.to_string();
        }

        // Determine object type
        if !obj_data.id.is_empty() {
            if skin.numbers.contains_key(&obj_data.id) {
                obj_data.object_type = SkinObjectType::Number;
            } else if skin.texts.contains_key(&obj_data.id) {
                obj_data.object_type = SkinObjectType::Text;
            } else if skin.image_sets.contains_key(&obj_data.id) {
                obj_data.object_type = SkinObjectType::ImageSet;
            } else {
                obj_data.object_type = SkinObjectType::Image;
            }
        }

        // Parse op
        if let Ok(op_table) = sub_table.get::<Table>("op") {
            for pair in op_table.pairs::<Value, Value>() {
                let (_, op_value) = pair.to_anyhow()?;
                match op_value {
                    Value::Integer(op_id) => obj_data.op.push(op_id as i32),
                    Value::Number(op_id) => obj_data.op.push(op_id as i32),
                    _ => {}
                }
            }
        }

        if let Some(timer) = Self::get_i32(sub_table, "timer") {
            obj_data.timer = timer;
        }

        if let Some(loop_count) = Self::get_i32(sub_table, "loop") {
            obj_data.loop_count = loop_count;
        }

        if let Some(offset) = Self::get_i32(sub_table, "offset") {
            obj_data.offset = offset;
        }

        if let Ok(offsets_table) = sub_table.get::<Table>("offsets") {
            for value in offsets_table.sequence_values::<Value>() {
                let value = value.to_anyhow()?;
                let offset_id = match value {
                    Value::Integer(id) => Some(id as i32),
                    Value::Number(id) => Some(id as i32),
                    _ => None,
                };
                if let Some(offset_id) = offset_id {
                    obj_data.offsets.push(offset_id);
                }
            }
        }

        if let Some(blend) = Self::get_i32(sub_table, "blend") {
            obj_data.blend = blend;
        }

        if let Some(filter) = Self::get_i32(sub_table, "filter") {
            obj_data.filter = filter;
        }

        if let Some(stretch) = Self::get_i32(sub_table, "stretch") {
            obj_data.stretch = stretch;
        }

        // Parse dst array (keyframes)
        if let Ok(dst_array) = sub_table.get::<Table>("dst") {
            for keyframe_entry in dst_array.sequence_values::<Table>() {
                let keyframe_table = keyframe_entry.to_anyhow()?;
                let dst = self.parse_destination_keyframe(&keyframe_table)?;
                obj_data.dst.push(dst);
            }
        }

        Ok(obj_data)
    }

    fn parse_destination_keyframe(&self, table: &Table) -> Result<Destination> {
        let mut dst = Destination::default();

        if let Some(time) = Self::get_i32(table, "time") {
            dst.time = time;
        }

        if let Some(x) = Self::get_f32(table, "x") {
            dst.x = x;
        }

        if let Some(y) = Self::get_f32(table, "y") {
            dst.y = y;
        }

        if let Some(w) = Self::get_f32(table, "w") {
            dst.w = w;
        }

        if let Some(h) = Self::get_f32(table, "h") {
            dst.h = h;
        }

        if let Some(acc) = Self::get_i32(table, "acc") {
            dst.acc = acc;
        }

        if let Some(a) = Self::get_f32(table, "a") {
            dst.a = a;
        }

        if let Some(r) = Self::get_f32(table, "r") {
            dst.r = r;
        }

        if let Some(g) = Self::get_f32(table, "g") {
            dst.g = g;
        }

        if let Some(b) = Self::get_f32(table, "b") {
            dst.b = b;
        }

        if let Some(angle) = Self::get_f32(table, "angle") {
            dst.angle = angle;
        }

        if let Some(center) = Self::get_i32(table, "center") {
            dst.center = center;
        }

        Ok(dst)
    }
}

impl Default for LuaSkinLoader {
    fn default() -> Self {
        Self::new().expect("Failed to create Lua skin loader")
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn test_loader_creation() {
        let loader = LuaSkinLoader::new();
        assert!(loader.is_ok());
    }

    #[test]
    fn test_load_ecfn_header() {
        let loader = LuaSkinLoader::new().unwrap();
        let skin_path = PathBuf::from("skins/ECFN/play/play7main.lua");

        if skin_path.exists() {
            let header = loader.load_header(&skin_path);
            assert!(header.is_ok(), "Failed to load header: {:?}", header.err());

            let header = header.unwrap();
            assert_eq!(header.name, "EC:FN(7K:AC)");
            assert_eq!(header.skin_type, SkinType::Play7);
            assert_eq!(header.width, 1920);
            assert_eq!(header.height, 1080);
        }
    }

    #[test]
    fn test_load_ecfn_skin() {
        let loader = LuaSkinLoader::new().unwrap();
        let skin_path = PathBuf::from("skins/ECFN/play/play7main.lua");

        if skin_path.exists() {
            let mut options = HashMap::new();
            // Set default options
            options.insert("プレーサイド".to_string(), 920); // 1P
            options.insert("スコアグラフ".to_string(), 900); // Off
            options.insert("スコアグラフ位置".to_string(), 902); // default
            options.insert("ジャッジカウント".to_string(), 906); // On
            options.insert("手元用クロマキー".to_string(), 924); // Off
            options.insert("スコア差分".to_string(), 907); // Off
            options.insert("判定タイミング表示".to_string(), 910); // Off
            options.insert("判定タイミング値表示(グラフ必須)".to_string(), 915); // Off
            options.insert("ステージファイル表示".to_string(), 926); // Off
            options.insert("BGA表示方法".to_string(), 929); // Full
            options.insert("閉店アニメ".to_string(), 931); // Light
            options.insert("判定文字規格".to_string(), 932); // FullHD

            let skin = loader.load(&skin_path, &options);
            assert!(skin.is_ok(), "Failed to load skin: {:?}", skin.err());

            let skin = skin.unwrap();
            assert!(!skin.sources.is_empty(), "Sources should not be empty");
            assert!(!skin.objects.is_empty(), "Objects should not be empty");
        }
    }
}
