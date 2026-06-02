use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegionItem {
    pub id: String,
    pub level: u32,
    pub pos_x: i32,
    pub pos_y: i32,
    pub range_w: u32,
    pub range_h: u32,
    pub icon: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RegionData {
    pub list: Vec<RegionItem>,
}

pub type RegionTable = HashMap<String, RegionData>;
