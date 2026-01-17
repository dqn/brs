mod audio;
mod bms;
mod config;
mod dan;
mod database;
mod game;
mod ir;
mod render;
mod scene;
mod skin;

use macroquad::camera::{Camera2D, set_camera, set_default_camera};
use macroquad::prelude::*;
use macroquad::texture::{FilterMode, RenderTarget, render_target};
use macroquad::window::Conf;

use config::GameSettings;
use render::{VIRTUAL_HEIGHT, VIRTUAL_WIDTH};
use scene::{GameplayScene, Scene, SceneTransition, SongSelectScene};

fn window_conf() -> Conf {
    let settings = GameSettings::load();
    Conf {
        window_title: "BMS Player".to_owned(),
        window_width: settings.display.width as i32,
        window_height: settings.display.height as i32,
        fullscreen: settings.display.fullscreen,
        ..Default::default()
    }
}

struct SceneManager {
    scenes: Vec<Box<dyn Scene>>,
    render_target: RenderTarget,
    camera: Camera2D,
}

impl SceneManager {
    fn new(initial_scene: Box<dyn Scene>) -> Self {
        let rt = render_target(VIRTUAL_WIDTH as u32, VIRTUAL_HEIGHT as u32);
        rt.texture.set_filter(FilterMode::Linear);

        let mut camera =
            Camera2D::from_display_rect(Rect::new(0.0, 0.0, VIRTUAL_WIDTH, VIRTUAL_HEIGHT));
        camera.render_target = Some(rt.clone());

        Self {
            scenes: vec![initial_scene],
            render_target: rt,
            camera,
        }
    }

    fn update(&mut self) {
        if let Some(scene) = self.scenes.last_mut() {
            match scene.update() {
                SceneTransition::None => {}
                SceneTransition::Push(new_scene) => {
                    self.scenes.push(new_scene);
                }
                SceneTransition::Pop => {
                    self.scenes.pop();
                }
                SceneTransition::Replace(new_scene) => {
                    self.scenes.pop();
                    self.scenes.push(new_scene);
                }
            }
        }
    }

    fn draw(&self) {
        // Render to virtual resolution
        set_camera(&self.camera);
        if let Some(scene) = self.scenes.last() {
            scene.draw();
        }

        // Draw to actual window with scaling
        set_default_camera();
        clear_background(BLACK);

        let scale = f32::min(
            screen_width() / VIRTUAL_WIDTH,
            screen_height() / VIRTUAL_HEIGHT,
        );

        draw_texture_ex(
            &self.render_target.texture,
            (screen_width() - VIRTUAL_WIDTH * scale) * 0.5,
            (screen_height() - VIRTUAL_HEIGHT * scale) * 0.5,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(VIRTUAL_WIDTH * scale, VIRTUAL_HEIGHT * scale)),
                flip_y: true,
                ..Default::default()
            },
        );
    }

    fn is_empty(&self) -> bool {
        self.scenes.is_empty()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    render::font::init_font();

    let args: Vec<String> = std::env::args().collect();

    let initial_scene: Box<dyn Scene> = if args.len() > 1 {
        let arg = &args[1];
        let path = std::path::Path::new(arg);

        if path.is_file() {
            Box::new(GameplayScene::new(arg.clone()))
        } else if path.is_dir() {
            Box::new(SongSelectScene::new(Some(arg)))
        } else {
            Box::new(SongSelectScene::new(None))
        }
    } else {
        Box::new(SongSelectScene::new(None))
    };

    let mut manager = SceneManager::new(initial_scene);

    loop {
        manager.update();

        if manager.is_empty() {
            break;
        }

        manager.draw();
        next_frame().await
    }
}
