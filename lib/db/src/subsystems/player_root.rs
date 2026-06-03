use crate::error::{DbError, Result};
use perlica_logic::character::char_bag::CharBag;
use perlica_logic::player::WorldState;
use perlica_logic::scene::RevivalMode;
use sqlx::{Row as _, Sqlite, SqlitePool, Transaction};

pub(crate) struct Row {
    pub role_level: i32,
    pub role_exp: i32,
    pub last_scene: String,
    pub pos_x: f32,
    pub pos_y: f32,
    pub pos_z: f32,
    pub rot_x: f32,
    pub rot_y: f32,
    pub rot_z: f32,
    pub curr_team_index: u32,
    pub track_mission_id: String,
    pub revival_mode: RevivalMode,
    pub checkpoint_scene: Option<String>,
    pub checkpoint_x: Option<f32>,
    pub checkpoint_y: Option<f32>,
    pub checkpoint_z: Option<f32>,
    pub weapon_next_inst_id: u64,
    pub gem_next_inst_id: u64,
    pub equip_next_inst_id: u64,
    pub mail_next_id: u64,
    pub updated_at: i64,
}

pub(crate) struct Loaded {
    pub role_level: i32,
    pub role_exp: i32,
    pub last_scene: String,
    pub pos_x: f32,
    pub pos_y: f32,
    pub pos_z: f32,
    pub rot_x: f32,
    pub rot_y: f32,
    pub rot_z: f32,
    pub curr_team_index: u32,
    pub track_mission_id: String,
    pub revival_mode: RevivalMode,
    pub checkpoint_scene: Option<String>,
    pub checkpoint_x: Option<f32>,
    pub checkpoint_y: Option<f32>,
    pub checkpoint_z: Option<f32>,
    pub weapon_next_inst_id: u64,
    pub gem_next_inst_id: u64,
    pub equip_next_inst_id: u64,
    pub mail_next_id: u64,
}

pub(crate) async fn load(pool: &SqlitePool, uid: &str) -> Result<Option<Loaded>> {
    let row = sqlx::query(
        "SELECT role_level, role_exp, last_scene,
                pos_x, pos_y, pos_z, rot_x, rot_y, rot_z,
                curr_team_index, track_mission_id, revival_mode,
                checkpoint_scene, checkpoint_x, checkpoint_y, checkpoint_z,
                weapon_next_inst_id, gem_next_inst_id, equip_next_inst_id,
                mail_next_id
         FROM beyond_players WHERE uid = ?1",
    )
    .bind(uid)
    .fetch_optional(pool)
    .await?;

    let Some(row) = row else {
        return Ok(None);
    };

    let revival_mode_i: i64 = row.try_get("revival_mode")?;
    let revival_mode = match revival_mode_i {
        0 => RevivalMode::Default,
        1 => RevivalMode::RepatriatePoint,
        2 => RevivalMode::CheckPoint,
        other => {
            return Err(DbError::Corruption {
                uid: uid.to_string(),
                what: "revival_mode",
                reason: format!("unknown discriminant {other}"),
            });
        }
    };

    Ok(Some(Loaded {
        role_level: row.try_get::<i64, _>("role_level")? as i32,
        role_exp: row.try_get::<i64, _>("role_exp")? as i32,
        last_scene: row.try_get("last_scene")?,
        pos_x: row.try_get::<f64, _>("pos_x")? as f32,
        pos_y: row.try_get::<f64, _>("pos_y")? as f32,
        pos_z: row.try_get::<f64, _>("pos_z")? as f32,
        rot_x: row.try_get::<f64, _>("rot_x")? as f32,
        rot_y: row.try_get::<f64, _>("rot_y")? as f32,
        rot_z: row.try_get::<f64, _>("rot_z")? as f32,
        curr_team_index: row.try_get::<i64, _>("curr_team_index")? as u32,
        track_mission_id: row.try_get("track_mission_id")?,
        revival_mode,
        checkpoint_scene: row.try_get("checkpoint_scene")?,
        checkpoint_x: row
            .try_get::<Option<f64>, _>("checkpoint_x")?
            .map(|v| v as f32),
        checkpoint_y: row
            .try_get::<Option<f64>, _>("checkpoint_y")?
            .map(|v| v as f32),
        checkpoint_z: row
            .try_get::<Option<f64>, _>("checkpoint_z")?
            .map(|v| v as f32),
        weapon_next_inst_id: row.try_get::<i64, _>("weapon_next_inst_id")? as u64,
        gem_next_inst_id: row.try_get::<i64, _>("gem_next_inst_id")? as u64,
        equip_next_inst_id: row.try_get::<i64, _>("equip_next_inst_id")? as u64,
        mail_next_id: row.try_get::<i64, _>("mail_next_id")? as u64,
    }))
}

pub(crate) async fn write(tx: &mut Transaction<'_, Sqlite>, uid: &str, row: Row) -> Result<()> {
    sqlx::query(
        "INSERT INTO beyond_players (
            uid, role_level, role_exp, last_scene,
            pos_x, pos_y, pos_z, rot_x, rot_y, rot_z,
            curr_team_index, track_mission_id, revival_mode,
            checkpoint_scene, checkpoint_x, checkpoint_y, checkpoint_z,
            weapon_next_inst_id, gem_next_inst_id, equip_next_inst_id,
            mail_next_id, updated_at
         ) VALUES (
            ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13,
            ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22
         )
         ON CONFLICT(uid) DO UPDATE SET
            role_level          = excluded.role_level,
            role_exp            = excluded.role_exp,
            last_scene          = excluded.last_scene,
            pos_x               = excluded.pos_x,
            pos_y               = excluded.pos_y,
            pos_z               = excluded.pos_z,
            rot_x               = excluded.rot_x,
            rot_y               = excluded.rot_y,
            rot_z               = excluded.rot_z,
            curr_team_index     = excluded.curr_team_index,
            track_mission_id    = excluded.track_mission_id,
            revival_mode        = excluded.revival_mode,
            checkpoint_scene    = excluded.checkpoint_scene,
            checkpoint_x        = excluded.checkpoint_x,
            checkpoint_y        = excluded.checkpoint_y,
            checkpoint_z        = excluded.checkpoint_z,
            weapon_next_inst_id = excluded.weapon_next_inst_id,
            gem_next_inst_id    = excluded.gem_next_inst_id,
            equip_next_inst_id  = excluded.equip_next_inst_id,
            mail_next_id        = excluded.mail_next_id,
            updated_at          = excluded.updated_at",
    )
    .bind(uid)
    .bind(row.role_level as i64)
    .bind(row.role_exp as i64)
    .bind(&row.last_scene)
    .bind(row.pos_x as f64)
    .bind(row.pos_y as f64)
    .bind(row.pos_z as f64)
    .bind(row.rot_x as f64)
    .bind(row.rot_y as f64)
    .bind(row.rot_z as f64)
    .bind(row.curr_team_index as i64)
    .bind(&row.track_mission_id)
    .bind(row.revival_mode as i32 as i64)
    .bind(row.checkpoint_scene.as_deref())
    .bind(row.checkpoint_x.map(|v| v as f64))
    .bind(row.checkpoint_y.map(|v| v as f64))
    .bind(row.checkpoint_z.map(|v| v as f64))
    .bind(row.weapon_next_inst_id as i64)
    .bind(row.gem_next_inst_id as i64)
    .bind(row.equip_next_inst_id as i64)
    .bind(row.mail_next_id as i64)
    .bind(row.updated_at)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

pub(crate) async fn update_world(
    tx: &mut Transaction<'_, Sqlite>,
    uid: &str,
    world: &WorldState,
) -> Result<()> {
    sqlx::query(
        "UPDATE beyond_players SET
            role_level = ?2, role_exp = ?3, last_scene = ?4,
            pos_x = ?5, pos_y = ?6, pos_z = ?7,
            rot_x = ?8, rot_y = ?9, rot_z = ?10,
            updated_at = ?11
         WHERE uid = ?1",
    )
    .bind(uid)
    .bind(world.role_level as i64)
    .bind(world.role_exp as i64)
    .bind(&world.last_scene)
    .bind(world.pos_x as f64)
    .bind(world.pos_y as f64)
    .bind(world.pos_z as f64)
    .bind(world.rot_x as f64)
    .bind(world.rot_y as f64)
    .bind(world.rot_z as f64)
    .bind(common::time::now_ms() as i64)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

pub(crate) async fn update_char_bag_scalars(
    tx: &mut Transaction<'_, Sqlite>,
    uid: &str,
    bag: &CharBag,
) -> Result<()> {
    sqlx::query(
        "UPDATE beyond_players SET
            curr_team_index     = ?2,
            weapon_next_inst_id = ?3,
            gem_next_inst_id    = ?4,
            equip_next_inst_id  = ?5,
            updated_at          = ?6
         WHERE uid = ?1",
    )
    .bind(uid)
    .bind(bag.meta.curr_team_index as i64)
    .bind(bag.item_manager.weapons.next_inst_id() as i64)
    .bind(bag.item_manager.gems.next_inst_id() as i64)
    .bind(bag.item_manager.equips.next_inst_id() as i64)
    .bind(common::time::now_ms() as i64)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

pub(crate) async fn update_track_mission(
    tx: &mut Transaction<'_, Sqlite>,
    uid: &str,
    track: &str,
) -> Result<()> {
    sqlx::query("UPDATE beyond_players SET track_mission_id = ?2, updated_at = ?3 WHERE uid = ?1")
        .bind(uid)
        .bind(track)
        .bind(common::time::now_ms() as i64)
        .execute(&mut **tx)
        .await?;
    Ok(())
}

pub(crate) async fn update_mail_next_id(
    tx: &mut Transaction<'_, Sqlite>,
    uid: &str,
    next_id: u64,
) -> Result<()> {
    sqlx::query("UPDATE beyond_players SET mail_next_id = ?2, updated_at = ?3 WHERE uid = ?1")
        .bind(uid)
        .bind(next_id as i64)
        .bind(common::time::now_ms() as i64)
        .execute(&mut **tx)
        .await?;
    Ok(())
}
