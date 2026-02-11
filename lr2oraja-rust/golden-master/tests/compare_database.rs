// Golden master tests: compare Rust SongData::from_model() against Java SongData.setBMSModel()

use std::path::Path;

use bms_database::SongData;
use bms_model::{BmsDecoder, BmsonDecoder};
use golden_master::database_fixtures::{DatabaseFixture, SongDataFixture};

#[path = "support/random_seeds.rs"]
mod random_seeds;

fn fixtures_dir() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .leak()
}

fn test_bms_dir() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../test-bms")
        .leak()
}

fn load_database_fixture() -> DatabaseFixture {
    let path = fixtures_dir().join("database_song_data.json");
    assert!(
        path.exists(),
        "Database fixture not found: {}. Run `just golden-master-database-gen` first.",
        path.display()
    );
    let content = std::fs::read_to_string(&path).expect("Failed to read fixture");
    serde_json::from_str(&content).expect("Failed to parse fixture")
}

fn find_test_case<'a>(fixture: &'a DatabaseFixture, filename: &str) -> &'a SongDataFixture {
    fixture
        .test_cases
        .iter()
        .find(|tc| tc.filename == filename)
        .unwrap_or_else(|| panic!("Test case not found for {filename}"))
}

fn compare_song_data(rust: &SongData, java: &SongDataFixture) -> Vec<String> {
    let mut diffs = Vec::new();

    if rust.md5 != java.md5 {
        diffs.push(format!("md5: rust={:?} java={:?}", rust.md5, java.md5));
    }
    if rust.sha256 != java.sha256 {
        diffs.push(format!(
            "sha256: rust={:?} java={:?}",
            rust.sha256, java.sha256
        ));
    }
    if rust.title != java.title {
        diffs.push(format!(
            "title: rust={:?} java={:?}",
            rust.title, java.title
        ));
    }
    if rust.subtitle != java.subtitle {
        diffs.push(format!(
            "subtitle: rust={:?} java={:?}",
            rust.subtitle, java.subtitle
        ));
    }
    if rust.genre != java.genre {
        diffs.push(format!(
            "genre: rust={:?} java={:?}",
            rust.genre, java.genre
        ));
    }
    if rust.artist != java.artist {
        diffs.push(format!(
            "artist: rust={:?} java={:?}",
            rust.artist, java.artist
        ));
    }
    if rust.subartist != java.subartist {
        diffs.push(format!(
            "subartist: rust={:?} java={:?}",
            rust.subartist, java.subartist
        ));
    }
    if rust.banner != java.banner {
        diffs.push(format!(
            "banner: rust={:?} java={:?}",
            rust.banner, java.banner
        ));
    }
    if rust.stagefile != java.stagefile {
        diffs.push(format!(
            "stagefile: rust={:?} java={:?}",
            rust.stagefile, java.stagefile
        ));
    }
    if rust.backbmp != java.backbmp {
        diffs.push(format!(
            "backbmp: rust={:?} java={:?}",
            rust.backbmp, java.backbmp
        ));
    }
    if rust.preview != java.preview {
        diffs.push(format!(
            "preview: rust={:?} java={:?}",
            rust.preview, java.preview
        ));
    }
    if rust.level != java.level {
        diffs.push(format!("level: rust={} java={}", rust.level, java.level));
    }
    if rust.mode != java.mode {
        diffs.push(format!("mode: rust={} java={}", rust.mode, java.mode));
    }
    if rust.difficulty != java.difficulty {
        diffs.push(format!(
            "difficulty: rust={} java={}",
            rust.difficulty, java.difficulty
        ));
    }
    if rust.judge != java.judge {
        diffs.push(format!("judge: rust={} java={}", rust.judge, java.judge));
    }
    if rust.minbpm != java.minbpm {
        diffs.push(format!("minbpm: rust={} java={}", rust.minbpm, java.minbpm));
    }
    if rust.maxbpm != java.maxbpm {
        diffs.push(format!("maxbpm: rust={} java={}", rust.maxbpm, java.maxbpm));
    }
    // length: allow ±1ms tolerance (Java getLastTime vs Rust total_time_us/1000)
    if (rust.length - java.length).abs() > 1 {
        diffs.push(format!(
            "length: rust={} java={} (diff={})",
            rust.length,
            java.length,
            rust.length - java.length
        ));
    }
    if rust.notes != java.notes {
        diffs.push(format!("notes: rust={} java={}", rust.notes, java.notes));
    }
    if rust.feature != java.feature {
        diffs.push(format!(
            "feature: rust={:#010b} java={:#010b} (rust={} java={})",
            rust.feature, java.feature, rust.feature, java.feature
        ));
    }
    if rust.content != java.content {
        diffs.push(format!(
            "content: rust={:#010b} java={:#010b} (rust={} java={})",
            rust.content, java.content, rust.content, java.content
        ));
    }

    diffs
}

fn assert_song_data_matches(rust: &SongData, java: &SongDataFixture, filename: &str) {
    let diffs = compare_song_data(rust, java);
    if !diffs.is_empty() {
        panic!(
            "SongData mismatch for {} ({} differences):\n{}",
            filename,
            diffs.len(),
            diffs
                .iter()
                .map(|d| format!("  - {d}"))
                .collect::<Vec<_>>()
                .join("\n")
        );
    }
}

/// Run a single BMS golden master database test
fn run_database_test(bms_name: &str) {
    let fixture = load_database_fixture();
    let test_case = find_test_case(&fixture, bms_name);

    let bms_path = test_bms_dir().join(bms_name);
    assert!(
        bms_path.exists(),
        "BMS file not found: {}",
        bms_path.display()
    );

    let model = BmsDecoder::decode(&bms_path).expect("Failed to parse BMS");
    let song_data = SongData::from_model(&model, false);

    assert_song_data_matches(&song_data, test_case, bms_name);
}

/// Run a BMS test with fixed random selections
fn run_database_test_with_randoms(bms_name: &str, randoms: &[i32]) {
    let fixture = load_database_fixture();
    let test_case = find_test_case(&fixture, bms_name);

    let bms_path = test_bms_dir().join(bms_name);
    assert!(
        bms_path.exists(),
        "BMS file not found: {}",
        bms_path.display()
    );

    let model = BmsDecoder::decode_with_randoms(&bms_path, randoms).expect("Failed to parse BMS");
    let song_data = SongData::from_model(&model, false);

    assert_song_data_matches(&song_data, test_case, bms_name);
}

/// Run a bmson golden master database test
fn run_database_test_bmson(bmson_name: &str) {
    let fixture = load_database_fixture();
    let test_case = find_test_case(&fixture, bmson_name);

    let bmson_path = test_bms_dir().join(bmson_name);
    assert!(
        bmson_path.exists(),
        "bmson file not found: {}",
        bmson_path.display()
    );

    let model = BmsonDecoder::decode(&bmson_path).expect("Failed to parse bmson");
    let song_data = SongData::from_model(&model, false);

    assert_song_data_matches(&song_data, test_case, bmson_name);
}

// --- BMS tests ---

#[test]
fn database_minimal_7k() {
    run_database_test("minimal_7k.bms");
}

#[test]
fn database_5key() {
    run_database_test("5key.bms");
}

#[test]
fn database_14key_dp() {
    run_database_test("14key_dp.bms");
}

#[test]
fn database_9key_pms() {
    run_database_test("9key_pms.bms");
}

#[test]
fn database_9key_pms_pms() {
    run_database_test("9key_pms.pms");
}

#[test]
fn database_bpm_change() {
    run_database_test("bpm_change.bms");
}

#[test]
fn database_stop_sequence() {
    run_database_test("stop_sequence.bms");
}

#[test]
fn database_longnote_types() {
    run_database_test("longnote_types.bms");
}

#[test]
fn database_mine_notes() {
    run_database_test("mine_notes.bms");
}

#[test]
fn database_scratch_bss() {
    run_database_test("scratch_bss.bms");
}

#[test]
fn database_empty_measures() {
    run_database_test("empty_measures.bms");
}

#[test]
fn database_random_if() {
    let selected_randoms = random_seeds::load_selected_randoms(test_bms_dir(), "random_if.bms");
    run_database_test_with_randoms("random_if.bms", &selected_randoms);
}

#[test]
fn database_encoding_sjis() {
    run_database_test("encoding_sjis.bms");
}

#[test]
fn database_encoding_utf8() {
    run_database_test("encoding_utf8.bms");
}

#[test]
fn database_defexrank() {
    run_database_test("defexrank.bms");
}

// --- bmson tests ---

#[test]
fn database_bmson_minimal_7k() {
    run_database_test_bmson("bmson_minimal_7k.bmson");
}

#[test]
fn database_bmson_bpm_change() {
    run_database_test_bmson("bmson_bpm_change.bmson");
}

#[test]
fn database_bmson_longnote() {
    run_database_test_bmson("bmson_longnote.bmson");
}

#[test]
fn database_bmson_stop_sequence() {
    run_database_test_bmson("bmson_stop_sequence.bmson");
}

#[test]
fn database_bmson_mine_invisible() {
    run_database_test_bmson("bmson_mine_invisible.bmson");
}
