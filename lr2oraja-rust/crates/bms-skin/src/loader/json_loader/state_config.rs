// State-specific config collection for JSON skin loading.
//
// Scans skin objects and extracts state-specific objects into their
// corresponding config structs (play, select, result, course result).

use crate::skin_object_type::SkinObjectType;

/// Scans skin objects and extracts play-specific objects into PlaySkinConfig.
pub(super) fn collect_play_config(
    objects: &[SkinObjectType],
) -> Option<crate::play_skin::PlaySkinConfig> {
    let mut config = crate::play_skin::PlaySkinConfig::default();
    let mut found = false;

    for obj in objects {
        match obj {
            SkinObjectType::Note(n) => {
                config.note = Some(n.clone());
                found = true;
            }
            SkinObjectType::Judge(j) => {
                config.judges.push(*j.clone());
                found = true;
            }
            SkinObjectType::Bga(b) => {
                config.bga = Some(b.clone());
                found = true;
            }
            SkinObjectType::Hidden(h) => {
                config.hidden_cover = Some(h.clone());
                found = true;
            }
            SkinObjectType::LiftCover(l) => {
                config.lift_cover = Some(l.clone());
                found = true;
            }
            _ => {}
        }
    }

    if found { Some(config) } else { None }
}

/// Scans skin objects and extracts select-specific objects into MusicSelectSkinConfig.
pub(super) fn collect_select_config(
    objects: &[SkinObjectType],
) -> Option<crate::music_select_skin::MusicSelectSkinConfig> {
    let mut config = crate::music_select_skin::MusicSelectSkinConfig::default();
    let mut found = false;

    for obj in objects {
        match obj {
            SkinObjectType::Bar(bar) => {
                config.bar = Some(bar.clone());
                found = true;
            }
            SkinObjectType::DistributionGraph(g) => {
                config.distribution_graph = Some(g.clone());
                found = true;
            }
            _ => {}
        }
    }

    if found { Some(config) } else { None }
}

/// Scans skin objects and extracts result-specific objects into ResultSkinConfig.
pub(super) fn collect_result_config(
    objects: &[SkinObjectType],
) -> Option<crate::result_skin::ResultSkinConfig> {
    let mut config = crate::result_skin::ResultSkinConfig::default();
    let mut found = false;

    for obj in objects {
        match obj {
            SkinObjectType::GaugeGraph(g) => {
                config.gauge_graph = Some(g.clone());
                found = true;
            }
            SkinObjectType::NoteDistributionGraph(g) => {
                config.note_graph = Some(g.clone());
                found = true;
            }
            SkinObjectType::BpmGraph(g) => {
                config.bpm_graph = Some(g.clone());
                found = true;
            }
            SkinObjectType::TimingDistributionGraph(g) => {
                config.timing_graph = Some(g.clone());
                found = true;
            }
            _ => {}
        }
    }

    if found { Some(config) } else { None }
}

/// Scans skin objects and extracts course-result-specific objects.
pub(super) fn collect_course_result_config(
    objects: &[SkinObjectType],
) -> Option<crate::result_skin::CourseResultSkinConfig> {
    let mut config = crate::result_skin::CourseResultSkinConfig::default();
    let mut found = false;

    for obj in objects {
        match obj {
            SkinObjectType::GaugeGraph(g) => {
                config.gauge_graph = Some(g.clone());
                found = true;
            }
            SkinObjectType::NoteDistributionGraph(g) => {
                config.note_graph = Some(g.clone());
                found = true;
            }
            _ => {}
        }
    }

    if found { Some(config) } else { None }
}
