use crate::skin_distribution_graph::SkinDistributionGraph;
use crate::stubs::*;

/// Skin object for rendering song bars
/// Translates: bms.player.beatoraja.select.SkinBar
pub struct SkinBar {
    /// SkinImage for selected bars
    pub barimageon: Vec<Option<SkinImage>>,
    /// SkinImage for non-selected bars
    pub barimageoff: Vec<Option<SkinImage>>,
    /// Trophy SkinImages (relative to bar position)
    pub trophy: Vec<Option<SkinImage>>,
    /// Bar text SkinText objects
    /// Index: 0=normal, 1=new, 2=SongBar(normal), 3=SongBar(new), 4=FolderBar(normal),
    /// 5=FolderBar(new), 6=TableBar/HashBar, 7=GradeBar(songs exist),
    /// 8=(SongBar/GradeBar)(songs missing), 9=CommandBar/ContainerBar, 10=SearchWordBar
    pub text: Vec<Option<SkinText>>,
    /// Level SkinNumbers (relative to bar position)
    pub barlevel: Vec<Option<SkinNumber>>,
    /// Label SkinImages (relative to bar position)
    pub label: Vec<Option<SkinImage>>,
    /// Distribution graph
    pub graph: Option<SkinDistributionGraph>,
    /// Position mode
    pub position: i32,
    /// Lamp images
    pub lamp: Vec<Option<SkinImage>>,
    /// Player lamp images (for rival display)
    pub mylamp: Vec<Option<SkinImage>>,
    /// Rival lamp images (for rival display)
    pub rivallamp: Vec<Option<SkinImage>>,
    /// SkinObject base data
    pub draw: bool,
    pub region: SkinRegion,
}

impl SkinBar {
    pub const BAR_COUNT: usize = 60;
    pub const BARTROPHY_COUNT: usize = 3;
    pub const BARTEXT_NORMAL: usize = 0;
    pub const BARTEXT_NEW: usize = 1;
    pub const BARTEXT_SONG_NORMAL: usize = 2;
    pub const BARTEXT_SONG_NEW: usize = 3;
    pub const BARTEXT_FOLDER_NORMAL: usize = 4;
    pub const BARTEXT_FOLDER_NEW: usize = 5;
    pub const BARTEXT_TABLE: usize = 6;
    pub const BARTEXT_GRADE: usize = 7;
    pub const BARTEXT_NO_SONGS: usize = 8;
    pub const BARTEXT_COMMAND: usize = 9;
    pub const BARTEXT_SEARCH: usize = 10;
    pub const BARTEXT_COUNT: usize = 11;
    pub const BARLEVEL_COUNT: usize = 7;
    pub const BARLABEL_COUNT: usize = 5;
    pub const BARLAMP_COUNT: usize = 11;

    pub fn new(position: i32) -> Self {
        Self {
            barimageon: vec![None; Self::BAR_COUNT],
            barimageoff: vec![None; Self::BAR_COUNT],
            trophy: vec![None; Self::BARTROPHY_COUNT],
            text: vec![None; Self::BARTEXT_COUNT],
            barlevel: vec![None; Self::BARLEVEL_COUNT],
            label: vec![None; Self::BARLABEL_COUNT],
            graph: None,
            position,
            lamp: vec![None; Self::BARLAMP_COUNT],
            mylamp: vec![None; Self::BARLAMP_COUNT],
            rivallamp: vec![None; Self::BARLAMP_COUNT],
            draw: false,
            region: SkinRegion::default(),
        }
    }

    pub fn set_bar_image(
        &mut self,
        onimage: Vec<Option<SkinImage>>,
        offimage: Vec<Option<SkinImage>>,
    ) {
        self.barimageon = onimage;
        self.barimageoff = offimage;
    }

    pub fn get_bar_images(&self, on: bool, index: usize) -> Option<&SkinImage> {
        if index < self.barimageoff.len() {
            if on {
                self.barimageon[index].as_ref()
            } else {
                self.barimageoff[index].as_ref()
            }
        } else {
            None
        }
    }

    pub fn get_lamp(&self, id: i32) -> Option<&SkinImage> {
        if id >= 0 && (id as usize) < self.lamp.len() {
            self.lamp[id as usize].as_ref()
        } else {
            None
        }
    }

    pub fn get_player_lamp(&self, id: i32) -> Option<&SkinImage> {
        if id >= 0 && (id as usize) < self.mylamp.len() {
            self.mylamp[id as usize].as_ref()
        } else {
            None
        }
    }

    pub fn get_rival_lamp(&self, id: i32) -> Option<&SkinImage> {
        if id >= 0 && (id as usize) < self.rivallamp.len() {
            self.rivallamp[id as usize].as_ref()
        } else {
            None
        }
    }

    pub fn get_trophy(&self, id: i32) -> Option<&SkinImage> {
        if id >= 0 && (id as usize) < self.trophy.len() {
            self.trophy[id as usize].as_ref()
        } else {
            None
        }
    }

    pub fn get_text(&self, id: usize) -> Option<&SkinText> {
        if id < self.text.len() {
            self.text[id].as_ref()
        } else {
            None
        }
    }

    pub fn set_trophy(&mut self, id: i32, trophy: SkinImage) {
        if id >= 0 && (id as usize) < self.trophy.len() {
            self.trophy[id as usize] = Some(trophy);
        }
    }

    pub fn set_lamp_image(&mut self, id: i32, lamp: SkinImage) {
        if id >= 0 && (id as usize) < self.lamp.len() {
            self.lamp[id as usize] = Some(lamp);
        }
    }

    pub fn set_player_lamp(&mut self, id: i32, mylamp: SkinImage) {
        if id >= 0 && (id as usize) < self.mylamp.len() {
            self.mylamp[id as usize] = Some(mylamp);
        }
    }

    pub fn set_text(&mut self, id: usize, text: SkinText) {
        if id < self.text.len() {
            self.text[id] = Some(text);
        }
    }

    pub fn set_rival_lamp(&mut self, id: i32, rivallamp: SkinImage) {
        if id >= 0 && (id as usize) < self.rivallamp.len() {
            self.rivallamp[id as usize] = Some(rivallamp);
        }
    }

    /// Validate all sub-objects, removing invalid ones.
    /// Translates: Java SkinBar.validate()
    pub fn validate(&mut self) -> bool {
        fn validate_images(images: &mut [Option<SkinImage>]) {
            for img in images.iter_mut() {
                if img.as_ref().is_some_and(|i| !i.validate()) {
                    *img = None;
                }
            }
        }
        fn validate_texts(texts: &mut [Option<SkinText>]) {
            for txt in texts.iter_mut() {
                if txt.as_ref().is_some_and(|t| !t.validate()) {
                    *txt = None;
                }
            }
        }

        validate_images(&mut self.barimageon);
        validate_images(&mut self.barimageoff);
        validate_images(&mut self.trophy);
        validate_images(&mut self.label);
        validate_images(&mut self.lamp);
        validate_images(&mut self.mylamp);
        validate_images(&mut self.rivallamp);
        validate_texts(&mut self.text);
        true
    }

    /// Prepare all sub-objects for rendering.
    /// In Java: prepares all child SkinImage/SkinText/SkinNumber, then calls render.prepare(this, time).
    /// In Rust: sub-object preparation is done here. BarRenderer.prepare() is called separately
    /// by MusicSelector since it requires context (center_bar, currentsongs, selectedindex)
    /// that can't be obtained from &dyn MainState without downcasting.
    pub fn prepare(&mut self, time: i64, state: &dyn MainState) {
        // Prepare all child skin objects
        for bar in self.barimageon.iter().flatten() {
            bar.prepare(time, state);
        }
        for bar in self.barimageoff.iter().flatten() {
            bar.prepare(time, state);
        }
        for trophy in self.trophy.iter().flatten() {
            trophy.prepare(time, state);
        }
        for text in self.text.iter().flatten() {
            text.prepare(time, state);
        }
        for barlevel in self.barlevel.iter().flatten() {
            barlevel.prepare(time, state);
        }
        for label in self.label.iter().flatten() {
            label.prepare(time, state);
        }
        for lamp in self.lamp.iter().flatten() {
            lamp.prepare(time, state);
        }
        for mylamp in self.mylamp.iter().flatten() {
            mylamp.prepare(time, state);
        }
        for rivallamp in self.rivallamp.iter().flatten() {
            rivallamp.prepare(time, state);
        }
        if let Some(ref mut graph) = self.graph {
            graph.prepare(time, state);
        }
        // NOTE: BarRenderer.prepare(baro, time, ctx) is called by MusicSelector
        // after this method, since it requires PrepareContext with center_bar, etc.
    }

    /// Draw all bar elements.
    /// In Java: render.render(sprite, this).
    /// In Rust: BarRenderer.render() is called separately by MusicSelector
    /// since it requires RenderContext with center_bar, currentsongs, rival, etc.
    pub fn draw(&mut self, _sprite: &mut SkinObjectRenderer) {
        // NOTE: BarRenderer.render(sprite, baro, ctx) is called by MusicSelector
        // after prepare(), since it requires RenderContext.
        // This stub remains for API compatibility with the skin pipeline.
    }

    pub fn dispose(&self) {
        // In Java: disposes all sub-objects
    }

    pub fn get_barlevel(&self, id: i32) -> Option<&SkinNumber> {
        if id >= 0 && (id as usize) < self.barlevel.len() {
            self.barlevel[id as usize].as_ref()
        } else {
            None
        }
    }

    pub fn set_barlevel(&mut self, id: i32, barlevel: SkinNumber) {
        if id >= 0 && (id as usize) < self.barlevel.len() {
            self.barlevel[id as usize] = Some(barlevel);
        }
    }

    pub fn get_position(&self) -> i32 {
        self.position
    }

    pub fn get_label(&self, id: i32) -> Option<&SkinImage> {
        if id >= 0 && (id as usize) < self.label.len() {
            self.label[id as usize].as_ref()
        } else {
            None
        }
    }

    pub fn set_label(&mut self, id: i32, label: SkinImage) {
        if id >= 0 && (id as usize) < self.label.len() {
            self.label[id as usize] = Some(label);
        }
    }

    /// Handle mouse press on bar.
    /// In Java: return ((MusicSelector) state).getBarRender().mousePressed(this, button, x, y).
    /// In Rust: BarRenderer.mouse_pressed() is called separately by MusicSelector
    /// since it requires MousePressedContext. This stub returns false for API compatibility.
    pub fn mouse_pressed(&self, _state: &dyn MainState, _button: i32, _x: i32, _y: i32) -> bool {
        // NOTE: BarRenderer.mouse_pressed(baro, button, x, y, ctx) is called by MusicSelector.
        false
    }

    pub fn get_graph(&self) -> Option<&SkinDistributionGraph> {
        self.graph.as_ref()
    }

    pub fn set_graph(&mut self, graph: SkinDistributionGraph) {
        self.graph = Some(graph);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skin_bar_new_initializes_arrays() {
        let bar = SkinBar::new(0);
        assert_eq!(bar.barimageon.len(), SkinBar::BAR_COUNT);
        assert_eq!(bar.barimageoff.len(), SkinBar::BAR_COUNT);
        assert_eq!(bar.trophy.len(), SkinBar::BARTROPHY_COUNT);
        assert_eq!(bar.text.len(), SkinBar::BARTEXT_COUNT);
        assert_eq!(bar.barlevel.len(), SkinBar::BARLEVEL_COUNT);
        assert_eq!(bar.label.len(), SkinBar::BARLABEL_COUNT);
        assert_eq!(bar.lamp.len(), SkinBar::BARLAMP_COUNT);
        assert_eq!(bar.position, 0);
        assert!(!bar.draw);
    }

    #[test]
    fn test_skin_bar_two_phase_prepare_draw_signatures() {
        // Phase 40a: verify that SkinBar follows the two-phase pattern:
        //   prepare(&mut self, time, state) — mutable phase
        //   draw(&mut self, &mut sprite)    — mutable phase (for scratch-space)
        // This test verifies the signatures compile and can be called sequentially.
        let mut bar = SkinBar::new(0);

        // Phase 1: prepare (stub — logs warning but doesn't panic)
        // We can't call prepare without a real MainState, but we can verify draw flag
        assert!(!bar.draw);

        // Phase 2: draw (stub — logs warning but doesn't panic)
        let mut renderer = SkinObjectRenderer;
        bar.draw(&mut renderer);
        // No panic = success
    }

    #[test]
    fn test_skin_bar_position_preserved() {
        let bar = SkinBar::new(1);
        assert_eq!(bar.get_position(), 1);
    }

    #[test]
    fn test_skin_bar_get_bar_images_bounds() {
        let bar = SkinBar::new(0);
        // Valid index returns None (no images set)
        assert!(bar.get_bar_images(true, 0).is_none());
        assert!(bar.get_bar_images(false, 0).is_none());
        // Out of bounds returns None
        assert!(bar.get_bar_images(true, SkinBar::BAR_COUNT).is_none());
    }

    #[test]
    fn test_skin_bar_accessors_bounds_checked() {
        let bar = SkinBar::new(0);
        assert!(bar.get_lamp(-1).is_none());
        assert!(bar.get_lamp(0).is_none());
        assert!(bar.get_lamp(SkinBar::BARLAMP_COUNT as i32).is_none());
        assert!(bar.get_trophy(-1).is_none());
        assert!(bar.get_trophy(0).is_none());
        assert!(bar.get_text(SkinBar::BARTEXT_COUNT).is_none());
        assert!(bar.get_barlevel(-1).is_none());
        assert!(bar.get_label(-1).is_none());
    }
}
