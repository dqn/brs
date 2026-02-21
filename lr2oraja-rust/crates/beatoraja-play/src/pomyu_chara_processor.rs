/// PMS character animation timer processor
pub struct PomyuCharaProcessor {
    /// Motion cycle times: 0:1P_NEUTRAL 1:1P_FEVER 2:1P_GREAT 3:1P_GOOD 4:1P_BAD 5:2P_NEUTRAL 6:2P_GREAT 7:2P_BAD
    pm_chara_time: [i32; 8],
    /// Processed note count at neutral motion start {1P, 2P}
    pm_chara_lastnotes: [i32; 2],
    /// PMS character judge
    pub pm_chara_judge: i32,
}

impl Default for PomyuCharaProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl PomyuCharaProcessor {
    pub fn new() -> Self {
        PomyuCharaProcessor {
            pm_chara_time: [1, 1, 1, 1, 1, 1, 1, 1],
            pm_chara_lastnotes: [0, 0],
            pm_chara_judge: 0,
        }
    }

    pub fn init(&mut self) {
        self.pm_chara_lastnotes[0] = 0;
        self.pm_chara_lastnotes[1] = 0;
        self.pm_chara_judge = 0;
    }

    pub fn get_pm_chara_time(&self, index: i32) -> i32 {
        if index < 0 || index >= self.pm_chara_time.len() as i32 {
            return 1;
        }
        self.pm_chara_time[index as usize]
    }

    pub fn set_pm_chara_time(&mut self, index: i32, value: i32) {
        if index >= 0 && (index as usize) < self.pm_chara_time.len() && value >= 1 {
            self.pm_chara_time[index as usize] = value;
        }
    }

    // updateTimer requires BMSPlayer which has heavy dependencies
    // Stub for now - will be connected when BMSPlayer is fully implemented
    pub fn update_timer_stub(&mut self, past_notes: i32, gauge_is_max: bool) {
        // TODO: Phase 7+ dependency - requires TimerManager, BMSPlayer
        let _ = (past_notes, gauge_is_max);
    }
}
