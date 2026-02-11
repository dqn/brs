// SkinWidgetManager menu — edit skin object positions and sizes.
//
// Corresponds to Java `SkinWidgetManager.java`.
// Provides a table of skin widgets with position/size editing,
// undo history, and column visibility controls.
// Skin integration (load_from_skin / apply_to_skin) is deferred
// to a future phase; current implementation works with snapshot data.

use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangeField {
    X,
    Y,
    W,
    H,
}

impl ChangeField {
    pub fn label(self) -> &'static str {
        match self {
            Self::X => "x",
            Self::Y => "y",
            Self::W => "w",
            Self::H => "h",
        }
    }
}

#[derive(Debug, Clone)]
pub enum WidgetEvent {
    ChangeField {
        field: ChangeField,
        widget_name: String,
        dst_name: String,
        previous: f32,
        current: f32,
    },
    ToggleVisible {
        widget_name: String,
        was_visible: bool,
    },
}

impl WidgetEvent {
    pub fn widget_name(&self) -> &str {
        match self {
            Self::ChangeField { widget_name, .. } | Self::ToggleVisible { widget_name, .. } => {
                widget_name
            }
        }
    }

    pub fn description(&self) -> String {
        match self {
            Self::ChangeField {
                field,
                widget_name,
                dst_name,
                previous,
                current,
            } => {
                format!(
                    "{}.{}.{}: {} -> {}",
                    widget_name,
                    dst_name,
                    field.label(),
                    format_float(*previous),
                    format_float(*current),
                )
            }
            Self::ToggleVisible {
                widget_name,
                was_visible,
            } => {
                let action = if *was_visible { "hidden" } else { "shown" };
                format!("{}: {}", widget_name, action)
            }
        }
    }
}

fn format_float(v: f32) -> String {
    if v == v.trunc() {
        format!("{:.0}", v)
    } else {
        format!("{}", v)
    }
}

#[derive(Debug, Clone, Default)]
pub struct EventHistory {
    events: Vec<WidgetEvent>,
    name_to_events: HashMap<String, Vec<usize>>,
}

impl EventHistory {
    pub fn push(&mut self, event: WidgetEvent) {
        let idx = self.events.len();
        self.name_to_events
            .entry(event.widget_name().to_string())
            .or_default()
            .push(idx);
        self.events.push(event);
    }

    pub fn undo(&mut self) -> Option<WidgetEvent> {
        let event = self.events.pop()?;
        self.rebuild_name_index();
        Some(event)
    }

    pub fn has_event(&self, name: &str, field: ChangeField) -> bool {
        let Some(indices) = self.name_to_events.get(name) else {
            return false;
        };
        indices.iter().any(|&i| {
            matches!(
                &self.events[i],
                WidgetEvent::ChangeField { field: f, .. } if *f == field
            )
        })
    }

    pub fn events(&self) -> &[WidgetEvent] {
        &self.events
    }

    pub fn events_for_name(&self, name: &str) -> Vec<&WidgetEvent> {
        let Some(indices) = self.name_to_events.get(name) else {
            return Vec::new();
        };
        indices.iter().map(|&i| &self.events[i]).collect()
    }

    pub fn clear(&mut self) {
        self.events.clear();
        self.name_to_events.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    fn rebuild_name_index(&mut self) {
        self.name_to_events.clear();
        for (i, event) in self.events.iter().enumerate() {
            self.name_to_events
                .entry(event.widget_name().to_string())
                .or_default()
                .push(i);
        }
    }
}

#[derive(Debug, Clone)]
pub struct WidgetDestination {
    pub name: String,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

#[derive(Debug, Clone)]
pub struct SkinWidget {
    pub name: String,
    pub visible: bool,
    pub drawing: bool,
    pub destinations: Vec<WidgetDestination>,
}

#[derive(Debug, Clone)]
pub struct SkinWidgetManagerState {
    pub widgets: Vec<SkinWidget>,
    pub history: EventHistory,
    pub column_visible: [bool; 4],
    pub show_cursor_position: bool,
    pub active_tab: usize,
    // Edit popup state
    pub editing_widget: Option<(usize, usize)>, // (widget_idx, dst_idx)
    pub edit_x: f32,
    pub edit_y: f32,
    pub edit_w: f32,
    pub edit_h: f32,
}

impl Default for SkinWidgetManagerState {
    fn default() -> Self {
        Self {
            widgets: Vec::new(),
            history: EventHistory::default(),
            column_visible: [true; 4],
            show_cursor_position: false,
            active_tab: 0,
            editing_widget: None,
            edit_x: 0.0,
            edit_y: 0.0,
            edit_w: 0.0,
            edit_h: 0.0,
        }
    }
}

impl SkinWidgetManagerState {
    /// Start editing a destination, copying its current values to the edit fields.
    pub fn begin_edit(&mut self, widget_idx: usize, dst_idx: usize) {
        if let Some(dst) = self
            .widgets
            .get(widget_idx)
            .and_then(|w| w.destinations.get(dst_idx))
        {
            self.edit_x = dst.x;
            self.edit_y = dst.y;
            self.edit_w = dst.w;
            self.edit_h = dst.h;
            self.editing_widget = Some((widget_idx, dst_idx));
        }
    }

    /// Apply the edit field values to the destination, recording events for changes.
    pub fn submit_edit(&mut self) {
        let Some((widget_idx, dst_idx)) = self.editing_widget else {
            return;
        };
        let Some(widget) = self.widgets.get_mut(widget_idx) else {
            self.editing_widget = None;
            return;
        };
        let Some(dst) = widget.destinations.get_mut(dst_idx) else {
            self.editing_widget = None;
            return;
        };

        let fields = [
            (ChangeField::X, dst.x, self.edit_x),
            (ChangeField::Y, dst.y, self.edit_y),
            (ChangeField::W, dst.w, self.edit_w),
            (ChangeField::H, dst.h, self.edit_h),
        ];

        for (field, previous, current) in fields {
            if (previous - current).abs() > f32::EPSILON {
                self.history.push(WidgetEvent::ChangeField {
                    field,
                    widget_name: widget.name.clone(),
                    dst_name: dst.name.clone(),
                    previous,
                    current,
                });
            }
        }

        dst.x = self.edit_x;
        dst.y = self.edit_y;
        dst.w = self.edit_w;
        dst.h = self.edit_h;
        self.editing_widget = None;
    }

    /// Toggle visibility for a widget, recording an event.
    pub fn toggle_visible(&mut self, widget_idx: usize) {
        let Some(widget) = self.widgets.get_mut(widget_idx) else {
            return;
        };
        self.history.push(WidgetEvent::ToggleVisible {
            widget_name: widget.name.clone(),
            was_visible: widget.visible,
        });
        widget.visible = !widget.visible;
    }

    /// Export all widget changes as a formatted string.
    pub fn export_changes(&self) -> String {
        let mut lines = Vec::new();
        for event in self.history.events() {
            lines.push(event.description());
        }
        lines.join("\n")
    }
}

pub fn render(ctx: &egui::Context, open: &mut bool, state: &mut SkinWidgetManagerState) {
    egui::Window::new("Skin Widgets")
        .open(open)
        .resizable(true)
        .default_width(600.0)
        .show(ctx, |ui| {
            // Tab bar
            ui.horizontal(|ui| {
                if ui
                    .selectable_label(state.active_tab == 0, "SkinWidgets")
                    .clicked()
                {
                    state.active_tab = 0;
                }
                if ui
                    .selectable_label(state.active_tab == 1, "History")
                    .clicked()
                {
                    state.active_tab = 1;
                }
            });

            ui.separator();

            match state.active_tab {
                0 => render_widgets_tab(ui, state),
                1 => render_history_tab(ui, state),
                _ => {}
            }
        });
}

fn render_widgets_tab(ui: &mut egui::Ui, state: &mut SkinWidgetManagerState) {
    // Toolbar
    ui.horizontal(|ui| {
        if ui
            .add_enabled(!state.history.is_empty(), egui::Button::new("Undo"))
            .clicked()
        {
            state.history.undo();
        }

        // Column visibility dropdown
        ui.menu_button("Columns", |ui| {
            ui.checkbox(&mut state.column_visible[0], "x");
            ui.checkbox(&mut state.column_visible[1], "y");
            ui.checkbox(&mut state.column_visible[2], "w");
            ui.checkbox(&mut state.column_visible[3], "h");
        });

        ui.checkbox(&mut state.show_cursor_position, "Show Position");

        if ui.button("Export").clicked() {
            let text = state.export_changes();
            ui.ctx().copy_text(text);
        }
    });

    ui.separator();

    // Calculate column count: ID + visible columns + Operation
    let visible_count = state.column_visible.iter().filter(|&&v| v).count();
    let num_columns = 2 + visible_count; // ID + visible fields + Operation

    // Widget table
    egui::Grid::new("skin_widgets_grid")
        .num_columns(num_columns)
        .striped(true)
        .show(ui, |ui| {
            // Header
            ui.strong("ID");
            if state.column_visible[0] {
                ui.strong("x");
            }
            if state.column_visible[1] {
                ui.strong("y");
            }
            if state.column_visible[2] {
                ui.strong("w");
            }
            if state.column_visible[3] {
                ui.strong("h");
            }
            ui.strong("Operation");
            ui.end_row();

            // Track actions to apply after iteration
            let mut toggle_idx = None;
            let mut edit_request = None;

            for (widget_idx, widget) in state.widgets.iter().enumerate() {
                let id = ui.make_persistent_id(format!("sw_{}", widget_idx));
                let header_response =
                    egui::collapsing_header::CollapsingState::load_with_default_open(
                        ui.ctx(),
                        id,
                        false,
                    );

                header_response
                    .show_header(ui, |ui| {
                        ui.label(&widget.name);
                    })
                    .body(|ui| {
                        for (dst_idx, dst) in widget.destinations.iter().enumerate() {
                            egui::Grid::new(format!("sw_dst_{}_{}", widget_idx, dst_idx))
                                .num_columns(num_columns)
                                .show(ui, |ui| {
                                    ui.label(&dst.name);
                                    if state.column_visible[0] {
                                        let changed =
                                            state.history.has_event(&widget.name, ChangeField::X);
                                        let text = format_float(dst.x);
                                        if changed {
                                            ui.colored_label(
                                                egui::Color32::from_rgb(255, 100, 100),
                                                &text,
                                            );
                                        } else {
                                            ui.label(&text);
                                        }
                                    }
                                    if state.column_visible[1] {
                                        let changed =
                                            state.history.has_event(&widget.name, ChangeField::Y);
                                        let text = format_float(dst.y);
                                        if changed {
                                            ui.colored_label(
                                                egui::Color32::from_rgb(255, 100, 100),
                                                &text,
                                            );
                                        } else {
                                            ui.label(&text);
                                        }
                                    }
                                    if state.column_visible[2] {
                                        let changed =
                                            state.history.has_event(&widget.name, ChangeField::W);
                                        let text = format_float(dst.w);
                                        if changed {
                                            ui.colored_label(
                                                egui::Color32::from_rgb(255, 100, 100),
                                                &text,
                                            );
                                        } else {
                                            ui.label(&text);
                                        }
                                    }
                                    if state.column_visible[3] {
                                        let changed =
                                            state.history.has_event(&widget.name, ChangeField::H);
                                        let text = format_float(dst.h);
                                        if changed {
                                            ui.colored_label(
                                                egui::Color32::from_rgb(255, 100, 100),
                                                &text,
                                            );
                                        } else {
                                            ui.label(&text);
                                        }
                                    }
                                    if ui.button("Edit").clicked() {
                                        edit_request = Some((widget_idx, dst_idx));
                                    }
                                });
                        }
                    });

                // Widget row: show "--" for field columns + Toggle button
                if state.column_visible[0] {
                    ui.label("--");
                }
                if state.column_visible[1] {
                    ui.label("--");
                }
                if state.column_visible[2] {
                    ui.label("--");
                }
                if state.column_visible[3] {
                    ui.label("--");
                }
                let toggle_label = if widget.visible { "Hide" } else { "Show" };
                if ui.button(toggle_label).clicked() {
                    toggle_idx = Some(widget_idx);
                }
                ui.end_row();
            }

            // Apply deferred actions
            if let Some(idx) = toggle_idx {
                state.toggle_visible(idx);
            }
            if let Some((w, d)) = edit_request {
                state.begin_edit(w, d);
            }
        });

    // Edit popup
    if state.editing_widget.is_some() {
        egui::Window::new("Edit Destination")
            .collapsible(false)
            .resizable(false)
            .show(ui.ctx(), |ui| {
                ui.horizontal(|ui| {
                    ui.label("x:");
                    ui.add(egui::DragValue::new(&mut state.edit_x).speed(1.0));
                });
                ui.horizontal(|ui| {
                    ui.label("y:");
                    ui.add(egui::DragValue::new(&mut state.edit_y).speed(1.0));
                });
                ui.horizontal(|ui| {
                    ui.label("w:");
                    ui.add(egui::DragValue::new(&mut state.edit_w).speed(1.0));
                });
                ui.horizontal(|ui| {
                    ui.label("h:");
                    ui.add(egui::DragValue::new(&mut state.edit_h).speed(1.0));
                });

                ui.separator();

                ui.horizontal(|ui| {
                    if ui.button("Submit").clicked() {
                        state.submit_edit();
                    }
                    if ui.button("Cancel").clicked() {
                        state.editing_widget = None;
                    }
                });
            });
    }
}

fn render_history_tab(ui: &mut egui::Ui, state: &SkinWidgetManagerState) {
    if state.history.is_empty() {
        ui.label("No changes recorded.");
        return;
    }

    egui::Grid::new("history_grid")
        .num_columns(1)
        .striped(true)
        .show(ui, |ui| {
            for event in state.history.events() {
                ui.label(event.description());
                ui.end_row();
            }
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_state() {
        let state = SkinWidgetManagerState::default();
        assert!(state.widgets.is_empty());
        assert!(state.history.is_empty());
        assert_eq!(state.column_visible, [true; 4]);
        assert!(!state.show_cursor_position);
        assert_eq!(state.active_tab, 0);
        assert!(state.editing_widget.is_none());
    }

    #[test]
    fn event_history_push_and_undo() {
        let mut history = EventHistory::default();

        history.push(WidgetEvent::ChangeField {
            field: ChangeField::X,
            widget_name: "widget1".into(),
            dst_name: "dst0".into(),
            previous: 0.0,
            current: 10.0,
        });

        assert!(!history.is_empty());
        assert!(history.has_event("widget1", ChangeField::X));
        assert!(!history.has_event("widget1", ChangeField::Y));
        assert!(!history.has_event("widget2", ChangeField::X));

        let undone = history.undo();
        assert!(undone.is_some());
        assert!(history.is_empty());
        assert!(!history.has_event("widget1", ChangeField::X));
    }

    #[test]
    fn event_history_clear() {
        let mut history = EventHistory::default();

        history.push(WidgetEvent::ChangeField {
            field: ChangeField::X,
            widget_name: "w1".into(),
            dst_name: "d0".into(),
            previous: 0.0,
            current: 5.0,
        });
        history.push(WidgetEvent::ToggleVisible {
            widget_name: "w2".into(),
            was_visible: true,
        });

        assert_eq!(history.events().len(), 2);
        history.clear();
        assert!(history.is_empty());
        assert!(history.events_for_name("w1").is_empty());
    }

    #[test]
    fn event_description() {
        let change = WidgetEvent::ChangeField {
            field: ChangeField::W,
            widget_name: "button".into(),
            dst_name: "dst0".into(),
            previous: 100.0,
            current: 150.5,
        };
        assert_eq!(change.description(), "button.dst0.w: 100 -> 150.5");

        let toggle_hide = WidgetEvent::ToggleVisible {
            widget_name: "label".into(),
            was_visible: true,
        };
        assert_eq!(toggle_hide.description(), "label: hidden");

        let toggle_show = WidgetEvent::ToggleVisible {
            widget_name: "label".into(),
            was_visible: false,
        };
        assert_eq!(toggle_show.description(), "label: shown");
    }

    #[test]
    fn column_visibility() {
        let mut state = SkinWidgetManagerState::default();
        assert!(state.column_visible.iter().all(|&v| v));

        state.column_visible[1] = false; // hide y
        state.column_visible[3] = false; // hide h
        assert!(state.column_visible[0]);
        assert!(!state.column_visible[1]);
        assert!(state.column_visible[2]);
        assert!(!state.column_visible[3]);
    }

    #[test]
    fn format_float_display() {
        assert_eq!(format_float(100.0), "100");
        assert_eq!(format_float(0.0), "0");
        assert_eq!(format_float(150.5), "150.5");
        assert_eq!(format_float(-10.0), "-10");
        assert_eq!(format_float(3.14), "3.14");
    }

    #[test]
    fn submit_edit_records_events() {
        let mut state = SkinWidgetManagerState::default();
        state.widgets.push(SkinWidget {
            name: "btn".into(),
            visible: true,
            drawing: true,
            destinations: vec![WidgetDestination {
                name: "dst0".into(),
                x: 10.0,
                y: 20.0,
                w: 100.0,
                h: 50.0,
            }],
        });

        state.begin_edit(0, 0);
        assert!(state.editing_widget.is_some());
        assert!((state.edit_x - 10.0).abs() < f32::EPSILON);

        // Change x and w
        state.edit_x = 15.0;
        state.edit_w = 120.0;
        state.submit_edit();

        assert!(state.editing_widget.is_none());
        assert_eq!(state.history.events().len(), 2);
        assert!(state.history.has_event("btn", ChangeField::X));
        assert!(!state.history.has_event("btn", ChangeField::Y));
        assert!(state.history.has_event("btn", ChangeField::W));

        // Verify destination was updated
        assert!((state.widgets[0].destinations[0].x - 15.0).abs() < f32::EPSILON);
        assert!((state.widgets[0].destinations[0].w - 120.0).abs() < f32::EPSILON);
    }

    #[test]
    fn toggle_visible_records_event() {
        let mut state = SkinWidgetManagerState::default();
        state.widgets.push(SkinWidget {
            name: "panel".into(),
            visible: true,
            drawing: false,
            destinations: Vec::new(),
        });

        state.toggle_visible(0);
        assert!(!state.widgets[0].visible);
        assert_eq!(state.history.events().len(), 1);

        state.toggle_visible(0);
        assert!(state.widgets[0].visible);
        assert_eq!(state.history.events().len(), 2);
    }

    #[test]
    fn export_changes() {
        let mut state = SkinWidgetManagerState::default();
        state.history.push(WidgetEvent::ChangeField {
            field: ChangeField::X,
            widget_name: "btn".into(),
            dst_name: "dst0".into(),
            previous: 0.0,
            current: 10.0,
        });
        state.history.push(WidgetEvent::ToggleVisible {
            widget_name: "label".into(),
            was_visible: true,
        });

        let exported = state.export_changes();
        assert_eq!(exported, "btn.dst0.x: 0 -> 10\nlabel: hidden");
    }

    #[test]
    fn events_for_name() {
        let mut history = EventHistory::default();
        history.push(WidgetEvent::ChangeField {
            field: ChangeField::X,
            widget_name: "w1".into(),
            dst_name: "d0".into(),
            previous: 0.0,
            current: 5.0,
        });
        history.push(WidgetEvent::ChangeField {
            field: ChangeField::Y,
            widget_name: "w2".into(),
            dst_name: "d0".into(),
            previous: 0.0,
            current: 10.0,
        });
        history.push(WidgetEvent::ChangeField {
            field: ChangeField::W,
            widget_name: "w1".into(),
            dst_name: "d1".into(),
            previous: 50.0,
            current: 60.0,
        });

        let w1_events = history.events_for_name("w1");
        assert_eq!(w1_events.len(), 2);

        let w2_events = history.events_for_name("w2");
        assert_eq!(w2_events.len(), 1);

        let w3_events = history.events_for_name("w3");
        assert!(w3_events.is_empty());
    }
}
