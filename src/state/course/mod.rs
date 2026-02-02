//! Course (Dan/Class) mode for consecutive song play.

mod course_data;
mod course_player;

pub use course_data::{Course, CourseConstraints, CourseSong};
pub use course_player::{CoursePlayer, CourseState};
