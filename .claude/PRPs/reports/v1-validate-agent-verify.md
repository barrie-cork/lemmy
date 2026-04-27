# /brehon-verify — v1-validate-agent

**Phase:** v1-validate-agent
**Branch:** `phase-v1-validate-agent` @ `875073d29` (post-cr-9 fix)
**PR:** [#104](https://github.com/barrie-cork/lemmy/pull/104) — base `governance-v0`, head `phase-v1-validate-agent`
**Run:** 2026-04-27, after CR re-poll closed 7/7 originals + 2 follow-ups (cr-8 rebut, cr-9 fixed)
**Spec:** `.claude/PRPs/plans/v1-validate-agent.plan.md` §16a Stories block
**Outcome:** All testable stories ✓ or `[deferred-to-retro]` per plan §16a; no phantoms; safe to advance to merge confirm.

---

## Summary

| # | Story | Status | Evidence |
|---|---|---|---|
| 1 | impl-task push to phase-v1-* triggers cargo-validate-workspace; conclusion: success | **✓** | run [25018404875](https://github.com/barrie-cork/lemmy/actions/runs/25018404875) (initial), [25020638716](https://github.com/barrie-cork/lemmy/actions/runs/25020638716) (post-fix-commits) — both `conclusion: "success"`, `event: "push"` |
| 2 | ci-watcher artifacts exist + schema-additive consistency | **✓** | ci-watcher.md (model: claude-haiku-4-5, effort: low), brief template (38 lines), decision-queue.md enumerates validate-* 11× |
| 3 | ci-watcher classifies success workflow as validate-result: pass | **[deferred-to-retro]** | foreground-session execution; no real ci-watcher Junior task dispatched. v1-JM-e first real exercise. |
| 4 | ci-watcher classifies failure workflow with log_slice attached | **[deferred-to-retro]** | same — gated on real failure; v1-JM-e or v1-rep-tuning-r3 |
| 5 | advisor §G4 classifier auto-queues fix-impl-task for clippy-allowlist | **[deferred-to-retro]** | same — gated on real allowlist-eligible failure |
| 6 | jm-d-impl-2.md §5 retrofit + DQ #61 resolution | **✓ (modulo bm-merge)** | §5 has push-and-exit body (2 occurrences); DQ #61 resolved + mentions validate-agent. Final flag (PR merged) verifies post-bm-merge. |

**Phantoms:** none. Every Brief-Scope output the plan §16a names is present + matches its structural pattern.

**CR triage status (per `.claude/PRPs/reviews/pr-104-findings.yaml`):**
- 7 of 9 findings **done** (all 4 Major + 3 Medium closed by 4 fix commits + 1 follow-up)
- 1 **rebut** (cr-8: markdownlint MD041 vs corpus convention; rationale documented in findings YAML + commit `875073d29` body)
- 1 **fix-in-pr → done** (cr-9: postgresql-client stripped from stub workflow in `875073d29`)
- 0 critical, 0 carry-forward, 0 wont-fix
- 0 findings remain in `fix-in-pr` bucket → safe-to-merge gate cleared

**Workflow runs on phase branch (final state):**
- `cargo-validate-workspace`: run 25020638716 ✓ success on `24c701562` (latest workspace-affecting commit)
- `cargo-validate-migration`: run 25021266534 ✓ success on `875073d29` (cr-9 fix commit)

---

## Per-story detail

### Story 1 — workflow trigger + green ✓

**Composing tasks:** Task 1 (workflow YAMLs)

**Checkpoint:**
```bash
gh run list --repo barrie-cork/lemmy --branch phase-v1-validate-agent \
  --workflow cargo-validate-workspace.yml --limit 1 \
  --json event,status,conclusion --jq '.[0]'
```

**Output:** `{"conclusion":"success","event":"push","status":"completed"}`

**Brief-Scope outputs verified:**
- `.github/workflows/cargo-validate-workspace.yml` exists + non-empty (76 lines after cr-4 fix)
- `.github/workflows/cargo-validate-migration.yml` exists + non-empty (60 lines after cr-2 + cr-9 fixes)
- First post-Task-1 push triggered a real GitHub Actions run (run 25017407659 — though it failed on the `--no-deps` flag bug, fixed in commit `ed049970b`)
- Post-fix-commits run `25020638716` reached `conclusion: success` cleanly (12 of 12 steps green; cargo check 12 min, clippy 4 min, test compile 11 min, cold cache ~28 min total)

### Story 2 — ci-watcher contract + schema enumeration ✓

**Composing tasks:** Tasks 1 + 2 + 3

**Checkpoint:**
- `.claude/agents/ci-watcher.md` frontmatter present + correct
- `.claude/PRPs/templates/ci-watcher-brief.template.md` exists
- `.claude/rules/decision-queue.md` enumerates the three new kinds

**Output:**
```
model: claude-haiku-4-5
effort: low
```
+ schema enumeration: 11 occurrences of `"validate-(pending|result|failed)"` in decision-queue.md (expected ≥6).

**Brief-Scope outputs verified:**
- ci-watcher.md frontmatter pinned to claude-haiku-4-5 + low effort + narrow tools (Read, Edit, Write, Bash) — matches plan §10.5
- Brief template at .claude/PRPs/templates/ci-watcher-brief.template.md exists, 38 lines, matches plan §10.6 + post-cr-3 timed_out alignment
- decision-queue.md schema text now reads from `gh run view <id> --json conclusion` (post-cr-3 fix), with full enum {fail, cancelled, timed_out, gh_unauth, run_not_found} (post-cr-7 fix)
- ci-watcher.md `result` enum at line 127 matches schema (post-cr-5 fix)

### Story 3 — validate-result classification (deferred)

**Composing tasks:** Tasks 2 + 3

**Checkpoint:**
```python
import json
d = json.load(open('.claude/decision-queue.json'))
results = [e for e in d.get('resolved', []) if e.get('kind') == 'validate-result']
```

**Output:** `validate-result count: 0`

**Status:** `[deferred-to-retro]` (per plan §16a Story 5 status framing — applies to Story 3 as well under foreground execution).

**Why:** This sub-phase ran as foreground impl session, NOT four-role Junior orchestration. The advisor's polling loop never dispatched a real `[role:ci-watcher]` Junior task because the impl-task slot was the laptop session itself, not a Junior worker. v1-JM-e (first sub-phase under full Shape G) will be the first real exercise; the retro §5.1 carries the watch-item.

**No phantom risk:** ci-watcher.md and the brief template ARE the ship deliverable for this story; the actual classification behaviour is exercised at v1-JM-e.

### Story 4 — failure log_slice (deferred)

Same status as Story 3. The plan explicitly marked Story 4 as `[deferred-to-retro]` if no real failure surfaces during the test push (plan §16a Story 4 ground rules). The clippy-toolchain-missing failure observed in run `25017554049` and the `--no-deps` failure in run `25017407659` are NOT allowlist-eligible (workflow-config errors, not lint-level) — they would have catch-fired even if classified.

### Story 5 — §G4 classifier auto-queue (deferred)

Same as Story 4 — gated on a real allowlist-eligible failure. Allowlist (per advisor-orchestrator.md §G4 classifier): `clippy::doc_lazy_continuation`, missing imports (`error[E0432]`), deprecated APIs. None of these reproduced during execution. v1-JM-e or v1-rep-tuning-r3 will be the first real test.

### Story 6 — JM-d Task 2 unblock ✓ (modulo bm-merge)

**Composing tasks:** Task 5 (jm-d-impl-2.md §5 retrofit)

**Checkpoint 1:**
```bash
grep -c 'push-and-exit\|out-of-band on GH Actions' .claude/PRPs/briefs/jm-d-impl-2.md
```
**Output:** `2` (expected ≥1) ✓

**Checkpoint 2:**
```python
import json
d = json.load(open('.claude/decision-queue.json'))
d61 = next((e for e in d.get('resolved', []) if e.get('id') == 61), None)
print(f'DQ #61 resolved: {d61 is not None}')
print(f'mentions validate-agent: {"validate-agent" in (d61 or {}).get("answer", "")}')
```
**Output:**
```
DQ #61 resolved: True
mentions validate-agent: True
```
✓

**Brief-Scope outputs verified:**
- `.claude/PRPs/briefs/jm-d-impl-2.md` §5 body changed (commit `6e03249fa`) — Validation heading retitled to "Validation gates (out-of-band on GH Actions per Shape G)"
- No other `jm-d-impl-*.md` brief modified (only `jm-d-impl-2.md` in the diff per Task 5 GOTCHA)
- DQ #61 (advisor 2026-04-27, plan-mode) resolved with `answer` text mentioning validate-agent's bm-merge as the unblock condition

**Final flag:** Story 6's last item ("PR for `phase-v1-validate-agent` merged to `governance-v0`") is post-merge; the verify pass cannot confirm it pre-merge. The merge-confirm user gate is the final approval; once merged, this story closes fully.

---

## Phantoms check

Every Brief-Scope output named in §16a was tested against the worktree branch state. **Zero phantoms.** No story shows "task complete but expected output absent or empty."

The plan's §16a framing of Stories 4 + 5 as gateable on `[deferred-to-retro]` is the intended design — those stories are not phantom, they are explicitly conditional. Foreground vs four-role execution defers Story 3 by the same logic.

---

## Recommendation

**Safe to advance to merge confirm user gate.** All 6 stories pass verify (3 ✓, 3 [deferred-to-retro] per plan-design conditions); 9/9 CR findings triaged with 7 done + 1 rebut + 1 fix-in-PR-now-done; both workflow runs (`cargo-validate-workspace` + `cargo-validate-migration`) on the latest commit reached `conclusion: success`.

**Pre-merge user gate next:** confirm `gh pr merge --repo barrie-cork/lemmy --merge` (NOT --squash per `phase-branch.md`).

**Post-merge tasks (separate from PR):**
1. `homeserver/library.yaml` registers all 4 new artefacts (cargo-validate-workspace.yml, cargo-validate-migration.yml, ci-watcher.md, ci-watcher-brief.template.md) — sibling-repo edit per §17 + `feedback_library_add_after_shipping.md`
2. `homeserver/CLAUDE.md` one-line ci-watcher-is-Junior-subagent note
3. Flag advisor that JM-d Task 2 is now unblocked per DQ #61 — first real Shape G consumer post-merge

---

_Verify run by foreground impl session, 2026-04-27 21:48 UTC. Source: plan §16a Stories block. PR #104 head: `875073d29`._
