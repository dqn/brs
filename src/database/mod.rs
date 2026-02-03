// Database subsystem using rusqlite.

mod connection;
mod models;
mod scan_task;
mod scanner;
mod score_db;
mod song_db;

pub use connection::{Database, DatabaseType};
pub use models::{ClearType, Mode, ScanResult, ScoreData, SongData};
pub use scan_task::{ScanProgress, ScanStage, SongScanTask};
pub use scanner::{SongScanner, calc_md5, calc_sha256};
pub use score_db::ScoreDatabaseAccessor;
pub use song_db::SongDatabaseAccessor;
