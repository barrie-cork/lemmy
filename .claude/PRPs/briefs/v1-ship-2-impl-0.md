# Brief: v1-ship-2 impl Task 0 — pre-flight harness audit

## 1. Role + dispatch line

`[role:impl-task] v1-ship-2 Task 0 pre-flight harness audit — see .claude/PRPs/briefs/v1-ship-2-impl-0.md`

## 2. Scope

### 2.1 What to produce

Run the Task 0 pre-flight harness audit defined in `.claude/PRPs/plans/v1-ship-2.plan.md` §13 Task 0. This is verification only — **no code changes, no commit**. Report probe results to task output.

### 2.2 Probes to execute (verbatim from plan §13 Task 0)

```bash
# Probe 0 — Docker daemon
docker ps > /dev/null 2>&1 && echo "DOCKER OK" || { echo "DOCKER NOT RUNNING"; exit 1; }

# Probe 1 — branch
git branch --show-current
# EXPECT: phase-v1-ship-2

# Probe 2 — wrapper sanity: cargo-check honors -p (positive)
./scripts/brehon/cargo-check.sh -p lemmy_utils > .claude/audit-cargo-check-p.log 2>&1
echo "exit: $?"
tail -20 .claude/audit-cargo-check-p.log
# EXPECT: exit 0; only lemmy_utils compiles

# Probe 3 — wrapper sanity: cargo-check honors --features full (positive)
./scripts/brehon/cargo-check.sh -p lemmy_db_schema --features full > .claude/audit-cargo-check-features.log 2>&1
echo "exit: $?"
tail -20 .claude/audit-cargo-check-features.log
# EXPECT: exit 0; compiles with features enabled

# Probe 4 — wrapper sanity: cargo-test honors target selection (positive)
./scripts/brehon/cargo-test.sh --test e2e --no-run -p lemmy_server > .claude/audit-cargo-test.log 2>&1
echo "exit: $?"
tail -20 .claude/audit-cargo-test.log
# EXPECT: exit 0; only e2e test target compiles

# Probe 5 — wrapper sanity: non-zero exit propagation (negative)
./scripts/brehon/cargo-test.sh --test e2e --no-run -p lemmy_server --features nonexistent_xyz > .claude/audit-cargo-test-negative.log 2>&1
echo "cargo-test.sh exit on bogus feature: $?"
./scripts/brehon/cargo-check.sh -p lemmy_server --features nonexistent_xyz > .claude/audit-cargo-check-negative.log 2>&1
echo "cargo-check.sh exit on bogus feature: $?"
# EXPECT: BOTH exits NON-ZERO (typically 101)

# Probe 6 — workspace clippy baseline
./scripts/brehon/cargo-clippy.sh --workspace --features full --no-deps -- -D warnings > .claude/audit-clippy-baseline.log 2>&1
echo "exit: $?"
tail -40 .claude/audit-clippy-baseline.log
# EXPECT: exit 0; no clippy warnings

# Probe 7 — workspace e2e --no-run baseline
./scripts/brehon/cargo-test.sh --workspace --features full --test e2e --no-run > .claude/audit-e2e-no-run-baseline.log 2>&1
echo "exit: $?"
tail -20 .claude/audit-e2e-no-run-baseline.log
# EXPECT: exit 0; all existing tests compile

# Probe 8 — v1-ship-1 AGPL test still passes on base
./scripts/brehon/cargo-test.sh --workspace --test e2e --features full agpl_source_disclosure_surface_returns_notice > .claude/audit-v1-ship-1-baseline.log 2>&1
echo "exit: $?"
tail -20 .claude/audit-v1-ship-1-baseline.log
# EXPECT: exit 0; 1 passed; 0 failed

# Probe 9 — canonical sibling module exists; v1_ship_2 module absent
grep -n "mod v1_federation_inbound_a_fixtures" crates/server/tests/e2e.rs
# EXPECT: exactly one match near line 15388
grep -n "mod v1_ship_2_fixtures" crates/server/tests/e2e.rs
# EXPECT: zero matches (not yet created)

# Probe 10 — concurrent-PR check (no open PR touches e2e.rs)
gh pr list --repo barrie-cork/lemmy --state open --json number,title,headRefName,files \
  --jq '.[] | select(.files[]?.path == "crates/server/tests/e2e.rs") | {number, title, headRefName}'
# EXPECT: empty output
```

**EXPECT block:**
- Probes 0-4, 6-10: exit 0
- Probe 5: BOTH exits NON-ZERO

**No commit at Task 0** — this is verification only. Report all probe outputs and PASS/FAIL in task output.

### 2.3 Explicit boundaries

- **DO NOT** write any Rust code.
- **DO NOT** commit or push anything.
- **DO NOT** run anything other than the 11 probes above.
- If any probe fails: raise a `kind: "blocker"` DQ entry and STOP (per `.claude/rules/pre-phase-harness-audit.md`).

## 3. Required reading

- `.claude/PRPs/plans/v1-ship-2.plan.md` §13 Task 0 — the canonical probe list (this brief excerpts verbatim from it)
- `.claude/rules/pre-phase-harness-audit.md` — full harness audit discipline

## 4. Constraints

1. **Linux invocation** — the EliteDesk Junior daemon runs Linux. Use `./scripts/brehon/cargo-check.sh`, `./scripts/brehon/cargo-test.sh`, `./scripts/brehon/cargo-clippy.sh` (not `.bat`).
2. **DQ mid-task push** — if any blocker DQ is raised, commit + push immediately per `.claude/rules/decision-queue.md` "Mid-task visibility".
3. **Attribution** — never write `answered_by: "advisor"` in a DQ entry from this session.
4. **No LESSON: trailer needed** — Task 0 is pure verification; no code authorship.
