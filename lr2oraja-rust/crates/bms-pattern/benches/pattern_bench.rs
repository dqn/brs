use std::path::Path;

use bms_model::BmsDecoder;
use bms_pattern::{
    LaneMirrorShuffle, LanePlayableRandomShuffle, LaneRandomShuffle, NoteShuffleModifier,
    PatternModifier, RandomType,
};
use criterion::{Criterion, criterion_group, criterion_main};

fn test_bms_path(name: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-bms")
        .join(name)
}

fn bench_lane_mirror(c: &mut Criterion) {
    let path = test_bms_path("minimal_7k.bms");
    let model = BmsDecoder::decode(&path).unwrap();

    c.bench_function("lane_mirror", |b| {
        b.iter(|| {
            let mut m = model.clone();
            let mut shuffle = LaneMirrorShuffle::new(0, false);
            shuffle.modify(&mut m);
        });
    });
}

fn bench_lane_random(c: &mut Criterion) {
    let path = test_bms_path("minimal_7k.bms");
    let model = BmsDecoder::decode(&path).unwrap();

    c.bench_function("lane_random", |b| {
        b.iter(|| {
            let mut m = model.clone();
            let mut shuffle = LaneRandomShuffle::new(0, false, 12345);
            shuffle.modify(&mut m);
        });
    });
}

fn bench_note_srandom(c: &mut Criterion) {
    let path = test_bms_path("minimal_7k.bms");
    let model = BmsDecoder::decode(&path).unwrap();

    c.bench_function("note_srandom", |b| {
        b.iter(|| {
            let mut m = model.clone();
            let mut shuffle = NoteShuffleModifier::new(RandomType::SRandom, 0, 12345, 0);
            shuffle.modify(&mut m);
        });
    });
}

fn bench_playable_random(c: &mut Criterion) {
    let path = test_bms_path("minimal_7k.bms");
    let model = BmsDecoder::decode(&path).unwrap();

    c.bench_function("playable_random", |b| {
        b.iter(|| {
            let mut m = model.clone();
            let mut shuffle = LanePlayableRandomShuffle::new(0, false, 12345);
            shuffle.modify(&mut m);
        });
    });
}

criterion_group!(
    benches,
    bench_lane_mirror,
    bench_lane_random,
    bench_note_srandom,
    bench_playable_random,
);
criterion_main!(benches);
