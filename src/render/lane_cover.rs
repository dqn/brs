/// Lane cover settings for SUDDEN+/HIDDEN/LIFT
#[derive(Debug, Clone, Copy, Default)]
pub struct LaneCover {
    /// SUDDEN+ cover amount (0-1000, covers top of lane)
    pub sudden: u16,
    /// HIDDEN+ cover amount (0-1000, covers bottom after judge line)
    pub hidden: u16,
    /// LIFT amount (0-1000, raises judge line from bottom)
    pub lift: u16,
}

impl LaneCover {
    /// Create a new LaneCover with specified values
    pub fn new(sudden: u16, hidden: u16, lift: u16) -> Self {
        Self {
            sudden,
            hidden,
            lift,
        }
    }

    /// Get visible portion of lane (0.0-1.0)
    #[allow(dead_code)]
    pub fn visible_ratio(&self) -> f32 {
        let covered = (self.sudden + self.hidden + self.lift) as f32 / 1000.0;
        (1.0 - covered).max(0.1) // Minimum 10% visible
    }

    /// Get top edge of visible area (0.0-1.0 from bottom, relative to lane height)
    #[allow(dead_code)]
    pub fn visible_top(&self) -> f32 {
        1.0 - (self.sudden as f32 / 1000.0)
    }

    /// Get bottom edge of visible area (0.0-1.0 from bottom)
    #[allow(dead_code)]
    pub fn visible_bottom(&self) -> f32 {
        (self.lift as f32 + self.hidden as f32) / 1000.0
    }

    /// Get judge line position (0.0-1.0 from bottom)
    pub fn judge_line_position(&self) -> f32 {
        self.lift as f32 / 1000.0
    }

    /// Check if a position is within visible area
    /// position is 0.0-1.0 from bottom of lane
    #[allow(dead_code)]
    pub fn is_visible(&self, position: f32) -> bool {
        position >= self.visible_bottom() && position <= self.visible_top()
    }

    /// Adjust SUDDEN+ value
    pub fn adjust_sudden(&mut self, delta: i16) {
        let new_val = (self.sudden as i16 + delta).clamp(0, 900);
        self.sudden = new_val as u16;
    }

    /// Adjust LIFT value
    pub fn adjust_lift(&mut self, delta: i16) {
        let new_val = (self.lift as i16 + delta).clamp(0, 500);
        self.lift = new_val as u16;
    }

    /// Adjust HIDDEN+ value
    pub fn adjust_hidden(&mut self, delta: i16) {
        let new_val = (self.hidden as i16 + delta).clamp(0, 500);
        self.hidden = new_val as u16;
    }

    /// Toggle SUDDEN+ on/off
    #[allow(dead_code)]
    pub fn toggle_sudden(&mut self) {
        if self.sudden > 0 {
            self.sudden = 0;
        } else {
            self.sudden = 300; // Default SUDDEN+ value
        }
    }

    /// White number for SUDDEN+ (0-1000)
    #[allow(dead_code)]
    pub fn white_number(&self) -> u16 {
        self.sudden
    }
}
