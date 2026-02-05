use criterion::{Criterion, criterion_group, criterion_main};

fn play_benchmark(_c: &mut Criterion) {
    // TODO: Add play benchmarks
}

criterion_group!(benches, play_benchmark);
criterion_main!(benches);
