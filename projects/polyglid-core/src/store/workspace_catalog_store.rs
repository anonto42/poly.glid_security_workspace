use std::sync::{Arc, Mutex};

use rusqlite::{params, Connection};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DbWorkspace {
    pub id: String,
    pub name: String,
    pub root_path: String,
    pub is_active: bool,
    pub discovery_state: String,
    pub last_error: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
    pub last_opened_at: Option<i64>,
}

pub struct WorkspaceCatalogStore {
    conn: Arc<Mutex<Connection>>,
}

impl WorkspaceCatalogStore {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    pub fn upsert(&self, workspace: &DbWorkspace) -> Result<(), String> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO workspaces (id,name,root_path,is_active,discovery_state,last_error,created_at,updated_at,last_opened_at)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)
             ON CONFLICT(root_path) DO UPDATE SET name=excluded.name,updated_at=excluded.updated_at",
            params![workspace.id, workspace.name, workspace.root_path, workspace.is_active as i32,
                workspace.discovery_state, workspace.last_error, workspace.created_at,
                workspace.updated_at, workspace.last_opened_at],
        ).map(|_| ()).map_err(|err| format!("failed to save workspace: {err}"))
    }

    pub fn list(&self) -> Result<Vec<DbWorkspace>, String> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id,name,root_path,is_active,discovery_state,last_error,created_at,updated_at,last_opened_at
             FROM workspaces ORDER BY is_active DESC, name COLLATE NOCASE",
        ).map_err(|err| format!("failed to prepare workspace query: {err}"))?;
        let rows = stmt
            .query_map([], map_workspace)
            .map_err(|err| format!("failed to query workspaces: {err}"))?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|err| format!("failed to read workspace row: {err}"))
    }

    pub fn get(&self, id: &str) -> Result<Option<DbWorkspace>, String> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id,name,root_path,is_active,discovery_state,last_error,created_at,updated_at,last_opened_at FROM workspaces WHERE id=?1",
        ).map_err(|err| format!("failed to prepare workspace lookup: {err}"))?;
        match stmt.query_row([id], map_workspace) {
            Ok(value) => Ok(Some(value)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(err) => Err(format!("failed to load workspace: {err}")),
        }
    }

    pub fn set_active(&self, id: &str, now: i64) -> Result<(), String> {
        let mut conn = self.conn.lock().unwrap();
        let tx = conn
            .transaction()
            .map_err(|err| format!("failed to activate workspace: {err}"))?;
        tx.execute("UPDATE workspaces SET is_active=0", [])
            .map_err(|err| format!("failed to clear active workspace: {err}"))?;
        let changed = tx
            .execute(
                "UPDATE workspaces SET is_active=1,last_opened_at=?2,updated_at=?2 WHERE id=?1",
                params![id, now],
            )
            .map_err(|err| format!("failed to set active workspace: {err}"))?;
        if changed != 1 {
            return Err(format!("workspace '{id}' was not found"));
        }
        tx.commit()
            .map_err(|err| format!("failed to commit active workspace: {err}"))
    }

    pub fn set_discovery(
        &self,
        id: &str,
        state: &str,
        error: Option<&str>,
        now: i64,
    ) -> Result<(), String> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE workspaces SET discovery_state=?2,last_error=?3,updated_at=?4 WHERE id=?1",
            params![id, state, error, now],
        )
        .map(|_| ())
        .map_err(|err| format!("failed to update discovery state: {err}"))
    }
}

fn map_workspace(row: &rusqlite::Row<'_>) -> rusqlite::Result<DbWorkspace> {
    Ok(DbWorkspace {
        id: row.get(0)?,
        name: row.get(1)?,
        root_path: row.get(2)?,
        is_active: row.get::<_, i32>(3)? != 0,
        discovery_state: row.get(4)?,
        last_error: row.get(5)?,
        created_at: row.get(6)?,
        updated_at: row.get(7)?,
        last_opened_at: row.get(8)?,
    })
}
