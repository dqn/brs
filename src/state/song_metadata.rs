/// Metadata about the currently selected song, shared across scenes.
#[derive(Debug, Clone, Default)]
pub struct SongMetadata {
    pub title: String,
    pub subtitle: String,
    pub artist: String,
    pub subartist: String,
    pub genre: String,
    pub level: i32,
    pub max_bpm: i32,
    pub min_bpm: i32,
}
