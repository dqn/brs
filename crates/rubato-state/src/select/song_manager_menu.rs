// SongManagerMenu: last-played-sort state moved to beatoraja-types (Phase 18e-8)
// Thin wrapper preserved for API compatibility

/// Stub for beatoraja.select.SongManagerMenu
pub struct SongManagerMenu;

impl SongManagerMenu {
    pub fn is_last_played_sort_enabled() -> bool {
        rubato_types::last_played_sort::is_enabled()
    }
    pub fn force_disable_last_played_sort() {
        rubato_types::last_played_sort::force_disable();
    }
}
