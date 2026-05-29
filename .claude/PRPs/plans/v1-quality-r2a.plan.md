# Plan: v1-quality-r2a — PR #155 carry-forward bundle (DQ duration lint + C3 deferral DQ)

> **Split notice (advisor, 2026-05-28):** this plan is the **r2a** half of the
> original `v1-quality-r2` scope, narrowed at gate-1 per user instruction
> after the planner's split-DQ recommendation. The original full-scope plan
> at `v1-quality-r2.plan.md` (1057 lines) remains on this phase branch as
> historical context for `v1-quality-r2b` planning. Branch name stays
> `phase-v1-quality-r2` (no rename); plan filename signals scope.
>
> **r2a (this plan)** covers C2 (Tasks T1 = DQ lint scripts + sweep) + C3
> (Task T2 = deferral DQ for #158).
>
> **r2b (future)** will cover C1 (T3 = 14 fixtures doc-comment headers) +
> C4 (T4 = EnvVarGuard hoist + boot_context refactor + T5 = 13 setter
> sites) — scheduled after r2a merges. Cut a fresh `phase-v1-quality-r2b`
> from updated trunk; r2a + r2b have zero file overlap (r2a touches only
> scripts + DQ JSON; r2b is e2e.rs-only).

## 1. Summary

Retire **2 of 5** PR #155 carry-forward issues filed during the v1-RT-r3 BM
triage (2026-05-28): #157 (DQ negative-duration lint + bulk sweep +
precheck wiring) and #158 (formal deferral DQ for `emit_reputation_event`
helper extraction). Headline acceptance: `dq-lint-durations.sh` exits 0
on trunk + non-zero on a synthetic back-dated entry; `precheck.sh` exits
0; one new `kind: "log"` DQ entry records the #158 deferral verbatim per
WP-2 (premature-DRY footgun). Zero `crates/**` edits; zero migrations;
zero new dependencies.

## 2. Source

- Brief: `.claude/PRPs/briefs/v1-quality-r2-planning-1.md` (clusters C2 +
  C3 sections); narrowed-scope authorisation by user at gate-1 2026-05-28.
- Full-scope predecessor plan (historical context only):
  `.claude/PRPs/plans/v1-quality-r2.plan.md` on this phase branch.
- PR #155 findings YAML: `.claude/PRPs/reviews/pr-155-findings.yaml`
  (rows `cr-4` → #157, `cr-7` → #158).
- Issues: #157 (DQ negative-duration data corruption risk), #158
  (`emit_reputation_event` shared helper extraction — DEFERRED).
- Lessons that materially shaped the plan:
  - `feedback_principles_not_rules.md` — drove the WP-2 DEFER decision.
  - `feedback_complexity_score_pre_split.md` — drove the split decision
    (full-scope was 10/10; r2a alone is 1/10).
  - `feedback_explicit_file_arrays_on_tasks.md` — every §13 task carries
    `creates:` + `modifies:` + `requires:` YAML.
  - `feedback_falsifiable_hypothesis_before_structural_fix.md` — T1's
    "Option B" (resolve from git history) is hypothesis-class; T1 falls
    back to "Option A floor" when git history is ambiguous.
  - `feedback_principles_not_rules.md` (twice — also drives Option A
    floor on the bulk sweep).
- ADRs: none affected.

## 3. Problem statement

Two distinct defect classes from PR #155, both addressable without
touching `crates/**`:

1. **DQ negative-duration data corruption risk (#157).** Three DQ entries
   (id=315, id=1b8527b076d4-001, id=81719cf8ca8d-001 — the real back-dated
   entries as of phase-v1-quality-r2 HEAD; #157 body uses file LINE numbers
   3999/4007/4016/4035, not DQ entry ids — see DQ #3ef987b66db4-001 for the
   investigation) carry `resolved_at < timestamp`,
   producing negative durations in any time-series consumer. No script
   gates this on write; the issue can recur on every
   `dq-v3-append-fragment.sh` invocation that lets the caller pre-compute
   `resolved_at` before `timestamp`. **Tied to Task 1** (lint script +
   precheck wiring + one-shot bulk sweep).
2. **`emit_reputation_event` shared helper extraction (#158).** Code
   duplication finding (two consumers: `admin_emergency_remove.rs:448`
   and `submit_jury_vote.rs:1096` carry byte-identical bodies). The
   issue body itself flags "wait until a third consumer materialises
   before introducing the abstraction (premature DRY is its own footgun)".
   Per `feedback_principles_not_rules.md`. **Tied to Task 2** — file a
   `kind: "log"` DQ recording the deferral; issue #158 stays OPEN.

The remaining 3 PR #155 carry-forwards (#156 doc audit, #159 boot_context
env-leak, #160 LEMMY_DATABASE_URL setter sites — all e2e.rs-confined)
are deferred to `v1-quality-r2b`.

## 4. Solution statement

Three tasks plus Task 0 (harness audit) and a retro task. **All tasks
are pure metadata work** (scripts + JSON + DQ append) — zero Rust
changes, zero cargo gates beyond Task 0's baseline.

| Task | Cluster | Surface | Edit shape | Risk |
|---|---|---|---|---|
| T0 | — | (harness audit only) | n/a | — |
| T1 | C2 | `scripts/brehon/dq-lint-durations.sh` (new), `scripts/brehon/precheck.sh` (new), `.claude/decision-queue.json` (bulk sweep edit) | Author shell scripts + run sweep | low |
| T2 | C3 | `.claude/decision-queue.json` (append deferral entry via `dq-v3-append-fragment.sh`) | Single DQ append + close-issue comment | low |
| T3 | — | `.claude/PRPs/reports/v1-quality-r2a-retro.md` (new) | Retro authorship per `feedback_retro_not_report.md` | low |

The `§13` task order is naturally serial: T1 + T2 BOTH modify
`.claude/decision-queue.json` (T1 bulk sweep + T2 append) → cohort YAML
overlap rejects `[P]`. No `[P]` markers in this plan.

## 5. Metadata

- **Phase:** `v1-quality-r2a` (narrowed scope; branch name stays `phase-v1-quality-r2`)
- **Branch:** `phase-v1-quality-r2` (cut from `governance-v0` at `1edb8b94c`; bm-cut log at `d5f0114eb`)
- **Target impl-task model:** `sonnet-4-6` (default)
- **Estimated tasks:** 4 (Task 0 pre-flight + 2 impl tasks + 1 retro)
- **Estimated cargo budget:** N/A (no Rust changes; Task 0 cargo probes
  run inline on EliteDesk per RT-r3 precedent; Shape G SUSPENDED but
  irrelevant — no per-task cargo gates beyond Task 0)
- **Forbidden-window applicability:** Task 0 binding (runs cargo probes
  inline on EliteDesk); Tasks 1–2 non-binding (no cargo)
- **Complexity score:** **1/10** (well below Sonnet threshold 8)

### 5.1 Complexity factor breakdown

| Factor | Weight | This plan | Source |
|---|---|---|---|
| §13 impl tasks above 5 | +1 each | **0** | 2 impl tasks (T1, T2); count excludes Task 0 + retro; 2 − 5 < 0 → 0. |
| Migrations touched | +2 each | **0** | None. |
| Crates touched | +1 each | **0** | `scripts/brehon/*` and `.claude/decision-queue.json` are NOT Cargo crates. |
| `crates/server/tests/e2e.rs` edits | +3 each | **0** | Zero edits. |
| New ADR-affecting decisions | +2 each | **0** | None. |
| Cargo budget peak above 6 GB | +1 per GB | **0** | No per-task cargo gates beyond Task 0. |
| **Narrative calibration** | — | **+1** | DQ JSON mutation (Hard refusal #6 atomic protocol) carries non-trivial concurrency discipline. |
| **Total** | — | **1** | Well below Sonnet threshold 8. **No split-DQ.** |

### 5.2 Per-task complexity ceiling (Sonnet)

- `count(union(creates, modifies)) ≤ 4` files per task
- `count(distinct crates/<X>/ prefixes) ≤ 2` crates per task

Verified: T1 = 3 files (`dq-lint-durations.sh`, `precheck.sh`,
`decision-queue.json`), 0 crates. T2 = 1 file, 0 crates. T3 = 1 file, 0
crates. All under ceiling.

## 6. Relationship to other v1-quality-* sub-phases

- **Predecessor (merged):** `v1-quality-r1` (PR #145 `eec20a102`,
  merged 2026-05-22). No carry-forward overlap.
- **Triggering phase (merged):** `v1-RT-r3` (PR #155 `5ebd8ae23`, merged
  2026-05-28). The 5 issues this lane addresses were filed during PR
  #155's BM triage; r2a addresses 2 of them.
- **Successor (planned):** `v1-quality-r2b` — addresses C1 (#156) +
  C4 (#159 + #160) on a fresh phase branch cut from trunk after r2a
  merges. Zero file overlap with r2a (r2a = scripts + DQ JSON; r2b =
  e2e.rs).
- **Concurrent lane:** `v1-redaction-r1` (in-flight 2026-05-28). Zero
  file overlap with this lane. 2 concurrent lanes ≪ cohort-3
  `.git/index.lock` threshold.

## 7. Preflight guardrails inherited from prior phases

- **R5** (per JM-b retro): Task 0 enumerates ALL probes explicitly.
- **R8** (per `feedback_features_full_p_crate_incompatible.md`): never
  `-p <crate> --features full` unless the crate defines `full` (Task 0
  cargo probes use `--workspace --features full` uniformly).
- **R9** (per `feedback_validate_pending_laptop_must_use_wrapper.md`):
  every cargo gate goes through the wrapper. Task 0 uses
  `bash scripts/brehon/cargo-*.sh` (Linux daemon worker — translated
  from plan's `cmd //c "...bat"` Windows form per
  `feedback_handover_assumptions_need_empirical_verification.md`).
- **R10** (per `cargo-output-capture.md`): all cargo invocations
  redirect to `.claude/PRPs/debug/<phase>-<task>-<verb>.log`.

## 8. Flow design

```
Task 0 (harness audit)
  │  no commit
  ▼
Task 1 (C2: dq-lint-durations.sh + precheck.sh + bulk sweep)
  │  modifies .claude/decision-queue.json, creates scripts/brehon/*
  ▼
Task 2 (C3: deferral DQ for #158)
  │  modifies .claude/decision-queue.json (append-only)
  ▼
Task 3 (Retro)
  │  creates .claude/PRPs/reports/v1-quality-r2a-retro.md
  ▼
bm-pr / bm-poll-cr / bm-triage / bm-merge (BM session)
```

## 9. Mandatory reading

Files the impl-task subagent MUST Read before its first Edit on each task:

**Schema / type definitions (every task):**
- `.claude/rules/decision-queue.md` (DQ schema-v3 + attribution; Hard
  refusals #1, #6, #8, #9)
- `.claude/rules/multi-lane-worktree.md` Hard refusal #6 (atomic
  read-mutate-commit-push protocol for DQ JSON)

**Existing patterns (per-task MIRROR refs):**
- T1: `scripts/brehon/resolve-dq-canonical.sh` (DQ-iteration pattern);
  `scripts/brehon/dq-schema-v3-migrate.sh` (JSON mutation idiom);
  `scripts/brehon/dq-v3-append-fragment.sh` (append helper).
- T2: any historical `kind: "log"`, `from: "planner"`,
  `answered_by: "planner"` entry in `.claude/decision-queue.json`'s
  `resolved[]` for shape reference.

**Lessons (impl-task brief authoring):**
- `feedback_principles_not_rules.md` — premature-DRY footgun (T2 deferral
  rationale).
- `feedback_falsifiable_hypothesis_before_structural_fix.md` — T1 Option
  B path (resolve from git history) is hypothesis-class; if git log
  doesn't unambiguously identify a single resolving commit, fall back to
  Option A floor (`resolved_at = timestamp`).
- `feedback_explicit_file_arrays_on_tasks.md` — every task carries FILES
  YAML.
- `feedback_dq_v3_append_via_helper_script.md` — T2 uses the append
  helper; no hand-rolled fragment append.
- `feedback_windows_backslash_path_dq_via_write_fragment.md` — fragment
  files authored via Write tool then read by the helper; no `python -c`
  with backslash paths.

## 10. Patterns to mirror

### 10.1 dq-lint-durations.sh shape (T1 target)

**Mirror:** structurally similar to `scripts/brehon/dq-schema-v3-migrate.sh`
and `scripts/brehon/resolve-dq-canonical.sh`.

Skeleton (~80 LOC):

```bash
#!/usr/bin/env bash
# scripts/brehon/dq-lint-durations.sh — flag DQ entries where resolved_at < timestamp.
# Exits 0 if no negative-duration entries; non-zero with a list otherwise.
# Composite-id-aware (schema-v3): reports by `id` verbatim.
#
set -euo pipefail
DQ_PATH="${1:-.claude/decision-queue.json}"
[ -f "$DQ_PATH" ] || { echo "FATAL: not found: $DQ_PATH" >&2; exit 2; }

python3 -c "
import json, sys
from datetime import datetime
def parse(t):
    if not t: return None
    return datetime.fromisoformat(t.replace('Z', '+00:00'))
with open('$DQ_PATH') as f:
    dq = json.load(f)
bad = []
for arr in ('pending', 'resolved'):
    for e in dq.get(arr, []):
        ts = parse(e.get('timestamp'))
        ra = parse(e.get('resolved_at'))
        if ts and ra and ra < ts:
            delta = ts - ra
            bad.append((str(e.get('id')), e.get('timestamp'), e.get('resolved_at'), str(delta)))
for (eid, ts, ra, d) in bad:
    print(f'DQ-LINT FAIL: entry \"{eid}\" has resolved_at ({ra}) earlier than timestamp ({ts}) by {d}')
sys.exit(1 if bad else 0)
"
```

### 10.2 precheck.sh wiring (T1 target — new file)

```bash
#!/usr/bin/env bash
# scripts/brehon/precheck.sh — run before queueing Junior real-work tasks.
# Exits 0 if all gates pass; non-zero on failure.
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
echo "[precheck] dq-lint-durations.sh ..."
"$SCRIPT_DIR/dq-lint-durations.sh"
echo "[precheck] OK"
```

### 10.3 Deferral DQ shape (T2 target — `kind: "log"` entry)

**Mirror:** any historical `kind: "log"`, `from: "planner"`,
`answered_by: "planner"` entry.

Fragment JSON for T2 to author via Write tool at
`.claude/PRPs/debug/v1-quality-r2a-c3-defer.json`:

```json
{
  "from": "planner",
  "kind": "log",
  "timestamp": "<ISO8601 at T2 execution time>",
  "question": "C3 (Issue #158) emit_reputation_event helper extraction deferred per premature-DRY gate",
  "answer": "v1-quality-r2 brief WP-2 default: defer C3 unless planner identifies a 3rd consumer in near-term roadmap. As of 2026-05-28, federation_inbound lane status='done' in .claude/PRPs/v1-roadmap.json — no upcoming sub-phase introduces a 3rd reputation-event emitter. Therefore: 2 consumers as of plan author time; revisit when 3rd materialises. Per feedback_principles_not_rules.md.",
  "options": ["defer", "extract"],
  "context": "Two existing emitter sites: crates/api/api/src/governance/admin_emergency_remove.rs:448 (emit_reputation_event_local) and crates/api/api/src/governance/submit_jury_vote.rs:1096 (emit_reputation_event). Bodies are byte-identical. Extraction target: crates/api/api/src/governance/reputation_helpers.rs. Trigger condition for v1-quality-r3 follow-on: any sub-phase introduces a 3rd reputation-event emit path.",
  "answered_by": "planner",
  "resolved_at": "<ISO8601 same as timestamp>",
  "approved_by": null,
  "approved_at": null
}
```

The helper `dq-v3-append-fragment.sh` injects `id` automatically.

## 11. Files to change

**Scripts (T1):**
- `scripts/brehon/dq-lint-durations.sh` — NEW (~80 LOC)
- `scripts/brehon/precheck.sh` — NEW (~25 LOC)

**Decision queue (T1 + T2):**
- `.claude/decision-queue.json` — bulk sweep of 4 entries (T1) + append
  C3 deferral entry (T2)

**Reports (T3):**
- `.claude/PRPs/reports/v1-quality-r2a-retro.md` — NEW

**Crate manifests, migrations, public API surface:** none modified.

## 12. NOT building in v1-quality-r2a

- **C1 (Issue #156)** — fixtures doc-comment audit. Deferred to r2b.
- **C4 (Issue #159 + #160)** — EnvVarGuard refactor. Deferred to r2b.
- **C3 helper extraction (Issue #158)** — deferred per WP-2 (premature-DRY).
  T2 files the deferral DQ; issue #158 stays OPEN with a comment.
- **Bulk sweep Option B (re-author from git history)** — only if git log
  unambiguously identifies the resolving commit; T1 brief picks
  per-entry and surfaces ambiguous cases as a blocker DQ.

## 13. Step-by-step tasks

> Cohort dispatch: no `[P]` markers. T1 + T2 both mutate
> `.claude/decision-queue.json` → serial.

### Task 0: Pre-flight harness audit + branch verification

**Goal:** verify environment ready; confirm base is `phase-v1-quality-r2`
(via ancestor check on `junior/*` worker branch); confirm wrappers honour
flags; confirm clippy baseline clean.

**FILES:**
```yaml
creates: []
modifies: []
requires: []
```

**Probes:**

```bash
# Probe 0 — Docker daemon
docker ps > /dev/null 2>&1 && echo "DOCKER OK" || { echo "DOCKER NOT RUNNING"; exit 1; }

# Probe 1 — branch base confirmation (Junior on junior/*)
git branch --show-current
git merge-base --is-ancestor phase-v1-quality-r2 HEAD && echo "BASE_OK phase-v1-quality-r2 is ancestor" || echo "BASE_MISMATCH — STOP"
# EXPECT: junior/...; BASE_OK

# Probe 2 — wrapper sanity (cargo-check honors -p)
bash scripts/brehon/cargo-check.sh -p lemmy_utils > .claude/PRPs/debug/v1-quality-r2a-task0-check-p.log 2>&1
echo "check-p exit: $?"
# EXPECT: exit 0

# Probe 3 — feature flag activation
bash scripts/brehon/cargo-check.sh --workspace --features full > .claude/PRPs/debug/v1-quality-r2a-task0-check-features.log 2>&1
echo "check-features exit: $?"
# EXPECT: exit 0

# Probe 4 — negative-feature exit-code propagation
bash scripts/brehon/cargo-check.sh --workspace --features nonexistent_xyz > .claude/PRPs/debug/v1-quality-r2a-task0-negative.log 2>&1
echo "negative exit: $?"
# EXPECT: NON-ZERO (typically 101)

# Probe 5 — clippy baseline
bash scripts/brehon/cargo-clippy.sh --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/v1-quality-r2a-task0-clippy.log 2>&1
echo "clippy exit: $?"
# EXPECT: exit 0

# Probe 6 — confirm 4 named DQ entries exist and are currently back-dated
python3 -c "
import json
d = json.load(open('.claude/decision-queue.json'))
ids = {315, '315', '1b8527b076d4-001', '81719cf8ca8d-001'}
hits = [e for e in d.get('resolved', []) if e.get('id') in ids or e.get('id_v1') in ids]
print(f'found {len(hits)} of 3 target entries')
backdated = [e for e in hits if e.get('resolved_at') and e.get('timestamp') and e['resolved_at'] < e['timestamp']]
print(f'{len(backdated)} are currently back-dated')
"
# EXPECT: found 3 of 3; 3 are currently back-dated

# Probe 7 — confirm dq-v3-new-entry.sh + dq-v3-append-fragment.sh present
ls -la scripts/brehon/dq-v3-*.sh
# EXPECT: both scripts present

# Probe 8 — confirm no other PR open touches r2a's IMPLEMENT files
gh pr list --repo barrie-cork/lemmy --state open --json number,title,headRefName,files \
  --jq '.[] | select(.files[]?.path | test("scripts/brehon/dq-lint|scripts/brehon/precheck|decision-queue\\.json")) | {number, title, headRefName}'
# EXPECT: empty output
```

**EXPECT block:**
- Probes 0, 1, 2, 3, 5, 6, 7, 8 exit 0
- Probe 4 exits NON-ZERO (negative test)
- Probe 1 returns `junior/...` + `BASE_OK`
- Probe 6 confirms 3 of 3 target entries present + all 3 back-dated
- Probe 7 confirms helper scripts present
- Probe 8 confirms no concurrent PR overlap

**No commit at Task 0** — verification only. Any probe failure files a
`kind: "blocker"` DQ pending and stops.

### Task 1: C2 — DQ negative-duration lint + precheck wiring + bulk sweep

**Goal:** add a non-zero-exit lint that catches `resolved_at < timestamp`
DQ entries; wire it as the first gate in a new `scripts/brehon/precheck.sh`;
one-shot floor-sweep the 3 currently back-dated entries (id=315,
id=1b8527b076d4-001, id=81719cf8ca8d-001; #157's body LINE numbers 3999/4007/4016/4035
were CR-finding file-line references, not DQ ids — confirmed by Task 0 worker
investigation in DQ #3ef987b66db4-001).

**FILES:**
```yaml
creates:
  - scripts/brehon/dq-lint-durations.sh
  - scripts/brehon/precheck.sh
modifies:
  - .claude/decision-queue.json
requires: []
```

**ACTION:** ship the lint script per §10.1 + the precheck wrapper per
§10.2 + execute a one-shot floor sweep on `.claude/decision-queue.json`
entries `id=315`, `id=1b8527b076d4-001`, `id=81719cf8ca8d-001`. Verify post-sweep that the lint
exits 0; verify the lint exits non-zero on a synthetic back-dated entry.

**IMPLEMENT (file 1 of 3):** in `scripts/brehon/dq-lint-durations.sh`,
write the script per §10.1. Honour: `#!/usr/bin/env bash`,
`set -euo pipefail`, accept positional `${1:-.claude/decision-queue.json}`,
iterate `pending[]` + `resolved[]`, compare `resolved_at` vs `timestamp`,
emit `DQ-LINT FAIL: entry "<id>" has resolved_at (<rfc3339>) earlier than timestamp (<rfc3339>) by <duration>` on stdout, exit non-zero on any finding, support both pre-v3 integer ids AND v3 composite `<session>-<seq>` ids verbatim. `chmod +x`.

**IMPLEMENT (file 2 of 3):** in `scripts/brehon/precheck.sh`, write the
wrapper per §10.2. The script calls `dq-lint-durations.sh` (paths via
`SCRIPT_DIR`) and surfaces non-zero exits. `chmod +x`.

**IMPLEMENT (file 3 of 3):** in `.claude/decision-queue.json`, edit 3
named entries' `resolved_at`:
- For entries id=315, id=1b8527b076d4-001, id=81719cf8ca8d-001: if
  `resolved_at < timestamp`, set `resolved_at = timestamp` (Option A floor).
- Option B (resolve from git history) is allowed PER ENTRY only if
  `git log --all --format='%H %cI' --grep='<id-citation-pattern>' | head -1`
  unambiguously returns a single commit-date AFTER `timestamp` AND BEFORE
  the current `resolved_at`. Per `feedback_falsifiable_hypothesis_before_structural_fix.md`
  — ambiguous git-log output triggers Option A floor for that entry.
- Use the atomic read-mutate-commit-push protocol per
  `multi-lane-worktree.md` Hard refusal #6: `git fetch` →
  `python3 read+mutate` → verify → `git add` → `git commit` → `git push`
  as a single uninterrupted sequence.

**MIRROR:** `scripts/brehon/resolve-dq-canonical.sh` (DQ-iteration pattern);
`scripts/brehon/dq-schema-v3-migrate.sh` (JSON mutation idiom).

**GOTCHA:** `.claude/decision-queue.json` may be concurrently written by
the advisor session on `governance-v0`. Re-fetch immediately before
read, immediately before write. The sweep is a SINGLE commit, not 4
commits.

**VALIDATE (story-checkpoint feeds §16a Story 1):**

```bash
bash scripts/brehon/dq-lint-durations.sh > .claude/PRPs/debug/v1-quality-r2a-task1-postsweep.log 2>&1
echo "exit: $?"
tail -20 .claude/PRPs/debug/v1-quality-r2a-task1-postsweep.log
# EXPECT: exit 0; no DQ-LINT FAIL lines

# Synthetic-negative check
cp .claude/decision-queue.json .claude/PRPs/debug/dq-fixture.json
python3 -c "
import json
with open('.claude/PRPs/debug/dq-fixture.json') as f: dq = json.load(f)
if dq['resolved']:
    dq['resolved'][0]['resolved_at'] = '2020-01-01T00:00:00+00:00'
with open('.claude/PRPs/debug/dq-fixture.json', 'w') as f: json.dump(dq, f)
"
bash scripts/brehon/dq-lint-durations.sh .claude/PRPs/debug/dq-fixture.json > .claude/PRPs/debug/v1-quality-r2a-task1-synthetic.log 2>&1
echo "exit: $?"
# EXPECT: exit non-zero; output contains DQ-LINT FAIL

bash scripts/brehon/precheck.sh > .claude/PRPs/debug/v1-quality-r2a-task1-precheck.log 2>&1
echo "exit: $?"
# EXPECT: exit 0; "[precheck] OK"
```

**Commit subject:** `feat(scripts/brehon): add dq-lint-durations.sh + precheck.sh + sweep back-dated DQ entries (closes #157, task 1)`.

### Task 2: C3 — file `kind: "log"` deferral DQ for emit_reputation_event helper extraction (#158)

**Goal:** record the planner's WP-2 DEFER decision as an immutable DQ
log entry; do NOT modify any `crates/**` files; issue #158 stays open
with a deferral comment filed by the BM session post-merge.

**FILES:**
```yaml
creates: []
modifies:
  - .claude/decision-queue.json
requires:
  - task: 1
    reason: "Task 1 sweeps existing back-dated entries first; Task 2 appends a new entry to the cleaned-state DQ. Re-fetch trunk between T1 commit and T2 read."
```

**ACTION:** generate a composite v3 DQ id via the helper, author the
deferral fragment via Write tool at
`.claude/PRPs/debug/v1-quality-r2a-c3-defer.json`, append via
`bash scripts/brehon/dq-v3-append-fragment.sh <fragment.json>`. Single
commit. No `--pending` flag (entry goes directly to `resolved[]`).

**IMPLEMENT (file 1 of 1):** in `.claude/decision-queue.json`, append
exactly one entry to `resolved[]`:
1. Write fragment file at `.claude/PRPs/debug/v1-quality-r2a-c3-defer.json`
   with the JSON shown in §10.3 (substitute `<ISO8601 at T2 execution time>`
   with the actual current timestamp).
2. Run `bash scripts/brehon/dq-v3-append-fragment.sh .claude/PRPs/debug/v1-quality-r2a-c3-defer.json`
   (NO `--pending` flag — entry goes to `resolved[]`).
3. The helper computes `id` via `bash scripts/brehon/dq-v3-new-entry.sh`,
   injects it, appends to `resolved[]`, writes the file.
4. Verify the entry exists (see VALIDATE below).
5. Re-run `bash scripts/brehon/dq-lint-durations.sh` to confirm no
   negative durations introduced.

**MIRROR:** any historical `kind: "log"`, `from: "planner"`,
`answered_by: "planner"` entry in the resolved array.

**GOTCHA:** Hard refusals #1, #8, #9 (`decision-queue.md`): never write
`answered_by: "advisor"` or `approved_by: <non-null>` from an impl-task
session; always use the helper; never use `max(all_ids)+1`.

**VALIDATE (story-checkpoint feeds §16a Story 2):**

```bash
python3 -c "
import json
d = json.load(open('.claude/decision-queue.json'))
hits = [e for e in d['resolved'] if e.get('answered_by') == 'planner' and 'C3' in e.get('question', '') and '#158' in e.get('question', '')]
assert len(hits) == 1, f'expected 1 C3 deferral entry, found {len(hits)}'
print('C3 deferral DQ entry id:', hits[0]['id'])
assert hits[0]['kind'] == 'log'
assert hits[0]['answered_by'] == 'planner'
assert hits[0].get('approved_by') is None
print('OK')
" > .claude/PRPs/debug/v1-quality-r2a-task2-verify.log 2>&1
echo "exit: $?"

bash scripts/brehon/dq-lint-durations.sh > .claude/PRPs/debug/v1-quality-r2a-task2-lint.log 2>&1
echo "exit: $?"
# EXPECT: exit 0
```

**Commit subject:** `chore(decision-queue): planner-defer C3 emit_reputation_event helper (Issue #158, task 2)`.

### Task 3: Retro

**Goal:** author the phase retro per `feedback_retro_not_report.md` +
`feedback_four_role_retro_signals.md`.

**FILES:**
```yaml
creates:
  - .claude/PRPs/reports/v1-quality-r2a-retro.md
modifies: []
requires:
  - task: 2
    reason: "Retro is authored after all impl tasks ship."
```

**ACTION:** author `.claude/PRPs/reports/v1-quality-r2a-retro.md`
following the retro template + four-role-signals discipline. Length
target: 150-300 lines.

**Commit subject:** `docs(retro): v1-quality-r2a phase retro - PR #155 carry-forward bundle (#157 + #158)`.

## 14. Testing strategy

- **Static analysis (Task 0 baseline):** workspace check + clippy.
- **Functional tests (Task 1):** synthetic-negative check on
  `dq-lint-durations.sh`; positive check on live `decision-queue.json`
  post-sweep.
- **Functional tests (Task 2):** Python assertion the deferral entry is
  shape-correct and lint still passes.
- **No per-task cargo gates beyond Task 0** — r2a touches zero Rust.

## 15. Validation commands (DoD)

### 15.1 Static analysis (Task 0 only — no per-task cargo for T1/T2)

Task 0 already runs static analysis baseline (Probes 2, 3, 5). No
re-run after T1/T2 because neither touches Rust.

### 15.2 Lint script self-test (T1, T2)

```bash
bash scripts/brehon/dq-lint-durations.sh > .claude/PRPs/debug/v1-quality-r2a-doda-lint.log 2>&1
echo "exit: $?"
# EXPECT: exit 0
```

### 15.3 Cross-cutting verification

- [ ] R5: Task 0 enumerates all 9 probes.
- [ ] R8: `--workspace --features full` uniformly (no `-p <crate> --features full`).
- [ ] R9: every cargo gate uses `bash scripts/brehon/cargo-*.sh`.
- [ ] R10: every cargo invocation redirects to a file.
- [ ] `dq-lint-durations.sh` exits 0 on `.claude/decision-queue.json` post-T1+T2.
- [ ] `dq-lint-durations.sh` exits non-zero on a synthetic back-dated fixture.
- [ ] `precheck.sh` sources `dq-lint-durations.sh` via `SCRIPT_DIR`.
- [ ] No edits to `crates/**`, `migrations/**`, `tests/**` (r2a is zero-Rust by design; PR also includes the plan files and `.claude/PRPs/debug/v1-quality-r2a-c3-defer.json` per CR cr-7).
- [ ] Issues #157 closed with PR reference at merge time; #158 has a deferral comment + remains open.

### 15.4 No e2e gate

r2a runs zero e2e tests. The e2e gate moves to r2b.

## 16. Acceptance criteria

- [ ] Task 0 (pre-flight) all 9 probes pass
- [ ] Task 1 commit lands on `phase-v1-quality-r2`
- [ ] Task 2 commit lands on `phase-v1-quality-r2`
- [ ] §15.2 (`dq-lint-durations.sh` exit 0 post-sweep) green
- [ ] §15.3 cross-cutting — all boxes ticked
- [ ] §16a stories — both stories `[done]`
- [ ] Retro committed per §13 Task 3
- [ ] PR opens against `governance-v0` with `--repo barrie-cork/lemmy`
- [ ] Issue #157 closed with merge-commit + PR references
- [ ] Issue #158 carries a deferral comment cross-referencing the C3
      deferral DQ id; remains OPEN

## 16a. Stories (independently-testable behaviour units)

### Story 1: DQ negative-duration entries are blocked at gate time

- **Composing tasks:** Task 1
- **Checkpoint command (positive):** `bash scripts/brehon/dq-lint-durations.sh; echo "exit: $?"`
- **Expected output:** `exit: 0`.
- **Checkpoint command (negative):** synthetic-fixture invocation per
  Task 1 §VALIDATE block.
- **Expected output:** non-zero exit, `DQ-LINT FAIL: entry "<id>" has resolved_at ... earlier than timestamp ...`.
- **Brief-Scope outputs to verify:**
  - `scripts/brehon/dq-lint-durations.sh` exists, has `#!/usr/bin/env bash`,
    `set -euo pipefail`.
  - `scripts/brehon/precheck.sh` exists, sources the lint via `SCRIPT_DIR`.
  - `.claude/decision-queue.json` entries id=315, id=1b8527b076d4-001,
    id=81719cf8ca8d-001 have `resolved_at >= timestamp`.

### Story 2: C3 helper extraction is deferred with audit trail

- **Composing tasks:** Task 2
- **Checkpoint command:** Task 2 §VALIDATE Python block.
- **Expected output:** `OK`.
- **Brief-Scope outputs to verify:**
  - `.claude/decision-queue.json` `resolved[]` contains exactly one
    entry whose `question` contains `"C3 (Issue #158)"`, `kind == "log"`,
    `answered_by == "planner"`, `approved_by is None`.
  - The entry's `answer` cites the 2-consumer enumeration and names
    `feedback_principles_not_rules.md`.

## 17. Completion checklist

- [ ] Task 0 audit complete (9 probes confirmed)
- [ ] Task 1 (lint + sweep) committed
- [ ] Task 2 (deferral DQ) committed
- [ ] §15.2 lint script post-sweep exit 0
- [ ] §16a stories all `[done]`
- [ ] Retro committed (Task 3)
- [ ] PR opened by BM session against `governance-v0`
- [ ] CodeRabbit review complete with findings triaged
- [ ] `/brehon-verify` report shows both stories ✓
- [ ] Issue #157 closed; #158 comment posted (deferral cross-ref)

## 18. Risks and mitigations

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Concurrent advisor write to `.claude/decision-queue.json` clobbers T1 sweep | LOW | MED | Hard refusal #6 atomic protocol; re-fetch immediately before mutate; verify post-push |
| Git log for one of the 3 entries returns multiple resolving-commit candidates | MED | LOW | Per-entry: Option B requires a single unambiguous candidate; default Option A floor |
| `dq-lint-durations.sh` regex misses an edge-case timestamp format | LOW | MED | Python `fromisoformat` handles all valid ISO-8601; non-ISO timestamps surface as `None` and skip the comparison |
| `dq-v3-append-fragment.sh` rejects fragment due to schema-v3 mismatch | LOW | LOW | Mirror an existing `kind: "log"` `from: "planner"` entry verbatim; helper validates and emits a clear error |
| r2b regression: r2b adds a new fixtures module that re-violates the doc-comment audit | LOW | LOW | r2b's Task 0 re-counts; defect class is monotonic |

## 19. Notes

### Plan-author-time DoD dry-run results

Per `feedback_plan_dod_dry_run_at_write.md`, the advisor verified:
- §15.2 lint script self-test: cannot dry-run before T1 ships the
  script; verified by mirror to `scripts/brehon/resolve-dq-canonical.sh`.
- Task 0 cargo probes: green at gate-1 smoke against `phase-v1-quality-r2`
  HEAD (cargo check 1m10s exit 0, clippy 1m46s exit 0; both warm cache
  from Lane A's smoke). Re-run by Task 0 on the Junior worker.

### Cross-lane safety

Lane A (v1-redaction-r1, Task 0 #489 just completed all 13 probes; ready
for Task 1) is on `phase-v1-redaction-r1` touching
`crates/db_schema/src/source/governance/redaction.rs` only. Zero file
overlap with this lane (scripts + DQ JSON). Safe to run in parallel.

### Branch name semantics

The branch is `phase-v1-quality-r2` but the plan + scope is now `r2a`.
This semantic drift is documented at the top of this plan. r2b will get
its own branch `phase-v1-quality-r2b` cut from trunk after r2a merges.

## 20. Confidence score

- **Plan correctness:** 9/10 — narrow scope, all-metadata work, mature
  helper scripts as MIRROR refs, atomic-protocol discipline well-defined.
- **Cargo budget:** N/A — Task 0 only.
- **Test coverage:** 8/10 — synthetic-negative + positive self-test on
  the lint; deferral entry shape verified by Python assertion.
