// FreqTrainer menu — modifies chart playback rate (50-200%).
//
// Corresponds to Java `FreqTrainerMenu.java`.
// Rate is stored as integer percentage (100 = normal speed).

const FREQ_MIN: i32 = 50;
const FREQ_MAX: i32 = 200;

const BUTTON_DELTAS: &[(i32, &str)] = &[
    (-10, "-10%"),
    (-5, "-5%"),
    (-1, "-1%"),
    (0, "Reset"),
    (1, "+1%"),
    (5, "+5%"),
    (10, "+10%"),
];

#[derive(Debug, Clone)]
pub struct FreqTrainerState {
    pub enabled: bool,
    pub freq: i32,
}

impl Default for FreqTrainerState {
    fn default() -> Self {
        Self {
            enabled: false,
            freq: 100,
        }
    }
}

impl FreqTrainerState {
    pub fn is_negative(&self) -> bool {
        self.freq < 100
    }

    pub fn freq_string(&self) -> String {
        format!("[{:.02}x]", self.freq as f32 / 100.0)
    }

    fn clamp(value: i32) -> i32 {
        value.clamp(FREQ_MIN, FREQ_MAX)
    }
}

pub fn render(ctx: &egui::Context, open: &mut bool, state: &mut FreqTrainerState) {
    egui::Window::new("Rate Modifier")
        .open(open)
        .resizable(false)
        .show(ctx, |ui| {
            ui.label("Modifies the chart playback rate to be faster or");
            ui.label("slower by a given percent.");

            ui.horizontal(|ui| {
                for &(delta, label) in BUTTON_DELTAS {
                    if ui.button(label).clicked() {
                        if delta == 0 {
                            state.freq = 100;
                        } else {
                            state.freq = FreqTrainerState::clamp(state.freq + delta);
                        }
                    }
                }
            });

            ui.add(egui::Slider::new(&mut state.freq, FREQ_MIN..=FREQ_MAX).text("%"));
            state.freq = FreqTrainerState::clamp(state.freq);

            ui.separator();
            ui.label("Controls");
            ui.indent("freq_controls", |ui| {
                ui.horizontal(|ui| {
                    ui.checkbox(&mut state.enabled, "Rate Enabled");
                    ui.label("(?)").on_hover_text(
                        "When enabled positive rate scores will save locally, \
                         however scores will not submit to IR and result lamp \
                         will always be NO PLAY.",
                    );
                });
            });
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_state() {
        let state = FreqTrainerState::default();
        assert!(!state.enabled);
        assert_eq!(state.freq, 100);
    }

    #[test]
    fn clamp_freq() {
        assert_eq!(FreqTrainerState::clamp(49), FREQ_MIN);
        assert_eq!(FreqTrainerState::clamp(201), FREQ_MAX);
        assert_eq!(FreqTrainerState::clamp(100), 100);
    }

    #[test]
    fn freq_string_format() {
        let state = FreqTrainerState {
            enabled: false,
            freq: 100,
        };
        assert_eq!(state.freq_string(), "[1.00x]");

        let state = FreqTrainerState {
            enabled: false,
            freq: 150,
        };
        assert_eq!(state.freq_string(), "[1.50x]");
    }

    #[test]
    fn is_negative() {
        assert!(
            FreqTrainerState {
                enabled: false,
                freq: 99
            }
            .is_negative()
        );
        assert!(
            !FreqTrainerState {
                enabled: false,
                freq: 100
            }
            .is_negative()
        );
        assert!(
            !FreqTrainerState {
                enabled: false,
                freq: 101
            }
            .is_negative()
        );
    }
}
