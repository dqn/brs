// Golden master tests: compare Rust rule engine output against Java fixtures

use std::path::Path;

use bms_rule::{Gauge, GaugeProperty, GaugeType, JudgeNoteType, JudgeProperty, gauge_property};
use golden_master::rule_fixtures::{
    GaugePropertyFixture, GaugeSequenceFixture, JudgeWindowFixture,
};

fn fixtures_dir() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .leak()
}

fn load_json<T: serde::de::DeserializeOwned>(name: &str) -> T {
    let path = fixtures_dir().join(name);
    assert!(path.exists(), "Fixture not found: {}", path.display());
    let content = std::fs::read_to_string(&path).expect("Failed to read fixture");
    serde_json::from_str(&content).expect("Failed to parse fixture")
}

fn mode_to_judge_property(name: &str) -> JudgeProperty {
    match name {
        "FIVEKEYS" => JudgeProperty::fivekeys(),
        "SEVENKEYS" => JudgeProperty::sevenkeys(),
        "PMS" => JudgeProperty::pms(),
        "KEYBOARD" => JudgeProperty::keyboard(),
        "LR2" => JudgeProperty::lr2(),
        _ => panic!("Unknown mode: {name}"),
    }
}

fn mode_to_gauge_property(name: &str) -> GaugeProperty {
    match name {
        "FIVEKEYS" => gauge_property::fivekeys(),
        "SEVENKEYS" => gauge_property::sevenkeys(),
        "PMS" => gauge_property::pms(),
        "KEYBOARD" => gauge_property::keyboard(),
        "LR2" => gauge_property::lr2(),
        _ => panic!("Unknown mode: {name}"),
    }
}

fn note_type_to_enum(name: &str) -> JudgeNoteType {
    match name {
        "NOTE" => JudgeNoteType::Note,
        "LONGNOTE_END" => JudgeNoteType::LongNoteEnd,
        "SCRATCH" => JudgeNoteType::Scratch,
        "LONGSCRATCH_END" => JudgeNoteType::LongScratchEnd,
        _ => panic!("Unknown note type: {name}"),
    }
}

// =========================================================================
// Judge Window GM Test
// =========================================================================

#[test]
fn golden_master_judge_windows() {
    let fixture: JudgeWindowFixture = load_json("judge_windows.json");
    let mut failures = Vec::new();

    for (i, tc) in fixture.test_cases.iter().enumerate() {
        let prop = mode_to_judge_property(&tc.mode);
        let note_type = note_type_to_enum(&tc.note_type);
        let jwr: [i32; 3] = [
            tc.judge_window_rate[0],
            tc.judge_window_rate[1],
            tc.judge_window_rate[2],
        ];

        let rust_windows = prop.judge_windows(note_type, tc.judgerank, &jwr);

        // Compare window count
        if rust_windows.len() != tc.windows.len() {
            failures.push(format!(
                "[{i}] {}/{}/{}/jwr={:?}: window_count rust={} java={}",
                tc.mode,
                tc.note_type,
                tc.judgerank,
                tc.judge_window_rate,
                rust_windows.len(),
                tc.windows.len()
            ));
            continue;
        }

        // Compare each window pair (exact match for integer arithmetic)
        for (j, (rw, jw)) in rust_windows.iter().zip(tc.windows.iter()).enumerate() {
            if rw[0] != jw[0] || rw[1] != jw[1] {
                failures.push(format!(
                    "[{i}] {}/{}/{}/jwr={:?} window[{j}]: rust={:?} java={:?}",
                    tc.mode, tc.note_type, tc.judgerank, tc.judge_window_rate, rw, jw
                ));
            }
        }
    }

    if !failures.is_empty() {
        panic!(
            "Judge window GM mismatch ({} failures out of {} cases):\n{}",
            failures.len(),
            fixture.test_cases.len(),
            failures
                .iter()
                .take(20)
                .map(|f| format!("  - {f}"))
                .collect::<Vec<_>>()
                .join("\n")
        );
    }

    println!(
        "Judge window GM: all {} cases passed",
        fixture.test_cases.len()
    );
}

// =========================================================================
// Gauge Property GM Test
// =========================================================================

#[test]
fn golden_master_gauge_properties() {
    let fixture: GaugePropertyFixture = load_json("gauge_properties.json");
    let mut failures = Vec::new();
    let tol = 1e-4_f32;

    for (i, tc) in fixture.test_cases.iter().enumerate() {
        let prop = mode_to_gauge_property(&tc.mode);
        let elem = &prop.elements[tc.gauge_type_index];
        let label = format!(
            "[{i}] {}/{}/total={}/notes={}",
            tc.mode, tc.gauge_type, tc.total, tc.total_notes
        );

        // Static properties
        if (elem.min - tc.min).abs() > tol {
            failures.push(format!("{label}: min rust={} java={}", elem.min, tc.min));
        }
        if (elem.max - tc.max).abs() > tol {
            failures.push(format!("{label}: max rust={} java={}", elem.max, tc.max));
        }
        if (elem.init - tc.init).abs() > tol {
            failures.push(format!("{label}: init rust={} java={}", elem.init, tc.init));
        }
        if (elem.border - tc.border).abs() > tol {
            failures.push(format!(
                "{label}: border rust={} java={}",
                elem.border, tc.border
            ));
        }
        if (elem.death - tc.death).abs() > tol {
            failures.push(format!(
                "{label}: death rust={} java={}",
                elem.death, tc.death
            ));
        }

        // Base values
        for (j, (rv, jv)) in elem.values.iter().zip(tc.base_values.iter()).enumerate() {
            if (rv - jv).abs() > tol {
                failures.push(format!("{label}: base_values[{j}] rust={rv} java={jv}"));
            }
        }

        // Modified values (create a Gauge to get pre-computed values)
        let gauge = Gauge::new(elem.clone(), tc.total, tc.total_notes);
        // Access the gauge's pre-computed values through update simulation
        // We can't access gauge_values directly, so simulate with a fresh gauge
        // and compare the modified values from Java
        for (j, jv) in tc.modified_values.iter().enumerate() {
            let base = elem.values[j];
            let modified = if let Some(modifier) = elem.modifier {
                modifier.modify(base, tc.total, tc.total_notes)
            } else {
                base
            };
            if (modified - jv).abs() > tol {
                failures.push(format!(
                    "{label}: modified_values[{j}] rust={modified} java={jv}"
                ));
            }
        }

        // Guts table
        if elem.guts.len() != tc.guts.len() {
            failures.push(format!(
                "{label}: guts_count rust={} java={}",
                elem.guts.len(),
                tc.guts.len()
            ));
        } else {
            for (j, (rg, jg)) in elem.guts.iter().zip(tc.guts.iter()).enumerate() {
                if (rg.threshold - jg.threshold).abs() > tol
                    || (rg.multiplier - jg.multiplier).abs() > tol
                {
                    failures.push(format!(
                        "{label}: guts[{j}] rust=({},{}) java=({},{})",
                        rg.threshold, rg.multiplier, jg.threshold, jg.multiplier
                    ));
                }
            }
        }

        // Verify Gauge initial value matches
        if (gauge.value() - tc.init).abs() > tol {
            failures.push(format!(
                "{label}: gauge_init rust={} java={}",
                gauge.value(),
                tc.init
            ));
        }

        // Suppress unused variable warning
        let _ = gauge;
    }

    if !failures.is_empty() {
        panic!(
            "Gauge property GM mismatch ({} failures out of {} cases):\n{}",
            failures.len(),
            fixture.test_cases.len(),
            failures
                .iter()
                .take(20)
                .map(|f| format!("  - {f}"))
                .collect::<Vec<_>>()
                .join("\n")
        );
    }

    println!(
        "Gauge property GM: all {} cases passed",
        fixture.test_cases.len()
    );
}

// =========================================================================
// Gauge Sequence GM Test
// =========================================================================

#[test]
fn golden_master_gauge_sequences() {
    let fixture: GaugeSequenceFixture = load_json("gauge_sequences.json");
    let mut failures = Vec::new();
    let tol = 1e-3_f32; // cumulative floating-point tolerance

    for (i, tc) in fixture.test_cases.iter().enumerate() {
        let prop = mode_to_gauge_property(&tc.mode);
        let label = format!(
            "[{i}] {}/{}/total={}/notes={}",
            tc.mode, tc.sequence_name, tc.total, tc.total_notes
        );

        // Create 9 gauges
        let mut gauges: Vec<Gauge> = (0..9)
            .map(|g| Gauge::new(prop.elements[g].clone(), tc.total, tc.total_notes))
            .collect();

        // Run sequence
        for (step_idx, step) in tc.sequence.iter().enumerate() {
            let rate = step.rate_x100 as f32 / 100.0;
            for gauge in &mut gauges {
                gauge.update(step.judge, rate);
            }

            // Compare all 9 gauge values after this step
            let expected = &tc.values_after_each_step[step_idx];
            for (g, (rust_val, java_val)) in gauges
                .iter()
                .map(|g| g.value())
                .zip(expected.iter())
                .enumerate()
            {
                if (rust_val - java_val).abs() > tol {
                    let gauge_name = GaugeType::ALL
                        .get(g)
                        .map(|gt| format!("{gt:?}"))
                        .unwrap_or_else(|| format!("{g}"));
                    failures.push(format!(
                        "{label} step[{step_idx}] gauge[{gauge_name}]: rust={rust_val} java={java_val} (diff={})",
                        (rust_val - java_val).abs()
                    ));
                }
            }
        }
    }

    if !failures.is_empty() {
        panic!(
            "Gauge sequence GM mismatch ({} failures out of {} cases):\n{}",
            failures.len(),
            fixture.test_cases.len(),
            failures
                .iter()
                .take(30)
                .map(|f| format!("  - {f}"))
                .collect::<Vec<_>>()
                .join("\n")
        );
    }

    println!(
        "Gauge sequence GM: all {} cases passed",
        fixture.test_cases.len()
    );
}
