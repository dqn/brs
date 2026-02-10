use serde::{Deserialize, Serialize};

use bms_rule::{ClearType, ScoreData};

/// IR score data for transmission.
///
/// Corresponds to Java `IRScoreData`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IRScoreData {
    pub sha256: String,
    /// LN TYPE (0: LN, 1: CN, 2: HCN)
    pub lntype: i32,
    /// Player name (empty string for self)
    pub player: String,
    pub clear: ClearType,
    /// Last score date (unix timestamp, seconds)
    pub date: i64,
    pub epg: i32,
    pub lpg: i32,
    pub egr: i32,
    pub lgr: i32,
    pub egd: i32,
    pub lgd: i32,
    pub ebd: i32,
    pub lbd: i32,
    pub epr: i32,
    pub lpr: i32,
    pub ems: i32,
    pub lms: i32,
    pub avgjudge: i64,
    pub maxcombo: i32,
    pub notes: i32,
    pub passnotes: i32,
    pub minbp: i32,
    pub option: i32,
    pub seed: i64,
    pub assist: i32,
    pub gauge: i32,
    pub skin: String,
}

impl IRScoreData {
    /// Calculate EX score: PGREAT * 2 + GREAT
    pub fn exscore(&self) -> i32 {
        (self.epg + self.lpg) * 2 + self.egr + self.lgr
    }

    /// Convert to ScoreData.
    ///
    /// Note: Java has a bug in passnotes conversion:
    /// `this.passnotes != 0 ? this.notes : this.passnotes`
    /// This is faithfully reproduced here.
    pub fn to_score_data(&self) -> ScoreData {
        // Java bug: `this.passnotes != 0 ? this.notes : this.passnotes`
        // When passnotes != 0, it returns notes instead of passnotes.
        let passnotes = if self.passnotes != 0 {
            self.notes
        } else {
            self.passnotes
        };

        ScoreData {
            sha256: self.sha256.clone(),
            mode: self.lntype,
            player: self.player.clone(),
            clear: self.clear,
            date: self.date,
            epg: self.epg,
            lpg: self.lpg,
            egr: self.egr,
            lgr: self.lgr,
            egd: self.egd,
            lgd: self.lgd,
            ebd: self.ebd,
            lbd: self.lbd,
            epr: self.epr,
            lpr: self.lpr,
            ems: self.ems,
            lms: self.lms,
            maxcombo: self.maxcombo,
            notes: self.notes,
            passnotes,
            minbp: self.minbp,
            avgjudge: self.avgjudge,
            option: self.option,
            seed: self.seed,
            assist: self.assist,
            gauge: self.gauge,
            ..Default::default()
        }
    }
}

impl From<&ScoreData> for IRScoreData {
    fn from(score: &ScoreData) -> Self {
        Self {
            sha256: score.sha256.clone(),
            lntype: score.mode,
            player: score.player.clone(),
            clear: score.clear,
            date: score.date,
            epg: score.epg,
            lpg: score.lpg,
            egr: score.egr,
            lgr: score.lgr,
            egd: score.egd,
            lgd: score.lgd,
            ebd: score.ebd,
            lbd: score.lbd,
            epr: score.epr,
            lpr: score.lpr,
            ems: score.ems,
            lms: score.lms,
            avgjudge: score.avgjudge,
            maxcombo: score.maxcombo,
            notes: score.notes,
            passnotes: score.passnotes,
            minbp: score.minbp,
            option: score.option,
            seed: score.seed,
            assist: score.assist,
            gauge: score.gauge,
            skin: String::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_score_data() -> ScoreData {
        let mut sd = ScoreData::default();
        sd.sha256 = "abc123".to_string();
        sd.mode = 1;
        sd.player = "TestPlayer".to_string();
        sd.clear = ClearType::Hard;
        sd.date = 1700000000;
        sd.epg = 100;
        sd.lpg = 50;
        sd.egr = 30;
        sd.lgr = 20;
        sd.egd = 5;
        sd.lgd = 3;
        sd.ebd = 2;
        sd.lbd = 1;
        sd.epr = 1;
        sd.lpr = 0;
        sd.ems = 0;
        sd.lms = 0;
        sd.maxcombo = 200;
        sd.notes = 500;
        sd.passnotes = 500;
        sd.minbp = 4;
        sd.avgjudge = 1000;
        sd.option = 0;
        sd.seed = 42;
        sd.assist = 0;
        sd.gauge = 2;
        sd
    }

    #[test]
    fn from_score_data() {
        let sd = sample_score_data();
        let ir = IRScoreData::from(&sd);
        assert_eq!(ir.sha256, "abc123");
        assert_eq!(ir.lntype, 1);
        assert_eq!(ir.player, "TestPlayer");
        assert_eq!(ir.clear, ClearType::Hard);
        assert_eq!(ir.epg, 100);
        assert_eq!(ir.lpg, 50);
        assert_eq!(ir.egr, 30);
        assert_eq!(ir.lgr, 20);
        assert_eq!(ir.maxcombo, 200);
        assert_eq!(ir.notes, 500);
        assert_eq!(ir.minbp, 4);
    }

    #[test]
    fn exscore_calculation() {
        let sd = sample_score_data();
        let ir = IRScoreData::from(&sd);
        // (100 + 50) * 2 + 30 + 20 = 300 + 50 = 350
        assert_eq!(ir.exscore(), 350);
    }

    #[test]
    fn to_score_data_round_trip() {
        let sd = sample_score_data();
        let ir = IRScoreData::from(&sd);
        let converted = ir.to_score_data();
        assert_eq!(converted.sha256, sd.sha256);
        assert_eq!(converted.mode, sd.mode);
        assert_eq!(converted.player, sd.player);
        assert_eq!(converted.clear, sd.clear);
        assert_eq!(converted.epg, sd.epg);
        assert_eq!(converted.lpg, sd.lpg);
        assert_eq!(converted.egr, sd.egr);
        assert_eq!(converted.lgr, sd.lgr);
        assert_eq!(converted.maxcombo, sd.maxcombo);
        assert_eq!(converted.minbp, sd.minbp);
    }

    #[test]
    fn to_score_data_passnotes_bug() {
        // Java bug: passnotes != 0 returns notes instead of passnotes
        let mut sd = ScoreData::default();
        sd.notes = 500;

        // When passnotes is 0, it stays 0
        let ir_zero = IRScoreData::from(&sd);
        assert_eq!(ir_zero.to_score_data().passnotes, 0);

        // When passnotes != 0, Java returns notes instead
        sd.passnotes = 300;
        let ir_nonzero = IRScoreData::from(&sd);
        assert_eq!(ir_nonzero.to_score_data().passnotes, 500); // notes, not 300
    }

    #[test]
    fn serde_round_trip() {
        let sd = sample_score_data();
        let ir = IRScoreData::from(&sd);
        let json = serde_json::to_string(&ir).unwrap();
        let deserialized: IRScoreData = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.sha256, ir.sha256);
        assert_eq!(deserialized.exscore(), ir.exscore());
    }
}
