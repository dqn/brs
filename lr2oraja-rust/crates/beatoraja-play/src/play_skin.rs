use crate::pomyu_chara_processor::PomyuCharaProcessor;

/// Play skin
pub struct PlaySkin {
    /// Margin from STATE_READY to STATE_PLAY (ms)
    playstart: i32,
    /// Judge region count
    judgeregion: i32,
    /// Margin from STATE_FAILED to exit (ms)
    close: i32,
    /// Margin from STATE_FINISHED to fadeout (ms)
    finish_margin: i32,
    loadstart: i32,
    loadend: i32,
    /// Judge timer trigger condition (0:PG, 1:GR, 2:GD, 3:BD)
    judgetimer: i32,
    /// PMS rhythm-based note expansion rate (%) [w, h]
    note_expansion_rate: [i32; 2],
    /// PMS character processor
    pub pomyu: PomyuCharaProcessor,
}

impl Default for PlaySkin {
    fn default() -> Self {
        Self::new()
    }
}

impl PlaySkin {
    pub fn new() -> Self {
        PlaySkin {
            playstart: 0,
            judgeregion: 0,
            close: 0,
            finish_margin: 0,
            loadstart: 0,
            loadend: 0,
            judgetimer: 1,
            note_expansion_rate: [100, 100],
            pomyu: PomyuCharaProcessor::new(),
        }
    }

    pub fn get_judgeregion(&self) -> i32 {
        self.judgeregion
    }

    pub fn set_judgeregion(&mut self, jr: i32) {
        self.judgeregion = jr;
    }

    pub fn get_close(&self) -> i32 {
        self.close
    }

    pub fn set_close(&mut self, close: i32) {
        self.close = close;
    }

    pub fn get_finish_margin(&self) -> i32 {
        self.finish_margin
    }

    pub fn set_finish_margin(&mut self, finish_margin: i32) {
        self.finish_margin = finish_margin;
    }

    pub fn get_playstart(&self) -> i32 {
        self.playstart
    }

    pub fn set_playstart(&mut self, playstart: i32) {
        self.playstart = playstart;
    }

    pub fn get_loadstart(&self) -> i32 {
        self.loadstart
    }

    pub fn set_loadstart(&mut self, loadstart: i32) {
        self.loadstart = loadstart;
    }

    pub fn get_loadend(&self) -> i32 {
        self.loadend
    }

    pub fn set_loadend(&mut self, loadend: i32) {
        self.loadend = loadend;
    }

    pub fn get_judgetimer(&self) -> i32 {
        self.judgetimer
    }

    pub fn set_judgetimer(&mut self, judgetimer: i32) {
        self.judgetimer = judgetimer;
    }

    pub fn get_note_expansion_rate(&self) -> &[i32; 2] {
        &self.note_expansion_rate
    }

    pub fn set_note_expansion_rate(&mut self, rate: [i32; 2]) {
        self.note_expansion_rate = rate;
    }
}
