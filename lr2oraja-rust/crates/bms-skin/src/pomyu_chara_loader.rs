// Pomyu character skin loader (stub).
//
// Will be implemented when BGA/chara rendering is ported.

use std::path::Path;

/// Pomyu character skin loader.
pub struct PomyuCharaLoader;

impl PomyuCharaLoader {
    /// Checks if the given path is a Pomyu character skin.
    pub fn is_pomyu_chara(_path: &Path) -> bool {
        false // stub
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_pomyu_chara_stub() {
        assert!(!PomyuCharaLoader::is_pomyu_chara(Path::new(
            "/some/path/chara.json"
        )));
    }
}
