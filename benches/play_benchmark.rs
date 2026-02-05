use brs::state::play::{GaugeProperty, GrooveGauge, JudgeRank};
use criterion::{Criterion, black_box, criterion_group, criterion_main};

fn gauge_update_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("gauge");

    group.bench_function("normal_gauge_pg", |b| {
        let mut gauge = GrooveGauge::new(GaugeProperty::sevenkeys_normal(), 100.0, 100);
        b.iter(|| {
            gauge.update(black_box(JudgeRank::PerfectGreat));
        });
    });

    group.bench_function("hard_gauge_miss", |b| {
        let mut gauge = GrooveGauge::new(GaugeProperty::sevenkeys_hard(), 100.0, 100);
        b.iter(|| {
            gauge.update(black_box(JudgeRank::Miss));
            // Reset to avoid gauge going to 0
            if gauge.value() < 10.0 {
                gauge = GrooveGauge::new(GaugeProperty::sevenkeys_hard(), 100.0, 100);
            }
        });
    });

    group.finish();
}

fn judge_rank_benchmark(c: &mut Criterion) {
    c.bench_function("judge_rank_continues_combo", |b| {
        let ranks = [
            JudgeRank::PerfectGreat,
            JudgeRank::Great,
            JudgeRank::Good,
            JudgeRank::Bad,
            JudgeRank::Poor,
            JudgeRank::Miss,
        ];
        let mut i = 0;
        b.iter(|| {
            let rank = black_box(ranks[i % ranks.len()]);
            let _ = black_box(rank.continues_combo());
            i += 1;
        });
    });
}

criterion_group!(benches, gauge_update_benchmark, judge_rank_benchmark);
criterion_main!(benches);
