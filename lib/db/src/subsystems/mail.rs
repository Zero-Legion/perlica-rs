use crate::error::Result;
use crate::subsystems::prune;
use perlica_logic::mail::{MailManager, StoredMail};
use sqlx::{Row, Sqlite, SqlitePool, Transaction};

pub(crate) async fn load(pool: &SqlitePool, uid: &str) -> Result<MailManager> {
    let mail_rows = sqlx::query(
        "SELECT mail_id, mail_type, is_read, is_attachment_got,
                send_time, expire_time, template_id, title, content,
                sender_name, sender_icon
         FROM beyond_mails
         WHERE uid = ?1
         ORDER BY mail_id",
    )
    .bind(uid)
    .fetch_all(pool)
    .await?;

    let mut mgr = MailManager::new();
    let mut max_id: u64 = 0;
    for r in mail_rows {
        let mail_id: i64 = r.try_get("mail_id")?;
        let mail_type: i64 = r.try_get("mail_type")?;
        let is_read: i64 = r.try_get("is_read")?;
        let is_attachment_got: i64 = r.try_get("is_attachment_got")?;
        let send_time: i64 = r.try_get("send_time")?;
        let expire_time: i64 = r.try_get("expire_time")?;
        let template_id: String = r.try_get("template_id")?;
        let title: String = r.try_get("title")?;
        let content: String = r.try_get("content")?;
        let sender_name: String = r.try_get("sender_name")?;
        let sender_icon: String = r.try_get("sender_icon")?;

        let attachment_rows = sqlx::query(
            "SELECT item_template_id, item_count FROM beyond_mail_attachments
             WHERE uid = ?1 AND mail_id = ?2
             ORDER BY item_index",
        )
        .bind(uid)
        .bind(mail_id)
        .fetch_all(pool)
        .await?;

        let items: Vec<(String, i64)> = attachment_rows
            .into_iter()
            .map(|ar| {
                let tid: String = ar.try_get("item_template_id")?;
                let count: i64 = ar.try_get("item_count")?;
                Ok::<_, sqlx::Error>((tid, count))
            })
            .collect::<std::result::Result<Vec<_>, _>>()?;

        let id_u = mail_id as u64;
        max_id = max_id.max(id_u);
        mgr.mails.push(StoredMail {
            mail_id: id_u,
            mail_type: mail_type as i32,
            is_read: is_read != 0,
            is_attachment_got: is_attachment_got != 0,
            send_time,
            expire_time,
            template_id,
            title,
            content,
            sender_name,
            sender_icon,
            items,
        });
    }
    mgr.set_next_id(max_id + 1);
    Ok(mgr)
}

pub(crate) async fn write(
    tx: &mut Transaction<'_, Sqlite>,
    uid: &str,
    mgr: &MailManager,
) -> Result<()> {
    for mail in &mgr.mails {
        sqlx::query(
            "INSERT INTO beyond_mails (
                uid, mail_id, mail_type, is_read, is_attachment_got,
                send_time, expire_time, template_id, title, content,
                sender_name, sender_icon
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
             ON CONFLICT(uid, mail_id) DO UPDATE SET
                mail_type         = excluded.mail_type,
                is_read           = excluded.is_read,
                is_attachment_got = excluded.is_attachment_got,
                send_time         = excluded.send_time,
                expire_time       = excluded.expire_time,
                template_id       = excluded.template_id,
                title             = excluded.title,
                content           = excluded.content,
                sender_name       = excluded.sender_name,
                sender_icon       = excluded.sender_icon",
        )
        .bind(uid)
        .bind(mail.mail_id as i64)
        .bind(mail.mail_type as i64)
        .bind(if mail.is_read { 1i64 } else { 0i64 })
        .bind(if mail.is_attachment_got { 1i64 } else { 0i64 })
        .bind(mail.send_time)
        .bind(mail.expire_time)
        .bind(&mail.template_id)
        .bind(&mail.title)
        .bind(&mail.content)
        .bind(&mail.sender_name)
        .bind(&mail.sender_icon)
        .execute(&mut **tx)
        .await?;

        for (idx, (template_id, count)) in mail.items.iter().enumerate() {
            sqlx::query(
                "INSERT INTO beyond_mail_attachments
                    (uid, mail_id, item_index, item_template_id, item_count)
                 VALUES (?1, ?2, ?3, ?4, ?5)
                 ON CONFLICT(uid, mail_id, item_index) DO UPDATE SET
                    item_template_id = excluded.item_template_id,
                    item_count       = excluded.item_count",
            )
            .bind(uid)
            .bind(mail.mail_id as i64)
            .bind(idx as i64)
            .bind(template_id)
            .bind(*count)
            .execute(&mut **tx)
            .await?;
        }

        // Attachments form a contiguous Vec - drop any index beyond
        // the current length. Far cheaper than building an IN-list.
        prune::prune_tail(
            tx,
            "beyond_mail_attachments",
            uid,
            "mail_id",
            mail.mail_id as i64,
            "item_index",
            mail.items.len(),
        )
        .await?;
    }

    let mail_keep: Vec<i64> = mgr.mails.iter().map(|m| m.mail_id as i64).collect();
    prune::prune_i64_pk(tx, "beyond_mails", uid, "mail_id", &mail_keep).await?;
    // Cascading FK from beyond_mail_attachments handles its rows.

    Ok(())
}
