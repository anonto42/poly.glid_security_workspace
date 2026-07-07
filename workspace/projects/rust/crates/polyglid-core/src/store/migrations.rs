use rusqlite::Connection;
use crate::store::schema::MIGRATIONS;

pub struct MigrationManager;

impl MigrationManager {
    pub fn run(conn: &mut Connection) -> Result<(), String> {
        // Enforce foreign key constraints
        conn.execute("PRAGMA foreign_keys = ON;", [])
            .map_err(|err| format!("failed to enable foreign keys: {err}"))?;

        // Read current version
        let current_version: i32 = conn
            .query_row("PRAGMA user_version;", [], |row| row.get(0))
            .map_err(|err| format!("failed to read user_version: {err}"))?;

        for migration in MIGRATIONS {
            if migration.version > current_version {
                let tx = conn
                    .transaction()
                    .map_err(|err| format!("failed to start migration transaction: {err}"))?;

                for sql in migration.sqls {
                    tx.execute(sql, [])
                        .map_err(|err| format!("failed to execute migration statement: {err}"))?;
                }

                // Update the user_version pragma inside the transaction
                tx.execute(&format!("PRAGMA user_version = {};", migration.version), [])
                    .map_err(|err| format!("failed to update user_version: {err}"))?;

                tx.commit()
                    .map_err(|err| format!("failed to commit migration version {}: {err}", migration.version))?;
            }
        }

        Ok(())
    }
}
