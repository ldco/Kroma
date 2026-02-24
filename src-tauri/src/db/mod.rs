pub mod projects;

use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DbConfig {
    pub app_db_path: PathBuf,
}

impl DbConfig {
    pub fn new(app_db_path: impl Into<PathBuf>) -> Self {
        Self {
            app_db_path: app_db_path.into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct DbFacade {
    config: DbConfig,
}

impl DbFacade {
    pub fn new(config: DbConfig) -> Self {
        Self { config }
    }

    pub fn app_db_path(&self) -> &Path {
        self.config.app_db_path.as_path()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PostgresConfig {
    pub database_url: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DatabaseBackendConfig {
    Sqlite(DbConfig),
    Postgres(PostgresConfig),
}

pub fn resolve_backend_config(repo_root: &Path) -> DatabaseBackendConfig {
    let db_url = std::env::var("KROMA_BACKEND_DB_URL").ok();
    let sqlite_path = std::env::var("KROMA_BACKEND_DB").ok();
    select_backend_config(db_url.as_deref(), sqlite_path.as_deref(), repo_root)
}

fn select_backend_config(
    db_url: Option<&str>,
    sqlite_path: Option<&str>,
    repo_root: &Path,
) -> DatabaseBackendConfig {
    let db_url = db_url
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(str::to_string);
    if let Some(database_url) = db_url {
        return DatabaseBackendConfig::Postgres(PostgresConfig { database_url });
    }

    let sqlite_raw = sqlite_path
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| String::from("var/backend/app.db"));
    let sqlite_candidate = PathBuf::from(sqlite_raw);
    let sqlite_abs = if sqlite_candidate.is_absolute() {
        sqlite_candidate
    } else {
        repo_root.join(sqlite_candidate)
    };
    DatabaseBackendConfig::Sqlite(DbConfig::new(sqlite_abs))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selects_postgres_when_db_url_is_set() {
        let cfg = select_backend_config(
            Some("postgres://user:pw@localhost:5432/kroma"),
            Some("var/backend/app.db"),
            Path::new("/tmp/repo"),
        );
        match cfg {
            DatabaseBackendConfig::Postgres(pg) => {
                assert_eq!(pg.database_url, "postgres://user:pw@localhost:5432/kroma");
            }
            other => panic!("expected postgres backend, got {other:?}"),
        }
    }

    #[test]
    fn selects_sqlite_default_under_repo_root() {
        let cfg = select_backend_config(None, None, Path::new("/tmp/repo"));
        match cfg {
            DatabaseBackendConfig::Sqlite(sqlite) => {
                assert_eq!(
                    sqlite.app_db_path,
                    PathBuf::from("/tmp/repo/var/backend/app.db")
                );
            }
            other => panic!("expected sqlite backend, got {other:?}"),
        }
    }
}
