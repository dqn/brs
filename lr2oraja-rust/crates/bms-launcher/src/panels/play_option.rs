use bms_config::{Config, PlayerConfig};

use crate::panel::LauncherPanel;
use crate::tab::Tab;
use crate::widgets::clamped::{clamped_f32, clamped_f64, clamped_i32};

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

const FIX_HISPEED_LABELS: &[&str] = &["Off", "Start BPM", "Max BPM", "Main BPM", "Min BPM"];

const LONGNOTE_MODE_LABELS: &[&str] = &["Off", "Remove", "Add LN", "Add CN", "Add HCN", "Add All"];

const SEVEN_TO_NINE_PATTERN_LABELS: &[&str] = &[
    "Off",
    "SC1KEY2~8",
    "SC1KEY3~9",
    "SC2KEY3~9",
    "SC8KEY1~7",
    "SC9KEY1~7",
    "SC9KEY2~8",
];

const SEVEN_TO_NINE_TYPE_LABELS: &[&str] = &["As-is", "No Mashing", "Alternation"];

const JUDGE_TYPE_VALUES: &[&str] = &["Combo", "Duration", "Lowest", "Score"];
const JUDGE_ALGORITHM_LABELS: &[&str] =
    &["LR2 (Combo)", "AC (Duration)", "Bottom (Lowest)", "Score"];

const AUTOSAVE_REPLAY_LABELS: &[&str] = &[
    "None",
    "Better Score",
    "Better/Same Score",
    "Better Misscount",
    "Better/Same Misscount",
    "Better Combo",
    "Better/Same Combo",
    "Better Lamp",
    "Better/Same Lamp",
    "Better All",
    "Always",
];

const BOTTOM_GAUGE_LABELS: &[&str] = &["Assist Easy", "Easy", "Normal"];

const PLAY_MODE_LABELS: &[(&str, i32)] = &[
    ("5KEYS", 5),
    ("7KEYS", 7),
    ("10KEYS", 10),
    ("14KEYS", 14),
    ("9KEYS", 9),
    ("24KEYS", 25),
    ("24KEYS DOUBLE", 50),
];

/// Per-mode play configuration state (maps to PlayConfig fields).
#[derive(Clone)]
struct PerModeState {
    hispeed: f32,
    duration: i32,
    fixhispeed: i32,
    hispeedmargin: f32,
    hispeedautoadjust: bool,
    enablelanecover: bool,
    lanecover: i32,
    lanecovermarginlow: i32,
    lanecovermarginhigh: i32,
    lanecoverswitchduration: i32,
    enablelift: bool,
    lift: i32,
    enablehidden: bool,
    hidden: i32,
    enable_constant: bool,
    constant_fadein_time: i32,
    judgetype: i32,
}

impl PerModeState {
    fn from_play_config(pc: &bms_config::PlayConfig) -> Self {
        Self {
            hispeed: pc.hispeed,
            duration: pc.duration,
            fixhispeed: pc.fixhispeed,
            hispeedmargin: pc.hispeedmargin,
            hispeedautoadjust: pc.hispeedautoadjust,
            enablelanecover: pc.enablelanecover,
            lanecover: (pc.lanecover * 1000.0).round() as i32,
            lanecovermarginlow: (pc.lanecovermarginlow * 1000.0).round() as i32,
            lanecovermarginhigh: (pc.lanecovermarginhigh * 1000.0).round() as i32,
            lanecoverswitchduration: pc.lanecoverswitchduration,
            enablelift: pc.enablelift,
            lift: (pc.lift * 1000.0).round() as i32,
            enablehidden: pc.enablehidden,
            hidden: (pc.hidden * 1000.0).round() as i32,
            enable_constant: pc.enable_constant,
            constant_fadein_time: pc.constant_fadein_time,
            judgetype: JUDGE_TYPE_VALUES
                .iter()
                .position(|&s| s == pc.judgetype)
                .unwrap_or(0) as i32,
        }
    }

    fn apply_to_play_config(&self, pc: &mut bms_config::PlayConfig) {
        pc.hispeed = self.hispeed;
        pc.duration = self.duration;
        pc.fixhispeed = self.fixhispeed;
        pc.hispeedmargin = self.hispeedmargin;
        pc.hispeedautoadjust = self.hispeedautoadjust;
        pc.enablelanecover = self.enablelanecover;
        pc.lanecover = self.lanecover as f32 / 1000.0;
        pc.lanecovermarginlow = self.lanecovermarginlow as f32 / 1000.0;
        pc.lanecovermarginhigh = self.lanecovermarginhigh as f32 / 1000.0;
        pc.lanecoverswitchduration = self.lanecoverswitchduration;
        pc.enablelift = self.enablelift;
        pc.lift = self.lift as f32 / 1000.0;
        pc.enablehidden = self.enablehidden;
        pc.hidden = self.hidden as f32 / 1000.0;
        pc.enable_constant = self.enable_constant;
        pc.constant_fadein_time = self.constant_fadein_time;
        pc.judgetype = JUDGE_TYPE_VALUES
            .get(self.judgetype as usize)
            .unwrap_or(&"Combo")
            .to_string();
    }
}

impl Default for PerModeState {
    fn default() -> Self {
        Self::from_play_config(&bms_config::PlayConfig::default())
    }
}

pub struct PlayOptionPanel {
    // Mode selection
    selected_mode: usize,
    mode_states: [PerModeState; 7],
    // Global fields (PlayerConfig)
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
    bpmguide: bool,
    showjudgearea: bool,
    markprocessednote: bool,
    showhiddennote: bool,
    showpastnote: bool,
    is_guide_se: bool,
    is_window_hold: bool,
    chart_preview: bool,
    key_judge_window_rate_perfect_great: i32,
    key_judge_window_rate_great: i32,
    key_judge_window_rate_good: i32,
    scratch_judge_window_rate_perfect_great: i32,
    scratch_judge_window_rate_great: i32,
    scratch_judge_window_rate_good: i32,
    hran_threshold_bpm: i32,
    extranote_depth: i32,
    longnote_mode: i32,
    longnote_rate: f64,
    forcedcnendings: bool,
    seven_to_nine_pattern: i32,
    seven_to_nine_type: i32,
    exit_press_duration: i32,
    targetid: String,
    targetlist: Vec<String>,
    autosavereplay: [i32; 4],
    dirty: bool,
}

fn autosavereplay_from_option(opt: &Option<Vec<i32>>) -> [i32; 4] {
    let v = opt.as_deref().unwrap_or(&[0; 4]);
    [
        v.first().copied().unwrap_or(0),
        v.get(1).copied().unwrap_or(0),
        v.get(2).copied().unwrap_or(0),
        v.get(3).copied().unwrap_or(0),
    ]
}

impl Default for PlayOptionPanel {
    fn default() -> Self {
        let pc = PlayerConfig::default();
        Self {
            selected_mode: 1, // 7KEYS
            mode_states: std::array::from_fn(|i| {
                let mode_id = PLAY_MODE_LABELS[i].1;
                PerModeState::from_play_config(&pc.play_config(mode_id).playconfig)
            }),
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
            bpmguide: pc.bpmguide,
            showjudgearea: pc.showjudgearea,
            markprocessednote: pc.markprocessednote,
            showhiddennote: pc.showhiddennote,
            showpastnote: pc.showpastnote,
            is_guide_se: pc.is_guide_se,
            is_window_hold: pc.is_window_hold,
            chart_preview: pc.chart_preview,
            key_judge_window_rate_perfect_great: pc.key_judge_window_rate_perfect_great,
            key_judge_window_rate_great: pc.key_judge_window_rate_great,
            key_judge_window_rate_good: pc.key_judge_window_rate_good,
            scratch_judge_window_rate_perfect_great: pc.scratch_judge_window_rate_perfect_great,
            scratch_judge_window_rate_great: pc.scratch_judge_window_rate_great,
            scratch_judge_window_rate_good: pc.scratch_judge_window_rate_good,
            hran_threshold_bpm: pc.hran_threshold_bpm,
            extranote_depth: pc.extranote_depth,
            longnote_mode: pc.longnote_mode,
            longnote_rate: pc.longnote_rate,
            forcedcnendings: pc.forcedcnendings,
            seven_to_nine_pattern: pc.seven_to_nine_pattern,
            seven_to_nine_type: pc.seven_to_nine_type,
            exit_press_duration: pc.exit_press_duration,
            targetid: pc.targetid,
            targetlist: pc.targetlist,
            autosavereplay: autosavereplay_from_option(&pc.autosavereplay),
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

fn drag_f32(
    ui: &mut egui::Ui,
    label: &str,
    value: &mut f32,
    min: f32,
    max: f32,
    speed: f64,
    dirty: &mut bool,
) {
    let prev = *value;
    clamped_f32(ui, label, value, min, max, speed);
    if *value != prev {
        *dirty = true;
    }
}

fn drag_f64(
    ui: &mut egui::Ui,
    label: &str,
    value: &mut f64,
    min: f64,
    max: f64,
    speed: f64,
    dirty: &mut bool,
) {
    let prev = *value;
    clamped_f64(ui, label, value, min, max, speed);
    if (*value - prev).abs() > f64::EPSILON {
        *dirty = true;
    }
}

impl PlayOptionPanel {
    fn ui_hispeed_section(&mut self, ui: &mut egui::Ui) {
        let idx = self.selected_mode;
        let dirty = &mut self.dirty;
        let ms = &mut self.mode_states[idx];

        drag_f32(ui, "HI-SPEED", &mut ms.hispeed, 0.01, 20.0, 0.01, dirty);
        drag_i32(ui, "Duration", &mut ms.duration, 1, 10000, dirty);
        combo_i32(
            ui,
            "Fix HI-SPEED",
            &mut ms.fixhispeed,
            FIX_HISPEED_LABELS,
            dirty,
        );
        drag_f32(
            ui,
            "HI-SPEED Margin",
            &mut ms.hispeedmargin,
            0.0,
            10.0,
            0.01,
            dirty,
        );
        checkbox(ui, &mut ms.hispeedautoadjust, "Auto Adjust HI-SPEED", dirty);

        ui.separator();

        checkbox(ui, &mut ms.enablelanecover, "Lane Cover", dirty);
        drag_i32(
            ui,
            "Lane Cover (\u{2030})",
            &mut ms.lanecover,
            0,
            1000,
            dirty,
        );
        drag_i32(
            ui,
            "Margin Low (\u{2030})",
            &mut ms.lanecovermarginlow,
            0,
            1000,
            dirty,
        );
        drag_i32(
            ui,
            "Margin High (\u{2030})",
            &mut ms.lanecovermarginhigh,
            0,
            1000,
            dirty,
        );
        drag_i32(
            ui,
            "Switch Duration",
            &mut ms.lanecoverswitchduration,
            0,
            1000000,
            dirty,
        );

        ui.separator();

        checkbox(ui, &mut ms.enablelift, "Lift", dirty);
        drag_i32(ui, "Lift (\u{2030})", &mut ms.lift, 0, 1000, dirty);

        checkbox(ui, &mut ms.enablehidden, "Hidden", dirty);
        drag_i32(ui, "Hidden (\u{2030})", &mut ms.hidden, 0, 1000, dirty);

        ui.separator();

        checkbox(ui, &mut ms.enable_constant, "Constant", dirty);
        drag_i32(
            ui,
            "Fade-in Time",
            &mut ms.constant_fadein_time,
            -1000,
            1000,
            dirty,
        );
    }

    fn ui_note_options_section(&mut self, ui: &mut egui::Ui) {
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

        combo_i32(
            ui,
            "LN Type",
            &mut self.lnmode,
            LN_MODE_LABELS,
            &mut self.dirty,
        );
        checkbox(
            ui,
            &mut self.forcedcnendings,
            "Forced CN Endings",
            &mut self.dirty,
        );
    }

    fn ui_timing_judge_section(&mut self, ui: &mut egui::Ui) {
        drag_i32(
            ui,
            "Judge Timing",
            &mut self.judgetiming,
            -500,
            500,
            &mut self.dirty,
        );
        checkbox(
            ui,
            &mut self.notes_display_timing_auto_adjust,
            "Auto Adjust Display Timing",
            &mut self.dirty,
        );

        let idx = self.selected_mode;
        combo_i32(
            ui,
            "Judge Algorithm",
            &mut self.mode_states[idx].judgetype,
            JUDGE_ALGORITHM_LABELS,
            &mut self.dirty,
        );

        checkbox(
            ui,
            &mut self.custom_judge,
            "Custom Judge Window",
            &mut self.dirty,
        );

        ui.add_enabled_ui(self.custom_judge, |ui| {
            ui.group(|ui| {
                ui.label("Key Judge Window Rate (%)");
                drag_i32(
                    ui,
                    "PG/GR",
                    &mut self.key_judge_window_rate_perfect_great,
                    25,
                    400,
                    &mut self.dirty,
                );
                drag_i32(
                    ui,
                    "GR",
                    &mut self.key_judge_window_rate_great,
                    0,
                    400,
                    &mut self.dirty,
                );
                drag_i32(
                    ui,
                    "GD",
                    &mut self.key_judge_window_rate_good,
                    0,
                    400,
                    &mut self.dirty,
                );

                ui.separator();

                ui.label("Scratch Judge Window Rate (%)");
                drag_i32(
                    ui,
                    "PG/GR",
                    &mut self.scratch_judge_window_rate_perfect_great,
                    25,
                    400,
                    &mut self.dirty,
                );
                drag_i32(
                    ui,
                    "GR",
                    &mut self.scratch_judge_window_rate_great,
                    0,
                    400,
                    &mut self.dirty,
                );
                drag_i32(
                    ui,
                    "GD",
                    &mut self.scratch_judge_window_rate_good,
                    0,
                    400,
                    &mut self.dirty,
                );
            });
        });

        drag_i32(
            ui,
            "H-RANDOM Threshold BPM",
            &mut self.hran_threshold_bpm,
            1,
            1000,
            &mut self.dirty,
        );
    }

    fn ui_gauge_target_section(&mut self, ui: &mut egui::Ui) {
        combo_i32(
            ui,
            "Gauge Auto Shift",
            &mut self.gauge_auto_shift,
            GAUGE_AUTO_SHIFT_LABELS,
            &mut self.dirty,
        );
        combo_i32(
            ui,
            "Bottom Shiftable Gauge",
            &mut self.bottom_shiftable_gauge,
            BOTTOM_GAUGE_LABELS,
            &mut self.dirty,
        );

        let prev = self.targetid.clone();
        let selected = self.targetid.clone();
        egui::ComboBox::from_label("Target Score")
            .selected_text(selected)
            .show_ui(ui, |ui| {
                for target in &self.targetlist {
                    ui.selectable_value(&mut self.targetid, target.clone(), target.as_str());
                }
            });
        if self.targetid != prev {
            self.dirty = true;
        }
    }

    fn ui_note_modifier_section(&mut self, ui: &mut egui::Ui) {
        combo_i32(
            ui,
            "Mine Mode",
            &mut self.mine_mode,
            MINE_MODE_LABELS,
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
            "LongNote Mode",
            &mut self.longnote_mode,
            LONGNOTE_MODE_LABELS,
            &mut self.dirty,
        );
        drag_f64(
            ui,
            "LongNote Rate",
            &mut self.longnote_rate,
            0.0,
            1.0,
            0.01,
            &mut self.dirty,
        );
        drag_i32(
            ui,
            "Extra Note Depth",
            &mut self.extranote_depth,
            0,
            100,
            &mut self.dirty,
        );
    }

    fn ui_assist_display_section(&mut self, ui: &mut egui::Ui) {
        checkbox(ui, &mut self.bpmguide, "BPM Guide", &mut self.dirty);
        checkbox(ui, &mut self.is_guide_se, "Guide SE", &mut self.dirty);
        checkbox(
            ui,
            &mut self.showjudgearea,
            "Show Judge Area",
            &mut self.dirty,
        );
        checkbox(
            ui,
            &mut self.markprocessednote,
            "Mark Processed Note",
            &mut self.dirty,
        );
        checkbox(
            ui,
            &mut self.showhiddennote,
            "Show Hidden Note",
            &mut self.dirty,
        );
        checkbox(
            ui,
            &mut self.showpastnote,
            "Show Past Note",
            &mut self.dirty,
        );
        checkbox(ui, &mut self.is_window_hold, "Window Hold", &mut self.dirty);
        checkbox(
            ui,
            &mut self.chart_preview,
            "Chart Preview",
            &mut self.dirty,
        );
    }

    fn ui_7to9_section(&mut self, ui: &mut egui::Ui) {
        combo_i32(
            ui,
            "7to9 Pattern",
            &mut self.seven_to_nine_pattern,
            SEVEN_TO_NINE_PATTERN_LABELS,
            &mut self.dirty,
        );
        combo_i32(
            ui,
            "7to9 Type",
            &mut self.seven_to_nine_type,
            SEVEN_TO_NINE_TYPE_LABELS,
            &mut self.dirty,
        );
    }

    fn ui_replay_section(&mut self, ui: &mut egui::Ui) {
        for i in 0..4 {
            let label = format!("Auto Save Replay {}", i + 1);
            combo_i32(
                ui,
                &label,
                &mut self.autosavereplay[i],
                AUTOSAVE_REPLAY_LABELS,
                &mut self.dirty,
            );
        }
    }

    fn ui_misc_section(&mut self, ui: &mut egui::Ui) {
        drag_i32(
            ui,
            "Exit Press Duration (ms)",
            &mut self.exit_press_duration,
            0,
            100000,
            &mut self.dirty,
        );
    }
}

impl LauncherPanel for PlayOptionPanel {
    fn tab(&self) -> Tab {
        Tab::PlayOption
    }

    fn load(&mut self, _config: &Config, player_config: &PlayerConfig) {
        // Load per-mode states for all modes
        for (i, &(_, mode_id)) in PLAY_MODE_LABELS.iter().enumerate() {
            self.mode_states[i] =
                PerModeState::from_play_config(&player_config.play_config(mode_id).playconfig);
        }

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
        self.bpmguide = player_config.bpmguide;
        self.showjudgearea = player_config.showjudgearea;
        self.markprocessednote = player_config.markprocessednote;
        self.showhiddennote = player_config.showhiddennote;
        self.showpastnote = player_config.showpastnote;
        self.is_guide_se = player_config.is_guide_se;
        self.is_window_hold = player_config.is_window_hold;
        self.chart_preview = player_config.chart_preview;
        self.key_judge_window_rate_perfect_great =
            player_config.key_judge_window_rate_perfect_great;
        self.key_judge_window_rate_great = player_config.key_judge_window_rate_great;
        self.key_judge_window_rate_good = player_config.key_judge_window_rate_good;
        self.scratch_judge_window_rate_perfect_great =
            player_config.scratch_judge_window_rate_perfect_great;
        self.scratch_judge_window_rate_great = player_config.scratch_judge_window_rate_great;
        self.scratch_judge_window_rate_good = player_config.scratch_judge_window_rate_good;
        self.hran_threshold_bpm = player_config.hran_threshold_bpm;
        self.extranote_depth = player_config.extranote_depth;
        self.longnote_mode = player_config.longnote_mode;
        self.longnote_rate = player_config.longnote_rate;
        self.forcedcnendings = player_config.forcedcnendings;
        self.seven_to_nine_pattern = player_config.seven_to_nine_pattern;
        self.seven_to_nine_type = player_config.seven_to_nine_type;
        self.exit_press_duration = player_config.exit_press_duration;
        self.targetid = player_config.targetid.clone();
        self.targetlist = player_config.targetlist.clone();
        self.autosavereplay = autosavereplay_from_option(&player_config.autosavereplay);
        self.dirty = false;
    }

    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.heading("Play Option");
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
            egui::CollapsingHeader::new("HI-SPEED & Lane Cover")
                .default_open(true)
                .show(ui, |ui| {
                    self.ui_hispeed_section(ui);
                });

            egui::CollapsingHeader::new("Note Options")
                .default_open(true)
                .show(ui, |ui| {
                    self.ui_note_options_section(ui);
                });

            egui::CollapsingHeader::new("Timing & Judge")
                .default_open(true)
                .show(ui, |ui| {
                    self.ui_timing_judge_section(ui);
                });

            egui::CollapsingHeader::new("Gauge & Target")
                .default_open(false)
                .show(ui, |ui| {
                    self.ui_gauge_target_section(ui);
                });

            egui::CollapsingHeader::new("Note Modifier")
                .default_open(false)
                .show(ui, |ui| {
                    self.ui_note_modifier_section(ui);
                });

            egui::CollapsingHeader::new("Assist & Display")
                .default_open(false)
                .show(ui, |ui| {
                    self.ui_assist_display_section(ui);
                });

            egui::CollapsingHeader::new("7to9 Conversion")
                .default_open(false)
                .show(ui, |ui| {
                    self.ui_7to9_section(ui);
                });

            egui::CollapsingHeader::new("Replay")
                .default_open(false)
                .show(ui, |ui| {
                    self.ui_replay_section(ui);
                });

            egui::CollapsingHeader::new("Misc")
                .default_open(false)
                .show(ui, |ui| {
                    self.ui_misc_section(ui);
                });
        });
    }

    fn apply(&self, _config: &mut Config, player_config: &mut PlayerConfig) {
        // Apply per-mode states for all modes
        for (i, &(_, mode_id)) in PLAY_MODE_LABELS.iter().enumerate() {
            self.mode_states[i]
                .apply_to_play_config(&mut player_config.play_config_mut(mode_id).playconfig);
        }

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
        player_config.bpmguide = self.bpmguide;
        player_config.showjudgearea = self.showjudgearea;
        player_config.markprocessednote = self.markprocessednote;
        player_config.showhiddennote = self.showhiddennote;
        player_config.showpastnote = self.showpastnote;
        player_config.is_guide_se = self.is_guide_se;
        player_config.is_window_hold = self.is_window_hold;
        player_config.chart_preview = self.chart_preview;
        player_config.key_judge_window_rate_perfect_great =
            self.key_judge_window_rate_perfect_great;
        player_config.key_judge_window_rate_great = self.key_judge_window_rate_great;
        player_config.key_judge_window_rate_good = self.key_judge_window_rate_good;
        player_config.scratch_judge_window_rate_perfect_great =
            self.scratch_judge_window_rate_perfect_great;
        player_config.scratch_judge_window_rate_great = self.scratch_judge_window_rate_great;
        player_config.scratch_judge_window_rate_good = self.scratch_judge_window_rate_good;
        player_config.hran_threshold_bpm = self.hran_threshold_bpm;
        player_config.extranote_depth = self.extranote_depth;
        player_config.longnote_mode = self.longnote_mode;
        player_config.longnote_rate = self.longnote_rate;
        player_config.forcedcnendings = self.forcedcnendings;
        player_config.seven_to_nine_pattern = self.seven_to_nine_pattern;
        player_config.seven_to_nine_type = self.seven_to_nine_type;
        player_config.exit_press_duration = self.exit_press_duration;
        player_config.targetid = self.targetid.clone();
        player_config.autosavereplay = Some(self.autosavereplay.to_vec());
    }

    fn has_changes(&self) -> bool {
        self.dirty
    }
}
