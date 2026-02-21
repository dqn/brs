use std::collections::HashMap;

use crate::chart_information::ChartInformation;
use crate::mode::Mode;
use crate::note::Note;
use crate::time_line::TimeLine;

pub const LNTYPE_LONGNOTE: i32 = 0;
pub const LNTYPE_CHARGENOTE: i32 = 1;
pub const LNTYPE_HELLCHARGENOTE: i32 = 2;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum JudgeRankType {
    BmsRank,
    BmsDefexrank,
    BmsonJudgerank,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TotalType {
    Bms,
    Bmson,
}

pub struct BMSModel {
    player: i32,
    mode: Option<Mode>,
    title: String,
    sub_title: String,
    genre: String,
    artist: String,
    subartist: String,
    banner: String,
    stagefile: String,
    backbmp: String,
    preview: String,
    bpm: f64,
    playlevel: String,
    difficulty: i32,
    judgerank: i32,
    judgerank_type: JudgeRankType,
    total: f64,
    total_type: TotalType,
    volwav: i32,
    md5: String,
    sha256: String,
    wavmap: Vec<String>,
    bgamap: Vec<String>,
    base: i32,
    lnmode: i32,
    lnobj: i32,
    from_osu: bool,
    timelines: Vec<TimeLine>,
    info: Option<ChartInformation>,
    values: HashMap<String, String>,
}

impl Default for BMSModel {
    fn default() -> Self {
        Self::new()
    }
}

impl BMSModel {
    pub fn new() -> Self {
        BMSModel {
            player: 0,
            mode: None,
            title: String::new(),
            sub_title: String::new(),
            genre: String::new(),
            artist: String::new(),
            subartist: String::new(),
            banner: String::new(),
            stagefile: String::new(),
            backbmp: String::new(),
            preview: String::new(),
            bpm: 0.0,
            playlevel: String::new(),
            difficulty: 0,
            judgerank: 2,
            judgerank_type: JudgeRankType::BmsRank,
            total: 100.0,
            total_type: TotalType::Bmson,
            volwav: 0,
            md5: String::new(),
            sha256: String::new(),
            wavmap: Vec::new(),
            bgamap: Vec::new(),
            base: 36,
            lnmode: crate::note::TYPE_UNDEFINED,
            lnobj: -1,
            from_osu: false,
            timelines: Vec::new(),
            info: None,
            values: HashMap::new(),
        }
    }

    pub fn get_player(&self) -> i32 {
        self.player
    }

    pub fn set_player(&mut self, player: i32) {
        self.player = player;
    }

    pub fn get_title(&self) -> &str {
        &self.title
    }

    pub fn set_title(&mut self, title: impl Into<String>) {
        let t: String = title.into();
        self.title = t;
    }

    pub fn get_sub_title(&self) -> &str {
        &self.sub_title
    }

    pub fn set_sub_title(&mut self, sub_title: impl Into<String>) {
        let t: String = sub_title.into();
        self.sub_title = t;
    }

    pub fn get_genre(&self) -> &str {
        &self.genre
    }

    pub fn set_genre(&mut self, genre: impl Into<String>) {
        let t: String = genre.into();
        self.genre = t;
    }

    pub fn get_artist(&self) -> &str {
        &self.artist
    }

    pub fn set_artist(&mut self, artist: impl Into<String>) {
        let t: String = artist.into();
        self.artist = t;
    }

    pub fn get_sub_artist(&self) -> &str {
        &self.subartist
    }

    pub fn set_sub_artist(&mut self, artist: impl Into<String>) {
        let t: String = artist.into();
        self.subartist = t;
    }

    pub fn set_banner(&mut self, banner: impl Into<String>) {
        let t: String = banner.into();
        self.banner = t;
    }

    pub fn get_banner(&self) -> &str {
        &self.banner
    }

    pub fn get_bpm(&self) -> f64 {
        self.bpm
    }

    pub fn set_bpm(&mut self, bpm: f64) {
        self.bpm = bpm;
    }

    pub fn get_playlevel(&self) -> &str {
        &self.playlevel
    }

    pub fn set_playlevel(&mut self, playlevel: impl Into<String>) {
        self.playlevel = playlevel.into();
    }

    pub fn get_judgerank(&self) -> i32 {
        self.judgerank
    }

    pub fn set_judgerank(&mut self, judgerank: i32) {
        self.judgerank = judgerank;
    }

    pub fn get_total(&self) -> f64 {
        self.total
    }

    pub fn set_total(&mut self, total: f64) {
        self.total = total;
    }

    pub fn get_volwav(&self) -> i32 {
        self.volwav
    }

    pub fn set_volwav(&mut self, volwav: i32) {
        self.volwav = volwav;
    }

    pub fn get_min_bpm(&self) -> f64 {
        let mut bpm = self.get_bpm();
        for time in &self.timelines {
            let d = time.get_bpm();
            bpm = if bpm <= d { bpm } else { d };
        }
        bpm
    }

    pub fn get_max_bpm(&self) -> f64 {
        let mut bpm = self.get_bpm();
        for time in &self.timelines {
            let d = time.get_bpm();
            bpm = if bpm >= d { bpm } else { d };
        }
        bpm
    }

    pub fn set_all_time_line(&mut self, timelines: Vec<TimeLine>) {
        self.timelines = timelines;
    }

    pub fn get_all_time_lines(&self) -> &[TimeLine] {
        &self.timelines
    }

    pub fn get_all_time_lines_mut(&mut self) -> &mut [TimeLine] {
        &mut self.timelines
    }

    pub fn take_all_time_lines(&mut self) -> Vec<TimeLine> {
        std::mem::take(&mut self.timelines)
    }

    pub fn get_all_times(&self) -> Vec<i64> {
        let times = self.get_all_time_lines();
        let mut result = Vec::with_capacity(times.len());
        for tl in times {
            result.push(tl.get_time() as i64);
        }
        result
    }

    pub fn get_last_time(&self) -> i32 {
        self.get_last_milli_time() as i32
    }

    pub fn get_last_milli_time(&self) -> i64 {
        let keys = self.mode.as_ref().map(|m| m.key()).unwrap_or(0);
        for i in (0..self.timelines.len()).rev() {
            let tl = &self.timelines[i];
            for lane in 0..keys {
                if tl.exist_note_at(lane)
                    || tl.get_hidden_note(lane).is_some()
                    || !tl.get_back_ground_notes().is_empty()
                    || tl.get_bga() != -1
                    || tl.get_layer() != -1
                {
                    return tl.get_milli_time();
                }
            }
        }
        0
    }

    pub fn get_last_note_time(&self) -> i32 {
        self.get_last_note_milli_time() as i32
    }

    pub fn get_last_note_milli_time(&self) -> i64 {
        let keys = self.mode.as_ref().map(|m| m.key()).unwrap_or(0);
        for i in (0..self.timelines.len()).rev() {
            let tl = &self.timelines[i];
            for lane in 0..keys {
                if tl.exist_note_at(lane) {
                    return tl.get_milli_time();
                }
            }
        }
        0
    }

    pub fn get_difficulty(&self) -> i32 {
        self.difficulty
    }

    pub fn set_difficulty(&mut self, difficulty: i32) {
        self.difficulty = difficulty;
    }

    pub fn get_full_title(&self) -> String {
        let mut s = self.title.clone();
        if !self.sub_title.is_empty() {
            s.push(' ');
            s.push_str(&self.sub_title);
        }
        s
    }

    pub fn get_full_artist(&self) -> String {
        let mut s = self.artist.clone();
        if !self.subartist.is_empty() {
            s.push(' ');
            s.push_str(&self.subartist);
        }
        s
    }

    pub fn set_md5(&mut self, hash: impl Into<String>) {
        self.md5 = hash.into();
    }

    pub fn get_md5(&self) -> &str {
        &self.md5
    }

    pub fn get_sha256(&self) -> &str {
        &self.sha256
    }

    pub fn set_sha256(&mut self, sha256: impl Into<String>) {
        self.sha256 = sha256.into();
    }

    pub fn set_mode(&mut self, mode: Mode) {
        let key = mode.key();
        self.mode = Some(mode);
        for tl in &mut self.timelines {
            tl.set_lane_count(key);
        }
    }

    pub fn get_mode(&self) -> Option<&Mode> {
        self.mode.as_ref()
    }

    pub fn get_wav_list(&self) -> &[String] {
        &self.wavmap
    }

    pub fn set_wav_list(&mut self, wavmap: Vec<String>) {
        self.wavmap = wavmap;
    }

    pub fn get_bga_list(&self) -> &[String] {
        &self.bgamap
    }

    pub fn set_bga_list(&mut self, bgamap: Vec<String>) {
        self.bgamap = bgamap;
    }

    pub fn get_chart_information(&self) -> Option<&ChartInformation> {
        self.info.as_ref()
    }

    pub fn set_chart_information(&mut self, info: ChartInformation) {
        self.info = Some(info);
    }

    pub fn get_random(&self) -> Option<&[i32]> {
        self.info
            .as_ref()
            .and_then(|i| i.selected_randoms.as_deref())
    }

    pub fn get_path(&self) -> Option<String> {
        self.info
            .as_ref()
            .and_then(|i| i.path.as_ref())
            .map(|p| p.to_string_lossy().to_string())
    }

    pub fn get_lntype(&self) -> i32 {
        self.info
            .as_ref()
            .map(|i| i.lntype)
            .unwrap_or(LNTYPE_LONGNOTE)
    }

    pub fn get_stagefile(&self) -> &str {
        &self.stagefile
    }

    pub fn set_stagefile(&mut self, stagefile: impl Into<String>) {
        let t: String = stagefile.into();
        self.stagefile = t;
    }

    pub fn get_backbmp(&self) -> &str {
        &self.backbmp
    }

    pub fn set_backbmp(&mut self, backbmp: impl Into<String>) {
        let t: String = backbmp.into();
        self.backbmp = t;
    }

    pub fn get_total_notes(&self) -> i32 {
        crate::bms_model_utils::get_total_notes(self)
    }

    pub fn is_from_osu(&self) -> bool {
        self.from_osu
    }

    pub fn set_from_osu(&mut self, from_osu: bool) {
        self.from_osu = from_osu;
    }

    pub fn contains_undefined_long_note(&self) -> bool {
        let keys = self.mode.as_ref().map(|m| m.key()).unwrap_or(0);
        for tl in &self.timelines {
            for i in 0..keys {
                if let Some(note) = tl.get_note(i)
                    && note.is_long()
                    && note.get_long_note_type() == crate::note::TYPE_UNDEFINED
                {
                    return true;
                }
            }
        }
        false
    }

    pub fn contains_long_note(&self) -> bool {
        let keys = self.mode.as_ref().map(|m| m.key()).unwrap_or(0);
        for tl in &self.timelines {
            for i in 0..keys {
                if let Some(note) = tl.get_note(i)
                    && note.is_long()
                {
                    return true;
                }
            }
        }
        false
    }

    pub fn contains_mine_note(&self) -> bool {
        let keys = self.mode.as_ref().map(|m| m.key()).unwrap_or(0);
        for tl in &self.timelines {
            for i in 0..keys {
                if let Some(note) = tl.get_note(i)
                    && note.is_mine()
                {
                    return true;
                }
            }
        }
        false
    }

    pub fn get_preview(&self) -> &str {
        &self.preview
    }

    pub fn set_preview(&mut self, preview: impl Into<String>) {
        self.preview = preview.into();
    }

    pub fn get_lnobj(&self) -> i32 {
        self.lnobj
    }

    pub fn set_lnobj(&mut self, lnobj: i32) {
        self.lnobj = lnobj;
    }

    pub fn get_lnmode(&self) -> i32 {
        self.lnmode
    }

    pub fn set_lnmode(&mut self, lnmode: i32) {
        self.lnmode = lnmode;
    }

    pub fn get_values(&self) -> &HashMap<String, String> {
        &self.values
    }

    pub fn get_values_mut(&mut self) -> &mut HashMap<String, String> {
        &mut self.values
    }

    pub fn to_chart_string(&self) -> String {
        let mode = match &self.mode {
            Some(m) => m,
            None => return String::new(),
        };
        let key = mode.key();
        let mut sb = String::new();
        sb.push_str(&format!("JUDGERANK:{}\n", self.judgerank));
        sb.push_str(&format!("TOTAL:{}\n", self.total));
        if self.lnmode != 0 {
            sb.push_str(&format!("LNMODE:{}\n", self.lnmode));
        }
        let mut nowbpm = -f64::MIN_POSITIVE;
        for tl in &self.timelines {
            let mut tlsb = String::new();
            tlsb.push_str(&format!("{}:", tl.get_time()));
            let mut write = false;
            if nowbpm != tl.get_bpm() {
                nowbpm = tl.get_bpm();
                tlsb.push_str(&format!("B({})", nowbpm));
                write = true;
            }
            if tl.get_stop() != 0 {
                tlsb.push_str(&format!("S({})", tl.get_stop()));
                write = true;
            }
            if tl.get_section_line() {
                tlsb.push('L');
                write = true;
            }

            tlsb.push('[');
            for lane in 0..key {
                if let Some(n) = tl.get_note(lane) {
                    match n {
                        Note::Normal(_) => {
                            tlsb.push('1');
                            write = true;
                        }
                        Note::Long { end, note_type, .. } => {
                            if !end {
                                let lnchars = ['l', 'L', 'C', 'H'];
                                tlsb.push(lnchars[*note_type as usize]);
                                tlsb.push_str(&format!("{}", n.get_milli_duration()));
                                write = true;
                            }
                        }
                        Note::Mine { damage, .. } => {
                            tlsb.push_str(&format!("m{}", damage));
                            write = true;
                        }
                    }
                } else {
                    tlsb.push('0');
                }
                if lane < key - 1 {
                    tlsb.push(',');
                }
            }
            tlsb.push_str("]\n");

            if write {
                sb.push_str(&tlsb);
            }
        }
        sb
    }

    pub fn get_judgerank_type(&self) -> &JudgeRankType {
        &self.judgerank_type
    }

    pub fn set_judgerank_type(&mut self, judgerank_type: JudgeRankType) {
        self.judgerank_type = judgerank_type;
    }

    pub fn get_total_type(&self) -> &TotalType {
        &self.total_type
    }

    pub fn set_total_type(&mut self, total_type: TotalType) {
        self.total_type = total_type;
    }

    pub fn get_base(&self) -> i32 {
        self.base
    }

    pub fn set_base(&mut self, base: i32) {
        if base == 62 {
            self.base = base;
        } else {
            self.base = 36;
        }
    }
}
