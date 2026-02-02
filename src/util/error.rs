use std::path::Path;

/// User-friendly error messages for common error scenarios.
pub struct UserError;

impl UserError {
    /// Get user-friendly message for BMS file not found.
    pub fn bms_not_found(path: &Path) -> String {
        format!(
            "BMS file not found: {}\n楽曲ファイルが見つかりません: {}",
            path.display(),
            path.display()
        )
    }

    /// Get user-friendly message for BMS parse error.
    pub fn bms_parse_error(path: &Path) -> String {
        format!(
            "Failed to parse BMS file: {}\n楽曲ファイルの読み込みに失敗しました",
            path.display()
        )
    }

    /// Get user-friendly message for audio file not found.
    pub fn audio_not_found(path: &Path) -> String {
        format!(
            "Audio file not found: {}\n音声ファイルが見つかりません",
            path.display()
        )
    }

    /// Get user-friendly message for audio load error.
    pub fn audio_load_error() -> &'static str {
        "Failed to load audio file\n音声ファイルの読み込みに失敗しました"
    }

    /// Get user-friendly message for database error.
    pub fn database_error(operation: &str) -> String {
        format!(
            "Database error during {}\nデータベースエラーが発生しました",
            operation
        )
    }

    /// Get user-friendly message for skin not found.
    pub fn skin_not_found(path: &Path) -> String {
        format!(
            "Skin file not found: {}\nスキンファイルが見つかりません",
            path.display()
        )
    }

    /// Get user-friendly message for skin parse error.
    pub fn skin_parse_error() -> &'static str {
        "Failed to parse skin file\nスキンの読み込みに失敗しました"
    }

    /// Get user-friendly message for replay save error.
    pub fn replay_save_error() -> &'static str {
        "Failed to save replay\nリプレイの保存に失敗しました"
    }

    /// Get user-friendly message for replay load error.
    pub fn replay_load_error() -> &'static str {
        "Failed to load replay\nリプレイの読み込みに失敗しました"
    }

    /// Get user-friendly message for input initialization error.
    pub fn input_init_error() -> &'static str {
        "Failed to initialize input system\n入力システムの初期化に失敗しました"
    }

    /// Get user-friendly message for audio system initialization error.
    pub fn audio_init_error() -> &'static str {
        "Failed to initialize audio system\nオーディオシステムの初期化に失敗しました"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_bms_not_found() {
        let msg = UserError::bms_not_found(Path::new("/path/to/file.bms"));
        assert!(msg.contains("BMS file not found"));
        assert!(msg.contains("楽曲ファイルが見つかりません"));
    }

    #[test]
    fn test_database_error() {
        let msg = UserError::database_error("open");
        assert!(msg.contains("Database error"));
        assert!(msg.contains("データベースエラー"));
    }
}
