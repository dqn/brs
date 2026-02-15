use bms_database::CourseSongData;

/// Display a list of CourseSongData with remove and reorder buttons.
///
/// Returns `true` if the list was modified.
pub fn show_song_list(ui: &mut egui::Ui, id: &str, songs: &mut Vec<CourseSongData>) -> bool {
    let mut changed = false;
    let mut remove_idx = None;
    let mut swap = None;

    ui.group(|ui| {
        if songs.is_empty() {
            ui.label("(no songs)");
        }

        for (i, song) in songs.iter().enumerate() {
            ui.horizontal(|ui| {
                let hash_preview = if !song.sha256.is_empty() {
                    &song.sha256[..song.sha256.len().min(8)]
                } else if !song.md5.is_empty() {
                    &song.md5[..song.md5.len().min(8)]
                } else {
                    "?"
                };
                let label = if song.title.is_empty() {
                    format!("[{}]", hash_preview)
                } else {
                    format!("{} [{}]", song.title, hash_preview)
                };
                ui.label(label);

                if ui.small_button("\u{2191}").clicked() && i > 0 {
                    swap = Some((i - 1, i));
                }
                if ui.small_button("\u{2193}").clicked() && i + 1 < songs.len() {
                    swap = Some((i, i + 1));
                }
                if ui.small_button("\u{2717}").clicked() {
                    remove_idx = Some(i);
                }
            });
        }
    });

    egui::ScrollArea::vertical()
        .id_salt(id)
        .max_height(200.0)
        .show(ui, |_ui| {});

    if let Some(idx) = remove_idx {
        songs.remove(idx);
        changed = true;
    }
    if let Some((a, b)) = swap {
        songs.swap(a, b);
        changed = true;
    }

    changed
}
