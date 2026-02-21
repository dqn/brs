use std::sync::Mutex;

static BATTLE: Mutex<Option<Settings>> = Mutex::new(None);

#[derive(Clone, Debug)]
pub struct Settings {
    pub random: i32,
    pub lanes: i32,
}

pub fn consume() -> Option<Settings> {
    let mut lock = BATTLE.lock().unwrap();
    lock.take()
}

pub fn setup(random: i32, lane_sequence: i32) {
    let mut lock = BATTLE.lock().unwrap();
    *lock = Some(Settings {
        random,
        lanes: lane_sequence,
    });
}
