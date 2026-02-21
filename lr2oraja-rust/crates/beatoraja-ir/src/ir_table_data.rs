use beatoraja_core::table_data::TableData;

use crate::ir_chart_data::IRChartData;
use crate::ir_course_data::IRCourseData;

/// IR table data
///
/// Translated from: IRTableData.java
#[derive(Clone, Debug)]
pub struct IRTableData {
    /// Table name
    pub name: String,
    /// Table folder data
    pub folders: Vec<IRTableFolder>,
    /// Course data
    pub courses: Vec<IRCourseData>,
}

impl IRTableData {
    pub fn new(name: String, folders: Vec<IRTableFolder>, courses: Vec<IRCourseData>) -> Self {
        Self {
            name,
            folders,
            courses,
        }
    }

    pub fn from_table_data(table: &TableData) -> Self {
        let mut folders = Vec::with_capacity(table.folder.len());
        for tf in &table.folder {
            let mut charts = Vec::with_capacity(tf.songs.len());
            for song in &tf.songs {
                // TableFolder songs are beatoraja_core::stubs::SongData
                charts.push(create_ir_chart_data_from_core_song(song));
            }
            folders.push(IRTableFolder::new(
                tf.name.clone().unwrap_or_default(),
                charts,
            ));
        }

        let mut courses = Vec::with_capacity(table.course.len());
        for course in &table.course {
            courses.push(IRCourseData::new(course));
        }

        Self {
            name: table.get_name().to_string(),
            folders,
            courses,
        }
    }
}

/// Create IRChartData from beatoraja_core::stubs::SongData
fn create_ir_chart_data_from_core_song(song: &beatoraja_core::stubs::SongData) -> IRChartData {
    IRChartData {
        md5: song.md5.clone().unwrap_or_default(),
        sha256: song.sha256.clone().unwrap_or_default(),
        title: song.title.clone().unwrap_or_default(),
        subtitle: String::new(),
        genre: String::new(),
        artist: String::new(),
        subartist: String::new(),
        url: song.url.clone().unwrap_or_default(),
        appendurl: String::new(),
        level: 0,
        total: 0,
        mode: None,
        lntype: 0,
        judge: 0,
        minbpm: 0,
        maxbpm: 0,
        notes: 0,
        has_undefined_ln: false,
        has_ln: false,
        has_cn: false,
        has_hcn: false,
        has_mine: false,
        has_random: false,
        has_stop: false,
        values: std::collections::HashMap::new(),
    }
}

/// Table folder data
///
/// Translated from: IRTableData.IRTableFolder (inner class)
#[derive(Clone, Debug)]
pub struct IRTableFolder {
    /// Folder name
    pub name: String,
    /// Chart data
    pub charts: Vec<IRChartData>,
}

impl IRTableFolder {
    pub fn new(name: String, charts: Vec<IRChartData>) -> Self {
        Self { name, charts }
    }
}
