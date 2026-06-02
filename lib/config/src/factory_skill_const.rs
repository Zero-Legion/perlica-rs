use serde::Deserialize;

use crate::error::{ConfigError, Result};
use crate::tables::factory_skill_const::FSkillConst;
use std::path::Path;

#[derive(Debug, Clone, Deserialize)]
pub struct FSkillConstAssets {
    pub data: FSkillConst,
}

impl FSkillConstAssets {
    pub(super) fn load(tables_dir: &Path) -> Result<Self> {
        let path = tables_dir.join("FacSkillConst.json");
        let contents = std::fs::read_to_string(&path).map_err(|e| ConfigError::ReadFile {
            path: path.clone(),
            source: e,
        })?;

        let table: FSkillConst =
            serde_json::from_str(&contents).map_err(|e| ConfigError::ParseJson {
                path: path.clone(),
                source: e,
            })?;

        Ok(Self { data: table })
    }
}
