use beatoraja_core::validatable::Validatable;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PatternModifyLog {
    pub section: f64,
    pub modify: Option<Vec<i32>>,
}

impl Default for PatternModifyLog {
    fn default() -> Self {
        PatternModifyLog {
            section: -1.0,
            modify: None,
        }
    }
}

impl PatternModifyLog {
    pub fn new(section: f64, modify: Vec<i32>) -> Self {
        PatternModifyLog {
            section,
            modify: Some(modify),
        }
    }
}

impl Validatable for PatternModifyLog {
    fn validate(&mut self) -> bool {
        self.section >= 0.0 && self.modify.is_some()
    }
}
