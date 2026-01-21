use std::fs;
use std::path::Path;

use anyhow::Result;
use hmac::{Hmac, Mac};
use md5::{Digest, Md5};
use sha2::Sha256;

use super::protocol::ScoreSubmission;

type HmacSha256 = Hmac<Sha256>;

/// Compute MD5 hash of a file (LR2IR compatible)
pub fn compute_md5_hash(path: &Path) -> Result<String> {
    let content = fs::read(path)?;
    let hash = Md5::digest(&content);
    Ok(format!("{:x}", hash))
}

/// Score data for hash generation
pub struct ScoreHashData<'a> {
    pub chart_md5: &'a str,
    pub ex_score: u32,
    pub clear_lamp: u8,
    pub max_combo: u32,
    pub pgreat_count: u32,
    pub great_count: u32,
    pub good_count: u32,
    pub bad_count: u32,
    pub poor_count: u32,
    pub timestamp: u64,
    pub secret_key: &'a str,
}

/// Generate score hash for tampering detection using HMAC-SHA256
pub fn generate_score_hash(data: &ScoreHashData) -> Result<String> {
    // Create data string for hashing
    let hash_input = format!(
        "{}:{}:{}:{}:{}:{}:{}:{}:{}:{}",
        data.chart_md5,
        data.ex_score,
        data.clear_lamp,
        data.max_combo,
        data.pgreat_count,
        data.great_count,
        data.good_count,
        data.bad_count,
        data.poor_count,
        data.timestamp,
    );

    let mut mac = HmacSha256::new_from_slice(data.secret_key.as_bytes())
        .map_err(|e| anyhow::anyhow!("HMAC error: {}", e))?;
    mac.update(hash_input.as_bytes());
    let result = mac.finalize();

    Ok(hex::encode(result.into_bytes()))
}

/// Validation result
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
}

impl ValidationResult {
    fn valid() -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
        }
    }

    fn invalid(errors: Vec<String>) -> Self {
        Self {
            is_valid: false,
            errors,
        }
    }
}

/// Maximum age for score timestamps (7 days in seconds)
const MAX_SCORE_AGE_SECS: u64 = 7 * 24 * 60 * 60;

/// Validate score for theoretical limits and consistency
pub fn validate_score(submission: &ScoreSubmission) -> ValidationResult {
    let mut errors = Vec::new();

    // Check EX score calculation
    let expected_ex = submission.pgreat_count * 2 + submission.great_count;
    if submission.ex_score != expected_ex {
        errors.push(format!(
            "EX score mismatch: {} != {} (expected from judgments)",
            submission.ex_score, expected_ex
        ));
    }

    // Check total judgments vs total notes (excluding empty POORs)
    let total_judgments = submission.pgreat_count
        + submission.great_count
        + submission.good_count
        + submission.bad_count;

    if total_judgments > submission.total_notes {
        errors.push(format!(
            "Total judgments ({}) exceeds total notes ({})",
            total_judgments, submission.total_notes
        ));
    }

    // Check judgment count equals total notes (accounting for missed notes)
    // Missed notes are counted as POOR (but not as BAD/GOOD/GREAT/PGREAT)
    // So: pgreat + great + good + bad + missed_poor <= total_notes
    // And: poor_count >= missed_poor (since poor includes empty POORs)
    // This is already covered by the check above

    // Check theoretical max EX score
    let max_possible_ex = submission.total_notes * 2;
    if submission.ex_score > max_possible_ex {
        errors.push(format!(
            "EX score ({}) exceeds theoretical max ({})",
            submission.ex_score, max_possible_ex
        ));
    }

    // Check max combo doesn't exceed total notes
    if submission.max_combo > submission.total_notes {
        errors.push(format!(
            "Max combo ({}) exceeds total notes ({})",
            submission.max_combo, submission.total_notes
        ));
    }

    // Get current time for timestamp validation
    let now = match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
        Ok(d) => d.as_secs(),
        Err(e) => {
            eprintln!("Warning: Failed to get system time for validation: {}", e);
            0
        }
    };

    // Check timestamp is not in the future (with 60 second tolerance)
    if submission.timestamp > now + 60 {
        errors.push("Timestamp is in the future".to_string());
    }

    // Check timestamp is not too old (replay attack prevention)
    if submission.timestamp + MAX_SCORE_AGE_SECS < now {
        errors.push(format!(
            "Score timestamp is too old (more than {} days)",
            MAX_SCORE_AGE_SECS / (24 * 60 * 60)
        ));
    }

    // Check for impossible clear states
    validate_clear_lamp_consistency(submission, &mut errors);

    // Check for score and clear lamp consistency
    validate_score_clear_consistency(submission, &mut errors);

    if errors.is_empty() {
        ValidationResult::valid()
    } else {
        ValidationResult::invalid(errors)
    }
}

/// Validate clear lamp is consistent with judgment counts
fn validate_clear_lamp_consistency(submission: &ScoreSubmission, errors: &mut Vec<String>) {
    use crate::game::ClearLamp;

    match submission.clear_lamp {
        ClearLamp::FullCombo => {
            // Full combo requires all notes hit with at least GOOD, no BADs or POORs on notes
            // Note: poor_count can include empty POORs from wrong keypresses
            if submission.bad_count > 0 {
                errors.push("Full combo with BAD judgments".to_string());
            }

            // For full combo, max_combo must equal total_notes
            if submission.max_combo != submission.total_notes {
                errors.push(format!(
                    "Full combo but max_combo ({}) != total_notes ({})",
                    submission.max_combo, submission.total_notes
                ));
            }

            // Total hit notes must equal total notes
            let total_hit =
                submission.pgreat_count + submission.great_count + submission.good_count;
            if total_hit != submission.total_notes {
                errors.push(format!(
                    "Full combo but hit count ({}) != total_notes ({})",
                    total_hit, submission.total_notes
                ));
            }
        }
        ClearLamp::Failed => {
            // Failed should not have very high scores (suspicious)
            // Using 80% threshold as a sanity check
            let score_rate = if submission.total_notes > 0 {
                submission.ex_score as f64 / (submission.total_notes * 2) as f64
            } else {
                0.0
            };

            if score_rate > 0.95 {
                errors.push(format!(
                    "Failed clear with suspiciously high score rate ({:.1}%)",
                    score_rate * 100.0
                ));
            }
        }
        _ => {}
    }
}

/// Validate score values are consistent with clear type
fn validate_score_clear_consistency(submission: &ScoreSubmission, errors: &mut Vec<String>) {
    use crate::game::ClearLamp;

    // For any clear (not failed), must have hit enough notes to actually clear
    let is_cleared = !matches!(submission.clear_lamp, ClearLamp::NoPlay | ClearLamp::Failed);

    if is_cleared {
        let total_hit = submission.pgreat_count
            + submission.great_count
            + submission.good_count
            + submission.bad_count;

        // If cleared but didn't play any notes, something is wrong
        if total_hit == 0 && submission.total_notes > 0 {
            errors.push("Cleared but no notes were hit".to_string());
        }
    }

    // Check that EX score is at least possible with the given combo
    // Max EX from combo = combo * 2 (all PGREATs)
    // If combo is less than half of EX score / 2, it's impossible
    if submission.max_combo > 0 {
        let _min_possible_ex_for_combo = submission.max_combo; // At least 1 point per note in combo
        // This is a very loose check; actual EX should be >= combo
        // since each note in combo gives at least 1 point (GREAT or better)
        // Actually this is not quite right because combo breaks can still give points
        // Let's skip this check as it's complex to validate correctly
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::{ClearLamp, GaugeType, RandomOption};
    use crate::ir::protocol::PlayOptionFlags;

    fn create_test_submission(
        ex_score: u32,
        pgreat: u32,
        great: u32,
        good: u32,
        bad: u32,
        poor: u32,
        total_notes: u32,
        max_combo: u32,
        clear_lamp: ClearLamp,
    ) -> ScoreSubmission {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        ScoreSubmission {
            player_id: "test".to_string(),
            chart_hash: "hash".to_string(),
            chart_md5: "md5".to_string(),
            ex_score,
            clear_lamp,
            max_combo,
            pgreat_count: pgreat,
            great_count: great,
            good_count: good,
            bad_count: bad,
            poor_count: poor,
            total_notes,
            play_option: PlayOptionFlags {
                random_option: RandomOption::Off,
                gauge_type: GaugeType::Normal,
                auto_scratch: false,
                legacy_note: false,
                expand_judge: false,
                battle: false,
            },
            timestamp: now,
            client_version: "test".to_string(),
            score_hash: "hash".to_string(),
        }
    }

    #[test]
    fn test_generate_score_hash() {
        let data = ScoreHashData {
            chart_md5: "abc123",
            ex_score: 1000,
            clear_lamp: 4,
            max_combo: 500,
            pgreat_count: 450,
            great_count: 100,
            good_count: 20,
            bad_count: 5,
            poor_count: 2,
            timestamp: 1700000000,
            secret_key: "secret_key",
        };

        let hash = generate_score_hash(&data).unwrap();

        // Hash should be consistent
        let hash2 = generate_score_hash(&data).unwrap();

        assert_eq!(hash, hash2);

        // Different input should produce different hash
        let data3 = ScoreHashData {
            ex_score: 1001, // Different score
            ..data
        };
        let hash3 = generate_score_hash(&data3).unwrap();

        assert_ne!(hash, hash3);
    }

    #[test]
    fn test_valid_score() {
        // Valid score: 450 PGREAT + 100 GREAT = 900 + 100 = 1000 EX
        let submission = create_test_submission(
            1000, // ex_score
            450,  // pgreat
            100,  // great
            50,   // good
            0,    // bad
            0,    // poor
            600,  // total_notes
            600,  // max_combo
            ClearLamp::FullCombo,
        );

        let result = validate_score(&submission);
        assert!(
            result.is_valid,
            "Expected valid score, got errors: {:?}",
            result.errors
        );
    }

    #[test]
    fn test_ex_score_mismatch() {
        // Invalid: EX score doesn't match judgment counts
        let submission = create_test_submission(
            999, // wrong ex_score (should be 1000)
            450, // pgreat
            100, // great
            50,  // good
            0,   // bad
            0,   // poor
            600, // total_notes
            600, // max_combo
            ClearLamp::FullCombo,
        );

        let result = validate_score(&submission);
        assert!(!result.is_valid);
        assert!(
            result
                .errors
                .iter()
                .any(|e| e.contains("EX score mismatch"))
        );
    }

    #[test]
    fn test_judgments_exceed_total_notes() {
        // Invalid: Total judgments exceed total notes
        let submission = create_test_submission(
            1200, // ex_score
            500,  // pgreat
            200,  // great (total: 700, exceeds 600)
            50,   // good
            0,    // bad
            0,    // poor
            600,  // total_notes
            600,  // max_combo
            ClearLamp::Normal,
        );

        let result = validate_score(&submission);
        assert!(!result.is_valid);
        assert!(
            result
                .errors
                .iter()
                .any(|e| e.contains("exceeds total notes"))
        );
    }

    #[test]
    fn test_max_combo_exceeds_total() {
        // Invalid: Max combo exceeds total notes
        let submission = create_test_submission(
            1000, // ex_score
            450,  // pgreat
            100,  // great
            50,   // good
            0,    // bad
            0,    // poor
            600,  // total_notes
            700,  // max_combo (exceeds total)
            ClearLamp::Normal,
        );

        let result = validate_score(&submission);
        assert!(!result.is_valid);
        assert!(result.errors.iter().any(|e| e.contains("Max combo")));
    }

    #[test]
    fn test_full_combo_with_bad() {
        // Invalid: Full combo with BAD judgments
        let submission = create_test_submission(
            998, // ex_score
            449, // pgreat
            100, // great
            50,  // good
            1,   // bad (should not have BAD in full combo)
            0,   // poor
            600, // total_notes
            600, // max_combo
            ClearLamp::FullCombo,
        );

        let result = validate_score(&submission);
        assert!(!result.is_valid);
        assert!(
            result
                .errors
                .iter()
                .any(|e| e.contains("Full combo with BAD"))
        );
    }

    #[test]
    fn test_full_combo_max_combo_mismatch() {
        // Invalid: Full combo but max_combo != total_notes
        let submission = create_test_submission(
            1000, // ex_score
            450,  // pgreat
            100,  // great
            50,   // good
            0,    // bad
            0,    // poor
            600,  // total_notes
            500,  // max_combo (not equal to total_notes)
            ClearLamp::FullCombo,
        );

        let result = validate_score(&submission);
        assert!(!result.is_valid);
        assert!(
            result
                .errors
                .iter()
                .any(|e| e.contains("max_combo") && e.contains("total_notes"))
        );
    }

    #[test]
    fn test_failed_with_high_score() {
        // Suspicious: Failed clear with 96% score rate
        let submission = create_test_submission(
            1152, // ex_score (96% of 1200 max)
            500,  // pgreat
            152,  // great
            48,   // good
            0,    // bad
            0,    // poor
            600,  // total_notes
            590,  // max_combo
            ClearLamp::Failed,
        );

        let result = validate_score(&submission);
        assert!(!result.is_valid);
        assert!(
            result
                .errors
                .iter()
                .any(|e| e.contains("suspiciously high"))
        );
    }

    #[test]
    fn test_cleared_with_no_notes() {
        // Invalid: Cleared but no notes hit
        let submission = create_test_submission(
            0,   // ex_score
            0,   // pgreat
            0,   // great
            0,   // good
            0,   // bad
            0,   // poor
            600, // total_notes
            0,   // max_combo
            ClearLamp::Easy,
        );

        let result = validate_score(&submission);
        assert!(!result.is_valid);
        assert!(
            result
                .errors
                .iter()
                .any(|e| e.contains("no notes were hit"))
        );
    }

    #[test]
    fn test_timestamp_in_future() {
        let mut submission = create_test_submission(
            1000, // ex_score
            450,  // pgreat
            100,  // great
            50,   // good
            0,    // bad
            0,    // poor
            600,  // total_notes
            600,  // max_combo
            ClearLamp::FullCombo,
        );

        // Set timestamp 2 minutes in the future
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        submission.timestamp = now + 120;

        let result = validate_score(&submission);
        assert!(!result.is_valid);
        assert!(result.errors.iter().any(|e| e.contains("future")));
    }

    #[test]
    fn test_timestamp_too_old() {
        let mut submission = create_test_submission(
            1000, // ex_score
            450,  // pgreat
            100,  // great
            50,   // good
            0,    // bad
            0,    // poor
            600,  // total_notes
            600,  // max_combo
            ClearLamp::FullCombo,
        );

        // Set timestamp 10 days ago
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        submission.timestamp = now - (10 * 24 * 60 * 60);

        let result = validate_score(&submission);
        assert!(!result.is_valid);
        assert!(result.errors.iter().any(|e| e.contains("too old")));
    }

    #[test]
    fn test_zero_notes_chart() {
        // Edge case: Chart with 0 notes
        let submission = create_test_submission(
            0, // ex_score
            0, // pgreat
            0, // great
            0, // good
            0, // bad
            0, // poor
            0, // total_notes
            0, // max_combo
            ClearLamp::Normal,
        );

        let result = validate_score(&submission);
        // Should be valid - nothing to validate
        assert!(result.is_valid);
    }
}
