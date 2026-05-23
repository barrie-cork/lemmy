# Brief: impl-task 0 — v1-RT-r2 pre-flight harness audit

## 1. Role + dispatch

`[role:impl-task] v1-RT-r2 task 0 — pre-flight harness audit — see .claude/PRPs/briefs/v1-RT-r2-impl-0.md`

## 2. Scope

Pre-flight audit only. **No code edits. No commits.** Verify the environment is ready for v1-RT-r2 impl tasks.

Run the 10 probes from plan §13 Task 0 in order:

```bash
# Probe 0 — Docker daemon
docker ps > /dev/null 2>&1 && echo "DOCKER OK" || { echo "DOCKER NOT RUNNING"; exit 1; }

# Probe 1 — wrapper sanity
cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_utils > .claude/audit-cargo-check-p.log 2>&1"
tail -20 .claude/audit-cargo-check-p.log

# Probe 2 — feature flag activation
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/audit-cargo-check-features.log 2>&1"
echo "exit: $?"
tail -20 .claude/audit-cargo-check-features.log

# Probe 3 — cargo-test wrapper honors target selection
cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --no-run --features full > .claude/audit-cargo-test.log 2>&1"
tail -20 .claude/audit-cargo-test.log

# Probe 4 — wrappers fail loud on cargo errors (exit-code propagation)
cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --no-run --features nonexistent_xyz > .claude/audit-cargo-test-negative.log 2>&1"
echo "cargo-test.bat exit on bogus feature: $?"
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features nonexistent_xyz > .claude/audit-cargo-check-negative.log 2>&1"
echo "cargo-check.bat exit on bogus feature: $?"

# Probe 5 — clippy baseline
cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/audit-clippy-baseline.log 2>&1"
echo "exit: $?"
tail -40 .claude/audit-clippy-baseline.log

# Probe 6 — current branch is phase-v1-RT-r2
git branch --show-current

# Probe 7 — r1 schema effects landed on the base
git log governance-v0 --oneline | head -25

# Probe 8 — 17 governance_config keys r2 reads are present
rg "decay\.(reporting_accuracy|jury_reliability|participation_consistency|endorsement_strength)\.(positive|negative)_half_life_days" crates/api/api/src/governance/config.rs | wc -l
rg "bounds\.(reporting_accuracy|jury_reliability|participation_consistency|endorsement_strength)\.(floor|ceiling)" crates/api/api/src/governance/config.rs | wc -l
rg 'feature\.reputation_v1_decay_enabled' crates/api/api/src/governance/config.rs | wc -l

# Probe 9 — concurrent-PR check
gh pr list --repo barrie-cork/lemmy --state open --json number,title,headRefName,files \
  --jq '.[] | select(.files[]?.path | contains("reputation_snapshot.rs")) | {number, title, headRefName}'
```

**EXPECT block:**
- Probes 0, 1, 2, 3, 5 exit 0
- Probe 4: both lines print non-zero (negative test)
- Probe 6: `phase-v1-RT-r2`
- Probe 8: lines 1 = 8, lines 2 = 8, lines 3 ≥ 1
- Probe 9: empty output

If any probe fails: write a `kind: "blocker"` DQ entry to `.claude/decision-queue.json`, commit + push it, and stop.

**Task 0 produces NO commit** if all probes pass — write results to task output only.

## 3. Required reading

- `.claude/PRPs/plans/v1-RT-r2.plan.md` §13 Task 0 (full probe list + EXPECT block)
- `.claude/rules/pre-phase-harness-audit.md` (if exists)

## 4. Constraints

- **No code edits** — this task is verification only.
- **No commit** on success — audit output goes to task output, not git.
- If any probe fails: file `kind: "blocker"` DQ, commit + push, stop. Do not proceed to Task 1.
- Branch must be `phase-v1-RT-r2` (Probe 6 confirms this).
