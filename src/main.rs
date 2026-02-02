use brs::model::{BMSModel, LaneConfig, load_bms};
use brs::render::{LaneRenderer, NoteRenderer};
use macroquad::prelude::*;
use std::path::Path;

fn window_conf() -> Conf {
    Conf {
        window_title: "brs".to_owned(),
        window_width: 1920,
        window_height: 1080,
        fullscreen: false,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let bms_path = Path::new("bms/bms-002/_take_7N.bms");

    let model = match load_bms(bms_path) {
        Ok(bms) => match BMSModel::from_bms(&bms) {
            Ok(model) => {
                println!("Loaded: {}", model.title);
                println!("Artist: {}", model.artist);
                println!("BPM: {}", model.initial_bpm);
                println!("Total notes: {}", model.total_notes);
                Some(model)
            }
            Err(e) => {
                eprintln!("Failed to create model: {}", e);
                None
            }
        },
        Err(e) => {
            eprintln!("Failed to load BMS: {}", e);
            None
        }
    };

    let lane_config = LaneConfig::default_7k();
    let mut current_time_ms = 0.0;
    let mut hi_speed = 1.0_f32;
    let mut auto_scroll = false;

    loop {
        clear_background(Color::new(0.1, 0.1, 0.1, 1.0));

        if is_key_pressed(KeyCode::Space) {
            auto_scroll = !auto_scroll;
        }

        if is_key_pressed(KeyCode::Up) {
            hi_speed = (hi_speed + 0.25).min(5.0);
        }
        if is_key_pressed(KeyCode::Down) {
            hi_speed = (hi_speed - 0.25).max(0.25);
        }

        if is_key_down(KeyCode::Right) {
            current_time_ms += 100.0;
        }
        if is_key_down(KeyCode::Left) {
            current_time_ms = (current_time_ms - 100.0_f64).max(0.0);
        }

        if auto_scroll {
            current_time_ms += get_frame_time() as f64 * 1000.0;
        }

        if let Some(ref model) = model {
            let lane_renderer = LaneRenderer::new(&lane_config);
            lane_renderer.draw(&model.timelines, current_time_ms, hi_speed);

            let note_renderer = NoteRenderer::new(&lane_config);
            note_renderer.draw(&model.timelines, current_time_ms, hi_speed);

            draw_info(model, current_time_ms, hi_speed, auto_scroll);
        } else {
            draw_text(
                "No BMS loaded. Place a BMS file at bms/bms-002/_take_7N.bms",
                100.0,
                100.0,
                24.0,
                WHITE,
            );
        }

        next_frame().await;
    }
}

fn draw_info(model: &BMSModel, current_time_ms: f64, hi_speed: f32, auto_scroll: bool) {
    let x = 600.0;
    let mut y = 120.0;
    let line_height = 24.0;

    draw_text(&format!("Title: {}", model.title), x, y, 20.0, WHITE);
    y += line_height;

    draw_text(&format!("Artist: {}", model.artist), x, y, 20.0, WHITE);
    y += line_height;

    draw_text(&format!("BPM: {:.1}", model.initial_bpm), x, y, 20.0, WHITE);
    y += line_height;

    draw_text(
        &format!("Total notes: {}", model.total_notes),
        x,
        y,
        20.0,
        WHITE,
    );
    y += line_height;

    draw_text(
        &format!("Time: {:.1}ms", current_time_ms),
        x,
        y,
        20.0,
        YELLOW,
    );
    y += line_height;

    draw_text(&format!("Hi-Speed: {:.2}x", hi_speed), x, y, 20.0, YELLOW);
    y += line_height;

    draw_text(
        &format!("Auto scroll: {}", if auto_scroll { "ON" } else { "OFF" }),
        x,
        y,
        20.0,
        if auto_scroll { GREEN } else { GRAY },
    );
    y += line_height * 2.0;

    draw_text("Controls:", x, y, 18.0, GRAY);
    y += line_height;

    draw_text("  Space: Toggle auto scroll", x, y, 16.0, GRAY);
    y += 20.0;

    draw_text("  Up/Down: Adjust hi-speed", x, y, 16.0, GRAY);
    y += 20.0;

    draw_text("  Left/Right: Seek", x, y, 16.0, GRAY);
}
