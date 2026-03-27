//! Database access layer for the rubato BMS player.
//!
//! Consolidates SQLite-based database accessors for scores, score logs,
//! player data, and play data.

// Base SQLite abstraction
pub mod sqlite_database_accessor;

// Score database accessors
pub mod score_data_log_database_accessor;
pub mod score_database_accessor;
pub mod score_log_database_accessor;

// Composite play data accessor
pub mod play_data_accessor;

// Score data importer (IR merge)
pub mod score_data_importer;
