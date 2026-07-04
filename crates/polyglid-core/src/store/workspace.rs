use std::path::Path;
use std::sync::{Arc, Mutex};
use rusqlite::Connection;
use crate::store::migrations::MigrationManager;

pub struct WorkspaceStore {
    pub(crate) conn: Arc<Mutex<Connection>>,
}

impl Clone for WorkspaceStore {
    fn clone(&self) -> Self {
        Self {
            conn: Arc::clone(&self.conn),
        }
    }
}

impl WorkspaceStore {
    pub fn new(path: &Path) -> Result<Self, String> {
        let mut conn = Connection::open(path)
            .map_err(|err| format!("failed to open database file '{}': {err}", path.display()))?;

        MigrationManager::run(&mut conn)?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    pub fn plugins(&self) -> crate::store::plugin_store::PluginStore {
        crate::store::plugin_store::PluginStore::new(Arc::clone(&self.conn))
    }

    pub fn executions(&self) -> crate::store::execution_store::ExecutionStore {
        crate::store::execution_store::ExecutionStore::new(Arc::clone(&self.conn))
    }

    pub fn settings(&self) -> crate::store::settings_store::SettingsStore {
        crate::store::settings_store::SettingsStore::new(Arc::clone(&self.conn))
    }

    pub fn targets(&self) -> crate::store::target_store::TargetStore {
        crate::store::target_store::TargetStore::new(Arc::clone(&self.conn))
    }

    pub fn permissions(&self) -> crate::store::permission_store::DbPermissionStore {
        crate::store::permission_store::DbPermissionStore::new(Arc::clone(&self.conn))
    }

    pub fn reports(&self) -> crate::store::report_store::ReportStore {
        crate::store::report_store::ReportStore::new(Arc::clone(&self.conn))
    }

    /// Run a set of database actions atomically inside a transaction.
    pub fn transaction<F, T>(&self, f: F) -> Result<T, String>
    where
        F: FnOnce(&rusqlite::Transaction) -> Result<T, String>,
    {
        let mut conn = self.conn.lock().unwrap();
        let tx = conn
            .transaction()
            .map_err(|err| format!("failed to start transaction: {err}"))?;
        let res = f(&tx)?;
        tx.commit()
            .map_err(|err| format!("failed to commit transaction: {err}"))?;
        Ok(res)
    }
}
