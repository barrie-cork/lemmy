// Bridge cross-instance mute power-level path.
//
// `compute_mute_all_override` raises the `m.call.member` and MSC3401 alias
// event power-level bar above `users_default`, so every non-elevated participant
// loses publish authority in a single PUT, which Matrix federates cross-instance.
//
// `mute_all_power_levels` is `#[allow(dead_code)]` until Phase-6 wires the live
// trigger; reachable from the Task-4 `#[ignore]` integration test.
//
// MIRROR: per-room loop follows sanction_handler.rs:188-228;
// GET/PUT helpers reused via pub(crate) widened in Task 2 (sanction_handler.rs).

use anyhow::Result;

use crate::appservice::AppState;
use crate::bridge_room;

/// Mute-ALL generalisation of compute_power_override: instead of dropping ONE
/// subject below a threshold, RAISE the publish/voice power requirement above
/// users_default so EVERY non-elevated participant loses publish in one PUT.
/// The chair keeps publish via the room-admin power-level seated at provisioning.
/// Returns the mutated full content (always PUT the whole object — never fragment).
fn compute_mute_all_override(content: &serde_json::Value) -> serde_json::Value {
    let mut out = content.clone();
    let users_default = content.get("users_default").and_then(|v| v.as_i64()).unwrap_or(0);
    // One above users_default: non-elevated users (watchers) lose publish; the
    // chair (room-admin, e.g. 100) stays above it.
    let mute_bar = users_default + 1;
    if let Some(obj) = out.as_object_mut() {
        let events = obj.entry("events").or_insert_with(|| serde_json::json!({}));
        if let Some(ev) = events.as_object_mut() {
            ev.insert("m.call.member".to_string(), serde_json::json!(mute_bar));
            ev.insert("org.matrix.msc3401.call.member".to_string(), serde_json::json!(mute_bar));
        }
    }
    out
}

/// Drop ALL non-chair publishers in every room for `case_id` via Matrix
/// power-levels. ONE PUT per room; Matrix federation propagates it
/// cross-instance (OQ-V2-06 — chair authority is room-global). Best-effort:
/// one room's failure MUST NOT abort the others (mirror sanction loop).
#[allow(dead_code)] // live HTTP trigger lands Phase 6; reachable from the #[ignore] test now.
pub(crate) async fn mute_all_power_levels(state: &AppState, case_id: i64) -> Result<usize> {
    let conn = bridge_room::open(&state.bridge_db_path)?;
    let rooms = bridge_room::lookup_by_case(&conn, case_id)?;
    let mut applied = 0usize;
    for (_room_type, room_id) in &rooms {
        let content = match crate::sanction_handler::get_power_levels(state, room_id).await {
            Ok(c) => c,
            Err(e) => {
                tracing::warn!(err = %e, room_id = %room_id, "mute_all: get_power_levels failed — skipping room");
                continue;
            }
        };
        let muted = compute_mute_all_override(&content);
        match crate::sanction_handler::put_power_levels(state, room_id, &muted).await {
            Ok(()) => applied += 1,
            Err(e) => tracing::warn!(err = %e, room_id = %room_id, "mute_all: put_power_levels failed — skipping room"),
        }
    }
    Ok(applied)
}

#[cfg(test)]
mod tests {
    /// Test: compute_mute_all_override pure-function cases — no I/O.
    #[test]
    fn compute_mute_all_override_cases() {
        // Case 1: users_default=0, events already present but empty.
        let content = serde_json::json!({ "users_default": 0, "events": {} });
        let result = super::compute_mute_all_override(&content);
        assert_eq!(
            result["events"]["m.call.member"],
            serde_json::json!(1),
            "m.call.member must be > users_default (0)"
        );
        assert_eq!(
            result["events"]["org.matrix.msc3401.call.member"],
            serde_json::json!(1),
            "MSC3401 alias must be > users_default (0)"
        );

        // Case 2: users_default=50, no events key (must be created).
        let content = serde_json::json!({ "users_default": 50 });
        let result = super::compute_mute_all_override(&content);
        assert_eq!(
            result["events"]["m.call.member"],
            serde_json::json!(51),
            "m.call.member must be users_default + 1 = 51"
        );
    }
}
