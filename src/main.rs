mod audio;
mod bms;
mod database;
mod game;
mod render;
mod scene;

use macroquad::prelude::*;

use scene::{GameplayScene, Scene, SceneTransition, SongSelectScene};

struct SceneManager {
    scenes: Vec<Box<dyn Scene>>,
}

impl SceneManager {
    fn new(initial_scene: Box<dyn Scene>) -> Self {
        Self {
            scenes: vec![initial_scene],
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
        if let Some(scene) = self.scenes.last() {
            scene.draw();
        }
    }

    fn is_empty(&self) -> bool {
        self.scenes.is_empty()
    }
}

#[macroquad::main("BMS Player")]
async fn main() {
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
