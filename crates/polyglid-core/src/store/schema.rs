pub struct Migration {
    pub version: i32,
    pub sqls: &'static [&'static str],
}

pub const MIGRATIONS: &[Migration] = &[
    Migration {
        version: 1,
        sqls: &[
            r#"
            CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                scope TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            );
            "#,
            r#"
            CREATE TABLE IF NOT EXISTS plugins (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                version TEXT NOT NULL,
                author TEXT NOT NULL,
                description TEXT NOT NULL,
                capabilities TEXT NOT NULL,
                checksum TEXT NOT NULL,
                status TEXT NOT NULL,
                source TEXT NOT NULL,
                file_size INTEGER NOT NULL,
                path TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL,
                last_used_at INTEGER
            );
            "#,
            r#"
            CREATE INDEX IF NOT EXISTS idx_plugins_status ON plugins(status);
            "#,
            r#"
            CREATE TABLE IF NOT EXISTS targets (
                name TEXT PRIMARY KEY,
                group_name TEXT,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            );
            "#,
            r#"
            CREATE INDEX IF NOT EXISTS idx_targets_name ON targets(name);
            "#,
            r#"
            CREATE TABLE IF NOT EXISTS permissions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                plugin_id TEXT,
                capability TEXT NOT NULL,
                scope TEXT,
                created_at INTEGER NOT NULL,
                FOREIGN KEY(plugin_id) REFERENCES plugins(id) ON DELETE CASCADE
            );
            "#,
            r#"
            CREATE INDEX IF NOT EXISTS idx_permissions_plugin ON permissions(plugin_id);
            "#,
            r#"
            CREATE TABLE IF NOT EXISTS execution_history (
                job_id TEXT PRIMARY KEY,
                plugin_id TEXT NOT NULL,
                target TEXT NOT NULL,
                state TEXT NOT NULL,
                started_at INTEGER NOT NULL,
                duration_ms INTEGER NOT NULL,
                error_message TEXT,
                fuel_consumed INTEGER NOT NULL,
                created_at INTEGER NOT NULL,
                FOREIGN KEY(plugin_id) REFERENCES plugins(id) ON DELETE CASCADE
            );
            "#,
            r#"
            CREATE INDEX IF NOT EXISTS idx_execution_plugin ON execution_history(plugin_id);
            "#,
            r#"
            CREATE INDEX IF NOT EXISTS idx_execution_started ON execution_history(started_at);
            "#,
            r#"
            CREATE TABLE IF NOT EXISTS reports (
                id TEXT PRIMARY KEY,
                job_id TEXT NOT NULL,
                plugin_id TEXT NOT NULL,
                target TEXT NOT NULL,
                summary TEXT NOT NULL,
                issues TEXT NOT NULL,
                filepath TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                FOREIGN KEY(job_id) REFERENCES execution_history(job_id) ON DELETE CASCADE,
                FOREIGN KEY(plugin_id) REFERENCES plugins(id) ON DELETE CASCADE
            );
            "#,
            r#"
            CREATE INDEX IF NOT EXISTS idx_reports_job ON reports(job_id);
            "#,
        ],
    }
];
