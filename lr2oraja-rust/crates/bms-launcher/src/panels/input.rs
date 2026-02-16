use bms_config::{Config, PlayerConfig};

use crate::panel::LauncherPanel;
use crate::tab::Tab;
use crate::widgets::clamped::clamped_i32;

const PLAY_MODE_LABELS: &[(&str, i32)] = &[
    ("5KEYS", 5),
    ("7KEYS", 7),
    ("10KEYS", 10),
    ("14KEYS", 14),
    ("9KEYS", 9),
    ("24KEYS", 25),
    ("24KEYS DOUBLE", 50),
];

const SCRATCH_MODE_LABELS: &[&str] = &["Ver. 2", "Ver. 1"];

fn combo_i32(ui: &mut egui::Ui, label: &str, value: &mut i32, labels: &[&str], dirty: &mut bool) {
    let prev = *value;
    let selected = labels.get(*value as usize).unwrap_or(&"Unknown");
    egui::ComboBox::from_label(label)
        .selected_text(*selected)
        .show_ui(ui, |ui| {
            for (i, &lbl) in labels.iter().enumerate() {
                ui.selectable_value(value, i as i32, lbl);
            }
        });
    if *value != prev {
        *dirty = true;
    }
}

fn checkbox(ui: &mut egui::Ui, value: &mut bool, label: &str, dirty: &mut bool) {
    if ui.checkbox(value, label).changed() {
        *dirty = true;
    }
}

fn drag_i32(ui: &mut egui::Ui, label: &str, value: &mut i32, min: i32, max: i32, dirty: &mut bool) {
    let prev = *value;
    clamped_i32(ui, label, value, min, max);
    if *value != prev {
        *dirty = true;
    }
}

#[derive(Clone)]
struct ControllerState {
    name: String,
    jkoc_hack: bool,
    analog_scratch: bool,
    analog_scratch_threshold: i32,
    analog_scratch_mode: i32,
}

#[derive(Clone)]
struct PerModeInputState {
    keyboard_duration: i32,
    mouse_scratch_enabled: bool,
    mouse_scratch_time_threshold: i32,
    mouse_scratch_distance: i32,
    mouse_scratch_mode: i32,
    controllers: Vec<ControllerState>,
}

impl Default for PerModeInputState {
    fn default() -> Self {
        Self {
            keyboard_duration: 16,
            mouse_scratch_enabled: false,
            mouse_scratch_time_threshold: 150,
            mouse_scratch_distance: 12,
            mouse_scratch_mode: 0,
            controllers: Vec::new(),
        }
    }
}

impl PerModeInputState {
    fn from_play_mode_config(pmc: &bms_config::PlayModeConfig) -> Self {
        let kb = &pmc.keyboard;
        let ms = &kb.mouse_scratch_config;
        Self {
            keyboard_duration: kb.duration,
            mouse_scratch_enabled: ms.mouse_scratch_enabled,
            mouse_scratch_time_threshold: ms.mouse_scratch_time_threshold,
            mouse_scratch_distance: ms.mouse_scratch_distance,
            mouse_scratch_mode: ms.mouse_scratch_mode,
            controllers: pmc
                .controller
                .iter()
                .map(|c| ControllerState {
                    name: c.name.clone(),
                    jkoc_hack: c.jkoc_hack,
                    analog_scratch: c.analog_scratch,
                    analog_scratch_threshold: c.analog_scratch_threshold,
                    analog_scratch_mode: c.analog_scratch_mode,
                })
                .collect(),
        }
    }

    fn apply_to_play_mode_config(&self, pmc: &mut bms_config::PlayModeConfig) {
        pmc.keyboard.duration = self.keyboard_duration;
        let ms = &mut pmc.keyboard.mouse_scratch_config;
        ms.mouse_scratch_enabled = self.mouse_scratch_enabled;
        ms.mouse_scratch_time_threshold = self.mouse_scratch_time_threshold;
        ms.mouse_scratch_distance = self.mouse_scratch_distance;
        ms.mouse_scratch_mode = self.mouse_scratch_mode;

        for (i, ctrl) in self.controllers.iter().enumerate() {
            if let Some(c) = pmc.controller.get_mut(i) {
                c.jkoc_hack = ctrl.jkoc_hack;
                c.analog_scratch = ctrl.analog_scratch;
                c.analog_scratch_threshold = ctrl.analog_scratch_threshold;
                c.analog_scratch_mode = ctrl.analog_scratch_mode;
            }
        }
    }
}

pub struct InputPanel {
    selected_mode: usize,
    mode_states: [PerModeInputState; 7],
    musicselectinput: i32,
    dirty: bool,
}

impl Default for InputPanel {
    fn default() -> Self {
        Self {
            selected_mode: 1, // 7KEYS
            mode_states: std::array::from_fn(|_| PerModeInputState::default()),
            musicselectinput: 0,
            dirty: false,
        }
    }
}

impl InputPanel {
    fn ui_keyboard_section(&mut self, ui: &mut egui::Ui) {
        let idx = self.selected_mode;
        let dirty = &mut self.dirty;
        let state = &mut self.mode_states[idx];

        ui.label("Key assignment is configured via the in-game Key Config screen.");
        ui.add_space(4.0);
        drag_i32(
            ui,
            "Keyboard Input Duration",
            &mut state.keyboard_duration,
            1,
            1000,
            dirty,
        );
    }

    fn ui_mouse_scratch_section(&mut self, ui: &mut egui::Ui) {
        let idx = self.selected_mode;
        let dirty = &mut self.dirty;
        let state = &mut self.mode_states[idx];

        checkbox(
            ui,
            &mut state.mouse_scratch_enabled,
            "Enable Mouse Scratch",
            dirty,
        );

        if state.mouse_scratch_enabled {
            drag_i32(
                ui,
                "Time Threshold",
                &mut state.mouse_scratch_time_threshold,
                1,
                1000,
                dirty,
            );
            drag_i32(
                ui,
                "Distance",
                &mut state.mouse_scratch_distance,
                1,
                100,
                dirty,
            );
            combo_i32(
                ui,
                "Scratch Mode",
                &mut state.mouse_scratch_mode,
                SCRATCH_MODE_LABELS,
                dirty,
            );
        }
    }

    fn ui_controllers_section(&mut self, ui: &mut egui::Ui) {
        let idx = self.selected_mode;
        let dirty = &mut self.dirty;
        let state = &mut self.mode_states[idx];

        if state.controllers.is_empty() {
            ui.label("No controllers configured.");
            return;
        }

        for (ci, ctrl) in state.controllers.iter_mut().enumerate() {
            let header = if ctrl.name.is_empty() {
                format!("Controller {}", ci + 1)
            } else {
                ctrl.name.clone()
            };

            egui::CollapsingHeader::new(header)
                .id_salt(format!("ctrl_{ci}"))
                .show(ui, |ui| {
                    if !ctrl.name.is_empty() {
                        ui.label(format!("Name: {}", ctrl.name));
                    }

                    checkbox(ui, &mut ctrl.jkoc_hack, "JKOC Hack", dirty);
                    checkbox(ui, &mut ctrl.analog_scratch, "Analog Scratch", dirty);

                    if ctrl.analog_scratch {
                        drag_i32(
                            ui,
                            "Analog Scratch Threshold",
                            &mut ctrl.analog_scratch_threshold,
                            1,
                            1000,
                            dirty,
                        );

                        let prev = ctrl.analog_scratch_mode;
                        let selected = SCRATCH_MODE_LABELS
                            .get(ctrl.analog_scratch_mode as usize)
                            .unwrap_or(&"Unknown");
                        egui::ComboBox::from_id_salt(format!("analog_mode_{ci}"))
                            .selected_text(*selected)
                            .show_ui(ui, |ui| {
                                for (i, &lbl) in SCRATCH_MODE_LABELS.iter().enumerate() {
                                    ui.selectable_value(
                                        &mut ctrl.analog_scratch_mode,
                                        i as i32,
                                        lbl,
                                    );
                                }
                            });
                        if ctrl.analog_scratch_mode != prev {
                            *dirty = true;
                        }
                    }
                });
        }
    }
}

impl LauncherPanel for InputPanel {
    fn tab(&self) -> Tab {
        Tab::Input
    }

    fn load(&mut self, _config: &Config, player_config: &PlayerConfig) {
        for (i, &(_, mode_id)) in PLAY_MODE_LABELS.iter().enumerate() {
            self.mode_states[i] =
                PerModeInputState::from_play_mode_config(player_config.play_config(mode_id));
        }
        self.musicselectinput = player_config.musicselectinput;
        self.dirty = false;
    }

    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.heading("Input Settings");
        ui.separator();

        // Play Mode selector
        let prev = self.selected_mode;
        egui::ComboBox::from_label("Play Mode")
            .selected_text(PLAY_MODE_LABELS[self.selected_mode].0)
            .show_ui(ui, |ui| {
                for (i, &(lbl, _)) in PLAY_MODE_LABELS.iter().enumerate() {
                    ui.selectable_value(&mut self.selected_mode, i, lbl);
                }
            });
        if self.selected_mode != prev {
            self.dirty = true;
        }

        ui.separator();

        egui::ScrollArea::vertical().show(ui, |ui| {
            egui::CollapsingHeader::new("Keyboard")
                .default_open(true)
                .show(ui, |ui| {
                    self.ui_keyboard_section(ui);
                });

            egui::CollapsingHeader::new("Mouse Scratch")
                .default_open(true)
                .show(ui, |ui| {
                    self.ui_mouse_scratch_section(ui);
                });

            egui::CollapsingHeader::new("Controllers")
                .default_open(true)
                .show(ui, |ui| {
                    self.ui_controllers_section(ui);
                });

            ui.separator();

            // Music Select Input (global setting)
            let prev = self.musicselectinput;
            egui::ComboBox::from_label("Music Select Input")
                .selected_text(match self.musicselectinput {
                    0 => "Keyboard",
                    _ => "Controller",
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.musicselectinput, 0, "Keyboard");
                    ui.selectable_value(&mut self.musicselectinput, 1, "Controller");
                });
            if self.musicselectinput != prev {
                self.dirty = true;
            }
        });
    }

    fn apply(&self, _config: &mut Config, player_config: &mut PlayerConfig) {
        for (i, &(_, mode_id)) in PLAY_MODE_LABELS.iter().enumerate() {
            self.mode_states[i].apply_to_play_mode_config(player_config.play_config_mut(mode_id));
        }
        player_config.musicselectinput = self.musicselectinput;
    }

    fn has_changes(&self) -> bool {
        self.dirty
    }
}
