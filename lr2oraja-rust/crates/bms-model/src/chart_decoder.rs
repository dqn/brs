use std::path::Path;

use crate::bms_decoder::BMSDecoder;
use crate::bms_model::LNTYPE_LONGNOTE;
use crate::chart_information::ChartInformation;
use crate::decode_log::DecodeLog;
use crate::time_line::TimeLine;

pub struct TimeLineCache {
    pub time: f64,
    pub timeline: TimeLine,
}

impl TimeLineCache {
    pub fn new(time: f64, timeline: TimeLine) -> Self {
        TimeLineCache { time, timeline }
    }
}

pub enum ChartDecoderImpl {
    Bms(BMSDecoder),
}

impl ChartDecoderImpl {
    pub fn decode_path(&mut self, path: &Path) -> Option<crate::bms_model::BMSModel> {
        let info = ChartInformation::new(
            Some(path.to_path_buf()),
            match self {
                ChartDecoderImpl::Bms(d) => d.lntype,
            },
            None,
        );
        self.decode(info)
    }

    pub fn decode(&mut self, info: ChartInformation) -> Option<crate::bms_model::BMSModel> {
        match self {
            ChartDecoderImpl::Bms(d) => d.decode(info),
        }
    }

    pub fn get_decode_log(&self) -> &[DecodeLog] {
        match self {
            ChartDecoderImpl::Bms(d) => &d.log,
        }
    }
}

pub fn get_decoder(p: &Path) -> Option<ChartDecoderImpl> {
    let s = p
        .file_name()
        .map(|f| f.to_string_lossy().to_lowercase())
        .unwrap_or_default();
    if s.ends_with(".bms") || s.ends_with(".bme") || s.ends_with(".bml") || s.ends_with(".pms") {
        return Some(ChartDecoderImpl::Bms(BMSDecoder::new_with_lntype(
            LNTYPE_LONGNOTE,
        )));
    } else if s.ends_with(".bmson") {
        todo!("BMSONDecoder not yet implemented");
    } else if s.ends_with(".osu") {
        todo!("OSUDecoder not yet implemented");
    }
    None
}

pub fn parse_int36_str(s: &str, index: usize) -> Result<i32, ()> {
    let bytes = s.as_bytes();
    if index + 1 >= bytes.len() {
        return Err(());
    }
    let result = parse_int36(bytes[index] as char, bytes[index + 1] as char);
    if result == -1 { Err(()) } else { Ok(result) }
}

pub fn parse_int36(c1: char, c2: char) -> i32 {
    let mut result: i32;
    if ('0'..='9').contains(&c1) {
        result = ((c1 as i32) - ('0' as i32)) * 36;
    } else if ('a'..='z').contains(&c1) {
        result = (((c1 as i32) - ('a' as i32)) + 10) * 36;
    } else if ('A'..='Z').contains(&c1) {
        result = (((c1 as i32) - ('A' as i32)) + 10) * 36;
    } else {
        return -1;
    }

    if ('0'..='9').contains(&c2) {
        result += (c2 as i32) - ('0' as i32);
    } else if ('a'..='z').contains(&c2) {
        result += ((c2 as i32) - ('a' as i32)) + 10;
    } else if ('A'..='Z').contains(&c2) {
        result += ((c2 as i32) - ('A' as i32)) + 10;
    } else {
        return -1;
    }

    result
}

pub fn parse_int62_str(s: &str, index: usize) -> Result<i32, ()> {
    let bytes = s.as_bytes();
    if index + 1 >= bytes.len() {
        return Err(());
    }
    let result = parse_int62(bytes[index] as char, bytes[index + 1] as char);
    if result == -1 { Err(()) } else { Ok(result) }
}

pub fn parse_int62(c1: char, c2: char) -> i32 {
    let mut result: i32;
    if ('0'..='9').contains(&c1) {
        result = ((c1 as i32) - ('0' as i32)) * 62;
    } else if ('A'..='Z').contains(&c1) {
        result = (((c1 as i32) - ('A' as i32)) + 10) * 62;
    } else if ('a'..='z').contains(&c1) {
        result = (((c1 as i32) - ('a' as i32)) + 36) * 62;
    } else {
        return -1;
    }

    if ('0'..='9').contains(&c2) {
        result += (c2 as i32) - ('0' as i32);
    } else if ('A'..='Z').contains(&c2) {
        result += ((c2 as i32) - ('A' as i32)) + 10;
    } else if ('a'..='z').contains(&c2) {
        result += ((c2 as i32) - ('a' as i32)) + 36;
    } else {
        return -1;
    }

    result
}

pub fn to_base62(mut decimal: i32) -> String {
    let mut sb = Vec::with_capacity(2);
    for _ in 0..2 {
        let m = (decimal % 62);
        if m < 10 {
            sb.push((b'0' + m as u8) as char);
        } else if m < 36 {
            sb.push((b'A' + (m - 10) as u8) as char);
        } else if m < 62 {
            sb.push((b'a' + (m - 36) as u8) as char);
        } else {
            sb.push('0');
        }
        decimal /= 62;
    }
    sb.reverse();
    sb.into_iter().collect()
}
