use criterion::{Criterion, criterion_group, criterion_main};

fn render_benchmark(_c: &mut Criterion) {
    // TODO: Add render benchmarks
}

criterion_group!(benches, render_benchmark);
criterion_main!(benches);
