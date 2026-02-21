use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChartInformation {
    pub path: Option<PathBuf>,
    pub lntype: i32,
    pub selected_randoms: Option<Vec<i32>>,
}

impl ChartInformation {
    pub fn new(path: Option<PathBuf>, lntype: i32, selected_randoms: Option<Vec<i32>>) -> Self {
        ChartInformation {
            path,
            lntype,
            selected_randoms,
        }
    }
}
