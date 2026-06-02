use crate::error::{ConfigError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemBundle {
    pub id: String,
    #[serde(default)]
    pub count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewardEntry {
    #[serde(rename = "rewardId")]
    pub reward_id: String,
    #[serde(rename = "itemBundles", default)]
    pub item_bundles: Vec<ItemBundle>,
}

#[derive(Debug, Default)]
pub struct RewardAssets {
    by_id: HashMap<String, RewardEntry>,
}

impl RewardAssets {
    pub(crate) fn load(tables_dir: &Path) -> Result<Self> {
        let path = tables_dir.join("RewardTable.json");
        if !path.exists() {
            tracing::warn!(
                "RewardTable.json not found at {} - chests will not grant any rewards",
                path.display()
            );
            return Ok(Self::default());
        }
        let contents = std::fs::read_to_string(&path).map_err(|e| ConfigError::ReadFile {
            path: path.clone(),
            source: e,
        })?;
        let by_id: HashMap<String, RewardEntry> =
            serde_json::from_str(&contents).map_err(|e| ConfigError::ParseJson {
                path: path.clone(),
                source: e,
            })?;
        Ok(Self { by_id })
    }

    #[inline]
    pub fn get(&self, reward_id: &str) -> Option<&RewardEntry> {
        self.by_id.get(reward_id)
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.by_id.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.by_id.is_empty()
    }
}
