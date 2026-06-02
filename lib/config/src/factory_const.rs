use serde::Deserialize;

use crate::error::{ConfigError, Result};
use crate::tables::factory_const::FConst;
use std::path::Path;

#[derive(Debug, Clone, Deserialize)]
pub struct FConstAssets {
    pub data: FConst,
}

impl FConstAssets {
    pub(super) fn load(tables_dir: &Path) -> Result<Self> {
        let path = tables_dir.join("FactoryConst.json");
        let contents = std::fs::read_to_string(&path).map_err(|e| ConfigError::ReadFile {
            path: path.clone(),
            source: e,
        })?;

        let table: FConst =
            serde_json::from_str(&contents).map_err(|e| ConfigError::ParseJson {
                path: path.clone(),
                source: e,
            })?;

        Ok(Self { data: table })
    }
}
