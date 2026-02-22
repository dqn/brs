use std::sync::Arc;

use beatoraja_core::ir_config::IRConfig;
use beatoraja_ir::ir_connection::IRConnection;

/// MainController.IRStatus — IR connection status
pub struct IRStatus {
    pub connection: Arc<dyn IRConnection>,
    pub config: IRConfig,
}
