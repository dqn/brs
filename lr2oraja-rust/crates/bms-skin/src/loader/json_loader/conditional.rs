// Conditional processing for JSON skin loading.
//
// Evaluates option conditions and resolves conditional branches in
// JSON skin data. Supports file includes via `{"include": "path"}`.

use std::collections::HashSet;
use std::path::Path;

use serde_json::Value;

/// Tests whether an option condition is satisfied.
///
/// Condition format (matching Java's JsonSkinSerializer.testOption):
/// - `901` → option 901 is enabled
/// - `-901` → option 901 is NOT enabled
/// - `[901, 911]` → 901 AND 911 enabled
/// - `[[901, 902], 911]` → (901 OR 902) AND 911
pub fn test_option(condition: &Value, enabled: &HashSet<i32>) -> bool {
    match condition {
        Value::Null => true,
        Value::Number(n) => {
            let op = n.as_i64().unwrap_or(0) as i32;
            test_option_number(op, enabled)
        }
        Value::Array(arr) => {
            for item in arr {
                let ok = match item {
                    Value::Number(n) => {
                        let op = n.as_i64().unwrap_or(0) as i32;
                        test_option_number(op, enabled)
                    }
                    Value::Array(sub) => {
                        // OR group: at least one must be enabled
                        sub.iter().any(|v| {
                            if let Value::Number(n) = v {
                                let op = n.as_i64().unwrap_or(0) as i32;
                                test_option_number(op, enabled)
                            } else {
                                false
                            }
                        })
                    }
                    _ => false,
                };
                if !ok {
                    return false;
                }
            }
            true
        }
        _ => false,
    }
}

fn test_option_number(op: i32, enabled: &HashSet<i32>) -> bool {
    if op >= 0 {
        enabled.contains(&op)
    } else {
        !enabled.contains(&(-op))
    }
}

/// Pre-processes a JSON Value to resolve conditional branches.
///
/// For array elements with `{"if": condition, "value": obj}` or
/// `{"if": condition, "values": [objs]}`, evaluates the condition
/// and includes/excludes the items accordingly.
///
/// For objects with `{"include": "path"}`, loads the referenced file
/// relative to the given `base_path`, recursively resolving conditionals
/// in the included content. If the file does not exist or fails to parse,
/// the include is silently skipped (matching Java behavior).
pub fn resolve_conditionals(value: Value, enabled: &HashSet<i32>) -> Value {
    resolve_conditionals_impl(value, enabled, false, None)
}

/// Same as [`resolve_conditionals`] but with a base path for resolving
/// `{"include": "path"}` directives.
pub fn resolve_conditionals_with_base(
    value: Value,
    enabled: &HashSet<i32>,
    base_path: &Path,
) -> Value {
    resolve_conditionals_impl(value, enabled, false, Some(base_path))
}

fn resolve_conditionals_impl(
    value: Value,
    enabled: &HashSet<i32>,
    in_object_field: bool,
    base_path: Option<&Path>,
) -> Value {
    match value {
        Value::Array(arr) => {
            // ObjectSerializer behavior in Java:
            // when an object field is encoded as a conditional branch array
            // (`[{if, value}, ...]`), only the first matched branch is used.
            if in_object_field && is_object_conditional_branch_array(&arr) {
                for item in &arr {
                    if let Value::Object(obj) = item {
                        let condition = obj.get("if").unwrap_or(&Value::Null);
                        if test_option(condition, enabled)
                            && let Some(val) = obj.get("value")
                        {
                            return resolve_conditionals_impl(
                                val.clone(),
                                enabled,
                                false,
                                base_path,
                            );
                        }
                    }
                }
                return Value::Null;
            }

            let mut result = Vec::new();
            for item in arr {
                if let Value::Object(ref obj) = item {
                    if obj.contains_key("if")
                        && (obj.contains_key("value") || obj.contains_key("values"))
                    {
                        // Conditional branch
                        let condition = obj.get("if").unwrap_or(&Value::Null);
                        if test_option(condition, enabled) {
                            if let Some(val) = obj.get("value") {
                                result.push(resolve_conditionals_impl(
                                    val.clone(),
                                    enabled,
                                    false,
                                    base_path,
                                ));
                            }
                            if let Some(Value::Array(vals)) = obj.get("values") {
                                for v in vals {
                                    result.push(resolve_conditionals_impl(
                                        v.clone(),
                                        enabled,
                                        false,
                                        base_path,
                                    ));
                                }
                            }
                        }
                        continue;
                    }
                    if obj.contains_key("include") {
                        // File include: load and flatten into the array
                        if let Some(included) = load_include(obj, enabled, base_path) {
                            match included {
                                Value::Array(items) => result.extend(items),
                                other => result.push(other),
                            }
                        }
                        continue;
                    }
                }
                result.push(resolve_conditionals_impl(item, enabled, false, base_path));
            }
            Value::Array(result)
        }
        Value::Object(mut map) => {
            // Check if this object itself is an include
            if map.contains_key("include") {
                if let Some(included) = load_include(&map, enabled, base_path) {
                    return included;
                }
                return Value::Null;
            }
            // Check if this object itself is a conditional branch
            if map.contains_key("if") && map.contains_key("value") {
                let condition = map.get("if").unwrap_or(&Value::Null);
                if test_option(condition, enabled)
                    && let Some(val) = map.remove("value")
                {
                    return resolve_conditionals_impl(val, enabled, false, base_path);
                }
                return Value::Null;
            }
            // Recurse into object fields
            let resolved: serde_json::Map<String, Value> = map
                .into_iter()
                .map(|(k, v)| (k, resolve_conditionals_impl(v, enabled, true, base_path)))
                .collect();
            Value::Object(resolved)
        }
        other => other,
    }
}

/// Loads and resolves an include directive from a JSON object containing
/// an `"include"` key. Returns `None` if the file doesn't exist or fails
/// to parse.
fn load_include(
    obj: &serde_json::Map<String, Value>,
    enabled: &HashSet<i32>,
    base_path: Option<&Path>,
) -> Option<Value> {
    let include_path = obj.get("include")?.as_str()?;
    let base = base_path?;

    let parent = base.parent().unwrap_or(base);
    let full_path = parent.join(include_path);

    let content = std::fs::read_to_string(&full_path).ok()?;
    // Pre-process the included JSON (handle lenient JSON) using the same
    // preprocessor used by the main loader.
    let preprocessed = crate::loader::json_loader::preprocess_json(&content);
    let parsed: Value = serde_json::from_str(&preprocessed).ok()?;

    // Recursively resolve conditionals in the included file, using the
    // included file's path as the new base for nested includes.
    Some(resolve_conditionals_impl(
        parsed,
        enabled,
        false,
        Some(&full_path),
    ))
}

fn is_object_conditional_branch_array(arr: &[Value]) -> bool {
    !arr.is_empty()
        && arr.iter().all(|item| {
            let Value::Object(obj) = item else {
                return false;
            };
            obj.contains_key("if") && obj.contains_key("value") && !obj.contains_key("values")
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn include_loads_file() {
        let dir = tempfile::tempdir().unwrap();
        let sub_path = dir.path().join("sub.json");
        let mut f = std::fs::File::create(&sub_path).unwrap();
        write!(f, r#"{{"name": "included"}}"#).unwrap();

        let main_path = dir.path().join("main.json");
        let enabled = HashSet::new();

        let json = serde_json::json!([
            {"include": "sub.json"},
            {"id": "normal"}
        ]);

        let resolved = resolve_conditionals_with_base(json, &enabled, &main_path);
        let arr = resolved.as_array().unwrap();
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0]["name"], "included");
        assert_eq!(arr[1]["id"], "normal");
    }

    #[test]
    fn include_flattens_array() {
        let dir = tempfile::tempdir().unwrap();
        let sub_path = dir.path().join("items.json");
        let mut f = std::fs::File::create(&sub_path).unwrap();
        write!(f, r#"[{{"a": 1}}, {{"a": 2}}]"#).unwrap();

        let main_path = dir.path().join("main.json");
        let enabled = HashSet::new();

        let json = serde_json::json!([
            {"include": "items.json"},
            {"a": 3}
        ]);

        let resolved = resolve_conditionals_with_base(json, &enabled, &main_path);
        let arr = resolved.as_array().unwrap();
        assert_eq!(arr.len(), 3);
        assert_eq!(arr[0]["a"], 1);
        assert_eq!(arr[1]["a"], 2);
        assert_eq!(arr[2]["a"], 3);
    }

    #[test]
    fn include_missing_file_skipped() {
        let dir = tempfile::tempdir().unwrap();
        let main_path = dir.path().join("main.json");
        let enabled = HashSet::new();

        let json = serde_json::json!([
            {"include": "nonexistent.json"},
            {"id": "kept"}
        ]);

        let resolved = resolve_conditionals_with_base(json, &enabled, &main_path);
        let arr = resolved.as_array().unwrap();
        assert_eq!(arr.len(), 1);
        assert_eq!(arr[0]["id"], "kept");
    }

    #[test]
    fn include_without_base_path_skipped() {
        let enabled = HashSet::new();

        let json = serde_json::json!([
            {"include": "sub.json"},
            {"id": "kept"}
        ]);

        // No base_path — include is silently skipped
        let resolved = resolve_conditionals(json, &enabled);
        let arr = resolved.as_array().unwrap();
        assert_eq!(arr.len(), 1);
        assert_eq!(arr[0]["id"], "kept");
    }

    #[test]
    fn include_with_conditionals_in_subfile() {
        let dir = tempfile::tempdir().unwrap();
        let sub_path = dir.path().join("sub.json");
        let mut f = std::fs::File::create(&sub_path).unwrap();
        write!(
            f,
            r#"[{{"if": 901, "value": {{"id": "yes"}}}}, {{"id": "always"}}]"#
        )
        .unwrap();

        let main_path = dir.path().join("main.json");
        let mut enabled = HashSet::new();
        enabled.insert(901);

        let json = serde_json::json!([
            {"include": "sub.json"}
        ]);

        let resolved = resolve_conditionals_with_base(json, &enabled, &main_path);
        let arr = resolved.as_array().unwrap();
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0]["id"], "yes");
        assert_eq!(arr[1]["id"], "always");
    }

    #[test]
    fn include_in_object_field() {
        let dir = tempfile::tempdir().unwrap();
        let sub_path = dir.path().join("sub.json");
        let mut f = std::fs::File::create(&sub_path).unwrap();
        write!(f, r#"{{"x": 10, "y": 20}}"#).unwrap();

        let main_path = dir.path().join("main.json");
        let enabled = HashSet::new();

        let json = serde_json::json!({
            "position": {"include": "sub.json"}
        });

        let resolved = resolve_conditionals_with_base(json, &enabled, &main_path);
        assert_eq!(resolved["position"]["x"], 10);
        assert_eq!(resolved["position"]["y"], 20);
    }

    #[test]
    fn nested_include() {
        let dir = tempfile::tempdir().unwrap();

        // Create a nested include chain: main -> a.json -> b.json
        let b_path = dir.path().join("b.json");
        let mut f = std::fs::File::create(&b_path).unwrap();
        write!(f, r#"{{"deep": true}}"#).unwrap();

        let a_path = dir.path().join("a.json");
        let mut f = std::fs::File::create(&a_path).unwrap();
        write!(f, r#"[{{"include": "b.json"}}, {{"shallow": true}}]"#).unwrap();

        let main_path = dir.path().join("main.json");
        let enabled = HashSet::new();

        let json = serde_json::json!([
            {"include": "a.json"}
        ]);

        let resolved = resolve_conditionals_with_base(json, &enabled, &main_path);
        let arr = resolved.as_array().unwrap();
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0]["deep"], true);
        assert_eq!(arr[1]["shallow"], true);
    }
}
