use bms_player::game::{JudgeConfig, JudgeResult, JudgeSystem};

#[test]
fn test_pgreat_window() {
    let judge = JudgeSystem::new(JudgeConfig::normal());

    assert_eq!(judge.judge(0.0), Some(JudgeResult::PGreat));
    assert_eq!(judge.judge(20.0), Some(JudgeResult::PGreat));
    assert_eq!(judge.judge(-20.0), Some(JudgeResult::PGreat));
    assert_eq!(judge.judge(19.9), Some(JudgeResult::PGreat));
}

#[test]
fn test_great_window() {
    let judge = JudgeSystem::new(JudgeConfig::normal());

    assert_eq!(judge.judge(21.0), Some(JudgeResult::Great));
    assert_eq!(judge.judge(60.0), Some(JudgeResult::Great));
    assert_eq!(judge.judge(-21.0), Some(JudgeResult::Great));
    assert_eq!(judge.judge(-60.0), Some(JudgeResult::Great));
}

#[test]
fn test_good_window() {
    let judge = JudgeSystem::new(JudgeConfig::normal());

    assert_eq!(judge.judge(61.0), Some(JudgeResult::Good));
    assert_eq!(judge.judge(150.0), Some(JudgeResult::Good));
    assert_eq!(judge.judge(-61.0), Some(JudgeResult::Good));
    assert_eq!(judge.judge(-150.0), Some(JudgeResult::Good));
}

#[test]
fn test_bad_window() {
    let judge = JudgeSystem::new(JudgeConfig::normal());

    assert_eq!(judge.judge(151.0), Some(JudgeResult::Bad));
    assert_eq!(judge.judge(280.0), Some(JudgeResult::Bad));
    assert_eq!(judge.judge(-151.0), Some(JudgeResult::Bad));
    assert_eq!(judge.judge(-280.0), Some(JudgeResult::Bad));
}

#[test]
fn test_outside_window() {
    let judge = JudgeSystem::new(JudgeConfig::normal());

    assert_eq!(judge.judge(281.0), None);
    assert_eq!(judge.judge(-281.0), None);
    assert_eq!(judge.judge(1000.0), None);
}

#[test]
fn test_is_missed() {
    let judge = JudgeSystem::new(JudgeConfig::normal());

    assert!(!judge.is_missed(0.0));
    assert!(!judge.is_missed(-280.0));
    assert!(judge.is_missed(-281.0));
    assert!(judge.is_missed(-1000.0));
}
