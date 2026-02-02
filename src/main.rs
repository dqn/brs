use brs::audio::{AudioConfig, AudioDriver, KeysoundProcessor};
use brs::input::{InputManager, KeyConfig};
use brs::model::{BMSModel, load_bms};
use brs::state::play::{GaugeType, PlayState};
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

    // Initialize audio driver
    let audio_config = AudioConfig::default();
    let audio_driver = match AudioDriver::new(audio_config) {
        Ok(driver) => driver,
        Err(e) => {
            eprintln!("Failed to initialize audio: {}", e);
            return;
        }
    };

    // Load BMS model
    let model = match load_bms(bms_path) {
        Ok(bms) => match BMSModel::from_bms(&bms) {
            Ok(model) => {
                println!("Loaded: {}", model.title);
                println!("Artist: {}", model.artist);
                println!("BPM: {}", model.initial_bpm);
                println!("Total notes: {}", model.total_notes);
                println!("BGM events: {}", model.bgm_events.len());
                model
            }
            Err(e) => {
                eprintln!("Failed to create model: {}", e);
                return;
            }
        },
        Err(e) => {
            eprintln!("Failed to load BMS: {}", e);
            return;
        }
    };

    // Load audio files
    let mut audio_driver = audio_driver;
    let bms_dir = bms_path.parent().unwrap_or(Path::new("."));
    match audio_driver.load_sounds(&model, bms_dir) {
        Ok(progress) => {
            println!(
                "Loaded {} of {} sounds",
                progress.loaded(),
                progress.total()
            );
        }
        Err(e) => {
            eprintln!("Failed to load sounds: {}", e);
        }
    }

    // Setup keysound processor
    let mut keysound_processor = KeysoundProcessor::new();
    keysound_processor.load_bgm_events(model.bgm_events.clone());

    // Setup input manager
    let key_config = KeyConfig::default();
    let input_manager = match InputManager::new(key_config) {
        Ok(manager) => manager,
        Err(e) => {
            eprintln!("Failed to initialize input: {}", e);
            return;
        }
    };

    // Create play state
    let mut play_state = PlayState::new(
        model,
        audio_driver,
        keysound_processor,
        input_manager,
        GaugeType::Normal,
        1.0,
    );

    // Game loop
    loop {
        clear_background(Color::new(0.1, 0.1, 0.1, 1.0));

        let delta_ms = get_frame_time() as f64 * 1000.0;

        // Handle hi-speed adjustment
        if is_key_pressed(KeyCode::Up) {
            play_state.set_hi_speed(play_state.hi_speed() + 0.25);
        }
        if is_key_pressed(KeyCode::Down) {
            play_state.set_hi_speed(play_state.hi_speed() - 0.25);
        }

        // Update and draw
        if let Err(e) = play_state.update(delta_ms) {
            eprintln!("Play error: {}", e);
        }
        play_state.draw();

        // Draw controls help
        draw_controls_help();

        // Check for exit or restart
        if is_key_pressed(KeyCode::Escape) {
            break;
        }

        if play_state.is_finished() && is_key_pressed(KeyCode::Enter) {
            // Show result (for now just print)
            let result = play_state.take_result();
            println!("=== RESULT ===");
            println!("EX-SCORE: {}", result.ex_score());
            println!("MAX COMBO: {}", result.max_combo());
            println!("BP: {}", result.bp());
            println!("Rank: {}", result.rank().as_str());
            println!("Clear: {}", if result.is_clear { "YES" } else { "NO" });
            break;
        }

        next_frame().await;
    }
}

fn draw_controls_help() {
    let x = 550.0;
    let y = 400.0;

    draw_text("Controls:", x, y, 18.0, GRAY);
    draw_text("  Up/Down: Adjust hi-speed", x, y + 24.0, 16.0, GRAY);
    draw_text("  LShift: Scratch", x, y + 48.0, 16.0, GRAY);
    draw_text("  Z S X D C F V: Keys 1-7", x, y + 72.0, 16.0, GRAY);
    draw_text("  Escape: Exit", x, y + 96.0, 16.0, GRAY);
    draw_text(
        "  Enter (after finish): Show result",
        x,
        y + 120.0,
        16.0,
        GRAY,
    );
}
