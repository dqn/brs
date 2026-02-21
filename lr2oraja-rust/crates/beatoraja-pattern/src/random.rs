use bms_model::mode::Mode;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RandomUnit {
    None,
    Lane,
    Note,
    Player,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Random {
    Identity,
    Mirror,
    Random,
    Rotate,
    SRandom,
    Spiral,
    HRandom,
    AllScr,
    MirrorEx,
    RandomEx,
    RotateEx,
    SRandomEx,
    Cross,
    Converge,
    SRandomNoThreshold,
    RandomPlayable,
    SRandomPlayable,
    Flip,
    Battle,
}

impl Random {
    pub fn unit(&self) -> RandomUnit {
        match self {
            Random::Identity => RandomUnit::None,
            Random::Mirror => RandomUnit::Lane,
            Random::Random => RandomUnit::Lane,
            Random::Rotate => RandomUnit::Lane,
            Random::SRandom => RandomUnit::Note,
            Random::Spiral => RandomUnit::Note,
            Random::HRandom => RandomUnit::Note,
            Random::AllScr => RandomUnit::Note,
            Random::MirrorEx => RandomUnit::Lane,
            Random::RandomEx => RandomUnit::Lane,
            Random::RotateEx => RandomUnit::Lane,
            Random::SRandomEx => RandomUnit::Note,
            Random::Cross => RandomUnit::Lane,
            Random::Converge => RandomUnit::Note,
            Random::SRandomNoThreshold => RandomUnit::Note,
            Random::RandomPlayable => RandomUnit::Lane,
            Random::SRandomPlayable => RandomUnit::Note,
            Random::Flip => RandomUnit::Player,
            Random::Battle => RandomUnit::Player,
        }
    }

    pub fn is_scratch_lane_modify(&self) -> bool {
        match self {
            Random::Identity => false,
            Random::Mirror => false,
            Random::Random => false,
            Random::Rotate => false,
            Random::SRandom => false,
            Random::Spiral => false,
            Random::HRandom => false,
            Random::AllScr => true,
            Random::MirrorEx => true,
            Random::RandomEx => true,
            Random::RotateEx => true,
            Random::SRandomEx => true,
            Random::Cross => false,
            Random::Converge => true,
            Random::SRandomNoThreshold => false,
            Random::RandomPlayable => true,
            Random::SRandomPlayable => true,
            Random::Flip => true,
            Random::Battle => true,
        }
    }

    pub fn option_general() -> &'static [Random] {
        &[
            Random::Identity,
            Random::Mirror,
            Random::Random,
            Random::Rotate,
            Random::SRandom,
            Random::Spiral,
            Random::HRandom,
            Random::AllScr,
            Random::RandomEx,
            Random::SRandomEx,
        ]
    }

    pub fn option_pms() -> &'static [Random] {
        &[
            Random::Identity,
            Random::Mirror,
            Random::Random,
            Random::Rotate,
            Random::SRandomNoThreshold,
            Random::Spiral,
            Random::HRandom,
            Random::Converge,
            Random::RandomPlayable,
            Random::SRandomPlayable,
        ]
    }

    pub fn option_double() -> &'static [Random] {
        &[Random::Identity, Random::Flip]
    }

    pub fn option_single() -> &'static [Random] {
        &[Random::Identity, Random::Battle]
    }

    pub fn get_random(id: i32, mode: &Mode) -> Random {
        let randoms = match mode {
            Mode::POPN_5K | Mode::POPN_9K => Random::option_pms(),
            _ => Random::option_general(),
        };
        if id >= 0 && (id as usize) < randoms.len() {
            randoms[id as usize]
        } else {
            Random::Identity
        }
    }
}
