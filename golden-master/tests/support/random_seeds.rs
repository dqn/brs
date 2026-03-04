use std::collections::HashMap;
use std::path::Path;

#[allow(dead_code)]
pub fn try_load_selected_randoms(test_bms_dir: &Path, chart_name: &str) -> Option<Vec<i32>> {
    let seeds_path = test_bms_dir.join("random_seeds.json");
    let content = std::fs::read_to_string(&seeds_path)
        .unwrap_or_else(|e| panic!("Failed to read {}: {}", seeds_path.display(), e));

    let seeds_map: HashMap<String, Vec<i32>> = serde_json::from_str(&content)
        .unwrap_or_else(|e| panic!("Failed to parse {}: {}", seeds_path.display(), e));

    seeds_map.get(chart_name).cloned()
}

#[allow(dead_code)]
pub fn load_selected_randoms(test_bms_dir: &Path, chart_name: &str) -> Vec<i32> {
    try_load_selected_randoms(test_bms_dir, chart_name).unwrap_or_else(|| {
        let seeds_path = test_bms_dir.join("random_seeds.json");
        panic!(
            "No random seed entry for {} in {}",
            chart_name,
            seeds_path.display()
        )
    })
}
