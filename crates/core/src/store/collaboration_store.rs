use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbUser {
    pub id: String,
    pub username: String,
    pub password_hash: String,
    pub salt: String,
    pub role: String, // 'Owner', 'Editor', 'Viewer'
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbTeam {
    pub id: String,
    pub name: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbTeamMember {
    pub team_id: String,
    pub user_id: String,
    pub role: String,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbUserToken {
    pub token: String,
    pub user_id: String,
    pub created_at: i64,
    pub expires_at: i64,
}

pub struct CollaborationStore {
    conn: Arc<Mutex<Connection>>,
}

impl CollaborationStore {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    // ── User Management ────────────────────────────────────────────────────────

    pub fn create_user(&self, user: &DbUser) -> Result<(), String> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO users (id, username, password_hash, salt, role, created_at, updated_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                user.id,
                user.username,
                user.password_hash,
                user.salt,
                user.role,
                user.created_at,
                user.updated_at,
            ],
        )
        .map(|_| ())
        .map_err(|e| format!("create_user failed: {e}"))
    }

    pub fn get_user(&self, id: &str) -> Result<Option<DbUser>, String> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, username, password_hash, salt, role, created_at, updated_at FROM users WHERE id = ?1",
        ).map_err(|e| format!("prepare get_user: {e}"))?;
        let result: Result<Vec<DbUser>, _> = stmt
            .query_map([id], |row| {
                Ok(DbUser {
                    id: row.get(0)?,
                    username: row.get(1)?,
                    password_hash: row.get(2)?,
                    salt: row.get(3)?,
                    role: row.get(4)?,
                    created_at: row.get(5)?,
                    updated_at: row.get(6)?,
                })
            })
            .map_err(|e| format!("query get_user: {e}"))?
            .collect();
        Ok(result
            .map_err(|e| format!("row get_user: {e}"))?
            .into_iter()
            .next())
    }

    pub fn get_user_by_username(&self, username: &str) -> Result<Option<DbUser>, String> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, username, password_hash, salt, role, created_at, updated_at FROM users WHERE username = ?1",
        ).map_err(|e| format!("prepare get_user_by_username: {e}"))?;
        let result: Result<Vec<DbUser>, _> = stmt
            .query_map([username], |row| {
                Ok(DbUser {
                    id: row.get(0)?,
                    username: row.get(1)?,
                    password_hash: row.get(2)?,
                    salt: row.get(3)?,
                    role: row.get(4)?,
                    created_at: row.get(5)?,
                    updated_at: row.get(6)?,
                })
            })
            .map_err(|e| format!("query get_user_by_username: {e}"))?
            .collect();
        Ok(result
            .map_err(|e| format!("row get_user_by_username: {e}"))?
            .into_iter()
            .next())
    }

    pub fn count_users(&self) -> Result<i64, String> {
        let conn = self.conn.lock().unwrap();
        conn.query_row("SELECT COUNT(*) FROM users", [], |row| row.get(0))
            .map_err(|e| format!("count_users failed: {e}"))
    }

    pub fn list_users(&self) -> Result<Vec<DbUser>, String> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, username, password_hash, salt, role, created_at, updated_at FROM users ORDER BY username ASC",
        ).map_err(|e| format!("prepare list_users: {e}"))?;
        let result: Result<Vec<DbUser>, _> = stmt
            .query_map([], |row| {
                Ok(DbUser {
                    id: row.get(0)?,
                    username: row.get(1)?,
                    password_hash: row.get(2)?,
                    salt: row.get(3)?,
                    role: row.get(4)?,
                    created_at: row.get(5)?,
                    updated_at: row.get(6)?,
                })
            })
            .map_err(|e| format!("query list_users: {e}"))?
            .collect();
        result.map_err(|e| format!("row list_users: {e}"))
    }

    // ── Token / Session Management ──────────────────────────────────────────────

    pub fn create_token(&self, token: &DbUserToken) -> Result<(), String> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO user_tokens (token, user_id, created_at, expires_at) VALUES (?1, ?2, ?3, ?4)",
            params![token.token, token.user_id, token.created_at, token.expires_at],
        )
        .map(|_| ())
        .map_err(|e| format!("create_token failed: {e}"))
    }

    pub fn validate_token(&self, token: &str, now: i64) -> Result<Option<DbUser>, String> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT u.id, u.username, u.password_hash, u.salt, u.role, u.created_at, u.updated_at \
             FROM user_tokens t \
             JOIN users u ON t.user_id = u.id \
             WHERE t.token = ?1 AND t.expires_at > ?2",
        ).map_err(|e| format!("prepare validate_token: {e}"))?;
        let result: Result<Vec<DbUser>, _> = stmt
            .query_map(params![token, now], |row| {
                Ok(DbUser {
                    id: row.get(0)?,
                    username: row.get(1)?,
                    password_hash: row.get(2)?,
                    salt: row.get(3)?,
                    role: row.get(4)?,
                    created_at: row.get(5)?,
                    updated_at: row.get(6)?,
                })
            })
            .map_err(|e| format!("query validate_token: {e}"))?
            .collect();
        Ok(result
            .map_err(|e| format!("row validate_token: {e}"))?
            .into_iter()
            .next())
    }

    pub fn revoke_token(&self, token: &str) -> Result<(), String> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM user_tokens WHERE token = ?1", [token])
            .map(|_| ())
            .map_err(|e| format!("revoke_token failed: {e}"))
    }

    // ── Team Management ────────────────────────────────────────────────────────

    pub fn create_team(&self, team: &DbTeam) -> Result<(), String> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO teams (id, name, created_at, updated_at) VALUES (?1, ?2, ?3, ?4)",
            params![team.id, team.name, team.created_at, team.updated_at],
        )
        .map(|_| ())
        .map_err(|e| format!("create_team failed: {e}"))
    }

    pub fn list_teams(&self) -> Result<Vec<DbTeam>, String> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare("SELECT id, name, created_at, updated_at FROM teams ORDER BY name ASC")
            .map_err(|e| format!("prepare list_teams: {e}"))?;
        let result: Result<Vec<DbTeam>, _> = stmt
            .query_map([], |row| {
                Ok(DbTeam {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    created_at: row.get(2)?,
                    updated_at: row.get(3)?,
                })
            })
            .map_err(|e| format!("query list_teams: {e}"))?
            .collect();
        result.map_err(|e| format!("row list_teams: {e}"))
    }

    pub fn add_team_member(&self, member: &DbTeamMember) -> Result<(), String> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO team_members (team_id, user_id, role, created_at) VALUES (?1, ?2, ?3, ?4) \
             ON CONFLICT(team_id, user_id) DO UPDATE SET role = excluded.role",
            params![member.team_id, member.user_id, member.role, member.created_at],
        )
        .map(|_| ())
        .map_err(|e| format!("add_team_member failed: {e}"))
    }

    pub fn remove_team_member(&self, team_id: &str, user_id: &str) -> Result<(), String> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "DELETE FROM team_members WHERE team_id = ?1 AND user_id = ?2",
            params![team_id, user_id],
        )
        .map(|_| ())
        .map_err(|e| format!("remove_team_member failed: {e}"))
    }

    pub fn list_team_members(&self, team_id: &str) -> Result<Vec<(DbUser, String)>, String> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT u.id, u.username, u.password_hash, u.salt, u.role, u.created_at, u.updated_at, m.role \
             FROM team_members m \
             JOIN users u ON m.user_id = u.id \
             WHERE m.team_id = ?1 ORDER BY u.username ASC",
        ).map_err(|e| format!("prepare list_team_members: {e}"))?;
        let result: Result<Vec<(DbUser, String)>, _> = stmt
            .query_map([team_id], |row| {
                let user = DbUser {
                    id: row.get(0)?,
                    username: row.get(1)?,
                    password_hash: row.get(2)?,
                    salt: row.get(3)?,
                    role: row.get(4)?,
                    created_at: row.get(5)?,
                    updated_at: row.get(6)?,
                };
                let member_role: String = row.get(7)?;
                Ok((user, member_role))
            })
            .map_err(|e| format!("query list_team_members: {e}"))?
            .collect();
        result.map_err(|e| format!("row list_team_members: {e}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::WorkspaceStore;

    fn make_store() -> WorkspaceStore {
        WorkspaceStore::new(std::path::Path::new(":memory:")).expect("in-memory store")
    }

    #[test]
    fn test_collaboration_users_and_tokens() {
        let ws = make_store();
        let store = ws.collaboration();

        let user = DbUser {
            id: "u1".to_string(),
            username: "alice".to_string(),
            password_hash: "hash123".to_string(),
            salt: "salt123".to_string(),
            role: "Owner".to_string(),
            created_at: 0,
            updated_at: 0,
        };

        store.create_user(&user).unwrap();
        assert_eq!(store.count_users().unwrap(), 1);

        let retrieved = store.get_user("u1").unwrap().unwrap();
        assert_eq!(retrieved.username, "alice");
        assert_eq!(retrieved.role, "Owner");

        let ret_by_name = store.get_user_by_username("alice").unwrap().unwrap();
        assert_eq!(ret_by_name.id, "u1");

        // Token operations
        let token = DbUserToken {
            token: "session_tok".to_string(),
            user_id: "u1".to_string(),
            created_at: 100,
            expires_at: 200,
        };
        store.create_token(&token).unwrap();

        // Valid token before expiration
        let valid_user = store.validate_token("session_tok", 150).unwrap().unwrap();
        assert_eq!(valid_user.username, "alice");

        // Expired token
        let expired_user = store.validate_token("session_tok", 250).unwrap();
        assert!(expired_user.is_none());

        // Revoke token
        store.revoke_token("session_tok").unwrap();
        let revoked_user = store.validate_token("session_tok", 150).unwrap();
        assert!(revoked_user.is_none());
    }

    #[test]
    fn test_teams_and_members() {
        let ws = make_store();
        let store = ws.collaboration();

        let user = DbUser {
            id: "u2".to_string(),
            username: "bob".to_string(),
            password_hash: "pw".to_string(),
            salt: "salt".to_string(),
            role: "Editor".to_string(),
            created_at: 0,
            updated_at: 0,
        };
        store.create_user(&user).unwrap();

        let team = DbTeam {
            id: "t1".to_string(),
            name: "SecOps".to_string(),
            created_at: 0,
            updated_at: 0,
        };
        store.create_team(&team).unwrap();

        let teams = store.list_teams().unwrap();
        assert_eq!(teams.len(), 1);
        assert_eq!(teams[0].name, "SecOps");

        // Add member
        let member = DbTeamMember {
            team_id: "t1".to_string(),
            user_id: "u2".to_string(),
            role: "Editor".to_string(),
            created_at: 0,
        };
        store.add_team_member(&member).unwrap();

        let members = store.list_team_members("t1").unwrap();
        assert_eq!(members.len(), 1);
        assert_eq!(members[0].0.username, "bob");
        assert_eq!(members[0].1, "Editor");

        // Remove member
        store.remove_team_member("t1", "u2").unwrap();
        let members_empty = store.list_team_members("t1").unwrap();
        assert_eq!(members_empty.len(), 0);
    }
}
