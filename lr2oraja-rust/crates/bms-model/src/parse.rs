use std::collections::HashMap;
use std::path::Path;

use anyhow::Result;

use crate::mode::PlayMode;
use crate::model::BmsModel;
use crate::note::{BgNote, LnType, Note};
use crate::timeline::{BpmChange, StopEvent, TimeLine};

/// BMS file decoder
pub struct BmsDecoder;

/// Tracks #RANDOM state with optional fixed selections
struct RandomResolver {
    /// Pre-selected values (index into this as #RANDOM commands are encountered)
    selected: Option<Vec<i32>>,
    /// Count of #RANDOM commands seen so far
    count: usize,
}

/// Internal representation of a channel event during parsing
#[derive(Debug, Clone)]
struct ChannelEvent {
    measure: u32,
    channel: u16,
    /// Parsed pairs of (position_in_measure, wav_id)
    data: Vec<(f64, u16)>,
}

/// Active LN tracking per lane
struct LnState {
    wav_id: u16,
    time_us: i64,
}

impl RandomResolver {
    fn new(selected: Option<Vec<i32>>) -> Self {
        Self { selected, count: 0 }
    }

    /// Resolve the next #RANDOM value
    fn next(&mut self, bound: i32) -> i32 {
        let value = if let Some(ref selected) = self.selected {
            // Use pre-selected value if available
            if self.count < selected.len() {
                selected[self.count]
            } else {
                simple_random(bound)
            }
        } else {
            simple_random(bound)
        };
        self.count += 1;
        value
    }
}

impl BmsDecoder {
    pub fn decode(path: &Path) -> Result<BmsModel> {
        let raw_bytes = std::fs::read(path)?;
        let content = detect_encoding_and_decode(&raw_bytes);
        let mut model = Self::decode_str(&content, path)?;
        // Recompute hashes from raw bytes (matches Java DigestInputStream behavior)
        compute_hashes(&raw_bytes, &mut model);
        Ok(model)
    }

    /// Decode with pre-selected #RANDOM values (for deterministic golden master testing)
    pub fn decode_with_randoms(path: &Path, selected_randoms: &[i32]) -> Result<BmsModel> {
        let raw_bytes = std::fs::read(path)?;
        let content = detect_encoding_and_decode(&raw_bytes);
        let mut model =
            Self::decode_str_with_randoms(&content, path, Some(selected_randoms.to_vec()))?;
        compute_hashes(&raw_bytes, &mut model);
        Ok(model)
    }

    pub fn decode_str(content: &str, path: &Path) -> Result<BmsModel> {
        Self::decode_str_with_randoms(content, path, None)
    }

    fn decode_str_with_randoms(
        content: &str,
        path: &Path,
        selected_randoms: Option<Vec<i32>>,
    ) -> Result<BmsModel> {
        let mut model = BmsModel::default();
        let mut events: Vec<ChannelEvent> = Vec::new();
        let mut measure_lengths: HashMap<u32, f64> = HashMap::new();
        let mut extended_bpms: HashMap<u16, f64> = HashMap::new();
        let mut stop_defs: HashMap<u16, i64> = HashMap::new();
        let mut random_stack: Vec<RandomState> = Vec::new();
        let mut random_resolver = RandomResolver::new(selected_randoms);
        let mut max_measure: u32 = 0;

        // Track which key channels are used for mode detection
        let mut max_1p_channel: usize = 0;
        let mut has_2p = false;

        let base_dir = path.parent().unwrap_or(Path::new("."));

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('*') {
                continue;
            }

            // Handle #RANDOM / #IF / #ENDIF / #ENDRANDOM
            if let Some(rest) = line.strip_prefix('#') {
                let upper = rest.to_ascii_uppercase();

                if let Some(rest) = upper.strip_prefix("RANDOM ") {
                    let bound: i32 = rest.trim().parse().unwrap_or(1);
                    let value = random_resolver.next(bound);
                    random_stack.push(RandomState {
                        bound,
                        value,
                        active: true,
                    });
                    model.has_random = true;
                    continue;
                }
                if let Some(rest) = upper.strip_prefix("IF ") {
                    if let Some(state) = random_stack.last_mut() {
                        let target: i32 = rest.trim().parse().unwrap_or(0);
                        state.active = target == state.value;
                    }
                    continue;
                }
                if upper == "ENDIF" {
                    if let Some(state) = random_stack.last_mut() {
                        state.active = true;
                    }
                    continue;
                }
                if upper == "ENDRANDOM" {
                    random_stack.pop();
                    continue;
                }

                // Skip lines inside inactive #IF blocks
                if random_stack.iter().any(|s| !s.active) {
                    continue;
                }

                // Header commands
                if let Some(rest) = upper.strip_prefix("PLAYER ") {
                    model.player = rest.trim().parse().unwrap_or(1);
                } else if upper.starts_with("GENRE ") {
                    model.genre = rest[6..].trim().to_string();
                } else if upper.starts_with("TITLE ") {
                    model.title = rest[6..].trim().to_string();
                } else if upper.starts_with("SUBTITLE ") {
                    model.subtitle = rest[9..].trim().to_string();
                } else if upper.starts_with("ARTIST ") {
                    model.artist = rest[7..].trim().to_string();
                } else if upper.starts_with("SUBARTIST ") {
                    model.sub_artist = rest[10..].trim().to_string();
                } else if upper.starts_with("BPM ") && !upper.starts_with("BPM0") {
                    // #BPM (initial BPM, not #BPMxx)
                    model.initial_bpm = rest[4..].trim().parse().unwrap_or(130.0);
                } else if upper.starts_with("BPM")
                    && upper.len() >= 5
                    && upper.as_bytes()[5] == b' '
                {
                    // #BPMxx value (extended BPM definition)
                    let id = parse_base36(&upper[3..5]);
                    let bpm: f64 = rest[6..].trim().parse().unwrap_or(0.0);
                    extended_bpms.insert(id, bpm);
                } else if let Some(rest) = upper.strip_prefix("RANK ") {
                    let raw: i32 = rest.trim().parse().unwrap_or(2);
                    model.judge_rank_raw = raw;
                    // Convert rank 0-4 to judgerank value
                    model.judge_rank = match raw {
                        0 => 100, // VERY HARD
                        1 => 75,  // HARD
                        2 => 50,  // NORMAL
                        3 => 25,  // EASY
                        _ => raw,
                    };
                } else if let Some(rest) = upper.strip_prefix("DEFEXRANK ") {
                    let raw: i32 = rest.trim().parse().unwrap_or(100);
                    model.judge_rank = raw;
                    model.judge_rank_raw = raw;
                } else if let Some(rest) = upper.strip_prefix("TOTAL ") {
                    model.total = rest.trim().parse().unwrap_or(300.0);
                } else if let Some(rest) = upper.strip_prefix("PLAYLEVEL ") {
                    model.play_level = rest.trim().parse().unwrap_or(0);
                } else if let Some(rest) = upper.strip_prefix("DIFFICULTY ") {
                    model.difficulty = rest.trim().parse().unwrap_or(0);
                } else if let Some(rest) = upper.strip_prefix("LNTYPE ") {
                    let ln: i32 = rest.trim().parse().unwrap_or(1);
                    model.ln_type = match ln {
                        2 => LnType::ChargeNote,
                        3 => LnType::HellChargeNote,
                        _ => LnType::LongNote,
                    };
                } else if upper.starts_with("BANNER ") {
                    model.banner = rest[7..].trim().to_string();
                } else if upper.starts_with("STAGEFILE ") {
                    model.stage_file = rest[10..].trim().to_string();
                } else if upper.starts_with("BACKBMP ") {
                    model.back_bmp = rest[8..].trim().to_string();
                } else if upper.starts_with("PREVIEW ") {
                    model.preview = rest[8..].trim().to_string();
                } else if upper.starts_with("WAV") && upper.len() >= 5 {
                    let id = parse_base36(&upper[3..5]);
                    let filename = rest[5..].trim();
                    if !filename.is_empty() {
                        model.wav_defs.insert(id, base_dir.join(filename));
                    }
                } else if upper.starts_with("BMP")
                    && upper.len() >= 5
                    && upper.as_bytes()[3] != b' '
                {
                    let id = parse_base36(&upper[3..5]);
                    let filename = rest[5..].trim();
                    if !filename.is_empty() {
                        model.bmp_defs.insert(id, base_dir.join(filename));
                    }
                } else if upper.starts_with("STOP") && upper.len() >= 6 {
                    let id = parse_base36(&upper[4..6]);
                    let ticks: i64 = rest[6..].trim().parse().unwrap_or(0);
                    stop_defs.insert(id, ticks);
                } else if let Some(event) = parse_channel_line(&upper) {
                    // Channel data: #MMMCC:data
                    let measure = event.measure;
                    if measure > max_measure {
                        max_measure = measure;
                    }

                    let ch = event.channel;
                    // Track channel usage for mode detection
                    if (0x11..=0x19).contains(&ch) {
                        let lane = (ch - 0x11) as usize;
                        if lane + 1 > max_1p_channel {
                            max_1p_channel = lane + 1;
                        }
                    }
                    if (0x21..=0x29).contains(&ch) {
                        has_2p = true;
                    }
                    if (0x51..=0x59).contains(&ch) {
                        let lane = (ch - 0x51) as usize;
                        if lane + 1 > max_1p_channel {
                            max_1p_channel = lane + 1;
                        }
                    }

                    if ch == 0x02 {
                        // Measure length change
                        if let Some(&(_, _val)) = event.data.first() {
                            // Channel 02 data is special: raw float value
                            let len_str = &content.lines().find(|l| {
                                let l = l.trim();
                                l.starts_with('#') && {
                                    let u = l[1..].to_ascii_uppercase();
                                    u.starts_with(&format!("{:03}02:", measure))
                                }
                            });
                            if let Some(line) = len_str
                                && let Some(colon_pos) = line.find(':')
                            {
                                let val: f64 = line[colon_pos + 1..].trim().parse().unwrap_or(1.0);
                                measure_lengths.insert(measure, val);
                            }
                        }
                    } else {
                        events.push(event);
                    }
                }
            }
        }

        // Re-parse measure lengths more reliably
        for line in content.lines() {
            let line = line.trim();
            if let Some(rest) = line.strip_prefix('#') {
                let upper = rest.to_ascii_uppercase();
                if upper.len() >= 6 && &upper[3..5] == "02" && upper.as_bytes()[5] == b':' {
                    let measure: u32 = upper[..3].parse().unwrap_or(0);
                    let val: f64 = rest[6..].trim().parse().unwrap_or(1.0);
                    measure_lengths.insert(measure, val);
                }
            }
        }

        model.total_measures = max_measure + 1;

        // Detect play mode
        let is_pms = path
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("pms"));
        if has_2p || model.player == 3 {
            if max_1p_channel > 6 {
                model.mode = PlayMode::Beat14K;
            } else {
                model.mode = PlayMode::Beat10K;
            }
        } else if is_pms {
            model.mode = PlayMode::PopN9K;
        } else if max_1p_channel > 6 {
            model.mode = PlayMode::Beat7K;
        } else {
            model.mode = PlayMode::Beat5K;
        }

        // Build timeline: convert measure/position → microseconds
        // Phase 1: compute time for each measure start
        let mut measure_times: Vec<i64> = Vec::new();
        let mut current_time_us: i64 = 0;
        let mut current_bpm = model.initial_bpm;

        // Collect all BPM changes and stops per measure with position
        let mut bpm_events_by_measure: HashMap<u32, Vec<(f64, f64)>> = HashMap::new(); // pos -> new_bpm
        let mut stop_events_by_measure: HashMap<u32, Vec<(f64, u16)>> = HashMap::new(); // pos -> stop_id

        for event in &events {
            if event.channel == 0x03 {
                // Integer BPM change (channel 03 data is hex 00-FF, not base36)
                for &(pos, val) in &event.data {
                    if val > 0 {
                        let bpm = base36_to_hex(val) as f64;
                        bpm_events_by_measure
                            .entry(event.measure)
                            .or_default()
                            .push((pos, bpm));
                    }
                }
            } else if event.channel == 0x08 {
                // Extended BPM change
                for &(pos, id) in &event.data {
                    if id > 0
                        && let Some(&bpm) = extended_bpms.get(&id)
                    {
                        bpm_events_by_measure
                            .entry(event.measure)
                            .or_default()
                            .push((pos, bpm));
                    }
                }
            } else if event.channel == 0x09 {
                // STOP event
                for &(pos, id) in &event.data {
                    if id > 0 {
                        stop_events_by_measure
                            .entry(event.measure)
                            .or_default()
                            .push((pos, id));
                    }
                }
            }
        }

        // Phase 2: walk through measures, computing time for each position
        for measure in 0..=max_measure {
            measure_times.push(current_time_us);
            let measure_len = measure_lengths.get(&measure).copied().unwrap_or(1.0);

            // Collect events in this measure, sorted by position
            let mut timing_events: Vec<(f64, TimingEvent)> = Vec::new();

            if let Some(bpm_evts) = bpm_events_by_measure.get(&measure) {
                for &(pos, bpm) in bpm_evts {
                    timing_events.push((pos, TimingEvent::Bpm(bpm)));
                }
            }
            if let Some(stop_evts) = stop_events_by_measure.get(&measure) {
                for &(pos, id) in stop_evts {
                    if let Some(&ticks) = stop_defs.get(&id) {
                        timing_events.push((pos, TimingEvent::Stop(ticks)));
                    }
                }
            }

            timing_events.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

            let mut prev_pos = 0.0;
            for (pos, event) in &timing_events {
                // Advance time from prev_pos to this position
                let delta_beats = (*pos - prev_pos) * 4.0 * measure_len;
                let delta_us = beats_to_us(delta_beats, current_bpm);
                current_time_us += delta_us;

                match event {
                    TimingEvent::Bpm(new_bpm) => {
                        model.bpm_changes.push(BpmChange {
                            time_us: current_time_us,
                            bpm: *new_bpm,
                        });
                        current_bpm = *new_bpm;
                    }
                    TimingEvent::Stop(ticks) => {
                        let stop_us = ticks_to_us(*ticks, current_bpm);
                        model.stop_events.push(StopEvent {
                            time_us: current_time_us,
                            duration_ticks: *ticks,
                            duration_us: stop_us,
                        });
                        current_time_us += stop_us;
                    }
                }
                prev_pos = *pos;
            }

            // Advance to end of measure
            let remaining_beats = (1.0 - prev_pos) * 4.0 * measure_len;
            current_time_us += beats_to_us(remaining_beats, current_bpm);
        }

        model.total_time_us = current_time_us;

        // Phase 3: Place notes
        let mut ln_states: HashMap<(u32, usize), LnState> = HashMap::new(); // (channel_group, lane) -> state

        for event in &events {
            let ch = event.channel;

            // Skip timing channels (already processed), but not 0x01 (BGM)
            if matches!(ch, 0x02 | 0x03 | 0x04 | 0x06 | 0x07 | 0x08 | 0x09) {
                continue;
            }

            let measure = event.measure;
            let measure_time = measure_times.get(measure as usize).copied().unwrap_or(0);
            let measure_len = measure_lengths.get(&measure).copied().unwrap_or(1.0);

            for &(pos, wav_id) in &event.data {
                if wav_id == 0 {
                    continue;
                }

                let time_us = measure_time
                    + position_to_us(
                        pos,
                        measure,
                        measure_len,
                        &measure_times,
                        &model,
                        current_bpm,
                        &bpm_events_by_measure,
                        &stop_events_by_measure,
                        &stop_defs,
                        &extended_bpms,
                    );

                // BGM channel (01): add as background note
                if ch == 0x01 {
                    model.bg_notes.push(BgNote {
                        wav_id,
                        time_us,
                        micro_starttime: 0,
                        micro_duration: 0,
                    });
                    continue;
                }

                let (lane, note_kind) = match ch {
                    // 1P visible (11-19)
                    0x11..=0x19 => {
                        let idx = (ch - 0x11) as usize;
                        let assign = model.mode.channel_assign_1p();
                        let l = assign[idx];
                        if l < 0 {
                            continue;
                        }
                        (l as usize, NoteKind::Normal)
                    }
                    // 2P visible (21-29)
                    0x21..=0x29 => {
                        let idx = (ch - 0x21) as usize;
                        let assign = model.mode.channel_assign_2p();
                        let l = assign[idx];
                        if l < 0 {
                            continue;
                        }
                        (l as usize, NoteKind::Normal)
                    }
                    // 1P invisible (31-39)
                    0x31..=0x39 => {
                        let idx = (ch - 0x31) as usize;
                        let assign = model.mode.channel_assign_1p();
                        let l = assign[idx];
                        if l < 0 {
                            continue;
                        }
                        (l as usize, NoteKind::Invisible)
                    }
                    // 2P invisible (41-49)
                    0x41..=0x49 => {
                        let idx = (ch - 0x41) as usize;
                        let assign = model.mode.channel_assign_2p();
                        let l = assign[idx];
                        if l < 0 {
                            continue;
                        }
                        (l as usize, NoteKind::Invisible)
                    }
                    // 1P LN (51-59)
                    0x51..=0x59 => {
                        let idx = (ch - 0x51) as usize;
                        let assign = model.mode.channel_assign_1p();
                        let l = assign[idx];
                        if l < 0 {
                            continue;
                        }
                        (l as usize, NoteKind::LongNote)
                    }
                    // 2P LN (61-69)
                    0x61..=0x69 => {
                        let idx = (ch - 0x61) as usize;
                        let assign = model.mode.channel_assign_2p();
                        let l = assign[idx];
                        if l < 0 {
                            continue;
                        }
                        (l as usize, NoteKind::LongNote)
                    }
                    // 1P mine (D1-D9)
                    0xD1..=0xD9 => {
                        let idx = (ch - 0xD1) as usize;
                        let assign = model.mode.channel_assign_1p();
                        let l = assign[idx];
                        if l < 0 {
                            continue;
                        }
                        (l as usize, NoteKind::Mine)
                    }
                    // 2P mine (E1-E9)
                    0xE1..=0xE9 => {
                        let idx = (ch - 0xE1) as usize;
                        let assign = model.mode.channel_assign_2p();
                        let l = assign[idx];
                        if l < 0 {
                            continue;
                        }
                        (l as usize, NoteKind::Mine)
                    }
                    _ => continue,
                };

                match note_kind {
                    NoteKind::Normal => {
                        model.notes.push(Note::normal(lane, time_us, wav_id));
                    }
                    NoteKind::Invisible => {
                        model.notes.push(Note::invisible(lane, time_us, wav_id));
                    }
                    NoteKind::Mine => {
                        model
                            .notes
                            .push(Note::mine(lane, time_us, wav_id, wav_id as i32));
                    }
                    NoteKind::LongNote => {
                        let key = ((ch & 0x0F) as u32, lane);
                        if let Some(start_state) = ln_states.remove(&key) {
                            // End of LN
                            let note = Note::long_note(
                                lane,
                                start_state.time_us,
                                time_us,
                                start_state.wav_id,
                                wav_id,
                                model.ln_type,
                            );
                            model.notes.push(note);
                        } else {
                            // Start of LN
                            ln_states.insert(key, LnState { wav_id, time_us });
                        }
                    }
                }
            }
        }

        // Sort notes by time, then by lane
        model
            .notes
            .sort_by(|a, b| a.time_us.cmp(&b.time_us).then_with(|| a.lane.cmp(&b.lane)));

        // Sort background notes by time
        model.bg_notes.sort_by_key(|n| n.time_us);

        // Deduplicate: when same (lane, time_us), keep highest priority note
        // Priority: Normal/Invisible > LN > Mine
        model.notes.dedup_by(|b, a| {
            if a.lane == b.lane && a.time_us == b.time_us {
                if note_priority(b) > note_priority(a) {
                    std::mem::swap(a, b);
                }
                true
            } else {
                false
            }
        });

        // Build timelines
        let mut seen_times: Vec<i64> = model.notes.iter().map(|n| n.time_us).collect();
        seen_times.sort();
        seen_times.dedup();

        for &t in &seen_times {
            // Find BPM at this time
            let bpm = bpm_at_time(t, model.initial_bpm, &model.bpm_changes);
            model.timelines.push(TimeLine {
                time_us: t,
                measure: 0, // Could be computed but not critical
                position: 0.0,
                bpm,
            });
        }

        // Compute hashes
        compute_hashes(&raw_content_for_hash(content), &mut model);

        Ok(model)
    }
}

#[derive(Debug)]
enum TimingEvent {
    Bpm(f64),
    Stop(i64),
}

#[derive(Debug, Clone, Copy)]
enum NoteKind {
    Normal,
    Invisible,
    LongNote,
    Mine,
}

struct RandomState {
    #[allow(dead_code)]
    bound: i32,
    value: i32,
    active: bool,
}

/// Convert beats to microseconds at given BPM
fn beats_to_us(beats: f64, bpm: f64) -> i64 {
    if bpm <= 0.0 {
        return 0;
    }
    ((beats * 60_000_000.0) / bpm) as i64
}

/// Convert STOP ticks to microseconds (192 ticks = 1 measure = 4 beats)
fn ticks_to_us(ticks: i64, bpm: f64) -> i64 {
    if bpm <= 0.0 {
        return 0;
    }
    let beats = ticks as f64 / 48.0; // 192 ticks / 4 beats = 48 ticks per beat
    ((beats * 60_000_000.0) / bpm) as i64
}

/// Compute time offset for a position within a measure
/// This is a simplified version that accounts for BPM changes and stops within the measure
#[allow(clippy::too_many_arguments)]
fn position_to_us(
    pos: f64,
    measure: u32,
    measure_len: f64,
    _measure_times: &[i64],
    _model: &BmsModel,
    _current_bpm: f64,
    bpm_events: &HashMap<u32, Vec<(f64, f64)>>,
    stop_events: &HashMap<u32, Vec<(f64, u16)>>,
    stop_defs: &HashMap<u16, i64>,
    _extended_bpms: &HashMap<u16, f64>,
) -> i64 {
    let mut bpm = _model.initial_bpm;

    // Find the BPM at the start of this measure by looking at all previous BPM changes
    for m in 0..measure {
        if let Some(evts) = bpm_events.get(&m) {
            for &(_, new_bpm) in evts {
                bpm = new_bpm;
            }
        }
    }

    // Process events within this measure up to pos
    let mut time_offset: i64 = 0;
    let mut prev_pos = 0.0;

    let mut timing: Vec<(f64, TimingEvent)> = Vec::new();
    if let Some(evts) = bpm_events.get(&measure) {
        for &(p, b) in evts {
            if p < pos {
                timing.push((p, TimingEvent::Bpm(b)));
            }
        }
    }
    if let Some(evts) = stop_events.get(&measure) {
        for &(p, id) in evts {
            if p < pos
                && let Some(&ticks) = stop_defs.get(&id)
            {
                timing.push((p, TimingEvent::Stop(ticks)));
            }
        }
    }
    timing.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

    for (p, event) in &timing {
        let delta_beats = (*p - prev_pos) * 4.0 * measure_len;
        time_offset += beats_to_us(delta_beats, bpm);
        match event {
            TimingEvent::Bpm(new_bpm) => bpm = *new_bpm,
            TimingEvent::Stop(ticks) => time_offset += ticks_to_us(*ticks, bpm),
        }
        prev_pos = *p;
    }

    let remaining_beats = (pos - prev_pos) * 4.0 * measure_len;
    time_offset += beats_to_us(remaining_beats, bpm);

    time_offset
}

/// Convert a base36-parsed value back to hex interpretation.
/// Channel 03 data is hex (00-FF), but parse_channel_data reads it as base36.
/// E.g., "B4" → base36: 11*36+4=400 → hex: 0xB4=180
fn base36_to_hex(val: u16) -> u16 {
    (val / 36) * 16 + (val % 36)
}

/// Parse base-36 two-character string to u16
fn parse_base36(s: &str) -> u16 {
    let bytes = s.as_bytes();
    if bytes.len() < 2 {
        return 0;
    }
    let high = base36_digit(bytes[0]);
    let low = base36_digit(bytes[1]);
    high * 36 + low
}

fn base36_digit(b: u8) -> u16 {
    match b {
        b'0'..=b'9' => (b - b'0') as u16,
        b'A'..=b'Z' => (b - b'A' + 10) as u16,
        b'a'..=b'z' => (b - b'a' + 10) as u16,
        _ => 0,
    }
}

/// Parse a channel line: #MMMCC:data
fn parse_channel_line(upper: &str) -> Option<ChannelEvent> {
    if upper.len() < 7 {
        return None;
    }

    let measure: u32 = upper[..3].parse().ok()?;
    let channel = parse_hex_channel(&upper[3..5])?;

    if upper.as_bytes()[5] != b':' {
        return None;
    }

    let data_str = &upper[6..];
    let data = parse_channel_data(data_str);

    Some(ChannelEvent {
        measure,
        channel,
        data,
    })
}

/// Parse channel identifier (base-16 for some, base-36 for others)
fn parse_hex_channel(s: &str) -> Option<u16> {
    let bytes = s.as_bytes();
    if bytes.len() < 2 {
        return None;
    }
    // Channel identifiers use hex-like notation
    let high = hex_or_base36_digit(bytes[0])? as u16;
    let low = hex_or_base36_digit(bytes[1])? as u16;
    Some(high * 16 + low)
}

fn hex_or_base36_digit(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'A'..=b'F' => Some(b - b'A' + 10),
        b'a'..=b'f' => Some(b - b'a' + 10),
        _ => None,
    }
}

/// Parse channel data string into (position, value) pairs
/// Data is a sequence of base-36 pairs: "01020300" = [(0.0, 01), (0.333, 02), (0.666, 03), (1.0, 00)]
fn parse_channel_data(data: &str) -> Vec<(f64, u16)> {
    let data = data.trim();
    if data.len() < 2 {
        return Vec::new();
    }

    let count = data.len() / 2;
    let mut result = Vec::new();

    for i in 0..count {
        let s = &data[i * 2..i * 2 + 2];
        let val = parse_base36(s);
        if val > 0 {
            let pos = i as f64 / count as f64;
            result.push((pos, val));
        }
    }

    result
}

/// Note priority for deduplication: Normal/Invisible > LN > Mine
fn note_priority(n: &Note) -> u8 {
    match n.note_type {
        crate::note::NoteType::Normal | crate::note::NoteType::Invisible => 2,
        crate::note::NoteType::LongNote
        | crate::note::NoteType::ChargeNote
        | crate::note::NoteType::HellChargeNote => 1,
        crate::note::NoteType::Mine => 0,
    }
}

/// Find BPM at a given time
fn bpm_at_time(time_us: i64, initial_bpm: f64, bpm_changes: &[BpmChange]) -> f64 {
    let mut bpm = initial_bpm;
    for change in bpm_changes {
        if change.time_us <= time_us {
            bpm = change.bpm;
        } else {
            break;
        }
    }
    bpm
}

/// Detect encoding and decode bytes to string
fn detect_encoding_and_decode(raw: &[u8]) -> String {
    // Check for UTF-8 BOM
    if raw.starts_with(&[0xEF, 0xBB, 0xBF]) {
        return String::from_utf8_lossy(&raw[3..]).into_owned();
    }

    // Try UTF-8 first
    if let Ok(s) = std::str::from_utf8(raw) {
        return s.to_string();
    }

    // Try Shift_JIS
    let (decoded, _, had_errors) = encoding_rs::SHIFT_JIS.decode(raw);
    if !had_errors {
        return decoded.into_owned();
    }

    // Try EUC-JP
    let (decoded, _, had_errors) = encoding_rs::EUC_JP.decode(raw);
    if !had_errors {
        return decoded.into_owned();
    }

    // Fallback to Shift_JIS with lossy conversion
    let (decoded, _, _) = encoding_rs::SHIFT_JIS.decode(raw);
    decoded.into_owned()
}

/// Simple PRNG for #RANDOM (not Java LCG, just for basic functionality)
fn simple_random(bound: i32) -> i32 {
    if bound <= 1 {
        return 1;
    }
    // Use a simple random for now; Java LCG reproduction is in bms-pattern
    use std::time::{SystemTime, UNIX_EPOCH};
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos();
    (seed as i32 % bound) + 1
}

/// Extract raw content for hash computation (strip comments, normalize)
fn raw_content_for_hash(content: &str) -> Vec<u8> {
    content.as_bytes().to_vec()
}

/// Compute MD5 and SHA256 hashes
fn compute_hashes(raw: &[u8], model: &mut BmsModel) {
    model.md5 = format!("{:x}", md5::compute(raw));

    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(raw);
    model.sha256 = format!("{:x}", hasher.finalize());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_base36() {
        assert_eq!(parse_base36("00"), 0);
        assert_eq!(parse_base36("01"), 1);
        assert_eq!(parse_base36("0Z"), 35);
        assert_eq!(parse_base36("10"), 36);
        assert_eq!(parse_base36("ZZ"), 35 * 36 + 35);
    }

    #[test]
    fn test_parse_channel_data() {
        let data = parse_channel_data("01020300");
        assert_eq!(data.len(), 3); // 00 is filtered out
        assert_eq!(data[0].1, 1); // 01
        assert_eq!(data[1].1, 2); // 02
        assert_eq!(data[2].1, 3); // 03
    }

    #[test]
    fn test_beats_to_us() {
        // 1 beat at 120 BPM = 500000 us (0.5s)
        assert_eq!(beats_to_us(1.0, 120.0), 500000);
        // 4 beats at 120 BPM = 2000000 us (2s)
        assert_eq!(beats_to_us(4.0, 120.0), 2000000);
    }

    #[test]
    fn test_ticks_to_us() {
        // 192 ticks = 4 beats at 120 BPM = 2000000 us
        assert_eq!(ticks_to_us(192, 120.0), 2000000);
        // 48 ticks = 1 beat at 120 BPM = 500000 us
        assert_eq!(ticks_to_us(48, 120.0), 500000);
    }

    #[test]
    fn test_detect_encoding_utf8() {
        let content = "UTF-8テスト".as_bytes();
        let result = detect_encoding_and_decode(content);
        assert!(result.contains("テスト"));
    }

    #[test]
    fn test_decode_minimal_7k() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../test-bms/minimal_7k.bms");
        if path.exists() {
            let model = BmsDecoder::decode(&path).unwrap();
            assert_eq!(model.title, "Minimal 7K Test");
            assert_eq!(model.artist, "brs-test");
            assert_eq!(model.genre, "Test");
            assert_eq!(model.initial_bpm, 120.0);
            assert_eq!(model.mode, PlayMode::Beat7K);
            assert!(model.total_notes() > 0, "should have notes");
        }
    }

    #[test]
    fn test_decode_bpm_change() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../test-bms/bpm_change.bms");
        if path.exists() {
            let model = BmsDecoder::decode(&path).unwrap();
            assert_eq!(model.initial_bpm, 120.0);
            assert!(!model.bpm_changes.is_empty(), "should have BPM changes");
        }
    }

    #[test]
    fn test_decode_stop_sequence() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../test-bms/stop_sequence.bms");
        if path.exists() {
            let model = BmsDecoder::decode(&path).unwrap();
            assert!(!model.stop_events.is_empty(), "should have STOP events");
        }
    }

    #[test]
    fn test_decode_longnote() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../test-bms/longnote_types.bms");
        if path.exists() {
            let model = BmsDecoder::decode(&path).unwrap();
            assert!(model.total_long_notes() > 0, "should have long notes");
        }
    }

    #[test]
    fn test_decode_mine_notes() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../test-bms/mine_notes.bms");
        if path.exists() {
            let model = BmsDecoder::decode(&path).unwrap();
            let mines = model
                .notes
                .iter()
                .filter(|n| n.note_type == crate::note::NoteType::Mine)
                .count();
            assert!(mines > 0, "should have mine notes");
        }
    }

    #[test]
    fn test_decode_encoding_sjis() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../test-bms/encoding_sjis.bms");
        if path.exists() {
            let model = BmsDecoder::decode(&path).unwrap();
            assert!(model.title.contains("Shift_JIS"));
        }
    }

    #[test]
    fn test_decode_14key_dp() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../test-bms/14key_dp.bms");
        if path.exists() {
            let model = BmsDecoder::decode(&path).unwrap();
            assert_eq!(model.mode, PlayMode::Beat14K);
            assert_eq!(model.player, 3);
        }
    }

    // --- LN pair integrity tests ---

    /// Helper to decode inline BMS content with a .bms dummy path
    fn decode_inline(content: &str) -> BmsModel {
        BmsDecoder::decode_str(content, Path::new("test.bms")).unwrap()
    }

    #[test]
    fn test_ln_pair_basic() {
        let bms = "\
#PLAYER 1
#BPM 120
#WAV01 test.wav
#00151:01000001
";
        let model = decode_inline(bms);
        let lns: Vec<&Note> = model.notes.iter().filter(|n| n.is_long_note()).collect();
        assert_eq!(lns.len(), 1, "should have exactly 1 LN");
        let ln = lns[0];
        assert!(
            ln.end_time_us > ln.time_us,
            "end_time_us must be after time_us"
        );
        assert!(ln.is_long_note());
        assert_eq!(ln.note_type, crate::note::NoteType::LongNote);
    }

    #[test]
    fn test_ln_pair_sequential() {
        // Two consecutive LNs on the same lane (ch51) across two measures
        let bms = "\
#PLAYER 1
#BPM 120
#WAV01 test.wav
#00151:01000001
#00251:01000001
";
        let model = decode_inline(bms);
        let lns: Vec<&Note> = model.notes.iter().filter(|n| n.is_long_note()).collect();
        assert_eq!(lns.len(), 2, "should have 2 LNs");
        // Second LN starts after first LN ends
        assert!(
            lns[1].time_us >= lns[0].end_time_us,
            "second LN should start at or after first LN ends"
        );
    }

    #[test]
    fn test_ln_pair_multi_lane() {
        // Simultaneous LNs on ch51 (lane 0) and ch52 (lane 1)
        let bms = "\
#PLAYER 1
#BPM 120
#WAV01 test.wav
#WAV02 test.wav
#00151:01000001
#00152:02000002
";
        let model = decode_inline(bms);
        let lns: Vec<&Note> = model.notes.iter().filter(|n| n.is_long_note()).collect();
        assert_eq!(lns.len(), 2, "should have 2 LNs on different lanes");
        let lanes: Vec<usize> = lns.iter().map(|n| n.lane).collect();
        assert!(
            lanes.contains(&0) || lanes.contains(&1),
            "should have LNs on distinct lanes"
        );
        assert_ne!(lns[0].lane, lns[1].lane, "LNs should be on different lanes");
    }

    #[test]
    fn test_ln_unclosed_dropped() {
        // Only a start marker with no end → should NOT produce an LN
        let bms = "\
#PLAYER 1
#BPM 120
#WAV01 test.wav
#00151:01
";
        let model = decode_inline(bms);
        let lns: Vec<&Note> = model.notes.iter().filter(|n| n.is_long_note()).collect();
        assert_eq!(lns.len(), 0, "unclosed LN should not produce a note");
    }

    #[test]
    fn test_ln_end_wav_id() {
        // Start wav=01, end wav=02 → end_wav_id should be 2
        let bms = "\
#PLAYER 1
#BPM 120
#WAV01 start.wav
#WAV02 end.wav
#00151:01000002
";
        let model = decode_inline(bms);
        let lns: Vec<&Note> = model.notes.iter().filter(|n| n.is_long_note()).collect();
        assert_eq!(lns.len(), 1, "should have 1 LN");
        assert_eq!(lns[0].wav_id, 1, "start wav_id should be 1");
        assert_eq!(lns[0].end_wav_id, 2, "end wav_id should be 2");
    }

    #[test]
    fn test_ln_type_charge_note() {
        let bms = "\
#PLAYER 1
#BPM 120
#LNTYPE 2
#WAV01 test.wav
#00151:01000001
";
        let model = decode_inline(bms);
        let lns: Vec<&Note> = model.notes.iter().filter(|n| n.is_long_note()).collect();
        assert_eq!(lns.len(), 1, "should have 1 LN");
        assert_eq!(lns[0].note_type, crate::note::NoteType::ChargeNote);
    }

    #[test]
    fn test_ln_type_hell_charge_note() {
        let bms = "\
#PLAYER 1
#BPM 120
#LNTYPE 3
#WAV01 test.wav
#00151:01000001
";
        let model = decode_inline(bms);
        let lns: Vec<&Note> = model.notes.iter().filter(|n| n.is_long_note()).collect();
        assert_eq!(lns.len(), 1, "should have 1 LN");
        assert_eq!(lns[0].note_type, crate::note::NoteType::HellChargeNote);
    }
}
