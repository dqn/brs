//! beatoraja Lua skin parser
//!
//! Parses .luaskin format by executing Lua code in a sandboxed environment.

use std::fs;
use std::path::Path;

use anyhow::{Context, Result, bail};
use mlua::{Function, Lua, Table, Value};

use super::types::BeatorajaSkin;

/// Parse a beatoraja Lua skin
pub fn parse_lua_skin(wrapper_path: &Path, main_path: &Path) -> Result<BeatorajaSkin> {
    let lua = create_sandbox()?;

    // Set up the skin directory as the base path
    let skin_dir = main_path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("Invalid skin path"))?;

    // Read and execute the main Lua file
    let lua_code = fs::read_to_string(main_path)
        .with_context(|| format!("Failed to read Lua skin: {}", main_path.display()))?;

    // Set package.path to include the skin directory
    setup_package_path(&lua, skin_dir)?;

    // Execute the Lua code
    lua.load(&lua_code)
        .set_name(main_path.to_string_lossy())
        .exec()
        .map_err(|e| {
            anyhow::anyhow!(
                "Failed to execute Lua skin: {} - {}",
                main_path.display(),
                e
            )
        })?;

    // Get skin configuration
    let skin_config = get_skin_config(&lua)?;

    // Call main() function if it exists
    let skin_data = call_main_function(&lua, &skin_config)?;

    // Convert Lua table to BeatorajaSkin
    let mut skin = lua_table_to_skin(&lua, &skin_data)?;

    // Set the skin path
    skin.header.path = skin_dir.to_string_lossy().to_string();

    // Load wrapper metadata if available
    if let Ok(wrapper_content) = fs::read_to_string(wrapper_path) {
        if let Ok(wrapper_data) = serde_json::from_str::<serde_json::Value>(&wrapper_content) {
            if let Some(name) = wrapper_data.get("name").and_then(|v| v.as_str()) {
                if skin.header.name.is_empty() {
                    skin.header.name = name.to_string();
                }
            }
            if let Some(author) = wrapper_data.get("author").and_then(|v| v.as_str()) {
                if skin.header.author.is_empty() {
                    skin.header.author = author.to_string();
                }
            }
        }
    }

    Ok(skin)
}

/// Create a sandboxed Lua environment
fn create_sandbox() -> Result<Lua> {
    let lua = Lua::new();

    // Disable dangerous functions
    lua.globals()
        .set("os", Value::Nil)
        .map_err(|e| anyhow::anyhow!("Failed to disable os: {}", e))?;
    lua.globals()
        .set("io", Value::Nil)
        .map_err(|e| anyhow::anyhow!("Failed to disable io: {}", e))?;
    lua.globals()
        .set("loadfile", Value::Nil)
        .map_err(|e| anyhow::anyhow!("Failed to disable loadfile: {}", e))?;
    lua.globals()
        .set("dofile", Value::Nil)
        .map_err(|e| anyhow::anyhow!("Failed to disable dofile: {}", e))?;

    // Keep safe functions
    // math, string, table, pairs, ipairs, type, etc. are available by default

    Ok(lua)
}

/// Set up Lua package.path to include the skin directory
fn setup_package_path(lua: &Lua, skin_dir: &Path) -> Result<()> {
    let package: Table = lua
        .globals()
        .get("package")
        .map_err(|e| anyhow::anyhow!("Failed to get package: {}", e))?;
    let skin_path = skin_dir.to_string_lossy();

    // Add skin directory to package.path for require()
    let new_path = format!("{}/?.lua;{}/?/init.lua", skin_path, skin_path);
    package
        .set("path", new_path)
        .map_err(|e| anyhow::anyhow!("Failed to set package.path: {}", e))?;

    Ok(())
}

/// Get skin_config table from Lua globals
fn get_skin_config(lua: &Lua) -> Result<Table> {
    match lua.globals().get::<Table>("skin_config") {
        Ok(config) => Ok(config),
        Err(_) => {
            // Create empty config if not defined
            lua.create_table()
                .map_err(|e| anyhow::anyhow!("Failed to create empty config: {}", e))
        }
    }
}

/// Call the main() function to get skin data
fn call_main_function(lua: &Lua, _config: &Table) -> Result<Table> {
    // Try to call main() function
    if let Ok(main_fn) = lua.globals().get::<Function>("main") {
        let result: Table = main_fn
            .call(())
            .map_err(|e| anyhow::anyhow!("Failed to call main() function: {}", e))?;
        return Ok(result);
    }

    // If no main(), look for skin table directly
    if let Ok(skin) = lua.globals().get::<Table>("skin") {
        return Ok(skin);
    }

    // Return empty table if nothing found
    lua.create_table()
        .map_err(|e| anyhow::anyhow!("Failed to create empty skin table: {}", e))
}

/// Convert Lua table to BeatorajaSkin
fn lua_table_to_skin(lua: &Lua, table: &Table) -> Result<BeatorajaSkin> {
    // Convert the Lua table to JSON, then parse as BeatorajaSkin
    let json_value = lua_to_json(lua, &Value::Table(table.clone()))?;
    let json_str = serde_json::to_string(&json_value)?;

    serde_json::from_str(&json_str).context("Failed to convert Lua skin to internal format")
}

/// Convert Lua value to serde_json Value
fn lua_to_json(_lua: &Lua, value: &Value) -> Result<serde_json::Value> {
    match value {
        Value::Nil => Ok(serde_json::Value::Null),
        Value::Boolean(b) => Ok(serde_json::Value::Bool(*b)),
        Value::Integer(i) => Ok(serde_json::Value::Number((*i).into())),
        Value::Number(n) => {
            if let Some(num) = serde_json::Number::from_f64(*n) {
                Ok(serde_json::Value::Number(num))
            } else {
                Ok(serde_json::Value::Null)
            }
        }
        Value::String(s) => {
            let str_val = s
                .to_str()
                .map_err(|e| anyhow::anyhow!("Invalid UTF-8 string: {}", e))?;
            Ok(serde_json::Value::String(str_val.to_string()))
        }
        Value::Table(t) => {
            // Check if this is an array (sequential integer keys starting from 1)
            let is_array = is_lua_array(t);

            if is_array {
                let mut arr = Vec::new();
                let table_clone = t.clone();
                let pairs = table_clone.pairs::<i64, Value>();
                for pair in pairs {
                    match pair {
                        Ok((_, v)) => arr.push(lua_to_json(_lua, &v)?),
                        Err(_) => continue,
                    }
                }
                Ok(serde_json::Value::Array(arr))
            } else {
                let mut map = serde_json::Map::new();
                let table_clone = t.clone();
                let pairs = table_clone.pairs::<Value, Value>();
                for pair in pairs {
                    match pair {
                        Ok((k, v)) => {
                            let key = match k {
                                Value::String(s) => match s.to_str() {
                                    Ok(str_val) => str_val.to_string(),
                                    Err(_) => continue,
                                },
                                Value::Integer(i) => i.to_string(),
                                Value::Number(n) => n.to_string(),
                                _ => continue,
                            };
                            map.insert(key, lua_to_json(_lua, &v)?);
                        }
                        Err(_) => continue,
                    }
                }
                Ok(serde_json::Value::Object(map))
            }
        }
        Value::Function(_) | Value::Thread(_) | Value::UserData(_) | Value::LightUserData(_) => {
            // Skip non-serializable types
            Ok(serde_json::Value::Null)
        }
        Value::Error(e) => bail!("Lua error value: {}", e),
        // Handle any other variants
        _ => Ok(serde_json::Value::Null),
    }
}

/// Check if a Lua table is an array (sequential integer keys starting from 1)
fn is_lua_array(table: &Table) -> bool {
    let len = table.raw_len();
    if len == 0 {
        // Could be empty array or empty object, check for any keys
        let table_clone = table.clone();
        let pairs = table_clone.pairs::<Value, Value>();
        for pair in pairs {
            match pair {
                Ok((k, _)) => match k {
                    Value::Integer(_) => return true,
                    Value::String(_) => return false,
                    _ => {}
                },
                Err(_) => continue,
            }
        }
        // Empty table, treat as object
        return false;
    }

    // Check if all keys from 1 to len exist
    for i in 1..=len {
        match table.raw_get::<Value>(i) {
            Ok(Value::Nil) => return false,
            Err(_) => return false,
            _ => {}
        }
    }

    // Check if there are non-integer keys
    let table_clone = table.clone();
    let pairs = table_clone.pairs::<Value, Value>();
    for pair in pairs {
        match pair {
            Ok((k, _)) => match k {
                Value::Integer(i) if i >= 1 && i <= len as i64 => {}
                Value::Integer(_) => return false, // Out of range integer
                Value::String(_) => return false,  // String key = object
                _ => return false,
            },
            Err(_) => return false,
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_create_sandbox() {
        let lua = create_sandbox().unwrap();

        // Safe functions should be available
        assert!(lua.globals().get::<Table>("math").is_ok());
        assert!(lua.globals().get::<Table>("string").is_ok());

        // Dangerous functions should be disabled
        assert!(lua.globals().get::<Table>("os").is_err());
        assert!(lua.globals().get::<Table>("io").is_err());
    }

    #[test]
    fn test_lua_to_json_primitives() {
        let lua = Lua::new();

        assert_eq!(
            lua_to_json(&lua, &Value::Nil).unwrap(),
            serde_json::Value::Null
        );
        assert_eq!(
            lua_to_json(&lua, &Value::Boolean(true)).unwrap(),
            serde_json::Value::Bool(true)
        );
        assert_eq!(
            lua_to_json(&lua, &Value::Integer(42)).unwrap(),
            serde_json::json!(42)
        );
    }

    #[test]
    fn test_lua_to_json_table() {
        let lua = Lua::new();

        // Object table
        let obj = lua.create_table().unwrap();
        obj.set("name", "test").unwrap();
        obj.set("value", 123).unwrap();

        let json = lua_to_json(&lua, &Value::Table(obj)).unwrap();
        assert_eq!(json["name"], "test");
        assert_eq!(json["value"], 123);
    }

    #[test]
    fn test_lua_to_json_array() {
        let lua = Lua::new();

        // Array table
        let arr = lua.create_table().unwrap();
        arr.set(1, "a").unwrap();
        arr.set(2, "b").unwrap();
        arr.set(3, "c").unwrap();

        let json = lua_to_json(&lua, &Value::Table(arr)).unwrap();
        assert!(json.is_array());
        assert_eq!(json[0], "a");
        assert_eq!(json[1], "b");
        assert_eq!(json[2], "c");
    }

    #[test]
    fn test_parse_simple_lua_skin() {
        let dir = tempdir().unwrap();

        // Create wrapper file
        let wrapper_path = dir.path().join("test.luaskin");
        fs::write(
            &wrapper_path,
            r#"{"name": "Test Lua Skin", "author": "Test", "main": "skin.lua"}"#,
        )
        .unwrap();

        // Create main Lua file
        // Note: Empty source/image arrays are omitted because Lua empty tables
        // are ambiguous (could be array or object). The serde defaults handle this.
        let main_path = dir.path().join("skin.lua");
        fs::write(
            &main_path,
            r#"
            function main()
                return {
                    name = "Lua Test",
                    type = 0,
                    w = 1920,
                    h = 1080
                }
            end
            "#,
        )
        .unwrap();

        let skin = parse_lua_skin(&wrapper_path, &main_path).unwrap();
        assert_eq!(skin.header.name, "Lua Test");
        assert_eq!(skin.header.skin_type, 0);
        assert_eq!(skin.header.w, 1920);
        // source and image should default to empty vec
        assert!(skin.source.is_empty());
        assert!(skin.image.is_empty());
    }
}
