use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FConst {
    pub tick_based_total_progress: u64,
    pub machine_crafter_buffer_slot_count: u32,
    pub pack_merge_distance: u32,
    pub pack_pick_distance: u32,
    pub pack_drop_distance: u32,
    pub trader_default_order_type: String,
    pub statistic_bucket_steps: u32,
    pub statistic_bucket_count: u32,
    pub grid_cargo_min_distance: u64,
    pub default_knowing_items: Vec<String>,
    pub workshop_upgrade_mission_id: String,
    pub manual_work_queue_length: u32,
    pub manual_work_unit_size: u32,
    pub trade_order_gen_speed: u32,
    pub max_statistic_bookmark_num: u32,
    pub travel_pole_qte_timing: u32,
    pub travel_pole_1_template_key: String,
    pub travel_pole_1_speed: u32,
    pub travel_pole_1_radius: u32,
    pub travel_pole_1_visible_radius: u32,
    pub travel_pole_2_template_key: String,
    pub travel_pole_2_speed: u32,
    pub travel_pole_2_radius: u32,
    pub travel_pole_2_visible_radius: u32,
    pub travel_pole_speed_line_effect_key: String,
    pub travel_pole_speed_line_effect_key_default_next: String,
    pub out_of_range_height: u32,
}
