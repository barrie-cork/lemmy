# Brief: impl-task 0 — v1-redaction-r1 pre-flight harness audit

**Role:** `[role:impl-task]`
**Phase:** `v1-redaction-r1`
**Authored:** 2026-05-28
**Authored by:** advisor (canonical brehon-fork session, on `governance-v0`)
**Base branch (Junior forks from):** `phase-v1-redaction-r1`
**Lane mode:** Mode B (mobile remote-control) per `.claude/rules/multi-lane-worktree.md`. Junior worker runs on the EliteDesk daemon (Linux).

---

## 1. Role + dispatch

`[role:impl-task] v1-redaction-r1 task 0 — pre-flight harness audit — see .claude/PRPs/briefs/v1-redaction-r1-impl-0.md`

---

## 2. Scope

Pre-flight audit only. **No code edits. No commits on success.** Verify the environment is ready for v1-redaction-r1 impl tasks.

**Platform note:** The Junior worker runs on a Linux daemon. The plan §13 Task 0 cites `cmd //c "scripts\\brehon\\cargo-check.bat ..."` invocations (Windows form) — those are translated below to the equivalent Linux `bash scripts/brehon/cargo-check.sh ...` invocations. **The probe count, flags, expected exit codes, and log-tail tails are byte-identical to the plan**; only the wrapper extension + invocation shell differ. This translation per `feedback_handover_assumptions_need_empirical_verification.md` (verify each named assumption — the daemon has zero `cmd` binary; bare `cmd //c` would fail with `command not found`).

Run the 13 probes (Probe 0 through Probe 12) from plan §13 Task 0 in order. Probes are translated to daemon-native syntax (`bash scripts/brehon/cargo-*.sh` instead of `cmd //c "scripts\\brehon\\cargo-*.bat"`):

```bash
# Probe 0 — Docker daemon
docker ps > /dev/null 2>&1 && echo "DOCKER OK" || { echo "DOCKER NOT RUNNING"; exit 1; }

# Probe 1 — wrapper sanity (cargo-check honors -p)
bash scripts/brehon/cargo-check.sh -p lemmy_utils > .claude/PRPs/debug/v1-redaction-r1-audit-cargo-check-p.log 2>&1
echo "exit: $?"
tail -20 .claude/PRPs/debug/v1-redaction-r1-audit-cargo-check-p.log
# EXPECT: exit 0; only lemmy_utils compiles

# Probe 2 — feature flag activation
bash scripts/brehon/cargo-check.sh --workspace --features full > .claude/PRPs/debug/v1-redaction-r1-audit-cargo-check-features.log 2>&1
echo "exit: $?"
tail -20 .claude/PRPs/debug/v1-redaction-r1-audit-cargo-check-features.log
# EXPECT: exit 0; workspace compiles with --features full

# Probe 3 — cargo-test wrapper honors target selection
bash scripts/brehon/cargo-test.sh --workspace --test e2e --no-run --features full > .claude/PRPs/debug/v1-redaction-r1-audit-cargo-test.log 2>&1
echo "exit: $?"
tail -20 .claude/PRPs/debug/v1-redaction-r1-audit-cargo-test.log
# EXPECT: exit 0; e2e test target compiles workspace-wide

# Probe 4 — wrappers fail loud on cargo errors (exit-code propagation)
bash scripts/brehon/cargo-test.sh --workspace --test e2e --no-run --features nonexistent_xyz > .claude/PRPs/debug/v1-redaction-r1-audit-cargo-test-negative.log 2>&1
echo "cargo-test.sh exit on bogus feature: $?"
bash scripts/brehon/cargo-check.sh --workspace --features nonexistent_xyz > .claude/PRPs/debug/v1-redaction-r1-audit-cargo-check-negative.log 2>&1
echo "cargo-check.sh exit on bogus feature: $?"
# EXPECT: BOTH non-zero (typically 101)

# Probe 5 — clippy baseline against pre-r1 HEAD
bash scripts/brehon/cargo-clippy.sh --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/v1-redaction-r1-audit-clippy-baseline.log 2>&1
echo "exit: $?"
tail -40 .claude/PRPs/debug/v1-redaction-r1-audit-clippy-baseline.log
# EXPECT: exit 0

# Probe 6 — WP-6 call-site re-enumeration (lane-cut-time drift check)
rg "scrub\(|scrub_json\(" crates/ --type rust | tee .claude/PRPs/debug/v1-redaction-r1-callsite-audit.log
rg "scrub\(|scrub_json\(" crates/ --type rust | wc -l
# EXPECT: count >= 21 (plan-author tip recorded 23); any new caller
# triggers a re-read of ±5 lines + a §16a Story 4 update if the new
# caller is on a write path into the three gated columns. If count
# differs from 23 by ≥2, file a kind:"blocker" DQ asking advisor to
# rerun the bypass-path analysis before any §13 impl task runs.

# Probe 7 — current branch (Junior worker runs on junior/* per harness)
git branch --show-current
# EXPECT: starts with "junior/role-impl-task-v1-redaction-r1-task-0-..." or similar
# Cross-check the base: `git merge-base --is-ancestor phase-v1-redaction-r1 HEAD`
git merge-base --is-ancestor phase-v1-redaction-r1 HEAD && echo "BASE_OK phase-v1-redaction-r1 is ancestor" || echo "BASE_MISMATCH — STOP"
# EXPECT: BASE_OK
# NOTE: this is per RT-r3 Task 0 DQ 1b8527b076d4-001 methodology gap acceptance —
# Junior framework always runs on junior/* branch; base IS phase-v1-redaction-r1
# (confirmed by ancestor check). This is NOT a probe failure.

# Probe 8 — canonical definition + shim invariant
wc -l crates/db_schema/src/source/governance/redaction.rs
# EXPECT: 159 (canonical)
wc -l crates/api/api/src/governance/redaction.rs
# EXPECT: 22 (shim, plan-author tip; tolerance ±2 — DO NOT EDIT per plan §11)
rg "^pub use lemmy_db_schema::source::governance::redaction::" crates/api/api/src/governance/redaction.rs
# EXPECT: 1 hit (the re-export line)

# Probe 9 — existing 5 unit tests are intact at the expected line range
rg -n "fn scrub_strips_mentions|fn scrub_strips_email|fn scrub_strips_profile_urls|fn scrub_json_walks_nested_structure|fn scrub_preserves_keys_and_non_string_scalars" crates/db_schema/src/source/governance/redaction.rs
# EXPECT: 5 hits (one per existing test)

# Probe 10 — pretty_assertions already in scope (no Cargo.toml change needed for new tests)
rg "pretty_assertions::assert_eq" crates/db_schema/src/source/governance/redaction.rs
# EXPECT: 1 hit (line 106)

# Probe 11 — lemmy_db_schema declares the `full` feature
rg '^\[features\]' crates/db_schema/Cargo.toml -A 5
# EXPECT: `full = ...` line present
rg '^full = ' crates/db_schema/Cargo.toml | head -1
# EXPECT: 1 hit (confirms -p lemmy_db_schema --features full is valid in §14)

# Probe 12 — concurrent-PR check (no other PR touches r1's IMPLEMENT file)
gh pr list --repo barrie-cork/lemmy --state open --json number,title,headRefName,files \
  --jq '.[] | select(.files[]?.path | test("governance/redaction\\.rs|governance/governance_log\\.rs")) | {number, title, headRefName}'
# EXPECT: empty output; if any other lane is touching r1 files, STOP and file kind:"blocker" DQ
```

**EXPECT block:**
- Probes 0..3, 5..12 exit 0 (or as documented per probe)
- Probe 4 exits NON-ZERO (negative test confirms exit-code propagation)
- Probe 6 returns count >= 21 (planner-time count was 23; drift of ±2 acceptable, > 2 triggers blocker DQ)
- Probe 7 returns `junior/...` branch + `BASE_OK phase-v1-redaction-r1 is ancestor` (per RT-r3 methodology gap acceptance)
- Probe 8: 159 lines (canonical) + 22 lines (shim) + 1 re-export
- Probe 9: 5 existing test fns intact
- Probe 10: `pretty_assertions::assert_eq` already imported
- Probe 11: `lemmy_db_schema` defines `full` feature
- Probe 12: no concurrent PR overlap

**Task 0 produces NO commit** if all probes pass — write results to task output only. If any probe fails: write a `kind: "blocker"` DQ entry to `.claude/decision-queue.json`, commit + push it on the worker branch, and stop.

### 2.1 Common environment bootstrap (per `feedback_phase_lane_worktree_bootstrap_checklist.md`)

Junior worktrees do NOT auto-initialize submodules. Before Probe 1, run:

```bash
git submodule update --init --recursive
```

This fixes the `crates/email/translations` empty-gitlink → `Os { code: 3, kind: NotFound }` failure that breaks any cargo command touching `lemmy_email`. Per RT-r3 Task 0 DQ 1b8527b076d4-001 the submodule init was the infrastructure recovery; this brief pre-empts the failure.

---

## 3. Required reading

- `.claude/PRPs/plans/v1-redaction-r1.plan.md` §13 Task 0 (the canonical probe list + EXPECT block; this brief mirrors that block with Linux-syntax wrapper invocations)
- `.claude/rules/pre-phase-harness-audit.md` (R5: enumerate ALL probes explicitly)
- `.claude/rules/decision-queue.md` (DQ schema-v3 + Junior subagent attribution rules)
- `.claude/lessons/feedback_phase_lane_worktree_bootstrap_checklist.md` (submodule init pre-empt)
- `.claude/lessons/feedback_handover_assumptions_need_empirical_verification.md` (why the brief translates `.bat` → `.sh` rather than betting on Junior's adaptive recovery)
- `.claude/lessons/feedback_wrapper_script_flag_silence.md` (wrapper scripts pass `$@` literally; the `-p` / `--features` / `--no-deps` flag set IS load-bearing — do not strip)

---

## 4. Constraints

- **No code edits** — this task is verification only.
- **No commit on success** — audit output goes to task output, not git.
- If any probe fails: file `kind: "blocker"` DQ (use `bash scripts/brehon/dq-v3-new-entry.sh` for the composite id; use `bash scripts/brehon/dq-v3-append-fragment.sh <fragment.json> --pending` to append), commit + push on the worker branch, then stop. Do not proceed to Task 1.
- Branch base MUST be `phase-v1-redaction-r1` (Probe 7's `git merge-base --is-ancestor` ancestor check confirms this; if it returns BASE_MISMATCH, file a blocker DQ).
- **Translation discipline:** the `.bat` → `.sh` wrapper translation is platform-mandatory (daemon has no `cmd` binary). All other probe parameters (flags, log paths, expected exit codes, `tail` line counts, `rg` regexes, `wc -l` expectations) are **byte-identical** to plan §13 Task 0. If a probe in this brief diverges from the plan in anything other than the wrapper extension, that's an authoring mistake — refuse and file a blocker DQ.
- **Attribution:** the worker's DQ writes use `from: "impl"`. **NEVER write `answered_by: "advisor"` or `answered_by: "user"`** per `.claude/rules/decision-queue.md` Hard refusal #1.
- **Mid-task push discipline:** if a blocker DQ fires, commit + push the DQ entry on the worker branch in the SAME atomic sequence (per `decision-queue.md` §"Mid-task visibility"). The advisor's next poll sees the new pending DQ; without the push the advisor cannot see it.
- Shape G is **SUSPENDED** per DQ #229 — this is a pre-Shape-G plan. Cargo runs on the laptop via `validate-pending-laptop` for impl tasks 1-3. **Task 0 cargo probes (1, 2, 3, 4, 5) are exceptions: they run inline on the EliteDesk worker because Task 0 has no commit/push and the per-task validate-pending pathway doesn't apply to verification-only tasks.** This exception precedent is established by RT-r3 Task 0 (commit `46b73c130`).

---

## 5. Forbidden-window check (advisor pre-queue)

Per `.claude/rules/advisor-orchestrator.md` §5.1: Shape G suspended → forbidden-window check IS binding for this task (Task 0 runs cargo inline on EliteDesk). Advisor verified at queue time (2026-05-28T19:40+ UTC = 19:40 UK; outside all standard forbidden windows: 02:00-04:00 backups, 04:00-06:00 weekly review Sundays, 22:00-06:00 web-archive crawls). Subagent's task-0 pre-flight refuses with `FORBIDDEN_WINDOW: <window>` if mis-queued.

---

## 6. Context

- **Phase:** v1-redaction-r1 (P0 GDPR-critical redaction hardening — issue #58)
- **Plan:** `.claude/PRPs/plans/v1-redaction-r1.plan.md` on `phase-v1-redaction-r1` at `5e9b1e35b` (merged via `410385334`)
- **Phase branch:** `phase-v1-redaction-r1` (cut from governance-v0 `ab973e9ef`; bm-cut runlog entry on trunk)
- **Base branch for this task:** `phase-v1-redaction-r1`
- **Lane mode:** Mode B (no laptop-side phase worktree)
- **Plan-approved at:** 2026-05-28T19:30 UTC (advisor gate-1 sign-off; DoD smoke green at §15.1 exit 0 + §15.2 exit 0 against `governance-v0` HEAD which is functionally equivalent to `phase-v1-redaction-r1` HEAD modulo zero Rust source changes)
- **Concurrent lane activity:** Lane Q (`phase-v1-quality-r2`) planning Junior #488 completed; Lane Q gate-1 not yet surfaced; both lanes touch different files (Lane A = `redaction.rs`; Lane Q = `e2e.rs` + governance handlers + scripts); zero `modifies:` overlap; below cohort-≥3 daemon `.git/index.lock` threshold.
- **After Task 0 passes (all 13 probes EXPECT block satisfied):** advisor reads task output, no merge needed (no commit), queues Task 1 brief author + dispatch.
- **If Task 0 surfaces a blocker DQ:** advisor reads the DQ on next poll, routes per §5.4 DQ triage decision tree (advisor-answer with citation OR catch-fire to user OR user-relay).
