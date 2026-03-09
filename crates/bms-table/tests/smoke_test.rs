//! Smoke tests for bms-table crate: zero-coverage API surface.
//!
//! These tests verify that core data types can be constructed and accessed
//! without panicking.

use std::collections::HashMap;

use serde_json::Value;

use bms_table::bms_table::BmsTable;
use bms_table::bms_table_element::BmsTableElement;
use bms_table::bms_table_manager::BmsTableManager;
use bms_table::bms_table_manager_listener::BmsTableManagerListener;
use bms_table::course::{Course, Trophy};
use bms_table::difficulty_table::DifficultyTable;
use bms_table::difficulty_table_element::DifficultyTableElement;
use bms_table::event_table::EventTable;
use bms_table::event_table_element::EventTableElement;

// ---------------------------------------------------------------------------
// DifficultyTable
// ---------------------------------------------------------------------------

#[test]
fn difficulty_table_default() {
    let dt = DifficultyTable::default();

    assert!(dt.elements().is_empty());
    assert!(dt.level_description().is_empty());
    assert!(dt.course().is_empty());
    assert!(dt.table.name().is_none());
    assert!(dt.table.id().is_none());
    assert!(dt.table.tag().is_none());
    assert!(dt.table.data_url.is_empty());
    assert!(dt.table.models.is_empty());
    assert!(!dt.table.editable);
    assert!(dt.table.auto_update);
    assert_eq!(dt.table.lastupdate, 0);
    assert_eq!(dt.table.access_count, 0);
}

#[test]
fn difficulty_table_with_source_url_and_elements() {
    let mut dt = DifficultyTable::new_with_source_url("https://example.com/table.html");

    assert_eq!(dt.table.source_url, "https://example.com/table.html");

    // Add an element
    let mut elem = DifficultyTableElement::new_with_params(
        &bms_table::difficulty_table_element::DifficultyTableElementParams {
            did: "12",
            title: "Test Song",
            bmsid: 42,
            url1: "https://example.com/dl",
            url2: "https://example.com/diff",
            comment: "hard chart",
            hash: "abc123",
            ipfs: "",
        },
    );
    elem.state = 1;
    elem.eval = 5;

    dt.table.add_element(elem);

    let elements = dt.elements();
    assert_eq!(elements.len(), 1);
    assert_eq!(elements[0].level, "12");
    assert_eq!(elements[0].element.title(), Some("Test Song"));
    assert_eq!(elements[0].bmsid(), 42);
    assert_eq!(elements[0].state, 1);
    assert_eq!(elements[0].eval, 5);
    assert_eq!(elements[0].comment(), "hard chart");
}

// ---------------------------------------------------------------------------
// BmsTableElement
// ---------------------------------------------------------------------------

#[test]
fn bms_table_element_hash_fields() {
    let mut elem = BmsTableElement::new();
    elem.set_md5("d41d8cd98f00b204e9800998ecf8427e");
    elem.set_sha256("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
    elem.set_title("Test Title");
    elem.set_artist("Test Artist");
    elem.set_mode("beat-7k");

    assert_eq!(elem.md5(), Some("d41d8cd98f00b204e9800998ecf8427e"));
    assert_eq!(
        elem.sha256(),
        Some("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855")
    );
    assert_eq!(elem.title(), Some("Test Title"));
    assert_eq!(elem.artist(), Some("Test Artist"));
    assert_eq!(elem.mode(), Some("beat-7k"));
    // Parent hash is None by default
    assert!(elem.parent_hash().is_none());
}

#[test]
fn bms_table_element_parent_hash_roundtrip() {
    let mut elem = BmsTableElement::new();

    // Set multiple parent hashes
    let hashes = vec!["hash1".to_string(), "hash2".to_string()];
    elem.set_parent_hash(Some(&hashes));
    assert_eq!(
        elem.parent_hash(),
        Some(vec!["hash1".to_string(), "hash2".to_string()])
    );

    // Clear parent hashes
    elem.set_parent_hash(None);
    assert!(elem.parent_hash().is_none());
}

// ---------------------------------------------------------------------------
// Course and Trophy
// ---------------------------------------------------------------------------

#[test]
fn course_construction() {
    let mut course = Course::new();
    // Default name is Japanese "新規段位"
    assert!(!course.name().is_empty());

    course.set_name("Dan Course A");
    assert_eq!(course.name(), "Dan Course A");

    course.set_style("7KEYS");
    assert_eq!(course.style, "7KEYS");

    course.constraint = vec!["GAUGE_LR2".to_string()];
    assert_eq!(course.constraint(), &["GAUGE_LR2".to_string()]);

    // Charts
    let mut chart = BmsTableElement::new();
    chart.set_md5("abc123");
    course.charts = vec![chart];
    assert_eq!(course.charts().len(), 1);
}

#[test]
fn trophy_construction() {
    let mut trophy = Trophy::new();
    // Default name is Japanese "新規トロフィー"
    assert!(!trophy.name().is_empty());

    trophy.set_name("Gold Trophy");
    trophy.set_style("gold");
    trophy.scorerate = 90.0;
    trophy.missrate = 5.0;

    assert_eq!(trophy.name(), "Gold Trophy");
    assert_eq!(trophy.style(), "gold");
    assert!((trophy.scorerate - 90.0).abs() < f64::EPSILON);
    assert!((trophy.missrate - 5.0).abs() < f64::EPSILON);
}

// ---------------------------------------------------------------------------
// BmsTableManager
// ---------------------------------------------------------------------------

struct TestListener {
    change_count: usize,
}

impl TestListener {
    fn new() -> Self {
        Self { change_count: 0 }
    }
}

impl BmsTableManagerListener for TestListener {
    fn model_changed(&mut self) {
        self.change_count += 1;
    }
}

#[test]
fn bms_table_manager_default() {
    let mgr = BmsTableManager::default();

    assert!(mgr.table_list.is_empty());
    assert!(mgr.bms_tables().is_empty());
    assert!(mgr.table_list().is_empty());
    assert!(mgr.user_list().is_empty());
    assert!(mgr.memo_map().is_empty());
}

#[test]
fn bms_table_manager_add_remove() {
    let mut mgr = BmsTableManager::new();

    let dt1 = DifficultyTable::new_with_source_url("https://example.com/t1");
    let dt2 = DifficultyTable::new_with_source_url("https://example.com/t2");

    mgr.add_bms_table(dt1);
    mgr.add_bms_table(dt2);
    assert_eq!(mgr.bms_tables().len(), 2);
    assert_eq!(
        mgr.table_list()[0].table.source_url,
        "https://example.com/t1"
    );
    assert_eq!(
        mgr.table_list()[1].table.source_url,
        "https://example.com/t2"
    );

    // Remove first table
    mgr.remove_bms_table(0);
    assert_eq!(mgr.bms_tables().len(), 1);
    assert_eq!(
        mgr.table_list()[0].table.source_url,
        "https://example.com/t2"
    );

    // Out-of-bounds remove is a no-op
    mgr.remove_bms_table(99);
    assert_eq!(mgr.bms_tables().len(), 1);
}

#[test]
fn bms_table_manager_user_list_and_memo_map() {
    let mut mgr = BmsTableManager::new();

    // user_list: get_user_difficulty_table_elements creates entry on first access
    let elems = mgr.get_user_difficulty_table_elements("favorites");
    assert!(elems.is_empty());

    // Push an element into the user list
    let mut elem = DifficultyTableElement::new();
    elem.set_level(Some("★12"));
    mgr.get_user_difficulty_table_elements("favorites")
        .push(elem);
    assert_eq!(mgr.user_list()["favorites"].len(), 1);
    assert_eq!(mgr.user_list()["favorites"][0].level, "★12");

    // memo_map
    mgr.memo_map
        .insert("key1".to_string(), "value1".to_string());
    mgr.memo_map
        .insert("key2".to_string(), "value2".to_string());
    assert_eq!(mgr.memo_map().len(), 2);
    assert_eq!(mgr.memo_map()["key1"], "value1");
}

#[test]
fn bms_table_manager_clear_all_table_elements() {
    let mut mgr = BmsTableManager::new();

    let mut dt = DifficultyTable::new();
    dt.table.add_element(DifficultyTableElement::new());
    dt.table.add_element(DifficultyTableElement::new());
    mgr.add_bms_table(dt);

    let mut dt2 = DifficultyTable::new();
    dt2.table.add_element(DifficultyTableElement::new());
    mgr.add_bms_table(dt2);

    assert_eq!(mgr.table_list()[0].elements().len(), 2);
    assert_eq!(mgr.table_list()[1].elements().len(), 1);

    mgr.clear_all_table_elements();
    assert!(mgr.table_list()[0].elements().is_empty());
    assert!(mgr.table_list()[1].elements().is_empty());
}

#[test]
fn bms_table_manager_table_list_mut() {
    let mut mgr = BmsTableManager::new();
    mgr.add_bms_table(DifficultyTable::new());

    // Mutate through table_list_mut
    mgr.table_list_mut()[0].table.set_name("Modified");
    assert_eq!(mgr.table_list()[0].table.name(), Some("Modified"));
}

#[test]
fn bms_table_manager_listener_fires() {
    let mut mgr = BmsTableManager::new();
    mgr.add_listener(Box::new(TestListener::new()));

    // fire_model_changed is called internally by add/remove, but we can also call it directly
    mgr.fire_model_changed();

    // Verify no panic (listener state is not accessible from outside, but we confirm it runs)
    mgr.add_bms_table(DifficultyTable::new());
    mgr.remove_bms_table(0);
}

// ---------------------------------------------------------------------------
// BmsTable generic operations
// ---------------------------------------------------------------------------

#[test]
fn bms_table_attrmap_roundtrip() {
    let mut table: BmsTable<String> = BmsTable::new();

    // Initially empty
    assert!(table.attrmap().is_empty());

    let mut attrs = HashMap::new();
    attrs.insert("difficulty".to_string(), "★".to_string());
    attrs.insert("genre".to_string(), "techno".to_string());
    table.set_attrmap(attrs.clone());

    let retrieved = table.attrmap();
    assert_eq!(retrieved.len(), 2);
    assert_eq!(retrieved["difficulty"], "★");
    assert_eq!(retrieved["genre"], "techno");
}

#[test]
fn bms_table_mode_roundtrip() {
    let mut table: BmsTable<String> = BmsTable::new();

    assert!(table.mode().is_none());

    table.set_mode("beat-7k");
    assert_eq!(table.mode(), Some("beat-7k"));

    table.set_mode("beat-14k");
    assert_eq!(table.mode(), Some("beat-14k"));
}

#[test]
fn bms_table_remove_element_at() {
    let mut table: BmsTable<String> = BmsTable::new();
    table.add_element("alpha".to_string());
    table.add_element("beta".to_string());
    table.add_element("gamma".to_string());
    assert_eq!(table.models.len(), 3);

    table.remove_element_at(1);
    assert_eq!(table.models, vec!["alpha".to_string(), "gamma".to_string()]);

    // Out-of-bounds is a no-op
    table.remove_element_at(99);
    assert_eq!(table.models.len(), 2);
}

#[test]
fn bms_table_remove_all_elements() {
    let mut table: BmsTable<i32> = BmsTable::new();
    table.add_element(1);
    table.add_element(2);
    table.add_element(3);
    assert_eq!(table.models.len(), 3);

    table.remove_all_elements();
    assert!(table.models.is_empty());
}

#[test]
fn bms_table_set_values() {
    let mut table: BmsTable<String> = BmsTable::new();

    let mut values = HashMap::new();
    values.insert("name".to_string(), Value::String("Test Table".to_string()));
    values.insert("symbol".to_string(), Value::String("TT".to_string()));
    values.insert("mode".to_string(), Value::String("beat-7k".to_string()));
    table.set_values(&values);

    assert_eq!(table.name(), Some("Test Table"));
    assert_eq!(table.id(), Some("TT"));
    assert_eq!(table.mode(), Some("beat-7k"));
}

#[test]
fn bms_table_name_id_tag_roundtrip() {
    let mut table: BmsTable<String> = BmsTable::new();

    // All None initially
    assert!(table.name().is_none());
    assert!(table.id().is_none());
    assert!(table.tag().is_none());

    table.set_name("My Table");
    assert_eq!(table.name(), Some("My Table"));

    table.set_id("MT");
    assert_eq!(table.id(), Some("MT"));
    // tag falls back to id when not explicitly set
    assert_eq!(table.tag(), Some("MT".to_string()));

    table.set_tag("custom-tag");
    assert_eq!(table.tag(), Some("custom-tag".to_string()));
}

#[test]
fn bms_table_set_models() {
    let mut table: BmsTable<i32> = BmsTable::new();
    assert!(table.models.is_empty());

    table.set_models(vec![10, 20, 30]);
    assert_eq!(table.models, vec![10, 20, 30]);

    // Overwrite
    table.set_models(vec![42]);
    assert_eq!(table.models, vec![42]);
}

#[test]
fn bms_table_add_element_updates_lastupdate() {
    let mut table: BmsTable<String> = BmsTable::new();
    assert_eq!(table.lastupdate, 0);

    table.add_element("first".to_string());
    assert!(
        table.lastupdate > 0,
        "lastupdate should be set after add_element"
    );
}

// ---------------------------------------------------------------------------
// EventTable / EventTableElement
// ---------------------------------------------------------------------------

#[test]
fn event_table_element_default() {
    let ete = EventTableElement::new();

    assert!(ete.artist().is_none());
    assert!(ete.team().is_none());
    assert!(ete.element.title().is_none());
    assert!(ete.element.md5().is_none());
}

#[test]
fn event_table_element_accessors() {
    let mut ete = EventTableElement::new();

    ete.set_artist(Some("DJ Test"));
    assert_eq!(ete.artist(), Some("DJ Test"));

    ete.set_team(Some("Team Alpha"));
    assert_eq!(ete.team(), Some("Team Alpha"));

    // Clear
    ete.set_artist(None);
    assert!(ete.artist().is_none());

    ete.set_team(None);
    assert!(ete.team().is_none());
}

#[test]
fn event_table_element_inner_element() {
    let mut ete = EventTableElement::new();

    ete.element.set_title("Event Song");
    ete.element.set_md5("abc123");
    ete.element.set_sha256("def456");

    assert_eq!(ete.element.title(), Some("Event Song"));
    assert_eq!(ete.element.md5(), Some("abc123"));
    assert_eq!(ete.element.sha256(), Some("def456"));
}

#[test]
fn event_table_element_default_trait() {
    let ete = EventTableElement::default();
    assert!(ete.artist().is_none());
    assert!(ete.team().is_none());
}

#[test]
fn event_table_as_bms_table() {
    let mut et: EventTable = EventTable::new();

    assert!(et.models.is_empty());
    assert!(et.name().is_none());

    et.set_name("Spring Event 2026");
    assert_eq!(et.name(), Some("Spring Event 2026"));

    let mut elem = EventTableElement::new();
    elem.set_artist(Some("Artist A"));
    elem.element.set_title("Song A");
    et.add_element(elem);

    let mut elem2 = EventTableElement::new();
    elem2.set_team(Some("Team B"));
    elem2.element.set_title("Song B");
    et.add_element(elem2);

    assert_eq!(et.models.len(), 2);
    assert_eq!(et.models[0].element.title(), Some("Song A"));
    assert_eq!(et.models[1].team(), Some("Team B"));

    et.remove_element_at(0);
    assert_eq!(et.models.len(), 1);
    assert_eq!(et.models[0].element.title(), Some("Song B"));

    et.remove_all_elements();
    assert!(et.models.is_empty());
}

// ---------------------------------------------------------------------------
// BmsTableElement additional coverage
// ---------------------------------------------------------------------------

#[test]
fn bms_table_element_url_and_ipfs() {
    let mut elem = BmsTableElement::new();

    assert!(elem.url().is_none());
    assert!(elem.ipfs().is_none());

    elem.set_url("https://example.com/download");
    assert_eq!(elem.url(), Some("https://example.com/download"));

    elem.set_ipfs("QmSomeHash");
    assert_eq!(elem.ipfs(), Some("QmSomeHash"));
}

#[test]
fn bms_table_element_set_values_replaces_all() {
    let mut elem = BmsTableElement::new();
    elem.set_title("Original");
    elem.set_md5("original_hash");

    let mut new_values = HashMap::new();
    new_values.insert("title".to_string(), Value::String("Replaced".to_string()));
    elem.set_values(&new_values);

    assert_eq!(elem.title(), Some("Replaced"));
    // md5 is gone because set_values replaces the entire map
    assert!(elem.md5().is_none());
}

#[test]
fn bms_table_element_default_trait() {
    let elem = BmsTableElement::default();
    assert!(elem.title().is_none());
    assert!(elem.values.is_empty());
}

// ---------------------------------------------------------------------------
// DifficultyTableElement additional coverage
// ---------------------------------------------------------------------------

#[test]
fn difficulty_table_element_package_url_roundtrip() {
    let mut dte = DifficultyTableElement::new();

    assert!(dte.package_url().is_none());
    assert!(dte.package_name().is_none());

    dte.set_package_url("https://example.com/pack.zip");
    dte.set_package_name("Song Pack");

    assert_eq!(dte.package_url(), Some("https://example.com/pack.zip"));
    assert_eq!(dte.package_name(), Some("Song Pack"));
}

#[test]
fn difficulty_table_element_append_url_and_ipfs() {
    let mut dte = DifficultyTableElement::new();

    assert!(dte.append_url().is_none());
    assert!(dte.append_ipfs().is_none());

    dte.set_append_url("https://example.com/diff");
    dte.set_append_ipfs("QmDiffHash");

    assert_eq!(dte.append_url(), Some("https://example.com/diff"));
    assert_eq!(dte.append_ipfs(), Some("QmDiffHash"));
}

#[test]
fn difficulty_table_element_append_artist() {
    let mut dte = DifficultyTableElement::new();

    assert_eq!(dte.append_artist(), "");

    dte.set_append_artist("Diff Author");
    assert_eq!(dte.append_artist(), "Diff Author");
}

#[test]
fn difficulty_table_element_set_values_populates_fields() {
    let mut dte = DifficultyTableElement::new();

    let mut values = HashMap::new();
    values.insert("level".to_string(), Value::String("★5".to_string()));
    values.insert(
        "name_diff".to_string(),
        Value::String("hard diff".to_string()),
    );
    values.insert(
        "comment".to_string(),
        Value::String("tricky patterns".to_string()),
    );
    values.insert("tag".to_string(), Value::String("scratch".to_string()));
    values.insert("proposer".to_string(), Value::String("tester".to_string()));
    values.insert("title".to_string(), Value::String("Song Title".to_string()));
    dte.set_values(&values);

    assert_eq!(dte.level, "★5");
    assert_eq!(dte.append_artist(), "hard diff");
    assert_eq!(dte.comment(), "tricky patterns");
    assert_eq!(dte.information(), "scratch");
    assert_eq!(dte.proposer(), "tester");
    assert_eq!(dte.element.title(), Some("Song Title"));
    // set_values resets state and eval to 0
    assert_eq!(dte.state, 0);
    assert_eq!(dte.eval, 0);
}

#[test]
fn difficulty_table_element_values_output() {
    let mut dte = DifficultyTableElement::new();
    dte.set_level(Some("10"));
    dte.set_comment("my comment");
    dte.set_information("my info");
    dte.set_proposer("me");
    dte.set_append_artist("diff name");
    dte.state = 2;
    dte.eval = 3;

    let vals = dte.values();
    assert_eq!(vals["level"].as_str().unwrap(), "10");
    assert_eq!(vals["comment"].as_str().unwrap(), "my comment");
    assert_eq!(vals["tag"].as_str().unwrap(), "my info");
    assert_eq!(vals["proposer"].as_str().unwrap(), "me");
    assert_eq!(vals["name_diff"].as_str().unwrap(), "diff name");
    assert_eq!(vals["state"].as_i64().unwrap(), 2);
    assert_eq!(vals["eval"].as_i64().unwrap(), 3);
}

#[test]
fn difficulty_table_element_values_omits_empty_proposer() {
    let dte = DifficultyTableElement::new();
    let vals = dte.values();
    assert!(
        !vals.contains_key("proposer"),
        "empty proposer should be omitted"
    );
}

#[test]
fn difficulty_table_element_set_level_none() {
    let mut dte = DifficultyTableElement::new();
    dte.set_level(Some("12"));
    assert_eq!(dte.level, "12");

    dte.set_level(None);
    assert_eq!(dte.level, "");
}

// ---------------------------------------------------------------------------
// DifficultyTable additional coverage
// ---------------------------------------------------------------------------

#[test]
fn difficulty_table_set_level_description_roundtrip() {
    let mut dt = DifficultyTable::new();

    let levels = vec!["★1".to_string(), "★2".to_string(), "★3".to_string()];
    dt.set_level_description(&levels);
    assert_eq!(dt.level_description(), levels);

    // Overwrite
    let levels2 = vec!["A".to_string(), "B".to_string()];
    dt.set_level_description(&levels2);
    assert_eq!(dt.level_description(), levels2);
}

// ---------------------------------------------------------------------------
// Course additional coverage
// ---------------------------------------------------------------------------

#[test]
fn course_trophy_integration() {
    let mut course = Course::new();

    let mut trophy1 = Trophy::new();
    trophy1.set_name("Bronze");
    trophy1.set_style("bronze");
    trophy1.scorerate = 50.0;
    trophy1.missrate = 30.0;

    let mut trophy2 = Trophy::new();
    trophy2.set_name("Silver");
    trophy2.set_style("silver");
    trophy2.scorerate = 75.0;
    trophy2.missrate = 10.0;

    course.trophy = vec![trophy1, trophy2];
    assert_eq!(course.trophy.len(), 2);
    assert_eq!(course.trophy[0].name(), "Bronze");
    assert_eq!(course.trophy[1].name(), "Silver");
}

#[test]
fn trophy_default_values() {
    let trophy = Trophy::default();
    // Default missrate is 100.0, scorerate is 0.0
    assert!((trophy.scorerate - 0.0).abs() < f64::EPSILON);
    assert!((trophy.missrate - 100.0).abs() < f64::EPSILON);
    assert!(trophy.style().is_empty());
}

#[test]
fn course_default_trait() {
    let course = Course::default();
    assert!(!course.name().is_empty()); // Default Japanese name
    assert!(course.charts().is_empty());
    assert!(course.constraint().is_empty());
    assert!(course.style.is_empty());
    assert!(course.trophy.is_empty());
}
