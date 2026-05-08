# ci-watcher brief — workflow run 25531818852

**Workflow run id:** 25531818852
**Branch:** phase-v1-SL-c-1 (DQ #163 was raised by impl-task on its worker branch and the daemon-merge already finalized to `phase-v1-SL-c-1`; canonical entry now lives at `phase-v1-SL-c-1` tip on origin)
**Phase task:** 2 (sl-c-1-impl-2 — sponsor_liability_grace scheduler block + atomic concurrency guard)
**Paired DQ entry:** #163 (kind: "validate-pending", in pending[] on `phase-v1-SL-c-1`)

## Action

Per option 2 (PMD #156, locked 2026-04-28): **MUTATE the existing `validate-pending` DQ entry in place by matching `workflow_run_id == 25531818852`. Do NOT write a new entry.** The entry's `kind` STAYS `"validate-pending"`; `from` STAYS `"impl"`; only `result` + `log_slice` + `failed_jobs` + `answer` + `answered_by` + `resolved_at` are populated. Entry moves `pending[]` → `resolved[]` ONLY on `result: "pass"`; failures stay in `pending[]` for advisor §G4 triage.

Poll `gh run watch 25531818852 --exit-status --repo barrie-cork/lemmy` (single long-poll, wrapped in `timeout 3600` for the 60-min cap). On return:

1. **Locate the paired entry:** find the entry in `.claude/decision-queue.json` `pending[]` whose `workflow_run_id` matches `25531818852`. The Junior daemon will worktree-cut from `phase-v1-SL-c-1` for this ci-watcher dispatch; DQ #163 is on that base branch already. Read the file directly. If absent, file a `kind: "blocker"` DQ entry from `from: "ci-watcher"` (the advisor's dispatch contract was violated) and exit non-zero. **Do NOT write a new `validate-pending` entry.**

2. **Pre-flight:** `gh auth status`. If unauthorised, mutate the paired entry with `result: "gh_unauth"`, `answer: "<one-liner>"`, `answered_by: "ci-watcher"`, `resolved_at`. Stays in `pending[]`. Commit + push, exit 0.

3. **Run-existence check:** `gh run view 25531818852 --json status`. If the run is not found, mutate the paired entry with `result: "run_not_found"`, `answer`, `answered_by`, `resolved_at`. Stays in `pending[]`. Commit + push, exit 0.

4. **Long-poll with shell-side timeout:**
   ```bash
   timeout 3600 gh run watch 25531818852 --exit-status --repo barrie-cork/lemmy > /tmp/ci-watch.log 2>&1
   status=$?
   ```
   If `status == 124` → mutate the paired entry with `result: "timed_out"`, populate `answer` + `answered_by: "ci-watcher"` + `resolved_at`. Stays in `pending[]`. Commit + push, exit 0.

5. **Disambiguate via conclusion (mandatory):**
   ```bash
   conclusion=$(gh run view 25531818852 --repo barrie-cork/lemmy --json conclusion --jq '.conclusion')
   ```

6. **Classify on `conclusion` and mutate the paired entry** per the template `.claude/PRPs/templates/ci-watcher-brief.template.md`. On `success`: set `result: "pass"`, populate `answer` ("All workspace-check jobs passed for sl-c-1 task 2 (scheduler block)."), `answered_by: "ci-watcher"`, `resolved_at`. **Move entry from `pending[]` to `resolved[]`.** On `failure`: set `result: "fail"`, populate `log_slice` (last ~200 lines from `/tmp/ci-watch.log`), `failed_jobs` (parse from gh API), `answer` (one-line failure summary), `answered_by: "ci-watcher"`, `resolved_at`. **Entry STAYS in `pending[]` for advisor §G4 triage.**

7. **Verify mutation post-write** per the template above. Commit subject: `chore(decision-queue): ci-watcher mutated DQ #163 — <pass|fail> workspace check sl-c-1-impl-2`. Push HEAD.

## Hard refusals (CRITICAL — read fully)

This brief explicitly enumerates the Hard refusals because the ci-watcher #145 dispatch on 2026-05-08 violated #2 + #7 — wrote a NEW DQ entry (id collision) instead of mutating the existing one. Re-read these refusals before any DQ write:

1. **#2 — NEVER reuse an existing id.** Compute `max(all_ids, default=0) + 1` from BOTH `pending` AND `resolved`. The DQ #50 collision incident (`e9fa1e01a`) and the DQ #145 incident (2026-05-08) were the lessons. With archives in play, ignoring archives when computing next_id is the new way to reproduce that bug.

2. **#7 — NEVER write a new `kind: "validate-pending" | "validate-result" | "validate-failed"` entry from any session.** Those kinds are deprecated for new writes (PMD #156, locked 2026-04-28). ci-watcher's job is to **MUTATE the existing `validate-pending` entry** by matching `workflow_run_id`. Mutate-in-place: populate `result` + `log_slice` + `failed_jobs` + `answer` + `answered_by: "ci-watcher"` + `resolved_at`. Move `pending[]` → `resolved[]` ONLY on `result: "pass"`. The entry's `kind` stays `"validate-pending"` — kind records what was raised, not current state. The entry's `from` stays `"impl"` — never rewrite to `"ci-watcher"`.

3. **NEVER write `answered_by: "advisor"` or `"user"`.** Self-resolve only with `answered_by: "ci-watcher"`.

4. **NEVER invoke cargo, never edit code in `crates/` / `migrations/` / `tests/` / `docs/` / `.github/workflows/`, never apply clippy auto-fixes** (those are advisor §G4 classifier work, not ci-watcher's lane).

5. **NEVER skip the mid-task commit + push** after mutation. Without the push, the entry is trapped on the worktree until daemon finalize. The advisor cannot see it.

See `.claude/agents/ci-watcher.md` "Hard refusals" sub-section and `.claude/PRPs/templates/ci-watcher-brief.template.md` for the full canonical contract.

## What "mutate-in-place" looks like in code

```python
import json
with open('.claude/decision-queue.json', 'r', encoding='utf-8') as f:
    d = json.load(f)

# Locate paired entry by workflow_run_id
target_wfid = 25531818852
paired = None
for entry in d['pending']:
    if entry.get('workflow_run_id') == target_wfid:
        paired = entry
        break

if paired is None:
    # File kind: blocker DQ (advisor contract violation), exit non-zero
    raise SystemExit('paired entry not found')

# Mutate in place — populate result fields ONLY
paired['result'] = 'pass'  # or 'fail' / 'timed_out' / etc
paired['log_slice'] = None  # or last 200 lines on fail
paired['failed_jobs'] = None  # or array on fail
paired['answer'] = 'All workspace-check jobs passed for sl-c-1 task 2 (scheduler block).'
paired['answered_by'] = 'ci-watcher'
paired['resolved_at'] = '2026-05-08T<HH:MM:SS>Z'

# On pass only: move pending[] → resolved[]
if paired['result'] == 'pass':
    d['pending'].remove(paired)
    d['resolved'].append(paired)
# On fail / timeout / etc: leave in pending[] — advisor handles §G4 triage

# Write back with ensure_ascii=False (UTF-8 preservation per feedback_json_dump_ensure_ascii_false)
with open('.claude/decision-queue.json', 'w', encoding='utf-8') as f:
    json.dump(d, f, indent=2, ensure_ascii=False)
    f.write('\n')
```

**Do NOT** create a new dict and append it to `pending[]` or `resolved[]`. **Mutate the existing entry that you found by `workflow_run_id` match.**

## What success looks like

After the ci-watcher run completes:
- `.claude/decision-queue.json` contains DQ #163 with `result`, `log_slice`, `failed_jobs`, `answer`, `answered_by: "ci-watcher"`, `resolved_at` populated.
- On `pass`: DQ #163 has moved from `pending[]` to `resolved[]`.
- On `fail`: DQ #163 stays in `pending[]` for advisor §G4 triage.
- A single commit on the ci-watcher worker branch with subject `chore(decision-queue): ci-watcher mutated DQ #163 — <pass|fail> workspace check sl-c-1-impl-2`, pushed to origin.
- **NO new DQ entry has been created.** A `git diff` against the prior `phase-v1-SL-c-1` tip shows only the mutation of #163, not an append of #164 or any other id.
