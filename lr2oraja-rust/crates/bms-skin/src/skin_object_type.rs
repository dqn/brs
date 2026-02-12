// SkinObject enum — dispatch type for all concrete skin objects.
//
// Uses enum dispatch instead of trait objects for performance and
// to match existing crate patterns.

use crate::skin_bar::SkinBar;
use crate::skin_bga::SkinBga;
use crate::skin_bpm_graph::SkinBpmGraph;
use crate::skin_distribution_graph::SkinDistributionGraph;
use crate::skin_float::SkinFloat;
use crate::skin_gauge::SkinGauge;
use crate::skin_gauge_graph::SkinGaugeGraph;
use crate::skin_graph::SkinGraph;
use crate::skin_hidden::{SkinHidden, SkinLiftCover};
use crate::skin_image::SkinImage;
use crate::skin_judge::SkinJudge;
use crate::skin_note::SkinNote;
use crate::skin_number::SkinNumber;
use crate::skin_object::SkinObjectBase;
use crate::skin_slider::SkinSlider;
use crate::skin_text::SkinText;
use crate::skin_visualizer::{
    SkinHitErrorVisualizer, SkinNoteDistributionGraph, SkinTimingDistributionGraph,
    SkinTimingVisualizer,
};

/// All concrete skin object types.
#[derive(Debug, Clone)]
pub enum SkinObjectType {
    Bga(SkinBga),
    Image(SkinImage),
    Number(SkinNumber),
    Text(SkinText),
    Slider(SkinSlider),
    Graph(SkinGraph),
    Gauge(SkinGauge),
    BpmGraph(SkinBpmGraph),
    HitErrorVisualizer(SkinHitErrorVisualizer),
    NoteDistributionGraph(SkinNoteDistributionGraph),
    TimingDistributionGraph(SkinTimingDistributionGraph),
    TimingVisualizer(SkinTimingVisualizer),
    Note(SkinNote),
    Judge(Box<SkinJudge>),
    Hidden(SkinHidden),
    LiftCover(SkinLiftCover),
    Bar(SkinBar),
    DistributionGraph(SkinDistributionGraph),
    Float(SkinFloat),
    GaugeGraph(SkinGaugeGraph),
}

impl SkinObjectType {
    /// Returns a reference to the base properties shared by all object types.
    pub fn base(&self) -> &SkinObjectBase {
        match self {
            Self::Bga(o) => &o.base,
            Self::Image(o) => &o.base,
            Self::Number(o) => &o.base,
            Self::Text(o) => &o.base,
            Self::Slider(o) => &o.base,
            Self::Graph(o) => &o.base,
            Self::Gauge(o) => &o.base,
            Self::BpmGraph(o) => &o.base,
            Self::HitErrorVisualizer(o) => &o.base,
            Self::NoteDistributionGraph(o) => &o.base,
            Self::TimingDistributionGraph(o) => &o.base,
            Self::TimingVisualizer(o) => &o.base,
            Self::Note(o) => &o.base,
            Self::Judge(o) => &o.base,
            Self::Hidden(o) => &o.base,
            Self::LiftCover(o) => &o.base,
            Self::Bar(o) => &o.base,
            Self::DistributionGraph(o) => &o.base,
            Self::Float(o) => &o.base,
            Self::GaugeGraph(o) => &o.base,
        }
    }

    /// Returns a mutable reference to the base properties.
    pub fn base_mut(&mut self) -> &mut SkinObjectBase {
        match self {
            Self::Bga(o) => &mut o.base,
            Self::Image(o) => &mut o.base,
            Self::Number(o) => &mut o.base,
            Self::Text(o) => &mut o.base,
            Self::Slider(o) => &mut o.base,
            Self::Graph(o) => &mut o.base,
            Self::Gauge(o) => &mut o.base,
            Self::BpmGraph(o) => &mut o.base,
            Self::HitErrorVisualizer(o) => &mut o.base,
            Self::NoteDistributionGraph(o) => &mut o.base,
            Self::TimingDistributionGraph(o) => &mut o.base,
            Self::TimingVisualizer(o) => &mut o.base,
            Self::Note(o) => &mut o.base,
            Self::Judge(o) => &mut o.base,
            Self::Hidden(o) => &mut o.base,
            Self::LiftCover(o) => &mut o.base,
            Self::Bar(o) => &mut o.base,
            Self::DistributionGraph(o) => &mut o.base,
            Self::Float(o) => &mut o.base,
            Self::GaugeGraph(o) => &mut o.base,
        }
    }
}

// Convenience From impls for each variant.
impl From<SkinBga> for SkinObjectType {
    fn from(v: SkinBga) -> Self {
        Self::Bga(v)
    }
}

impl From<SkinImage> for SkinObjectType {
    fn from(v: SkinImage) -> Self {
        Self::Image(v)
    }
}

impl From<SkinNumber> for SkinObjectType {
    fn from(v: SkinNumber) -> Self {
        Self::Number(v)
    }
}

impl From<SkinText> for SkinObjectType {
    fn from(v: SkinText) -> Self {
        Self::Text(v)
    }
}

impl From<SkinSlider> for SkinObjectType {
    fn from(v: SkinSlider) -> Self {
        Self::Slider(v)
    }
}

impl From<SkinGraph> for SkinObjectType {
    fn from(v: SkinGraph) -> Self {
        Self::Graph(v)
    }
}

impl From<SkinGauge> for SkinObjectType {
    fn from(v: SkinGauge) -> Self {
        Self::Gauge(v)
    }
}

impl From<SkinBpmGraph> for SkinObjectType {
    fn from(v: SkinBpmGraph) -> Self {
        Self::BpmGraph(v)
    }
}

impl From<SkinHitErrorVisualizer> for SkinObjectType {
    fn from(v: SkinHitErrorVisualizer) -> Self {
        Self::HitErrorVisualizer(v)
    }
}

impl From<SkinNoteDistributionGraph> for SkinObjectType {
    fn from(v: SkinNoteDistributionGraph) -> Self {
        Self::NoteDistributionGraph(v)
    }
}

impl From<SkinTimingDistributionGraph> for SkinObjectType {
    fn from(v: SkinTimingDistributionGraph) -> Self {
        Self::TimingDistributionGraph(v)
    }
}

impl From<SkinTimingVisualizer> for SkinObjectType {
    fn from(v: SkinTimingVisualizer) -> Self {
        Self::TimingVisualizer(v)
    }
}

impl From<SkinNote> for SkinObjectType {
    fn from(v: SkinNote) -> Self {
        Self::Note(v)
    }
}

impl From<SkinJudge> for SkinObjectType {
    fn from(v: SkinJudge) -> Self {
        Self::Judge(Box::new(v))
    }
}

impl From<SkinHidden> for SkinObjectType {
    fn from(v: SkinHidden) -> Self {
        Self::Hidden(v)
    }
}

impl From<SkinLiftCover> for SkinObjectType {
    fn from(v: SkinLiftCover) -> Self {
        Self::LiftCover(v)
    }
}

impl From<SkinBar> for SkinObjectType {
    fn from(v: SkinBar) -> Self {
        Self::Bar(v)
    }
}

impl From<SkinDistributionGraph> for SkinObjectType {
    fn from(v: SkinDistributionGraph) -> Self {
        Self::DistributionGraph(v)
    }
}

impl From<SkinFloat> for SkinObjectType {
    fn from(v: SkinFloat) -> Self {
        Self::Float(v)
    }
}

impl From<SkinGaugeGraph> for SkinObjectType {
    fn from(v: SkinGaugeGraph) -> Self {
        Self::GaugeGraph(v)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base_access() {
        let mut obj = SkinObjectType::Image(SkinImage::default());
        assert!(obj.base().destinations.is_empty());
        obj.base_mut().blend = 2;
        assert_eq!(obj.base().blend, 2);
    }

    #[test]
    fn test_from_conversions() {
        let _: SkinObjectType = SkinBga::default().into();
        let _: SkinObjectType = SkinImage::default().into();
        let _: SkinObjectType = SkinNumber::default().into();
        let _: SkinObjectType = SkinText::default().into();
        let _: SkinObjectType = SkinSlider::default().into();
        let _: SkinObjectType = SkinGraph::default().into();
        let _: SkinObjectType = SkinGauge::default().into();
        let _: SkinObjectType = SkinBpmGraph::default().into();
        let _: SkinObjectType = SkinHitErrorVisualizer::default().into();
        let _: SkinObjectType = SkinNoteDistributionGraph::default().into();
        let _: SkinObjectType = SkinTimingDistributionGraph::default().into();
        let _: SkinObjectType = SkinTimingVisualizer::default().into();
        let _: SkinObjectType = SkinNote::default().into();
        let _: SkinObjectType = SkinJudge::default().into();
        let _: SkinObjectType = SkinHidden::default().into();
        let _: SkinObjectType = SkinLiftCover::default().into();
        let _: SkinObjectType = SkinBar::default().into();
        let _: SkinObjectType = SkinDistributionGraph::default().into();
        let _: SkinObjectType = SkinFloat::default().into();
    }

    #[test]
    fn test_variant_match() {
        let obj = SkinObjectType::from(SkinSlider::new(
            crate::property_id::FloatId(17),
            1,
            200,
            true,
        ));
        assert!(matches!(obj, SkinObjectType::Slider(_)));
    }

    #[test]
    fn test_new_variant_matches() {
        assert!(matches!(
            SkinObjectType::from(SkinNote::default()),
            SkinObjectType::Note(_)
        ));
        assert!(matches!(
            SkinObjectType::from(SkinJudge::default()),
            SkinObjectType::Judge(_)
        ));
        assert!(matches!(
            SkinObjectType::from(SkinHidden::default()),
            SkinObjectType::Hidden(_)
        ));
        assert!(matches!(
            SkinObjectType::from(SkinLiftCover::default()),
            SkinObjectType::LiftCover(_)
        ));
        assert!(matches!(
            SkinObjectType::from(SkinBar::default()),
            SkinObjectType::Bar(_)
        ));
        assert!(matches!(
            SkinObjectType::from(SkinDistributionGraph::default()),
            SkinObjectType::DistributionGraph(_)
        ));
        assert!(matches!(
            SkinObjectType::from(SkinFloat::default()),
            SkinObjectType::Float(_)
        ));
    }

    #[test]
    fn test_new_variants_base_access() {
        let mut obj = SkinObjectType::Note(SkinNote::default());
        obj.base_mut().blend = 5;
        assert_eq!(obj.base().blend, 5);

        let mut obj = SkinObjectType::Judge(Box::new(SkinJudge::default()));
        obj.base_mut().blend = 3;
        assert_eq!(obj.base().blend, 3);

        let mut obj = SkinObjectType::Float(SkinFloat::default());
        obj.base_mut().filter = 1;
        assert_eq!(obj.base().filter, 1);
    }
}
