use crate::skin::renderer::SkinStateSnapshot;
use crate::skin::skin_property::*;
use crate::state::decide::decide_state::{DecidePhase, DecideState};
use crate::state::song_metadata::SongMetadata;

pub struct DecideSkinAccessor;

impl DecideSkinAccessor {
    pub fn snapshot(
        state: &DecideState,
        elapsed_us: i64,
        metadata: &SongMetadata,
    ) -> SkinStateSnapshot {
        let mut snap = SkinStateSnapshot {
            time_ms: elapsed_us / 1000,
            ..Default::default()
        };

        // Timers
        snap.timers.insert(TIMER_STARTINPUT, 0);
        if state.phase() == DecidePhase::FadeOut {
            snap.timers.insert(TIMER_FADEOUT, elapsed_us);
        }

        // Options
        snap.options
            .insert(OPTION_NOW_LOADING, state.phase() == DecidePhase::Loading);
        snap.options
            .insert(OPTION_LOADED, state.phase() != DecidePhase::Loading);

        // Load progress
        let progress = if state.phase() == DecidePhase::Loading {
            0.5
        } else {
            1.0
        };
        snap.floats.insert(BARGRAPH_LOAD_PROGRESS, progress);
        snap.floats.insert(RATE_LOAD_PROGRESS, progress);

        // Song metadata
        snap.strings.insert(STRING_TITLE, metadata.title.clone());
        snap.strings
            .insert(STRING_SUBTITLE, metadata.subtitle.clone());
        snap.strings.insert(STRING_ARTIST, metadata.artist.clone());
        snap.strings
            .insert(STRING_SUBARTIST, metadata.subartist.clone());
        snap.strings.insert(STRING_GENRE, metadata.genre.clone());
        snap.numbers.insert(NUMBER_PLAYLEVEL, metadata.level);
        snap.numbers.insert(NUMBER_MAXBPM, metadata.max_bpm);
        snap.numbers.insert(NUMBER_MINBPM, metadata.min_bpm);

        snap
    }
}
