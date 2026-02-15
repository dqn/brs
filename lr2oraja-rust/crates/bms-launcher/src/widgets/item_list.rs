/// Generic selectable list with add, remove, and reorder buttons.
///
/// Returns `true` if the list was modified.
pub fn show_item_list<T>(
    ui: &mut egui::Ui,
    id: &str,
    items: &mut Vec<T>,
    selected: &mut Option<usize>,
    display_name: impl Fn(&T) -> String,
    create_new: impl FnOnce() -> T,
    add_label: &str,
) -> bool {
    let mut changed = false;

    ui.group(|ui| {
        let mut remove_idx = None;
        let mut swap = None;

        for (i, item) in items.iter().enumerate() {
            ui.horizontal(|ui| {
                let is_selected = *selected == Some(i);
                if ui
                    .selectable_label(is_selected, display_name(item))
                    .clicked()
                {
                    *selected = Some(i);
                }
                if ui.small_button("\u{2191}").clicked() && i > 0 {
                    swap = Some((i - 1, i));
                }
                if ui.small_button("\u{2193}").clicked() && i + 1 < items.len() {
                    swap = Some((i, i + 1));
                }
                if ui.small_button("\u{2717}").clicked() {
                    remove_idx = Some(i);
                }
            });
        }

        if let Some(idx) = remove_idx {
            items.remove(idx);
            changed = true;
            // Adjust selection
            if items.is_empty() {
                *selected = None;
            } else if let Some(sel) = selected
                && *sel >= items.len()
            {
                *sel = items.len() - 1;
            }
        }
        if let Some((a, b)) = swap {
            items.swap(a, b);
            changed = true;
            // Follow the swapped item
            if *selected == Some(a) {
                *selected = Some(b);
            } else if *selected == Some(b) {
                *selected = Some(a);
            }
        }

        if ui.button(format!("{} {}", "\u{2795}", add_label)).clicked() {
            items.push(create_new());
            *selected = Some(items.len() - 1);
            changed = true;
        }
    });

    egui::ScrollArea::vertical()
        .id_salt(id)
        .max_height(300.0)
        .show(ui, |_ui| {});

    changed
}
