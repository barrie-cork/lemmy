use rusqlite::{Connection, Result, params};

pub fn open(path: &str) -> Result<Connection> {
    let conn = Connection::open(path)?;
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS bridge_room (
            case_id   INTEGER NOT NULL,
            room_type TEXT    NOT NULL,
            matrix_room_id TEXT,
            lifecycle_state TEXT,
            last_seen_governance_log_row_id INTEGER,
            reveal_state TEXT,
            PRIMARY KEY (case_id, room_type)
        );"
    )?;
    Ok(conn)
}

pub fn lookup(conn: &Connection, case_id: i64, room_type: &str) -> Result<Option<String>> {
    let mut stmt = conn.prepare(
        "SELECT matrix_room_id FROM bridge_room WHERE case_id = ?1 AND room_type = ?2"
    )?;
    let mut rows = stmt.query(params![case_id, room_type])?;
    if let Some(row) = rows.next()? {
        Ok(row.get(0)?)
    } else {
        Ok(None)
    }
}

pub fn lookup_by_case(conn: &Connection, case_id: i64) -> Result<Vec<(String, String)>> {
    let mut stmt = conn.prepare(
        "SELECT room_type, matrix_room_id FROM bridge_room WHERE case_id = ?1 AND matrix_room_id IS NOT NULL"
    )?;
    let rows = stmt.query_map(params![case_id], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    })?;
    rows.collect()
}

pub fn upsert(
    conn: &Connection,
    case_id: i64,
    room_type: &str,
    matrix_room_id: &str,
    lifecycle_state: &str,
    reveal_state: Option<&str>,
) -> Result<()> {
    conn.execute(
        "INSERT INTO bridge_room (case_id, room_type, matrix_room_id, lifecycle_state, reveal_state)
         VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT(case_id, room_type) DO UPDATE SET
           matrix_room_id = excluded.matrix_room_id,
           lifecycle_state = excluded.lifecycle_state,
           reveal_state = excluded.reveal_state",
        params![case_id, room_type, matrix_room_id, lifecycle_state, reveal_state],
    )?;
    Ok(())
}

pub fn set_watermark(conn: &Connection, case_id: i64, row_id: i64) -> Result<()> {
    conn.execute(
        "UPDATE bridge_room SET last_seen_governance_log_row_id = ?1 WHERE case_id = ?2",
        params![row_id, case_id],
    )?;
    Ok(())
}
