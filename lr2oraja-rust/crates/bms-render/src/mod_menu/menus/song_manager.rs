// SongManager menu — displays current song info and last play time.
//
// Corresponds to Java `SongManagerMenu.java`.
// Song data is injected from the game state; this module only handles display.

#[derive(Debug, Clone, Default)]
pub struct SongManagerState {
    pub current_song_title: String,
    pub last_played: Option<String>,
    pub sort_by_last_played: bool,
}

pub fn render(ctx: &egui::Context, open: &mut bool, state: &mut SongManagerState) {
    egui::Window::new("Song Manager")
        .open(open)
        .resizable(false)
        .show(ctx, |ui| {
            let song_name = if state.current_song_title.is_empty() {
                "(none)"
            } else {
                &state.current_song_title
            };
            ui.label(format!("Current picking: {song_name}"));

            let last_played = state.last_played.as_deref().unwrap_or("Not played");
            ui.label(format!("Last played: {last_played}"));

            ui.checkbox(&mut state.sort_by_last_played, "Sort by last played");

            if state.current_song_title.is_empty() {
                ui.label("Not a selectable song");
            }
        });
}
