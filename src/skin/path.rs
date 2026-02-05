use std::path::{Path, PathBuf};

fn find_dir_upwards(start: &Path, dir_name: &str) -> Option<PathBuf> {
    let mut current = Some(start);
    while let Some(dir) = current {
        let candidate = dir.join(dir_name);
        if candidate.is_dir() {
            return Some(candidate);
        }
        current = dir.parent();
    }
    None
}

fn find_root_named(dir_name: &str) -> Option<PathBuf> {
    let cwd_candidate = Path::new(dir_name);
    if cwd_candidate.is_dir() {
        return Some(cwd_candidate.to_path_buf());
    }

    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|dir| dir.to_path_buf()))?;
    find_dir_upwards(&exe_dir, dir_name)
}

pub fn find_skin_root() -> Option<PathBuf> {
    find_root_named("skins").or_else(|| find_root_named("skin"))
}

pub fn resolve_skin_path(path: &Path) -> Option<PathBuf> {
    if path.is_absolute() {
        return resolve_candidate(path);
    }

    if let Some(candidate) = resolve_candidate(path) {
        return Some(candidate);
    }

    resolve_with_root(path, "skins").or_else(|| resolve_with_root(path, "skin"))
}

fn resolve_with_root(path: &Path, root_name: &str) -> Option<PathBuf> {
    let root = find_root_named(root_name)?;
    let candidate = match path.strip_prefix(root_name) {
        Ok(relative) => root.join(relative),
        Err(_) => root.join(path),
    };
    resolve_candidate(&candidate)
}

fn resolve_candidate(path: &Path) -> Option<PathBuf> {
    if path.exists() {
        return Some(path.to_path_buf());
    }
    resolve_luaskin_fallback(path)
}

fn resolve_luaskin_fallback(path: &Path) -> Option<PathBuf> {
    let ext = path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("");
    if !ext.eq_ignore_ascii_case("luaskin") {
        return None;
    }

    let stem = path.file_stem()?.to_string_lossy();
    let dir = path.parent()?;

    let main_candidate = dir.join(format!("{}main.lua", stem));
    if main_candidate.exists() {
        return Some(main_candidate);
    }

    let lua_candidate = dir.join(format!("{}.lua", stem));
    lua_candidate.exists().then_some(lua_candidate)
}
