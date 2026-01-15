# Long Note Implementation

## Overview

Long notes (LN) require players to hold a key for a duration. Different BMS players and formats support varying types of long notes with different judgment mechanics.

## Long Note Types

### LN (Long Note) - BMS/LR2 Style

- Press at the start
- Hold until the end
- **No release timing judgment** - just need to hold until end
- Early release = miss

### CN (Charge Note) - IIDX Style

- Press at the start (start timing is judged)
- Hold until the end
- **No release timing judgment**
- Early release = BAD/POOR

### HCN (Hell Charge Note) - IIDX Copula+

- Press at the start
- Must hold continuously
- **Releasing during hold = continuous damage**
- Must release at the end (end timing not strictly judged)

### BSS (Back Spin Scratch)

- Start by spinning scratch in one direction
- Hold/continue spinning
- Release by spinning in the opposite direction
- End timing is judged

## BMS Format Specification

### Long Note Channels

| Channel | Description |
|---------|-------------|
| 51-59 | 1P Long Note end positions |
| 61-69 | 2P Long Note end positions |

**Mapping:**
- 51 → Key 1 (channel 11)
- 52 → Key 2 (channel 12)
- ...
- 56 → Scratch (channel 16)

### #LNTYPE Header

```
#LNTYPE 1  ; LN (Ez2DJ style, default)
#LNTYPE 2  ; CN (IIDX style)
```

### #LNOBJ Command

Alternative way to specify LN:
```
#LNOBJ ZZ  ; Object ZZ marks LN end
```

When a regular note uses object ZZ, it becomes the end of a previous LN in the same lane.

## Judgment Behavior

### LR2 Style (LN)

| Event | Judgment |
|-------|----------|
| Press at start | Normal judgment (PGREAT/GREAT/etc.) |
| Hold during | - |
| Release at end | No judgment (just need to be holding) |
| Early release | POOR |

### beatoraja Style (CN)

| Event | Judgment |
|-------|----------|
| Press at start | Normal judgment |
| Hold during | - |
| Release at end | Release judgment (wider window) |
| Early release | POOR |

**Release Windows (beatoraja):**
- ±120ms (PGREAT equivalent)
- ±160ms (GREAT equivalent)
- ±200ms (GOOD equivalent)
- ±220-280ms (BAD equivalent)

### LR2 vs beatoraja Early Release

| System | Early Release Penalty |
|--------|----------------------|
| beatoraja | POOR |
| LR2 | BAD |

This affects survival gauge damage.

## Implementation

### Data Structures

```rust
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum LongNoteType {
    /// Ez2DJ style - no release judgment
    LN,
    /// IIDX style - with release judgment
    CN,
    /// Hell Charge Note - damage while released
    HCN,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum NoteType {
    Normal,
    LongStart { end_time_ms: i64, ln_type: LongNoteType },
    LongEnd,
    Mine,
}

pub struct Note {
    pub time_ms: i64,
    pub lane: usize,
    pub note_type: NoteType,
    pub keysound_id: Option<u32>,
}

/// State tracking for active long notes
pub struct ActiveLongNote {
    pub lane: usize,
    pub start_time_ms: i64,
    pub end_time_ms: i64,
    pub ln_type: LongNoteType,
    pub start_judgment: Judgment,
    pub is_holding: bool,
}
```

### Long Note State Machine

```rust
pub struct LongNoteManager {
    /// Currently active long notes per lane
    active: [Option<ActiveLongNote>; LANE_COUNT],
    /// HCN damage accumulator
    hcn_damage_timer: [f32; LANE_COUNT],
}

impl LongNoteManager {
    pub fn new() -> Self {
        Self {
            active: Default::default(),
            hcn_damage_timer: [0.0; LANE_COUNT],
        }
    }

    /// Called when a long note start is hit
    pub fn start_long_note(
        &mut self,
        lane: usize,
        start_time: i64,
        end_time: i64,
        ln_type: LongNoteType,
        judgment: Judgment,
    ) {
        self.active[lane] = Some(ActiveLongNote {
            lane,
            start_time_ms: start_time,
            end_time_ms: end_time,
            ln_type,
            start_judgment: judgment,
            is_holding: true,
        });
    }

    /// Called when key is released
    pub fn key_released(
        &mut self,
        lane: usize,
        release_time: i64,
        judge_property: &JudgeProperty,
    ) -> Option<LongNoteResult> {
        let ln = self.active[lane].take()?;

        let timing_diff = release_time - ln.end_time_ms;

        match ln.ln_type {
            LongNoteType::LN => {
                // LR2 style: no release judgment, just check if early
                if release_time < ln.end_time_ms {
                    Some(LongNoteResult::EarlyRelease)
                } else {
                    Some(LongNoteResult::Complete(ln.start_judgment))
                }
            }
            LongNoteType::CN => {
                // IIDX style: judge the release timing
                if release_time < ln.end_time_ms - judge_property.good.late {
                    Some(LongNoteResult::EarlyRelease)
                } else {
                    let release_judgment = self.judge_release(timing_diff, judge_property);
                    Some(LongNoteResult::CompleteWithRelease {
                        start: ln.start_judgment,
                        release: release_judgment,
                    })
                }
            }
            LongNoteType::HCN => {
                // HCN: always complete if released near end
                Some(LongNoteResult::Complete(ln.start_judgment))
            }
        }
    }

    fn judge_release(&self, timing_diff: i64, prop: &JudgeProperty) -> Judgment {
        // Use wider release windows
        let release_windows = JudgeProperty::release_windows(prop);

        if release_windows.pgreat.contains(timing_diff) {
            Judgment::PGreat
        } else if release_windows.great.contains(timing_diff) {
            Judgment::Great
        } else if release_windows.good.contains(timing_diff) {
            Judgment::Good
        } else {
            Judgment::Bad
        }
    }

    /// Called every frame to update HCN state
    pub fn update(&mut self, current_time: i64, delta_ms: f32) -> Vec<HcnDamageEvent> {
        let mut events = Vec::new();

        for lane in 0..LANE_COUNT {
            if let Some(ref mut ln) = self.active[lane] {
                if ln.ln_type == LongNoteType::HCN && !ln.is_holding {
                    // Accumulate damage while not holding HCN
                    self.hcn_damage_timer[lane] += delta_ms;

                    // Apply damage every 100ms of not holding
                    if self.hcn_damage_timer[lane] >= 100.0 {
                        self.hcn_damage_timer[lane] -= 100.0;
                        events.push(HcnDamageEvent { lane });
                    }
                }
            }
        }

        events
    }

    /// Called when key is pressed (for HCN re-press)
    pub fn key_pressed(&mut self, lane: usize, press_time: i64) {
        if let Some(ref mut ln) = self.active[lane] {
            if ln.ln_type == LongNoteType::HCN {
                ln.is_holding = true;
                self.hcn_damage_timer[lane] = 0.0;
            }
        }
    }

    /// Check if any long note has passed its end time without being released
    pub fn check_missed_ends(&mut self, current_time: i64) -> Vec<MissedLongNoteEnd> {
        let mut missed = Vec::new();

        for lane in 0..LANE_COUNT {
            if let Some(ref ln) = self.active[lane] {
                // If we're past the end time + tolerance and still holding
                if current_time > ln.end_time_ms + 500 {
                    missed.push(MissedLongNoteEnd { lane });
                    self.active[lane] = None;
                }
            }
        }

        missed
    }
}

pub enum LongNoteResult {
    EarlyRelease,
    Complete(Judgment),
    CompleteWithRelease {
        start: Judgment,
        release: Judgment,
    },
}

pub struct HcnDamageEvent {
    pub lane: usize,
}

pub struct MissedLongNoteEnd {
    pub lane: usize,
}
```

### Integration with Chart Loading

```rust
pub fn parse_long_notes(bms: &Bms) -> Vec<Note> {
    let mut notes = Vec::new();
    let mut pending_starts: HashMap<usize, Note> = HashMap::new();

    let ln_type = match bms.header.lntype {
        Some(1) | None => LongNoteType::LN,
        Some(2) => LongNoteType::CN,
        _ => LongNoteType::LN,
    };

    for obj in &bms.objects {
        match obj.channel {
            // Regular note channels
            0x11..=0x19 | 0x21..=0x29 => {
                if let Some(lnobj) = bms.header.lnobj {
                    if obj.object_id == lnobj {
                        // This is an LN end marker
                        if let Some(mut start) = pending_starts.remove(&obj.lane) {
                            start.note_type = NoteType::LongStart {
                                end_time_ms: obj.time_ms,
                                ln_type,
                            };
                            notes.push(start);
                            notes.push(Note {
                                time_ms: obj.time_ms,
                                lane: obj.lane,
                                note_type: NoteType::LongEnd,
                                keysound_id: None,
                            });
                        }
                    } else {
                        // Potential LN start
                        pending_starts.insert(obj.lane, Note {
                            time_ms: obj.time_ms,
                            lane: obj.lane,
                            note_type: NoteType::Normal,
                            keysound_id: Some(obj.object_id),
                        });
                    }
                } else {
                    // No LNOBJ, treat as normal note
                    notes.push(Note {
                        time_ms: obj.time_ms,
                        lane: obj.lane,
                        note_type: NoteType::Normal,
                        keysound_id: Some(obj.object_id),
                    });
                }
            }
            // Explicit LN channels (51-59, 61-69)
            0x51..=0x59 | 0x61..=0x69 => {
                let key_lane = (obj.channel & 0x0F) as usize;
                if let Some(mut start) = pending_starts.remove(&key_lane) {
                    start.note_type = NoteType::LongStart {
                        end_time_ms: obj.time_ms,
                        ln_type,
                    };
                    notes.push(start);
                    notes.push(Note {
                        time_ms: obj.time_ms,
                        lane: key_lane,
                        note_type: NoteType::LongEnd,
                        keysound_id: None,
                    });
                }
            }
            _ => {}
        }
    }

    // Flush any remaining pending notes as normal notes
    for (_, note) in pending_starts {
        notes.push(note);
    }

    notes.sort_by_key(|n| n.time_ms);
    notes
}
```

## LN Mode Options

beatoraja supports transforming notes:

| Mode | Description |
|------|-------------|
| REMOVE | Convert all LN to normal notes |
| ADD LN | Convert normal notes to LN |
| ADD CN | Convert normal notes to CN |
| ADD HCN | Convert normal notes to HCN |
| ADD ALL | Mix of LN/CN/HCN randomly |

### LEGACY NOTE Assist Option

Converts all LN types to normal notes. Disables score saving.

## Scoring for Long Notes

### EX Score

| Event | EX Score |
|-------|----------|
| LN Start PGREAT | +2 |
| LN Start GREAT | +1 |
| LN Complete (no early release) | +0 |
| CN Release PGREAT | +2 |
| CN Release GREAT | +1 |
| Early Release | -0 (but miss damage) |

### Combo

- LN start adds to combo
- LN end adds to combo (if not early release)
- Early release breaks combo

## Reference Links

- [BMS Format Specification](https://hitkey.nekokan.dyndns.info/cmds.htm)
- [bmson Specification](https://bmson-spec.readthedocs.io/)
- [BMS Terminology](https://news.keysounds.net/terminology)
- [beatoraja Long Note Implementation](https://github.com/exch-bms2/beatoraja)
