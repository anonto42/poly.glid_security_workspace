use std::sync::{Arc, Mutex};
use rusqlite::{params, Connection};
use std::time::{SystemTime, UNIX_EPOCH};

pub struct DbTrustStore {
    conn: Arc<Mutex<Connection>>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PublisherRecord {
    pub id: String,
    pub name: String,
    pub public_key: String,
    pub fingerprint: String,
    pub created_at: u64,
    pub last_verified_at: u64,
    pub trust_level: String,
    pub revocation_status: i32, // 0=Active, 1=Revoked
}

impl DbTrustStore {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    pub fn add_publisher(
        &self,
        id: &str,
        name: &str,
        public_key: &str,
        fingerprint: &str,
        trust_level: &str,
    ) -> Result<(), String> {
        let conn = self.conn.lock().unwrap();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        conn.execute(
            "INSERT OR REPLACE INTO trusted_publishers (id, name, public_key, fingerprint, created_at, last_verified_at, trust_level, revocation_status)
             VALUES (?, ?, ?, ?, ?, ?, ?, 0)",
            params![id, name, public_key, fingerprint, now, now, trust_level],
        )
        .map_err(|err| format!("failed to insert trusted publisher: {err}"))?;
        Ok(())
    }

    pub fn get_publisher(&self, id: &str) -> Result<Option<PublisherRecord>, String> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare("SELECT id, name, public_key, fingerprint, created_at, last_verified_at, trust_level, revocation_status FROM trusted_publishers WHERE id = ?")
            .map_err(|err| format!("failed to prepare statement: {err}"))?;

        let mut rows = stmt
            .query(params![id])
            .map_err(|err| format!("query failed: {err}"))?;

        if let Some(row) = rows.next().map_err(|err| format!("row retrieve failed: {err}"))? {
            Ok(Some(PublisherRecord {
                id: row.get(0).unwrap(),
                name: row.get(1).unwrap(),
                public_key: row.get(2).unwrap(),
                fingerprint: row.get(3).unwrap(),
                created_at: row.get(4).unwrap(),
                last_verified_at: row.get(5).unwrap(),
                trust_level: row.get(6).unwrap(),
                revocation_status: row.get(7).unwrap(),
            }))
        } else {
            Ok(None)
        }
    }

    pub fn get_publisher_by_fingerprint(&self, fingerprint: &str) -> Result<Option<PublisherRecord>, String> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare("SELECT id, name, public_key, fingerprint, created_at, last_verified_at, trust_level, revocation_status FROM trusted_publishers WHERE fingerprint = ?")
            .map_err(|err| format!("failed to prepare statement: {err}"))?;

        let mut rows = stmt
            .query(params![fingerprint])
            .map_err(|err| format!("query failed: {err}"))?;

        if let Some(row) = rows.next().map_err(|err| format!("row retrieve failed: {err}"))? {
            Ok(Some(PublisherRecord {
                id: row.get(0).unwrap(),
                name: row.get(1).unwrap(),
                public_key: row.get(2).unwrap(),
                fingerprint: row.get(3).unwrap(),
                created_at: row.get(4).unwrap(),
                last_verified_at: row.get(5).unwrap(),
                trust_level: row.get(6).unwrap(),
                revocation_status: row.get(7).unwrap(),
            }))
        } else {
            Ok(None)
        }
    }

    pub fn revoke_publisher(&self, id: &str) -> Result<(), String> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE trusted_publishers SET revocation_status = 1 WHERE id = ?",
            params![id],
        )
        .map_err(|err| format!("failed to revoke publisher: {err}"))?;
        Ok(())
    }

    pub fn remove_publisher(&self, id: &str) -> Result<(), String> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM trusted_publishers WHERE id = ?", params![id])
            .map_err(|err| format!("failed to delete publisher: {err}"))?;
        Ok(())
    }

    pub fn list(&self) -> Result<Vec<PublisherRecord>, String> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare("SELECT id, name, public_key, fingerprint, created_at, last_verified_at, trust_level, revocation_status FROM trusted_publishers")
            .map_err(|err| format!("failed to prepare statement: {err}"))?;

        let rows = stmt
            .query_map([], |row| {
                Ok(PublisherRecord {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    public_key: row.get(2)?,
                    fingerprint: row.get(3)?,
                    created_at: row.get(4)?,
                    last_verified_at: row.get(5)?,
                    trust_level: row.get(6)?,
                    revocation_status: row.get(7)?,
                })
            })
            .map_err(|err| format!("query failed: {err}"))?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r.map_err(|err| err.to_string())?);
        }
        Ok(list)
    }
}
