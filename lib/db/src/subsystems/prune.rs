use crate::error::Result;
use sqlx::{Sqlite, Transaction};

const CHUNK_SIZE: usize = 500;

pub(crate) async fn prune_str_pk(
    tx: &mut Transaction<'_, Sqlite>,
    table: &str,
    uid: &str,
    pk_col: &str,
    keep: &[&str],
) -> Result<()> {
    if keep.is_empty() {
        sqlx::query(&format!("DELETE FROM {table} WHERE uid = ?1"))
            .bind(uid)
            .execute(&mut **tx)
            .await?;
        return Ok(());
    }
    for chunk in keep.chunks(CHUNK_SIZE) {
        let placeholders = comma_qs(chunk.len());
        let sql =
            format!("DELETE FROM {table} WHERE uid = ?1 AND {pk_col} NOT IN ({placeholders})");
        let mut q = sqlx::query(&sql).bind(uid);
        for v in chunk {
            q = q.bind(*v);
        }
        q.execute(&mut **tx).await?;
    }
    Ok(())
}

pub(crate) async fn prune_i64_pk(
    tx: &mut Transaction<'_, Sqlite>,
    table: &str,
    uid: &str,
    pk_col: &str,
    keep: &[i64],
) -> Result<()> {
    if keep.is_empty() {
        sqlx::query(&format!("DELETE FROM {table} WHERE uid = ?1"))
            .bind(uid)
            .execute(&mut **tx)
            .await?;
        return Ok(());
    }
    for chunk in keep.chunks(CHUNK_SIZE) {
        let placeholders = comma_qs(chunk.len());
        let sql =
            format!("DELETE FROM {table} WHERE uid = ?1 AND {pk_col} NOT IN ({placeholders})");
        let mut q = sqlx::query(&sql).bind(uid);
        for v in chunk {
            q = q.bind(*v);
        }
        q.execute(&mut **tx).await?;
    }
    Ok(())
}

pub(crate) async fn prune_tail(
    tx: &mut Transaction<'_, Sqlite>,
    table: &str,
    uid: &str,
    parent_col: &str,
    parent_value: i64,
    child_col: &str,
    keep_count: usize,
) -> Result<()> {
    sqlx::query(&format!(
        "DELETE FROM {table} WHERE uid = ?1 AND {parent_col} = ?2 AND {child_col} >= ?3"
    ))
    .bind(uid)
    .bind(parent_value)
    .bind(keep_count as i64)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

fn comma_qs(n: usize) -> String {
    debug_assert!(n > 0);
    let mut s = String::with_capacity(n * 3);
    s.push('?');
    for _ in 1..n {
        s.push_str(", ?");
    }
    s
}
