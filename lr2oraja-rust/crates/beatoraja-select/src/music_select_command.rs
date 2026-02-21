/// Music select commands
/// Translates: bms.player.beatoraja.select.MusicSelectCommand
///
/// In Java, each enum variant holds a Consumer<MusicSelector>.
/// In Rust, we use an enum and dispatch via a method on MusicSelector
/// (since the commands need MusicSelector context to execute).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MusicSelectCommand {
    ResetReplay,
    NextReplay,
    PrevReplay,
    CopyMd5Hash,
    CopySha256Hash,
    DownloadIpfs,
    DownloadHttp,
    DownloadCourseHttp,
    ShowSongsOnSameFolder,
    ShowContextMenu,
    CopyHighlightedMenuText,
}
