/// Song trophy enum
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SongTrophy {
    Easy,
    Groove,
    Hard,
    ExHard,
    Normal,
    Mirror,
    Random,
    RRandom,
    SRandom,
    HRandom,
    Spiral,
    AllScr,
    ExRandom,
    ExSRandom,
    Battle,
    BattleAssist,
}

impl SongTrophy {
    pub fn character(&self) -> char {
        match self {
            SongTrophy::Easy => 'g',
            SongTrophy::Groove => 'G',
            SongTrophy::Hard => 'h',
            SongTrophy::ExHard => 'H',
            SongTrophy::Normal => 'n',
            SongTrophy::Mirror => 'm',
            SongTrophy::Random => 'r',
            SongTrophy::RRandom => 'o',
            SongTrophy::SRandom => 's',
            SongTrophy::HRandom => 'p',
            SongTrophy::Spiral => 'P',
            SongTrophy::AllScr => 'a',
            SongTrophy::ExRandom => 'R',
            SongTrophy::ExSRandom => 'S',
            SongTrophy::Battle => 'B',
            SongTrophy::BattleAssist => 'b',
        }
    }

    pub fn values() -> &'static [SongTrophy] {
        &[
            SongTrophy::Easy,
            SongTrophy::Groove,
            SongTrophy::Hard,
            SongTrophy::ExHard,
            SongTrophy::Normal,
            SongTrophy::Mirror,
            SongTrophy::Random,
            SongTrophy::RRandom,
            SongTrophy::SRandom,
            SongTrophy::HRandom,
            SongTrophy::Spiral,
            SongTrophy::AllScr,
            SongTrophy::ExRandom,
            SongTrophy::ExSRandom,
            SongTrophy::Battle,
            SongTrophy::BattleAssist,
        ]
    }

    pub fn trophy(c: char) -> Option<SongTrophy> {
        for trophy in SongTrophy::values() {
            if trophy.character() == c {
                return Some(*trophy);
            }
        }
        None
    }
}
