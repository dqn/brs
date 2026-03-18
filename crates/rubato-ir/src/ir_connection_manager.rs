use std::sync::Mutex;

use log::{error, warn};

use crate::ir_connection::IRConnection;

/// Registry entry for an IR connection implementation
pub struct IRConnectionEntry {
    pub name: String,
    pub home: Option<String>,
    pub factory: Box<dyn Fn() -> Box<dyn IRConnection + Send + Sync> + Send + Sync>,
}

/// IR connection manager
///
/// Translated from: IRConnectionManager.java
///
/// In Java, this uses reflection and classpath scanning to discover IRConnection
/// implementations. In Rust, we use a manual registry since there's no reflection.
/// Implementations register themselves via `register_ir_connections`.
///
/// Uses Mutex instead of OnceLock so that connections can be registered at any
/// time (including after first access). This matches the Java pattern where
/// classpath scanning can discover JARs added at runtime.
static IR_CONNECTIONS: Mutex<Vec<IRConnectionEntry>> = Mutex::new(Vec::new());

pub struct IRConnectionManager;

impl IRConnectionManager {
    /// Get all available IR connection names
    pub fn all_available_ir_connection_name() -> Vec<String> {
        let entries = IR_CONNECTIONS.lock().unwrap_or_else(|e| e.into_inner());
        let names: Vec<String> = entries.iter().map(|e| e.name.clone()).collect();
        if names.is_empty() {
            warn!("No IR connections registered. IR features are disabled.");
        }
        names
    }

    /// Get an IRConnection instance by name. Returns None if not found.
    pub fn ir_connection(name: &str) -> Option<Box<dyn IRConnection + Send + Sync>> {
        if name.is_empty() {
            return None;
        }
        let entries = IR_CONNECTIONS.lock().unwrap_or_else(|e| e.into_inner());
        for entry in entries.iter() {
            if entry.name == name {
                return Some((entry.factory)());
            }
        }
        None
    }

    /// Get the home URL for an IR by name. Returns None if not found.
    pub fn home_url(name: &str) -> Option<String> {
        let entries = IR_CONNECTIONS.lock().unwrap_or_else(|e| e.into_inner());
        for entry in entries.iter() {
            if entry.name == name {
                return entry.home.clone();
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Guard that clears the IR_CONNECTIONS after each test to avoid interference.
    struct IrConnectionsGuard;

    impl Drop for IrConnectionsGuard {
        fn drop(&mut self) {
            // Use unwrap_or_else to handle poisoned lock during cleanup
            let mut entries = IR_CONNECTIONS.lock().unwrap_or_else(|e| e.into_inner());
            entries.clear();
        }
    }

    #[test]
    fn all_available_ir_connection_name_recovers_from_poisoned_lock() {
        // Regression: .expect() would panic on a poisoned lock.
        // After fix, .unwrap_or_else(|e| e.into_inner()) recovers gracefully.
        let _guard = IrConnectionsGuard;

        // Poison the lock by panicking while holding it
        let result = std::panic::catch_unwind(|| {
            let _lock = IR_CONNECTIONS.lock().unwrap();
            panic!("intentional panic to poison lock");
        });
        assert!(result.is_err(), "should have caught the panic");

        // This should NOT panic even though the lock is poisoned
        let names = IRConnectionManager::all_available_ir_connection_name();
        assert!(names.is_empty());
    }

    #[test]
    fn ir_connection_recovers_from_poisoned_lock() {
        let _guard = IrConnectionsGuard;

        let result = std::panic::catch_unwind(|| {
            let _lock = IR_CONNECTIONS.lock().unwrap();
            panic!("intentional panic to poison lock");
        });
        assert!(result.is_err());

        // Should NOT panic
        let conn = IRConnectionManager::ir_connection("nonexistent");
        assert!(conn.is_none());
    }

    #[test]
    fn home_url_recovers_from_poisoned_lock() {
        let _guard = IrConnectionsGuard;

        let result = std::panic::catch_unwind(|| {
            let _lock = IR_CONNECTIONS.lock().unwrap();
            panic!("intentional panic to poison lock");
        });
        assert!(result.is_err());

        // Should NOT panic
        let url = IRConnectionManager::home_url("nonexistent");
        assert!(url.is_none());
    }
}

/// Register IR connection implementations.
///
/// Can be called at any time, including after `IRConnectionManager` methods
/// have already been used. New entries are appended to the existing registry.
///
/// Duplicate names are allowed (first match wins in lookups).
/// No warning is logged for duplicates; current callers never register the same name twice.
pub fn register_ir_connections(entries: Vec<IRConnectionEntry>) {
    match IR_CONNECTIONS.lock() {
        Ok(mut connections) => {
            for entry in &entries {
                log::info!("Registering IR connection: {}", entry.name);
            }
            connections.extend(entries);
        }
        Err(e) => {
            error!("Failed to register IR connections (lock poisoned): {}", e);
        }
    }
}
