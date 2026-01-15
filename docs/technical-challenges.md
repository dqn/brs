# Technical Challenges and Solutions

This document describes the key technical challenges in implementing a BMS player and their solutions.

## Challenge 1: Audio-Visual Synchronization

### Problem

Audio and visual must be synchronized within tight tolerances:
- PGREAT window: ±20ms
- Standard game loops introduce variable latency
- Frame rate variations cause drift

### Solution

Use Kira's clock as the single source of truth:

```rust
pub struct GameClock {
    kira_clock: kira::clock::ClockHandle,
    start_time: kira::ClockTime,
}

impl GameClock {
    pub fn current_time_ms(&self) -> f64 {
        let ticks = self.kira_clock.time().ticks;
        (ticks as f64 / TICKS_PER_BEAT) * (60000.0 / self.current_bpm)
    }
}
```

Key insight: Derive visual position from audio clock, never the reverse.

## Challenge 2: BPM Changes and Timing Calculation

### Problem

BMS supports complex timing:
- BPM changes (channel 03, 08)
- Variable measure lengths (channel 02)
- Stop sequences (channel 09)
- Floating-point accumulation causes drift

### Solution

Use fraction arithmetic for measure-based calculations:

```rust
use fraction::Fraction;

pub fn calculate_time_ms(
    measure_position: Fraction,
    timing_data: &TimingData,
) -> f64 {
    let mut time_ms = 0.0;
    let mut current_bpm = timing_data.initial_bpm;
    let mut last_position = Fraction::from(0);

    // Process BPM changes up to this position
    for change in &timing_data.bpm_changes {
        if change.position > measure_position {
            break;
        }
        let duration = position_to_beats(change.position - last_position, timing_data);
        time_ms += beats_to_ms(duration, current_bpm);
        current_bpm = change.bpm;
        last_position = change.position;
    }

    // Add remaining time
    let duration = position_to_beats(measure_position - last_position, timing_data);
    time_ms += beats_to_ms(duration, current_bpm);

    // Add stop durations
    time_ms += calculate_stop_time(measure_position, timing_data);

    time_ms
}
```

## Challenge 3: Keysound Loading Performance

### Problem

- A single BMS chart can reference 1000+ keysounds
- Loading all at startup causes unacceptable delays
- Memory pressure on systems with limited RAM

### Solution

Progressive loading with priority queue:

```rust
pub struct KeysoundManager {
    loaded: HashMap<ObjId, StaticSoundData>,
    loading_queue: VecDeque<ObjId>,
}

impl KeysoundManager {
    pub fn prepare_chart(&mut self, chart: &Chart) {
        // Extract keysound IDs sorted by first occurrence
        let mut keysounds: Vec<_> = chart.notes.iter()
            .map(|n| (n.time_ms, n.keysound_id))
            .collect();
        keysounds.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

        // Load first 30 seconds synchronously
        let critical: HashSet<_> = keysounds.iter()
            .filter(|(time, _)| *time < 30000.0)
            .map(|(_, id)| *id)
            .collect();

        for id in &critical {
            self.load_sync(*id);
        }

        // Queue remainder for background loading
        for (_, id) in keysounds {
            if !critical.contains(&id) {
                self.loading_queue.push_back(id);
            }
        }

        self.start_background_loading();
    }
}
```

## Challenge 4: Input Latency

### Problem

- macroquad's `is_key_pressed()` samples once per frame
- At 60fps, this adds up to 16ms latency
- Rhythm games require sub-frame precision

### Solution

Timestamp input events and process retroactively:

```rust
pub struct InputBuffer {
    events: Vec<InputEvent>,
    audio_offset: f64,
}

pub struct InputEvent {
    key: KeyCode,
    pressed: bool,
    timestamp: f64,  // System clock time
}

impl InputBuffer {
    pub fn process(&mut self, game_time: f64, notes: &mut [NoteState]) {
        for event in self.events.drain(..) {
            let adjusted_time = event.timestamp - self.audio_offset;

            for note in notes.iter_mut() {
                if note.channel.matches_key(event.key) && !note.judged {
                    let delta = adjusted_time - note.time_ms;
                    if delta.abs() < self.judge_config.bad_window {
                        note.judge(delta, &self.judge_config);
                        break;
                    }
                }
            }
        }
    }
}
```

## Challenge 5: Long Note Handling

### Problem

- Long notes have start and end with different judgment rules
- Player must hold through the duration
- Early release should be penalized

### Solution

State machine per long note:

```rust
pub enum LongNoteState {
    Pending,
    Active { start_judgment: JudgeResult },
    Completed { end_judgment: JudgeResult },
    Failed,
}

impl LongNote {
    pub fn update(&mut self, key_held: bool, current_time: f64) {
        match self.state {
            LongNoteState::Pending => {
                // Wait for press within window
            }
            LongNoteState::Active { .. } => {
                if !key_held {
                    self.state = LongNoteState::Failed;
                } else if current_time >= self.end_time {
                    self.state = LongNoteState::Completed {
                        end_judgment: self.judge_end(current_time)
                    };
                }
            }
            _ => {}
        }
    }
}
```

## Challenge 6: Variable Measure Lengths

### Problem

- Channel 02 can change measure length (e.g., 0.75 for 3/4 time)
- Affects all timing calculations for subsequent notes
- Can change mid-song

### Solution

Pre-process measure boundaries:

```rust
pub struct MeasureInfo {
    pub number: u32,
    pub start_time_ms: f64,
    pub length: f64,  // 1.0 = 4/4, 0.75 = 3/4
    pub duration_ms: f64,
}

pub fn build_measure_table(timing_data: &TimingData, max_measure: u32) -> Vec<MeasureInfo> {
    let mut measures = Vec::new();
    let mut current_time = 0.0;
    let mut current_bpm = timing_data.initial_bpm;

    for m in 0..=max_measure {
        let length = timing_data.get_measure_length(m);
        let beats = 4.0 * length;
        let duration = beats * (60000.0 / current_bpm);

        measures.push(MeasureInfo {
            number: m,
            start_time_ms: current_time,
            length,
            duration_ms: duration,
        });

        // Apply BPM changes within this measure
        // Apply stops within this measure
        current_time += duration + stops_in_measure(m, timing_data);
    }

    measures
}
```

## Challenge 7: Memory Management for Audio

### Problem

- Keysounds can be large WAV files
- Playing same keysound rapidly (e.g., hi-hats) requires multiple instances
- Memory usage can spike

### Solution

Use Kira's `StaticSoundData` with shared references:

```rust
pub struct KeysoundPool {
    data: HashMap<ObjId, Arc<StaticSoundData>>,
}

impl KeysoundPool {
    pub fn play(&self, id: ObjId, manager: &mut AudioManager) -> Option<StaticSoundHandle> {
        self.data.get(&id).map(|data| {
            manager.play(data.clone()).ok()
        }).flatten()
    }
}
```

Kira handles polyphony internally - same sound data can play multiple overlapping instances.

## Testing Strategy

### Unit Tests

- Timing calculations with known BMS files
- Judgment window edge cases
- Long note state transitions

### Integration Tests

- Parse → Convert → Calculate timing → Verify against known good values
- Audio scheduling accuracy (compare scheduled vs actual play times)

### Manual Testing

- Real BMS charts from various sources
- Edge cases: very high BPM, extreme measure lengths, dense patterns
