#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Video,
    Audio,
    Input,
    Resource,
    MusicSelect,
    PlayOption,
    Skin,
    Ir,
    Discord,
    Obs,
    Stream,
}

impl Tab {
    pub const ALL: &[Tab] = &[
        Tab::Video,
        Tab::Audio,
        Tab::Input,
        Tab::Resource,
        Tab::MusicSelect,
        Tab::PlayOption,
        Tab::Skin,
        Tab::Ir,
        Tab::Discord,
        Tab::Obs,
        Tab::Stream,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Video => "Video",
            Self::Audio => "Audio",
            Self::Input => "Input",
            Self::Resource => "Resource",
            Self::MusicSelect => "Music Select",
            Self::PlayOption => "Play Option",
            Self::Skin => "Skin",
            Self::Ir => "IR",
            Self::Discord => "Discord",
            Self::Obs => "OBS",
            Self::Stream => "Stream",
        }
    }
}
