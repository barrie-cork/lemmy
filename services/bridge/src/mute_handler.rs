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
///
/// cr-9: max() guard — only raises thresholds, never lowers a pre-existing one.
/// cr-10: absent/non-object events key is propagated as an error (Matrix spec
///        guarantees the events key in a valid power-levels event).
fn compute_mute_all_override(content: &serde_json::Value) -> Result<serde_json::Value> {
    let mut out = content.clone();
    let users_default = content.get("users_default").and_then(|v| v.as_i64()).unwrap_or(0);
    // One above users_default: non-elevated users (watchers) lose publish; the
    // chair (room-admin, e.g. 100) stays above it.
    let mute_level = users_default + 1;
    {
        let obj = out
            .as_object_mut()
            .ok_or_else(|| anyhow::anyhow!("power_levels content is not an object"))?;
        let events_val = obj
            .get_mut("events")
            .ok_or_else(|| anyhow::anyhow!("power_levels.events missing or non-object"))?;
        let events = events_val
            .as_object_mut()
            .ok_or_else(|| anyhow::anyhow!("power_levels.events is not an object"))?;
        // cr-9: only raise, never lower — skip if a higher threshold is already set.
        let current = events.get("m.call.member").and_then(|v| v.as_i64()).unwrap_or(0);
        if mute_level > current {
            events.insert("m.call.member".to_string(), serde_json::json!(mute_level));
        }
        let current_alias = events
            .get("org.matrix.msc3401.call.member")
            .and_then(|v| v.as_i64())
            .unwrap_or(0);
        if mute_level > current_alias {
            events.insert("org.matrix.msc3401.call.member".to_string(), serde_json::json!(mute_level));
        }
    }
    Ok(out)
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
        let muted = match compute_mute_all_override(&content) {
            Ok(m) => m,
            Err(e) => {
                tracing::warn!(err = %e, room_id = %room_id, "mute_all: compute_mute_all_override failed — skipping room");
                continue;
            }
        };
        match crate::sanction_handler::put_power_levels(state, room_id, &muted).await {
            Ok(()) => applied += 1,
            Err(e) => tracing::warn!(err = %e, room_id = %room_id, "mute_all: put_power_levels failed — skipping room"),
        }
    }
    Ok(applied)
}

#[cfg(test)]
mod tests {
    use anyhow::Result;

    /// Test: compute_mute_all_override pure-function cases — no I/O.
    ///
    /// cr-9: max() guard — only raise, never lower a pre-existing threshold.
    /// cr-10: absent/non-object events propagates as an error.
    #[test]
    fn compute_mute_all_override_cases() -> Result<()> {
        // Case 1: users_default=0, events present and empty → sets both keys to 1.
        let content = serde_json::json!({ "users_default": 0, "events": {} });
        let result = super::compute_mute_all_override(&content)?;
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

        // Case 2 (cr-9): pre-existing threshold higher than mute_level — must NOT lower it.
        let content = serde_json::json!({
            "users_default": 50,
            "events": { "m.call.member": 150, "org.matrix.msc3401.call.member": 200 }
        });
        let result = super::compute_mute_all_override(&content)?;
        assert_eq!(
            result["events"]["m.call.member"],
            serde_json::json!(150),
            "max() guard: must not lower pre-existing m.call.member threshold"
        );
        assert_eq!(
            result["events"]["org.matrix.msc3401.call.member"],
            serde_json::json!(200),
            "max() guard: must not lower pre-existing MSC3401 alias threshold"
        );

        // Case 3 (cr-10): events key absent → must propagate as error, not silently no-op.
        let content = serde_json::json!({ "users_default": 50 });
        assert!(
            super::compute_mute_all_override(&content).is_err(),
            "absent events key must return an error (cr-10: Matrix spec guarantees events exists)"
        );

        Ok(())
    }
}
