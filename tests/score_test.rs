use brs::game::{JudgeResult, ScoreManager};

#[test]
fn test_ex_score_calculation() {
    let mut score = ScoreManager::new();

    score.add_judgment(JudgeResult::PGreat);
    assert_eq!(score.ex_score(), 2);

    score.add_judgment(JudgeResult::Great);
    assert_eq!(score.ex_score(), 3);

    score.add_judgment(JudgeResult::Good);
    assert_eq!(score.ex_score(), 3);

    score.add_judgment(JudgeResult::Bad);
    assert_eq!(score.ex_score(), 3);

    score.add_judgment(JudgeResult::Poor);
    assert_eq!(score.ex_score(), 3);
}

#[test]
fn test_combo() {
    let mut score = ScoreManager::new();

    score.add_judgment(JudgeResult::PGreat);
    assert_eq!(score.combo, 1);

    score.add_judgment(JudgeResult::Great);
    assert_eq!(score.combo, 2);

    score.add_judgment(JudgeResult::Good);
    assert_eq!(score.combo, 3);

    // Bad breaks combo
    score.add_judgment(JudgeResult::Bad);
    assert_eq!(score.combo, 0);

    score.add_judgment(JudgeResult::PGreat);
    assert_eq!(score.combo, 1);

    // Poor breaks combo
    score.add_judgment(JudgeResult::Poor);
    assert_eq!(score.combo, 0);
}

#[test]
fn test_max_combo() {
    let mut score = ScoreManager::new();

    for _ in 0..5 {
        score.add_judgment(JudgeResult::PGreat);
    }
    assert_eq!(score.max_combo, 5);

    score.add_judgment(JudgeResult::Poor);
    assert_eq!(score.max_combo, 5);

    for _ in 0..3 {
        score.add_judgment(JudgeResult::Great);
    }
    assert_eq!(score.max_combo, 5);
}

#[test]
fn test_judgment_counts() {
    let mut score = ScoreManager::new();

    score.add_judgment(JudgeResult::PGreat);
    score.add_judgment(JudgeResult::PGreat);
    score.add_judgment(JudgeResult::Great);
    score.add_judgment(JudgeResult::Good);
    score.add_judgment(JudgeResult::Bad);
    score.add_judgment(JudgeResult::Poor);
    score.add_judgment(JudgeResult::Poor);

    assert_eq!(score.pgreat_count, 2);
    assert_eq!(score.great_count, 1);
    assert_eq!(score.good_count, 1);
    assert_eq!(score.bad_count, 1);
    assert_eq!(score.poor_count, 2);
}

#[test]
fn test_reset() {
    let mut score = ScoreManager::new();

    score.add_judgment(JudgeResult::PGreat);
    score.add_judgment(JudgeResult::Great);
    score.reset();

    assert_eq!(score.pgreat_count, 0);
    assert_eq!(score.great_count, 0);
    assert_eq!(score.combo, 0);
    assert_eq!(score.max_combo, 0);
    assert_eq!(score.ex_score(), 0);
}
