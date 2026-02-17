// Conditional processing for JSON skin loading.
//
// Evaluates option conditions and resolves conditional branches in
// JSON skin data.

use std::collections::HashSet;

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
/// For objects with `{"include": "path"}`, loads the referenced file.
/// (File includes are NOT implemented in this phase — they return null.)
pub fn resolve_conditionals(value: Value, enabled: &HashSet<i32>) -> Value {
    resolve_conditionals_with_context(value, enabled, false)
}

fn resolve_conditionals_with_context(
    value: Value,
    enabled: &HashSet<i32>,
    in_object_field: bool,
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
                            return resolve_conditionals_with_context(val.clone(), enabled, false);
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
                                result.push(resolve_conditionals_with_context(
                                    val.clone(),
                                    enabled,
                                    false,
                                ));
                            }
                            if let Some(Value::Array(vals)) = obj.get("values") {
                                for v in vals {
                                    result.push(resolve_conditionals_with_context(
                                        v.clone(),
                                        enabled,
                                        false,
                                    ));
                                }
                            }
                        }
                        continue;
                    }
                    if obj.contains_key("include") {
                        // File include — deferred to Phase 10 integration
                        continue;
                    }
                }
                result.push(resolve_conditionals_with_context(item, enabled, false));
            }
            Value::Array(result)
        }
        Value::Object(mut map) => {
            // Check if this object itself is a conditional branch
            if map.contains_key("if") && map.contains_key("value") {
                let condition = map.get("if").unwrap_or(&Value::Null);
                if test_option(condition, enabled)
                    && let Some(val) = map.remove("value")
                {
                    return resolve_conditionals_with_context(val, enabled, false);
                }
                return Value::Null;
            }
            // Recurse into object fields
            let resolved: serde_json::Map<String, Value> = map
                .into_iter()
                .map(|(k, v)| (k, resolve_conditionals_with_context(v, enabled, true)))
                .collect();
            Value::Object(resolved)
        }
        other => other,
    }
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
