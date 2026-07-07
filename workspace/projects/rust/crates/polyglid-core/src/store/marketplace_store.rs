use std::sync::{Arc, Mutex};
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbPublisherProfile {
    pub id: String,
    pub name: String,
    pub display_name: String,
    pub bio: Option<String>,
    pub website: Option<String>,
    pub public_key: String,
    pub fingerprint: String,
    pub verified: bool,
    pub plugin_count: i64,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbMarketplacePackage {
    pub id: String,
    pub name: String,
    pub display_name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub publisher_id: Option<String>,
    pub categories: String,
    pub tags: String,
    pub capabilities: String,
    pub download_url: String,
    pub checksum: String,
    pub download_count: i64,
    pub rating_avg: f64,
    pub rating_count: i64,
    pub license: String,
    pub repository_url: Option<String>,
    pub documentation_url: Option<String>,
    pub published_at: i64,
    pub updated_at: i64,
    pub is_featured: bool,
    pub is_verified: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbMarketplaceRating {
    pub id: String,
    pub package_id: String,
    pub rating: i64,
    pub review: Option<String>,
    pub reviewer_id: Option<String>,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbMarketplaceInstall {
    pub id: String,
    pub package_id: String,
    pub plugin_id: Option<String>,
    pub installed_at: i64,
}

pub struct MarketplaceStore {
    conn: Arc<Mutex<Connection>>,
}

impl MarketplaceStore {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    // ── Publisher Profiles ────────────────────────────────────────────────────

    pub fn add_publisher(&self, profile: &DbPublisherProfile) -> Result<(), String> {
        let conn = self.conn.lock().unwrap();
        let now = now_secs();
        conn.execute(
            "INSERT OR IGNORE INTO publisher_profiles \
             (id, name, display_name, bio, website, public_key, fingerprint, verified, plugin_count, created_at, updated_at) \
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)",
            params![
                profile.id, profile.name, profile.display_name, profile.bio, profile.website,
                profile.public_key, profile.fingerprint, profile.verified as i64,
                profile.plugin_count, now, now,
            ],
        )
        .map(|_| ()).map_err(|e| format!("add_publisher: {e}"))
    }

    pub fn list_publishers(&self) -> Result<Vec<DbPublisherProfile>, String> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id,name,display_name,bio,website,public_key,fingerprint,verified,plugin_count,created_at,updated_at \
             FROM publisher_profiles ORDER BY name ASC",
        ).map_err(|e| format!("prepare: {e}"))?;
        let result: Result<Vec<DbPublisherProfile>, _> = stmt.query_map([], |row| {
            Ok(DbPublisherProfile {
                id: row.get(0)?, name: row.get(1)?, display_name: row.get(2)?,
                bio: row.get(3)?, website: row.get(4)?, public_key: row.get(5)?,
                fingerprint: row.get(6)?, verified: row.get::<_, i64>(7)? != 0,
                plugin_count: row.get(8)?, created_at: row.get(9)?, updated_at: row.get(10)?,
            })
        }).map_err(|e| format!("query: {e}"))?.collect();
        result.map_err(|e| format!("row: {e}"))
    }

    pub fn get_publisher(&self, id: &str) -> Result<Option<DbPublisherProfile>, String> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id,name,display_name,bio,website,public_key,fingerprint,verified,plugin_count,created_at,updated_at \
             FROM publisher_profiles WHERE id = ?1",
        ).map_err(|e| format!("prepare: {e}"))?;
        let result: Result<Vec<DbPublisherProfile>, _> = stmt.query_map([id], |row| {
            Ok(DbPublisherProfile {
                id: row.get(0)?, name: row.get(1)?, display_name: row.get(2)?,
                bio: row.get(3)?, website: row.get(4)?, public_key: row.get(5)?,
                fingerprint: row.get(6)?, verified: row.get::<_, i64>(7)? != 0,
                plugin_count: row.get(8)?, created_at: row.get(9)?, updated_at: row.get(10)?,
            })
        }).map_err(|e| format!("query: {e}"))?.collect();
        Ok(result.map_err(|e: rusqlite::Error| format!("row: {e}"))?.into_iter().next())
    }

    // ── Marketplace Packages ──────────────────────────────────────────────────

    pub fn publish_package(&self, pkg: &DbMarketplacePackage) -> Result<(), String> {
        let conn = self.conn.lock().unwrap();
        let now = now_secs();
        conn.execute(
            "INSERT OR REPLACE INTO marketplace_packages \
             (id,name,display_name,version,description,author,publisher_id,categories,tags,capabilities, \
              download_url,checksum,download_count,rating_avg,rating_count,license,repository_url, \
              documentation_url,published_at,updated_at,is_featured,is_verified) \
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19,?20,?21,?22)",
            params![
                pkg.id, pkg.name, pkg.display_name, pkg.version, pkg.description, pkg.author,
                pkg.publisher_id, pkg.categories, pkg.tags, pkg.capabilities,
                pkg.download_url, pkg.checksum, pkg.download_count, pkg.rating_avg, pkg.rating_count,
                pkg.license, pkg.repository_url, pkg.documentation_url, now, now,
                pkg.is_featured as i64, pkg.is_verified as i64,
            ],
        ).map(|_| ()).map_err(|e| format!("publish_package: {e}"))
    }

    fn collect_packages(stmt: &mut rusqlite::Statement, p: impl rusqlite::Params) -> Result<Vec<DbMarketplacePackage>, String> {
        let result: Result<Vec<DbMarketplacePackage>, _> = stmt.query_map(p, |row| {
            Ok(DbMarketplacePackage {
                id: row.get(0)?, name: row.get(1)?, display_name: row.get(2)?,
                version: row.get(3)?, description: row.get(4)?, author: row.get(5)?,
                publisher_id: row.get(6)?, categories: row.get(7)?, tags: row.get(8)?,
                capabilities: row.get(9)?, download_url: row.get(10)?, checksum: row.get(11)?,
                download_count: row.get(12)?, rating_avg: row.get(13)?, rating_count: row.get(14)?,
                license: row.get(15)?, repository_url: row.get(16)?, documentation_url: row.get(17)?,
                published_at: row.get(18)?, updated_at: row.get(19)?,
                is_featured: row.get::<_, i64>(20)? != 0,
                is_verified: row.get::<_, i64>(21)? != 0,
            })
        }).map_err(|e| format!("query: {e}"))?.collect();
        result.map_err(|e| format!("row: {e}"))
    }

    const PKG_COLS: &'static str = "id,name,display_name,version,description,author,publisher_id,categories,tags,capabilities,\
        download_url,checksum,download_count,rating_avg,rating_count,license,repository_url,documentation_url,\
        published_at,updated_at,is_featured,is_verified FROM marketplace_packages";

    pub fn list_featured(&self) -> Result<Vec<DbMarketplacePackage>, String> {
        let conn = self.conn.lock().unwrap();
        let sql = format!("SELECT {} ORDER BY is_featured DESC, download_count DESC, rating_avg DESC LIMIT 20", Self::PKG_COLS);
        let mut stmt = conn.prepare(&sql).map_err(|e| format!("prepare: {e}"))?;
        Self::collect_packages(&mut stmt, [])
    }

    pub fn search_packages(&self, query: &str, category: Option<&str>) -> Result<Vec<DbMarketplacePackage>, String> {
        let conn = self.conn.lock().unwrap();
        let search = format!("%{}%", query.to_lowercase());
        let order = "ORDER BY is_featured DESC, download_count DESC";
        if let Some(cat) = category {
            let cat_pattern = format!("%{}%", cat);
            let sql = format!(
                "SELECT {} WHERE (lower(name) LIKE ?1 OR lower(description) LIKE ?1 OR lower(tags) LIKE ?1) AND categories LIKE ?2 {}",
                Self::PKG_COLS, order
            );
            let mut stmt = conn.prepare(&sql).map_err(|e| format!("prepare: {e}"))?;
            Self::collect_packages(&mut stmt, params![search, cat_pattern])
        } else {
            let sql = format!(
                "SELECT {} WHERE lower(name) LIKE ?1 OR lower(description) LIKE ?1 OR lower(tags) LIKE ?1 {}",
                Self::PKG_COLS, order
            );
            let mut stmt = conn.prepare(&sql).map_err(|e| format!("prepare: {e}"))?;
            Self::collect_packages(&mut stmt, params![search])
        }
    }

    pub fn get_package(&self, id: &str) -> Result<Option<DbMarketplacePackage>, String> {
        let conn = self.conn.lock().unwrap();
        let sql = format!("SELECT {} WHERE id = ?1", Self::PKG_COLS);
        let mut stmt = conn.prepare(&sql).map_err(|e| format!("prepare: {e}"))?;
        Ok(Self::collect_packages(&mut stmt, [id])?.into_iter().next())
    }

    pub fn increment_download_count(&self, package_id: &str) -> Result<(), String> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE marketplace_packages SET download_count = download_count + 1 WHERE id = ?1",
            [package_id],
        ).map(|_| ()).map_err(|e| format!("increment_download_count: {e}"))
    }

    // ── Ratings ───────────────────────────────────────────────────────────────

    pub fn add_rating(&self, rating: &DbMarketplaceRating) -> Result<(), String> {
        let conn = self.conn.lock().unwrap();
        let now = now_secs();
        conn.execute(
            "INSERT INTO marketplace_ratings (id,package_id,rating,review,reviewer_id,created_at) \
             VALUES (?1,?2,?3,?4,?5,?6)",
            params![rating.id, rating.package_id, rating.rating, rating.review, rating.reviewer_id, now],
        ).map_err(|e| format!("add_rating: {e}"))?;
        conn.execute(
            "UPDATE marketplace_packages \
             SET rating_avg = (SELECT AVG(CAST(rating AS REAL)) FROM marketplace_ratings WHERE package_id = ?1), \
                 rating_count = (SELECT COUNT(*) FROM marketplace_ratings WHERE package_id = ?1), \
                 updated_at = ?2 \
             WHERE id = ?1",
            params![rating.package_id, now],
        ).map(|_| ()).map_err(|e| format!("update_avg: {e}"))
    }

    pub fn list_ratings(&self, package_id: &str) -> Result<Vec<DbMarketplaceRating>, String> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id,package_id,rating,review,reviewer_id,created_at \
             FROM marketplace_ratings WHERE package_id = ?1 ORDER BY created_at DESC",
        ).map_err(|e| format!("prepare: {e}"))?;
        let result: Result<Vec<DbMarketplaceRating>, _> = stmt.query_map([package_id], |row| {
            Ok(DbMarketplaceRating {
                id: row.get(0)?, package_id: row.get(1)?, rating: row.get(2)?,
                review: row.get(3)?, reviewer_id: row.get(4)?, created_at: row.get(5)?,
            })
        }).map_err(|e| format!("query: {e}"))?.collect();
        result.map_err(|e| format!("row: {e}"))
    }

    // ── Install Tracking ──────────────────────────────────────────────────────

    pub fn record_install(&self, install: &DbMarketplaceInstall) -> Result<(), String> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO marketplace_installs (id,package_id,plugin_id,installed_at) VALUES (?1,?2,?3,?4)",
            params![install.id, install.package_id, install.plugin_id, install.installed_at],
        ).map(|_| ()).map_err(|e| format!("record_install: {e}"))
    }

    pub fn list_installed_packages(&self) -> Result<Vec<DbMarketplaceInstall>, String> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id,package_id,plugin_id,installed_at FROM marketplace_installs ORDER BY installed_at DESC",
        ).map_err(|e| format!("prepare: {e}"))?;
        let result: Result<Vec<DbMarketplaceInstall>, _> = stmt.query_map([], |row| {
            Ok(DbMarketplaceInstall {
                id: row.get(0)?, package_id: row.get(1)?,
                plugin_id: row.get(2)?, installed_at: row.get(3)?,
            })
        }).map_err(|e| format!("query: {e}"))?.collect();
        result.map_err(|e| format!("row: {e}"))
    }
}

fn now_secs() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::WorkspaceStore;

    fn make_store() -> WorkspaceStore {
        WorkspaceStore::new(std::path::Path::new(":memory:")).expect("in-memory store")
    }

    fn sample_pkg(id: &str, name: &str) -> DbMarketplacePackage {
        DbMarketplacePackage {
            id: id.to_string(), name: name.to_string(), display_name: name.to_string(),
            version: "1.0.0".to_string(),
            description: format!("A {} security plugin for port scanning and recon", name),
            author: "test-author".to_string(), publisher_id: None,
            categories: r#"["security","recon"]"#.to_string(),
            tags: r#"["port-scan","recon"]"#.to_string(),
            capabilities: r#"["network:connect"]"#.to_string(),
            download_url: "https://registry.polyglid.dev/test.wasm".to_string(),
            checksum: "sha256:test".to_string(),
            download_count: 0, rating_avg: 0.0, rating_count: 0,
            license: "MIT".to_string(), repository_url: None, documentation_url: None,
            published_at: 0, updated_at: 0, is_featured: false, is_verified: false,
        }
    }

    #[test]
    fn test_publish_and_search() {
        let ws = make_store();
        let store = ws.marketplace();
        store.publish_package(&sample_pkg("pkg-001", "recon-probe")).unwrap();
        let results = store.search_packages("recon", None).unwrap();
        assert_eq!(results.len(), 1);
        let empty = store.search_packages("xyznonexistent", None).unwrap();
        assert_eq!(empty.len(), 0);
    }

    #[test]
    fn test_publisher_roundtrip() {
        let ws = make_store();
        let store = ws.marketplace();
        let profile = DbPublisherProfile {
            id: "pub-001".to_string(), name: "security-team".to_string(),
            display_name: "Security Team".to_string(), bio: None, website: None,
            public_key: "ed25519:abc".to_string(), fingerprint: "fp:001".to_string(),
            verified: true, plugin_count: 3, created_at: 0, updated_at: 0,
        };
        store.add_publisher(&profile).unwrap();
        let retrieved = store.get_publisher("pub-001").unwrap().unwrap();
        assert_eq!(retrieved.name, "security-team");
    }

    #[test]
    fn test_rating_avg() {
        let ws = make_store();
        let store = ws.marketplace();
        store.publish_package(&sample_pkg("pkg-002", "dns-enum")).unwrap();
        store.add_rating(&DbMarketplaceRating {
            id: "r1".to_string(), package_id: "pkg-002".to_string(),
            rating: 4, review: None, reviewer_id: None, created_at: 0,
        }).unwrap();
        store.add_rating(&DbMarketplaceRating {
            id: "r2".to_string(), package_id: "pkg-002".to_string(),
            rating: 2, review: None, reviewer_id: None, created_at: 0,
        }).unwrap();
        let pkg = store.get_package("pkg-002").unwrap().unwrap();
        assert_eq!(pkg.rating_count, 2);
        assert!((pkg.rating_avg - 3.0).abs() < 0.01);
    }

    #[test]
    fn test_download_count() {
        let ws = make_store();
        let store = ws.marketplace();
        store.publish_package(&sample_pkg("pkg-003", "ssl-checker")).unwrap();
        store.increment_download_count("pkg-003").unwrap();
        store.increment_download_count("pkg-003").unwrap();
        let pkg = store.get_package("pkg-003").unwrap().unwrap();
        assert_eq!(pkg.download_count, 2);
    }
}
