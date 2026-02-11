use std::path::Path;

use bms_model::{BmsDecoder, LnType, PlayMode};
use bms_rule::judge_manager::{JudgeConfig, JudgeManager};
use bms_rule::{
    GaugeType, GrooveGauge, JUDGE_GR, JUDGE_MS, JUDGE_PG, JudgeAlgorithm, JudgeProperty,
    JudgeWindowRule,
};
use criterion::{Criterion, criterion_group, criterion_main};

fn test_bms_path(name: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-bms")
        .join(name)
}

fn bench_judge_manager_new(c: &mut Criterion) {
    let path = test_bms_path("minimal_7k.bms");
    let model = BmsDecoder::decode(&path).unwrap();
    let notes = model.build_judge_notes();
    let jp = JudgeProperty::sevenkeys();

    let config = JudgeConfig {
        notes: &notes,
        play_mode: PlayMode::Beat7K,
        ln_type: LnType::LongNote,
        judge_rank: model.judge_rank,
        judge_window_rate: [100, 100, 100],
        scratch_judge_window_rate: [100, 100, 100],
        algorithm: JudgeAlgorithm::Combo,
        autoplay: true,
        judge_property: &jp,
        lane_property: None,
    };

    c.bench_function("judge_manager_new", |b| {
        b.iter(|| JudgeManager::new(&config));
    });
}

fn bench_autoplay_simulation(c: &mut Criterion) {
    let path = test_bms_path("minimal_7k.bms");
    let model = BmsDecoder::decode(&path).unwrap();
    let notes = model.build_judge_notes();
    let jp = JudgeProperty::sevenkeys();
    let gauge_prop = bms_rule::gauge_property::sevenkeys();

    c.bench_function("autoplay_simulation", |b| {
        b.iter(|| {
            let config = JudgeConfig {
                notes: &notes,
                play_mode: PlayMode::Beat7K,
                ln_type: LnType::LongNote,
                judge_rank: model.judge_rank,
                judge_window_rate: [100, 100, 100],
                scratch_judge_window_rate: [100, 100, 100],
                algorithm: JudgeAlgorithm::Combo,
                autoplay: true,
                judge_property: &jp,
                lane_property: None,
            };
            let mut jm = JudgeManager::new(&config);
            let mut gauge =
                GrooveGauge::new(&gauge_prop, GaugeType::Normal, model.total, notes.len());

            let key_count = PlayMode::Beat7K.key_count();
            let key_states = vec![false; key_count];
            let key_times = vec![0i64; key_count];

            // Simulate time progression through the chart
            if let Some(last_note) = notes.last() {
                let end_time = last_note.time_us + 1_000_000;
                let step = 16_666; // ~60fps
                let mut time = 0i64;
                while time <= end_time {
                    jm.update(time, &notes, &key_states, &key_times, &mut gauge);
                    time += step;
                }
            }
        });
    });
}

fn bench_groove_gauge_update(c: &mut Criterion) {
    let gauge_prop = bms_rule::gauge_property::sevenkeys();

    c.bench_function("groove_gauge_update", |b| {
        b.iter(|| {
            let mut gauge = GrooveGauge::new(&gauge_prop, GaugeType::Normal, 300.0, 100);
            // Simulate a sequence of judge results
            for _ in 0..100 {
                gauge.update(JUDGE_PG);
                gauge.update(JUDGE_GR);
                gauge.update(JUDGE_MS);
            }
        });
    });
}

fn bench_judge_window_create(c: &mut Criterion) {
    let rules = [
        ("normal", JudgeWindowRule::Normal),
        ("pms", JudgeWindowRule::Pms),
        ("lr2", JudgeWindowRule::Lr2),
    ];

    for (name, rule) in rules {
        let base = JudgeProperty::sevenkeys();
        c.bench_function(&format!("judge_window_create_{name}"), |b| {
            b.iter(|| {
                rule.create(&base.note, 100, &[100, 100, 100]);
            });
        });
    }
}

criterion_group!(
    benches,
    bench_judge_manager_new,
    bench_autoplay_simulation,
    bench_groove_gauge_update,
    bench_judge_window_create,
);
criterion_main!(benches);
