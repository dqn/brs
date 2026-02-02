use crate::model::note::Note;

/// A timeline entry representing a point in the BMS chart.
#[derive(Debug, Clone)]
pub struct Timeline {
    pub time_ms: f64,
    pub measure: u32,
    pub position_in_measure: f64,
    pub bpm: f64,
    pub notes: Vec<Note>,
    pub is_measure_line: bool,
}

impl Timeline {
    /// Create a new timeline entry.
    pub fn new(time_ms: f64, measure: u32, position_in_measure: f64, bpm: f64) -> Self {
        Self {
            time_ms,
            measure,
            position_in_measure,
            bpm,
            notes: Vec::new(),
            is_measure_line: false,
        }
    }

    /// Create a measure line entry.
    pub fn measure_line(time_ms: f64, measure: u32, bpm: f64) -> Self {
        Self {
            time_ms,
            measure,
            position_in_measure: 0.0,
            bpm,
            notes: Vec::new(),
            is_measure_line: true,
        }
    }

    /// Add a note to this timeline.
    pub fn add_note(&mut self, note: Note) {
        self.notes.push(note);
    }
}

/// Collection of all timelines in a BMS chart.
#[derive(Debug, Clone, Default)]
pub struct Timelines {
    entries: Vec<Timeline>,
}

impl Timelines {
    /// Create a new empty timelines collection.
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Add a timeline entry.
    pub fn push(&mut self, timeline: Timeline) {
        self.entries.push(timeline);
    }

    /// Sort timelines by time.
    pub fn sort_by_time(&mut self) {
        self.entries
            .sort_by(|a, b| a.time_ms.partial_cmp(&b.time_ms).unwrap());
    }

    /// Get all timelines.
    pub fn entries(&self) -> &[Timeline] {
        &self.entries
    }

    /// Get all timelines mutably.
    pub fn entries_mut(&mut self) -> &mut [Timeline] {
        &mut self.entries
    }

    /// Get all notes from all timelines.
    pub fn all_notes(&self) -> impl Iterator<Item = &Note> {
        self.entries.iter().flat_map(|t| t.notes.iter())
    }

    /// Get all measure lines.
    pub fn measure_lines(&self) -> impl Iterator<Item = &Timeline> {
        self.entries.iter().filter(|t| t.is_measure_line)
    }

    /// Get the last time in milliseconds.
    pub fn last_time_ms(&self) -> f64 {
        self.entries
            .iter()
            .map(|t| {
                t.notes
                    .iter()
                    .map(|n| n.end_time_ms.unwrap_or(n.start_time_ms))
                    .fold(t.time_ms, f64::max)
            })
            .fold(0.0, f64::max)
    }
}
