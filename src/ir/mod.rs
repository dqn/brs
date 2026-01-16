#![allow(dead_code)]
#![allow(unused_imports)]

mod client;
mod protocol;
mod validation;

pub use client::IrClient;
pub use protocol::{
    ChartRanking, IrServerType, IrSubmitState, PlayOptionFlags, RankingEntry, ScoreSubmission,
    SubmissionResponse,
};
pub use validation::{
    ScoreHashData, ValidationResult, compute_md5_hash, generate_score_hash, validate_score,
};
