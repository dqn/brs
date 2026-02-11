use std::path::Path;

use bms_model::BmsDecoder;
use criterion::{Criterion, criterion_group, criterion_main};

fn test_bms_path(name: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-bms")
        .join(name)
}

fn bench_parse(c: &mut Criterion) {
    let files = ["minimal_7k.bms", "bpm_change.bms", "longnote_types.bms"];
    for file in files {
        let path = test_bms_path(file);
        c.bench_function(&format!("parse_{file}"), |b| {
            b.iter(|| BmsDecoder::decode(&path).unwrap());
        });
    }
}

fn bench_build_judge_notes(c: &mut Criterion) {
    let path = test_bms_path("minimal_7k.bms");
    let model = BmsDecoder::decode(&path).unwrap();
    c.bench_function("build_judge_notes", |b| {
        b.iter(|| model.build_judge_notes());
    });
}

criterion_group!(benches, bench_parse, bench_build_judge_notes);
criterion_main!(benches);
