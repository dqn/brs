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

        if self.state.is_finished() && !self.finished {
            self.finished = true;
            let result = self.state.get_result();
            return SceneTransition::Replace(Box::new(ResultScene::new(result)));
        }

        SceneTransition::None
    }

    fn draw(&self) {
        self.state.draw();
    }
}
