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

    // Check total judgments vs total notes
    let total_judgments = submission.pgreat_count
        + submission.great_count
        + submission.good_count
        + submission.bad_count;
    // Note: poor_count can include empty POORs, so we don't include it in total

    if total_judgments > submission.total_notes {
        errors.push(format!(
            "Total judgments ({}) exceeds total notes ({})",
            total_judgments, submission.total_notes
        ));
    }

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

    // Check timestamp is reasonable (not in future, not too old)
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    if submission.timestamp > now + 60 {
        errors.push("Timestamp is in the future".to_string());
    }

    // Check for impossible clear states
    // Full combo requires all notes hit with at least GOOD
    if matches!(submission.clear_lamp, crate::game::ClearLamp::FullCombo)
        && (submission.bad_count > 0 || submission.poor_count > 0)
    {
        errors.push("Full combo with BAD or POOR judgments".to_string());
    }

    if errors.is_empty() {
        ValidationResult::valid()
    } else {
        ValidationResult::invalid(errors)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
