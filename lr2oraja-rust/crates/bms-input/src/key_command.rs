/// UI key commands mapped from control key combinations.
///
/// Ported from Java `KeyCommand` enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyCommand {
    ShowFps,
    UpdateFolder,
    OpenExplorer,
    CopySongMd5Hash,
    CopySongSha256Hash,
    SwitchScreenMode,
    SaveScreenshot,
    PostTwitter,
    AddFavoriteSong,
    AddFavoriteChart,
    AutoplayFolder,
    OpenIr,
    OpenSkinConfiguration,
    ToggleModMenu,
    CopyHighlightedMenuText,
}
