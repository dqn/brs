// JudgeTrainer menu — overrides chart judge difficulty.
//
// Corresponds to Java `JudgeTrainer.java` + `JudgeTrainerMenu.java`.
// Judge rank indices: EASY(0) NORMAL(1) HARD(2) VERY_HARD(3).
// Maps to windowrule ranks via transformation: index -> 3 - index.

pub const JUDGE_OPTIONS: &[&str] = &["EASY", "NORMAL", "HARD", "VERY_HARD"];

#[derive(Debug, Clone, Default)]
pub struct JudgeTrainerState {
    pub active: bool,
    pub judge_rank: usize,
}

impl JudgeTrainerState {
    /// Returns the transformed judge rank for BMSPlayerRule lookup.
    ///
    /// The windowrule order is VERY_HARD(0) HARD(1) NORMAL(2) EASY(3),
    /// which is reversed from the UI order. Sum is always 3.
    pub fn window_rule_index(&self) -> usize {
        3 - self.judge_rank
    }
}

pub fn render(ctx: &egui::Context, open: &mut bool, state: &mut JudgeTrainerState) {
    egui::Window::new("Judge Trainer")
        .open(open)
        .resizable(false)
        .show(ctx, |ui| {
            ui.checkbox(&mut state.active, "Override chart's judge");
            egui::ComboBox::from_label("Judge")
                .selected_text(JUDGE_OPTIONS[state.judge_rank])
                .show_ui(ui, |ui| {
                    for (i, &option) in JUDGE_OPTIONS.iter().enumerate() {
                        ui.selectable_value(&mut state.judge_rank, i, option);
                    }
                });
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_state() {
        let state = JudgeTrainerState::default();
        assert!(!state.active);
        assert_eq!(state.judge_rank, 0);
    }

    #[test]
    fn window_rule_index_transformation() {
        // EASY(0) -> 3, NORMAL(1) -> 2, HARD(2) -> 1, VERY_HARD(3) -> 0
        let mut state = JudgeTrainerState::default();

        state.judge_rank = 0; // EASY
        assert_eq!(state.window_rule_index(), 3);

        state.judge_rank = 1; // NORMAL
        assert_eq!(state.window_rule_index(), 2);

        state.judge_rank = 2; // HARD
        assert_eq!(state.window_rule_index(), 1);

        state.judge_rank = 3; // VERY_HARD
        assert_eq!(state.window_rule_index(), 0);
    }
}
