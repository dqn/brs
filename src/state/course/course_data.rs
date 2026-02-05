//! Course data structures for Dan/Class mode.

use std::path::PathBuf;

use crate::state::play::GaugeType;

/// A song entry in a course.
#[derive(Debug, Clone)]
pub struct CourseSong {
    /// Path to the BMS file.
    pub path: PathBuf,
    /// Title of the song (for display).
    pub title: String,
}

impl CourseSong {
    /// Create a new course song entry.
    pub fn new(path: PathBuf, title: String) -> Self {
        Self { path, title }
    }
}

/// Constraints for course play.
#[derive(Debug, Clone, Default)]
pub struct CourseConstraints {
    /// Whether gauge carries over between songs.
    pub gauge_carry: bool,
    /// Whether hi-speed changes are prohibited.
    pub no_speed_change: bool,
    /// Whether GOOD judgments count as failure.
    pub no_good: bool,
    /// Whether random options are prohibited.
    pub no_random: bool,
}

impl CourseConstraints {
    /// Create default constraints for class/dan mode.
    pub fn class_mode() -> Self {
        Self {
            gauge_carry: true,
            no_speed_change: false,
            no_good: false,
            no_random: false,
        }
    }

    /// Create strict constraints (no options allowed).
    pub fn strict() -> Self {
        Self {
            gauge_carry: true,
            no_speed_change: true,
            no_good: true,
            no_random: true,
        }
    }
}

/// A course definition (Dan/Class).
#[derive(Debug, Clone)]
pub struct Course {
    /// Name of the course (e.g., "発狂初段").
    pub name: String,
    /// Songs in the course (usually 4).
    pub songs: Vec<CourseSong>,
    /// Gauge type for the course.
    pub gauge_type: GaugeType,
    /// Constraints for the course.
    pub constraints: CourseConstraints,
}

impl Course {
    /// Create a new course.
    pub fn new(
        name: String,
        songs: Vec<CourseSong>,
        gauge_type: GaugeType,
        constraints: CourseConstraints,
    ) -> Self {
        Self {
            name,
            songs,
            gauge_type,
            constraints,
        }
    }

    /// Get the number of songs in the course.
    pub fn song_count(&self) -> usize {
        self.songs.len()
    }

    /// Get a song by index.
    pub fn get_song(&self, index: usize) -> Option<&CourseSong> {
        self.songs.get(index)
    }

    /// Check if this is a valid course (at least 1 song).
    pub fn is_valid(&self) -> bool {
        !self.songs.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_course_creation() {
        let songs = vec![
            CourseSong::new(PathBuf::from("song1.bms"), "Song 1".to_string()),
            CourseSong::new(PathBuf::from("song2.bms"), "Song 2".to_string()),
        ];

        let course = Course::new(
            "Test Course".to_string(),
            songs,
            GaugeType::Hard,
            CourseConstraints::class_mode(),
        );

        assert_eq!(course.name, "Test Course");
        assert_eq!(course.song_count(), 2);
        assert!(course.is_valid());
    }

    #[test]
    fn test_course_constraints() {
        let strict = CourseConstraints::strict();
        assert!(strict.gauge_carry);
        assert!(strict.no_speed_change);
        assert!(strict.no_good);
        assert!(strict.no_random);
    }
}
