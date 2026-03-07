use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::bms_model::LnType;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChartInformation {
    pub path: Option<PathBuf>,
    pub lntype: LnType,
    pub selected_randoms: Option<Vec<i32>>,
}

impl ChartInformation {
    pub fn new(path: Option<PathBuf>, lntype: LnType, selected_randoms: Option<Vec<i32>>) -> Self {
        ChartInformation {
            path,
            lntype,
            selected_randoms,
        }
    }
}
