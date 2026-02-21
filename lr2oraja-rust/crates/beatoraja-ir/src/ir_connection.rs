use crate::ir_account::IRAccount;
use crate::ir_chart_data::IRChartData;
use crate::ir_course_data::IRCourseData;
use crate::ir_player_data::IRPlayerData;
use crate::ir_response::IRResponse;
use crate::ir_score_data::IRScoreData;
use crate::ir_table_data::IRTableData;

/// IR connection interface
///
/// Translated from: IRConnection.java (interface)
pub trait IRConnection {
    /// Register a new user on IR.
    fn register(&self, account: &IRAccount) -> IRResponse<IRPlayerData> {
        let _ = account;
        panic!(
            "Use of this function with this signature without providing an implementation is not permitted"
        );
    }

    /// Register a new user on IR with id, pass, name.
    fn register_with_credentials(
        &self,
        id: &str,
        pass: &str,
        name: &str,
    ) -> IRResponse<IRPlayerData> {
        let _ = (id, pass, name);
        panic!(
            "Use of this function with this signature without providing an implementation is not permitted"
        );
    }

    /// Login to IR. Called at startup.
    fn login(&self, account: &IRAccount) -> IRResponse<IRPlayerData> {
        let _ = account;
        panic!(
            "Use of this function with this signature without providing an implementation is not permitted"
        );
    }

    /// Login to IR with id and pass.
    fn login_with_credentials(&self, id: &str, pass: &str) -> IRResponse<IRPlayerData> {
        let _ = (id, pass);
        panic!(
            "Use of this function with this signature without providing an implementation is not permitted"
        );
    }

    /// Get rival data
    fn get_rivals(&self) -> IRResponse<Vec<IRPlayerData>>;

    /// Get table data configured on IR
    fn get_table_datas(&self) -> IRResponse<Vec<IRTableData>>;

    /// Get score data
    fn get_play_data(
        &self,
        player: Option<&IRPlayerData>,
        chart: &IRChartData,
    ) -> IRResponse<Vec<IRScoreData>>;

    /// Get course play data
    fn get_course_play_data(
        &self,
        player: Option<&IRPlayerData>,
        course: &IRCourseData,
    ) -> IRResponse<Vec<IRScoreData>>;

    /// Send score data
    fn send_play_data(&self, model: &IRChartData, score: &IRScoreData) -> IRResponse<()>;

    /// Send course score data
    fn send_course_play_data(&self, course: &IRCourseData, score: &IRScoreData) -> IRResponse<()>;

    /// Get song URL. Returns None if not found.
    fn get_song_url(&self, chart: &IRChartData) -> Option<String>;

    /// Get course URL. Returns None if not found.
    fn get_course_url(&self, course: &IRCourseData) -> Option<String>;

    /// Get player URL.
    fn get_player_url(&self, player: &IRPlayerData) -> Option<String>;

    /// Get the NAME constant for this IR connection
    fn name(&self) -> &str;
}
