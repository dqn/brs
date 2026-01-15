use macroquad::prelude::*;

use super::{ResultScene, Scene, SceneTransition};
use crate::game::GameState;

pub struct GameplayScene {
    state: GameState,
    chart_path: String,
    loaded: bool,
    finished: bool,
}

impl GameplayScene {
    pub fn new(chart_path: String) -> Self {
        Self {
            state: GameState::new(),
            chart_path,
            loaded: false,
            finished: false,
        }
    }
}

impl Scene for GameplayScene {
    fn update(&mut self) -> SceneTransition {
        if !self.loaded {
            if let Err(e) = self.state.load_chart(&self.chart_path) {
                eprintln!("Error loading chart: {}", e);
                return SceneTransition::Pop;
            }
            self.loaded = true;
        }

        if is_key_pressed(KeyCode::Escape) {
            return SceneTransition::Pop;
        }

        self.state.update();

        // Check for game completion or failure
        let should_finish = self.state.is_finished() || self.state.is_failed();
        if should_finish && !self.finished {
            self.finished = true;
            let result = self.state.get_result(&self.chart_path);
            return SceneTransition::Replace(Box::new(ResultScene::new(result)));
        }

        SceneTransition::None
    }

    fn draw(&self) {
        self.state.draw();
    }
}
