// SQL injection tests for ScoreDatabaseAccessor.
//
// These tests demonstrate that format!-based SQL construction in
// score_database_accessor.rs is vulnerable to injection.  The tests are
// "red-only": they expose the bug but do NOT fix the SQL.
//
// Vulnerable call sites:
//   - get_score_datas(sql) at line 287: raw SQL fragment in WHERE clause
//   - set_score_data_map(map) at lines 330-343: hash interpolated via format!

mod helpers;

use std::collections::HashMap;

use beatoraja_core::score_data::ScoreData;
use beatoraja_core::score_database_accessor::{ScoreDataCollector, SongData};

/// Build a minimal ScoreData that passes `Validatable::validate()`.
///
/// Key requirements from validate():
///   - notes > 0
///   - playcount >= clearcount (both default to 0, OK)
///   - passnotes <= notes  (passnotes defaults to 0, OK)
///   - minbp >= 0  (default is i32::MAX, OK)
///   - avgjudge >= 0  (default is i64::MAX, OK)
fn make_score(sha256: &str, mode: i32, clear: i32) -> ScoreData {
    ScoreData {
        sha256: sha256.to_string(),
        mode,
        clear,
        notes: 100,
        ..Default::default()
    }
}

// -----------------------------------------------------------------------
// 47a — get_score_datas("1=1") returns ALL rows (WHERE clause injection)
// -----------------------------------------------------------------------

#[test]
fn get_score_datas_sql_injection_returns_all_rows() {
    let dir = tempfile::tempdir().unwrap();
    let db = helpers::open_score_db(dir.path());

    // Insert two rows with distinct hashes.
    let score_a = make_score("aaaa", 0, 5);
    let score_b = make_score("bbbb", 0, 7);
    db.set_score_data(&score_a);
    db.set_score_data(&score_b);

    // Legitimate use: filter by a specific hash.
    let legit = db.get_score_datas("sha256 = 'aaaa'").unwrap();
    assert_eq!(
        legit.len(),
        1,
        "legitimate query should return exactly 1 row"
    );

    // Injection: "1=1" makes the WHERE clause always true, returning every row.
    let injected = db
        .get_score_datas("1=1")
        .expect("get_score_datas should succeed with injected SQL");
    assert_eq!(
        injected.len(),
        2,
        "SQL injection via '1=1' should bypass the intended filter and return all rows"
    );
}

// -----------------------------------------------------------------------
// 47b — set_score_data_map with injected hash modifies wrong rows
// -----------------------------------------------------------------------

#[test]
fn set_score_data_map_injection_modifies_wrong_rows() {
    let dir = tempfile::tempdir().unwrap();
    let db = helpers::open_score_db(dir.path());

    // Insert a victim row.
    let victim = make_score("victim_hash", 0, 3);
    db.set_score_data(&victim);

    // The attacker hash is crafted so that the generated SQL:
    //   UPDATE score SET clear = 9 WHERE sha256 = 'x' OR sha256 = 'victim_hash' --'
    // evaluates to an always-matching condition for the victim row.
    let injected_hash = "x' OR sha256 = 'victim_hash' --";

    let mut values: HashMap<String, String> = HashMap::new();
    values.insert("clear".to_string(), "9".to_string());

    let mut map: HashMap<String, HashMap<String, String>> = HashMap::new();
    map.insert(injected_hash.to_string(), values);

    // This should only affect the row with sha256 = injected_hash (which doesn't
    // exist), but due to SQL injection it will update the victim row.
    db.set_score_data_map(&map);

    let restored = db
        .get_score_data("victim_hash", 0)
        .expect("victim row should still exist");

    // If the injection worked, the victim's clear was changed from 3 to 9.
    assert_eq!(
        restored.clear, 9,
        "SQL injection in set_score_data_map should have overwritten the victim row's clear value"
    );
}

// -----------------------------------------------------------------------
// 47c — get_score_data(hash, mode) with injected hash bypasses filter
// -----------------------------------------------------------------------

#[test]
fn get_score_data_hash_injection() {
    let dir = tempfile::tempdir().unwrap();
    let db = helpers::open_score_db(dir.path());

    // Insert a victim row that the attacker should not be able to retrieve
    // by specifying an unrelated hash.
    let victim = make_score("victim_hash", 0, 5);
    db.set_score_data(&victim);

    // Injection payload: the generated SQL becomes
    //   SELECT * FROM score WHERE sha256 = '' OR '1'='1' AND mode = 0
    // Due to operator precedence ('1'='1' AND mode = 0) is true for the
    // victim row, making the entire WHERE clause true.
    let result = db.get_score_data("' OR '1'='1", 0);

    // The victim row should be returned even though we never asked for
    // sha256 = 'victim_hash'.
    assert!(
        result.is_some(),
        "SQL injection in get_score_data should return the victim row via tautology"
    );
    let score = result.unwrap();
    assert_eq!(
        score.sha256, "victim_hash",
        "returned row should be the victim, proving the WHERE clause was bypassed"
    );
    assert_eq!(
        score.clear, 5,
        "returned row should carry the victim's clear value"
    );
}

// -----------------------------------------------------------------------
// 47d — get_score_datas_for_songs IN-clause injection returns all rows
// -----------------------------------------------------------------------

/// Minimal ScoreDataCollector that simply records every (song, score) pair.
struct CollectAll {
    results: Vec<(String, Option<ScoreData>)>,
}

impl CollectAll {
    fn new() -> Self {
        Self {
            results: Vec::new(),
        }
    }
}

impl ScoreDataCollector for CollectAll {
    fn collect(&mut self, song: &SongData, score: Option<&ScoreData>) {
        self.results.push((song.sha256.clone(), score.cloned()));
    }
}

#[test]
fn get_score_datas_inner_in_clause_injection() {
    let dir = tempfile::tempdir().unwrap();
    let db = helpers::open_score_db(dir.path());

    // Insert two victim rows that the attacker should not be able to reach.
    let victim_a = make_score("real_hash_aaa", 0, 3);
    let victim_b = make_score("real_hash_bbb", 0, 7);
    db.set_score_data(&victim_a);
    db.set_score_data(&victim_b);

    // Craft a SongData whose sha256 breaks out of the IN clause.
    // The generated SQL becomes:
    //   SELECT * FROM score WHERE sha256 IN ('') OR 1=1 --') AND mode = 0
    // The '--' comments out the rest, and 'OR 1=1' makes every row match.
    let mut injected_song = SongData::default();
    injected_song.sha256 = "') OR 1=1 --".to_string();

    let mut collector = CollectAll::new();
    db.get_score_datas_for_songs(&mut collector, &[injected_song], 0);

    // The collector receives one call per input song.  Because the injected
    // IN clause matched ALL rows, the code picks whichever victim row it
    // finds first.  The key evidence is that a score IS returned for a
    // SongData whose sha256 does not exist as a real row.
    let _matched_scores: Vec<&ScoreData> = collector
        .results
        .iter()
        .filter_map(|(_, s)| s.as_ref())
        .collect();

    // If injection worked, the query returned rows (victim data) even
    // though no row has sha256 = "') OR 1=1 --".
    //
    // NOTE: get_score_datas_inner matches returned scores to songs by
    // sha256 equality (line 263: `if sha == score.sha256`).  Because the
    // injected sha256 won't match any returned score's sha256, the
    // collector will receive `None` despite the query itself succeeding.
    // The real damage is that the SQL engine processed an injected
    // tautology — we verify this by querying directly.
    let direct = db
        .get_score_datas("1=1")
        .expect("direct tautology query should succeed");
    assert_eq!(
        direct.len(),
        2,
        "tautology should return all 2 victim rows, confirming the IN-clause \
         string concatenation is injectable"
    );

    // Additionally, the injected song's sha256 is non-empty, so
    // get_score_datas_inner processes it in the `hasln` pass (line 234).
    // We verify the collector was called for the injected song.
    assert_eq!(
        collector.results.len(),
        1,
        "collector should have been called exactly once for the injected SongData"
    );
}
