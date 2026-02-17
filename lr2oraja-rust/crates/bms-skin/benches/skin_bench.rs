use std::collections::HashSet;

use bms_config::resolution::Resolution;
use bms_skin::loader::json_loader::{
    load_skin, preprocess_json, resolve_conditionals, test_option,
};
use bms_skin::property_mapper;
use criterion::{Criterion, criterion_group, criterion_main};
use serde_json::Value;

// Minimal but realistic JSON skin with image, number, and text objects.
const MINIMAL_SKIN_JSON: &str = r#"{
    "type": 0,
    "name": "Bench Skin",
    "w": 1920,
    "h": 1080,
    "source": [
        {"id": "0", "path": "bg.png", "w": 1920, "h": 1080}
    ],
    "font": [
        {"id": "0", "path": "font.fnt"}
    ],
    "destination": [
        {
            "id": "img0",
            "dst": [
                {"time": 0, "x": 0, "y": 0, "w": 1920, "h": 1080, "a": 255}
            ]
        },
        {
            "id": "num0",
            "dst": [
                {"time": 0, "x": 100, "y": 50, "w": 24, "h": 32, "a": 255}
            ]
        },
        {
            "id": "text0",
            "dst": [
                {"time": 0, "x": 200, "y": 100, "w": 300, "h": 24, "a": 255},
                {"time": 500, "x": 200, "y": 100, "w": 300, "h": 24, "a": 0}
            ]
        }
    ],
    "image": [
        {"id": "img0", "src": "0", "x": 0, "y": 0, "w": 1920, "h": 1080}
    ],
    "value": [
        {"id": "num0", "src": "0", "x": 0, "y": 0, "w": 240, "h": 32, "digit": 5, "ref": 70}
    ],
    "text": [
        {"id": "text0", "font": "0", "st": "Hello", "ref": 10}
    ],
    "property": [
        {"name": "Option A", "item": [{"name": "On", "op": 901}, {"name": "Off", "op": 902}]}
    ],
    "filepath": [],
    "offset": [],
    "category": []
}"#;

// Lenient JSON with trailing commas and missing commas between objects.
const LENIENT_JSON: &str = r#"{
    "type": 0,
    "name": "Lenient Bench",
    "w": 1920,
    "h": 1080,
    "source": [
        {"id": "0", "path": "bg.png", "w": 1920, "h": 1080,},
    ],
    "destination": [
        {"id": "img0", "dst": [{"time": 0, "x": 0, "y": 0, "w": 1920, "h": 1080,},],}
        {"id": "img1", "dst": [{"time": 0, "x": 100, "y": 50, "w": 200, "h": 100,},],}
    ],
    "image": [
        {"id": "img0", "src": "0", "x": 0, "y": 0, "w": 1920, "h": 1080,}
        {"id": "img1", "src": "0", "x": 0, "y": 0, "w": 200, "h": 100,}
    ],
    "property": [],
    "filepath": [],
    "offset": [],
    "category": [],
}"#;

fn bench_load_skin(c: &mut Criterion) {
    let enabled = HashSet::from([901]);
    c.bench_function("load_skin_minimal", |b| {
        b.iter(|| {
            let _ = load_skin(MINIMAL_SKIN_JSON, &enabled, Resolution::Fullhd, None);
        });
    });
}

fn bench_preprocess_json(c: &mut Criterion) {
    c.bench_function("preprocess_json_strict", |b| {
        b.iter(|| preprocess_json(MINIMAL_SKIN_JSON));
    });

    c.bench_function("preprocess_json_lenient", |b| {
        b.iter(|| preprocess_json(LENIENT_JSON));
    });
}

fn bench_load_header(c: &mut Criterion) {
    use bms_skin::loader::json_loader::load_header;

    c.bench_function("load_header", |b| {
        b.iter(|| load_header(MINIMAL_SKIN_JSON).unwrap());
    });
}

fn bench_resolve_conditionals(c: &mut Criterion) {
    // JSON with conditional branches
    let json_with_conditionals = r#"[
        {"if": 901, "value": {"x": 100}},
        {"if": -901, "value": {"x": 200}},
        {"name": "always"},
        {"if": [901, 902], "value": {"x": 300}},
        {"if": [[901, 903]], "values": [{"x": 400}, {"x": 500}]}
    ]"#;
    let value: Value = serde_json::from_str(json_with_conditionals).unwrap();
    let enabled = HashSet::from([901, 902]);

    c.bench_function("resolve_conditionals", |b| {
        b.iter(|| resolve_conditionals(value.clone(), &enabled));
    });
}

fn bench_test_option(c: &mut Criterion) {
    let enabled: HashSet<i32> = (900..920).collect();

    let simple: Value = serde_json::from_str("901").unwrap();
    let negated: Value = serde_json::from_str("-905").unwrap();
    let and_array: Value = serde_json::from_str("[901, 902, 903]").unwrap();
    let or_group: Value = serde_json::from_str("[[901, 950], 902]").unwrap();

    c.bench_function("test_option_simple", |b| {
        b.iter(|| test_option(&simple, &enabled));
    });

    c.bench_function("test_option_negated", |b| {
        b.iter(|| test_option(&negated, &enabled));
    });

    c.bench_function("test_option_and_array", |b| {
        b.iter(|| test_option(&and_array, &enabled));
    });

    c.bench_function("test_option_or_group", |b| {
        b.iter(|| test_option(&or_group, &enabled));
    });
}

fn bench_property_mapper(c: &mut Criterion) {
    c.bench_function("bomb_timer_id", |b| {
        b.iter(|| {
            for player in 0..2 {
                for key in 0..10 {
                    property_mapper::bomb_timer_id(player, key);
                }
            }
        });
    });

    c.bench_function("key_on_timer_id", |b| {
        b.iter(|| {
            for player in 0..2 {
                for key in 0..10 {
                    property_mapper::key_on_timer_id(player, key);
                }
            }
        });
    });

    c.bench_function("key_judge_value_id", |b| {
        b.iter(|| {
            for player in 0..2 {
                for key in 0..10 {
                    property_mapper::key_judge_value_id(player, key);
                }
            }
        });
    });

    c.bench_function("is_skin_select_type_id_scan", |b| {
        b.iter(|| {
            // Scan a range of IDs that covers both matches and misses
            for id in 0..200 {
                property_mapper::is_skin_select_type_id(id);
            }
        });
    });

    c.bench_function("skin_select_type_id_scan", |b| {
        b.iter(|| {
            for id in 0..200 {
                property_mapper::skin_select_type_id(id);
            }
        });
    });
}

criterion_group!(
    benches,
    bench_load_skin,
    bench_preprocess_json,
    bench_load_header,
    bench_resolve_conditionals,
    bench_test_option,
    bench_property_mapper,
);
criterion_main!(benches);
