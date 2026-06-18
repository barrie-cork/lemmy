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
            chair_id        TEXT,   -- current chair pseudonym
            queue_state     TEXT,   -- FIFO raised-hand queue (JSON text)
            recording_config TEXT,  -- recording knobs (JSON text)
            PRIMARY KEY (case_id, room_type)
        );"
    )?;
    // Idempotent guards for DBs created before M3 (CREATE TABLE IF NOT EXISTS
    // does NOT add columns to an existing table). SQLite has no ADD COLUMN IF
    // NOT EXISTS — swallow the "duplicate column name" error per column.
    for col in ["chair_id", "queue_state", "recording_config"] {
        let stmt = format!("ALTER TABLE bridge_room ADD COLUMN {col} TEXT");
        match conn.execute(&stmt, []) {
            Ok(_) => {}
            Err(rusqlite::Error::SqliteFailure(_, Some(msg)))
                if msg.contains("duplicate column name") => {}
            Err(e) => return Err(e),
        }
    }
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

/// Idempotency watermark for restart-safe provisioning (last-seen
/// governance_log row per case). Not yet called — the log-tail consumer that
/// advances it lands with the m2 "resume without duplicate Room::Created"
/// work. Retained as the schema-aligned writer until then.
#[allow(dead_code)]
pub fn set_watermark(conn: &Connection, case_id: i64, row_id: i64) -> Result<()> {
    conn.execute(
        "UPDATE bridge_room SET last_seen_governance_log_row_id = ?1 WHERE case_id = ?2",
        params![row_id, case_id],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn column_names(conn: &Connection) -> Vec<String> {
        let mut stmt = conn
            .prepare("PRAGMA table_info(bridge_room)")
            .expect("PRAGMA ok");
        stmt.query_map([], |row| row.get::<_, String>(1))
            .expect("query ok")
            .map(|r| r.expect("row ok"))
            .collect()
    }

    #[test]
    fn fresh_db_has_rtc_columns() {
        let conn = open(":memory:").expect("open fresh db");
        let cols = column_names(&conn);
        assert!(cols.contains(&"chair_id".to_string()), "missing chair_id");
        assert!(cols.contains(&"queue_state".to_string()), "missing queue_state");
        assert!(cols.contains(&"recording_config".to_string()), "missing recording_config");
    }

    #[test]
    fn old_shape_db_gets_rtc_columns_and_second_open_is_idempotent() {
        // Simulate a DB created before M3 (6-column schema, no RTC columns).
        let conn = Connection::open(":memory:").expect("open raw db");
        conn.execute_batch(
            "CREATE TABLE bridge_room (
                case_id   INTEGER NOT NULL,
                room_type TEXT    NOT NULL,
                matrix_room_id TEXT,
                lifecycle_state TEXT,
                last_seen_governance_log_row_id INTEGER,
                reveal_state TEXT,
                PRIMARY KEY (case_id, room_type)
            );",
        )
        .expect("create old-shape table");

        // Simulate open() ALTER loop on the old-shape connection.
        // (Each Connection::open(":memory:") creates a new DB, so we exercise the
        // ALTER loop directly on this connection rather than calling open() again.)
        for col in ["chair_id", "queue_state", "recording_config"] {
            let stmt = format!("ALTER TABLE bridge_room ADD COLUMN {col} TEXT");
            match conn.execute(&stmt, []) {
                Ok(_) => {}
                Err(rusqlite::Error::SqliteFailure(_, Some(ref msg)))
                    if msg.contains("duplicate column name") => {}
                Err(e) => panic!("unexpected ALTER error: {e}"),
            }
        }
        let cols = column_names(&conn);
        assert!(cols.contains(&"chair_id".to_string()), "chair_id missing after ALTER");
        assert!(cols.contains(&"queue_state".to_string()), "queue_state missing after ALTER");
        assert!(cols.contains(&"recording_config".to_string()), "recording_config missing after ALTER");

        // Running the ALTER loop a second time must be idempotent (swallows duplicate-column errors).
        for col in ["chair_id", "queue_state", "recording_config"] {
            let stmt = format!("ALTER TABLE bridge_room ADD COLUMN {col} TEXT");
            match conn.execute(&stmt, []) {
                Ok(_) => {}
                Err(rusqlite::Error::SqliteFailure(_, Some(ref msg)))
                    if msg.contains("duplicate column name") => {}
                Err(e) => panic!("second ALTER pass — unexpected error: {e}"),
            }
        }
        // Column count must not grow (no duplicates).
        let cols_after = column_names(&conn);
        assert_eq!(cols.len(), cols_after.len(), "column count changed on second open");
    }
}
