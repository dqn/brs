use crate::state::play::{GaugeType, Score};

/// Rank based on clear rate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rank {
    Max,
    AAA,
    AA,
    A,
    B,
    C,
    D,
    E,
    F,
}

impl Rank {
    /// Calculate rank from clear rate (0.0 - 100.0).
    pub fn from_clear_rate(rate: f64) -> Self {
        if rate >= 100.0 {
            Self::Max
        } else if rate >= 88.89 {
            Self::AAA
        } else if rate >= 77.78 {
            Self::AA
        } else if rate >= 66.67 {
            Self::A
        } else if rate >= 55.56 {
            Self::B
        } else if rate >= 44.44 {
            Self::C
        } else if rate >= 33.33 {
            Self::D
        } else if rate >= 22.22 {
            Self::E
        } else {
            Self::F
        }
    }

    /// Get the display string for this rank.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Max => "MAX",
            Self::AAA => "AAA",
            Self::AA => "AA",
            Self::A => "A",
            Self::B => "B",
            Self::C => "C",
            Self::D => "D",
            Self::E => "E",
            Self::F => "F",
        }
    }
}

/// Result of a play session.
#[derive(Debug, Clone)]
pub struct PlayResult {
    pub score: Score,
    pub gauge_value: f64,
    pub gauge_type: GaugeType,
    pub is_clear: bool,
    pub play_time_ms: f64,
    pub fast_count: u32,
    pub slow_count: u32,
}

impl PlayResult {
    /// Create a new play result.
    pub fn new(
        score: Score,
        gauge_value: f64,
        gauge_type: GaugeType,
        is_clear: bool,
        play_time_ms: f64,
        fast_count: u32,
        slow_count: u32,
    ) -> Self {
        Self {
            score,
            gauge_value,
            gauge_type,
            is_clear,
            play_time_ms,
            fast_count,
            slow_count,
        }
    }

    /// Get the rank based on clear rate.
    pub fn rank(&self) -> Rank {
        Rank::from_clear_rate(self.score.clear_rate())
    }

    /// Get the EX-SCORE.
    pub fn ex_score(&self) -> u32 {
        self.score.ex_score()
    }

    /// Get the max combo.
    pub fn max_combo(&self) -> u32 {
        self.score.max_combo
    }

    /// Get the BP count.
    pub fn bp(&self) -> u32 {
        self.score.bp()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rank_from_clear_rate() {
        assert_eq!(Rank::from_clear_rate(100.0), Rank::Max);
        assert_eq!(Rank::from_clear_rate(99.9), Rank::AAA);
        assert_eq!(Rank::from_clear_rate(88.89), Rank::AAA);
        assert_eq!(Rank::from_clear_rate(88.88), Rank::AA);
        assert_eq!(Rank::from_clear_rate(0.0), Rank::F);
    }

    #[test]
    fn test_rank_display() {
        assert_eq!(Rank::Max.as_str(), "MAX");
        assert_eq!(Rank::AAA.as_str(), "AAA");
        assert_eq!(Rank::F.as_str(), "F");
    }
}
