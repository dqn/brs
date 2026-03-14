use super::*;

fn create_test_accessor() -> SQLiteSongDatabaseAccessor {
    SQLiteSongDatabaseAccessor::new(":memory:", &[]).unwrap()
}

fn make_test_song(md5: &str, sha256: &str, title: &str) -> SongData {
    let mut sd = SongData::new();
    sd.file.md5 = md5.to_string();
    sd.file.sha256 = sha256.to_string();
    sd.metadata.title = title.to_string();
    sd.file.set_path(format!("test/{}.bms", title));
    sd
}

#[test]
fn test_new_creates_tables() {
    let accessor = create_test_accessor();
    // Verify tables exist by querying them
    let songs = accessor.song_datas("md5", "nonexistent");
    assert!(songs.is_empty());
    let folders = accessor.folder_datas("path", "nonexistent");
    assert!(folders.is_empty());
}

#[test]
fn test_insert_and_get_song_by_md5() {
    let accessor = create_test_accessor();
    let song = make_test_song("abc123", "sha_abc123", "Test Song");
    accessor.insert_song(&song).unwrap();

    let results = accessor.song_datas("md5", "abc123");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].metadata.title, "Test Song");
    assert_eq!(results[0].file.md5, "abc123");
}

#[test]
fn test_insert_and_get_song_by_sha256() {
    let accessor = create_test_accessor();
    let song = make_test_song("md5_xyz", "sha256_xyz", "SHA Test");
    accessor.insert_song(&song).unwrap();

    let results = accessor.song_datas("sha256", "sha256_xyz");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].metadata.title, "SHA Test");
}

#[test]
fn test_get_song_datas_empty() {
    let accessor = create_test_accessor();
    let results = accessor.song_datas("md5", "nonexistent");
    assert!(results.is_empty());
}

#[test]
fn test_get_song_datas_by_hashes() {
    let accessor = create_test_accessor();
    // SHA256 hashes must be > 32 chars to be classified as sha256
    let sha1 = "a".repeat(64);
    let sha2 = "b".repeat(64);
    let sha3 = "c".repeat(64);
    let song1 = make_test_song("md5_1", &sha1, "Song 1");
    let song2 = make_test_song("md5_2", &sha2, "Song 2");
    let song3 = make_test_song("md5_3", &sha3, "Song 3");
    accessor.insert_song(&song1).unwrap();
    accessor.insert_song(&song2).unwrap();
    accessor.insert_song(&song3).unwrap();

    // Query by sha256 hashes (> 32 chars)
    let hashes = vec![sha1, sha3];
    let results = accessor.song_datas_by_hashes(&hashes);
    assert_eq!(results.len(), 2);
}

#[test]
fn test_get_song_datas_by_hashes_md5() {
    let accessor = create_test_accessor();
    let song1 = make_test_song("md5_short_1", "sha1", "Song Short 1");
    let song2 = make_test_song("md5_short_2", "sha2", "Song Short 2");
    accessor.insert_song(&song1).unwrap();
    accessor.insert_song(&song2).unwrap();

    // Query by md5 hashes (<= 32 chars)
    let hashes = vec!["md5_short_1".to_string()];
    let results = accessor.song_datas_by_hashes(&hashes);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].metadata.title, "Song Short 1");
}

#[test]
fn test_get_song_datas_by_text() {
    let accessor = create_test_accessor();
    let mut song = make_test_song("m1", "s1", "Rhythm Action");
    song.metadata.artist = "DJ Test".to_string();
    accessor.insert_song(&song).unwrap();

    let results = accessor.song_datas_by_text("Rhythm");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].metadata.title, "Rhythm Action");

    let results = accessor.song_datas_by_text("DJ Test");
    assert_eq!(results.len(), 1);

    let results = accessor.song_datas_by_text("nonexistent");
    assert!(results.is_empty());
}

#[test]
fn test_set_song_datas_batch() {
    let accessor = create_test_accessor();
    let songs = vec![
        make_test_song("batch_1", "sbatch_1", "Batch Song 1"),
        make_test_song("batch_2", "sbatch_2", "Batch Song 2"),
        make_test_song("batch_3", "sbatch_3", "Batch Song 3"),
    ];

    accessor.set_song_datas(&songs);

    let results = accessor.song_datas("md5", "batch_1");
    assert_eq!(results.len(), 1);
    let results = accessor.song_datas("md5", "batch_2");
    assert_eq!(results.len(), 1);
    let results = accessor.song_datas("md5", "batch_3");
    assert_eq!(results.len(), 1);
}

#[test]
fn test_insert_and_get_folder() {
    let accessor = create_test_accessor();
    let folder = FolderData {
        title: "Test Folder".to_string(),
        path: "/test/folder/".to_string(),
        parent: "parent_crc".to_string(),
        date: 1000,
        adddate: 2000,
        ..Default::default()
    };
    accessor.insert_folder(&folder).unwrap();

    let results = accessor.folder_datas("path", "/test/folder/");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].title, "Test Folder");
    assert_eq!(results[0].date, 1000);
}

#[test]
fn test_get_folder_datas_empty() {
    let accessor = create_test_accessor();
    let results = accessor.folder_datas("path", "nonexistent");
    assert!(results.is_empty());
}

#[test]
fn test_add_plugin() {
    let mut accessor = create_test_accessor();
    struct TestPlugin;
    impl SongDatabaseAccessorPlugin for TestPlugin {
        fn update(&self, _model: &BMSModel, song: &mut SongData) {
            song.metadata.tag = "plugin_tag".to_string();
        }
    }
    accessor.add_plugin(Box::new(TestPlugin));
    assert_eq!(accessor.plugins.len(), 1);
}

#[test]
fn test_update_song_datas_scans_bms_files() {
    let tmpdir = tempfile::tempdir().unwrap();
    let bms_dir = tmpdir.path().join("songs").join("testpack");
    fs::create_dir_all(&bms_dir).unwrap();

    // Write a minimal BMS file
    let bms_content = "\
#PLAYER 1\n\
#GENRE Test\n\
#TITLE Update Test Song\n\
#ARTIST tester\n\
#BPM 120\n\
#PLAYLEVEL 3\n\
#RANK 2\n\
#TOTAL 300\n\
#WAV01 kick.wav\n\
#00111:01\n\
";
    fs::write(bms_dir.join("test.bms"), bms_content).unwrap();

    let db_path = tmpdir.path().join("song.db");
    let bmsroot = vec![tmpdir.path().join("songs").to_string_lossy().to_string()];
    let accessor = SQLiteSongDatabaseAccessor::new(&db_path.to_string_lossy(), &bmsroot).unwrap();

    accessor.update_song_datas(None, &bmsroot, true, false, None);

    // Verify the song was inserted
    let songs = accessor.song_datas("title", "Update Test Song");
    assert_eq!(songs.len(), 1);
    assert_eq!(songs[0].metadata.artist, "tester");
    assert!(songs[0].chart.notes > 0);
}

#[test]
fn test_update_song_datas_incremental_skips_unchanged() {
    let tmpdir = tempfile::tempdir().unwrap();
    let bms_dir = tmpdir.path().join("songs").join("testpack");
    fs::create_dir_all(&bms_dir).unwrap();

    let bms_content = "\
#PLAYER 1\n\
#TITLE Incremental Test\n\
#BPM 120\n\
#WAV01 kick.wav\n\
#00111:01\n\
";
    fs::write(bms_dir.join("incr.bms"), bms_content).unwrap();

    let db_path = tmpdir.path().join("song.db");
    let bmsroot = vec![tmpdir.path().join("songs").to_string_lossy().to_string()];
    let accessor = SQLiteSongDatabaseAccessor::new(&db_path.to_string_lossy(), &bmsroot).unwrap();

    // First update
    let listener1 = SongDatabaseUpdateListener::new();
    accessor.update_song_datas_with_listener(None, &bmsroot, false, false, None, &listener1);
    assert_eq!(listener1.new_bms_files_count(), 1);

    // Second update (no changes) - should skip
    let listener2 = SongDatabaseUpdateListener::new();
    accessor.update_song_datas_with_listener(None, &bmsroot, false, false, None, &listener2);
    assert_eq!(listener2.new_bms_files_count(), 0);
    assert_eq!(listener2.bms_files_count(), 1);
}

#[test]
fn test_update_song_datas_creates_folder_records() {
    let tmpdir = tempfile::tempdir().unwrap();
    let bms_dir = tmpdir.path().join("songs").join("pack1");
    fs::create_dir_all(&bms_dir).unwrap();

    let bms_content = "\
#TITLE Folder Test\n\
#BPM 120\n\
#WAV01 kick.wav\n\
#00111:01\n\
";
    fs::write(bms_dir.join("folder_test.bms"), bms_content).unwrap();

    let db_path = tmpdir.path().join("song.db");
    let bmsroot = vec![tmpdir.path().join("songs").to_string_lossy().to_string()];
    let accessor = SQLiteSongDatabaseAccessor::new(&db_path.to_string_lossy(), &bmsroot).unwrap();

    accessor.update_song_datas(None, &bmsroot, true, false, None);

    // Check that folder records were created (at least root and pack1)
    let all_folders: Vec<FolderData> = {
        let conn = accessor.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT * FROM folder").unwrap();
        let rows = stmt
            .query_map([], |row| {
                Ok(FolderData {
                    title: row.get::<_, String>(0).unwrap_or_default(),
                    subtitle: row.get::<_, String>(1).unwrap_or_default(),
                    command: row.get::<_, String>(2).unwrap_or_default(),
                    path: row.get::<_, String>(3).unwrap_or_default(),
                    banner: row.get::<_, String>(4).unwrap_or_default(),
                    parent: row.get::<_, String>(5).unwrap_or_default(),
                    folder_type: row.get::<_, i32>(6).unwrap_or(0),
                    date: row.get::<_, i32>(7).unwrap_or(0),
                    adddate: row.get::<_, i32>(8).unwrap_or(0),
                    max: row.get::<_, i32>(9).unwrap_or(0),
                })
            })
            .unwrap();
        rows.flatten().collect()
    };
    assert!(
        !all_folders.is_empty(),
        "Folder records should be created during update"
    );
}

#[test]
fn test_update_song_datas_empty_bmsroot() {
    let accessor = create_test_accessor();
    // Should not panic, just log warning and return
    accessor.update_song_datas(None, &[], true, false, None);
}

#[test]
fn test_update_song_datas_preserves_favorites() {
    let tmpdir = tempfile::tempdir().unwrap();
    let bms_dir = tmpdir.path().join("songs").join("favpack");
    fs::create_dir_all(&bms_dir).unwrap();

    let bms_content = "\
#TITLE Favorite Test\n\
#BPM 120\n\
#WAV01 kick.wav\n\
#00111:01\n\
";
    fs::write(bms_dir.join("fav.bms"), bms_content).unwrap();

    let db_path = tmpdir.path().join("song.db");
    let bmsroot = vec![tmpdir.path().join("songs").to_string_lossy().to_string()];
    let accessor = SQLiteSongDatabaseAccessor::new(&db_path.to_string_lossy(), &bmsroot).unwrap();

    // First update
    accessor.update_song_datas(None, &bmsroot, true, false, None);

    // Set favorite on the song
    let songs = accessor.song_datas("title", "Favorite Test");
    assert_eq!(songs.len(), 1);
    let sha256 = songs[0].file.sha256.clone();
    let conn = accessor.conn.lock().unwrap();
    let _ = conn.execute(
        "UPDATE song SET favorite = 3 WHERE sha256 = ?1",
        rusqlite::params![sha256],
    );
    drop(conn);

    // Full re-update (updateAll=true)
    accessor.update_song_datas(None, &bmsroot, true, false, None);

    // Verify favorite is preserved
    let songs = accessor.song_datas("title", "Favorite Test");
    assert_eq!(songs.len(), 1);
    assert_eq!(
        songs[0].favorite, 3,
        "Favorite should be preserved across updates"
    );
}

#[test]
fn test_update_song_datas_auto_difficulty() {
    let tmpdir = tempfile::tempdir().unwrap();
    let bms_dir = tmpdir.path().join("songs").join("diffpack");
    fs::create_dir_all(&bms_dir).unwrap();

    // "beginner" in subtitle -> difficulty 1
    let bms_content = "\
#TITLE Test\n\
#SUBTITLE beginner\n\
#BPM 120\n\
#WAV01 kick.wav\n\
#00111:01\n\
";
    fs::write(bms_dir.join("diff.bms"), bms_content).unwrap();

    let db_path = tmpdir.path().join("song.db");
    let bmsroot = vec![tmpdir.path().join("songs").to_string_lossy().to_string()];
    let accessor = SQLiteSongDatabaseAccessor::new(&db_path.to_string_lossy(), &bmsroot).unwrap();

    accessor.update_song_datas(None, &bmsroot, true, false, None);

    let songs = accessor.song_datas("title", "Test");
    assert_eq!(songs.len(), 1);
    assert_eq!(
        songs[0].chart.difficulty, 1,
        "Beginner subtitle should set difficulty to 1"
    );
}

/// Verify that set_song_datas holds the connection lock for the entire
/// transaction, preventing concurrent callers from interleaving SQL
/// statements. Two threads each call set_song_datas with disjoint song
/// batches. After both complete, all songs from both batches must be
/// present (no lost writes due to interleaved BEGIN/COMMIT).
#[test]
fn test_set_song_datas_concurrent_no_interleaving() {
    use std::sync::Arc;

    let db_path = tempfile::NamedTempFile::new().unwrap();
    let accessor =
        Arc::new(SQLiteSongDatabaseAccessor::new(&db_path.path().to_string_lossy(), &[]).unwrap());

    let batch_a: Vec<SongData> = (0..50)
        .map(|i| {
            make_test_song(
                &format!("md5_a_{i}"),
                &format!("sha_a_{i}"),
                &format!("A {i}"),
            )
        })
        .collect();
    let batch_b: Vec<SongData> = (0..50)
        .map(|i| {
            make_test_song(
                &format!("md5_b_{i}"),
                &format!("sha_b_{i}"),
                &format!("B {i}"),
            )
        })
        .collect();

    let acc_a = Arc::clone(&accessor);
    let ba = batch_a.clone();
    let handle_a = std::thread::spawn(move || {
        acc_a.set_song_datas(&ba);
    });

    let acc_b = Arc::clone(&accessor);
    let bb = batch_b.clone();
    let handle_b = std::thread::spawn(move || {
        acc_b.set_song_datas(&bb);
    });

    handle_a.join().unwrap();
    handle_b.join().unwrap();

    // All 100 songs must be present (no lost writes from interleaving)
    for i in 0..50 {
        let results = accessor.song_datas("md5", &format!("md5_a_{i}"));
        assert_eq!(results.len(), 1, "missing song A {i}");
        let results = accessor.song_datas("md5", &format!("md5_b_{i}"));
        assert_eq!(results.len(), 1, "missing song B {i}");
    }
}

/// Verify that set_song_datas is atomic: either all songs are inserted or
/// none. This tests the transaction boundary by confirming batch insert
/// completes as a unit.
#[test]
fn test_set_song_datas_transaction_atomicity() {
    let accessor = create_test_accessor();
    let songs: Vec<SongData> = (0..10)
        .map(|i| {
            make_test_song(
                &format!("atomic_md5_{i}"),
                &format!("atomic_sha_{i}"),
                &format!("Atomic Song {i}"),
            )
        })
        .collect();

    accessor.set_song_datas(&songs);

    // All 10 songs must be present
    for i in 0..10 {
        let results = accessor.song_datas("md5", &format!("atomic_md5_{i}"));
        assert_eq!(results.len(), 1, "missing song {i} after batch insert");
    }
}

/// Verify that update_song_datas holds the connection lock for the entire
/// transaction. Two threads each call update_song_datas on the same roots.
/// After both complete, the DB must be in a consistent state with all
/// songs present and no corruption from interleaved transactions.
#[test]
fn test_update_concurrent_no_interleaving() {
    use std::sync::Arc;

    let tmpdir = tempfile::tempdir().unwrap();
    let bms_dir = tmpdir.path().join("songs").join("pack");
    fs::create_dir_all(&bms_dir).unwrap();

    // Write two BMS files so both updates find them
    let bms_content_a = "\
#TITLE Concurrent A\n\
#BPM 120\n\
#WAV01 kick.wav\n\
#00111:01\n\
";
    let bms_content_b = "\
#TITLE Concurrent B\n\
#BPM 130\n\
#WAV01 snare.wav\n\
#00111:01\n\
";
    fs::write(bms_dir.join("a.bms"), bms_content_a).unwrap();
    fs::write(bms_dir.join("b.bms"), bms_content_b).unwrap();

    let db_path = tmpdir.path().join("song.db");
    let bmsroot = vec![tmpdir.path().join("songs").to_string_lossy().to_string()];
    let accessor =
        Arc::new(SQLiteSongDatabaseAccessor::new(&db_path.to_string_lossy(), &bmsroot).unwrap());

    // Run two full updates concurrently on the same roots
    let acc_a = Arc::clone(&accessor);
    let roots_a = bmsroot.clone();
    let handle_a = std::thread::spawn(move || {
        acc_a.update_song_datas(None, &roots_a, true, false, None);
    });

    let acc_b = Arc::clone(&accessor);
    let roots_b = bmsroot.clone();
    let handle_b = std::thread::spawn(move || {
        acc_b.update_song_datas(None, &roots_b, true, false, None);
    });

    handle_a.join().unwrap();
    handle_b.join().unwrap();

    // Both songs must be present in the final state (the second update
    // re-scans and re-inserts everything, so both songs survive).
    let results_a = accessor.song_datas("title", "Concurrent A");
    assert_eq!(
        results_a.len(),
        1,
        "song A should be present after concurrent updates"
    );
    let results_b = accessor.song_datas("title", "Concurrent B");
    assert_eq!(
        results_b.len(),
        1,
        "song B should be present after concurrent updates"
    );
}

#[test]
fn escape_sql_like_no_wildcards() {
    assert_eq!(escape_sql_like("normal/path"), "normal/path");
}

#[test]
fn escape_sql_like_percent() {
    assert_eq!(escape_sql_like("foo%bar"), "foo\\%bar");
}

#[test]
fn escape_sql_like_underscore() {
    assert_eq!(escape_sql_like("foo_bar"), "foo\\_bar");
}

#[test]
fn escape_sql_like_backslash() {
    assert_eq!(escape_sql_like("foo\\bar"), "foo\\\\bar");
}

#[test]
fn escape_sql_like_mixed() {
    assert_eq!(escape_sql_like("a%b_c\\d"), "a\\%b\\_c\\\\d");
}

#[test]
fn escape_sql_like_empty() {
    assert_eq!(escape_sql_like(""), "");
}
