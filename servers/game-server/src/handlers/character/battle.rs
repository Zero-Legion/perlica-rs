//! Battle-state synchronisation for characters.
//!
//! HP / ultimate-SP updates arrive on every damage tick during combat,
//! which is the highest-frequency mutator in the game loop. We do
//! **not** persist here — `update_battle_info` flips the dirty flag
//! through `get_char_by_objid_mut`, and the session-level periodic
//! flusher (`PERSIST_INTERVAL` in `net::session`) takes care of
//! writing it to disk every ~30 s. A clean disconnect also flushes,
//! so the only loss window is an unclean crash.

use crate::net::NetContext;
use perlica_proto::CsCharSetBattleInfo;
use tracing::{debug, warn};

pub async fn on_cs_char_set_battle_info(ctx: &mut NetContext<'_>, req: CsCharSetBattleInfo) {
    debug!(
        "Battle info update: objid={}, has_battle_info={}",
        req.objid,
        req.battle_info.is_some()
    );
    if let Some(bi) = &req.battle_info {
        ctx.player
            .char_bag
            .update_battle_info(req.objid, bi.hp, bi.ultimatesp);
    } else {
        warn!(
            "Battle info update ignored: missing data for objid={}",
            req.objid
        );
    }
    // no `.persist()` here - the dirty flag is enough.
}
