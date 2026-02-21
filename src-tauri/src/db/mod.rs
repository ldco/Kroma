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
