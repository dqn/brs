// Toast notification system — fade-in/wait/fade-out overlay notifications.
//
// Corresponds to Java `ImGuiNotify.java`.
// Uses Unicode characters instead of FontAwesome icons.

use std::time::Instant;

const NOTIFY_PADDING_X: f32 = 20.0;
const NOTIFY_PADDING_Y: f32 = 20.0;
const NOTIFY_PADDING_MESSAGE_Y: f32 = 10.0;
const NOTIFY_FADE_IN_OUT_MS: u64 = 150;
const NOTIFY_DEFAULT_DISMISS_MS: u64 = 3000;
const NOTIFY_OPACITY: f32 = 0.9;
const NOTIFY_RENDER_LIMIT: usize = 7;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToastType {
    None,
    Success,
    Warning,
    Error,
    Info,
}

impl ToastType {
    fn color(self) -> egui::Color32 {
        match self {
            Self::None => egui::Color32::WHITE,
            Self::Success => egui::Color32::from_rgb(0, 255, 0),
            Self::Warning => egui::Color32::from_rgb(255, 255, 0),
            Self::Error => egui::Color32::from_rgb(255, 0, 0),
            Self::Info => egui::Color32::from_rgb(0, 157, 255),
        }
    }

    fn icon(self) -> &'static str {
        match self {
            Self::None => "",
            Self::Success => "\u{2705}", // check mark
            Self::Warning => "\u{26A0}", // warning sign
            Self::Error => "\u{274C}",   // cross mark
            Self::Info => "\u{2139}",    // info
        }
    }

    fn default_title(self) -> &'static str {
        match self {
            Self::None => "",
            Self::Success => "Success",
            Self::Warning => "Warning",
            Self::Error => "Error",
            Self::Info => "Info",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ToastPhase {
    FadeIn,
    Wait,
    FadeOut,
    Expired,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ToastPos {
    #[default]
    TopLeft,
    TopCenter,
    TopRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
    Center,
}

impl ToastPos {
    fn pivot(self) -> egui::Vec2 {
        match self {
            Self::TopLeft => egui::vec2(0.0, 0.0),
            Self::TopCenter => egui::vec2(0.5, 0.0),
            Self::TopRight => egui::vec2(1.0, 0.0),
            Self::BottomLeft => egui::vec2(0.0, 1.0),
            Self::BottomCenter => egui::vec2(0.5, 1.0),
            Self::BottomRight => egui::vec2(1.0, 1.0),
            Self::Center => egui::vec2(0.5, 0.5),
        }
    }

    pub fn from_index(index: usize) -> Self {
        match index {
            0 => Self::TopLeft,
            1 => Self::TopCenter,
            2 => Self::TopRight,
            3 => Self::BottomLeft,
            4 => Self::BottomCenter,
            5 => Self::BottomRight,
            _ => Self::Center,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Toast {
    toast_type: ToastType,
    title: String,
    content: String,
    dismiss_ms: u64,
    creation_time: Instant,
    pos: ToastPos,
}

impl Toast {
    pub fn new(toast_type: ToastType, content: impl Into<String>) -> Self {
        Self {
            toast_type,
            title: String::new(),
            content: content.into(),
            dismiss_ms: NOTIFY_DEFAULT_DISMISS_MS,
            creation_time: Instant::now(),
            pos: ToastPos::TopLeft,
        }
    }

    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    pub fn with_dismiss_ms(mut self, ms: u64) -> Self {
        self.dismiss_ms = ms;
        self
    }

    pub fn with_pos(mut self, pos: ToastPos) -> Self {
        self.pos = pos;
        self
    }

    fn elapsed_ms(&self) -> u64 {
        self.creation_time.elapsed().as_millis() as u64
    }

    fn phase(&self) -> ToastPhase {
        let elapsed = self.elapsed_ms();
        let total = NOTIFY_FADE_IN_OUT_MS + self.dismiss_ms + NOTIFY_FADE_IN_OUT_MS;

        if elapsed > total {
            ToastPhase::Expired
        } else if elapsed > NOTIFY_FADE_IN_OUT_MS + self.dismiss_ms {
            ToastPhase::FadeOut
        } else if elapsed > NOTIFY_FADE_IN_OUT_MS {
            ToastPhase::Wait
        } else {
            ToastPhase::FadeIn
        }
    }

    fn fade_percent(&self) -> f32 {
        let elapsed = self.elapsed_ms();
        match self.phase() {
            ToastPhase::FadeIn => (elapsed as f32 / NOTIFY_FADE_IN_OUT_MS as f32) * NOTIFY_OPACITY,
            ToastPhase::FadeOut => {
                let fade_start = NOTIFY_FADE_IN_OUT_MS + self.dismiss_ms;
                let progress = (elapsed - fade_start) as f32 / NOTIFY_FADE_IN_OUT_MS as f32;
                (1.0 - progress) * NOTIFY_OPACITY
            }
            _ => NOTIFY_OPACITY,
        }
    }

    fn display_title(&self) -> &str {
        if self.title.is_empty() {
            self.toast_type.default_title()
        } else {
            &self.title
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct NotificationState {
    notifications: Vec<Toast>,
    default_pos: ToastPos,
}

impl NotificationState {
    pub fn set_default_pos(&mut self, pos: ToastPos) {
        self.default_pos = pos;
    }

    pub fn success(&mut self, content: impl Into<String>) {
        self.insert(Toast::new(ToastType::Success, content));
    }

    pub fn warning(&mut self, content: impl Into<String>) {
        self.insert(Toast::new(ToastType::Warning, content));
    }

    pub fn error(&mut self, content: impl Into<String>) {
        self.insert(Toast::new(ToastType::Error, content));
    }

    pub fn info(&mut self, content: impl Into<String>) {
        self.insert(Toast::new(ToastType::Info, content));
    }

    pub fn insert(&mut self, mut toast: Toast) {
        toast.pos = self.default_pos;
        self.notifications.push(toast);
    }
}

pub fn render_notifications(ctx: &egui::Context, state: &mut NotificationState) {
    // Remove expired notifications
    state
        .notifications
        .retain(|t| t.phase() != ToastPhase::Expired);

    let screen = ctx.screen_rect();
    let mut height_offset = 0.0_f32;

    for (i, toast) in state
        .notifications
        .iter()
        .take(NOTIFY_RENDER_LIMIT)
        .enumerate()
    {
        let opacity = toast.fade_percent();
        let pivot = toast.pos.pivot();

        // Calculate position
        let init_pos = initial_position(toast.pos, screen);
        let y_dir = if matches!(
            toast.pos,
            ToastPos::BottomLeft | ToastPos::BottomCenter | ToastPos::BottomRight
        ) {
            -1.0
        } else {
            1.0
        };
        let pos = egui::pos2(init_pos.x, init_pos.y + height_offset * y_dir);

        let window_id = format!("##TOAST{i}");

        let frame = egui::Frame::window(&ctx.style())
            .fill(ctx.style().visuals.window_fill().gamma_multiply(opacity));

        let response = egui::Window::new(&window_id)
            .fixed_pos(pos)
            .pivot(egui::Align2([
                if pivot.x < 0.3 {
                    egui::Align::LEFT
                } else if pivot.x > 0.7 {
                    egui::Align::RIGHT
                } else {
                    egui::Align::Center
                },
                if pivot.y < 0.3 {
                    egui::Align::TOP
                } else if pivot.y > 0.7 {
                    egui::Align::BOTTOM
                } else {
                    egui::Align::Center
                },
            ]))
            .collapsible(false)
            .title_bar(false)
            .resizable(false)
            .frame(frame)
            .show(ctx, |ui| {
                ui.set_max_width(screen.width() / 3.0);

                // Icon + title
                let icon = toast.toast_type.icon();
                let title = toast.display_title();
                let color = toast.toast_type.color();
                let alpha_color = egui::Color32::from_rgba_unmultiplied(
                    color.r(),
                    color.g(),
                    color.b(),
                    (opacity * 255.0) as u8,
                );

                ui.horizontal(|ui| {
                    if !icon.is_empty() {
                        ui.colored_label(alpha_color, icon);
                    }
                    if !title.is_empty() {
                        ui.label(title);
                    }
                });

                // Content
                if !toast.content.is_empty() {
                    ui.add_space(5.0);
                    ui.label(&toast.content);
                }
            });

        if let Some(response) = response {
            height_offset += response.response.rect.height() + NOTIFY_PADDING_MESSAGE_Y;
        }
    }
}

fn initial_position(pos: ToastPos, screen: egui::Rect) -> egui::Pos2 {
    let w = screen.width();
    let h = screen.height();
    match pos {
        ToastPos::TopLeft => egui::pos2(NOTIFY_PADDING_X, NOTIFY_PADDING_Y),
        ToastPos::TopCenter => egui::pos2(w * 0.5, NOTIFY_PADDING_Y),
        ToastPos::TopRight => egui::pos2(w - NOTIFY_PADDING_X, NOTIFY_PADDING_Y),
        ToastPos::BottomLeft => egui::pos2(NOTIFY_PADDING_X, h - NOTIFY_PADDING_Y),
        ToastPos::BottomCenter => egui::pos2(w * 0.5, h - NOTIFY_PADDING_Y),
        ToastPos::BottomRight => egui::pos2(w - NOTIFY_PADDING_X, h - NOTIFY_PADDING_Y),
        ToastPos::Center => egui::pos2(w * 0.5, h * 0.5),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toast_type_colors() {
        assert_eq!(
            ToastType::Success.color(),
            egui::Color32::from_rgb(0, 255, 0)
        );
        assert_eq!(ToastType::Error.color(), egui::Color32::from_rgb(255, 0, 0));
    }

    #[test]
    fn toast_default_titles() {
        assert_eq!(ToastType::Success.default_title(), "Success");
        assert_eq!(ToastType::Warning.default_title(), "Warning");
        assert_eq!(ToastType::Error.default_title(), "Error");
        assert_eq!(ToastType::Info.default_title(), "Info");
        assert_eq!(ToastType::None.default_title(), "");
    }

    #[test]
    fn toast_custom_title() {
        let toast = Toast::new(ToastType::Info, "content").with_title("Custom");
        assert_eq!(toast.display_title(), "Custom");
    }

    #[test]
    fn toast_default_title_fallback() {
        let toast = Toast::new(ToastType::Info, "content");
        assert_eq!(toast.display_title(), "Info");
    }

    #[test]
    fn toast_pos_from_index() {
        assert_eq!(ToastPos::from_index(0), ToastPos::TopLeft);
        assert_eq!(ToastPos::from_index(3), ToastPos::BottomLeft);
        assert_eq!(ToastPos::from_index(6), ToastPos::Center);
        assert_eq!(ToastPos::from_index(99), ToastPos::Center);
    }

    #[test]
    fn notification_state_insert() {
        let mut state = NotificationState::default();
        state.success("Test success");
        state.error("Test error");
        assert_eq!(state.notifications.len(), 2);
    }
}
