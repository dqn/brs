// Select-specific skin state synchronization.
//
// Updates SharedGameState with song metadata, bar type, and mode flags
// from the current selection in MusicSelect state.

use bms_skin::property_id::{
    NUMBER_MAXBPM, NUMBER_MINBPM, NUMBER_TOTALNOTES2, OPTION_5KEYSONG, OPTION_7KEYSONG,
    OPTION_9KEYSONG, OPTION_10KEYSONG, OPTION_14KEYSONG, OPTION_BGA, OPTION_FOLDERBAR, OPTION_LN,
    OPTION_NO_BGA, OPTION_NO_LN, OPTION_PLAYABLEBAR, OPTION_SONGBAR, RATE_MUSICSELECT_POSITION,
    STRING_ARTIST, STRING_FULLTITLE, STRING_GENRE, STRING_SUBARTIST, STRING_SUBTITLE, STRING_TITLE,
};

use crate::game_state::SharedGameState;
use crate::state::select::bar_manager::{Bar, BarManager};

/// Synchronize select-specific state into SharedGameState for skin rendering.
///
/// Called once per frame during the MusicSelect state.
pub fn sync_select_state(
    state: &mut SharedGameState,
    bar_manager: &BarManager,
    has_ln: bool,
    bga_on: bool,
) {
    // Bar type booleans (clear previous)
    state.booleans.insert(OPTION_SONGBAR, false);
    state.booleans.insert(OPTION_FOLDERBAR, false);
    state.booleans.insert(OPTION_PLAYABLEBAR, false);

    match bar_manager.current() {
        Some(Bar::Song(song_data)) => {
            state.booleans.insert(OPTION_SONGBAR, true);
            state.booleans.insert(OPTION_PLAYABLEBAR, true);

            // Song metadata strings
            state.strings.insert(STRING_TITLE, song_data.title.clone());
            state
                .strings
                .insert(STRING_SUBTITLE, song_data.subtitle.clone());
            state.strings.insert(
                STRING_FULLTITLE,
                format!("{} {}", song_data.title, song_data.subtitle),
            );
            state
                .strings
                .insert(STRING_ARTIST, song_data.artist.clone());
            state
                .strings
                .insert(STRING_SUBARTIST, song_data.subartist.clone());
            state.strings.insert(STRING_GENRE, song_data.genre.clone());

            // BPM
            state.integers.insert(NUMBER_MINBPM, song_data.minbpm);
            state.integers.insert(NUMBER_MAXBPM, song_data.maxbpm);

            // Total notes
            state.integers.insert(NUMBER_TOTALNOTES2, song_data.notes);

            // Mode flags
            let mode_id = song_data.mode;
            sync_mode_flags(state, mode_id);
        }
        Some(Bar::Folder { .. }) => {
            state.booleans.insert(OPTION_FOLDERBAR, true);
            clear_song_metadata(state);
        }
        Some(Bar::Course(_)) => {
            // Course bars are not folders or songs
            clear_song_metadata(state);
        }
        None => {
            clear_song_metadata(state);
        }
    }

    // Select position (fraction of cursor within bar list)
    let total = bar_manager.bar_count();
    let cursor = bar_manager.cursor_pos();
    if total > 0 {
        state
            .floats
            .insert(RATE_MUSICSELECT_POSITION, cursor as f32 / total as f32);
    }

    // LN / BGA feature flags
    state.booleans.insert(OPTION_LN, has_ln);
    state.booleans.insert(OPTION_NO_LN, !has_ln);
    state.booleans.insert(OPTION_BGA, bga_on);
    state.booleans.insert(OPTION_NO_BGA, !bga_on);
}

/// Set mode-specific booleans from song mode ID.
fn sync_mode_flags(state: &mut SharedGameState, mode_id: i32) {
    state.booleans.insert(OPTION_7KEYSONG, false);
    state.booleans.insert(OPTION_5KEYSONG, false);
    state.booleans.insert(OPTION_14KEYSONG, false);
    state.booleans.insert(OPTION_10KEYSONG, false);
    state.booleans.insert(OPTION_9KEYSONG, false);

    // mode_id matches PlayMode::mode_id() values
    match mode_id {
        7 => {
            state.booleans.insert(OPTION_7KEYSONG, true);
        }
        5 => {
            state.booleans.insert(OPTION_5KEYSONG, true);
        }
        14 => {
            state.booleans.insert(OPTION_14KEYSONG, true);
        }
        10 => {
            state.booleans.insert(OPTION_10KEYSONG, true);
        }
        9 => {
            state.booleans.insert(OPTION_9KEYSONG, true);
        }
        _ => {}
    }
}

fn clear_song_metadata(state: &mut SharedGameState) {
    state.strings.insert(STRING_TITLE, String::new());
    state.strings.insert(STRING_SUBTITLE, String::new());
    state.strings.insert(STRING_FULLTITLE, String::new());
    state.strings.insert(STRING_ARTIST, String::new());
    state.strings.insert(STRING_SUBARTIST, String::new());
    state.strings.insert(STRING_GENRE, String::new());
    state.integers.insert(NUMBER_MINBPM, 0);
    state.integers.insert(NUMBER_MAXBPM, 0);
    state.integers.insert(NUMBER_TOTALNOTES2, 0);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sync_mode_flags_7k() {
        let mut state = SharedGameState::default();
        sync_mode_flags(&mut state, 7);
        assert!(*state.booleans.get(&OPTION_7KEYSONG).unwrap());
        assert!(!*state.booleans.get(&OPTION_5KEYSONG).unwrap());
    }

    #[test]
    fn sync_mode_flags_14k() {
        let mut state = SharedGameState::default();
        sync_mode_flags(&mut state, 14);
        assert!(*state.booleans.get(&OPTION_14KEYSONG).unwrap());
        assert!(!*state.booleans.get(&OPTION_7KEYSONG).unwrap());
    }

    #[test]
    fn clear_song_metadata_empties_strings() {
        let mut state = SharedGameState::default();
        state.strings.insert(STRING_TITLE, "test".to_string());
        clear_song_metadata(&mut state);
        assert_eq!(state.strings.get(&STRING_TITLE).unwrap(), "");
    }

    #[test]
    fn sync_select_no_bar_clears_metadata() {
        let mut state = SharedGameState::default();
        let bm = BarManager::new();
        sync_select_state(&mut state, &bm, false, true);
        assert!(!*state.booleans.get(&OPTION_SONGBAR).unwrap());
        assert!(!*state.booleans.get(&OPTION_FOLDERBAR).unwrap());
    }

    #[test]
    fn sync_select_feature_flags() {
        let mut state = SharedGameState::default();
        let bm = BarManager::new();
        sync_select_state(&mut state, &bm, true, false);
        assert!(*state.booleans.get(&OPTION_LN).unwrap());
        assert!(!*state.booleans.get(&OPTION_NO_LN).unwrap());
        assert!(!*state.booleans.get(&OPTION_BGA).unwrap());
        assert!(*state.booleans.get(&OPTION_NO_BGA).unwrap());
    }
}
