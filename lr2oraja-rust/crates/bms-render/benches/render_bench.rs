use bms_render::coord::{
    RotationParams, ScreenSize, SkinRect, skin_to_bevy_position, skin_to_bevy_transform,
};
use bms_render::eval::{
    apply_offset, check_draw_conditions, resolve_common, resolve_text_content, resolve_timer_time,
    shadow_color_from_main,
};
use bms_render::state_provider::StaticStateProvider;
use bms_skin::property_id::{BooleanId, StringId, TimerId};
use bms_skin::skin_object::{Color, Destination, Rect, SkinObjectBase, SkinOffset};
use bms_skin::skin_text::SkinText;
use criterion::{Criterion, criterion_group, criterion_main};

fn make_base_with_animation() -> SkinObjectBase {
    let mut base = SkinObjectBase::default();
    // Multi-keyframe animation
    for i in 0..10 {
        let t = i as i64 * 100;
        let x = i as f32 * 50.0;
        base.add_destination(Destination {
            time: t,
            region: Rect::new(x, 0.0, 200.0, 100.0),
            color: Color::white(),
            angle: i * 36,
            acc: 0,
        });
    }
    base
}

fn make_provider_with_state() -> StaticStateProvider {
    let mut p = StaticStateProvider::default();
    p.time_ms = 500;
    for i in 1..=20 {
        p.booleans.insert(i, i % 3 != 0);
        p.timers.insert(i, i as i64 * 100);
        p.integers.insert(i, i * 10);
        p.strings.insert(i, format!("value_{i}"));
    }
    for i in 1..=5 {
        p.offsets.insert(
            i,
            SkinOffset {
                x: i as f32 * 2.0,
                y: i as f32 * -1.0,
                w: 0.0,
                h: 0.0,
                r: i as f32 * 5.0,
                a: i as f32 * 10.0,
            },
        );
    }
    p
}

fn bench_check_draw_conditions(c: &mut Criterion) {
    let provider = make_provider_with_state();

    // Benchmark with no conditions (fast path)
    let base_empty = SkinObjectBase::default();
    c.bench_function("check_draw_conditions_empty", |b| {
        b.iter(|| check_draw_conditions(&base_empty, &provider));
    });

    // Benchmark with several conditions
    let mut base_many = SkinObjectBase::default();
    base_many.draw_conditions = (1..=10).map(BooleanId).collect();
    c.bench_function("check_draw_conditions_10", |b| {
        b.iter(|| check_draw_conditions(&base_many, &provider));
    });
}

fn bench_resolve_timer_time(c: &mut Criterion) {
    let provider = make_provider_with_state();

    // No timer — uses now_time_ms
    let base_no_timer = SkinObjectBase::default();
    c.bench_function("resolve_timer_time_no_timer", |b| {
        b.iter(|| resolve_timer_time(&base_no_timer, &provider));
    });

    // With timer
    let mut base_timer = SkinObjectBase::default();
    base_timer.timer = Some(TimerId(5));
    c.bench_function("resolve_timer_time_active", |b| {
        b.iter(|| resolve_timer_time(&base_timer, &provider));
    });
}

fn bench_resolve_common(c: &mut Criterion) {
    let provider = make_provider_with_state();

    // Simple: single destination, no conditions, no offsets
    let mut base_simple = SkinObjectBase::default();
    base_simple.add_destination(Destination {
        time: 0,
        region: Rect::new(100.0, 50.0, 200.0, 100.0),
        color: Color::white(),
        angle: 0,
        acc: 0,
    });
    c.bench_function("resolve_common_simple", |b| {
        b.iter(|| resolve_common(&base_simple, &provider));
    });

    // Complex: multi-keyframe animation + conditions + offsets
    let mut base_complex = make_base_with_animation();
    base_complex.draw_conditions = vec![BooleanId(1), BooleanId(2)];
    base_complex.set_offset_ids(&[1, 2, 3]);
    c.bench_function("resolve_common_complex", |b| {
        b.iter(|| resolve_common(&base_complex, &provider));
    });
}

fn bench_apply_offset(c: &mut Criterion) {
    let offset = SkinOffset {
        x: 10.0,
        y: -5.0,
        w: 2.0,
        h: -1.0,
        r: 15.0,
        a: 32.0,
    };
    c.bench_function("apply_offset", |b| {
        b.iter(|| {
            let mut rect = Rect::new(100.0, 200.0, 300.0, 150.0);
            let mut angle = 0.0_f32;
            let mut alpha = 0.0_f32;
            apply_offset(&mut rect, &offset, &mut angle, &mut alpha);
        });
    });

    // Multiple offsets in sequence
    let offsets: Vec<SkinOffset> = (0..5)
        .map(|i| SkinOffset {
            x: i as f32 * 3.0,
            y: i as f32 * -2.0,
            w: 1.0,
            h: -0.5,
            r: i as f32 * 10.0,
            a: i as f32 * 5.0,
        })
        .collect();
    c.bench_function("apply_offset_x5", |b| {
        b.iter(|| {
            let mut rect = Rect::new(0.0, 0.0, 100.0, 100.0);
            let mut angle = 0.0_f32;
            let mut alpha = 0.0_f32;
            for off in &offsets {
                apply_offset(&mut rect, off, &mut angle, &mut alpha);
            }
        });
    });
}

fn bench_resolve_text_content(c: &mut Criterion) {
    let provider = make_provider_with_state();

    // With ref_id that resolves
    let text_ref = SkinText {
        ref_id: Some(StringId(1)),
        constant_text: Some("fallback".to_string()),
        ..Default::default()
    };
    c.bench_function("resolve_text_content_ref", |b| {
        b.iter(|| resolve_text_content(&text_ref, &provider));
    });

    // With constant text only
    let text_const = SkinText {
        ref_id: None,
        constant_text: Some("static text".to_string()),
        ..Default::default()
    };
    c.bench_function("resolve_text_content_constant", |b| {
        b.iter(|| resolve_text_content(&text_const, &provider));
    });
}

fn bench_shadow_color(c: &mut Criterion) {
    c.bench_function("shadow_color_from_main", |b| {
        b.iter(|| shadow_color_from_main(0.8, 0.6, 0.4, 0.9));
    });
}

fn bench_coord_conversion(c: &mut Criterion) {
    c.bench_function("skin_to_bevy_position", |b| {
        b.iter(|| {
            skin_to_bevy_position(100.0, 200.0, 300.0, 150.0, 1920.0, 1080.0);
        });
    });

    c.bench_function("skin_to_bevy_transform_no_rotation", |b| {
        b.iter(|| {
            skin_to_bevy_transform(
                SkinRect {
                    x: 100.0,
                    y: 200.0,
                    w: 300.0,
                    h: 150.0,
                },
                ScreenSize {
                    w: 1920.0,
                    h: 1080.0,
                },
                5.0,
                RotationParams {
                    angle_deg: 0,
                    center_x: 0.5,
                    center_y: 0.5,
                },
            );
        });
    });

    c.bench_function("skin_to_bevy_transform_rotated", |b| {
        b.iter(|| {
            skin_to_bevy_transform(
                SkinRect {
                    x: 100.0,
                    y: 200.0,
                    w: 300.0,
                    h: 150.0,
                },
                ScreenSize {
                    w: 1920.0,
                    h: 1080.0,
                },
                5.0,
                RotationParams {
                    angle_deg: 45,
                    center_x: 0.3,
                    center_y: 0.7,
                },
            );
        });
    });

    // Batch of 100 transforms
    let rects: Vec<SkinRect> = (0..100)
        .map(|i| SkinRect {
            x: (i % 20) as f32 * 96.0,
            y: (i / 20) as f32 * 216.0,
            w: 80.0,
            h: 80.0,
        })
        .collect();
    c.bench_function("skin_to_bevy_transform_batch_100", |b| {
        let screen = ScreenSize {
            w: 1920.0,
            h: 1080.0,
        };
        let rot = RotationParams {
            angle_deg: 0,
            center_x: 0.5,
            center_y: 0.5,
        };
        b.iter(|| {
            for (i, r) in rects.iter().enumerate() {
                skin_to_bevy_transform(*r, screen, i as f32 * 0.01, rot);
            }
        });
    });
}

criterion_group!(
    benches,
    bench_check_draw_conditions,
    bench_resolve_timer_time,
    bench_resolve_common,
    bench_apply_offset,
    bench_resolve_text_content,
    bench_shadow_color,
    bench_coord_conversion,
);
criterion_main!(benches);
