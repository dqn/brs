use crate::layer::Layer;
use crate::note::{Note, TYPE_CHARGENOTE, TYPE_HELLCHARGENOTE, TYPE_UNDEFINED};

pub struct TimeLine {
    time: i64,
    section: f64,
    notes: Vec<Option<Note>>,
    hiddennotes: Vec<Option<Note>>,
    bgnotes: Vec<Note>,
    section_line: bool,
    bpm: f64,
    stop: i64,
    scroll: f64,
    bga: i32,
    layer: i32,
    eventlayer: Vec<Layer>,
}

impl TimeLine {
    pub fn new(section: f64, time: i64, notesize: i32) -> Self {
        let notesize = notesize as usize;
        let mut notes = Vec::with_capacity(notesize);
        let mut hiddennotes = Vec::with_capacity(notesize);
        for _ in 0..notesize {
            notes.push(None);
            hiddennotes.push(None);
        }
        TimeLine {
            section,
            time,
            notes,
            hiddennotes,
            bgnotes: Vec::new(),
            section_line: false,
            bpm: 0.0,
            stop: 0,
            scroll: 1.0,
            bga: -1,
            layer: -1,
            eventlayer: Vec::new(),
        }
    }

    pub fn get_time(&self) -> i32 {
        (self.time / 1000) as i32
    }

    pub fn get_milli_time(&self) -> i64 {
        self.time / 1000
    }

    pub fn get_micro_time(&self) -> i64 {
        self.time
    }

    pub fn set_micro_time(&mut self, time: i64) {
        self.time = time;
        for note in self.notes.iter_mut().flatten() {
            note.set_micro_time(time);
        }
        for note in self.hiddennotes.iter_mut().flatten() {
            note.set_micro_time(time);
        }
        for n in &mut self.bgnotes {
            n.set_micro_time(time);
        }
    }

    pub fn get_lane_count(&self) -> i32 {
        self.notes.len() as i32
    }

    pub fn set_lane_count(&mut self, lanes: i32) {
        let lanes = lanes as usize;
        if self.notes.len() != lanes {
            let mut newnotes: Vec<Option<Note>> = Vec::with_capacity(lanes);
            let mut newhiddennotes: Vec<Option<Note>> = Vec::with_capacity(lanes);
            for i in 0..lanes {
                if i < self.notes.len() {
                    newnotes.push(self.notes[i].take());
                    newhiddennotes.push(self.hiddennotes[i].take());
                } else {
                    newnotes.push(None);
                    newhiddennotes.push(None);
                }
            }
            self.notes = newnotes;
            self.hiddennotes = newhiddennotes;
        }
    }

    pub fn get_total_notes(&self) -> i32 {
        self.get_total_notes_with_lntype(super::bms_model::LNTYPE_LONGNOTE)
    }

    pub fn get_total_notes_with_lntype(&self, lntype: i32) -> i32 {
        let mut count = 0;
        for note in self.notes.iter().flatten() {
            match note {
                Note::Long { note_type, end, .. } => {
                    if *note_type == TYPE_CHARGENOTE
                        || *note_type == TYPE_HELLCHARGENOTE
                        || (*note_type == TYPE_UNDEFINED
                            && lntype != super::bms_model::LNTYPE_LONGNOTE)
                        || !end
                    {
                        count += 1;
                    }
                }
                Note::Normal(_) => {
                    count += 1;
                }
                Note::Mine { .. } => {}
            }
        }
        count
    }

    pub fn exist_note(&self) -> bool {
        for n in &self.notes {
            if n.is_some() {
                return true;
            }
        }
        false
    }

    pub fn exist_note_at(&self, lane: i32) -> bool {
        self.notes[lane as usize].is_some()
    }

    pub fn get_note(&self, lane: i32) -> Option<&Note> {
        self.notes[lane as usize].as_ref()
    }

    pub fn get_note_mut(&mut self, lane: i32) -> Option<&mut Note> {
        self.notes[lane as usize].as_mut()
    }

    pub fn set_note(&mut self, lane: i32, note: Option<Note>) {
        let lane = lane as usize;
        if let Some(mut n) = note {
            n.set_section(self.section);
            n.set_micro_time(self.time);
            self.notes[lane] = Some(n);
        } else {
            self.notes[lane] = None;
        }
    }

    pub fn set_hidden_note(&mut self, lane: i32, note: Option<Note>) {
        let lane = lane as usize;
        if let Some(mut n) = note {
            n.set_section(self.section);
            n.set_micro_time(self.time);
            self.hiddennotes[lane] = Some(n);
        } else {
            self.hiddennotes[lane] = None;
        }
    }

    pub fn exist_hidden_note(&self) -> bool {
        for n in &self.hiddennotes {
            if n.is_some() {
                return true;
            }
        }
        false
    }

    pub fn get_hidden_note(&self, lane: i32) -> Option<&Note> {
        self.hiddennotes[lane as usize].as_ref()
    }

    pub fn add_back_ground_note(&mut self, note: Note) {
        let mut n = note;
        n.set_section(self.section);
        n.set_micro_time(self.time);
        self.bgnotes.push(n);
    }

    pub fn remove_back_ground_note(&mut self, index: usize) {
        if index < self.bgnotes.len() {
            self.bgnotes.remove(index);
        }
    }

    pub fn get_back_ground_notes(&self) -> &[Note] {
        &self.bgnotes
    }

    pub fn set_bpm(&mut self, bpm: f64) {
        self.bpm = bpm;
    }

    pub fn get_bpm(&self) -> f64 {
        self.bpm
    }

    pub fn set_section_line(&mut self, section: bool) {
        self.section_line = section;
    }

    pub fn get_section_line(&self) -> bool {
        self.section_line
    }

    pub fn get_bga(&self) -> i32 {
        self.bga
    }

    pub fn set_bga(&mut self, bga: i32) {
        self.bga = bga;
    }

    pub fn get_layer(&self) -> i32 {
        self.layer
    }

    pub fn set_layer(&mut self, layer: i32) {
        self.layer = layer;
    }

    pub fn get_eventlayer(&self) -> &[Layer] {
        &self.eventlayer
    }

    pub fn set_eventlayer(&mut self, eventlayer: Vec<Layer>) {
        self.eventlayer = eventlayer;
    }

    pub fn get_section(&self) -> f64 {
        self.section
    }

    pub fn set_section(&mut self, section: f64) {
        for note in self.notes.iter_mut().flatten() {
            note.set_section(section);
        }
        for note in self.hiddennotes.iter_mut().flatten() {
            note.set_section(section);
        }
        for n in &mut self.bgnotes {
            n.set_section(section);
        }
        self.section = section;
    }

    pub fn get_stop(&self) -> i32 {
        (self.stop / 1000) as i32
    }

    pub fn get_milli_stop(&self) -> i64 {
        self.stop / 1000
    }

    pub fn get_micro_stop(&self) -> i64 {
        self.stop
    }

    pub fn set_stop(&mut self, stop: i64) {
        self.stop = stop;
    }

    pub fn get_scroll(&self) -> f64 {
        self.scroll
    }

    pub fn set_scroll(&mut self, scroll: f64) {
        self.scroll = scroll;
    }

    pub fn take_note(&mut self, lane: i32) -> Option<Note> {
        self.notes[lane as usize].take()
    }
}
