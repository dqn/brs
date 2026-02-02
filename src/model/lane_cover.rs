//! Lane cover settings for SUDDEN+, HIDDEN+, and LIFT.

/// Settings for lane cover effects during gameplay.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LaneCoverSettings {
    /// SUDDEN+ value (0.0 to 1.0).
    /// Hides notes from the top of the lane.
    pub sudden_plus: f32,
    /// HIDDEN+ value (0.0 to 1.0).
    /// Hides notes from the bottom of the lane (near judge line).
    pub hidden_plus: f32,
    /// LIFT value (0.0 to 1.0).
    /// Raises the judge line position.
    pub lift: f32,
}

impl Default for LaneCoverSettings {
    fn default() -> Self {
        Self {
            sudden_plus: 0.0,
            hidden_plus: 0.0,
            lift: 0.0,
        }
    }
}

impl LaneCoverSettings {
    /// Create new lane cover settings.
    pub fn new(sudden_plus: f32, hidden_plus: f32, lift: f32) -> Self {
        Self {
            sudden_plus: sudden_plus.clamp(0.0, 1.0),
            hidden_plus: hidden_plus.clamp(0.0, 1.0),
            lift: lift.clamp(0.0, 0.5), // Limit lift to 50%
        }
    }

    /// Adjustment step for SUDDEN+/HIDDEN+ (per key press).
    const ADJUSTMENT_STEP: f32 = 0.01;

    /// Increase SUDDEN+ value.
    pub fn increase_sudden(&mut self) {
        self.sudden_plus = (self.sudden_plus + Self::ADJUSTMENT_STEP).min(1.0);
    }

    /// Decrease SUDDEN+ value.
    pub fn decrease_sudden(&mut self) {
        self.sudden_plus = (self.sudden_plus - Self::ADJUSTMENT_STEP).max(0.0);
    }

    /// Increase HIDDEN+ value.
    pub fn increase_hidden(&mut self) {
        self.hidden_plus = (self.hidden_plus + Self::ADJUSTMENT_STEP).min(1.0);
    }

    /// Decrease HIDDEN+ value.
    pub fn decrease_hidden(&mut self) {
        self.hidden_plus = (self.hidden_plus - Self::ADJUSTMENT_STEP).max(0.0);
    }

    /// Increase LIFT value.
    pub fn increase_lift(&mut self) {
        self.lift = (self.lift + Self::ADJUSTMENT_STEP).min(0.5);
    }

    /// Decrease LIFT value.
    pub fn decrease_lift(&mut self) {
        self.lift = (self.lift - Self::ADJUSTMENT_STEP).max(0.0);
    }

    /// Check if any cover effect is active.
    pub fn is_active(&self) -> bool {
        self.sudden_plus > 0.0 || self.hidden_plus > 0.0 || self.lift > 0.0
    }

    /// Calculate effective judge line Y position with LIFT applied.
    pub fn effective_judge_line_y(&self, base_judge_line_y: f32, lane_top_y: f32) -> f32 {
        let lane_height = base_judge_line_y - lane_top_y;
        base_judge_line_y - (lane_height * self.lift)
    }

    /// Calculate SUDDEN+ cover top Y position.
    pub fn sudden_cover_bottom_y(&self, lane_top_y: f32, lane_height: f32) -> f32 {
        lane_top_y + (lane_height * self.sudden_plus)
    }

    /// Calculate HIDDEN+ cover top Y position.
    pub fn hidden_cover_top_y(&self, judge_line_y: f32, lane_height: f32) -> f32 {
        judge_line_y - (lane_height * self.hidden_plus)
    }

    /// Check if a Y position is visible (not covered).
    pub fn is_y_visible(&self, y: f32, lane_top_y: f32, judge_line_y: f32) -> bool {
        let lane_height = judge_line_y - lane_top_y;

        // Check SUDDEN+ (top cover)
        let sudden_bottom = self.sudden_cover_bottom_y(lane_top_y, lane_height);
        if y < sudden_bottom {
            return false;
        }

        // Check HIDDEN+ (bottom cover)
        let hidden_top = self.hidden_cover_top_y(judge_line_y, lane_height);
        if y > hidden_top {
            return false;
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_settings() {
        let settings = LaneCoverSettings::default();
        assert_eq!(settings.sudden_plus, 0.0);
        assert_eq!(settings.hidden_plus, 0.0);
        assert_eq!(settings.lift, 0.0);
        assert!(!settings.is_active());
    }

    #[test]
    fn test_clamping() {
        let settings = LaneCoverSettings::new(1.5, -0.5, 0.8);
        assert_eq!(settings.sudden_plus, 1.0);
        assert_eq!(settings.hidden_plus, 0.0);
        assert_eq!(settings.lift, 0.5);
    }

    #[test]
    fn test_adjustment() {
        let mut settings = LaneCoverSettings::default();

        settings.increase_sudden();
        assert!((settings.sudden_plus - 0.01).abs() < 0.001);

        settings.decrease_sudden();
        assert_eq!(settings.sudden_plus, 0.0);

        // Should not go below 0
        settings.decrease_sudden();
        assert_eq!(settings.sudden_plus, 0.0);
    }

    #[test]
    fn test_is_y_visible() {
        let settings = LaneCoverSettings::new(0.2, 0.1, 0.0);
        let lane_top = 100.0;
        let judge_line = 900.0;

        // At top (should be hidden by SUDDEN+)
        assert!(!settings.is_y_visible(150.0, lane_top, judge_line));

        // In middle (should be visible)
        assert!(settings.is_y_visible(500.0, lane_top, judge_line));

        // Near judge line (should be hidden by HIDDEN+)
        assert!(!settings.is_y_visible(850.0, lane_top, judge_line));
    }

    #[test]
    fn test_effective_judge_line_with_lift() {
        let settings = LaneCoverSettings::new(0.0, 0.0, 0.1);
        let lane_top = 100.0;
        let judge_line = 900.0;

        let effective = settings.effective_judge_line_y(judge_line, lane_top);
        // With 10% lift, judge line should move up by 80 pixels (10% of 800)
        assert!((effective - 820.0).abs() < 0.001);
    }
}
