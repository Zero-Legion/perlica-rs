use crate::net::NetContext;
use perlica_db::Persistable;
use perlica_logic::character::char_bag::handle_weapon_breakthrough;
use perlica_logic::item::WALLET_GOLD_AMOUNT;
use perlica_proto::{
    CsWeaponBreakthrough, MoneyInfo, ScItemBagSyncModify, ScSyncWallet, ScWeaponBreakthrough,
};
use std::collections::HashMap;
use tracing::{debug, error, info, warn};

/// Advances breakthrough level by one. Weapon must be at its current level cap.
pub async fn on_cs_weapon_breakthrough(
    ctx: &mut NetContext<'_>,
    req: CsWeaponBreakthrough,
) -> ScWeaponBreakthrough {
    debug!(
        "Weapon breakthrough request: uid={}, weapon_id={}",
        ctx.player.uid, req.weaponid
    );

    match handle_weapon_breakthrough(&mut ctx.player.char_bag, req.weaponid, ctx.assets) {
        Ok(result) => {
            let depot_modify = result.consumed.build_depot_map();

            if !depot_modify.is_empty() {
                let _ = ctx
                    .notify(ScItemBagSyncModify {
                        depot: depot_modify,
                        bag: None,
                        factory_depot: None,
                        cannot_destroy: HashMap::new(),
                        use_blackboard: None,
                        is_new: false,
                    })
                    .await;
            }

            // Consume gold from wallet. The wallet is currently not persisted
            // (hardcoded to 9_999_999 on login), but we still send the sync
            // so the client UI updates correctly.
            if result.gold_cost > 0 {
                let remaining_gold = WALLET_GOLD_AMOUNT.saturating_sub(result.gold_cost as u64);
                let _ = ctx
                    .notify(ScSyncWallet {
                        money_list: vec![MoneyInfo {
                            id: "item_gold".to_string(),
                            amount: remaining_gold,
                        }],
                    })
                    .await;
            }

            info!(
                "Weapon breakthrough success: uid={}, weapon_id={}, new_lv={}, gold_cost={}",
                ctx.player.uid, req.weaponid, result.response.breakthrough_lv, result.gold_cost
            );

            if let Err(e) = ctx.player.char_bag.persist(&ctx.player.uid, ctx.db).await {
                warn!(
                    "Failed to persist char_bag after weapon breakthrough: uid={}, error={}",
                    ctx.player.uid, e
                );
            }

            result.response
        }
        Err(e) => {
            error!(
                "Weapon breakthrough failed: uid={}, weapon_id={}, error={:?}",
                ctx.player.uid, req.weaponid, e
            );
            ScWeaponBreakthrough {
                weaponid: req.weaponid,
                breakthrough_lv: 0,
            }
        }
    }
}
