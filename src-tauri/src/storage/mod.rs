use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectStorage {
    pub local_project_root: Option<PathBuf>,
    pub s3: Option<S3Storage>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct S3Storage {
    pub bucket: String,
    pub prefix: String,
    pub region: String,
}

impl ProjectStorage {
    pub fn local_only(local_project_root: impl Into<PathBuf>) -> Self {
        Self {
            local_project_root: Some(local_project_root.into()),
            s3: None,
        }
    }

    pub fn resolve_local_root(&self, fallback: &Path) -> PathBuf {
        self.local_project_root
            .clone()
            .unwrap_or_else(|| fallback.to_path_buf())
    }
}
