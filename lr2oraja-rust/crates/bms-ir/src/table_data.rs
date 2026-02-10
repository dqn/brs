use serde::{Deserialize, Serialize};

use crate::chart_data::IRChartData;
use crate::course_data::IRCourseData;

/// IR table folder data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IRTableFolder {
    pub name: String,
    pub charts: Vec<IRChartData>,
}

/// IR table data for transmission.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IRTableData {
    pub name: String,
    pub folders: Vec<IRTableFolder>,
    pub courses: Vec<IRCourseData>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serde_round_trip() {
        let table = IRTableData {
            name: "Test Table".to_string(),
            folders: vec![IRTableFolder {
                name: "★1".to_string(),
                charts: vec![],
            }],
            courses: vec![],
        };
        let json = serde_json::to_string(&table).unwrap();
        let deserialized: IRTableData = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.name, "Test Table");
        assert_eq!(deserialized.folders.len(), 1);
        assert_eq!(deserialized.folders[0].name, "★1");
    }
}
