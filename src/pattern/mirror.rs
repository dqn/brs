use crate::model::BMSModel;
use crate::model::note::Lane;

/// Mirror modifier that flips key lanes horizontally.
pub struct MirrorModifier;

impl MirrorModifier {
    /// Apply mirror transformation to the model.
    /// Key lanes are reversed: Key1 <-> Key7, Key2 <-> Key6, etc.
    /// Scratch lane is not affected.
    pub fn modify(&self, model: &mut BMSModel) {
        for timeline in model.timelines.entries_mut() {
            for note in &mut timeline.notes {
                note.lane = Self::mirror_lane(note.lane);
            }
        }
    }

    fn mirror_lane(lane: Lane) -> Lane {
        match lane {
            // 1P side
            Lane::Scratch => Lane::Scratch,
            Lane::Key1 => Lane::Key7,
            Lane::Key2 => Lane::Key6,
            Lane::Key3 => Lane::Key5,
            Lane::Key4 => Lane::Key4,
            Lane::Key5 => Lane::Key3,
            Lane::Key6 => Lane::Key2,
            Lane::Key7 => Lane::Key1,
            // 2P side
            Lane::Scratch2 => Lane::Scratch2,
            Lane::Key8 => Lane::Key14,
            Lane::Key9 => Lane::Key13,
            Lane::Key10 => Lane::Key12,
            Lane::Key11 => Lane::Key11,
            Lane::Key12 => Lane::Key10,
            Lane::Key13 => Lane::Key9,
            Lane::Key14 => Lane::Key8,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mirror_lane_1p() {
        assert_eq!(MirrorModifier::mirror_lane(Lane::Scratch), Lane::Scratch);
        assert_eq!(MirrorModifier::mirror_lane(Lane::Key1), Lane::Key7);
        assert_eq!(MirrorModifier::mirror_lane(Lane::Key2), Lane::Key6);
        assert_eq!(MirrorModifier::mirror_lane(Lane::Key3), Lane::Key5);
        assert_eq!(MirrorModifier::mirror_lane(Lane::Key4), Lane::Key4);
        assert_eq!(MirrorModifier::mirror_lane(Lane::Key5), Lane::Key3);
        assert_eq!(MirrorModifier::mirror_lane(Lane::Key6), Lane::Key2);
        assert_eq!(MirrorModifier::mirror_lane(Lane::Key7), Lane::Key1);
    }

    #[test]
    fn test_mirror_lane_2p() {
        assert_eq!(MirrorModifier::mirror_lane(Lane::Scratch2), Lane::Scratch2);
        assert_eq!(MirrorModifier::mirror_lane(Lane::Key8), Lane::Key14);
        assert_eq!(MirrorModifier::mirror_lane(Lane::Key9), Lane::Key13);
        assert_eq!(MirrorModifier::mirror_lane(Lane::Key10), Lane::Key12);
        assert_eq!(MirrorModifier::mirror_lane(Lane::Key11), Lane::Key11);
        assert_eq!(MirrorModifier::mirror_lane(Lane::Key12), Lane::Key10);
        assert_eq!(MirrorModifier::mirror_lane(Lane::Key13), Lane::Key9);
        assert_eq!(MirrorModifier::mirror_lane(Lane::Key14), Lane::Key8);
    }
}
