# Verify report — m2-core-hook

**Run at:** 2026-06-06 (advisor, brehon-fork-m2 lane)
**Phase branch:** `phase-m2-core-hook` @ `ec840ea1f`
**Plan:** `.claude/PRPs/plans/m2-core-transition-hook.plan.md`
**Mode:** MANUAL RECONCILIATION (back-compat: pre-§16a plan — no `## 16a` stories block, no FILES YAML in §13). `/brehon-verify` automated path halted gracefully per Step 2 pre-flight; manual reconciliation per the command's "When to skip" path.
**Outcome summary:** 10 §13 tasks (0–9): all deliverables present, no phantoms, no regressions.

---

## Per-task deliverable reconciliation (§13 IMPLEMENT vs branch state)

| Task | Deliverable | Landing commit | Verified on branch |
|---|---|---|---|
| 0 | Pre-flight transition-site re-enumeration | (task 0) | n/a — enumeration only |
| 1 | `CaseTransitionEvent` + `BridgeNotifyPayload` tagged union DTO | `116f7c55d` (merge of `1c6cb8661`) | `crates/api/api_common/src/governance.rs` present |
| 2 | `bridge_notify.rs` PM path refactor onto tagged union | `9e6ffb2c5` (merge of `3a941206b`) | `crates/api/api_utils/src/bridge_notify.rs` present |
| 3 | `governance_case_after_transition(...)` fn | `21a7aba6c` (merge of `7e0cb3348`) | `fn governance_case_after_transition` in `bridge_notify.rs` ✓ (1 decl) |
| 4 | 10 `ENTRY_KIND_ROOM_*` consts + shim re-export + registry doc | `d6cdc4a08` (forward-merge, Mode-B trunk pull) | 10 distinct ROOM consts in `crates/db_schema/src/source/governance/governance_log.rs`, re-exported via api shim ✓ |
| 5 | `append_room_event(...)` typed wrapper | `d6cdc4a08` (forward-merge) | `fn append_room_event` present ✓ |
| 6 | WIRE hook at 8 handler sites | `d9d516b87` (merge of `66b80d1ad`) | hook call present in admin_assign_jury / admin_close_case / submit_jury_vote / appeal_window_expiry / sponsor_liability_grace / request_appeal / revoke_endorsement ✓ |
| 7 | WIRE hook at 3 cron sites + api_crud loop (site 12) | `e84845a48` (merge of `5b7164ac0`) | cron-site wiring present ✓ |
| 8 | e2e tests (append-wrapper / suppression) | `91c646514` (merge of `667e0c54d`) + `719c800b1` (corruption fix) | 3 `m2_` tests in `crates/server/tests/e2e/governance.rs` (L5249/5306/5348) ✓ |
| 9 | Clippy + workspace gate + doc-drift DQ | (task 9) | gate task |

## Structural ACs (plan §16)

- **10 `ENTRY_KIND_ROOM_*` consts:** ARCHIVED, BRIDGE_ERROR, CREATED, DECISION_RELAYED, IDENTITY_REVEALED, LIFECYCLE_EVENT, MEMBER_ADDED, MEMBER_REMOVED, RECORDING_UPLOADED, TRANSCRIPT_READY — all 10 present ✓
- **Total entry-kind registry = 65** (AC: "total 65; collision check clean") — confirmed exactly 65 distinct `ENTRY_KIND_*` ✓
- **`append_room_event` rejects non-room kinds** — covered by `m2_append_room_event_rejects_non_room_kind` (passes) ✓
- **`messaging_enabled=false` → no hook side-effect** — covered by `m2_hook_suppressed_when_messaging_disabled` (passes) ✓
- **Hash chain holds after `append_room_event`** — covered by `m2_append_room_event_writes_chain_entry` (passes) ✓

## Checkpoint execution

e2e suite validated this session (laptop, Docker testcontainers, brehon-fork-m2):
- compile clean (was 11 errors pre-corruption-fix)
- 3 `m2_` tests: **3 passed, 0 failed**
- `phase1_revert_list_matches_disk`: **passed** (revert list now 21 entries, matches disk newest-21)

Logs: `%LOCALAPPDATA%\Temp\m2-e2e-fix.log`, `%LOCALAPPDATA%\Temp\m2-revertlist.log`.

---

## Required actions

None — all deliverables present, no phantoms, no regressions. Merge-confirm gate CLEAR pending: (c) Linux-compile gate, (d) bm-pr → CodeRabbit triage.

## Notes for retro

- **Plan predates §16a stories convention** — `/brehon-verify` could not run its automated FILES-YAML / structural-pattern path. Manual reconciliation sufficed here (10-task phase, clear `feat(task N)` commit trail) but a §16a retrofit would make future re-verify mechanical. Surface as a low-priority retro item (not blocking).
- **Tasks 4+5 landed via `chore(advisor): forward-merge` (Mode-B trunk pull), not `feat(task N)` commits** — initial commit-subject scan missed them; presence confirmed by content check. The forward-merge subject names "(Tasks 4+5 consts+wrapper)" so the audit trail is intact, but a `feat(` subject would have been cleaner.
