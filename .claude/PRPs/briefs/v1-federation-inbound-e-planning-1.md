# Planning brief — v1-federation-inbound-e

**Role:** `[role:planning]`
**Phase:** `v1-federation-inbound-e`
**Authored:** 2026-05-22 (governance-v0 `673f1b8b0`)
**Next:** `/brehon-clarify` → queue planning Junior

---

## 1. Role + dispatch line

```
[role:planning] v1-federation-inbound-e — plan TOCTOU fix in evict_oldest_unreviewed_if_needed
```

---

## 2. Scope

### 2.1 What to produce

A complete plan file at `.claude/PRPs/plans/v1-federation-inbound-e.plan.md` following
the canonical template at `.claude/PRPs/templates/plan.template.md`.

### 2.2 Goal (one sentence)

Fix the TOCTOU race in `evict_oldest_unreviewed_if_needed` (`inbox.rs:646-703`) by
replacing the two-step `COUNT(*) → DELETE` sequence with a single atomic
`DELETE … RETURNING id` SQL statement that only evicts when the count exceeds cap.

### 2.3 Scope boundary (explicit)

**IN scope:**
- Rewrite `evict_oldest_unreviewed_if_needed` in
  `crates/apub/activities/src/governance/inbox.rs` to use atomic SQL
- One `#[cfg(test)] mod tests_evict_atomicity` unit test block inside the same
  file, verifying that a concurrent second insert does not double-evict (or that
  a single call at cap+1 evicts exactly one row)
- Update the `CountRow` helper struct if it becomes unused after the rewrite

**OUT of scope (explicitly deferred):**
- Per-peer bound enforcement changes (the cap-check logic already calls
  `evict_oldest_unreviewed_if_needed`; the eviction policy itself is unchanged)
- SHA-256 key-hashing for the per-actor rate-map (user clarify B3 from fed-in-d;
  still deferred)
- The per-peer rate-map bound (`inbox.rs:540-565` use site; user clarify B1 fed-in-d)
- Any other function in `inbox.rs` (the three callers of `evict_oldest_unreviewed_if_needed`
  at lines 191, 305, 761 are READ-ONLY MIRROR refs — do not edit them)
- Migration: no schema change; pure Rust logic rewrite
- New `governance_log` entry kinds: no new kinds required; existing
  `ENTRY_KIND_FEDERATION_INBOUND_DROPPED_STORAGE_CAP_EVICTED` continues unchanged
- `governance_config` knob for cap: the const `evict_cap` is already passed
  as a parameter from the callers; no new config key

### 2.4 Concurrency model (MANDATORY — this is the gate-1 question)

The TOCTOU race:
```
Thread A: COUNT(*) → 100 (at cap)      Thread B: INSERT → row 101
Thread A:                                Thread B: COUNT(*) → 101 (at cap)
Thread A: DELETE oldest                  Thread B: DELETE oldest  (DOUBLE eviction)
```

The fix must make the count+evict decision atomic. Two viable approaches:

**Option A — `DELETE … WHERE id IN (SELECT id FROM … LIMIT 1 OFFSET cap-1 ORDER BY received_at ASC) RETURNING id`**
- One SQL round-trip: delete the row that is exactly at position `cap` by rank, if any exists
- Returns 0 rows if count < cap; returns the deleted id if count ≥ cap
- No race: the DELETE is the atomic decision; if two threads race, only one gets a returned row

**Option B — `SELECT FOR UPDATE SKIP LOCKED` advisory lock**
- Lock the oldest unreviewed row, then count + conditionally delete
- More complex; SKIP LOCKED may allow two threads to each lock different rows
- Harder to reason about correctness under concurrent insert pressure

The planner MUST choose Option A or Option B, state the reasoning in §1 (plan goal),
and design the §13 task around the chosen approach. Option A is the preferred default
(one round-trip, simpler proof of atomicity) unless the planner finds a correctness
issue with it.

**Gate-1 DQ:** the advisor will raise `kind: "clarify"` asking the planner to confirm
the approach before the plan is committed. Planner MUST include a `§17 Concurrency
Model` section explaining the chosen atomic SQL and why it eliminates the race window.

### 2.5 Test scope

- **Story 1:** `evict_oldest_unreviewed_if_needed` with the new atomic SQL — unit test
  inside `inbox.rs`'s `#[cfg(test)]` block (NOT e2e; no testcontainers required)
- Test must follow the `LemmyResult<()>` return shape and `?` propagation (no `.unwrap()`),
  matching the existing test module style in `publish_sanction_notice.rs:612-720`
  (MIRROR ref)
- Phase-2 e2e regression gate: full `cargo test --workspace --test e2e --features full`
  confirming baseline 103/0/5 holds after the rewrite

---

## 3. Required reading

### 3.1 Canonical files (read before planning)

- `crates/apub/activities/src/governance/inbox.rs:634-703` — `CountRow` helper +
  `evict_oldest_unreviewed_if_needed` — the function being rewritten
- `crates/apub/activities/src/governance/inbox.rs:183-195` — first caller (sanction)
- `crates/apub/activities/src/governance/inbox.rs:297-310` — second caller (attestation)
- `crates/apub/activities/src/governance/inbox.rs:753-765` — third caller (label)
  (READ-ONLY — these callers are MIRROR refs; the function signature must remain
  compatible with all three calling sites)
- `crates/apub/activities/src/governance/publish_sanction_notice.rs:612-720` —
  MIRROR ref for `#[cfg(test)] mod tests` shape inside the apub-activities crate

### 3.2 Design refs

- `.claude/PRPs/prds/v1-federation-inbound.prd.md` §7.3 (storage cap with oldest-drop
  policy; TOCTOU is a risk explicitly noted there)
- `.claude/PRPs/plans/v1-federation-inbound-d.plan.md` §12 item 3 — the deferred item
  this phase ships; `§22 MIRROR refs` for the inbox.rs TOCTOU description

### 3.3 Mandatory lessons for §3 Required reading (file-class table matches)

- `.claude/lessons/feedback_clippy_test_style.md` — workspace denies `unwrap`/`expect`
  in test bodies; new test module must use `LemmyResult<()>` + `?`
- `.claude/lessons/feedback_multi_write_handlers_need_transactions.md` — the existing
  `run_transaction` in `evict_oldest_unreviewed_if_needed` already wraps three writes;
  the atomic SQL rewrite must preserve or replace this transaction scope correctly
- `.claude/lessons/feedback_pg_advisory_xact_lock_void_decode.md` — if Option B
  (SELECT FOR UPDATE) is chosen, this lesson is mandatory; if Option A (DELETE RETURNING)
  is chosen, note in §3 that this lesson was consulted and Option A was selected to avoid
  the void-decode trap entirely
- `.claude/lessons/feedback_daemon_stale_bm_verb_brief_miss.md` — pre-dispatch
  ref-currency check; planning Junior must confirm brief is visible on daemon before
  proceeding
- `.claude/lessons/feedback_explicit_file_arrays_on_tasks.md` — FILES YAML
  (`creates:` + `modifies:`) required on every §13 task; read before drafting §13
  so the verify gate and cohort-dispatch overlap check have complete file arrays
  (DQ `a3d0e9941441-010`)

### 3.4 Pre-queue lesson check

Run `memory_search_hybrid(query: "evict oldest row atomic SQL RETURNING postgres concurrency", tags: "lesson,brehon", limit: 5)` before drafting §13.

---

## 4. Constraints

### 4.1 PRE-PUSH MANDATE (extended — applies to BM-verb AND impl-task workers)

Before every `git push` on the worker branch:
1. Confirm `cargo check --workspace --features full` exits 0 (or the plan's §15 DoD
   commands pass) — run `bash scripts/brehon/cargo-check.sh --workspace --features full`
   (Linux wrapper on EliteDesk)
2. If non-zero: patch in same commit (if in-scope) OR file `kind: "blocker"` DQ and stop.
   NEVER `#[allow(...)]`-spam to bypass.

### 4.2 BM-verb brief visibility check (new — daemon stale-ref prevention)

Before creating any Junior task (including this planning task), the advisor will run:
```bash
ssh homeserver "cd /srv/brehon-fork && git fetch origin && git log --oneline origin/governance-v0 | head -3"
```
The brief commit (`<sha>`) must be visible before the task is dispatched. This constraint is
noted here so the plan includes it in §4 Constraints for impl-task briefs too.

### 4.3 File-ownership

The planning Junior writes ONLY:
- `.claude/PRPs/plans/v1-federation-inbound-e.plan.md`
- DQ entries (`kind: "blocker"` or `kind: "log"`) if needed
- One-line append to `.claude/runlog/v1-federation-inbound-e-runlog.md` if the file exists

NEVER writes:
- `crates/**`, `migrations/**`, `tests/**`, `docs/brehon-law-inspired-network/**`
- `.claude/PRPs/reviews/**`, `.claude/rules/**`, `.claude/lessons/**`
- Briefs, templates, or bootstrap files

### 4.4 Attribution integrity

- DQ entries: `from: "planner"`, `answered_by: null` for blockers; `answered_by: "planner"`
  with self-resolved evidence for log entries
- NEVER write `answered_by: "advisor"` from the planning session
- Commit subject: `docs(plan): v1-federation-inbound-e — <one-line>`

### 4.5 Plan shape requirements

The plan MUST include:
- `§17 Concurrency Model` — explains the chosen atomic SQL approach (Option A or B),
  the TOCTOU race window, and why the chosen approach eliminates it
- `§16a Stories` — at minimum Story 1 (eviction atomicity unit test) and Story 2
  (Phase-2 e2e baseline holds at 103/0/5)
- `§13` tasks with `[P]` markers where tasks are disjoint; FILES YAML (`creates:` +
  `modifies:`) on every task per `feedback_explicit_file_arrays_on_tasks.md`
- `§15 DoD` including `cargo test -p lemmy_apub_activities --lib` (lib tests) and
  `cargo test --workspace --test e2e --features full` (Phase-2 e2e regression gate)

### 4.6 LESSON trailers

End each planning-phase commit body with `LESSON: <one-line observation>` for any
load-bearing design decision, per `feedback_junior_pmd_write_convention.md`.

---

## 5. Estimated task count

2–3 impl tasks:
- Task 0: pre-flight harness audit
- Task 1: rewrite `evict_oldest_unreviewed_if_needed` with atomic `DELETE RETURNING` SQL
- Task 2: `#[cfg(test)] mod tests_evict_atomicity` unit test block

The planner may split or merge these based on the concurrency model choice.
