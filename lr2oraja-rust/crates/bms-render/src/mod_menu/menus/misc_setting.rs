// MiscSetting menu — Lift/Hidden/LaneCover/Constant settings.
//
// Corresponds to Java `MiscSettingMenu.java`.
// Values are stored as integers (0-1000) for UI display, matching
// the Java implementation. Conversion to float (0.0-1.0) happens
// when syncing to PlayConfig.

use super::super::notify;

pub const NOTIFICATION_POSITIONS: &[&str] = &[
    "TopLeft",
    "TopCenter",
    "TopRight",
    "BottomLeft",
    "BottomCenter",
    "BottomRight",
    "Center",
];

#[derive(Debug, Clone)]
pub struct MiscSettingState {
    // Notification
    pub notification_position: usize,

    // Lift
    pub enable_lift: bool,
    pub lift_value: i32, // 0-1000

    // Hidden
    pub enable_hidden: bool,
    pub hidden_value: i32, // 0-1000

    // Lane Cover
    pub enable_lanecover: bool,
    pub lanecover_value: i32, // 0-1000
    pub lanecover_margin_low: f32,
    pub lanecover_margin_high: f32,
    pub lanecover_switch_duration: i32,

    // Constant
    pub enable_constant: bool,
    pub constant_fadein: i32,
}

impl Default for MiscSettingState {
    fn default() -> Self {
        Self {
            notification_position: 0,
            enable_lift: false,
            lift_value: 100, // 0.1 * 1000
            enable_hidden: false,
            hidden_value: 100,
            enable_lanecover: true,
            lanecover_value: 200, // 0.2 * 1000
            lanecover_margin_low: 0.001,
            lanecover_margin_high: 0.01,
            lanecover_switch_duration: 500,
            enable_constant: false,
            constant_fadein: 100,
        }
    }
}

impl MiscSettingState {
    /// Load values from a PlayConfig.
    pub fn load_from_play_config(&mut self, config: &bms_config::PlayConfig) {
        self.enable_lift = config.enablelift;
        self.lift_value = (config.lift * 1000.0) as i32;
        self.enable_hidden = config.enablehidden;
        self.hidden_value = (config.hidden * 1000.0) as i32;
        self.enable_lanecover = config.enablelanecover;
        self.lanecover_value = (config.lanecover * 1000.0) as i32;
        self.lanecover_margin_low = config.lanecovermarginlow;
        self.lanecover_margin_high = config.lanecovermarginhigh;
        self.lanecover_switch_duration = config.lanecoverswitchduration;
        self.enable_constant = config.enable_constant;
        self.constant_fadein = config.constant_fadein_time;
    }

    /// Apply current values to a PlayConfig.
    pub fn apply_to_play_config(&self, config: &mut bms_config::PlayConfig) {
        config.enablelift = self.enable_lift;
        config.lift = self.lift_value as f32 / 1000.0;
        config.enablehidden = self.enable_hidden;
        config.hidden = self.hidden_value as f32 / 1000.0;
        config.enablelanecover = self.enable_lanecover;
        config.lanecover = self.lanecover_value as f32 / 1000.0;
        config.lanecovermarginlow = self.lanecover_margin_low;
        config.lanecovermarginhigh = self.lanecover_margin_high;
        config.lanecoverswitchduration = self.lanecover_switch_duration;
        config.enable_constant = self.enable_constant;
        config.constant_fadein_time = self.constant_fadein;
    }
}

pub fn render(ctx: &egui::Context, open: &mut bool, state: &mut MiscSettingState) {
    egui::Window::new("Misc Settings")
        .open(open)
        .resizable(false)
        .show(ctx, |ui| {
            // Notification position
            egui::ComboBox::from_label("Notification Positions")
                .selected_text(NOTIFICATION_POSITIONS[state.notification_position])
                .show_ui(ui, |ui| {
                    for (i, &pos) in NOTIFICATION_POSITIONS.iter().enumerate() {
                        ui.selectable_value(&mut state.notification_position, i, pos);
                    }
                });

            ui.separator();

            // Lift
            ui.horizontal(|ui| {
                ui.checkbox(&mut state.enable_lift, "Enable Lift");
                ui.add(
                    egui::DragValue::new(&mut state.lift_value)
                        .range(0..=1000)
                        .prefix("Lift: "),
                );
            });

            // Hidden
            ui.horizontal(|ui| {
                ui.checkbox(&mut state.enable_hidden, "Enable Hidden");
                ui.add(
                    egui::DragValue::new(&mut state.hidden_value)
                        .range(0..=1000)
                        .prefix("Hidden: "),
                );
            });

            // Lane Cover
            ui.checkbox(&mut state.enable_lanecover, "Enable LaneCover");
            ui.add(
                egui::DragValue::new(&mut state.lanecover_value)
                    .range(0..=1000)
                    .prefix("Lane Cover: "),
            );
            ui.add(
                egui::DragValue::new(&mut state.lanecover_margin_low)
                    .range(0.0..=1.0)
                    .speed(0.001)
                    .prefix("Margin (low): "),
            );
            ui.add(
                egui::DragValue::new(&mut state.lanecover_margin_high)
                    .range(0.0..=1.0)
                    .speed(0.001)
                    .prefix("Margin (high): "),
            );
            ui.add(
                egui::DragValue::new(&mut state.lanecover_switch_duration)
                    .range(0..=1000000)
                    .prefix("Switch Duration: "),
            );

            ui.separator();

            // Constant
            ui.horizontal(|ui| {
                ui.checkbox(&mut state.enable_constant, "Enable Constant");
                ui.add(
                    egui::DragValue::new(&mut state.constant_fadein)
                        .range(-1000..=1000)
                        .prefix("Fade-in: "),
                );
            });
        });

    // Sync notification position change
    let _ = notify::ToastPos::from_index(state.notification_position);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_values() {
        let state = MiscSettingState::default();
        assert!(!state.enable_lift);
        assert_eq!(state.lift_value, 100);
        assert!(state.enable_lanecover);
        assert_eq!(state.lanecover_value, 200);
    }

    #[test]
    fn load_and_apply_roundtrip() {
        let original = bms_config::PlayConfig::default();
        let mut state = MiscSettingState::default();
        state.load_from_play_config(&original);

        let mut restored = bms_config::PlayConfig::default();
        state.apply_to_play_config(&mut restored);

        assert_eq!(original.enablelift, restored.enablelift);
        assert_eq!(original.enablehidden, restored.enablehidden);
        assert_eq!(original.enablelanecover, restored.enablelanecover);
        assert_eq!(original.enable_constant, restored.enable_constant);
        // Float values may have minor precision loss from i32 conversion
        assert!((original.lift - restored.lift).abs() < 0.001);
        assert!((original.hidden - restored.hidden).abs() < 0.001);
        assert!((original.lanecover - restored.lanecover).abs() < 0.001);
    }
}
