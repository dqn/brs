use bms_config::{Config, PlayerConfig, SongPreview};

use crate::panel::LauncherPanel;
use crate::tab::Tab;
use crate::widgets::clamped::clamped_i32;

pub struct MusicSelectPanel {
    scrolldurationlow: i32,
    scrolldurationhigh: i32,
    analog_scroll: bool,
    analog_ticks_per_scroll: i32,
    song_preview: SongPreview,
    max_search_bar_count: i32,
    skip_decide_screen: bool,
    show_no_song_existing_bar: bool,
    folderlamp: bool,
    dirty: bool,
}

impl Default for MusicSelectPanel {
    fn default() -> Self {
        let config = Config::default();
        Self {
            scrolldurationlow: config.scrolldurationlow,
            scrolldurationhigh: config.scrolldurationhigh,
            analog_scroll: config.analog_scroll,
            analog_ticks_per_scroll: config.analog_ticks_per_scroll,
            song_preview: config.song_preview,
            max_search_bar_count: config.max_search_bar_count,
            skip_decide_screen: config.skip_decide_screen,
            show_no_song_existing_bar: config.show_no_song_existing_bar,
            folderlamp: config.folderlamp,
            dirty: false,
        }
    }
}

impl LauncherPanel for MusicSelectPanel {
    fn tab(&self) -> Tab {
        Tab::MusicSelect
    }

    fn load(&mut self, config: &Config, _player_config: &PlayerConfig) {
        self.scrolldurationlow = config.scrolldurationlow;
        self.scrolldurationhigh = config.scrolldurationhigh;
        self.analog_scroll = config.analog_scroll;
        self.analog_ticks_per_scroll = config.analog_ticks_per_scroll;
        self.song_preview = config.song_preview;
        self.max_search_bar_count = config.max_search_bar_count;
        self.skip_decide_screen = config.skip_decide_screen;
        self.show_no_song_existing_bar = config.show_no_song_existing_bar;
        self.folderlamp = config.folderlamp;
        self.dirty = false;
    }

    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.heading("Music Select Settings");
        ui.separator();

        let prev = self.scrolldurationlow;
        clamped_i32(
            ui,
            "Scroll Duration (Low)",
            &mut self.scrolldurationlow,
            2,
            1000,
        );
        if self.scrolldurationlow != prev {
            self.dirty = true;
        }

        let prev = self.scrolldurationhigh;
        clamped_i32(
            ui,
            "Scroll Duration (High)",
            &mut self.scrolldurationhigh,
            1,
            1000,
        );
        if self.scrolldurationhigh != prev {
            self.dirty = true;
        }

        if ui
            .checkbox(&mut self.analog_scroll, "Analog Scroll")
            .changed()
        {
            self.dirty = true;
        }

        if self.analog_scroll {
            let prev = self.analog_ticks_per_scroll;
            clamped_i32(
                ui,
                "Analog Ticks per Scroll",
                &mut self.analog_ticks_per_scroll,
                1,
                10,
            );
            if self.analog_ticks_per_scroll != prev {
                self.dirty = true;
            }
        }

        ui.separator();

        let prev = self.song_preview;
        egui::ComboBox::from_label("Song Preview")
            .selected_text(format!("{:?}", self.song_preview))
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut self.song_preview, SongPreview::None, "None");
                ui.selectable_value(&mut self.song_preview, SongPreview::Once, "Once");
                ui.selectable_value(&mut self.song_preview, SongPreview::Loop, "Loop");
            });
        if self.song_preview != prev {
            self.dirty = true;
        }

        let prev = self.max_search_bar_count;
        clamped_i32(
            ui,
            "Max Search Bars",
            &mut self.max_search_bar_count,
            1,
            100,
        );
        if self.max_search_bar_count != prev {
            self.dirty = true;
        }

        ui.separator();

        if ui
            .checkbox(&mut self.skip_decide_screen, "Skip Decide Screen")
            .changed()
        {
            self.dirty = true;
        }
        if ui
            .checkbox(&mut self.show_no_song_existing_bar, "Show 'No Song' Bar")
            .changed()
        {
            self.dirty = true;
        }
        if ui.checkbox(&mut self.folderlamp, "Folder Lamp").changed() {
            self.dirty = true;
        }
    }

    fn apply(&self, config: &mut Config, _player_config: &mut PlayerConfig) {
        config.scrolldurationlow = self.scrolldurationlow;
        config.scrolldurationhigh = self.scrolldurationhigh;
        config.analog_scroll = self.analog_scroll;
        config.analog_ticks_per_scroll = self.analog_ticks_per_scroll;
        config.song_preview = self.song_preview;
        config.max_search_bar_count = self.max_search_bar_count;
        config.skip_decide_screen = self.skip_decide_screen;
        config.show_no_song_existing_bar = self.show_no_song_existing_bar;
        config.folderlamp = self.folderlamp;
    }

    fn has_changes(&self) -> bool {
        self.dirty
    }
}
