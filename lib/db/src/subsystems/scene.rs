use crate::error::Result;
use perlica_logic::scene::{CheckpointInfo, RevivalMode};
use sqlx::{Sqlite, Transaction};

pub(crate) async fn update_checkpoint(
    tx: &mut Transaction<'_, Sqlite>,
    uid: &str,
    checkpoint: Option<&CheckpointInfo>,
    revival_mode: RevivalMode,
) -> Result<()> {
    sqlx::query(
        "UPDATE beyond_players SET
            checkpoint_scene = ?2,
            checkpoint_x     = ?3,
            checkpoint_y     = ?4,
            checkpoint_z     = ?5,
            revival_mode     = ?6,
            updated_at       = ?7
         WHERE uid = ?1",
    )
    .bind(uid)
    .bind(checkpoint.map(|c| c.scene_name.as_str()))
    .bind(checkpoint.map(|c| c.pos_x as f64))
    .bind(checkpoint.map(|c| c.pos_y as f64))
    .bind(checkpoint.map(|c| c.pos_z as f64))
    .bind(revival_mode as i32 as i64)
    .bind(common::time::now_ms() as i64)
    .execute(&mut **tx)
    .await?;
    Ok(())
}
