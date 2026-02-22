use std::sync::Arc;

use beatoraja_ir::ir_connection::IRConnection;
use beatoraja_types::song_data::SongData;

/// MainController.IRSendStatus — handles IR score submission
pub struct IRSendStatusMain {
    pub connection: Arc<dyn IRConnection>,
    pub songdata: SongData,
    pub score: beatoraja_core::score_data::ScoreData,
    pub retry: i32,
}

impl IRSendStatusMain {
    pub fn new(
        connection: Arc<dyn IRConnection>,
        songdata: &SongData,
        score: &beatoraja_core::score_data::ScoreData,
    ) -> Self {
        Self {
            connection,
            songdata: songdata.clone(),
            score: score.clone(),
            retry: 0,
        }
    }

    pub fn send(&mut self) -> bool {
        log::warn!("not yet implemented: IRSendStatus.send");
        false
    }
}
