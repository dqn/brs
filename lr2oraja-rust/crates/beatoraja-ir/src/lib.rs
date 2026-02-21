#![allow(clippy::needless_range_loop)]
#![allow(unused_imports)]

// Stubs for external dependencies
pub mod stubs;

// IR data types
pub mod ir_account;
pub mod ir_chart_data;
pub mod ir_connection;
pub mod ir_connection_manager;
pub mod ir_course_data;
pub mod ir_player_data;
pub mod ir_response;
pub mod ir_score_data;
pub mod ir_table_data;

// Leaderboard
pub mod leaderboard_entry;

// LR2 IR
pub mod lr2_ghost_data;
pub mod lr2_ir_connection;

// Ranking
pub mod ranking_data;
pub mod ranking_data_cache;
