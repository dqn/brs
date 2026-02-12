//! Random course data structures and lottery logic.
//!
//! Port of Java `RandomCourseData.java` and `RandomStageData.java`.

use chrono::Local;
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::course_data::{CourseData, CourseDataConstraint, CourseSongData, TrophyData};

/// Stage data for a random course, containing a SQL query to select candidates.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RandomStageData {
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub sql: String,
}

/// Random course constraint (distinct from CourseDataConstraint).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RandomCourseDataConstraint {
    /// No duplicate songs across stages.
    Distinct,
}

/// Random course data — picks songs from SQL query results via lottery.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RandomCourseData {
    pub name: String,
    #[serde(default)]
    pub stage: Vec<RandomStageData>,
    #[serde(default)]
    pub constraint: Vec<CourseDataConstraint>,
    #[serde(default)]
    pub rconstraint: Vec<RandomCourseDataConstraint>,
    #[serde(default)]
    pub trophy: Vec<TrophyData>,
}

impl RandomCourseData {
    /// Returns true if the distinct constraint is set.
    pub fn is_distinct(&self) -> bool {
        self.rconstraint
            .contains(&RandomCourseDataConstraint::Distinct)
    }

    /// Create a CourseData from lottery results.
    ///
    /// Name is appended with a timestamp in `yyyyMMdd_HHmmss` format.
    pub fn create_course_data(&self, songs: &[CourseSongData]) -> CourseData {
        let timestamp = Local::now().format("%Y%m%d_%H%M%S");
        CourseData {
            name: format!("{} {timestamp}", self.name),
            hash: songs.to_vec(),
            constraint: self.constraint.clone(),
            trophy: self.trophy.clone(),
            release: false,
        }
    }

    /// Pick one song per stage from candidates.
    ///
    /// `candidates_per_stage` must have the same length as `self.stage`.
    /// If DISTINCT constraint is set, avoids picking the same song (by sha256)
    /// across stages. Falls back to allowing duplicates if no unique candidate remains.
    pub fn lottery(
        &self,
        candidates_per_stage: &[Vec<CourseSongData>],
        rng: &mut impl Rng,
    ) -> Vec<Option<CourseSongData>> {
        let is_distinct = self.is_distinct();
        let stage_count = self.stage.len().min(candidates_per_stage.len());
        let mut results: Vec<Option<CourseSongData>> = vec![None; stage_count];

        for i in 0..stage_count {
            let candidates = &candidates_per_stage[i];
            if candidates.is_empty() {
                continue;
            }

            if !is_distinct {
                let idx = rng.random_range(0..candidates.len());
                results[i] = Some(candidates[idx].clone());
                continue;
            }

            // DISTINCT: try to pick a song not already selected
            let mut remaining: Vec<(usize, &CourseSongData)> =
                candidates.iter().enumerate().collect();

            loop {
                if remaining.is_empty() {
                    // All candidates are duplicates; fall back to any
                    let idx = rng.random_range(0..candidates.len());
                    results[i] = Some(candidates[idx].clone());
                    break;
                }

                let pick = rng.random_range(0..remaining.len());
                let candidate = remaining[pick].1;

                let is_duplicate = results[..i]
                    .iter()
                    .any(|prev| prev.as_ref().is_some_and(|p| p.sha256 == candidate.sha256));

                if is_duplicate {
                    remaining.remove(pick);
                } else {
                    results[i] = Some(candidate.clone());
                    break;
                }
            }
        }

        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn song(sha: &str) -> CourseSongData {
        CourseSongData {
            sha256: sha.to_string(),
            md5: String::new(),
            title: format!("Song {sha}"),
        }
    }

    #[test]
    fn is_distinct_check() {
        let rcd = RandomCourseData {
            rconstraint: vec![RandomCourseDataConstraint::Distinct],
            ..Default::default()
        };
        assert!(rcd.is_distinct());

        let rcd2 = RandomCourseData::default();
        assert!(!rcd2.is_distinct());
    }

    #[test]
    fn create_course_data_has_timestamp() {
        let rcd = RandomCourseData {
            name: "Test Course".to_string(),
            constraint: vec![CourseDataConstraint::Class],
            trophy: vec![TrophyData {
                name: "Gold".to_string(),
                missrate: 5.0,
                scorerate: 85.0,
            }],
            ..Default::default()
        };

        let songs = vec![song("a"), song("b")];
        let cd = rcd.create_course_data(&songs);

        assert!(cd.name.starts_with("Test Course "));
        assert_eq!(cd.hash.len(), 2);
        assert_eq!(cd.constraint.len(), 1);
        assert_eq!(cd.trophy.len(), 1);
        assert!(!cd.release);
    }

    #[test]
    fn lottery_basic() {
        let rcd = RandomCourseData {
            name: "Test".to_string(),
            stage: vec![
                RandomStageData {
                    title: "Stage 1".to_string(),
                    sql: String::new(),
                },
                RandomStageData {
                    title: "Stage 2".to_string(),
                    sql: String::new(),
                },
            ],
            ..Default::default()
        };

        let candidates = vec![vec![song("a"), song("b")], vec![song("c"), song("d")]];

        let mut rng = rand::rng();
        let results = rcd.lottery(&candidates, &mut rng);
        assert_eq!(results.len(), 2);
        assert!(results[0].is_some());
        assert!(results[1].is_some());
    }

    #[test]
    fn lottery_empty_candidates() {
        let rcd = RandomCourseData {
            name: "Test".to_string(),
            stage: vec![RandomStageData::default()],
            ..Default::default()
        };

        let candidates = vec![vec![]];
        let mut rng = rand::rng();
        let results = rcd.lottery(&candidates, &mut rng);
        assert_eq!(results.len(), 1);
        assert!(results[0].is_none());
    }

    #[test]
    fn lottery_distinct_avoids_duplicates() {
        let rcd = RandomCourseData {
            name: "Test".to_string(),
            stage: vec![
                RandomStageData::default(),
                RandomStageData::default(),
                RandomStageData::default(),
            ],
            rconstraint: vec![RandomCourseDataConstraint::Distinct],
            ..Default::default()
        };

        // All stages have the same 3 candidates
        let songs = vec![song("a"), song("b"), song("c")];
        let candidates = vec![songs.clone(), songs.clone(), songs];

        let mut rng = rand::rng();
        let results = rcd.lottery(&candidates, &mut rng);
        assert_eq!(results.len(), 3);

        // All should be Some
        let selected: Vec<String> = results
            .iter()
            .map(|r| r.as_ref().unwrap().sha256.clone())
            .collect();

        // With 3 candidates and 3 stages, all should be distinct
        let mut unique = selected.clone();
        unique.sort();
        unique.dedup();
        assert_eq!(unique.len(), 3);
    }

    #[test]
    fn lottery_distinct_fallback_when_not_enough_unique() {
        let rcd = RandomCourseData {
            name: "Test".to_string(),
            stage: vec![
                RandomStageData::default(),
                RandomStageData::default(),
                RandomStageData::default(),
            ],
            rconstraint: vec![RandomCourseDataConstraint::Distinct],
            ..Default::default()
        };

        // Only 1 unique song across all stages
        let candidates = vec![vec![song("a")], vec![song("a")], vec![song("a")]];

        let mut rng = rand::rng();
        let results = rcd.lottery(&candidates, &mut rng);
        assert_eq!(results.len(), 3);
        // All should be Some (fallback allows duplicates)
        for r in &results {
            assert!(r.is_some());
            assert_eq!(r.as_ref().unwrap().sha256, "a");
        }
    }

    #[test]
    fn serde_roundtrip() {
        let rcd = RandomCourseData {
            name: "Random Dan".to_string(),
            stage: vec![RandomStageData {
                title: "Stage 1".to_string(),
                sql: "SELECT * FROM song WHERE level > 10".to_string(),
            }],
            constraint: vec![CourseDataConstraint::Class],
            rconstraint: vec![RandomCourseDataConstraint::Distinct],
            trophy: vec![TrophyData {
                name: "Gold".to_string(),
                missrate: 3.0,
                scorerate: 80.0,
            }],
        };

        let json = serde_json::to_string_pretty(&rcd).unwrap();
        let parsed: RandomCourseData = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.name, "Random Dan");
        assert_eq!(parsed.stage.len(), 1);
        assert_eq!(parsed.stage[0].sql, "SELECT * FROM song WHERE level > 10");
        assert!(parsed.is_distinct());
    }
}
