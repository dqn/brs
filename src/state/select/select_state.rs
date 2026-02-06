use anyhow::Result;

use super::bar_manager::{BarManager, SortMode};
use super::search;
use crate::database::score_db::ScoreDatabase;
use crate::database::song_db::SongDatabase;
use crate::state::game_state::{GameState, StateTransition};

/// Input actions for the select screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectInput {
    Up,
    Down,
    Decide,
    Back,
    Sort,
    Search,
}

/// State of the select screen.
pub struct SelectState {
    bar_manager: BarManager,
    song_db: SongDatabase,
    score_db: ScoreDatabase,
    bms_roots: Vec<String>,
    /// Pending input actions to process in the next update.
    pending_inputs: Vec<SelectInput>,
    /// Search query string, if searching.
    search_query: Option<String>,
    /// Whether a song was selected and play should begin.
    song_selected: bool,
}

impl SelectState {
    pub fn new(song_db: SongDatabase, score_db: ScoreDatabase, bms_roots: Vec<String>) -> Self {
        Self {
            bar_manager: BarManager::new(),
            song_db,
            score_db,
            bms_roots,
            pending_inputs: Vec::new(),
            search_query: None,
            song_selected: false,
        }
    }

    /// Queue an input to be processed in the next update.
    pub fn push_input(&mut self, input: SelectInput) {
        self.pending_inputs.push(input);
    }

    /// Set a search query. Pass None to clear.
    pub fn set_search_query(&mut self, query: Option<String>) {
        self.search_query = query;
    }

    /// Get a reference to the bar manager.
    pub fn bar_manager(&self) -> &BarManager {
        &self.bar_manager
    }

    /// Get a reference to the song database.
    pub fn song_db(&self) -> &SongDatabase {
        &self.song_db
    }

    /// Get a reference to the score database.
    pub fn score_db(&self) -> &ScoreDatabase {
        &self.score_db
    }

    /// Process pending inputs and update state.
    fn process_inputs(&mut self) -> StateTransition {
        let inputs: Vec<SelectInput> = self.pending_inputs.drain(..).collect();
        for input in inputs {
            match input {
                SelectInput::Up => self.bar_manager.cursor_up(1),
                SelectInput::Down => self.bar_manager.cursor_down(1),
                SelectInput::Decide => {
                    if let Some(bar) = self.bar_manager.selected() {
                        if bar.is_directory() {
                            self.bar_manager.enter_folder(&self.song_db);
                        } else {
                            self.song_selected = true;
                            return StateTransition::Next;
                        }
                    }
                }
                SelectInput::Back => {
                    if self.search_query.is_some() {
                        self.search_query = None;
                        self.bar_manager.load_root(&self.song_db, &self.bms_roots);
                    } else if !self.bar_manager.leave_folder() {
                        return StateTransition::Back;
                    }
                }
                SelectInput::Sort => {
                    let next = match self.bar_manager.sort_mode() {
                        SortMode::Title => SortMode::Artist,
                        SortMode::Artist => SortMode::Level,
                        SortMode::Level => SortMode::Clear,
                        SortMode::Clear => SortMode::AddDate,
                        SortMode::AddDate => SortMode::Title,
                    };
                    self.bar_manager.set_sort_mode(next);
                }
                SelectInput::Search => {
                    if let Some(ref query) = self.search_query {
                        let results = search::search_songs(&self.song_db, query);
                        self.bar_manager.set_bars(results);
                    }
                }
            }
        }
        StateTransition::None
    }
}

impl GameState for SelectState {
    fn create(&mut self) -> Result<()> {
        self.bar_manager.load_root(&self.song_db, &self.bms_roots);
        Ok(())
    }

    fn update(&mut self, _dt_us: i64) -> Result<StateTransition> {
        let transition = self.process_inputs();
        Ok(transition)
    }

    fn dispose(&mut self) {
        self.pending_inputs.clear();
        self.search_query = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::models::SongData;

    fn setup() -> SelectState {
        use crate::state::select::bar_manager::crc32_folder;

        let song_db = SongDatabase::open_in_memory().unwrap();
        let score_db = ScoreDatabase::open_in_memory().unwrap();

        // The parent CRC must match crc32_folder("bms_root").
        let root_crc = crc32_folder("bms_root");

        // Insert test songs.
        for i in 0..5 {
            song_db
                .upsert_song(&SongData {
                    title: format!("Song {}", i),
                    sha256: format!("sha{}", i),
                    md5: format!("md{}", i),
                    path: format!("path/{}.bms", i),
                    parent: root_crc.clone(),
                    ..Default::default()
                })
                .unwrap();
        }

        SelectState::new(song_db, score_db, vec!["bms_root".to_string()])
    }

    #[test]
    fn lifecycle() {
        let mut state = setup();
        state.create().unwrap();
        // Should have loaded songs.
        assert!(!state.bar_manager().bars().is_empty());
    }

    #[test]
    fn navigation_up_down() {
        let mut state = setup();
        state.create().unwrap();

        let initial = state.bar_manager().cursor();
        state.push_input(SelectInput::Down);
        state.update(16_667).unwrap();
        assert_eq!(state.bar_manager().cursor(), initial + 1);

        state.push_input(SelectInput::Up);
        state.update(16_667).unwrap();
        assert_eq!(state.bar_manager().cursor(), initial);
    }

    #[test]
    fn sort_cycles() {
        let mut state = setup();
        state.create().unwrap();

        assert_eq!(state.bar_manager().sort_mode(), SortMode::Title);
        state.push_input(SelectInput::Sort);
        state.update(16_667).unwrap();
        assert_eq!(state.bar_manager().sort_mode(), SortMode::Artist);
    }

    #[test]
    fn select_song_transitions_next() {
        let mut state = setup();
        state.create().unwrap();
        state.push_input(SelectInput::Decide);
        let transition = state.update(16_667).unwrap();
        // Songs are at root, deciding should select.
        assert_eq!(transition, StateTransition::Next);
    }

    #[test]
    fn back_from_root_transitions_back() {
        let mut state = setup();
        state.create().unwrap();
        state.push_input(SelectInput::Back);
        let transition = state.update(16_667).unwrap();
        assert_eq!(transition, StateTransition::Back);
    }

    #[test]
    fn search_filters_songs() {
        let mut state = setup();
        state.create().unwrap();

        state.set_search_query(Some("Song 2".to_string()));
        state.push_input(SelectInput::Search);
        state.update(16_667).unwrap();

        assert_eq!(state.bar_manager().bars().len(), 1);
        assert_eq!(state.bar_manager().bars()[0].title(), "Song 2");
    }
}
