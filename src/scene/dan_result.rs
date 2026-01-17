use macroquad::prelude::*;

use crate::dan::{CoursePassResult, CourseState, DanGrade};
use crate::database::{DanRecord, DanRepository};
use crate::game::ClearLamp;
use crate::render::font::draw_text_jp;
use crate::render::{VIRTUAL_HEIGHT, VIRTUAL_WIDTH};

use super::{Scene, SceneTransition};

pub struct DanResultScene {
    course_state: CourseState,
    pass_result: CoursePassResult,
    is_new_record: bool,
}

impl DanResultScene {
    pub fn new(course_state: CourseState) -> Self {
        let pass_result = course_state.check_requirements();

        // Save result if passed
        let is_new_record = if pass_result == CoursePassResult::Passed {
            Self::save_result(&course_state)
        } else {
            false
        };

        Self {
            course_state,
            pass_result,
            is_new_record,
        }
    }

    fn save_result(course_state: &CourseState) -> bool {
        let mut repo = match DanRepository::new() {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Failed to load dan repository: {}", e);
                return false;
            }
        };

        let stats = course_state.total_stats();
        let record = DanRecord {
            grade: course_state.course().grade,
            course_name: course_state.course().name.clone(),
            ex_score: stats.ex_score,
            max_combo: stats.max_combo,
            pgreat_count: stats.pgreat_count,
            great_count: stats.great_count,
            good_count: stats.good_count,
            bad_count: stats.bad_count,
            poor_count: stats.poor_count,
            clear_count: 1,
            last_cleared: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
        };

        let is_new = repo.update(record);

        if let Err(e) = repo.save() {
            eprintln!("Failed to save dan records: {}", e);
        }

        is_new
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

    fn clear_lamp_color(lamp: ClearLamp) -> Color {
        match lamp {
            ClearLamp::NoPlay => GRAY,
            ClearLamp::Failed => Color::new(0.5, 0.0, 0.0, 1.0),
            ClearLamp::AssistEasy => Color::new(0.6, 0.3, 0.8, 1.0),
            ClearLamp::Easy => Color::new(0.0, 0.8, 0.3, 1.0),
            ClearLamp::Normal => Color::new(0.2, 0.6, 1.0, 1.0),
            ClearLamp::Hard => Color::new(1.0, 0.5, 0.0, 1.0),
            ClearLamp::ExHard => Color::new(1.0, 0.8, 0.0, 1.0),
            ClearLamp::FullCombo => Color::new(1.0, 0.0, 0.5, 1.0),
        }
    }
}

impl Scene for DanResultScene {
    fn update(&mut self) -> SceneTransition {
        if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Escape) {
            return SceneTransition::Pop;
        }

        SceneTransition::None
    }

    fn draw(&self) {
        clear_background(Color::new(0.02, 0.02, 0.08, 1.0));

        let center_x = VIRTUAL_WIDTH / 2.0;
        let course = self.course_state.course();
        let stats = self.course_state.total_stats();

        // Title
        draw_text_jp("DAN RESULT", center_x - 80.0, 40.0, 36.0, WHITE);

        // Pass/Fail display
        let (result_text, result_color) = match self.pass_result {
            CoursePassResult::Passed => ("PASSED!", GREEN),
            CoursePassResult::Failed => ("FAILED", RED),
            CoursePassResult::Incomplete => ("INCOMPLETE", YELLOW),
        };
        draw_text_jp(result_text, center_x - 60.0, 90.0, 48.0, result_color);

        // New record indicator
        if self.is_new_record {
            draw_text_jp("NEW RECORD!", center_x + 80.0, 90.0, 20.0, GOLD);
        }

        // Grade and course name
        let grade_color = Self::grade_color(&course.grade);
        draw_text_jp(
            &course.grade.display_name(),
            center_x - 150.0,
            140.0,
            28.0,
            grade_color,
        );
        draw_text_jp(&course.name, center_x - 80.0, 140.0, 28.0, YELLOW);

        // Stage results
        let stage_start_y = 180.0;
        let stage_height = 30.0;

        draw_text_jp("Stage Results:", 50.0, stage_start_y, 18.0, GRAY);

        for (i, stage_result) in self.course_state.stage_results().iter().enumerate() {
            let y = stage_start_y + 30.0 + (i as f32) * stage_height;

            // Stage number
            draw_text_jp(&format!("STAGE {}", i + 1), 50.0, y, 16.0, WHITE);

            // Title (truncated)
            let title: String = stage_result.title.chars().take(20).collect();
            draw_text_jp(&title, 130.0, y, 16.0, GRAY);

            // EX Score
            draw_text_jp(
                &format!("EX:{}", stage_result.ex_score),
                350.0,
                y,
                16.0,
                YELLOW,
            );

            // Clear lamp
            let lamp_color = Self::clear_lamp_color(stage_result.clear_lamp);
            draw_text_jp(
                stage_result.clear_lamp.display_name(),
                450.0,
                y,
                14.0,
                lamp_color,
            );

            // End gauge
            draw_text_jp(
                &format!("{:.1}%", stage_result.end_gauge_hp),
                580.0,
                y,
                14.0,
                if stage_result.end_gauge_hp > 0.0 {
                    GREEN
                } else {
                    RED
                },
            );
        }

        // Total statistics
        let stats_y =
            stage_start_y + 50.0 + (self.course_state.stage_results().len() as f32) * stage_height;

        draw_text_jp("Total Statistics:", 50.0, stats_y, 18.0, GRAY);

        let rank = stats.rank();
        let rank_color = match rank {
            "MAX" => Color::new(1.0, 0.8, 0.0, 1.0),
            "AAA" => Color::new(1.0, 0.6, 0.0, 1.0),
            "AA" => Color::new(0.8, 0.8, 0.0, 1.0),
            "A" => Color::new(0.0, 1.0, 0.5, 1.0),
            "B" => Color::new(0.0, 0.8, 1.0, 1.0),
            _ => WHITE,
        };

        draw_text_jp(rank, center_x - 30.0, stats_y + 60.0, 60.0, rank_color);
        draw_text_jp(
            &format!("{:.2}%", stats.accuracy()),
            center_x - 50.0,
            stats_y + 100.0,
            28.0,
            WHITE,
        );

        let line_height = 25.0;
        let left_col = 80.0;
        let right_col = 300.0;

        draw_text_jp(
            &format!("EX SCORE: {}", stats.ex_score),
            left_col,
            stats_y + 140.0,
            20.0,
            YELLOW,
        );
        draw_text_jp(
            &format!("MAX COMBO: {}", stats.max_combo),
            right_col,
            stats_y + 140.0,
            20.0,
            WHITE,
        );

        draw_text_jp(
            &format!("PGREAT: {}", stats.pgreat_count),
            left_col,
            stats_y + 140.0 + line_height,
            18.0,
            Color::new(1.0, 1.0, 0.0, 1.0),
        );
        draw_text_jp(
            &format!("GREAT: {}", stats.great_count),
            right_col,
            stats_y + 140.0 + line_height,
            18.0,
            Color::new(1.0, 0.8, 0.0, 1.0),
        );
        draw_text_jp(
            &format!("GOOD: {}", stats.good_count),
            left_col,
            stats_y + 140.0 + line_height * 2.0,
            18.0,
            Color::new(0.0, 1.0, 0.5, 1.0),
        );
        draw_text_jp(
            &format!("BAD: {}", stats.bad_count),
            right_col,
            stats_y + 140.0 + line_height * 2.0,
            18.0,
            Color::new(0.5, 0.5, 1.0, 1.0),
        );
        draw_text_jp(
            &format!("POOR: {}", stats.poor_count),
            left_col,
            stats_y + 140.0 + line_height * 3.0,
            18.0,
            Color::new(1.0, 0.3, 0.3, 1.0),
        );
        draw_text_jp(
            &format!("TOTAL NOTES: {}", stats.total_notes),
            right_col,
            stats_y + 140.0 + line_height * 3.0,
            18.0,
            GRAY,
        );

        // FAST/SLOW
        draw_text_jp(
            &format!("FAST:{} / SLOW:{}", stats.fast_count, stats.slow_count),
            left_col,
            stats_y + 140.0 + line_height * 4.0,
            16.0,
            GRAY,
        );

        // Requirements info
        let req = self.course_state.requirements();
        if req.max_bad_poor.is_some() || req.full_combo {
            let req_y = stats_y + 140.0 + line_height * 5.5;
            draw_text_jp("Requirements:", left_col, req_y, 16.0, GRAY);

            if let Some(max_bp) = req.max_bad_poor {
                let bp = stats.bad_count + stats.poor_count;
                let bp_color = if bp <= max_bp { GREEN } else { RED };
                draw_text_jp(
                    &format!("BAD+POOR: {}/{}", bp, max_bp),
                    left_col + 120.0,
                    req_y,
                    16.0,
                    bp_color,
                );
            }

            if req.full_combo {
                let fc_color = if stats.bad_count == 0 && stats.poor_count == 0 {
                    GREEN
                } else {
                    RED
                };
                draw_text_jp("FULL COMBO REQUIRED", right_col, req_y, 16.0, fc_color);
            }
        }

        draw_text_jp(
            "[Enter] Continue",
            center_x - 60.0,
            VIRTUAL_HEIGHT - 30.0,
            18.0,
            GRAY,
        );
    }
}
