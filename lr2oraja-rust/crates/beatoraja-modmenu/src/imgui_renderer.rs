use crate::download_task_menu::DownloadTaskMenu;
use crate::freq_trainer_menu::FreqTrainerMenu;
use crate::imgui_notify::ImGuiNotify;
use crate::judge_trainer_menu::JudgeTrainerMenu;
use crate::misc_setting_menu::MiscSettingMenu;
use crate::performance_monitor::PerformanceMonitor;
use crate::random_trainer_menu::RandomTrainerMenu;
use crate::skin_menu::SkinMenu;
use crate::skin_widget_manager::SkinWidgetManager;
use crate::stubs::{Version, version};

use std::sync::Mutex;

static WINDOW_WIDTH: Mutex<i32> = Mutex::new(0);
static WINDOW_HEIGHT: Mutex<i32> = Mutex::new(0);

static SHOW_MOD_MENU: Mutex<bool> = Mutex::new(false);
static SHOW_RANDOM_TRAINER: Mutex<bool> = Mutex::new(false);
static SHOW_FREQ_PLUS: Mutex<bool> = Mutex::new(false);
static SHOW_JUDGE_TRAINER: Mutex<bool> = Mutex::new(false);
static SHOW_SONG_MANAGER: Mutex<bool> = Mutex::new(false);
static SHOW_DOWNLOAD_MENU: Mutex<bool> = Mutex::new(false);
static SHOW_SKIN_WIDGET_MANAGER: Mutex<bool> = Mutex::new(false);
static SHOW_PERFORMANCE_MONITOR: Mutex<bool> = Mutex::new(false);
static SHOW_SKIN_MENU: Mutex<bool> = Mutex::new(false);
static SHOW_MISC_SETTING: Mutex<bool> = Mutex::new(false);

pub fn window_width() -> i32 {
    *WINDOW_WIDTH.lock().unwrap()
}

pub fn window_height() -> i32 {
    *WINDOW_HEIGHT.lock().unwrap()
}

pub struct ImGuiRenderer;

impl ImGuiRenderer {
    pub fn init(width: i32, height: i32) {
        *WINDOW_WIDTH.lock().unwrap() = width;
        *WINDOW_HEIGHT.lock().unwrap() = height;
        // egui context is initialized in beatoraja-bin; nothing to do here.
    }

    pub fn start() {
        // egui frame is managed by beatoraja-bin via egui_winit::State::take_egui_input()
    }

    /// Render mod menu overlay using egui.
    ///
    /// Java equivalent: ImGuiRenderer.render() — called between ImGui.newFrame() and ImGui.render().
    /// Called from beatoraja-bin's event loop within egui::Context::run().
    pub fn render_ui(ctx: &egui::Context) {
        let show_mod_menu = *SHOW_MOD_MENU.lock().unwrap();
        if show_mod_menu {
            let mut show = true;
            egui::Window::new("Endless Dream")
                .open(&mut show)
                .auto_sized()
                .show(ctx, |ui| {
                    // Sub-window toggle checkboxes
                    let mut freq = SHOW_FREQ_PLUS.lock().unwrap();
                    ui.checkbox(&mut freq, "Show Rate Modifier Window");
                    drop(freq);

                    let mut random = SHOW_RANDOM_TRAINER.lock().unwrap();
                    ui.checkbox(&mut random, "Show Random Trainer Window");
                    drop(random);

                    let mut judge = SHOW_JUDGE_TRAINER.lock().unwrap();
                    ui.checkbox(&mut judge, "Show Judge Trainer Window");
                    drop(judge);

                    {
                        let mut skin = SHOW_SKIN_MENU.lock().unwrap();
                        let old = *skin;
                        ui.checkbox(&mut skin, "Show Skin Configuration Window");
                        if *skin && !old {
                            SkinMenu::invalidate();
                        }
                    }

                    let mut swm = SHOW_SKIN_WIDGET_MANAGER.lock().unwrap();
                    ui.checkbox(&mut swm, "Show Skin Widget Manager Window");
                    drop(swm);

                    let mut song = SHOW_SONG_MANAGER.lock().unwrap();
                    ui.checkbox(&mut song, "Show Song Manager Window");
                    drop(song);

                    let mut dl = SHOW_DOWNLOAD_MENU.lock().unwrap();
                    ui.checkbox(&mut dl, "Show Download Tasks Window");
                    drop(dl);

                    {
                        let mut perf = SHOW_PERFORMANCE_MONITOR.lock().unwrap();
                        let old = *perf;
                        ui.checkbox(&mut perf, "Show Performance Monitor Window");
                        if *perf && !old {
                            PerformanceMonitor::reload_event_tree();
                        }
                    }

                    let mut misc = SHOW_MISC_SETTING.lock().unwrap();
                    ui.checkbox(&mut misc, "Show Misc Setting Window");
                    drop(misc);

                    // Debug information
                    ui.collapsing("Endless Dream Debug Information", |ui| {
                        let commit_hash = Version::get_git_commit_hash().unwrap_or("unknown");
                        let build_time = version::get_build_date().unwrap_or("unknown");
                        ui.label(format!("Commit hash: {}", commit_hash));
                        ui.label(format!("Build time: {}", build_time));
                    });
                });
            if !show {
                *SHOW_MOD_MENU.lock().unwrap() = false;
            }

            // Render sub-windows
            if *SHOW_FREQ_PLUS.lock().unwrap() {
                FreqTrainerMenu::show_ui(ctx);
            }
            if *SHOW_RANDOM_TRAINER.lock().unwrap() {
                RandomTrainerMenu::show_ui(ctx);
            }
            if *SHOW_JUDGE_TRAINER.lock().unwrap() {
                JudgeTrainerMenu::show_ui(ctx);
            }
            if *SHOW_SONG_MANAGER.lock().unwrap() {
                crate::song_manager_menu::SongManagerMenu::show_ui(ctx);
            }
            if *SHOW_DOWNLOAD_MENU.lock().unwrap() {
                DownloadTaskMenu::show_ui(ctx);
            }
            if *SHOW_SKIN_WIDGET_MANAGER.lock().unwrap() {
                SkinWidgetManager::set_focus(true);
                SkinWidgetManager::show_ui(ctx);
            } else {
                SkinWidgetManager::set_focus(false);
            }
            if *SHOW_PERFORMANCE_MONITOR.lock().unwrap() {
                PerformanceMonitor::show_ui(ctx);
            }
            if *SHOW_SKIN_MENU.lock().unwrap() {
                SkinMenu::show_ui(ctx);
            }
            if *SHOW_MISC_SETTING.lock().unwrap() {
                MiscSettingMenu::show_ui(ctx);
            }
        }

        // Render toast notifications overlay
        ImGuiNotify::render_notifications_ui(ctx);
    }

    /// Legacy render method — retained for backward compatibility with MainController stub calls.
    /// Actual rendering is now done via render_ui() called from beatoraja-bin.
    pub fn render() {}

    pub fn end() {
        // egui rendering is handled by beatoraja-bin via EguiIntegration::render()
    }

    pub fn dispose() {
        // egui context cleanup is handled by beatoraja-bin
    }

    pub fn get_show_mod_menu() -> bool {
        *SHOW_MOD_MENU.lock().unwrap()
    }

    pub fn toggle_menu() {
        let mut menu = SHOW_MOD_MENU.lock().unwrap();
        *menu = !*menu;
    }

    /// Show a "(?)" tooltip when hovering.
    ///
    /// Java: ImGui.textDisabled("(?)") + isItemHovered() → tooltip
    pub fn help_marker(ui: &mut egui::Ui, desc: &str) {
        ui.label(egui::RichText::new("(?)").weak())
            .on_hover_text(desc);
    }
}
