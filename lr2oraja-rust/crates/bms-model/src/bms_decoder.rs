use std::collections::BTreeMap;
use std::io::{BufRead, BufReader};
use std::path::Path;

use md5::Md5;
use sha2::{Digest, Sha256};

use crate::bms_model::{BMSModel, JudgeRankType, LNTYPE_LONGNOTE, TotalType};
use crate::chart_decoder::{self, TimeLineCache};
use crate::chart_information::ChartInformation;
use crate::decode_log::{DecodeLog, State};
use crate::mode::Mode;
use crate::section::{self, Section, f64_to_key};
use crate::time_line::TimeLine;

pub struct BMSDecoder {
    pub lntype: i32,
    pub log: Vec<DecodeLog>,
    wavlist: Vec<String>,
    wm: Vec<i32>,
    bgalist: Vec<String>,
    bm: Vec<i32>,
    lines: Vec<Option<Vec<String>>>,
    scrolltable: BTreeMap<i32, f64>,
    stoptable: BTreeMap<i32, f64>,
    bpmtable: BTreeMap<i32, f64>,
}

impl Default for BMSDecoder {
    fn default() -> Self {
        Self::new()
    }
}

impl BMSDecoder {
    pub fn new() -> Self {
        Self::new_with_lntype(LNTYPE_LONGNOTE)
    }

    pub fn new_with_lntype(lntype: i32) -> Self {
        BMSDecoder {
            lntype,
            log: Vec::new(),
            wavlist: Vec::with_capacity(62 * 62),
            wm: vec![-2; 62 * 62],
            bgalist: Vec::with_capacity(62 * 62),
            bm: vec![-2; 62 * 62],
            lines: Vec::new(),
            scrolltable: BTreeMap::new(),
            stoptable: BTreeMap::new(),
            bpmtable: BTreeMap::new(),
        }
    }

    pub fn decode_path(&mut self, f: &Path) -> Option<BMSModel> {
        log::debug!("BMSファイル解析開始 :{}", f.display());
        match std::fs::read(f) {
            Ok(data) => {
                let ispms = f.to_string_lossy().to_lowercase().ends_with(".pms");
                let model = self.decode_internal(Some(f), &data, ispms, None);
                if let Some(ref model) = model {
                    log::debug!(
                        "BMSファイル解析完了 :{} - TimeLine数:{}",
                        f.display(),
                        model.get_all_times().len()
                    );
                }
                model
            }
            Err(_) => {
                self.log
                    .push(DecodeLog::new(State::Error, "BMSファイルが見つかりません"));
                None
            }
        }
    }

    pub fn decode(&mut self, info: ChartInformation) -> Option<BMSModel> {
        self.lntype = info.lntype;
        let path = info.path.clone();
        let selected_randoms = info.selected_randoms.clone();
        match path {
            Some(ref p) => match std::fs::read(p) {
                Ok(data) => {
                    let ispms = p.to_string_lossy().to_lowercase().ends_with(".pms");
                    self.decode_internal(Some(p), &data, ispms, selected_randoms.as_deref())
                }
                Err(_) => {
                    self.log
                        .push(DecodeLog::new(State::Error, "BMSファイルが見つかりません"));
                    None
                }
            },
            None => None,
        }
    }

    pub fn decode_bytes(
        &mut self,
        data: &[u8],
        ispms: bool,
        random: Option<&[i32]>,
    ) -> Option<BMSModel> {
        self.decode_internal(None, data, ispms, random)
    }

    fn decode_internal(
        &mut self,
        path: Option<&Path>,
        data: &[u8],
        ispms: bool,
        selected_random: Option<&[i32]>,
    ) -> Option<BMSModel> {
        self.log.clear();
        let mut model = BMSModel::new();
        self.scrolltable.clear();
        self.stoptable.clear();
        self.bpmtable.clear();

        // Compute MD5 and SHA256
        let mut md5_hasher = Md5::new();
        let mut sha256_hasher = Sha256::new();
        md5_hasher.update(data);
        sha256_hasher.update(data);

        let mut maxsec: usize = 0;

        // Decode MS932 (Shift_JIS) to string
        let (decoded, _, _) = encoding_rs::SHIFT_JIS.decode(data);
        let text = decoded.into_owned();

        model.set_mode(if ispms { Mode::POPN_9K } else { Mode::BEAT_5K });

        self.wavlist.clear();
        for v in self.wm.iter_mut() {
            *v = -2;
        }
        self.bgalist.clear();
        for v in self.bm.iter_mut() {
            *v = -2;
        }

        // Ensure lines has 1000 slots
        self.lines.clear();
        self.lines.resize_with(1000, || None);

        let mut randoms: Vec<i32> = Vec::new();
        let mut srandoms: Vec<i32> = Vec::new();
        let mut crandom: Vec<i32> = Vec::new();
        let mut skip: Vec<bool> = Vec::new();

        let reader = BufReader::new(text.as_bytes());
        for line_result in reader.lines() {
            let line = match line_result {
                Ok(l) => l,
                Err(_) => continue,
            };
            if line.len() < 2 {
                continue;
            }

            let first_char = line.as_bytes()[0] as char;
            if first_char == '#' {
                if matches_reserve_word(&line, "RANDOM") {
                    match line[8..].trim().parse::<i32>() {
                        Ok(r) => {
                            randoms.push(r);
                            if let Some(sr) = selected_random {
                                if randoms.len() - 1 < sr.len() {
                                    crandom.push(sr[randoms.len() - 1]);
                                } else {
                                    crandom.push((rand_f64() * (r as f64)) as i32 + 1);
                                    srandoms.push(*crandom.last().unwrap());
                                }
                            } else {
                                crandom.push((rand_f64() * (r as f64)) as i32 + 1);
                                srandoms.push(*crandom.last().unwrap());
                            }
                        }
                        Err(_) => {
                            self.log.push(DecodeLog::new(
                                State::Warning,
                                "#RANDOMに数字が定義されていません",
                            ));
                        }
                    }
                } else if matches_reserve_word(&line, "IF") {
                    if !crandom.is_empty() {
                        match line[4..].trim().parse::<i32>() {
                            Ok(val) => {
                                skip.push(*crandom.last().unwrap() != val);
                            }
                            Err(_) => {
                                self.log.push(DecodeLog::new(
                                    State::Warning,
                                    "#IFに数字が定義されていません",
                                ));
                            }
                        }
                    } else {
                        self.log.push(DecodeLog::new(
                            State::Warning,
                            "#IFに対応する#RANDOMが定義されていません",
                        ));
                    }
                } else if matches_reserve_word(&line, "ENDIF") {
                    if !skip.is_empty() {
                        skip.pop();
                    } else {
                        self.log.push(DecodeLog::new(
                            State::Warning,
                            format!("ENDIFに対応するIFが存在しません: {}", line),
                        ));
                    }
                } else if matches_reserve_word(&line, "ENDRANDOM") {
                    if !crandom.is_empty() {
                        crandom.pop();
                    } else {
                        self.log.push(DecodeLog::new(
                            State::Warning,
                            format!("ENDRANDOMに対応するRANDOMが存在しません: {}", line),
                        ));
                    }
                } else if skip.is_empty() || !*skip.last().unwrap() {
                    let c = line.as_bytes()[1] as char;
                    let base = model.get_base();
                    if ('0'..='9').contains(&c) && line.len() > 6 {
                        let c2 = line.as_bytes()[2] as char;
                        let c3 = line.as_bytes()[3] as char;
                        if ('0'..='9').contains(&c2) && ('0'..='9').contains(&c3) {
                            let bar_index = ((c as usize) - ('0' as usize)) * 100
                                + ((c2 as usize) - ('0' as usize)) * 10
                                + ((c3 as usize) - ('0' as usize));
                            if bar_index < 1000 {
                                if self.lines[bar_index].is_none() {
                                    self.lines[bar_index] = Some(Vec::new());
                                }
                                self.lines[bar_index].as_mut().unwrap().push(line.clone());
                                maxsec = if maxsec > bar_index {
                                    maxsec
                                } else {
                                    bar_index
                                };
                            }
                        } else {
                            self.log.push(DecodeLog::new(
                                State::Warning,
                                format!("小節に数字が定義されていません : {}", line),
                            ));
                        }
                    } else if matches_reserve_word(&line, "BPM") {
                        if line.len() > 4 && line.as_bytes()[4] == b' ' {
                            match line[5..].trim().parse::<f64>() {
                                Ok(bpm) => {
                                    if bpm > 0.0 {
                                        model.set_bpm(bpm);
                                    } else {
                                        self.log.push(DecodeLog::new(
                                            State::Warning,
                                            format!(
                                                "#negative BPMはサポートされていません : {}",
                                                line
                                            ),
                                        ));
                                    }
                                }
                                Err(_) => {
                                    self.log.push(DecodeLog::new(
                                        State::Warning,
                                        format!("#BPMに数字が定義されていません : {}", line),
                                    ));
                                }
                            }
                        } else if line.len() > 7 {
                            match line[7..].trim().parse::<f64>() {
                                Ok(bpm) => {
                                    if bpm > 0.0 {
                                        if base == 62 {
                                            match chart_decoder::parse_int62_str(&line, 4) {
                                                Ok(idx) => {
                                                    self.bpmtable.insert(idx, bpm);
                                                }
                                                Err(_) => {
                                                    self.log.push(DecodeLog::new(
                                                        State::Warning,
                                                        format!(
                                                            "#BPMxxに数字が定義されていません : {}",
                                                            line
                                                        ),
                                                    ));
                                                }
                                            }
                                        } else {
                                            match chart_decoder::parse_int36_str(&line, 4) {
                                                Ok(idx) => {
                                                    self.bpmtable.insert(idx, bpm);
                                                }
                                                Err(_) => {
                                                    self.log.push(DecodeLog::new(
                                                        State::Warning,
                                                        format!(
                                                            "#BPMxxに数字が定義されていません : {}",
                                                            line
                                                        ),
                                                    ));
                                                }
                                            }
                                        }
                                    } else {
                                        self.log.push(DecodeLog::new(
                                            State::Warning,
                                            format!(
                                                "#negative BPMはサポートされていません : {}",
                                                line
                                            ),
                                        ));
                                    }
                                }
                                Err(_) => {
                                    self.log.push(DecodeLog::new(
                                        State::Warning,
                                        format!("#BPMxxに数字が定義されていません : {}", line),
                                    ));
                                }
                            }
                        }
                    } else if matches_reserve_word(&line, "WAV") {
                        if line.len() >= 8 {
                            let parse_result = if base == 62 {
                                chart_decoder::parse_int62_str(&line, 4)
                            } else {
                                chart_decoder::parse_int36_str(&line, 4)
                            };
                            match parse_result {
                                Ok(idx) => {
                                    let file_name = line[7..].trim().replace('\\', "/");
                                    if (idx as usize) < self.wm.len() {
                                        self.wm[idx as usize] = self.wavlist.len() as i32;
                                    }
                                    self.wavlist.push(file_name);
                                }
                                Err(_) => {
                                    self.log.push(DecodeLog::new(
                                        State::Warning,
                                        format!("#WAVxxは不十分な定義です : {}", line),
                                    ));
                                }
                            }
                        } else {
                            self.log.push(DecodeLog::new(
                                State::Warning,
                                format!("#WAVxxは不十分な定義です : {}", line),
                            ));
                        }
                    } else if matches_reserve_word(&line, "BMP") {
                        if line.len() >= 8 {
                            let parse_result = if base == 62 {
                                chart_decoder::parse_int62_str(&line, 4)
                            } else {
                                chart_decoder::parse_int36_str(&line, 4)
                            };
                            match parse_result {
                                Ok(idx) => {
                                    let file_name = line[7..].trim().replace('\\', "/");
                                    if (idx as usize) < self.bm.len() {
                                        self.bm[idx as usize] = self.bgalist.len() as i32;
                                    }
                                    self.bgalist.push(file_name);
                                }
                                Err(_) => {
                                    self.log.push(DecodeLog::new(
                                        State::Warning,
                                        format!("#BMPxxは不十分な定義です : {}", line),
                                    ));
                                }
                            }
                        } else {
                            self.log.push(DecodeLog::new(
                                State::Warning,
                                format!("#BMPxxは不十分な定義です : {}", line),
                            ));
                        }
                    } else if matches_reserve_word(&line, "STOP") {
                        if line.len() >= 9 {
                            let parse_result = if base == 62 {
                                chart_decoder::parse_int62_str(&line, 5)
                            } else {
                                chart_decoder::parse_int36_str(&line, 5)
                            };
                            match parse_result {
                                Ok(idx) => match line[8..].trim().parse::<f64>() {
                                    Ok(mut stop) => {
                                        stop /= 192.0;
                                        if stop < 0.0 {
                                            stop = stop.abs();
                                            self.log.push(DecodeLog::new(
                                                State::Warning,
                                                format!(
                                                    "#negative STOPはサポートされていません : {}",
                                                    line
                                                ),
                                            ));
                                        }
                                        self.stoptable.insert(idx, stop);
                                    }
                                    Err(_) => {
                                        self.log.push(DecodeLog::new(
                                            State::Warning,
                                            format!("#STOPxxに数字が定義されていません : {}", line),
                                        ));
                                    }
                                },
                                Err(_) => {
                                    self.log.push(DecodeLog::new(
                                        State::Warning,
                                        format!("#STOPxxに数字が定義されていません : {}", line),
                                    ));
                                }
                            }
                        } else {
                            self.log.push(DecodeLog::new(
                                State::Warning,
                                format!("#STOPxxは不十分な定義です : {}", line),
                            ));
                        }
                    } else if matches_reserve_word(&line, "SCROLL") {
                        if line.len() >= 11 {
                            let parse_result = if base == 62 {
                                chart_decoder::parse_int62_str(&line, 7)
                            } else {
                                chart_decoder::parse_int36_str(&line, 7)
                            };
                            match parse_result {
                                Ok(idx) => match line[10..].trim().parse::<f64>() {
                                    Ok(scroll) => {
                                        self.scrolltable.insert(idx, scroll);
                                    }
                                    Err(_) => {
                                        self.log.push(DecodeLog::new(
                                            State::Warning,
                                            format!(
                                                "#SCROLLxxに数字が定義されていません : {}",
                                                line
                                            ),
                                        ));
                                    }
                                },
                                Err(_) => {
                                    self.log.push(DecodeLog::new(
                                        State::Warning,
                                        format!("#SCROLLxxに数字が定義されていません : {}", line),
                                    ));
                                }
                            }
                        } else {
                            self.log.push(DecodeLog::new(
                                State::Warning,
                                format!("#SCROLLxxは不十分な定義です : {}", line),
                            ));
                        }
                    } else {
                        // Command words
                        let handled = process_command_word(&line, &mut model, &mut self.log);
                        let _ = handled;
                    }
                }
            } else if first_char == '%' {
                if let Some(index) = line.find(' ')
                    && line.len() > index + 1
                {
                    model
                        .get_values_mut()
                        .insert(line[1..index].to_string(), line[index + 1..].to_string());
                }
            } else if first_char == '@'
                && let Some(index) = line.find(' ')
                && line.len() > index + 1
            {
                model
                    .get_values_mut()
                    .insert(line[1..index].to_string(), line[index + 1..].to_string());
            }
        }

        model.set_wav_list(self.wavlist.clone());
        model.set_bga_list(self.bgalist.clone());

        let mut sections: Vec<Section> = Vec::with_capacity(maxsec + 1);
        let mut prev_sectionnum: f64 = 0.0;
        let mut prev_rate: f64 = 1.0;
        for i in 0..=maxsec {
            let empty_lines: Vec<String> = Vec::new();
            let lines_ref = self.lines[i].as_deref().unwrap_or(&empty_lines);
            let is_first = i == 0;
            let section = Section::new(
                &mut model,
                prev_sectionnum,
                prev_rate,
                is_first,
                lines_ref,
                &self.bpmtable,
                &self.stoptable,
                &self.scrolltable,
                &mut self.log,
            );
            prev_sectionnum = section.get_sectionnum();
            prev_rate = section.get_rate();
            sections.push(section);
        }

        let mode_key = model.get_mode().map(|m| m.key()).unwrap_or(0);
        let mut tlcache: BTreeMap<u64, TimeLineCache> = BTreeMap::new();
        let mut lnlist: Vec<Option<Vec<section::LnInfo>>> = vec![None; mode_key as usize];
        let mut lnendstatus: Vec<Option<section::StartLnInfo>> = vec![None; mode_key as usize];
        let basetl = TimeLine::new(0.0, 0, mode_key);
        let mut basetl = basetl;
        basetl.set_bpm(model.get_bpm());
        tlcache.insert(f64_to_key(0.0), TimeLineCache::new(0.0, basetl));

        for section in &sections {
            section.make_time_lines(
                &mut model,
                &self.wm,
                &self.bm,
                &mut tlcache,
                &mut lnlist,
                &mut lnendstatus,
                &mut self.log,
            );
        }

        let tl_vec: Vec<TimeLine> = tlcache.into_values().map(|tlc| tlc.timeline).collect();
        model.set_all_time_line(tl_vec);

        let all_tl = model.get_all_time_lines();
        if !all_tl.is_empty() && all_tl[0].get_bpm() == 0.0 {
            self.log.push(DecodeLog::new(
                State::Error,
                "開始BPMが定義されていないため、BMS解析に失敗しました",
            ));
            return None;
        }

        for i in 0..lnendstatus.len() {
            if let Some(ref status) = lnendstatus[i] {
                self.log.push(DecodeLog::new(
                    State::Warning,
                    format!(
                        "曲の終端までにLN終端定義されていないLNがあります。lane:{}",
                        i + 1
                    ),
                ));
                if status.section != f64::MIN {
                    // Find the timeline in model's timelines and clear the note
                    for tl in model.get_all_time_lines_mut() {
                        if tl.get_section() == status.section {
                            tl.set_note(i as i32, None);
                            break;
                        }
                    }
                }
            }
        }

        if *model.get_total_type() != TotalType::Bms {
            self.log
                .push(DecodeLog::new(State::Warning, "TOTALが未定義です"));
        }
        if model.get_total() <= 60.0 {
            self.log
                .push(DecodeLog::new(State::Warning, "TOTAL値が少なすぎます"));
        }
        let all_tl = model.get_all_time_lines();
        if !all_tl.is_empty()
            && all_tl[all_tl.len() - 1].get_time() >= model.get_last_time() + 30000
        {
            self.log.push(DecodeLog::new(
                State::Warning,
                "最後のノート定義から30秒以上の余白があります",
            ));
        }
        if model.get_player() > 1
            && (model.get_mode() == Some(&Mode::BEAT_5K)
                || model.get_mode() == Some(&Mode::BEAT_7K))
        {
            self.log.push(DecodeLog::new(
                State::Warning,
                "#PLAYER定義が2以上にもかかわらず2P側のノーツ定義が一切ありません",
            ));
        }
        if model.get_player() == 1
            && (model.get_mode() == Some(&Mode::BEAT_10K)
                || model.get_mode() == Some(&Mode::BEAT_14K))
        {
            self.log.push(DecodeLog::new(
                State::Warning,
                "#PLAYER定義が1にもかかわらず2P側のノーツ定義が存在します",
            ));
        }

        let md5_result = md5_hasher.finalize();
        let sha256_result = sha256_hasher.finalize();
        model.set_md5(convert_hex_string(&md5_result));
        model.set_sha256(convert_hex_string(&sha256_result));

        self.log.push(DecodeLog::new(
            State::Info,
            "#PLAYER定義が1にもかかわらず2P側のノーツ定義が存在します",
        ));

        let final_selected_random = if let Some(sr) = selected_random {
            sr.to_vec()
        } else {
            srandoms.clone()
        };

        model.set_chart_information(ChartInformation::new(
            path.map(|p| p.to_path_buf()),
            self.lntype,
            Some(final_selected_random),
        ));

        if let Some(p) = path {
            self.print_log(p);
        }

        Some(model)
    }

    fn print_log(&self, path: &Path) {
        for l in &self.log {
            match l.state {
                State::Info => {
                    log::info!("{} : {}", path.display(), l.message);
                }
                State::Warning => {
                    log::warn!("{} : {}", path.display(), l.message);
                }
                State::Error => {
                    log::error!("{} : {}", path.display(), l.message);
                }
            }
        }
    }
}

fn matches_reserve_word(line: &str, s: &str) -> bool {
    let len = s.len();
    if line.len() <= len {
        return false;
    }
    let line_bytes = line.as_bytes();
    let s_bytes = s.as_bytes();
    for i in 0..len {
        let c = line_bytes[i + 1];
        let c2 = s_bytes[i];
        if c != c2 && c != c2 + 32 {
            return false;
        }
    }
    true
}

pub fn convert_hex_string(data: &[u8]) -> String {
    let mut sb = String::with_capacity(data.len() * 2);
    for &b in data {
        sb.push(char::from_digit(((b >> 4) & 0xf) as u32, 16).unwrap());
        sb.push(char::from_digit((b & 0xf) as u32, 16).unwrap());
    }
    sb
}

fn rand_f64() -> f64 {
    // Simple random - use system time as seed
    use std::time::SystemTime;
    let nanos = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos();
    (nanos as f64) / 1_000_000_000.0
}

fn process_command_word(line: &str, model: &mut BMSModel, log: &mut Vec<DecodeLog>) -> bool {
    struct CmdDef {
        name: &'static str,
        handler: fn(&mut BMSModel, &str) -> Option<DecodeLog>,
    }

    let commands: &[CmdDef] = &[
        CmdDef {
            name: "PLAYER",
            handler: |model, arg| {
                match arg.parse::<i32>() {
                    Ok(player) => {
                        if (1..3).contains(&player) {
                            model.set_player(player);
                        } else {
                            return Some(DecodeLog::new(
                                State::Warning,
                                format!("#PLAYERに規定外の数字が定義されています : {}", player),
                            ));
                        }
                    }
                    Err(_) => {
                        return Some(DecodeLog::new(
                            State::Warning,
                            "#PLAYERに数字が定義されていません",
                        ));
                    }
                }
                None
            },
        },
        CmdDef {
            name: "GENRE",
            handler: |model, arg| {
                model.set_genre(arg);
                None
            },
        },
        CmdDef {
            name: "TITLE",
            handler: |model, arg| {
                model.set_title(arg);
                None
            },
        },
        CmdDef {
            name: "SUBTITLE",
            handler: |model, arg| {
                model.set_sub_title(arg);
                None
            },
        },
        CmdDef {
            name: "ARTIST",
            handler: |model, arg| {
                model.set_artist(arg);
                None
            },
        },
        CmdDef {
            name: "SUBARTIST",
            handler: |model, arg| {
                model.set_sub_artist(arg);
                None
            },
        },
        CmdDef {
            name: "PLAYLEVEL",
            handler: |model, arg| {
                model.set_playlevel(arg);
                None
            },
        },
        CmdDef {
            name: "RANK",
            handler: |model, arg| {
                match arg.parse::<i32>() {
                    Ok(rank) => {
                        if (0..5).contains(&rank) {
                            model.set_judgerank(rank);
                            model.set_judgerank_type(JudgeRankType::BmsRank);
                        } else {
                            return Some(DecodeLog::new(
                                State::Warning,
                                format!("#RANKに規定外の数字が定義されています : {}", rank),
                            ));
                        }
                    }
                    Err(_) => {
                        return Some(DecodeLog::new(
                            State::Warning,
                            "#RANKに数字が定義されていません",
                        ));
                    }
                }
                None
            },
        },
        CmdDef {
            name: "DEFEXRANK",
            handler: |model, arg| {
                match arg.parse::<i32>() {
                    Ok(rank) => {
                        if rank >= 1 {
                            model.set_judgerank(rank);
                            model.set_judgerank_type(JudgeRankType::BmsDefexrank);
                        } else {
                            return Some(DecodeLog::new(
                                State::Warning,
                                format!("#DEFEXRANK 1以下はサポートしていません{}", rank),
                            ));
                        }
                    }
                    Err(_) => {
                        return Some(DecodeLog::new(
                            State::Warning,
                            "#DEFEXRANKに数字が定義されていません",
                        ));
                    }
                }
                None
            },
        },
        CmdDef {
            name: "TOTAL",
            handler: |model, arg| {
                match arg.parse::<f64>() {
                    Ok(total) => {
                        if total > 0.0 {
                            model.set_total(total);
                            model.set_total_type(TotalType::Bms);
                        } else {
                            return Some(DecodeLog::new(State::Warning, "#TOTALが0以下です"));
                        }
                    }
                    Err(_) => {
                        return Some(DecodeLog::new(
                            State::Warning,
                            "#TOTALに数字が定義されていません",
                        ));
                    }
                }
                None
            },
        },
        CmdDef {
            name: "VOLWAV",
            handler: |model, arg| {
                match arg.parse::<i32>() {
                    Ok(v) => {
                        model.set_volwav(v);
                    }
                    Err(_) => {
                        return Some(DecodeLog::new(
                            State::Warning,
                            "#VOLWAVに数字が定義されていません",
                        ));
                    }
                }
                None
            },
        },
        CmdDef {
            name: "STAGEFILE",
            handler: |model, arg| {
                model.set_stagefile(arg.replace('\\', "/"));
                None
            },
        },
        CmdDef {
            name: "BACKBMP",
            handler: |model, arg| {
                model.set_backbmp(arg.replace('\\', "/"));
                None
            },
        },
        CmdDef {
            name: "PREVIEW",
            handler: |model, arg| {
                model.set_preview(arg.replace('\\', "/"));
                None
            },
        },
        CmdDef {
            name: "LNOBJ",
            handler: |model, arg| {
                if model.get_base() == 62 {
                    match chart_decoder::parse_int62_str(arg, 0) {
                        Ok(v) => model.set_lnobj(v),
                        Err(_) => {
                            return Some(DecodeLog::new(
                                State::Warning,
                                "#LNOBJに数字が定義されていません",
                            ));
                        }
                    }
                } else {
                    match i32::from_str_radix(&arg.to_uppercase(), 36) {
                        Ok(v) => model.set_lnobj(v),
                        Err(_) => {
                            return Some(DecodeLog::new(
                                State::Warning,
                                "#LNOBJに数字が定義されていません",
                            ));
                        }
                    }
                }
                None
            },
        },
        CmdDef {
            name: "LNMODE",
            handler: |model, arg| {
                match arg.parse::<i32>() {
                    Ok(mut lnmode) => {
                        if !(0..=3).contains(&lnmode) {
                            return Some(DecodeLog::new(
                                State::Warning,
                                "#LNMODEに無効な数字が定義されています",
                            ));
                        }
                        // LR2oraja Endless Dream: LR2 does not support LNMODE, suppress modes 1 or 2
                        lnmode = 0;
                        model.set_lnmode(lnmode);
                    }
                    Err(_) => {
                        return Some(DecodeLog::new(
                            State::Warning,
                            "#LNMODEに数字が定義されていません",
                        ));
                    }
                }
                None
            },
        },
        CmdDef {
            name: "DIFFICULTY",
            handler: |model, arg| {
                match arg.parse::<i32>() {
                    Ok(v) => model.set_difficulty(v),
                    Err(_) => {
                        return Some(DecodeLog::new(
                            State::Warning,
                            "#DIFFICULTYに数字が定義されていません",
                        ));
                    }
                }
                None
            },
        },
        CmdDef {
            name: "BANNER",
            handler: |model, arg| {
                model.set_banner(arg.replace('\\', "/"));
                None
            },
        },
        CmdDef {
            name: "COMMENT",
            handler: |_model, _arg| {
                // TODO: not implemented
                None
            },
        },
        CmdDef {
            name: "BASE",
            handler: |model, arg| {
                match arg.parse::<i32>() {
                    Ok(base) => {
                        if base != 62 {
                            return Some(DecodeLog::new(
                                State::Warning,
                                "#BASEに無効な数字が定義されています",
                            ));
                        }
                        model.set_base(base);
                    }
                    Err(_) => {
                        return Some(DecodeLog::new(
                            State::Warning,
                            "#BASEに数字が定義されていません",
                        ));
                    }
                }
                None
            },
        },
    ];

    for cmd in commands {
        if line.len() > cmd.name.len() + 2 && matches_reserve_word(line, cmd.name) {
            let arg = line[cmd.name.len() + 2..].trim();
            let result = (cmd.handler)(model, arg);
            if let Some(dl) = result {
                log.push(dl);
            }
            return true;
        }
    }
    false
}
