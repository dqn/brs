use anyhow::Result;

use crate::account::IRAccount;
use crate::chart_data::IRChartData;
use crate::connection::IRConnection;
use crate::course_data::IRCourseData;
use crate::lr2ir::LR2IRConnection;
use crate::player_data::IRPlayerData;
use crate::response::IRResponse;
use crate::score_data::IRScoreData;
use crate::table_data::IRTableData;

/// Known IR connection names.
const AVAILABLE_NAMES: &[&str] = &["LR2IR"];

/// IR connection type enum.
///
/// Enum dispatch to avoid `dyn` trait objects with async methods.
pub enum IRConnectionType {
    LR2IR(LR2IRConnection),
}

impl IRConnection for IRConnectionType {
    async fn register(&self, account: &IRAccount) -> Result<IRResponse<IRPlayerData>> {
        match self {
            IRConnectionType::LR2IR(conn) => conn.register(account).await,
        }
    }

    async fn login(&self, account: &IRAccount) -> Result<IRResponse<IRPlayerData>> {
        match self {
            IRConnectionType::LR2IR(conn) => conn.login(account).await,
        }
    }

    async fn get_rivals(&self) -> Result<IRResponse<Vec<IRPlayerData>>> {
        match self {
            IRConnectionType::LR2IR(conn) => conn.get_rivals().await,
        }
    }

    async fn get_table_datas(&self) -> Result<IRResponse<Vec<IRTableData>>> {
        match self {
            IRConnectionType::LR2IR(conn) => conn.get_table_datas().await,
        }
    }

    async fn get_play_data(
        &self,
        player: Option<&IRPlayerData>,
        chart: &IRChartData,
    ) -> Result<IRResponse<Vec<IRScoreData>>> {
        match self {
            IRConnectionType::LR2IR(conn) => conn.get_play_data(player, chart).await,
        }
    }

    async fn get_course_play_data(
        &self,
        player: Option<&IRPlayerData>,
        course: &IRCourseData,
    ) -> Result<IRResponse<Vec<IRScoreData>>> {
        match self {
            IRConnectionType::LR2IR(conn) => conn.get_course_play_data(player, course).await,
        }
    }

    async fn send_play_data(
        &self,
        chart: &IRChartData,
        score: &IRScoreData,
    ) -> Result<IRResponse<()>> {
        match self {
            IRConnectionType::LR2IR(conn) => conn.send_play_data(chart, score).await,
        }
    }

    async fn send_course_play_data(
        &self,
        course: &IRCourseData,
        score: &IRScoreData,
    ) -> Result<IRResponse<()>> {
        match self {
            IRConnectionType::LR2IR(conn) => conn.send_course_play_data(course, score).await,
        }
    }

    async fn get_song_url(&self, chart: &IRChartData) -> Option<String> {
        match self {
            IRConnectionType::LR2IR(conn) => conn.get_song_url(chart).await,
        }
    }

    async fn get_course_url(&self, course: &IRCourseData) -> Option<String> {
        match self {
            IRConnectionType::LR2IR(conn) => conn.get_course_url(course).await,
        }
    }

    async fn get_player_url(&self, player: &IRPlayerData) -> Option<String> {
        match self {
            IRConnectionType::LR2IR(conn) => conn.get_player_url(player).await,
        }
    }
}

/// IR connection manager.
///
/// Corresponds to Java `IRConnectionManager`.
/// Java uses reflection-based discovery; Rust uses a static match.
pub struct IRConnectionManager;

impl IRConnectionManager {
    /// Get all available IR connection names.
    pub fn available_names() -> &'static [&'static str] {
        AVAILABLE_NAMES
    }

    /// Create an IR connection by name.
    pub fn create(name: &str) -> Option<IRConnectionType> {
        match name {
            "LR2IR" => Some(IRConnectionType::LR2IR(LR2IRConnection::new())),
            _ => None,
        }
    }

    /// Get the home URL for an IR by name.
    pub fn home_url(name: &str) -> Option<&'static str> {
        match name {
            "LR2IR" => Some("http://dream-pro.info/~lavalse/LR2IR/2"),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn available_names_contains_lr2ir() {
        let names = IRConnectionManager::available_names();
        assert!(names.contains(&"LR2IR"));
    }

    #[test]
    fn create_lr2ir() {
        let conn = IRConnectionManager::create("LR2IR");
        assert!(conn.is_some());
    }

    #[test]
    fn create_unknown_returns_none() {
        let conn = IRConnectionManager::create("UnknownIR");
        assert!(conn.is_none());
    }

    #[test]
    fn home_url_lr2ir() {
        let url = IRConnectionManager::home_url("LR2IR");
        assert_eq!(url, Some("http://dream-pro.info/~lavalse/LR2IR/2"));
    }

    #[test]
    fn home_url_unknown_returns_none() {
        let url = IRConnectionManager::home_url("UnknownIR");
        assert!(url.is_none());
    }
}
