# brehon-conformance-audit — impl Task 7 brief (Cohort 3 — DOGFOOD)

## 1. Role + dispatch line

`[role:impl-task] brehon-conformance-audit Task 7 — dogfood against v1-federation-inbound-b two snapshots — see .claude/PRPs/briefs/brehon-conformance-audit-impl-7.md`

## 2. Scope

Implement plan §13 **Task 7** (lines 1182-1280). The dogfood is the calibration test: run the conformance-audit skill against TWO snapshots of `v1-federation-inbound-b` per PRECON-8 and produce four artifacts proving the skill catches the Finding 6.1 axis-4 latent footgun before it would have shipped.

```yaml
creates:
  - .claude/PRPs/reports/conformance-audit-v1-federation-inbound-b-dogfood-2026-05-20.md
  - .claude/PRPs/audit-metrics/v1-federation-inbound-b.json
modifies:
  - .gitignore                                                             # add .claude/PRPs/audit-metrics/
  - .claude/skills/brehon-conformance-audit/METRICS.md                     # backfill ## Worked example with real numbers
requires:
  - task: 1
    reason: "SKILL.md defines the invocation contract."
  - task: 2
    reason: "Six axis sub-files supply detection methods."
  - task: 3
    reason: "find-sibling.sh invoked during dogfood."
  - task: 4
    reason: "audit-metrics.schema.json is the contract for the seed file."
  - task: 5
    reason: "METRICS.md ## Worked example backfilled here."
  - task: 6
    reason: "compute-metrics.sh invoked at end of dogfood; all prereqs shipped at f48e90d1b."
  - task: 8
    reason: "clippy.toml seed entries must exist before dogfood reports on Track-B coverage of axis-4 (already shipped 6720dc72a)."
  - task: 9
    reason: "per-module #![deny] confirms federation enforcement (already shipped 4464a2c0f)."
```

**Branch context:**
- `base_branch` = `phase-brehon-conformance-audit` (at `f48e90d1b` — all prereq Tasks 1-6, 8, 9 shipped).
- Worker forks to its own `junior/...` worktree branch.
- ONE commit on worker branch; daemon finalize-merges.

**Snapshot SHAs (verified reachable):**
- **Snapshot 1 — pre-fix-impl-3 tip**: `649871f7d6a60d19fab57877003bd8da1c67ce86` (full SHA). Parent of the fix commit. MUST flag Finding 6.1 (axis-4 Tier-1) on `receive_remote_moderation_label`.
- **Snapshot 2 — current merged tip**: `4a60667c9a4938b62d3150ce8677ffcd429ca9c4` (full SHA). MUST NOT flag axis-4 on `receive_remote_moderation_label` (fix-impl-3 closed it).
- **Fix commit (ground truth)**: `8b04e69a6655698305a38d4c22c300480ccbe4e6`. Author-date and commit body cited in the report.

**Do NOT** author:
- Any change to `crates/**`, `migrations/**`, `tests/**`, `docs/**`.
- Any change to other skill files (`audit-metrics.schema.json`, `SKILL.md`, axis sub-files, scripts/).
- A LIVE `/<skill-name>` invocation against the worktree — the dogfood READS history per Watchpoint #6 + brief §2.3 ambiguity #5. The "predictions" in the metrics JSON are constructed by applying the skill's detection methods to the two snapshots' diffs MANUALLY (or via grep + find-sibling.sh — whichever the worker prefers), not by running the skill as a live invocation.

## 3. Required reading

### 3.0 Plan + axis-4 detection method (P0)

1. `.claude/PRPs/plans/brehon-conformance-audit.plan.md` §13 Task 7 (lines 1182-1280) — full task text.
2. `.claude/PRPs/plans/brehon-conformance-audit.plan.md` §10.5 (Worked example template — verbatim source for the report shape).
3. `.claude/skills/brehon-conformance-audit/SKILL.md` — invocation contract.
4. `.claude/skills/brehon-conformance-audit/axes/4-error-idiom.md` — axis-4 (error-idiom) detection method; this is the axis Finding 6.1 falls under.
5. `.claude/skills/brehon-conformance-audit/METRICS.md` — current placeholder content of `## Worked example`; this is what the worker REPLACES with real numbers.
6. `.claude/skills/brehon-conformance-audit/audit-metrics.schema.json` — read to understand the required shape of the seed JSON.
7. `.claude/skills/brehon-conformance-audit/scripts/find-sibling.sh` + `.claude/skills/brehon-conformance-audit/scripts/compute-metrics.sh` — invoked during the dogfood.

### 3.1 Federation inbox handler context (the actual code being audited)

8. `git show 649871f7d:crates/apub/activities/src/governance/inbox.rs` — Snapshot 1 file (axis-4 Finding 6.1 site at L735-ish: `.unwrap_or_default()`).
9. `git show 4a60667c9:crates/apub/activities/src/governance/inbox.rs` — Snapshot 2 file (axis-4 fix at L735-ish: `.ok_or_else(|| LemmyErrorType::Unknown(...))?`).
10. `git log -1 --format=%B 8b04e69a6` — fix commit body; cite for ground-truth seed.
11. `git log -1 --format=%aI 8b04e69a6` — fix commit author-date; used for lead-time computation.

Sibling references:
- `crates/apub/activities/src/governance/inbox.rs` line ~105 (`receive_remote_sanction_notice`) — sibling with `.ok_or_else()` idiom.
- `crates/apub/activities/src/governance/inbox.rs` line ~261-271 (`receive_remote_trust_attestation`) — sibling cited in fix-impl-3 commit body (the verbatim mirror source).

### 3.2 Rules (auto-loaded)

12. `.claude/rules/decision-queue.md` — `kind: "validate-pending-laptop"` shape for §5.3 DQ raise.
13. `.claude/rules/phase-branch.md` — worker branch push discipline.

### 3.3 §G4 CANONICAL RECIPE (no allowlist row applies)

This is a structural Task 7 implementation, NOT a fix-impl. No §G4 allowlist row applies. The plan §13 Task 7 IMPLEMENT block + §10.5 worked-example template are the contract.

## 4. Constraints

1. **Mid-task push discipline** (per `.claude/rules/decision-queue.md` "Mid-task visibility") — any DQ pushed to worker branch immediately after raise.
2. **Attribution integrity** — `from: "impl"`; never write `answered_by: "advisor"`/`"user"`.
3. **Four-file diff** — `git status --short` after the change must show EXACTLY:
   - `?? .claude/PRPs/reports/conformance-audit-v1-federation-inbound-b-dogfood-2026-05-20.md`
   - `?? .claude/PRPs/audit-metrics/v1-federation-inbound-b.json`
   - `M .gitignore`
   - `M .claude/skills/brehon-conformance-audit/METRICS.md`
   Any other file in the diff is out of scope.
4. **READ history, do NOT MUTATE fed-in-b state** — use `git show <sha>:<file>` to read the two snapshots. The worktree stays on `phase-brehon-conformance-audit`. No checkout into fed-in-b.
5. **No cargo invocation** — Watchpoint #3. The dogfood is analytical; no compile, no test.
6. **No `--no-verify`** — never skip hooks.
7. **Schema-validate the seed JSON BEFORE commit** (per plan §13 Task 7 GOTCHA, lines 1236-1246):

```bash
python3 -c "
import json, jsonschema
schema = json.load(open('.claude/skills/brehon-conformance-audit/audit-metrics.schema.json'))
data = json.load(open('.claude/PRPs/audit-metrics/v1-federation-inbound-b.json'))
jsonschema.validate(data, schema)
print('OK')
"
```
If `jsonschema` not available, fall back to compute-metrics.sh's manual-key-check fallback by invoking it — exit 0 = schema-shape OK.

8. **Commit subject** — `feat(brehon-conformance-audit): dogfood report + seed metrics for v1-federation-inbound-b (Task 7)`.
9. **Commit body** — cite plan §13 Task 7, fix-impl-3 SHA `8b04e69a6`, expected metrics (recall@axis-4=1.0, precision@axis-4=1.0, latent-footgun-catch=1).

## 5. Validation gate (DoD per plan §15)

### 5.1 Pre-push validation (worker runs locally before push)

```bash
# 1. Dogfood report exists, has both snapshot sections
test -f .claude/PRPs/reports/conformance-audit-v1-federation-inbound-b-dogfood-2026-05-20.md
echo "report exists: $?"
grep -E "^## Snapshot 1: pre-fix-impl-3" .claude/PRPs/reports/conformance-audit-v1-federation-inbound-b-dogfood-2026-05-20.md
grep -E "^## Snapshot 2: current merged tip" .claude/PRPs/reports/conformance-audit-v1-federation-inbound-b-dogfood-2026-05-20.md
grep -E "^## Cross-snapshot diff" .claude/PRPs/reports/conformance-audit-v1-federation-inbound-b-dogfood-2026-05-20.md
grep -E "^## Ground-truth seeding" .claude/PRPs/reports/conformance-audit-v1-federation-inbound-b-dogfood-2026-05-20.md
grep -E "^## Metrics computed" .claude/PRPs/reports/conformance-audit-v1-federation-inbound-b-dogfood-2026-05-20.md

# 2. Seed metrics file exists + schema-valid + axis-4 prediction + ground_truth_runtime entry
python3 -c "
import json, jsonschema
schema = json.load(open('.claude/skills/brehon-conformance-audit/audit-metrics.schema.json'))
data = json.load(open('.claude/PRPs/audit-metrics/v1-federation-inbound-b.json'))
jsonschema.validate(data, schema)
preds = data['predictions']
axis4 = [p for p in preds if p['axis'] == '4' and 'inbox.rs' in p['target']]
assert len(axis4) >= 1, 'axis-4 prediction missing'
assert len(data['ground_truth_runtime']) >= 1, 'ground_truth_runtime missing'
gt = [g for g in data['ground_truth_runtime'] if g.get('evidence_commit_sha', '').startswith('8b04e69a6')]
assert len(gt) >= 1, 'ground_truth from fix-impl-3 missing'
print('OK')
"

# 3. .gitignore updated
grep -F ".claude/PRPs/audit-metrics/" .gitignore

# 4. METRICS.md ## Worked example backfilled
grep -F "recall@axis-4" .claude/skills/brehon-conformance-audit/METRICS.md
grep -F "precision@axis-4" .claude/skills/brehon-conformance-audit/METRICS.md

# 5. compute-metrics.sh runs end-to-end against the seed file
bash .claude/skills/brehon-conformance-audit/scripts/compute-metrics.sh .claude/PRPs/audit-metrics/v1-federation-inbound-b.json > .claude/PRPs/debug/brehon-conformance-audit-task7-metrics.log 2>&1
echo "metrics exit: $?"
grep -F "axis-4 precision: 1.000" .claude/PRPs/debug/brehon-conformance-audit-task7-metrics.log
grep -F "axis-4 recall: 1.000" .claude/PRPs/debug/brehon-conformance-audit-task7-metrics.log
```

### 5.2 §15 validate-pending-laptop DQ (raise after worker push)

After worker pushes the worker branch, raise a `kind: "validate-pending-laptop"` DQ entry naming the compute-metrics smoke test against the REAL seed file (not the fixture).

```json
{
  "id": <next-id>,
  "from": "impl",
  "kind": "validate-pending-laptop",
  "timestamp": "<UTC ISO 8601>",
  "branch": "<worker-branch-name>",
  "phase_task": 7,
  "commands": [
    "bash .claude/skills/brehon-conformance-audit/scripts/compute-metrics.sh .claude/PRPs/audit-metrics/v1-federation-inbound-b.json"
  ],
  "result": null,
  "log_slice": null,
  "failed_commands": null,
  "answered_by": null,
  "resolved_at": null
}
```

After raising, commit + push to worker branch:

```
git add .claude/decision-queue.json
git commit -m "chore(decision-queue): impl raised DQ #<id> — task 7 validate-pending-laptop"
git push origin <worker-branch>
```

**CRITICAL** (per DQ #316 retro): the `commands` field MUST match §5.1 / §5.2 verbatim. Do NOT mis-transcribe `bash scripts/brehon/cargo-check.sh --workspace --features full` from a different brief template — that command is for Rust-impact tasks, not skill-asset tasks.

## 6. Report content guidance (anti-hallucination)

The dogfood report is the calibration evidence. The worker MUST cite REAL data, not hallucinated:

- **Snapshot 1 ground-truth target**: from the fix-impl-3 commit body — `receive_remote_moderation_label` in `crates/apub/activities/src/governance/inbox.rs` at L735. Run `git show 649871f7d:crates/apub/activities/src/governance/inbox.rs | head -800 | tail -150` to inspect; the `.unwrap_or_default()` on L742-743 is the smoking gun.
- **Snapshot 2 fix**: run `git show 4a60667c9:crates/apub/activities/src/governance/inbox.rs | head -800 | tail -150` — the same function now uses `.ok_or_else(|| LemmyErrorType::Unknown(...))?`.
- **Cross-snapshot diff**: cite the SHA pair + the file:line + the before/after idiom. Sibling: L105 (`receive_remote_sanction_notice`) or L261-271 (`receive_remote_trust_attestation`) — both pre-existing with the correct `.ok_or_else()` idiom.
- **Ground-truth seeding**: paste the commit body of `8b04e69a6` (or quote the key lines: "## 1-hunk change", "Mirrors receive_remote_trust_attestation L261-271 verbatim").
- **Metrics computed**: paste actual stdout from `compute-metrics.sh`. Expected: precision 1.000, recall 1.000, latent-footgun catch rate 1, lead time = author-date of `8b04e69a6` minus `run_at` (approx).

## 7. Failure-class mapping

| Symptom | Action |
|---|---|
| `compute-metrics.sh` smoke against seed JSON exits non-zero | `kind: "blocker"` DQ — schema validation or computation bug; do not push. Attach output. |
| Schema validation fails | `kind: "blocker"` DQ — seed JSON shape wrong. Inspect the schema vs the seed via diff. Do not push. |
| Smoke output missing "axis-4 precision: 1.000" or "axis-4 recall: 1.000" | `kind: "blocker"` DQ — prediction/ground-truth mismatch. Verify `target` strings match between predictions and ground_truth_runtime entries (same file:line). |
| Snapshot 1 `git show` returns empty | `kind: "blocker"` DQ — SHA unreachable; verify the lane has fetched fed-in-b history (`git fetch origin phase-v1-federation-inbound-b` — the SHAs are on origin even though the branch is merged + deleted, as merge commit `4480a1bdb` references them). |
| Any non-4-file diff in `git status --short` | `kind: "blocker"` DQ — accidental contamination. |
| Daemon finalize-merge conflict on `.claude/decision-queue.json` | Standard pattern per `feedback_junior_finalize_merge_race_lossless_reconcile.md`. |

## 8. Out of scope

- LIVE skill invocation (the dogfood READS history).
- Cargo invocations (Watchpoint #3).
- Adding tier-2 or tier-3 axis predictions for OTHER files — focus on the axis-4 Finding 6.1 calibration only.
- Editing the schema (Task 4 output; stays as-is).
- Editing axis sub-files.
- Adding a new lesson file (defer to Task 12 lesson-promotion phase).
- Cleaning up Snapshot 2 — it's the merged tip; that's the user's reference state.
