// LR2 ghost data parser for Internet Ranking.
//
// Matches Java `LR2GhostData.java`.

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};

use crate::lr2_random::LR2Random;

/// Ghost judgment values matching LR2IR encoding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GhostJudgment {
    PGreat = 0,
    Great = 1,
    Good = 2,
    Bad = 3,
    Poor = 4,
}

impl GhostJudgment {
    fn from_int(v: i32) -> Option<Self> {
        match v {
            0 => Some(Self::PGreat),
            1 => Some(Self::Great),
            2 => Some(Self::Good),
            3 => Some(Self::Bad),
            4 => Some(Self::Poor),
            _ => None,
        }
    }
}

/// LR2 random option for ghost data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LR2RandomOption {
    Identity,
    Mirror,
    Random,
}

/// Parsed LR2 ghost data from Internet Ranking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LR2GhostData {
    pub random: LR2RandomOption,
    pub seed: i32,
    pub lane_order: i32,
    pub judgements: Vec<GhostJudgment>,
    pub pgreat: i32,
    pub great: i32,
    pub good: i32,
    pub bad: i32,
    pub poor: i32,
}

impl LR2GhostData {
    /// Parse LR2IR ghost CSV data.
    ///
    /// CSV format: `name,options,seed,ghost`
    /// The options field is a 4-digit decimal: gauge, random1, random2, dpflip.
    pub fn parse(ghost_csv: &str) -> Result<Self> {
        // Simple CSV parsing (no external crate needed)
        // Format: name,options,seed,ghost
        // First line is header, second line is data
        let lines: Vec<&str> = ghost_csv.lines().collect();
        if lines.len() < 2 {
            return Err(anyhow!(
                "Ghost CSV must have at least 2 lines (header + data)"
            ));
        }

        let data_line = lines[1];
        let fields: Vec<&str> = split_csv_line(data_line);
        if fields.len() < 4 {
            return Err(anyhow!("Ghost CSV data must have at least 4 fields"));
        }

        let options: i32 = fields[1]
            .trim()
            .parse()
            .map_err(|_| anyhow!("Invalid options field"))?;
        let random_digit = (options / 10) % 10;

        if random_digit >= 3 {
            return Err(anyhow!("Unsupported random option: {}", random_digit));
        }

        let random = match random_digit {
            1 => LR2RandomOption::Mirror,
            2 => LR2RandomOption::Random,
            _ => LR2RandomOption::Identity,
        };

        let seed: i32 = fields[2]
            .trim()
            .parse()
            .map_err(|_| anyhow!("Invalid seed field"))?;

        // Generate lane ordering using LR2Random + Fisher-Yates
        let mut rng = LR2Random::new(seed);
        let mut targets = [0i32, 1, 2, 3, 4, 5, 6, 7];
        for lane in 1..7 {
            let swap = lane + rng.next_int(7 - lane as i32 + 1) as usize;
            targets.swap(lane, swap);
        }
        let mut lanes = [0i32, 1, 2, 3, 4, 5, 6, 7];
        for i in 1..8 {
            lanes[targets[i] as usize] = i as i32;
        }
        let mut encoded_lanes = 0i32;
        for lane in lanes.iter().skip(1) {
            encoded_lanes = encoded_lanes * 10 + lane;
        }

        let ghost_str = fields[3].trim();
        let judgement_ints = decode_play_ghost(ghost_str);

        let mut pgreat = 0;
        let mut great = 0;
        let mut good = 0;
        let mut bad = 0;
        let mut poor = 0;
        let mut judgements = Vec::with_capacity(judgement_ints.len());
        for &j in &judgement_ints {
            match j {
                0 => pgreat += 1,
                1 => great += 1,
                2 => good += 1,
                3 => bad += 1,
                _ => poor += 1,
            }
            if let Some(gj) = GhostJudgment::from_int(j) {
                judgements.push(gj);
            }
        }

        Ok(LR2GhostData {
            random,
            seed,
            lane_order: encoded_lanes,
            judgements,
            pgreat,
            great,
            good,
            bad,
            poor,
        })
    }
}

/// Simple CSV line splitter that handles quoted fields.
fn split_csv_line(line: &str) -> Vec<&str> {
    // LR2IR ghost CSV uses simple comma-separated values without quotes
    line.split(',').collect()
}

/// Decode LR2IR play ghost data.
///
/// Performs character substitution (matching Java order exactly),
/// then decodes the resulting run-length encoded sequence.
pub fn decode_play_ghost(data: &str) -> Vec<i32> {
    let mut data = data.to_string();

    // Character substitution table — ORDER MATTERS (matches Java exactly)
    // First pass: single-char → 2-char expansions
    data = data.replace("q", "XX");
    data = data.replace("r", "X1");
    data = data.replace("s", "X2");
    data = data.replace("t", "X3");
    data = data.replace("u", "X4");
    data = data.replace("v", "X5");
    data = data.replace("w", "X6");
    data = data.replace("x", "X7");
    data = data.replace("y", "X8");
    data = data.replace("z", "X9");

    data = data.replace("F", "E1");
    data = data.replace("G", "E2");
    data = data.replace("H", "E3");
    data = data.replace("I", "E4");
    data = data.replace("J", "E5");
    data = data.replace("K", "E6");
    data = data.replace("L", "E7");
    data = data.replace("M", "E8");
    data = data.replace("N", "E9");
    data = data.replace("P", "EC");
    data = data.replace("Q", "EB");
    data = data.replace("R", "EA");
    data = data.replace("S", "D2");
    data = data.replace("T", "D3");
    data = data.replace("U", "D4");
    data = data.replace("V", "D5");
    data = data.replace("W", "D6");
    data = data.replace("X", "DE");
    data = data.replace("Y", "DC");
    data = data.replace("a", "DB");
    data = data.replace("b", "DA");
    data = data.replace("c", "C2");
    data = data.replace("d", "C3");
    data = data.replace("e", "C4");
    data = data.replace("f", "C5");
    data = data.replace("g", "CE");
    data = data.replace("h", "CD");
    data = data.replace("i", "CB");
    data = data.replace("j", "CA");
    data = data.replace("k", "AB");
    data = data.replace("l", "AC");
    data = data.replace("m", "AD");
    data = data.replace("n", "AE");
    data = data.replace("o", "A2");
    data = data.replace("p", "A3");

    // Guard character
    data.push('?');

    // RLE decode: e.g., "ED3CE2" → [E, D, D, D, C, E, E]
    let mut notes: Vec<char> = Vec::new();
    let mut run_length: i32 = 0;
    let mut current_char: Option<char> = None;

    for next in data.chars() {
        if next == '?' {
            if let Some(ch) = current_char {
                if run_length == 0 {
                    run_length = 1;
                }
                for _ in 0..run_length {
                    notes.push(ch);
                }
            }
            break;
        } else if next.is_ascii_digit() {
            run_length = run_length * 10 + (next as i32 - '0' as i32);
        } else if ('@'..='E').contains(&next) {
            if let Some(ch) = current_char {
                if run_length == 0 {
                    run_length = 1;
                }
                for _ in 0..run_length {
                    notes.push(ch);
                }
            }
            current_char = Some(next);
            run_length = 0;
        }
        // Other characters are ignored
    }

    // Convert characters to judgment values, skipping '@' (mash poors)
    let mut ghost = Vec::new();
    for &ch in &notes {
        let judge = match ch {
            'E' => 0, // pgreat
            'D' => 1, // great
            'C' => 2, // good
            'B' => 3, // bad
            'A' => 4, // poor
            _ => continue,
        };
        ghost.push(judge);
    }

    ghost
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_simple() {
        // Simple: all pgreats
        let result = decode_play_ghost("E3");
        assert_eq!(result, vec![0, 0, 0]);
    }

    #[test]
    fn test_decode_mixed() {
        // EDDDCEE
        let result = decode_play_ghost("ED3CE2");
        assert_eq!(result, vec![0, 1, 1, 1, 2, 0, 0]);
    }

    #[test]
    fn test_decode_single_chars() {
        // Each character once
        let result = decode_play_ghost("EDCBA");
        assert_eq!(result, vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn test_decode_with_substitution() {
        // 'F' → 'E1' → one E followed by (1 is a digit, so run_length for next char)
        // Actually: F → E1, so it becomes "E" then "1"
        // The '1' digit is a run length for the next character
        let result = decode_play_ghost("FD");
        // F → E1 → "E" "1" "D" → E once, then D×1
        assert_eq!(result, vec![0, 1]);
    }

    #[test]
    fn test_decode_empty() {
        let result = decode_play_ghost("");
        assert!(result.is_empty());
    }

    #[test]
    fn test_parse_ghost_csv() {
        let csv = "name,options,seed,ghost\nplayer,0020,12345,E3D2";
        let ghost = LR2GhostData::parse(csv).unwrap();
        assert_eq!(ghost.random, LR2RandomOption::Random);
        assert_eq!(ghost.seed, 12345);
        assert_eq!(ghost.judgements.len(), 5);
    }

    #[test]
    fn test_parse_identity_option() {
        let csv = "name,options,seed,ghost\nplayer,0000,42,E";
        let ghost = LR2GhostData::parse(csv).unwrap();
        assert_eq!(ghost.random, LR2RandomOption::Identity);
    }

    #[test]
    fn test_parse_mirror_option() {
        let csv = "name,options,seed,ghost\nplayer,0010,42,E";
        let ghost = LR2GhostData::parse(csv).unwrap();
        assert_eq!(ghost.random, LR2RandomOption::Mirror);
    }

    #[test]
    fn test_parse_unsupported_random() {
        let csv = "name,options,seed,ghost\nplayer,0030,42,E";
        assert!(LR2GhostData::parse(csv).is_err());
    }

    #[test]
    fn test_lane_ordering_deterministic() {
        let csv1 = "name,options,seed,ghost\np,0000,100,E";
        let csv2 = "name,options,seed,ghost\np,0000,100,E";
        let g1 = LR2GhostData::parse(csv1).unwrap();
        let g2 = LR2GhostData::parse(csv2).unwrap();
        assert_eq!(g1.lane_order, g2.lane_order);
    }

    #[test]
    fn test_judgment_counts() {
        let csv = "name,options,seed,ghost\np,0000,1,E3D2CBA";
        let ghost = LR2GhostData::parse(csv).unwrap();
        assert_eq!(ghost.pgreat, 3);
        assert_eq!(ghost.great, 2);
        assert_eq!(ghost.good, 1);
        assert_eq!(ghost.bad, 1);
        assert_eq!(ghost.poor, 1);
    }
}
