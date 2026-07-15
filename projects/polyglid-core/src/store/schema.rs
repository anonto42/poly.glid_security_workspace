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
    },
    Migration {
        version: 2,
        sqls: &[
            r#"
            CREATE TABLE IF NOT EXISTS trusted_publishers (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                public_key TEXT NOT NULL,
                fingerprint TEXT NOT NULL UNIQUE,
                created_at INTEGER NOT NULL,
                last_verified_at INTEGER NOT NULL,
                trust_level TEXT NOT NULL,
                revocation_status INTEGER NOT NULL DEFAULT 0
            );
            "#,
            r#"
            CREATE TABLE IF NOT EXISTS plugin_signatures (
                plugin_id TEXT PRIMARY KEY,
                algorithm TEXT NOT NULL,
                key_id TEXT NOT NULL,
                signature TEXT NOT NULL,
                fingerprint TEXT NOT NULL,
                verified_at INTEGER NOT NULL,
                status TEXT NOT NULL CHECK(status IN ('Verified', 'Invalid', 'Missing', 'UnknownPublisher', 'Revoked')),
                FOREIGN KEY(plugin_id) REFERENCES plugins(id) ON DELETE CASCADE
            );
            "#,
            r#"
            CREATE TABLE IF NOT EXISTS audit_logs (
                id TEXT PRIMARY KEY,
                event_type TEXT NOT NULL,
                plugin_id TEXT,
                details TEXT NOT NULL,
                timestamp INTEGER NOT NULL
            );
            "#,
            r#"
            CREATE INDEX IF NOT EXISTS idx_audit_logs_plugin ON audit_logs(plugin_id);
            "#,
            r#"
            DROP TABLE IF EXISTS permissions;
            "#,
            r#"
            CREATE TABLE IF NOT EXISTS permissions (
                id TEXT PRIMARY KEY,
                plugin_id TEXT NOT NULL,
                capability TEXT NOT NULL,
                scope TEXT NOT NULL,
                workspace TEXT NOT NULL,
                decision TEXT NOT NULL CHECK(decision IN ('Allow', 'Deny')),
                timestamp INTEGER NOT NULL,
                expiration INTEGER,
                FOREIGN KEY(plugin_id) REFERENCES plugins(id) ON DELETE CASCADE
            );
            "#,
            r#"
            CREATE INDEX IF NOT EXISTS idx_permissions_plugin ON permissions(plugin_id);
            "#,
        ],
    },
    Migration {
        version: 3,
        sqls: &[
            r#"
            CREATE TABLE IF NOT EXISTS publisher_profiles (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                display_name TEXT NOT NULL,
                bio TEXT,
                website TEXT,
                public_key TEXT NOT NULL,
                fingerprint TEXT NOT NULL UNIQUE,
                verified INTEGER NOT NULL DEFAULT 0,
                plugin_count INTEGER NOT NULL DEFAULT 0,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            );
            "#,
            r#"
            CREATE INDEX IF NOT EXISTS idx_publisher_profiles_name ON publisher_profiles(name);
            "#,
            r#"
            CREATE TABLE IF NOT EXISTS marketplace_packages (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                display_name TEXT NOT NULL,
                version TEXT NOT NULL,
                description TEXT NOT NULL,
                author TEXT NOT NULL,
                publisher_id TEXT,
                categories TEXT NOT NULL DEFAULT '[]',
                tags TEXT NOT NULL DEFAULT '[]',
                capabilities TEXT NOT NULL DEFAULT '[]',
                download_url TEXT NOT NULL,
                checksum TEXT NOT NULL,
                download_count INTEGER NOT NULL DEFAULT 0,
                rating_avg REAL NOT NULL DEFAULT 0.0,
                rating_count INTEGER NOT NULL DEFAULT 0,
                license TEXT NOT NULL DEFAULT 'MIT',
                repository_url TEXT,
                documentation_url TEXT,
                published_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL,
                is_featured INTEGER NOT NULL DEFAULT 0,
                is_verified INTEGER NOT NULL DEFAULT 0,
                FOREIGN KEY(publisher_id) REFERENCES publisher_profiles(id) ON DELETE SET NULL
            );
            "#,
            r#"
            CREATE INDEX IF NOT EXISTS idx_marketplace_packages_name ON marketplace_packages(name);
            "#,
            r#"
            CREATE INDEX IF NOT EXISTS idx_marketplace_packages_publisher ON marketplace_packages(publisher_id);
            "#,
            r#"
            CREATE INDEX IF NOT EXISTS idx_marketplace_packages_featured ON marketplace_packages(is_featured);
            "#,
            r#"
            CREATE TABLE IF NOT EXISTS marketplace_ratings (
                id TEXT PRIMARY KEY,
                package_id TEXT NOT NULL,
                rating INTEGER NOT NULL CHECK(rating BETWEEN 1 AND 5),
                review TEXT,
                reviewer_id TEXT,
                created_at INTEGER NOT NULL,
                FOREIGN KEY(package_id) REFERENCES marketplace_packages(id) ON DELETE CASCADE
            );
            "#,
            r#"
            CREATE INDEX IF NOT EXISTS idx_marketplace_ratings_package ON marketplace_ratings(package_id);
            "#,
            r#"
            CREATE TABLE IF NOT EXISTS marketplace_installs (
                id TEXT PRIMARY KEY,
                package_id TEXT NOT NULL,
                plugin_id TEXT,
                installed_at INTEGER NOT NULL,
                FOREIGN KEY(package_id) REFERENCES marketplace_packages(id) ON DELETE CASCADE,
                FOREIGN KEY(plugin_id) REFERENCES plugins(id) ON DELETE SET NULL
            );
            "#,
        ],
    },
    Migration {
        version: 4,
        sqls: &[
            r#"
            CREATE TABLE IF NOT EXISTS users (
                id TEXT PRIMARY KEY,
                username TEXT UNIQUE NOT NULL,
                password_hash TEXT NOT NULL,
                salt TEXT NOT NULL,
                role TEXT NOT NULL CHECK(role IN ('Owner', 'Editor', 'Viewer')),
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            );
            "#,
            r#"
            CREATE INDEX IF NOT EXISTS idx_users_username ON users(username);
            "#,
            r#"
            CREATE TABLE IF NOT EXISTS teams (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            );
            "#,
            r#"
            CREATE TABLE IF NOT EXISTS team_members (
                team_id TEXT NOT NULL,
                user_id TEXT NOT NULL,
                role TEXT NOT NULL CHECK(role IN ('Owner', 'Editor', 'Viewer')),
                created_at INTEGER NOT NULL,
                PRIMARY KEY (team_id, user_id),
                FOREIGN KEY(team_id) REFERENCES teams(id) ON DELETE CASCADE,
                FOREIGN KEY(user_id) REFERENCES users(id) ON DELETE CASCADE
            );
            "#,
            r#"
            CREATE INDEX IF NOT EXISTS idx_team_members_user ON team_members(user_id);
            "#,
            r#"
            CREATE TABLE IF NOT EXISTS user_tokens (
                token TEXT PRIMARY KEY,
                user_id TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                expires_at INTEGER NOT NULL,
                FOREIGN KEY(user_id) REFERENCES users(id) ON DELETE CASCADE
            );
            "#,
            r#"
            CREATE INDEX IF NOT EXISTS idx_user_tokens_user ON user_tokens(user_id);
            "#,
        ],
    },
    Migration {
        version: 5,
        sqls: &[
            r#"
            CREATE TABLE IF NOT EXISTS workspaces (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                root_path TEXT NOT NULL UNIQUE,
                is_active INTEGER NOT NULL DEFAULT 0 CHECK(is_active IN (0, 1)),
                discovery_state TEXT NOT NULL DEFAULT 'idle'
                    CHECK(discovery_state IN ('idle', 'loading', 'ready', 'error')),
                last_error TEXT,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL,
                last_opened_at INTEGER
            );
            "#,
            r#"
            CREATE UNIQUE INDEX IF NOT EXISTS idx_workspaces_single_active
                ON workspaces(is_active) WHERE is_active = 1;
            "#,
            r#"
            CREATE TABLE IF NOT EXISTS projects (
                id TEXT PRIMARY KEY,
                workspace_id TEXT NOT NULL,
                name TEXT NOT NULL,
                path TEXT NOT NULL,
                kind TEXT NOT NULL,
                archived INTEGER NOT NULL DEFAULT 0 CHECK(archived IN (0, 1)),
                excluded INTEGER NOT NULL DEFAULT 0 CHECK(excluded IN (0, 1)),
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL,
                UNIQUE(workspace_id, path),
                FOREIGN KEY(workspace_id) REFERENCES workspaces(id) ON DELETE CASCADE
            );
            "#,
            r#"
            CREATE INDEX IF NOT EXISTS idx_projects_workspace
                ON projects(workspace_id, archived, name);
            "#,
        ],
    },
];
