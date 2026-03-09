use super::*;

#[test]
fn test_launcher_ui_new_defaults() {
    let config = Config::default();
    let player = PlayerConfig::default();
    let ui = LauncherUi::new(config, player);

    assert!(!ui.is_play_requested());
    assert!(!ui.exit_requested);
    assert_eq!(ui.selected_tab, Tab::Option);
    assert_eq!(ui.selected_play_mode, 1); // BEAT_7K
}

#[test]
fn test_launcher_ui_config_accessors() {
    let mut config = Config::default();
    config.display.vsync = true;
    config.display.max_frame_per_second = 120;
    let player = PlayerConfig::default();
    let ui = LauncherUi::new(config, player);

    assert!(ui.config().display.vsync);
    assert_eq!(ui.config().display.max_frame_per_second, 120);
}

#[test]
fn test_launcher_ui_player_accessor() {
    let config = Config::default();
    let mut player = PlayerConfig::default();
    player.name = "test_player".to_string();
    let ui = LauncherUi::new(config, player);

    assert_eq!(ui.player().name, "test_player");
}

#[test]
fn test_play_requested_initially_false() {
    let ui = LauncherUi::new(Config::default(), PlayerConfig::default());
    assert!(!ui.is_play_requested());
}

#[test]
fn test_tab_all_returns_11_tabs() {
    // Java: PlayConfigurationView has 11 tabs
    assert_eq!(Tab::all().len(), 11);
}

#[test]
fn test_tab_labels_non_empty() {
    for tab in Tab::all() {
        assert!(!tab.label().is_empty());
    }
}
