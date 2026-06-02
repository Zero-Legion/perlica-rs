use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct FRecyclerConst {
    #[serde(rename = "recTempStorageLength")]
    pub rec_temp_storage_length: i32,
    #[serde(rename = "recBasicGenerateTime")]
    pub rec_basic_generate_time: i32,
    #[serde(rename = "recRoundNeedValue")]
    pub rec_round_need_value: i32,
}
