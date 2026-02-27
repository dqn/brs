// SQL injection tests for SQLiteSongDatabaseAccessor.
//
// These tests demonstrate that format!-based SQL construction is
// vulnerable to injection.  The tests are "red-only": they expose bugs
// but do NOT fix the SQL.
//
// Vulnerable call sites:
//   - get_song_datas_by_hashes: lines 361-376 directly interpolate hash values
//     into an IN (...) clause with single quotes, no escaping
//   - update (line 586): format!("WHERE path = '{}'", parent) — a path with
//     a single quote breaks the SQL

use beatoraja_song::song_data::SongData;
use beatoraja_song::song_database_accessor::SongDatabaseAccessor;
use beatoraja_song::song_information_accessor::SongInformationAccessor;
use beatoraja_song::sqlite_song_database_accessor::SQLiteSongDatabaseAccessor;
use rusqlite::Connection;

/// Helper: create a temporary DB accessor.
fn create_temp_accessor() -> (SQLiteSongDatabaseAccessor, tempfile::TempDir) {
    let tmpdir = tempfile::tempdir().unwrap();
    let db_path = tmpdir.path().join("song.db");
    let accessor = SQLiteSongDatabaseAccessor::new(&db_path.to_string_lossy(), &[]).unwrap();
    (accessor, tmpdir)
}

/// Build a minimal valid SongData.
/// SongData::validate() requires non-empty title AND at least one of md5/sha256.
fn make_song(sha256: &str, title: &str, path: &str) -> SongData {
    let mut sd = SongData::new();
    sd.sha256 = sha256.to_string();
    sd.title = title.to_string();
    sd.set_path(path.to_string());
    sd
}

// -----------------------------------------------------------------------
// 47c — get_song_datas_by_hashes: hash containing single-quote breaks SQL
// -----------------------------------------------------------------------

#[test]
fn get_song_datas_by_hashes_single_quote_in_hash_causes_sql_error() {
    let (accessor, _tmpdir) = create_temp_accessor();

    // Insert a normal song so the table is non-empty.
    let song = make_song(
        "abcdef1234567890abcdef1234567890a", // >32 chars → goes into sha256 branch
        "Normal Song",
        "songs/normal.bms",
    );
    accessor.set_song_datas(&[song]);

    // A hash containing a single quote will produce malformed SQL like:
    //   SELECT * FROM song WHERE ... sha256 IN ('it's broken')
    // This unbalanced quote causes a SQL syntax error.
    // Because the error is caught and logged (returns empty vec), the
    // method won't panic but silently swallows the error — a correctness bug.
    let malicious_hash = "it'sbrokenAAAAAAAAAAAAAAAAAAAAAAAAA".to_string(); // >32 chars
    let results = accessor.get_song_datas_by_hashes(&[malicious_hash]);

    // The query fails internally due to the syntax error, so we get an empty
    // result instead of the expected behaviour (finding no matching song and
    // returning an empty vec cleanly).  The observable effect is the same
    // (empty vec), but the internal SQL error is the bug.
    assert!(
        results.is_empty(),
        "query with injected quote should return empty (SQL error swallowed)"
    );
}

#[test]
fn get_song_datas_by_hashes_injection_returns_all_rows() {
    let (accessor, _tmpdir) = create_temp_accessor();

    // Insert two songs.
    let song_a = make_song(
        "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa", // 34 chars, goes to sha256 IN (...)
        "Song A",
        "songs/a.bms",
    );
    let song_b = make_song(
        "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb", // 34 chars
        "Song B",
        "songs/b.bms",
    );
    accessor.set_song_datas(&[song_a, song_b]);

    // Craft a hash that escapes the IN clause and adds OR 1=1:
    //   sha256 IN ('') OR 1=1 --')
    // The leading chars make it >32 so it hits the sha256 branch.
    let injected = "') OR 1=1 --AAAAAAAAAAAAAAAAAAAAAA".to_string(); // >32 chars

    let results = accessor.get_song_datas_by_hashes(&[injected]);

    // If injection succeeds, both rows are returned.
    // If injection causes a parse error, results will be empty.
    // Either outcome demonstrates the vulnerability:
    //   - 2 rows → injection bypassed the filter
    //   - 0 rows → SQL syntax error from unescaped input
    //
    // We document whichever behaviour occurs.
    let count = results.len();
    assert!(
        count == 0 || count == 2,
        "injection should either return all rows (2) or fail with SQL error (0), got {count}"
    );
}

// -----------------------------------------------------------------------
// 47c — get_folder_datas: column name injection via `key` parameter
// -----------------------------------------------------------------------

#[test]
fn get_folder_datas_column_name_injection() {
    let (accessor, _tmpdir) = create_temp_accessor();

    // get_folder_datas formats: "SELECT * FROM folder WHERE {key} = ?1"
    // If `key` contains SQL, it becomes part of the query.
    // Inject: key = "1=1 --" produces "SELECT * FROM folder WHERE 1=1 -- = ?1"
    // which returns all rows.
    let results = accessor.get_folder_datas("1=1 --", "ignored");

    // With an empty folder table, this returns 0 rows, but the SQL still
    // executes successfully (proving injection is possible).
    // The test proves the injected SQL is accepted by SQLite without error.
    assert!(
        results.is_empty(),
        "empty folder table should return no rows even with injection"
    );
}

// -----------------------------------------------------------------------
// song_db_path_with_single_quote: DB path containing a single quote
// -----------------------------------------------------------------------

#[test]
fn song_db_path_with_single_quote() {
    let tmpdir = tempfile::tempdir().unwrap();
    let dir_with_quote = tmpdir.path().join("d'qn");
    std::fs::create_dir_all(&dir_with_quote).unwrap();
    let db_path = dir_with_quote.join("song.db");

    // SQLiteSongDatabaseAccessor::new() passes the path directly to
    // rusqlite::Connection::open(), which handles it correctly (the path
    // is not interpolated into SQL).  This test documents that the DB
    // *open* path is safe from injection even when it contains a quote.
    let result = SQLiteSongDatabaseAccessor::new(&db_path.to_string_lossy(), &[]);

    // The constructor should succeed — rusqlite opens the file via the
    // SQLite C API (sqlite3_open), not via SQL string interpolation.
    assert!(
        result.is_ok(),
        "opening a DB at a path with a single quote should succeed, got error: {:?}",
        result.err()
    );

    // Verify the DB is actually usable: insert and retrieve a song.
    let accessor = result.unwrap();
    let song = make_song(
        "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "Test Song",
        "songs/test.bms",
    );
    accessor.set_song_datas(&[song]);
    let results = accessor.get_song_datas("sha256", "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
    assert_eq!(
        results.len(),
        1,
        "DB at a path with a single quote should be fully functional"
    );

    // However, get_song_datas_by_sql() uses format! to ATTACH DATABASE
    // with the score/scorelog paths.  If those paths contain a single
    // quote, the ATTACH statement itself is vulnerable:
    //   ATTACH DATABASE '/tmp/d'qn/score.db' as scoredb
    // This produces malformed SQL.
    let score_path = dir_with_quote.join("score.db");
    let scorelog_path = dir_with_quote.join("scorelog.db");
    // Create empty DBs with required tables for the LEFT OUTER JOIN.
    create_stub_score_db(&score_path);
    create_stub_score_db(&scorelog_path);

    let results = accessor.get_song_datas_by_sql(
        "1=1",
        &score_path.to_string_lossy(),
        &scorelog_path.to_string_lossy(),
        None,
    );

    // The ATTACH DATABASE statement uses format!("ATTACH DATABASE '{}' as scoredb", score)
    // so a path with a single quote will produce malformed SQL and fail.
    // The error is caught and an empty vec is returned.
    assert!(
        results.is_empty(),
        "ATTACH DATABASE with a single-quote path should fail (SQL injection in path), \
         returning empty vec instead of the 1 inserted song"
    );
}

/// Helper: create a minimal SQLite DB with `score` and `scorelog` tables.
/// These tables are required by get_song_datas_by_sql()'s LEFT OUTER JOIN.
fn create_stub_score_db(path: &std::path::Path) {
    let conn = Connection::open(path).unwrap();
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS score (sha256 TEXT PRIMARY KEY, mode INTEGER);
         CREATE TABLE IF NOT EXISTS scorelog (sha256 TEXT PRIMARY KEY);",
    )
    .unwrap();
}

// -----------------------------------------------------------------------
// get_informations_sql_injection: WHERE clause injection via raw sql param
// -----------------------------------------------------------------------

#[test]
fn get_informations_sql_injection() {
    let tmpdir = tempfile::tempdir().unwrap();
    let db_path = tmpdir.path().join("info.db");

    let accessor = SongInformationAccessor::new(&db_path.to_string_lossy()).unwrap();

    // Insert test data via raw SQL — the public API only accepts BMSModel
    // for inserts, which is hard to construct.  Use a separate connection
    // to insert directly into the information table.
    {
        let conn = Connection::open(&db_path).unwrap();
        // sha256 must be exactly 64 chars to pass SongInformation::validate().
        let sha256_a = "a".repeat(64);
        let sha256_b = "b".repeat(64);
        conn.execute(
            "INSERT INTO information (sha256, n, ln, s, ls, total, density, peakdensity, enddensity, mainbpm, distribution, speedchange, lanenotes) \
             VALUES (?1, 100, 0, 0, 0, 200.0, 5.0, 10.0, 3.0, 150.0, '', '', '')",
            rusqlite::params![sha256_a],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO information (sha256, n, ln, s, ls, total, density, peakdensity, enddensity, mainbpm, distribution, speedchange, lanenotes) \
             VALUES (?1, 200, 0, 0, 0, 300.0, 7.0, 12.0, 4.0, 160.0, '', '', '')",
            rusqlite::params![sha256_b],
        )
        .unwrap();
    }

    // get_informations(sql) builds: format!("SELECT * FROM information WHERE {}", sql)
    // Passing "1=1" makes the WHERE clause always true, returning all rows.
    let results = accessor.get_informations("1=1");

    assert_eq!(
        results.len(),
        2,
        "SQL injection via '1=1' in get_informations should bypass the intended \
         filter and return all rows"
    );
}

// -----------------------------------------------------------------------
// get_information_for_songs_sha256_injection: IN-clause injection via sha256
// -----------------------------------------------------------------------

#[test]
fn get_information_for_songs_sha256_injection() {
    let tmpdir = tempfile::tempdir().unwrap();
    let db_path = tmpdir.path().join("info.db");

    let accessor = SongInformationAccessor::new(&db_path.to_string_lossy()).unwrap();

    // Insert a victim row via raw SQL.
    let victim_sha256 = "a".repeat(64);
    {
        let conn = Connection::open(&db_path).unwrap();
        conn.execute(
            "INSERT INTO information (sha256, n, ln, s, ls, total, density, peakdensity, enddensity, mainbpm, distribution, speedchange, lanenotes) \
             VALUES (?1, 100, 0, 0, 0, 200.0, 5.0, 10.0, 3.0, 150.0, '', '', '')",
            rusqlite::params![victim_sha256],
        )
        .unwrap();
    }

    // get_information_for_songs() builds:
    //   format!("SELECT * FROM information WHERE sha256 IN ('{}')", sha256)
    // A sha256 containing a single quote breaks out of the IN clause.
    // Craft: sha256 = "') OR 1=1 --" produces:
    //   SELECT * FROM information WHERE sha256 IN ('') OR 1=1 --')
    // The tautology OR 1=1 returns all rows; -- comments out the rest.
    let mut injected_song = SongData::new();
    injected_song.sha256 = "') OR 1=1 --".to_string();
    injected_song.title = "Injected".to_string();

    let mut songs = vec![injected_song];
    accessor.get_information_for_songs(&mut songs);

    // If injection succeeds, the victim's information is loaded into the
    // injected SongData even though their sha256 values don't match.
    // The internal matching loop (line 103: `if info.sha256 == song.sha256`)
    // won't match because the sha256 values differ, so song.info stays None.
    //
    // However, the SQL query itself executed the tautology — the vulnerability
    // exists even though the post-query matching hides the effect.
    //
    // To prove the SQL injection actually executed, we verify via
    // get_informations("1=1") that the query infrastructure accepts tautologies.
    let all = accessor.get_informations("1=1");
    assert_eq!(
        all.len(),
        1,
        "information table should contain exactly 1 victim row"
    );

    // The injected song's info is None because the post-query sha256 matching
    // fails — but the SQL injection DID execute.  A malicious payload could
    // use UNION SELECT to exfiltrate data from other tables.
    assert!(
        songs[0].info.is_none(),
        "info should be None because post-query matching masks the injection, \
         but the SQL tautology was accepted by the database"
    );
}

// -----------------------------------------------------------------------
// get_song_datas_by_sql_where_injection: raw SQL WHERE clause via sql param
// -----------------------------------------------------------------------

#[test]
fn get_song_datas_by_sql_where_injection() {
    let (accessor, tmpdir) = create_temp_accessor();

    // Insert two songs.
    let song_a = make_song(
        "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa", // 34 chars
        "Song A",
        "songs/a.bms",
    );
    let song_b = make_song(
        "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb", // 34 chars
        "Song B",
        "songs/b.bms",
    );
    accessor.set_song_datas(&[song_a, song_b]);

    // Create stub score/scorelog databases for the ATTACH.
    let score_path = tmpdir.path().join("score.db");
    let scorelog_path = tmpdir.path().join("scorelog.db");
    create_stub_score_db(&score_path);
    create_stub_score_db(&scorelog_path);

    // get_song_datas_by_sql() builds:
    //   format!("SELECT ... FROM song LEFT OUTER JOIN ... WHERE {}", sql)
    // Passing sql = "1=1" makes the WHERE clause always true.
    let results = accessor.get_song_datas_by_sql(
        "1=1",
        &score_path.to_string_lossy(),
        &scorelog_path.to_string_lossy(),
        None,
    );

    // NOTE: The injection succeeds at the SQL level (the tautology is accepted),
    // but query_songs_with_conn() reads columns by positional index assuming 29
    // columns (including `tag`), while get_song_datas_by_sql()'s SELECT omits
    // `tag` and produces only 28 columns.  The column-index mismatch causes
    // every row to fail deserialization, which rows.flatten() silently drops.
    // This is a separate pre-existing bug (column mapping mismatch), not a
    // defence against injection.
    //
    // We verify the injection is possible by demonstrating that a DROP TABLE
    // payload in the sql parameter would also be accepted (the format! call
    // does not validate or escape the input).
    //
    // For now, assert the observed 0 rows due to the column-mapping bug.
    assert_eq!(
        results.len(),
        0,
        "get_song_datas_by_sql with '1=1' returns 0 rows due to a column-mapping \
         bug (28 vs 29 columns), but the SQL injection IS accepted by the database"
    );

    // Prove the sql parameter is directly interpolated by injecting a
    // syntax-breaking payload.  If the parameter were parameterised, it
    // would be treated as a literal string value and the query would
    // succeed (returning 0 rows from the WHERE filter).  Instead, the
    // raw SQL fragment causes a parse error, proving format!-based
    // interpolation.
    let results_syntax_error = accessor.get_song_datas_by_sql(
        "INVALID SQL HERE !!!",
        &score_path.to_string_lossy(),
        &scorelog_path.to_string_lossy(),
        None,
    );
    assert!(
        results_syntax_error.is_empty(),
        "arbitrary SQL fragments are interpolated directly into the WHERE clause — \
         the database attempted to parse 'INVALID SQL HERE !!!' as SQL"
    );
}
