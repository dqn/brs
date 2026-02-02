use std::collections::HashMap;
use std::path::Path;

use anyhow::{Context, Result, anyhow};
use mlua::{Lua, Table, Value};

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

        // Set up skin_config global
        self.setup_skin_config(options)?;

        // Execute the skin file
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read skin file: {}", path.display()))?;

        let result: Table = self
            .lua
            .load(&content)
            .set_name(path.to_string_lossy())
            .eval()
            .to_anyhow()?;

        self.parse_skin(&result, skin_dir)
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

        if let Ok(name) = table.get::<String>("name") {
            header.name = name;
        }

        if let Ok(author) = table.get::<String>("author") {
            header.author = author;
        }

        if let Ok(skin_type) = table.get::<i32>("type") {
            if let Some(st) = SkinType::from_i32(skin_type) {
                header.skin_type = st;
            }
        }

        if let Ok(w) = table.get::<u32>("w") {
            header.width = w;
        }

        if let Ok(h) = table.get::<u32>("h") {
            header.height = h;
        }

        if let Ok(loadend) = table.get::<i32>("loadend") {
            header.loadend = loadend;
        }

        if let Ok(playstart) = table.get::<i32>("playstart") {
            header.playstart = playstart;
        }

        if let Ok(scene) = table.get::<i32>("scene") {
            header.scene = scene;
        }

        if let Ok(input) = table.get::<i32>("input") {
            header.input = input;
        }

        if let Ok(close) = table.get::<i32>("close") {
            header.close = close;
        }

        if let Ok(fadeout) = table.get::<i32>("fadeout") {
            header.fadeout = fadeout;
        }

        Ok(header)
    }

    fn parse_skin(&self, table: &Table, skin_dir: &Path) -> Result<Skin> {
        let header = self.parse_header(table)?;
        let mut skin = Skin::new(header);

        // Parse source array
        if let Ok(source_table) = table.get::<Table>("source") {
            self.parse_sources(&source_table, skin_dir, &mut skin)?;
        }

        // Parse image definitions
        if let Ok(image_table) = table.get::<Table>("image") {
            self.parse_images(&image_table, &mut skin)?;
        }

        // Parse imageset definitions
        if let Ok(imageset_table) = table.get::<Table>("imageset") {
            self.parse_imagesets(&imageset_table, &mut skin)?;
        }

        // Parse value (number) definitions
        if let Ok(value_table) = table.get::<Table>("value") {
            self.parse_values(&value_table, &mut skin)?;
        }

        // Parse font definitions
        if let Ok(font_table) = table.get::<Table>("font") {
            self.parse_fonts(&font_table, &mut skin)?;
        }

        // Parse text definitions
        if let Ok(text_table) = table.get::<Table>("text") {
            self.parse_texts(&text_table, &mut skin)?;
        }

        // Parse destination (objects)
        if let Ok(dst_table) = table.get::<Table>("destination") {
            self.parse_destinations(&dst_table, &mut skin)?;
        }

        Ok(skin)
    }

    fn parse_sources(&self, table: &Table, _skin_dir: &Path, skin: &mut Skin) -> Result<()> {
        for pair in table.pairs::<Value, Table>() {
            let (_, src_table) = pair.to_anyhow()?;

            let id: u32 = src_table.get("id").unwrap_or(0);
            let path: String = src_table.get("path").unwrap_or_default();

            skin.sources.insert(id, SkinSource { id, path });
        }

        Ok(())
    }

    fn parse_images(&self, table: &Table, skin: &mut Skin) -> Result<()> {
        for pair in table.pairs::<Value, Table>() {
            let (_, img_table) = pair.to_anyhow()?;

            let mut image_def = ImageDef::default();

            if let Ok(id) = img_table.get::<String>("id") {
                image_def.id = id;
            }

            if let Ok(src) = img_table.get::<u32>("src") {
                image_def.src = src;
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

    fn parse_imagesets(&self, table: &Table, skin: &mut Skin) -> Result<()> {
        for pair in table.pairs::<Value, Table>() {
            let (_, set_table) = pair.to_anyhow()?;

            let mut imageset_def = ImageSetDef::default();

            if let Ok(id) = set_table.get::<String>("id") {
                imageset_def.id = id;
            }

            if let Ok(mode) = set_table.get::<i32>("mode") {
                imageset_def.mode = mode;
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

    fn parse_values(&self, table: &Table, skin: &mut Skin) -> Result<()> {
        for pair in table.pairs::<Value, Table>() {
            let (_, val_table) = pair.to_anyhow()?;

            let mut number_def = NumberDef::default();

            if let Ok(id) = val_table.get::<String>("id") {
                number_def.id = id;
            }

            if let Ok(src) = val_table.get::<u32>("src") {
                number_def.src = src;
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

            if let Ok(zeropadding) = val_table.get::<i32>("padding") {
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

    fn parse_destinations(&self, table: &Table, skin: &mut Skin) -> Result<()> {
        for pair in table.pairs::<Value, Table>() {
            let (_, dst_table) = pair.to_anyhow()?;

            let mut obj_data = SkinObjectData::default();

            if let Ok(id) = dst_table.get::<String>("id") {
                obj_data.id = id.clone();

                // Determine object type based on definition type
                if skin.numbers.contains_key(&id) {
                    obj_data.object_type = SkinObjectType::Number;
                } else if skin.texts.contains_key(&id) {
                    obj_data.object_type = SkinObjectType::Text;
                } else if skin.image_sets.contains_key(&id) {
                    obj_data.object_type = SkinObjectType::ImageSet;
                } else {
                    obj_data.object_type = SkinObjectType::Image;
                }
            }

            // Parse op (option conditions)
            if let Ok(op_table) = dst_table.get::<Table>("op") {
                for pair in op_table.pairs::<Value, i32>() {
                    let (_, op_id) = pair.to_anyhow()?;
                    obj_data.op.push(op_id);
                }
            }

            if let Ok(timer) = dst_table.get::<i32>("timer") {
                obj_data.timer = timer;
            }

            if let Ok(loop_count) = dst_table.get::<i32>("loop") {
                obj_data.loop_count = loop_count;
            }

            if let Ok(offset) = dst_table.get::<i32>("offset") {
                obj_data.offset = offset;
            }

            if let Ok(blend) = dst_table.get::<i32>("blend") {
                obj_data.blend = blend;
            }

            if let Ok(filter) = dst_table.get::<i32>("filter") {
                obj_data.filter = filter;
            }

            if let Ok(stretch) = dst_table.get::<i32>("stretch") {
                obj_data.stretch = stretch;
            }

            // Parse dst array (keyframes)
            if let Ok(dst_array) = dst_table.get::<Table>("dst") {
                for pair in dst_array.pairs::<Value, Table>() {
                    let (_, keyframe_table) = pair.to_anyhow()?;
                    let dst = self.parse_destination_keyframe(&keyframe_table)?;
                    obj_data.dst.push(dst);
                }
            }

            if !obj_data.id.is_empty() && !obj_data.dst.is_empty() {
                skin.objects.push(obj_data);
            }
        }

        Ok(())
    }

    fn parse_destination_keyframe(&self, table: &Table) -> Result<Destination> {
        let mut dst = Destination::default();

        if let Ok(time) = table.get::<i32>("time") {
            dst.time = time;
        }

        if let Ok(x) = table.get::<f32>("x") {
            dst.x = x;
        }

        if let Ok(y) = table.get::<f32>("y") {
            dst.y = y;
        }

        if let Ok(w) = table.get::<f32>("w") {
            dst.w = w;
        }

        if let Ok(h) = table.get::<f32>("h") {
            dst.h = h;
        }

        if let Ok(acc) = table.get::<i32>("acc") {
            dst.acc = acc;
        }

        if let Ok(a) = table.get::<f32>("a") {
            dst.a = a;
        }

        if let Ok(r) = table.get::<f32>("r") {
            dst.r = r;
        }

        if let Ok(g) = table.get::<f32>("g") {
            dst.g = g;
        }

        if let Ok(b) = table.get::<f32>("b") {
            dst.b = b;
        }

        if let Ok(angle) = table.get::<f32>("angle") {
            dst.angle = angle;
        }

        if let Ok(center) = table.get::<i32>("center") {
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
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_loader_creation() {
        let loader = LuaSkinLoader::new();
        assert!(loader.is_ok());
    }

    #[test]
    fn test_load_ecfn_header() {
        let loader = LuaSkinLoader::new().unwrap();
        let skin_path = PathBuf::from("skins/ECFN/play/play7.luaskin");

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
        let skin_path = PathBuf::from("skins/ECFN/play/play7.luaskin");

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
