// Decide-specific skin state synchronization.
//
// Updates SharedGameState with song metadata for the Decide screen.

use bms_model::BmsModel;
use bms_skin::property_id::{
    NUMBER_MAXBPM, NUMBER_MINBPM, NUMBER_TOTALNOTES2, STRING_ARTIST, STRING_FULLTITLE,
    STRING_GENRE, STRING_SUBARTIST, STRING_SUBTITLE, STRING_TITLE,
};

use crate::game_state::SharedGameState;

/// Synchronize decide-specific state into SharedGameState for skin rendering.
pub fn sync_decide_state(state: &mut SharedGameState, model: &BmsModel) {
    // Song metadata strings
    state.strings.insert(STRING_TITLE, model.title.clone());
    state
        .strings
        .insert(STRING_SUBTITLE, model.subtitle.clone());
    state.strings.insert(
        STRING_FULLTITLE,
        format!("{} {}", model.title, model.subtitle),
    );
    state.strings.insert(STRING_ARTIST, model.artist.clone());
    state
        .strings
        .insert(STRING_SUBARTIST, model.sub_artist.clone());
    state.strings.insert(STRING_GENRE, model.genre.clone());

    // BPM
    state.integers.insert(NUMBER_MINBPM, model.min_bpm() as i32);
    state.integers.insert(NUMBER_MAXBPM, model.max_bpm() as i32);

    // Total notes
    state.integers.insert(
        NUMBER_TOTALNOTES2,
        model
            .build_judge_notes()
            .iter()
            .filter(|n| n.is_playable())
            .count() as i32,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sync_decide_populates_title() {
        let mut state = SharedGameState::default();
        let mut model = BmsModel::default();
        model.title = "Test Song".to_string();
        model.artist = "Test Artist".to_string();

        sync_decide_state(&mut state, &model);

        assert_eq!(state.strings.get(&STRING_TITLE).unwrap(), "Test Song");
        assert_eq!(state.strings.get(&STRING_ARTIST).unwrap(), "Test Artist");
    }

    #[test]
    fn sync_decide_populates_fulltitle() {
        let mut state = SharedGameState::default();
        let mut model = BmsModel::default();
        model.title = "Title".to_string();
        model.subtitle = "Sub".to_string();

        sync_decide_state(&mut state, &model);

        assert_eq!(state.strings.get(&STRING_FULLTITLE).unwrap(), "Title Sub");
    }

    #[test]
    fn sync_decide_populates_bpm() {
        let mut state = SharedGameState::default();
        let mut model = BmsModel::default();
        model.initial_bpm = 150.0;

        sync_decide_state(&mut state, &model);

        assert_eq!(*state.integers.get(&NUMBER_MINBPM).unwrap(), 150);
        assert_eq!(*state.integers.get(&NUMBER_MAXBPM).unwrap(), 150);
    }
}
