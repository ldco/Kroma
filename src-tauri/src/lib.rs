pub mod api;
pub mod contract;
pub mod db;
pub mod pipeline;
pub mod storage;
pub mod worker;

use std::path::PathBuf;

pub fn default_openapi_contract_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../openapi/backend-api.openapi.yaml")
}
