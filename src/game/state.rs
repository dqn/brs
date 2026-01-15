use super::JudgeResult;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoteState {
    Pending,
    Judged(JudgeResult),
    Missed,
}

impl NoteState {
    pub fn is_pending(&self) -> bool {
        matches!(self, Self::Pending)
    }
}

pub struct GamePlayState {
    pub note_states: Vec<NoteState>,
}

impl GamePlayState {
    pub fn new(note_count: usize) -> Self {
        Self {
            note_states: vec![NoteState::Pending; note_count],
        }
    }

    pub fn reset(&mut self) {
        for state in &mut self.note_states {
            *state = NoteState::Pending;
        }
    }

    pub fn set_judged(&mut self, index: usize, result: JudgeResult) {
        if index < self.note_states.len() {
            self.note_states[index] = NoteState::Judged(result);
        }
    }

    pub fn set_missed(&mut self, index: usize) {
        if index < self.note_states.len() {
            self.note_states[index] = NoteState::Missed;
        }
    }

    pub fn get_state(&self, index: usize) -> Option<NoteState> {
        self.note_states.get(index).copied()
    }

    pub fn all_notes_processed(&self, total: usize) -> bool {
        self.note_states.iter().take(total).all(|s| !s.is_pending())
    }
}
