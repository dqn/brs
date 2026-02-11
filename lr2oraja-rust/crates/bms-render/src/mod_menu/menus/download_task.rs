// DownloadTask menu — displays running and expired download tasks.
//
// Corresponds to Java `DownloadTaskMenu.java` + `DownloadTaskState.java`.

const MAXIMUM_TASK_NAME_LENGTH: usize = 10;

#[derive(Debug, Clone)]
pub enum DownloadTaskStatus {
    Queued,
    Downloading,
    Extracting,
    Extracted,
    Error,
}

impl DownloadTaskStatus {
    pub fn name(&self) -> &str {
        match self {
            Self::Queued => "Queued",
            Self::Downloading => "Downloading",
            Self::Extracting => "Extracting",
            Self::Extracted => "Extracted",
            Self::Error => "Error",
        }
    }
}

#[derive(Debug, Clone)]
pub struct DownloadTask {
    pub id: u32,
    pub name: String,
    pub status: DownloadTaskStatus,
    pub download_size: u64,
    pub content_length: u64,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct DownloadTaskState {
    pub running: Vec<DownloadTask>,
    pub expired: Vec<DownloadTask>,
}

pub fn render(ctx: &egui::Context, open: &mut bool, state: &mut DownloadTaskState) {
    egui::Window::new("Download Tasks")
        .open(open)
        .resizable(false)
        .show(ctx, |ui| {
            if state.running.is_empty() && state.expired.is_empty() {
                ui.label("No Download Task. Try selecting missing bms to submit new task!");
                return;
            }

            // Tab-like header with running/expired sections
            if !state.running.is_empty() {
                ui.label("Running");
                render_task_table(ui, "running_tasks", &state.running);
            }

            if !state.expired.is_empty() {
                ui.separator();
                ui.label("Expired");
                render_task_table(ui, "expired_tasks", &state.expired);
            }
        });
}

fn render_task_table(ui: &mut egui::Ui, id: &str, tasks: &[DownloadTask]) {
    egui::Grid::new(id)
        .num_columns(3)
        .striped(true)
        .show(ui, |ui| {
            ui.strong("Task");
            ui.strong("Progress");
            ui.strong("Op");
            ui.end_row();

            for task in tasks {
                // Task name (truncated)
                let name: String = task.name.chars().take(MAXIMUM_TASK_NAME_LENGTH).collect();
                ui.label(format!("{} ({})", name, task.status.name()));

                // Progress or error
                if let Some(ref err) = task.error_message {
                    ui.colored_label(egui::Color32::RED, err);
                } else {
                    ui.label(format!(
                        "{}/{}",
                        humanize_file_size(task.download_size),
                        humanize_file_size(task.content_length),
                    ));
                }

                // Retry button for errored tasks
                if matches!(task.status, DownloadTaskStatus::Error) {
                    // TODO: wire up retry callback when download system is integrated
                    let _ = ui.button("Retry");
                } else {
                    ui.label("");
                }

                ui.end_row();
            }
        });
}

/// Convert bytes to human-readable file size string.
pub fn humanize_file_size(bytes: u64) -> String {
    let thresh: u64 = 1000;
    if bytes < thresh {
        return format!("{bytes} B");
    }

    let units = ["KB", "MB", "GB", "TB"];
    let mut result = bytes as f64;
    let mut u = 0;

    loop {
        result /= thresh as f64;
        if (result.abs() * 100.0).round() / 100.0 < thresh as f64 || u >= units.len() - 1 {
            break;
        }
        u += 1;
    }

    format!("{result:.1} {}", units[u])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn humanize_bytes() {
        assert_eq!(humanize_file_size(0), "0 B");
        assert_eq!(humanize_file_size(999), "999 B");
        assert_eq!(humanize_file_size(1000), "1.0 KB");
        assert_eq!(humanize_file_size(1_500_000), "1.5 MB");
        assert_eq!(humanize_file_size(1_000_000_000), "1.0 GB");
    }

    #[test]
    fn default_state() {
        let state = DownloadTaskState::default();
        assert!(state.running.is_empty());
        assert!(state.expired.is_empty());
    }
}
