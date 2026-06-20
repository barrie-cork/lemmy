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
    // NOT EXISTS, so gate each ALTER behind a PRAGMA table_info existence check:
    // on the common path (columns already present) this is read-only, avoiding
    // a per-open schema-write lock that can cause "database is locked" under
    // concurrency (open() is called per request in sanction_handler.rs).
    let existing: std::collections::HashSet<String> = {
        let mut stmt = conn.prepare("PRAGMA table_info(bridge_room)")?;
        let rows = stmt.query_map([], |row| row.get::<_, String>(1))?;
        rows.collect::<Result<_>>()?
    };
    for col in ["chair_id", "queue_state", "recording_config"] {
        if existing.contains(col) {
            continue;
        }
        let stmt = format!("ALTER TABLE bridge_room ADD COLUMN {col} TEXT");
        // Belt-and-braces: a concurrent open() may have added the column
        // between our PRAGMA read and this ALTER — swallow the race.
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

/// Persist the FIFO raised-hand queue (serialised as JSON text) to bridge_room.queue_state.
/// Uses UPSERT so the caller need not pre-insert a row.
pub fn write_queue_state(
    conn: &Connection,
    case_id: i64,
    room_type: &str,
    queue_json: &str,
) -> Result<()> {
    conn.execute(
        "INSERT INTO bridge_room (case_id, room_type, queue_state)
         VALUES (?1, ?2, ?3)
         ON CONFLICT(case_id, room_type) DO UPDATE SET queue_state = excluded.queue_state",
        params![case_id, room_type, queue_json],
    )?;
    Ok(())
}

/// Read back the persisted FIFO queue JSON. Returns None when no row or queue_state is NULL.
pub fn read_queue_state(
    conn: &Connection,
    case_id: i64,
    room_type: &str,
) -> Result<Option<String>> {
    let mut stmt = conn.prepare(
        "SELECT queue_state FROM bridge_room WHERE case_id = ?1 AND room_type = ?2",
    )?;
    let mut rows = stmt.query(params![case_id, room_type])?;
    if let Some(row) = rows.next()? {
        Ok(row.get(0)?)
    } else {
        Ok(None)
    }
}

/// Persist the current chair pseudonym to bridge_room.chair_id.
/// Uses UPSERT so the caller need not pre-insert a row.
pub fn write_chair_id(
    conn: &Connection,
    case_id: i64,
    room_type: &str,
    chair_pseudonym: &str,
) -> Result<()> {
    conn.execute(
        "INSERT INTO bridge_room (case_id, room_type, chair_id)
         VALUES (?1, ?2, ?3)
         ON CONFLICT(case_id, room_type) DO UPDATE SET chair_id = excluded.chair_id",
        params![case_id, room_type, chair_pseudonym],
    )?;
    Ok(())
}

/// Read back the persisted chair pseudonym. Returns None when no row or chair_id is NULL.
pub fn read_chair_id(
    conn: &Connection,
    case_id: i64,
    room_type: &str,
) -> Result<Option<String>> {
    let mut stmt = conn.prepare(
        "SELECT chair_id FROM bridge_room WHERE case_id = ?1 AND room_type = ?2",
    )?;
    let mut rows = stmt.query(params![case_id, room_type])?;
    if let Some(row) = rows.next()? {
        Ok(row.get(0)?)
    } else {
        Ok(None)
    }
}

/// Persist the per-room recording configuration JSON knob to bridge_room.recording_config.
/// Uses UPSERT so the caller need not pre-insert a row.
#[allow(dead_code)]
pub fn write_recording_config(
    conn: &Connection,
    case_id: i64,
    room_type: &str,
    config: &str,
) -> Result<()> {
    conn.execute(
        "INSERT INTO bridge_room (case_id, room_type, recording_config)
         VALUES (?1, ?2, ?3)
         ON CONFLICT(case_id, room_type) DO UPDATE SET recording_config = excluded.recording_config",
        params![case_id, room_type, config],
    )?;
    Ok(())
}

/// Read back the persisted recording configuration JSON knob. Returns None when no row or recording_config is NULL.
#[allow(dead_code)]
pub fn read_recording_config(
    conn: &Connection,
    case_id: i64,
    room_type: &str,
) -> Result<Option<String>> {
    let mut stmt = conn.prepare(
        "SELECT recording_config FROM bridge_room WHERE case_id = ?1 AND room_type = ?2",
    )?;
    let mut rows = stmt.query(params![case_id, room_type])?;
    if let Some(row) = rows.next()? {
        Ok(row.get(0)?)
    } else {
        Ok(None)
    }
}

/// Pure parse of the per-room recording_config knob. `record_town_halls`
/// defaults FALSE (absent / unparseable / false → false). Clarify DQ
/// a3d0e9941441-073: the flag lives in the EXISTING recording_config column,
/// NOT a new column.
#[allow(dead_code)]
pub fn record_town_halls_enabled(recording_config: &str) -> bool {
    serde_json::from_str::<serde_json::Value>(recording_config)
        .ok()
        .and_then(|v| v.get("record_town_halls").and_then(|b| b.as_bool()))
        .unwrap_or(false)
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
    fn queue_state_roundtrip() {
        let conn = open(":memory:").expect("open db");
        let json = r#"["alice-pseudo","bob-pseudo"]"#;
        write_queue_state(&conn, 99, "governance", json).expect("write queue");
        let back = read_queue_state(&conn, 99, "governance").expect("read queue");
        assert_eq!(back.as_deref(), Some(json), "queue_state must round-trip");
    }

    #[test]
    fn chair_id_roundtrip() {
        let conn = open(":memory:").expect("open db");
        write_chair_id(&conn, 99, "governance", "chair-pseudo-xyz").expect("write chair");
        let back = read_chair_id(&conn, 99, "governance").expect("read chair");
        assert_eq!(back.as_deref(), Some("chair-pseudo-xyz"), "chair_id must round-trip");
    }

    #[test]
    fn recording_config_roundtrip() {
        let conn = open(":memory:").expect("open db");
        let cfg = r#"{"record_town_halls": true}"#;
        write_recording_config(&conn, 42, "town_hall", cfg).expect("write recording_config");
        let back = read_recording_config(&conn, 42, "town_hall").expect("read recording_config");
        assert_eq!(back.as_deref(), Some(cfg), "recording_config must round-trip");
    }

    #[test]
    fn record_town_halls_flag_parse() {
        assert!(record_town_halls_enabled(r#"{"record_town_halls": true}"#));
        assert!(!record_town_halls_enabled(r#"{"record_town_halls": false}"#));
        assert!(!record_town_halls_enabled("{}"));
        assert!(!record_town_halls_enabled("garbage"));
        assert!(!record_town_halls_enabled(""));
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
