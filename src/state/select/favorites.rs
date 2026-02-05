use std::collections::HashSet;
use std::fs;
use std::path::Path;

use anyhow::Result;
use serde::{Deserialize, Serialize};

const FAVORITES_FILE: &str = "favorites.json";

#[derive(Debug, Clone, Default)]
pub struct FavoriteStore {
    items: HashSet<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct FavoriteData {
    sha256: Vec<String>,
}

impl FavoriteStore {
    /// Load favorites from default file.
    /// 既定のファイルからお気に入りを読み込む。
    pub fn load() -> Result<Self> {
        Self::load_from(FAVORITES_FILE)
    }

    /// Load favorites from a path.
    /// 指定パスからお気に入りを読み込む。
    pub fn load_from<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = fs::read_to_string(path)?;
        let data: FavoriteData = serde_json::from_str(&content)?;
        Ok(Self {
            items: data.sha256.into_iter().collect(),
        })
    }

    /// Save favorites to default file.
    /// 既定のファイルへお気に入りを保存する。
    pub fn save(&self) -> Result<()> {
        self.save_to(FAVORITES_FILE)
    }

    /// Save favorites to a path.
    /// 指定パスへお気に入りを保存する。
    pub fn save_to<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let data = FavoriteData {
            sha256: self.items.iter().cloned().collect(),
        };
        let content = serde_json::to_string_pretty(&data)?;
        fs::write(path, content)?;
        Ok(())
    }

    pub fn contains(&self, sha256: &str) -> bool {
        self.items.contains(sha256)
    }

    pub fn items(&self) -> &HashSet<String> {
        &self.items
    }

    pub fn toggle(&mut self, sha256: &str) {
        if self.items.contains(sha256) {
            self.items.remove(sha256);
        } else {
            self.items.insert(sha256.to_string());
        }
    }
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::*;

    #[test]
    fn test_favorite_store_roundtrip() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("favorites.json");
        let mut store = FavoriteStore::default();
        store.toggle("abc");
        store.toggle("def");
        store.save_to(&path).unwrap();

        let loaded = FavoriteStore::load_from(&path).unwrap();
        assert!(loaded.contains("abc"));
        assert!(loaded.contains("def"));
    }
}
