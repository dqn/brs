use crate::bms_model::{BMSModel, LNTYPE_LONGNOTE};
use crate::note::{TYPE_CHARGENOTE, TYPE_HELLCHARGENOTE, TYPE_UNDEFINED};
use crate::time_line::TimeLine;

pub const TOTALNOTES_ALL: i32 = 0;
pub const TOTALNOTES_KEY: i32 = 1;
pub const TOTALNOTES_LONG_KEY: i32 = 2;
pub const TOTALNOTES_SCRATCH: i32 = 3;
pub const TOTALNOTES_LONG_SCRATCH: i32 = 4;
pub const TOTALNOTES_MINE: i32 = 5;

pub fn get_total_notes(model: &BMSModel) -> i32 {
    get_total_notes_range(model, 0, i32::MAX)
}

pub fn get_total_notes_with_type(model: &BMSModel, note_type: i32) -> i32 {
    get_total_notes_full(model, 0, i32::MAX, note_type, 0)
}

pub fn get_total_notes_range(model: &BMSModel, start: i32, end: i32) -> i32 {
    get_total_notes_full(model, start, end, TOTALNOTES_ALL, 0)
}

pub fn get_total_notes_range_type(model: &BMSModel, start: i32, end: i32, note_type: i32) -> i32 {
    get_total_notes_full(model, start, end, note_type, 0)
}

pub fn get_total_notes_full(
    model: &BMSModel,
    start: i32,
    end: i32,
    note_type: i32,
    side: i32,
) -> i32 {
    let mode = match model.get_mode() {
        Some(m) => m,
        None => return 0,
    };
    if mode.player() == 1 && side == 2 {
        return 0;
    }
    let scratch_key = mode.scratch_key();
    let mode_key = mode.key();
    let mode_player = mode.player();

    let slane_len = scratch_key.len() / (if side == 0 { 1 } else { mode_player as usize });
    let mut slane = Vec::with_capacity(slane_len);
    let start_idx = if side == 2 { slane_len } else { 0 };
    let mut i = start_idx;
    let mut index = 0;
    while index < slane_len {
        slane.push(scratch_key[i]);
        i += 1;
        index += 1;
    }

    let nlane_len = ((mode_key - scratch_key.len() as i32)
        / (if side == 0 { 1 } else { mode_player })) as usize;
    let mut nlane = Vec::with_capacity(nlane_len);
    let mut i = 0i32;
    let mut index = 0;
    while index < nlane_len {
        if !mode.is_scratch_key(i) {
            nlane.push(i);
            index += 1;
        }
        i += 1;
    }

    let lntype = model.get_lntype();
    let mut count = 0;
    for tl in model.get_all_time_lines() {
        if tl.get_time() >= start && tl.get_time() < end {
            match note_type {
                TOTALNOTES_ALL => {
                    count += tl.get_total_notes_with_lntype(lntype);
                }
                TOTALNOTES_KEY => {
                    for &lane in &nlane {
                        if tl.exist_note_at(lane)
                            && let Some(note) = tl.get_note(lane)
                            && note.is_normal()
                        {
                            count += 1;
                        }
                    }
                }
                TOTALNOTES_LONG_KEY => {
                    for &lane in &nlane {
                        if tl.exist_note_at(lane)
                            && let Some(note) = tl.get_note(lane)
                            && note.is_long()
                        {
                            let ln_type = note.get_long_note_type();
                            if ln_type == TYPE_CHARGENOTE
                                || ln_type == TYPE_HELLCHARGENOTE
                                || (ln_type == TYPE_UNDEFINED && lntype != LNTYPE_LONGNOTE)
                                || !note.is_end()
                            {
                                count += 1;
                            }
                        }
                    }
                }
                TOTALNOTES_SCRATCH => {
                    for &lane in &slane {
                        if tl.exist_note_at(lane)
                            && let Some(note) = tl.get_note(lane)
                            && note.is_normal()
                        {
                            count += 1;
                        }
                    }
                }
                TOTALNOTES_LONG_SCRATCH => {
                    for &lane in &slane {
                        if let Some(note) = tl.get_note(lane)
                            && note.is_long()
                        {
                            let ln_type = note.get_long_note_type();
                            if ln_type == TYPE_CHARGENOTE
                                || ln_type == TYPE_HELLCHARGENOTE
                                || (ln_type == TYPE_UNDEFINED && lntype != LNTYPE_LONGNOTE)
                                || !note.is_end()
                            {
                                count += 1;
                            }
                        }
                    }
                }
                TOTALNOTES_MINE => {
                    for &lane in &nlane {
                        if tl.exist_note_at(lane)
                            && let Some(note) = tl.get_note(lane)
                            && note.is_mine()
                        {
                            count += 1;
                        }
                    }
                    for &lane in &slane {
                        if tl.exist_note_at(lane)
                            && let Some(note) = tl.get_note(lane)
                            && note.is_mine()
                        {
                            count += 1;
                        }
                    }
                }
                _ => {}
            }
        }
    }
    count
}

pub fn change_frequency(model: &mut BMSModel, freq: f32) {
    model.set_bpm(model.get_bpm() * (freq as f64));
    for tl in model.get_all_time_lines_mut() {
        tl.set_bpm(tl.get_bpm() * (freq as f64));
        tl.set_stop((tl.get_micro_stop() as f64 / (freq as f64)) as i64);
        tl.set_micro_time((tl.get_micro_time() as f64 / (freq as f64)) as i64);
    }
}

pub fn get_max_notes_per_time(model: &BMSModel, range: i32) -> f64 {
    let mut maxnotes: i32 = 0;
    let tl = model.get_all_time_lines();
    let lntype = model.get_lntype();
    for i in 0..tl.len() {
        let mut notes = 0;
        let mut j = i;
        while j < tl.len() && tl[j].get_time() < tl[i].get_time() + range {
            notes += tl[j].get_total_notes_with_lntype(lntype);
            j += 1;
        }
        maxnotes = if maxnotes < notes { notes } else { maxnotes };
    }
    maxnotes as f64
}

pub fn set_start_note_time(model: &mut BMSModel, starttime: i64) -> i64 {
    let mut margin_time: i64 = 0;
    for tl in model.get_all_time_lines() {
        if tl.get_milli_time() >= starttime {
            break;
        }
        if tl.exist_note() {
            margin_time = starttime - tl.get_milli_time();
            break;
        }
    }

    if margin_time > 0 {
        let first_bpm = model.get_all_time_lines()[0].get_bpm();
        let margin_section = (margin_time as f64) * first_bpm / 240000.0;
        for tl in model.get_all_time_lines_mut() {
            tl.set_section(tl.get_section() + margin_section);
            tl.set_micro_time(tl.get_micro_time() + margin_time * 1000);
        }

        let mode_key = model.get_mode().map(|m| m.key()).unwrap_or(0);
        let bpm = model.get_bpm();

        let mut old_timelines = model.take_all_time_lines();
        let mut new_timelines: Vec<TimeLine> = Vec::with_capacity(old_timelines.len() + 1);
        let mut first = TimeLine::new(0.0, 0, mode_key);
        first.set_bpm(bpm);
        new_timelines.push(first);
        new_timelines.append(&mut old_timelines);
        model.set_all_time_line(new_timelines);
    }

    margin_time
}
