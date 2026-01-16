use std::path::PathBuf;

use macroquad::prelude::*;

use crate::dan::{DanCourse, DanGrade, load_courses};
use crate::database::DanRepository;

use super::{DanGameplayScene, Scene, SceneTransition};

/// Entry for a dan course in the list
struct CourseEntry {
    /// Path to the course JSON file
    path: PathBuf,
    /// The loaded course
    course: DanCourse,
    /// Whether this course has been cleared
    is_cleared: bool,
}

pub struct DanSelectScene {
    courses: Vec<CourseEntry>,
    selected_index: usize,
    scroll_offset: usize,
    visible_count: usize,
}

impl DanSelectScene {
    pub fn new() -> Self {
        // Load courses from dan_courses directory
        let dan_courses_dir = PathBuf::from("dan_courses");
        let loaded = load_courses(&dan_courses_dir);

        // Load dan records to check clear status
        let cleared_grades = DanRepository::new()
            .map(|repo| repo.cleared_grades())
            .unwrap_or_default();

        let courses: Vec<CourseEntry> = loaded
            .into_iter()
            .map(|(path, course)| {
                let is_cleared = cleared_grades.contains(&course.grade);
                CourseEntry {
                    path,
                    course,
                    is_cleared,
                }
            })
            .collect();

        Self {
            courses,
            selected_index: 0,
            scroll_offset: 0,
            visible_count: 12,
        }
    }

    fn update_scroll(&mut self) {
        if self.selected_index < self.scroll_offset {
            self.scroll_offset = self.selected_index;
        } else if self.selected_index >= self.scroll_offset + self.visible_count {
            self.scroll_offset = self.selected_index - self.visible_count + 1;
        }
    }

    fn grade_color(grade: &DanGrade) -> Color {
        match grade {
            DanGrade::Kyu(_) => Color::new(0.6, 0.8, 1.0, 1.0),
            DanGrade::Dan(n) if *n <= 3 => Color::new(0.8, 1.0, 0.6, 1.0),
            DanGrade::Dan(n) if *n <= 6 => Color::new(1.0, 0.9, 0.4, 1.0),
            DanGrade::Dan(n) if *n <= 9 => Color::new(1.0, 0.6, 0.3, 1.0),
            DanGrade::Dan(_) => Color::new(1.0, 0.4, 0.4, 1.0),
            DanGrade::Kaiden => Color::new(1.0, 0.8, 0.0, 1.0),
            DanGrade::Overjoy => Color::new(1.0, 0.0, 0.5, 1.0),
        }
    }
}

impl Default for DanSelectScene {
    fn default() -> Self {
        Self::new()
    }
}

impl Scene for DanSelectScene {
    fn update(&mut self) -> SceneTransition {
        if self.courses.is_empty() {
            if is_key_pressed(KeyCode::Escape) {
                return SceneTransition::Pop;
            }
            return SceneTransition::None;
        }

        if is_key_pressed(KeyCode::Up) && self.selected_index > 0 {
            self.selected_index -= 1;
            self.update_scroll();
        }

        if is_key_pressed(KeyCode::Down) && self.selected_index < self.courses.len() - 1 {
            self.selected_index += 1;
            self.update_scroll();
        }

        if is_key_pressed(KeyCode::Enter) {
            if let Some(entry) = self.courses.get(self.selected_index) {
                let course_dir = entry.path.parent().map(|p| p.to_path_buf());
                return SceneTransition::Push(Box::new(DanGameplayScene::new(
                    entry.course.clone(),
                    course_dir,
                )));
            }
        }

        if is_key_pressed(KeyCode::Escape) {
            return SceneTransition::Pop;
        }

        SceneTransition::None
    }

    fn draw(&self) {
        clear_background(Color::new(0.02, 0.02, 0.08, 1.0));

        draw_text("DAN CERTIFICATION", 20.0, 40.0, 32.0, WHITE);
        draw_text(
            &format!("{} courses", self.courses.len()),
            20.0,
            70.0,
            18.0,
            GRAY,
        );

        if self.courses.is_empty() {
            draw_text(
                "No courses found.",
                20.0,
                screen_height() / 2.0 - 40.0,
                24.0,
                YELLOW,
            );
            draw_text(
                "Place course JSON files in dan_courses/ directory.",
                20.0,
                screen_height() / 2.0,
                20.0,
                GRAY,
            );
            draw_text("[Esc] Back", 20.0, screen_height() - 20.0, 16.0, GRAY);
            return;
        }

        let start_y = 100.0;
        let item_height = 45.0;

        for (display_idx, entry) in self
            .courses
            .iter()
            .enumerate()
            .skip(self.scroll_offset)
            .take(self.visible_count)
        {
            let y = start_y + (display_idx - self.scroll_offset) as f32 * item_height;
            let is_selected = display_idx == self.selected_index;

            if is_selected {
                draw_rectangle(
                    10.0,
                    y - 5.0,
                    screen_width() - 20.0,
                    item_height,
                    Color::new(0.2, 0.3, 0.5, 1.0),
                );
            }

            // Clear indicator
            if entry.is_cleared {
                draw_rectangle(15.0, y, 6.0, item_height - 10.0, GREEN);
            } else {
                draw_rectangle(15.0, y, 6.0, item_height - 10.0, DARKGRAY);
            }

            // Grade display
            let grade_color = Self::grade_color(&entry.course.grade);
            draw_text(
                &entry.course.grade.display_name(),
                30.0,
                y + 20.0,
                20.0,
                grade_color,
            );

            // Course name
            let name_color = if is_selected { YELLOW } else { WHITE };
            draw_text(&entry.course.name, 100.0, y + 20.0, 22.0, name_color);

            // Stage count
            draw_text(
                &format!("{} stages", entry.course.stage_count()),
                100.0,
                y + 38.0,
                14.0,
                GRAY,
            );

            // Gauge type
            let gauge_text = match entry.course.gauge_type {
                crate::game::GaugeType::Hard => "HARD GAUGE",
                crate::game::GaugeType::ExHard => "EX-HARD GAUGE",
                crate::game::GaugeType::Hazard => "HAZARD GAUGE",
                _ => "GROOVE GAUGE",
            };
            draw_text(gauge_text, 250.0, y + 38.0, 14.0, GRAY);
        }

        draw_text(
            "[Up/Down] Select | [Enter] Start | [Esc] Back",
            20.0,
            screen_height() - 20.0,
            16.0,
            GRAY,
        );
    }
}
