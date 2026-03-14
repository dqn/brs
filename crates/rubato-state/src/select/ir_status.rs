use rubato_ir::ir_connection::IRConnection;

/// MainController.IRStatus -- uses dyn IRConnection trait
pub struct IRStatus {
    pub connection: Box<dyn IRConnection>,
    pub player: rubato_ir::ir_player_data::IRPlayerData,
}
