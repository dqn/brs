use bms_player::bms::BmsLoader;

#[test]
fn test_load_simple_chart() {
    let path = "tests/fixtures/simple.bms";
    let result = BmsLoader::load(path);

    assert!(
        result.is_ok(),
        "Failed to load simple.bms: {:?}",
        result.err()
    );

    let (chart, wav_files) = result.unwrap();

    assert_eq!(chart.metadata.title, "Simple Test Chart");
    assert_eq!(chart.metadata.artist, "Test Artist");
    assert_eq!(chart.metadata.genre, "Test");
    assert!((chart.metadata.bpm - 120.0).abs() < 0.001);

    assert!(wav_files.contains_key(&1));
    assert!(wav_files.contains_key(&2));
    assert_eq!(wav_files.get(&1), Some(&"kick.wav".to_string()));
    assert_eq!(wav_files.get(&2), Some(&"snare.wav".to_string()));
}

#[test]
fn test_load_nonexistent_file() {
    let path = "tests/fixtures/nonexistent.bms";
    let result = BmsLoader::load(path);

    assert!(result.is_err());
}

#[test]
fn test_chart_note_count() {
    let path = "tests/fixtures/simple.bms";
    let (chart, _) = BmsLoader::load(path).unwrap();

    assert!(chart.notes.len() > 0, "Chart should have notes");
    assert!(chart.note_count() > 0, "Chart should have playable notes");
}

#[test]
fn test_chart_timing_data() {
    let path = "tests/fixtures/simple.bms";
    let (chart, _) = BmsLoader::load(path).unwrap();

    assert!((chart.timing_data.initial_bpm - 120.0).abs() < 0.001);
}

#[test]
fn test_notes_are_sorted_by_time() {
    let path = "tests/fixtures/simple.bms";
    let (chart, _) = BmsLoader::load(path).unwrap();

    for i in 1..chart.notes.len() {
        assert!(
            chart.notes[i - 1].time_ms <= chart.notes[i].time_ms,
            "Notes should be sorted by time_ms"
        );
    }
}

#[test]
fn test_lane_index() {
    let path = "tests/fixtures/simple.bms";
    let (chart, _) = BmsLoader::load(path).unwrap();

    let lane_index = chart.build_lane_index();

    let total_indexed: usize = lane_index.iter().map(|v| v.len()).sum();
    assert_eq!(
        total_indexed,
        chart.notes.len(),
        "All notes should be indexed"
    );

    for (lane_idx, indices) in lane_index.iter().enumerate() {
        for &note_idx in indices {
            assert_eq!(
                chart.notes[note_idx].channel.lane_index(),
                lane_idx,
                "Note should be in correct lane"
            );
        }
    }
}
