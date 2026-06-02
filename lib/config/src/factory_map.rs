use serde::Deserialize;

use crate::error::{ConfigError, Result};
use crate::tables::factory_map::{RegionItem, RegionTable};
use std::path::Path;

#[derive(Debug, Clone, Deserialize)]
pub struct FRegionAssets {
    data: RegionTable,
}

impl FRegionAssets {
    pub(super) fn load(tables_dir: &Path) -> Result<Self> {
        let path = tables_dir.join("FactoryMapTable.json");
        let contents = std::fs::read_to_string(&path).map_err(|e| ConfigError::ReadFile {
            path: path.clone(),
            source: e,
        })?;

        let table: RegionTable =
            serde_json::from_str(&contents).map_err(|e| ConfigError::ParseJson {
                path: path.clone(),
                source: e,
            })?;

        Ok(Self { data: table })
    }

    pub fn get(&self, region: &str, level: u32) -> Option<&RegionItem> {
        self.data
            .get(region)
            .and_then(|r| r.list.iter().find(|i| i.level == level))
    }
}
