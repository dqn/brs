use bms_config::{AudioConfig, Config, DriverType, FrequencyType, PlayerConfig};

use crate::panel::LauncherPanel;
use crate::tab::Tab;
use crate::widgets::clamped::{clamped_f32, clamped_i32};

pub struct AudioPanel {
    driver: DriverType,
    device_buffer_size: i32,
    device_simultaneous_sources: i32,
    sample_rate: i32,
    freq_option: FrequencyType,
    fast_forward: FrequencyType,
    systemvolume: f32,
    keyvolume: f32,
    bgvolume: f32,
    normalize_volume: bool,
    is_loop_result_sound: bool,
    is_loop_course_result_sound: bool,
    dirty: bool,
}

impl Default for AudioPanel {
    fn default() -> Self {
        let ac = AudioConfig::default();
        Self {
            driver: ac.driver,
            device_buffer_size: ac.device_buffer_size,
            device_simultaneous_sources: ac.device_simultaneous_sources,
            sample_rate: ac.sample_rate,
            freq_option: ac.freq_option,
            fast_forward: ac.fast_forward,
            systemvolume: ac.systemvolume,
            keyvolume: ac.keyvolume,
            bgvolume: ac.bgvolume,
            normalize_volume: ac.normalize_volume,
            is_loop_result_sound: ac.is_loop_result_sound,
            is_loop_course_result_sound: ac.is_loop_course_result_sound,
            dirty: false,
        }
    }
}

impl LauncherPanel for AudioPanel {
    fn tab(&self) -> Tab {
        Tab::Audio
    }

    fn load(&mut self, config: &Config, _player_config: &PlayerConfig) {
        let a = &config.audio;
        self.driver = a.driver;
        self.device_buffer_size = a.device_buffer_size;
        self.device_simultaneous_sources = a.device_simultaneous_sources;
        self.sample_rate = a.sample_rate;
        self.freq_option = a.freq_option;
        self.fast_forward = a.fast_forward;
        self.systemvolume = a.systemvolume;
        self.keyvolume = a.keyvolume;
        self.bgvolume = a.bgvolume;
        self.normalize_volume = a.normalize_volume;
        self.is_loop_result_sound = a.is_loop_result_sound;
        self.is_loop_course_result_sound = a.is_loop_course_result_sound;
        self.dirty = false;
    }

    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.heading("Audio Settings");
        ui.separator();

        // Driver
        let prev = self.driver;
        egui::ComboBox::from_label("Audio Driver")
            .selected_text(format!("{:?}", self.driver))
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut self.driver, DriverType::OpenAL, "OpenAL");
                ui.selectable_value(&mut self.driver, DriverType::PortAudio, "PortAudio");
            });
        if self.driver != prev {
            self.dirty = true;
        }

        let prev = self.device_buffer_size;
        clamped_i32(ui, "Buffer Size", &mut self.device_buffer_size, 4, 4096);
        if self.device_buffer_size != prev {
            self.dirty = true;
        }

        let prev = self.device_simultaneous_sources;
        clamped_i32(
            ui,
            "Simultaneous Sources",
            &mut self.device_simultaneous_sources,
            16,
            1024,
        );
        if self.device_simultaneous_sources != prev {
            self.dirty = true;
        }

        let prev = self.sample_rate;
        clamped_i32(ui, "Sample Rate (0=auto)", &mut self.sample_rate, 0, 192000);
        if self.sample_rate != prev {
            self.dirty = true;
        }

        ui.separator();

        // Frequency option
        let prev = self.freq_option;
        egui::ComboBox::from_label("Frequency Option")
            .selected_text(format!("{:?}", self.freq_option))
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut self.freq_option, FrequencyType::Frequency, "Frequency");
                ui.selectable_value(
                    &mut self.freq_option,
                    FrequencyType::Unprocessed,
                    "Unprocessed",
                );
            });
        if self.freq_option != prev {
            self.dirty = true;
        }

        let prev = self.fast_forward;
        egui::ComboBox::from_label("Fast Forward")
            .selected_text(format!("{:?}", self.fast_forward))
            .show_ui(ui, |ui| {
                ui.selectable_value(
                    &mut self.fast_forward,
                    FrequencyType::Frequency,
                    "Frequency",
                );
                ui.selectable_value(
                    &mut self.fast_forward,
                    FrequencyType::Unprocessed,
                    "Unprocessed",
                );
            });
        if self.fast_forward != prev {
            self.dirty = true;
        }

        ui.separator();
        ui.label("Volume");

        let prev = self.systemvolume;
        clamped_f32(ui, "System", &mut self.systemvolume, 0.0, 1.0, 0.01);
        if (self.systemvolume - prev).abs() > f32::EPSILON {
            self.dirty = true;
        }

        let prev = self.keyvolume;
        clamped_f32(ui, "Key", &mut self.keyvolume, 0.0, 1.0, 0.01);
        if (self.keyvolume - prev).abs() > f32::EPSILON {
            self.dirty = true;
        }

        let prev = self.bgvolume;
        clamped_f32(ui, "BG", &mut self.bgvolume, 0.0, 1.0, 0.01);
        if (self.bgvolume - prev).abs() > f32::EPSILON {
            self.dirty = true;
        }

        ui.separator();

        if ui
            .checkbox(&mut self.normalize_volume, "Loudness Normalization")
            .changed()
        {
            self.dirty = true;
        }
        if ui
            .checkbox(&mut self.is_loop_result_sound, "Loop Result Sound")
            .changed()
        {
            self.dirty = true;
        }
        if ui
            .checkbox(
                &mut self.is_loop_course_result_sound,
                "Loop Course Result Sound",
            )
            .changed()
        {
            self.dirty = true;
        }
    }

    fn apply(&self, config: &mut Config, _player_config: &mut PlayerConfig) {
        config.audio.driver = self.driver;
        config.audio.device_buffer_size = self.device_buffer_size;
        config.audio.device_simultaneous_sources = self.device_simultaneous_sources;
        config.audio.sample_rate = self.sample_rate;
        config.audio.freq_option = self.freq_option;
        config.audio.fast_forward = self.fast_forward;
        config.audio.systemvolume = self.systemvolume;
        config.audio.keyvolume = self.keyvolume;
        config.audio.bgvolume = self.bgvolume;
        config.audio.normalize_volume = self.normalize_volume;
        config.audio.is_loop_result_sound = self.is_loop_result_sound;
        config.audio.is_loop_course_result_sound = self.is_loop_course_result_sound;
    }

    fn has_changes(&self) -> bool {
        self.dirty
    }
}
