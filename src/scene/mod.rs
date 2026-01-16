mod dan_gameplay;
mod dan_result;
mod dan_select;
mod gameplay;
mod result;
mod settings;
mod song_select;

pub use dan_gameplay::DanGameplayScene;
pub use dan_result::DanResultScene;
pub use dan_select::DanSelectScene;
pub use gameplay::GameplayScene;
pub use result::ResultScene;
pub use settings::SettingsScene;
pub use song_select::SongSelectScene;

pub enum SceneTransition {
    None,
    Push(Box<dyn Scene>),
    Pop,
    Replace(Box<dyn Scene>),
}

pub trait Scene {
    fn update(&mut self) -> SceneTransition;
    fn draw(&self);
}
