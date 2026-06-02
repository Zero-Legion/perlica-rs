use crate::net::NetContext;
use perlica_logic::enums::UnlockSystemType;
use perlica_proto::{AreaUnlockInfo, ScSyncAllRoleScene, ScSyncAllUnlock};
use tracing::{debug, error};

/// Pushes `ScSyncAllUnlock` with every system unlocked. Called once during login.
pub async fn push_unlocks(ctx: &mut NetContext<'_>) -> bool {
    let msg = ScSyncAllUnlock {
        unlock_systems: UnlockSystemType::all(),
    };
    debug!(
        "unlocks: uid={}, count={}",
        ctx.player.uid,
        msg.unlock_systems.len()
    );
    if let Err(e) = ctx.notify(msg).await {
        error!("unlocks push failed: uid={}, error={}", ctx.player.uid, e);
        return false;
    }
    true
}

//for now tho idk what its used for anyways
pub async fn all_role_sync(ctx: &mut NetContext<'_>) -> bool {
    let msg = ScSyncAllRoleScene {
        submit_ether_count: 0,
        submit_ether_level: 1,
        unlock_area_info: vec![AreaUnlockInfo {
            scene_id: "map01_lv001".to_string(),
            unlock_area_id: vec!["areaId101".to_string()],
        }],
    };
    if let Err(e) = ctx.notify(msg).await {
        error!("push failed: uid={}, error={}", ctx.player.uid, e);
        return false;
    }
    true
}
