use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct FSkillConst {
    #[serde(rename = "isOriginiumOre")]
    pub is_originium_ore: Vec<String>,
    #[serde(rename = "isNotOriginiumOre")]
    pub is_not_originium_ore: Vec<String>,
}
