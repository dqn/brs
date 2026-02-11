use bms_config::{Config, PlayerConfig};

use crate::panel::LauncherPanel;
use crate::tab::Tab;
use crate::widgets::clamped::clamped_i32;

const GAUGE_LABELS: &[&str] = &["Assist Easy", "Easy", "Normal", "Hard", "Ex-Hard", "Hazard"];

const RANDOM_LABELS: &[&str] = &[
    "Off",
    "Mirror",
    "Random",
    "R-Random",
    "S-Random",
    "Spiral",
    "H-Random",
    "All-SCR",
    "Random+",
    "S-Random+",
];

const LN_MODE_LABELS: &[&str] = &["Off", "LN", "CN"];

const SCROLL_MODE_LABELS: &[&str] = &["Off", "Fixed", "Proportional"];

const MINE_MODE_LABELS: &[&str] = &["Normal", "Random", "Off", "Attract", "Repel"];

const GAUGE_AUTO_SHIFT_LABELS: &[&str] = &[
    "None",
    "Continue",
    "Survival->Groove",
    "Best Clear",
    "Select and Under",
];

pub struct PlayOptionPanel {
    gauge: i32,
    random: i32,
    random2: i32,
    doubleoption: i32,
    judgetiming: i32,
    lnmode: i32,
    scroll_mode: i32,
    mine_mode: i32,
    gauge_auto_shift: i32,
    bottom_shiftable_gauge: i32,
    notes_display_timing_auto_adjust: bool,
    custom_judge: bool,
    dirty: bool,
}

impl Default for PlayOptionPanel {
    fn default() -> Self {
        let pc = PlayerConfig::default();
        Self {
            gauge: pc.gauge,
            random: pc.random,
            random2: pc.random2,
            doubleoption: pc.doubleoption,
            judgetiming: pc.judgetiming,
            lnmode: pc.lnmode,
            scroll_mode: pc.scroll_mode,
            mine_mode: pc.mine_mode,
            gauge_auto_shift: pc.gauge_auto_shift,
            bottom_shiftable_gauge: pc.bottom_shiftable_gauge,
            notes_display_timing_auto_adjust: pc.notes_display_timing_auto_adjust,
            custom_judge: pc.custom_judge,
            dirty: false,
        }
    }
}

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

impl LauncherPanel for PlayOptionPanel {
    fn tab(&self) -> Tab {
        Tab::PlayOption
    }

    fn load(&mut self, _config: &Config, player_config: &PlayerConfig) {
        self.gauge = player_config.gauge;
        self.random = player_config.random;
        self.random2 = player_config.random2;
        self.doubleoption = player_config.doubleoption;
        self.judgetiming = player_config.judgetiming;
        self.lnmode = player_config.lnmode;
        self.scroll_mode = player_config.scroll_mode;
        self.mine_mode = player_config.mine_mode;
        self.gauge_auto_shift = player_config.gauge_auto_shift;
        self.bottom_shiftable_gauge = player_config.bottom_shiftable_gauge;
        self.notes_display_timing_auto_adjust = player_config.notes_display_timing_auto_adjust;
        self.custom_judge = player_config.custom_judge;
        self.dirty = false;
    }

    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.heading("Play Option");
        ui.separator();

        combo_i32(ui, "Gauge", &mut self.gauge, GAUGE_LABELS, &mut self.dirty);
        combo_i32(
            ui,
            "Random (1P)",
            &mut self.random,
            RANDOM_LABELS,
            &mut self.dirty,
        );
        combo_i32(
            ui,
            "Random (2P)",
            &mut self.random2,
            RANDOM_LABELS,
            &mut self.dirty,
        );

        let prev = self.doubleoption;
        egui::ComboBox::from_label("Double Option")
            .selected_text(match self.doubleoption {
                0 => "Off",
                1 => "Flip",
                2 => "Battle",
                _ => "Battle+",
            })
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut self.doubleoption, 0, "Off");
                ui.selectable_value(&mut self.doubleoption, 1, "Flip");
                ui.selectable_value(&mut self.doubleoption, 2, "Battle");
                ui.selectable_value(&mut self.doubleoption, 3, "Battle+");
            });
        if self.doubleoption != prev {
            self.dirty = true;
        }

        ui.separator();

        let prev = self.judgetiming;
        clamped_i32(ui, "Judge Timing", &mut self.judgetiming, -500, 500);
        if self.judgetiming != prev {
            self.dirty = true;
        }

        if ui
            .checkbox(
                &mut self.notes_display_timing_auto_adjust,
                "Auto Adjust Display Timing",
            )
            .changed()
        {
            self.dirty = true;
        }

        if ui
            .checkbox(&mut self.custom_judge, "Custom Judge Window")
            .changed()
        {
            self.dirty = true;
        }

        ui.separator();

        combo_i32(
            ui,
            "LN Mode",
            &mut self.lnmode,
            LN_MODE_LABELS,
            &mut self.dirty,
        );
        combo_i32(
            ui,
            "Scroll Mode",
            &mut self.scroll_mode,
            SCROLL_MODE_LABELS,
            &mut self.dirty,
        );
        combo_i32(
            ui,
            "Mine Mode",
            &mut self.mine_mode,
            MINE_MODE_LABELS,
            &mut self.dirty,
        );

        ui.separator();

        combo_i32(
            ui,
            "Gauge Auto Shift",
            &mut self.gauge_auto_shift,
            GAUGE_AUTO_SHIFT_LABELS,
            &mut self.dirty,
        );
        let prev = self.bottom_shiftable_gauge;
        clamped_i32(
            ui,
            "Bottom Shiftable Gauge",
            &mut self.bottom_shiftable_gauge,
            0,
            5,
        );
        if self.bottom_shiftable_gauge != prev {
            self.dirty = true;
        }
    }

    fn apply(&self, _config: &mut Config, player_config: &mut PlayerConfig) {
        player_config.gauge = self.gauge;
        player_config.random = self.random;
        player_config.random2 = self.random2;
        player_config.doubleoption = self.doubleoption;
        player_config.judgetiming = self.judgetiming;
        player_config.lnmode = self.lnmode;
        player_config.scroll_mode = self.scroll_mode;
        player_config.mine_mode = self.mine_mode;
        player_config.gauge_auto_shift = self.gauge_auto_shift;
        player_config.bottom_shiftable_gauge = self.bottom_shiftable_gauge;
        player_config.notes_display_timing_auto_adjust = self.notes_display_timing_auto_adjust;
        player_config.custom_judge = self.custom_judge;
    }

    fn has_changes(&self) -> bool {
        self.dirty
    }
}
