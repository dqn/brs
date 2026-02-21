use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NoteData {
    pub section: f64,
    pub time: i64,
    pub wav: i32,
    pub start: i64,
    pub duration: i64,
    pub state: i32,
    pub playtime: i64,
    pub layerednotes: Vec<Note>,
}

impl Default for NoteData {
    fn default() -> Self {
        Self::new()
    }
}

impl NoteData {
    pub fn new() -> Self {
        NoteData {
            section: 0.0,
            time: 0,
            wav: 0,
            start: 0,
            duration: 0,
            state: 0,
            playtime: 0,
            layerednotes: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Note {
    Normal(NoteData),
    Long {
        data: NoteData,
        end: bool,
        pair: Option<usize>,
        note_type: i32,
    },
    Mine {
        data: NoteData,
        damage: f64,
    },
}

pub const TYPE_UNDEFINED: i32 = 0;
pub const TYPE_LONGNOTE: i32 = 1;
pub const TYPE_CHARGENOTE: i32 = 2;
pub const TYPE_HELLCHARGENOTE: i32 = 3;

impl Note {
    pub fn new_normal(wav: i32) -> Self {
        let mut data = NoteData::new();
        data.wav = wav;
        Note::Normal(data)
    }

    pub fn new_normal_with_start_duration(wav: i32, start: i64, duration: i64) -> Self {
        let mut data = NoteData::new();
        data.wav = wav;
        data.start = start;
        data.duration = duration;
        Note::Normal(data)
    }

    pub fn new_long(wav: i32) -> Self {
        let mut data = NoteData::new();
        data.wav = wav;
        Note::Long {
            data,
            end: false,
            pair: None,
            note_type: TYPE_UNDEFINED,
        }
    }

    pub fn new_long_with_start_duration(wav: i32, starttime: i64, duration: i64) -> Self {
        let mut data = NoteData::new();
        data.wav = wav;
        data.start = starttime;
        data.duration = duration;
        Note::Long {
            data,
            end: false,
            pair: None,
            note_type: TYPE_UNDEFINED,
        }
    }

    pub fn new_mine(wav: i32, damage: f64) -> Self {
        let mut data = NoteData::new();
        data.wav = wav;
        Note::Mine { data, damage }
    }

    pub fn data(&self) -> &NoteData {
        match self {
            Note::Normal(data) => data,
            Note::Long { data, .. } => data,
            Note::Mine { data, .. } => data,
        }
    }

    pub fn data_mut(&mut self) -> &mut NoteData {
        match self {
            Note::Normal(data) => data,
            Note::Long { data, .. } => data,
            Note::Mine { data, .. } => data,
        }
    }

    pub fn get_wav(&self) -> i32 {
        self.data().wav
    }

    pub fn set_wav(&mut self, wav: i32) {
        self.data_mut().wav = wav;
    }

    pub fn get_state(&self) -> i32 {
        self.data().state
    }

    pub fn set_state(&mut self, state: i32) {
        self.data_mut().state = state;
    }

    pub fn get_milli_starttime(&self) -> i64 {
        self.data().start / 1000
    }

    pub fn get_micro_starttime(&self) -> i64 {
        self.data().start
    }

    pub fn set_micro_starttime(&mut self, start: i64) {
        self.data_mut().start = start;
    }

    pub fn get_milli_duration(&self) -> i64 {
        self.data().duration / 1000
    }

    pub fn get_micro_duration(&self) -> i64 {
        self.data().duration
    }

    pub fn set_micro_duration(&mut self, duration: i64) {
        self.data_mut().duration = duration;
    }

    pub fn get_play_time(&self) -> i32 {
        (self.data().playtime / 1000) as i32
    }

    pub fn get_milli_play_time(&self) -> i64 {
        self.data().playtime / 1000
    }

    pub fn get_micro_play_time(&self) -> i64 {
        self.data().playtime
    }

    pub fn set_play_time(&mut self, playtime: i32) {
        self.data_mut().playtime = (playtime as i64) * 1000;
    }

    pub fn set_micro_play_time(&mut self, playtime: i64) {
        self.data_mut().playtime = playtime;
    }

    pub fn get_section(&self) -> f64 {
        self.data().section
    }

    pub fn set_section(&mut self, section: f64) {
        self.data_mut().section = section;
    }

    pub fn get_time(&self) -> i32 {
        (self.data().time / 1000) as i32
    }

    pub fn get_milli_time(&self) -> i64 {
        self.data().time / 1000
    }

    pub fn get_micro_time(&self) -> i64 {
        self.data().time
    }

    pub fn set_micro_time(&mut self, time: i64) {
        self.data_mut().time = time;
    }

    pub fn add_layered_note(&mut self, mut n: Note) {
        let section = self.data().section;
        let time = self.data().time;
        n.set_section(section);
        n.set_micro_time(time);
        self.data_mut().layerednotes.push(n);
    }

    pub fn get_layered_notes(&self) -> &[Note] {
        &self.data().layerednotes
    }

    pub fn is_normal(&self) -> bool {
        matches!(self, Note::Normal(_))
    }

    pub fn is_long(&self) -> bool {
        matches!(self, Note::Long { .. })
    }

    pub fn is_mine(&self) -> bool {
        matches!(self, Note::Mine { .. })
    }

    pub fn get_long_note_type(&self) -> i32 {
        match self {
            Note::Long { note_type, .. } => *note_type,
            _ => TYPE_UNDEFINED,
        }
    }

    pub fn set_long_note_type(&mut self, t: i32) {
        if let Note::Long { note_type, .. } = self {
            *note_type = t;
        }
    }

    pub fn is_end(&self) -> bool {
        match self {
            Note::Long { end, .. } => *end,
            _ => false,
        }
    }

    pub fn set_end(&mut self, e: bool) {
        if let Note::Long { end, .. } = self {
            *end = e;
        }
    }

    pub fn get_pair(&self) -> Option<usize> {
        match self {
            Note::Long { pair, .. } => *pair,
            _ => None,
        }
    }

    pub fn set_pair_index(&mut self, idx: Option<usize>) {
        if let Note::Long { pair, .. } = self {
            *pair = idx;
        }
    }

    pub fn get_damage(&self) -> f64 {
        match self {
            Note::Mine { damage, .. } => *damage,
            _ => 0.0,
        }
    }

    pub fn set_damage(&mut self, d: f64) {
        if let Note::Mine { damage, .. } = self {
            *damage = d;
        }
    }
}
