use bms_model::mode::Mode;

#[derive(Clone)]
pub struct LaneProperty {
    /// Key to lane mapping
    key_to_lane: Vec<i32>,
    /// Lane to key(s) mapping
    lane_to_key: Vec<Vec<i32>>,
    /// Lane to scratch index (-1 if not scratch)
    lane_to_scratch: Vec<i32>,
    /// Lane to skin offset mapping
    lane_to_skin_offset: Vec<i32>,
    /// Lane to player number mapping
    lane_to_player: Vec<i32>,
    /// Scratch to key mapping (2 keys per scratch)
    scratch_to_key: Vec<Vec<i32>>,
}

impl LaneProperty {
    pub fn new(mode: &Mode) -> Self {
        let (key_to_lane, lane_to_key, lane_to_scratch, lane_to_skin_offset, scratch_to_key) =
            match mode {
                Mode::BEAT_5K => (
                    vec![0, 1, 2, 3, 4, 5, 5],
                    vec![vec![0], vec![1], vec![2], vec![3], vec![4], vec![5, 6]],
                    vec![-1, -1, -1, -1, -1, 0],
                    vec![1, 2, 3, 4, 5, 0],
                    vec![vec![5, 6]],
                ),
                Mode::BEAT_7K => (
                    vec![0, 1, 2, 3, 4, 5, 6, 7, 7],
                    vec![
                        vec![0],
                        vec![1],
                        vec![2],
                        vec![3],
                        vec![4],
                        vec![5],
                        vec![6],
                        vec![7, 8],
                    ],
                    vec![-1, -1, -1, -1, -1, -1, -1, 0],
                    vec![1, 2, 3, 4, 5, 6, 7, 0],
                    vec![vec![7, 8]],
                ),
                Mode::BEAT_10K => (
                    vec![0, 1, 2, 3, 4, 5, 5, 6, 7, 8, 9, 10, 11, 11],
                    vec![
                        vec![0],
                        vec![1],
                        vec![2],
                        vec![3],
                        vec![4],
                        vec![5, 6],
                        vec![7],
                        vec![8],
                        vec![9],
                        vec![10],
                        vec![11],
                        vec![12, 13],
                    ],
                    vec![-1, -1, -1, -1, -1, 0, -1, -1, -1, -1, -1, 1],
                    vec![1, 2, 3, 4, 5, 0, 1, 2, 3, 4, 5, 0],
                    vec![vec![5, 6], vec![12, 13]],
                ),
                Mode::BEAT_14K => (
                    vec![0, 1, 2, 3, 4, 5, 6, 7, 7, 8, 9, 10, 11, 12, 13, 14, 15, 15],
                    vec![
                        vec![0],
                        vec![1],
                        vec![2],
                        vec![3],
                        vec![4],
                        vec![5],
                        vec![6],
                        vec![7, 8],
                        vec![9],
                        vec![10],
                        vec![11],
                        vec![12],
                        vec![13],
                        vec![14],
                        vec![15],
                        vec![16, 17],
                    ],
                    vec![-1, -1, -1, -1, -1, -1, -1, 0, -1, -1, -1, -1, -1, -1, -1, 1],
                    vec![1, 2, 3, 4, 5, 6, 7, 0, 1, 2, 3, 4, 5, 6, 7, 0],
                    vec![vec![7, 8], vec![16, 17]],
                ),
                Mode::POPN_5K | Mode::POPN_9K => (
                    vec![0, 1, 2, 3, 4, 5, 6, 7, 8],
                    vec![
                        vec![0],
                        vec![1],
                        vec![2],
                        vec![3],
                        vec![4],
                        vec![5],
                        vec![6],
                        vec![7],
                        vec![8],
                    ],
                    vec![-1, -1, -1, -1, -1, -1, -1, -1, -1],
                    vec![1, 2, 3, 4, 5, 6, 7, 8, 9],
                    vec![],
                ),
                Mode::KEYBOARD_24K => {
                    let mut key_to_lane = vec![0i32; 26];
                    let mut lane_to_key = vec![vec![0i32]; 26];
                    let mut lane_to_scratch = vec![0i32; 26];
                    let mut lane_to_skin_offset = vec![0i32; 26];
                    for i in 0..26 {
                        key_to_lane[i] = i as i32;
                        lane_to_key[i] = vec![i as i32];
                        lane_to_scratch[i] = -1;
                        lane_to_skin_offset[i] = i as i32 + 1;
                    }
                    (
                        key_to_lane,
                        lane_to_key,
                        lane_to_scratch,
                        lane_to_skin_offset,
                        vec![],
                    )
                }
                Mode::KEYBOARD_24K_DOUBLE => {
                    let mut key_to_lane = vec![0i32; 52];
                    let mut lane_to_key = vec![vec![0i32]; 52];
                    let mut lane_to_scratch = vec![0i32; 52];
                    let mut lane_to_skin_offset = vec![0i32; 52];
                    for i in 0..52 {
                        key_to_lane[i] = i as i32;
                        lane_to_key[i] = vec![i as i32];
                        lane_to_scratch[i] = -1;
                        lane_to_skin_offset[i] = (i % 26) as i32 + 1;
                    }
                    (
                        key_to_lane,
                        lane_to_key,
                        lane_to_scratch,
                        lane_to_skin_offset,
                        vec![],
                    )
                }
            };

        let key = mode.key() as usize;
        let player_count = mode.player() as usize;
        let mut lane_to_player = vec![0i32; key];
        for i in 0..key {
            lane_to_player[i] = (i / (key / player_count)) as i32;
        }

        LaneProperty {
            key_to_lane,
            lane_to_key,
            lane_to_scratch,
            lane_to_skin_offset,
            lane_to_player,
            scratch_to_key,
        }
    }

    pub fn get_key_lane_assign(&self) -> &[i32] {
        &self.key_to_lane
    }

    pub fn get_lane_key_assign(&self) -> &[Vec<i32>] {
        &self.lane_to_key
    }

    pub fn get_lane_scratch_assign(&self) -> &[i32] {
        &self.lane_to_scratch
    }

    pub fn get_lane_skin_offset(&self) -> &[i32] {
        &self.lane_to_skin_offset
    }

    pub fn get_lane_player(&self) -> &[i32] {
        &self.lane_to_player
    }

    pub fn get_scratch_key_assign(&self) -> &[Vec<i32>] {
        &self.scratch_to_key
    }
}
