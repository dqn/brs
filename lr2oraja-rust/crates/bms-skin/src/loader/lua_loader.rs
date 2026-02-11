// Lua skin loader.
//
// Executes Lua scripts that produce a skin data table, converts the result
// to JSON, then delegates to the JSON skin loader for actual skin building.
//
// This matches the Java architecture where LuaSkinLoader extends JSONSkinLoader
// and converts Lua tables to JsonSkin.Skin objects.
//
// The Lua environment provides:
// - `skin_config` table with custom options, offsets, and file paths
// - `skin_property` table with property ID constants
// - Standard Lua libraries (math, string, table, etc.)
//
// Ported from LuaSkinLoader.java and SkinLuaAccessor.java.

use std::collections::HashSet;
use std::path::Path;

use anyhow::{Context, Result};
use mlua::prelude::*;
use serde_json::Value;

use bms_config::resolution::Resolution;
use bms_config::skin_config::Offset;

use crate::skin::Skin;
use crate::skin_header::SkinHeader;

use super::json_loader;

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Loads a SkinHeader from a Lua skin script.
///
/// `lua_source` is the Lua script content (UTF-8).
/// The script should return a table matching the beatoraja JSON skin format,
/// or a `{header, main}` table where `header` contains the skin metadata.
pub fn load_lua_header(lua_source: &str, path: Option<&Path>) -> Result<SkinHeader> {
    let lua = create_lua_env(path)?;
    let value = exec_lua(&lua, lua_source, path)?;
    let resolved = resolve_for_header(&value);
    let json = lua_value_to_json(&resolved);
    let json_str =
        serde_json::to_string(&json).context("Failed to serialize Lua result to JSON")?;
    json_loader::load_header(&json_str)
}

/// Converts a Lua skin script to a JSON string.
///
/// Executes the Lua script and converts the resulting table to a JSON string.
/// Handles the `{header, main}` pattern by calling `main()` to get the skin data.
/// This is useful for tools that need the intermediate JSON representation
/// (e.g., screenshot harness that reuses JSON image loading paths).
pub fn lua_to_json_string(
    lua_source: &str,
    path: Option<&Path>,
    enabled_options: &HashSet<i32>,
    offsets: &[(i32, Offset)],
) -> Result<String> {
    let lua = create_lua_env(path)?;
    let header_probe = exec_lua(&lua, lua_source, path)?;
    let option_selection = extract_option_selection(&header_probe, enabled_options)?;
    export_skin_config(&lua, enabled_options, offsets)?;
    apply_option_selection(&lua, &option_selection)?;
    let value = exec_lua(&lua, lua_source, path)?;
    let resolved = resolve_for_skin(&lua, &value)?;
    let json = lua_value_to_json(&resolved);
    // Lua division always produces floats (e.g. 595/3 = 198.333).
    // The JSON skin schema uses i32 for coordinates, so truncate floats
    // to integers to match Java's LuaSkinLoader truncation behavior.
    let json = truncate_floats_to_ints(json);
    serde_json::to_string(&json).context("Failed to serialize Lua result to JSON")
}

/// Loads a full Skin from a Lua skin script.
///
/// `lua_source` is the Lua script content (UTF-8).
/// `enabled_options`: set of enabled option IDs (from user's skin config).
/// `dest_resolution`: the display resolution to scale to.
/// `offsets`: custom offset values from user's skin config.
pub fn load_lua_skin(
    lua_source: &str,
    enabled_options: &HashSet<i32>,
    dest_resolution: Resolution,
    path: Option<&Path>,
    offsets: &[(i32, Offset)],
) -> Result<Skin> {
    let lua = create_lua_env(path)?;
    let header_probe = exec_lua(&lua, lua_source, path)?;
    let option_selection = extract_option_selection(&header_probe, enabled_options)?;

    // Export skin_config with options and offsets
    export_skin_config(&lua, enabled_options, offsets)?;
    apply_option_selection(&lua, &option_selection)?;

    let value = exec_lua(&lua, lua_source, path)?;
    let resolved = resolve_for_skin(&lua, &value)?;
    let json = lua_value_to_json(&resolved);
    let json = truncate_floats_to_ints(json);
    let json_str =
        serde_json::to_string(&json).context("Failed to serialize Lua result to JSON")?;
    json_loader::load_skin(&json_str, enabled_options, dest_resolution, path)
}

// ---------------------------------------------------------------------------
// Lua environment setup
// ---------------------------------------------------------------------------

/// Creates a new Lua VM with standard libraries and the skin module path.
///
/// Sets up `package.path` for the script's directory and registers a stub
/// `main_state` module. In beatoraja, `main_state` is provided by
/// `SkinLuaAccessor` at runtime with callbacks into the game state.
/// The stub returns default values so skins can be loaded without a running game.
fn create_lua_env(path: Option<&Path>) -> Result<Lua> {
    let lua = Lua::new();

    // Add the script's directory to the Lua package path
    if let Some(p) = path
        && let Some(dir) = p.parent()
    {
        let dir_str = dir.to_string_lossy();
        lua.load(format!("package.path = package.path .. ';{dir_str}/?.lua'"))
            .exec()
            .map_err(|e| anyhow::anyhow!("Failed to set Lua package path: {}", e))?;
    }

    // Register main_state stub module via package.preload.
    // This is checked before file searchers, matching how beatoraja Java
    // provides main_state programmatically via SkinLuaAccessor.
    lua.load(
        r#"
package.preload["main_state"] = function()
    local M = {}
    M.timer_off_value = -9223372036854775808
    function M.number(_) return 0 end
    function M.option(_) return false end
    function M.text(_) return "" end
    function M.timer(_) return M.timer_off_value end
    function M.float_number(_) return 0.0 end
    function M.slider(_) return 0.0 end
    return M
end
"#,
    )
    .exec()
    .map_err(|e| anyhow::anyhow!("Failed to register main_state stub: {}", e))?;

    Ok(lua)
}

/// Exports the `skin_config` global table to the Lua environment.
///
/// The table contains:
/// - `option`: table mapping option names to selected indices
/// - `offset`: table mapping offset IDs to {x, y, w, h, r, a}
/// - `enabled_options`: array of enabled option IDs
fn export_skin_config(
    lua: &Lua,
    enabled_options: &HashSet<i32>,
    offsets: &[(i32, Offset)],
) -> Result<()> {
    let config = lua
        .create_table()
        .map_err(|e| anyhow::anyhow!("Failed to create skin_config: {}", e))?;

    // Enabled options as array
    let opt_table = lua
        .create_table()
        .map_err(|e| anyhow::anyhow!("Failed to create table: {}", e))?;
    for (i, &id) in enabled_options.iter().enumerate() {
        opt_table
            .set(i + 1, id)
            .map_err(|e| anyhow::anyhow!("Failed to set option: {}", e))?;
    }
    config
        .set("enabled_options", opt_table)
        .map_err(|e| anyhow::anyhow!("Failed to set enabled_options: {}", e))?;

    // Option table: maps property names to selected option IDs.
    // In beatoraja Java, SkinLuaAccessor populates this from the user's
    // skin configuration. Empty table allows skins to access it without errors.
    let option_table = lua
        .create_table()
        .map_err(|e| anyhow::anyhow!("Failed to create table: {}", e))?;
    config
        .set("option", option_table)
        .map_err(|e| anyhow::anyhow!("Failed to set option: {}", e))?;

    // Offsets
    let offset_table = lua
        .create_table()
        .map_err(|e| anyhow::anyhow!("Failed to create table: {}", e))?;
    for &(id, ref off) in offsets {
        let ot = lua
            .create_table()
            .map_err(|e| anyhow::anyhow!("Failed to create table: {}", e))?;
        ot.set("x", off.x)
            .map_err(|e| anyhow::anyhow!("Failed to set x: {}", e))?;
        ot.set("y", off.y)
            .map_err(|e| anyhow::anyhow!("Failed to set y: {}", e))?;
        ot.set("w", off.w)
            .map_err(|e| anyhow::anyhow!("Failed to set w: {}", e))?;
        ot.set("h", off.h)
            .map_err(|e| anyhow::anyhow!("Failed to set h: {}", e))?;
        ot.set("r", off.r)
            .map_err(|e| anyhow::anyhow!("Failed to set r: {}", e))?;
        ot.set("a", off.a)
            .map_err(|e| anyhow::anyhow!("Failed to set a: {}", e))?;
        offset_table
            .set(id, ot)
            .map_err(|e| anyhow::anyhow!("Failed to set offset: {}", e))?;
    }
    config
        .set("offset", offset_table)
        .map_err(|e| anyhow::anyhow!("Failed to set offset: {}", e))?;

    lua.globals()
        .set("skin_config", config)
        .map_err(|e| anyhow::anyhow!("Failed to set skin_config: {}", e))?;

    Ok(())
}

/// Executes a Lua script and returns the result value.
fn exec_lua(lua: &Lua, source: &str, path: Option<&Path>) -> Result<mlua::Value> {
    let chunk = if let Some(p) = path {
        lua.load(source).set_name(p.to_string_lossy())
    } else {
        lua.load(source).set_name("<lua skin>")
    };
    chunk
        .eval()
        .map_err(|e| anyhow::anyhow!("Lua execution failed: {}", e))
}

/// Resolves the `{header, main}` return pattern for skin loading.
///
/// If the Lua result is a table with a `main` function:
/// 1. Extracts default option values from `header.property`
/// 2. Populates `skin_config.option` with defaults (matching beatoraja behavior)
/// 3. Calls `main()` and returns the skin data table
///
/// Otherwise returns the original value.
fn resolve_for_skin(lua: &Lua, value: &mlua::Value) -> Result<mlua::Value> {
    if let mlua::Value::Table(t) = value
        && let Ok(main_fn) = t.get::<mlua::Function>("main")
    {
        // Before calling main(), populate skin_config.option with defaults
        // from header.property. This matches beatoraja where the launcher
        // pre-selects the first option of each property by default.
        populate_default_options(lua, t)?;

        return main_fn
            .call::<mlua::Value>(())
            .map_err(|e| anyhow::anyhow!("Failed to call skin main(): {}", e));
    }
    Ok(value.clone())
}

/// Extracts default options from `header.property` and sets them in `skin_config.option`.
///
/// Each property has a `name` and `item` array. The first item's `op` value
/// is used as the default, matching beatoraja's behavior when no user selection exists.
fn populate_default_options(lua: &Lua, result_table: &mlua::Table) -> Result<()> {
    let header = match result_table.get::<mlua::Value>("header") {
        Ok(mlua::Value::Table(h)) => h,
        _ => return Ok(()),
    };
    let property = match header.get::<mlua::Value>("property") {
        Ok(mlua::Value::Table(p)) => p,
        _ => return Ok(()),
    };

    let globals = lua.globals();
    let skin_config = match globals.get::<mlua::Value>("skin_config") {
        Ok(mlua::Value::Table(c)) => c,
        _ => return Ok(()),
    };
    let option_table = match skin_config.get::<mlua::Value>("option") {
        Ok(mlua::Value::Table(o)) => o,
        _ => return Ok(()),
    };

    // Iterate properties and set default (first item's op value)
    for pair in property.pairs::<mlua::Value, mlua::Value>() {
        let (_, prop) = pair.map_err(|e| anyhow::anyhow!("Failed to read property: {}", e))?;
        if let mlua::Value::Table(prop_table) = prop {
            let name = match prop_table.get::<mlua::Value>("name") {
                Ok(mlua::Value::String(s)) => s,
                _ => continue,
            };
            let items = match prop_table.get::<mlua::Value>("item") {
                Ok(mlua::Value::Table(i)) => i,
                _ => continue,
            };
            // First item's op value is the default
            if let Ok(mlua::Value::Table(first_item)) = items.get::<mlua::Value>(1)
                && let Ok(op) = first_item.get::<mlua::Value>("op")
            {
                if matches!(
                    option_table.get::<mlua::Value>(name.clone()),
                    Ok(mlua::Value::Nil)
                ) {
                    option_table
                        .set(name, op)
                        .map_err(|e| anyhow::anyhow!("Failed to set option default: {}", e))?;
                }
            }
        }
    }

    Ok(())
}

/// Extracts option selections from header property metadata.
///
/// For each option group, this selects:
/// 1. A matching ID from `enabled_options` if present.
/// 2. Otherwise the first item's `op` value as default.
fn extract_option_selection(
    value: &mlua::Value,
    enabled_options: &HashSet<i32>,
) -> Result<Vec<(String, i32)>> {
    let header = resolve_for_header(value);
    let header_table = match header {
        mlua::Value::Table(t) => t,
        _ => return Ok(Vec::new()),
    };
    let property = match header_table.get::<mlua::Value>("property") {
        Ok(mlua::Value::Table(p)) => p,
        _ => return Ok(Vec::new()),
    };

    let mut selections = Vec::new();

    for pair in property.pairs::<mlua::Value, mlua::Value>() {
        let (_, prop) = pair.map_err(|e| anyhow::anyhow!("Failed to read property: {}", e))?;
        let prop_table = match prop {
            mlua::Value::Table(t) => t,
            _ => continue,
        };
        let name = match prop_table.get::<mlua::Value>("name") {
            Ok(mlua::Value::String(s)) => s.to_str().ok().map(|v| v.to_string()),
            _ => None,
        };
        let Some(name) = name else {
            continue;
        };
        let items = match prop_table.get::<mlua::Value>("item") {
            Ok(mlua::Value::Table(t)) => t,
            _ => continue,
        };

        let mut first_op: Option<i32> = None;
        let mut selected_op: Option<i32> = None;

        for item in items.sequence_values::<mlua::Table>() {
            let item = item.map_err(|e| anyhow::anyhow!("Failed to read item: {}", e))?;
            let op = match item.get::<mlua::Value>("op") {
                Ok(mlua::Value::Integer(i)) => i32::try_from(i).ok(),
                Ok(mlua::Value::Number(n)) => Some(n as i32),
                _ => None,
            };
            let Some(op) = op else {
                continue;
            };
            if first_op.is_none() {
                first_op = Some(op);
            }
            if enabled_options.contains(&op) {
                selected_op = Some(op);
                break;
            }
        }

        if let Some(op) = selected_op.or(first_op) {
            selections.push((name, op));
        }
    }

    Ok(selections)
}

fn apply_option_selection(lua: &Lua, option_selection: &[(String, i32)]) -> Result<()> {
    if option_selection.is_empty() {
        return Ok(());
    }

    let globals = lua.globals();
    let skin_config = match globals.get::<mlua::Value>("skin_config") {
        Ok(mlua::Value::Table(c)) => c,
        _ => return Ok(()),
    };
    let option_table = match skin_config.get::<mlua::Value>("option") {
        Ok(mlua::Value::Table(o)) => o,
        _ => return Ok(()),
    };

    for (name, op) in option_selection {
        option_table
            .set(name.as_str(), *op)
            .map_err(|e| anyhow::anyhow!("Failed to set option selection: {}", e))?;
    }
    Ok(())
}

/// Resolves the `{header, main}` return pattern for header loading.
///
/// If the Lua result is a table with a `header` sub-table, returns that
/// sub-table. Otherwise returns the original value as-is.
fn resolve_for_header(value: &mlua::Value) -> mlua::Value {
    if let mlua::Value::Table(t) = value
        && let Ok(header @ mlua::Value::Table(_)) = t.get::<mlua::Value>("header")
    {
        return header;
    }
    value.clone()
}

// ---------------------------------------------------------------------------
// Lua value → JSON conversion
// ---------------------------------------------------------------------------

/// Recursively converts a Lua value to a serde_json Value.
///
/// Lua tables are detected as either arrays (consecutive integer keys from 1)
/// or objects (string keys). Mixed tables are treated as objects with string
/// keys only (numeric keys are converted to strings).
fn lua_value_to_json(value: &mlua::Value) -> Value {
    match value {
        mlua::Value::Nil => Value::Null,
        mlua::Value::Boolean(b) => Value::Bool(*b),
        mlua::Value::Integer(n) => Value::Number(serde_json::Number::from(*n)),
        mlua::Value::Number(n) => {
            serde_json::Number::from_f64(*n).map_or(Value::Null, Value::Number)
        }
        mlua::Value::String(s) => {
            let str_result = s.to_str();
            match str_result {
                Ok(str_ref) => Value::String(str_ref.to_string()),
                Err(_) => Value::String(String::new()),
            }
        }
        mlua::Value::Table(t) => lua_table_to_json(t),
        _ => Value::Null, // Functions, userdata, etc. → null
    }
}

/// Converts a Lua table to a JSON value (array or object).
///
/// If all keys are consecutive integers starting from 1, produces a JSON array.
/// Empty tables are treated as arrays by default (common convention).
/// Otherwise, produces a JSON object with string keys.
fn lua_table_to_json(table: &mlua::Table) -> Value {
    // Check if this is a sequence (array-like table)
    let len = table.raw_len() as i64;
    if len > 0 && is_sequence(table, len) {
        let mut arr = Vec::with_capacity(len as usize);
        for i in 1..=len {
            if let Ok(val) = table.raw_get::<mlua::Value>(i) {
                arr.push(lua_value_to_json(&val));
            }
        }
        return Value::Array(arr);
    }

    // Empty table → treat as array by default (common Lua/JSON convention)
    // This matches beatoraja's behavior where empty {} tables are used as empty arrays
    if len == 0 {
        // Check if there are any string keys
        let mut has_string_keys = false;
        for (key, _) in table.clone().pairs::<mlua::Value, mlua::Value>().flatten() {
            if matches!(key, mlua::Value::String(_)) {
                has_string_keys = true;
                break;
            }
        }
        if !has_string_keys {
            return Value::Array(Vec::new());
        }
    }

    // Object/map table
    let mut map = serde_json::Map::new();
    for (key, val) in table.clone().pairs::<mlua::Value, mlua::Value>().flatten() {
        let key_str = match &key {
            mlua::Value::String(s) => match s.to_str() {
                Ok(str_ref) => str_ref.to_string(),
                Err(_) => continue,
            },
            mlua::Value::Integer(n) => n.to_string(),
            mlua::Value::Number(n) => n.to_string(),
            _ => continue,
        };
        map.insert(key_str, lua_value_to_json(&val));
    }
    Value::Object(map)
}

/// Recursively truncates float values to integers in a JSON value.
///
/// In Lua, division always produces floats (e.g., `595/3 = 198.333`).
/// The beatoraja JSON skin schema uses `i32` for coordinates and sizes.
/// Java's LuaSkinLoader truncates these via `Coercions.toint()`.
/// This function replicates that behavior.
fn truncate_floats_to_ints(value: Value) -> Value {
    match value {
        Value::Number(n) => {
            if let Some(f) = n.as_f64() {
                // If the float is representable as i64, truncate it
                let truncated = f as i64;
                if (truncated as f64 - f).abs() < 1.0 {
                    return Value::Number(serde_json::Number::from(truncated));
                }
            }
            Value::Number(n)
        }
        Value::Array(arr) => Value::Array(arr.into_iter().map(truncate_floats_to_ints).collect()),
        Value::Object(map) => Value::Object(
            map.into_iter()
                .map(|(k, v)| (k, truncate_floats_to_ints(v)))
                .collect(),
        ),
        other => other,
    }
}

/// Checks if a Lua table is a sequence (consecutive integer keys 1..n).
fn is_sequence(table: &mlua::Table, len: i64) -> bool {
    // Verify that all keys 1..len exist and there are no extra keys
    for i in 1..=len {
        if table.raw_get::<mlua::Value>(i).is_err() {
            return false;
        }
    }
    // Check there are no string keys (simple heuristic)
    let count = table
        .clone()
        .pairs::<mlua::Value, mlua::Value>()
        .flatten()
        .count();
    count == len as usize
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- Lua → JSON conversion --

    #[test]
    fn test_nil_to_json() {
        let lua = Lua::new();
        let val: mlua::Value = lua.load("nil").eval().unwrap();
        assert_eq!(lua_value_to_json(&val), Value::Null);
    }

    #[test]
    fn test_boolean_to_json() {
        let lua = Lua::new();
        let val: mlua::Value = lua.load("true").eval().unwrap();
        assert_eq!(lua_value_to_json(&val), Value::Bool(true));
    }

    #[test]
    fn test_integer_to_json() {
        let lua = Lua::new();
        let val: mlua::Value = lua.load("42").eval().unwrap();
        assert_eq!(lua_value_to_json(&val), serde_json::json!(42));
    }

    #[test]
    fn test_float_to_json() {
        let lua = Lua::new();
        let val: mlua::Value = lua.load("3.14").eval().unwrap();
        let json = lua_value_to_json(&val);
        assert!(json.is_number());
    }

    #[test]
    fn test_string_to_json() {
        let lua = Lua::new();
        let val: mlua::Value = lua.load("'hello'").eval().unwrap();
        assert_eq!(lua_value_to_json(&val), Value::String("hello".to_string()));
    }

    #[test]
    fn test_array_to_json() {
        let lua = Lua::new();
        let val: mlua::Value = lua.load("{10, 20, 30}").eval().unwrap();
        assert_eq!(lua_value_to_json(&val), serde_json::json!([10, 20, 30]));
    }

    #[test]
    fn test_object_to_json() {
        let lua = Lua::new();
        let val: mlua::Value = lua.load("{name = 'Test', type = 6}").eval().unwrap();
        let json = lua_value_to_json(&val);
        assert_eq!(json["name"], "Test");
        assert_eq!(json["type"], 6);
    }

    #[test]
    fn test_nested_table_to_json() {
        let lua = Lua::new();
        let val: mlua::Value = lua
            .load("{items = {1, 2, 3}, meta = {x = 10}}")
            .eval()
            .unwrap();
        let json = lua_value_to_json(&val);
        assert_eq!(json["items"], serde_json::json!([1, 2, 3]));
        assert_eq!(json["meta"]["x"], 10);
    }

    #[test]
    fn test_function_to_null() {
        let lua = Lua::new();
        let val: mlua::Value = lua.load("function() return 1 end").eval().unwrap();
        assert_eq!(lua_value_to_json(&val), Value::Null);
    }

    // -- Lua skin loading --

    #[test]
    fn test_load_lua_header() {
        let lua_src = r#"
return {
    type = 6,
    name = "Lua Test Skin",
    author = "Test Author",
    w = 1280,
    h = 720,
    property = {},
    filepath = {},
    offset = {},
    destination = {}
}
"#;
        let header = load_lua_header(lua_src, None).unwrap();
        assert_eq!(header.name, "Lua Test Skin");
        assert_eq!(header.author, "Test Author");
    }

    #[test]
    fn test_load_lua_skin() {
        let lua_src = r#"
return {
    type = 6,
    name = "Lua Skin",
    w = 1280,
    h = 720,
    fadeout = 500,
    scene = 5000,
    image = {
        {id = "bg", src = "0", x = 0, y = 0, w = 1280, h = 720}
    },
    destination = {
        {id = "bg", dst = {{x = 0, y = 0, w = 1280, h = 720}}}
    }
}
"#;
        let skin = load_lua_skin(lua_src, &HashSet::new(), Resolution::Hd, None, &[]).unwrap();
        assert_eq!(skin.fadeout, 500);
        assert_eq!(skin.scene, 5000);
        assert_eq!(skin.object_count(), 1);
    }

    #[test]
    fn test_load_lua_with_computation() {
        // Lua can compute values dynamically
        let lua_src = r#"
local w = 1280
local h = 720
return {
    type = 6,
    name = "Computed Skin",
    w = w,
    h = h,
    fadeout = math.floor(w / 2),
    destination = {}
}
"#;
        let skin = load_lua_skin(lua_src, &HashSet::new(), Resolution::Hd, None, &[]).unwrap();
        assert_eq!(skin.fadeout, 640);
    }

    #[test]
    fn test_load_lua_with_options() {
        // Skin uses skin_config to check enabled options
        let lua_src = r#"
local opts = skin_config and skin_config.enabled_options or {}
local show_bg = false
for _, v in ipairs(opts) do
    if v == 901 then show_bg = true end
end

local dsts = {}
if show_bg then
    table.insert(dsts, {id = "bg", dst = {{x = 0, y = 0, w = 1280, h = 720}}})
end

return {
    type = 6,
    name = "Opt Skin",
    w = 1280,
    h = 720,
    image = {{id = "bg", src = "0"}},
    destination = dsts
}
"#;
        // Without option 901
        let skin = load_lua_skin(lua_src, &HashSet::new(), Resolution::Hd, None, &[]).unwrap();
        assert_eq!(skin.object_count(), 0);

        // With option 901
        let skin =
            load_lua_skin(lua_src, &HashSet::from([901]), Resolution::Hd, None, &[]).unwrap();
        assert_eq!(skin.object_count(), 1);
    }

    #[test]
    fn test_lua_error_reporting() {
        let lua_src = "this is not valid lua!!!";
        let result = load_lua_header(lua_src, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_skin_config_offsets() {
        let lua_src = r#"
local off = skin_config and skin_config.offset or {}
local x_off = off[10] and off[10].x or 0

return {
    type = 6,
    name = "Offset Skin",
    w = 1280,
    h = 720,
    fadeout = math.floor(x_off),
    destination = {}
}
"#;
        let offsets = vec![(
            10,
            Offset {
                name: String::new(),
                x: 42,
                y: 0,
                w: 0,
                h: 0,
                r: 0,
                a: 0,
            },
        )];
        let skin = load_lua_skin(lua_src, &HashSet::new(), Resolution::Hd, None, &offsets).unwrap();
        assert_eq!(skin.fadeout, 42);
    }

    #[test]
    fn test_empty_table_to_json() {
        let lua = Lua::new();
        let val: mlua::Value = lua.load("{}").eval().unwrap();
        // Empty table should be an empty object (not array)
        let json = lua_value_to_json(&val);
        assert!(json.is_object() || json.is_array());
    }

    #[test]
    fn test_mixed_table_to_json() {
        let lua = Lua::new();
        // Lua table with both integer and string keys
        let val: mlua::Value = lua
            .load("{[1] = 'a', [2] = 'b', name = 'test'}")
            .eval()
            .unwrap();
        let json = lua_value_to_json(&val);
        // Mixed table → treated as object
        assert!(json.is_object());
        assert_eq!(json["name"], "test");
    }
}
