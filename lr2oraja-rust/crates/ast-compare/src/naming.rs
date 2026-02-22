use heck::ToSnakeCase;

/// Convert a Java CamelCase class name to a Rust snake_case file name.
///
/// Handles special cases like acronyms: `BMSDecoder` → `bms_decoder`,
/// `JSONSkinLoader` → `json_skin_loader`.
pub fn class_to_module(java_class: &str) -> String {
    java_class.to_snake_case()
}

/// Convert a Java camelCase method name to Rust snake_case.
///
/// `getMicroTime` → `get_micro_time`, `toString` → `to_string`.
pub fn method_to_snake(java_method: &str) -> String {
    java_method.to_snake_case()
}

/// Generate candidate Rust method names for a Java getter.
///
/// `getTitle` → `["get_title", "title"]`
pub fn getter_candidates(java_method: &str) -> Vec<String> {
    let snake = method_to_snake(java_method);
    let mut candidates = vec![snake.clone()];
    if let Some(stripped) = snake.strip_prefix("get_") {
        candidates.push(stripped.to_string());
    }
    if let Some(stripped) = snake.strip_prefix("is_") {
        candidates.push(stripped.to_string());
    }
    candidates
}

/// Generate candidate Rust method names for a Java setter.
///
/// `setScore` → `["set_score"]`
pub fn setter_candidates(java_method: &str) -> Vec<String> {
    vec![method_to_snake(java_method)]
}

/// Check if a Java method name is a getter (`getX` or `isX`).
pub fn is_getter(name: &str) -> bool {
    (name.starts_with("get") && name.len() > 3 && name.as_bytes()[3].is_ascii_uppercase())
        || (name.starts_with("is") && name.len() > 2 && name.as_bytes()[2].is_ascii_uppercase())
}

/// Check if a Java method name is a setter (`setX`).
pub fn is_setter(name: &str) -> bool {
    name.starts_with("set") && name.len() > 3 && name.as_bytes()[3].is_ascii_uppercase()
}

/// Known Rust-specific method names that have no Java counterpart.
pub fn is_rust_specific_method(name: &str) -> bool {
    matches!(
        name,
        "fmt" | "clone" | "default" | "eq" | "ne" | "partial_cmp" | "cmp" | "hash"
    )
}

/// Known Rust trait impl names to skip in comparison.
pub fn is_rust_trait_impl(trait_name: &str) -> bool {
    matches!(
        trait_name,
        "Default"
            | "Display"
            | "Debug"
            | "Clone"
            | "PartialEq"
            | "Eq"
            | "PartialOrd"
            | "Ord"
            | "Hash"
            | "From"
            | "Into"
            | "TryFrom"
            | "TryInto"
            | "Serialize"
            | "Deserialize"
            | "Send"
            | "Sync"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_class_to_module() {
        assert_eq!(class_to_module("BMSDecoder"), "bms_decoder");
        assert_eq!(class_to_module("TimeLine"), "time_line");
        assert_eq!(class_to_module("JSONSkinLoader"), "json_skin_loader");
        assert_eq!(class_to_module("PlayerConfig"), "player_config");
        assert_eq!(class_to_module("AutoplayModifier"), "autoplay_modifier");
        assert_eq!(class_to_module("PCM"), "pcm");
        assert_eq!(class_to_module("Config"), "config");
        assert_eq!(class_to_module("GdxSoundDriver"), "gdx_sound_driver");
        assert_eq!(class_to_module("LR2SkinCSVLoader"), "lr2_skin_csv_loader");
    }

    #[test]
    fn test_method_to_snake() {
        assert_eq!(method_to_snake("getMicroTime"), "get_micro_time");
        assert_eq!(method_to_snake("setScore"), "set_score");
        assert_eq!(method_to_snake("toString"), "to_string");
        assert_eq!(method_to_snake("validate"), "validate");
        assert_eq!(method_to_snake("getTotalNotes"), "get_total_notes");
    }

    #[test]
    fn test_getter_candidates() {
        assert_eq!(getter_candidates("getTitle"), vec!["get_title", "title"]);
        assert_eq!(
            getter_candidates("isVisible"),
            vec!["is_visible", "visible"]
        );
        assert_eq!(getter_candidates("validate"), vec!["validate"]);
    }

    #[test]
    fn test_is_getter_setter() {
        assert!(is_getter("getTitle"));
        assert!(is_getter("isVisible"));
        assert!(!is_getter("get"));
        assert!(!is_getter("validate"));

        assert!(is_setter("setScore"));
        assert!(!is_setter("set"));
        assert!(!is_setter("setup"));
    }
}
