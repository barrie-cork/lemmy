// SQLite cache for `app_local_id ↔ brehon_actor_id` mapping.
//
// ADR-015 (load-bearing): `brehon_actor_id` stored here is the pseudonym UUID
// string (`actor_pseudonym.pseudonym`), NEVER a Lemmy `person_id`, username,
// or email address. The incoming value comes from `LinkConfirmRequest.brehon_actor_id`
// (which Brehon echoes from the claim — already pseudonymised at source).
//
// MIRROR: followed bridge_room.rs open/upsert/lookup shape exactly.

use rusqlite::{Connection, Result, params};

pub fn open(path: &str) -> Result<Connection> {
    let conn = Connection::open(path)?;
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS app_actor_link (
            app_id          TEXT    NOT NULL,
            app_local_id    TEXT    NOT NULL,
            brehon_actor_id TEXT    NOT NULL,
            linked_at       INTEGER,
            revoked_at      INTEGER,
            PRIMARY KEY (app_id, app_local_id)
        );"
    )?;
    Ok(conn)
}

pub fn upsert(conn: &Connection, app_id: &str, app_local_id: &str, brehon_actor_id: &str) -> Result<()> {
    conn.execute(
        "INSERT INTO app_actor_link (app_id, app_local_id, brehon_actor_id, linked_at)
         VALUES (?1, ?2, ?3, strftime('%s', 'now'))
         ON CONFLICT(app_id, app_local_id) DO UPDATE SET
           brehon_actor_id = excluded.brehon_actor_id,
           linked_at       = excluded.linked_at,
           revoked_at      = NULL",
        params![app_id, app_local_id, brehon_actor_id],
    )?;
    Ok(())
}

#[allow(dead_code)]
pub fn lookup(conn: &Connection, app_id: &str, app_local_id: &str) -> Result<Option<String>> {
    let mut stmt = conn.prepare(
        "SELECT brehon_actor_id FROM app_actor_link WHERE app_id = ?1 AND app_local_id = ?2 AND revoked_at IS NULL"
    )?;
    let mut rows = stmt.query(params![app_id, app_local_id])?;
    if let Some(row) = rows.next()? {
        Ok(row.get(0)?)
    } else {
        Ok(None)
    }
}

#[allow(dead_code)]
pub fn revoke(conn: &Connection, app_id: &str, app_local_id: &str) -> Result<()> {
    conn.execute(
        "UPDATE app_actor_link SET revoked_at = strftime('%s', 'now') WHERE app_id = ?1 AND app_local_id = ?2",
        params![app_id, app_local_id],
    )?;
    Ok(())
}
