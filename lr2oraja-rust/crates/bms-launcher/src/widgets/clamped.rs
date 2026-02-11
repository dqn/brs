/// Drag value widget with min/max clamping for i32.
pub fn clamped_i32(ui: &mut egui::Ui, label: &str, value: &mut i32, min: i32, max: i32) {
    ui.horizontal(|ui| {
        ui.label(label);
        ui.add(egui::DragValue::new(value).range(min..=max));
    });
}

/// Drag value widget with min/max clamping for f32.
pub fn clamped_f32(
    ui: &mut egui::Ui,
    label: &str,
    value: &mut f32,
    min: f32,
    max: f32,
    speed: f64,
) {
    ui.horizontal(|ui| {
        ui.label(label);
        ui.add(egui::DragValue::new(value).range(min..=max).speed(speed));
    });
}
