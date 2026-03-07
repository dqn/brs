use crate::bms_model::BMSModel;
use crate::note::Note;

pub struct Lane {
    notes: Vec<Note>,
    notebasepos: usize,
    noteseekpos: usize,
    hiddens: Vec<Note>,
    hiddenbasepos: usize,
    hiddenseekpos: usize,
}

impl Lane {
    pub fn new(model: &BMSModel, lane: i32) -> Self {
        let mut notes = Vec::new();
        let mut hiddens = Vec::new();
        for tl in &model.timelines {
            if tl.exist_note_at(lane)
                && let Some(note) = tl.note(lane)
            {
                notes.push(note.clone());
            }
            if let Some(hnote) = tl.hidden_note(lane) {
                hiddens.push(hnote.clone());
            }
        }
        Lane {
            notes,
            notebasepos: 0,
            noteseekpos: 0,
            hiddens,
            hiddenbasepos: 0,
            hiddenseekpos: 0,
        }
    }

    pub fn notes(&self) -> &[Note] {
        &self.notes
    }

    pub fn hiddens(&self) -> &[Note] {
        &self.hiddens
    }

    pub fn note(&mut self) -> Option<&Note> {
        if self.noteseekpos < self.notes.len() {
            let pos = self.noteseekpos;
            self.noteseekpos += 1;
            Some(&self.notes[pos])
        } else {
            None
        }
    }

    pub fn hidden(&mut self) -> Option<&Note> {
        if self.hiddenseekpos < self.hiddens.len() {
            let pos = self.hiddenseekpos;
            self.hiddenseekpos += 1;
            Some(&self.hiddens[pos])
        } else {
            None
        }
    }

    pub fn reset(&mut self) {
        self.noteseekpos = self.notebasepos;
        self.hiddenseekpos = self.hiddenbasepos;
    }

    pub fn mark(&mut self, time: i32) {
        while self.notebasepos < self.notes.len() - 1
            && self.notes[self.notebasepos + 1].time() < time
        {
            self.notebasepos += 1;
        }
        while self.notebasepos > 0 && self.notes[self.notebasepos].time() > time {
            self.notebasepos -= 1;
        }
        self.noteseekpos = self.notebasepos;
        while self.hiddenbasepos < self.hiddens.len() - 1
            && self.hiddens[self.hiddenbasepos + 1].time() < time
        {
            self.hiddenbasepos += 1;
        }
        while self.hiddenbasepos > 0 && self.hiddens[self.hiddenbasepos].time() > time {
            self.hiddenbasepos -= 1;
        }
        self.hiddenseekpos = self.hiddenbasepos;
    }
}
