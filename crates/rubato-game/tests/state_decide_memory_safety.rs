// Phase 60c: Verify MusicDecide can be constructed and dropped without leak.
//
// Previously tested MainControllerRef memory safety. After migration to
// direct config fields, this test validates basic construction/drop safety.

use rubato_game::state::decide::NullPlayerResource;
use rubato_game::state::decide::music_decide::MusicDecide;

/// MusicDecide can be constructed and dropped without leak.
#[test]
fn construction_and_drop_is_safe() {
    let decide = MusicDecide::new(
        rubato_types::config::Config::default(),
        Box::new(NullPlayerResource::new()),
        rubato_game::core::timer_manager::TimerManager::new(),
    );
    drop(decide);
}
