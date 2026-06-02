use crate::error::{ConfigError, Result};
use crate::tables::factory_recycler_const::FRecyclerConst;
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Clone, Deserialize)]
pub struct FRecyclerConstAssets {
    pub data: FRecyclerConst,
}

impl FRecyclerConstAssets {
    pub(super) fn load(tables_dir: &Path) -> Result<Self> {
        let path = tables_dir.join("FacRecyclerConst.json");
        let contents = std::fs::read_to_string(&path).map_err(|e| ConfigError::ReadFile {
            path: path.clone(),
            source: e,
        })?;

        let table: FRecyclerConst =
            serde_json::from_str(&contents).map_err(|e| ConfigError::ParseJson {
                path: path.clone(),
                source: e,
            })?;

        Ok(Self { data: table })
    }
}
