use crate::skin::renderer::SkinStateSnapshot;
use crate::skin::skin_property::*;
use crate::state::select::bar::Bar;
use crate::state::select::select_state::SelectState;

pub struct SelectSkinAccessor;

impl SelectSkinAccessor {
    pub fn snapshot(state: &SelectState, elapsed_us: i64) -> SkinStateSnapshot {
        let mut snap = SkinStateSnapshot {
            time_ms: elapsed_us / 1000,
            ..Default::default()
        };

        // Timers
        snap.timers.insert(TIMER_STARTINPUT, 0); // always on from start

        // Selected bar info
        if let Some(bar) = state.bar_manager().selected() {
            // Options
            snap.options.insert(OPTION_FOLDERBAR, bar.is_directory());
            snap.options.insert(OPTION_SONGBAR, !bar.is_directory());

            // Strings
            snap.strings.insert(STRING_TITLE, bar.title().to_string());

            if let Bar::Song(song_bar) = bar {
                snap.strings
                    .insert(STRING_ARTIST, song_bar.song.artist.clone());
                snap.strings
                    .insert(STRING_GENRE, song_bar.song.genre.clone());
                snap.strings
                    .insert(STRING_SUBTITLE, song_bar.song.subtitle.clone());
                snap.strings
                    .insert(STRING_SUBARTIST, song_bar.song.subartist.clone());
                snap.numbers.insert(NUMBER_PLAYLEVEL, song_bar.song.level);
                snap.numbers.insert(NUMBER_MAXBPM, song_bar.song.maxbpm);
                snap.numbers.insert(NUMBER_MINBPM, song_bar.song.minbpm);
            }
        }

        // Scroll position
        let bars = state.bar_manager().bars();
        let cursor = state.bar_manager().cursor();
        if !bars.is_empty() {
            snap.floats
                .insert(RATE_MUSICSELECT_POSITION, cursor as f32 / bars.len() as f32);
        }

        snap
    }
}
