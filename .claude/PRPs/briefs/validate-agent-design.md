---
role: design-brief
target: planning-task (next session)
phase: cross-phase architectural — applies to v1-JM-e onwards
created: 2026-04-27
related_pmd: 109, 110
status: design-only — no code in this brief
---

# Design brief — Out-of-Junior cargo validation with model-escalation path (Shape C + A)

## 1. Purpose

Decouple long-running validation work (`cargo check`, `cargo clippy`, `cargo nextest`, migration round-trips) from the impl-task subagent's wall-clock occupancy of the Junior worker slot. Deliver: a non-Junior cargo runner that handles 95% of validations binary-pass-fail, with a model-escalation path (Sonnet 4.6 `validate` subagent) that takes over only when the cargo failure isn't pattern-matched to a known shape.

This is **architecture work, not feature work.** It changes how Brehon's validation gates execute, not what they validate.

## 2. Why this exists (problem statement)

**Observed cost on v1-JM-d task #13 (2026-04-27):**

- Total impl-task runtime: ~85+ minutes (still running at write time).
- Of that, ~33+ minutes was the model passively waiting on `cargo check --workspace --features full` (PID 210539) to finish.
- During those ~33+ minutes:
  - Junior worker slot (concurrency=1) blocked — no other Brehon task could run.
  - Junior cgroup memory cap (10 GB) at 8.6 GB — close to spillover.
  - Worktree held — sibling sessions warned-off (per `feedback_commit_aggressively_in_shared_repos`).
  - Opus tokens consumed: zero (Bash tool sleeping). But minute-of-day occupancy: 100%.

**Across a full sub-phase:** v1-JM-d has 6 impl-tasks. If each spends ~30 minutes on cargo, that's ~3 hours of impl-slot occupancy doing nothing the model needs to be there for. Across the remaining JM phases (JM-e through JM-h ≈ 4 sub-phases × 6 tasks each), conservatively 12 hours of recoverable wall-clock per phase-pair.

**Brehon goal alignment:** this directly serves goal #4 of `feedback_brehon_autonomy_goals` — model-efficient. The current shape pays for model wall-clock during cargo runs that any shell would handle.

## 3. Solution shape — "C with A escalation"

### Layer C (default path — 95% of cases)

A non-Junior cron-driven validation runner. When an impl-task subagent commits its work, it appends a `validate-pending` entry to `.claude/decision-queue.json` and exits. A systemd timer (or recurring cron) picks up `validate-pending` entries, runs the cargo commands in a bounded screen session **outside any Junior daemon**, and writes the result back to the DQ as `validate-result`. The advisor's polling loop picks up the result on the next tick.

The impl-task slot frees immediately on commit — typically 20–40 minutes earlier than today.

### Layer A (escalation path — 5% of cases)

When the validation runner produces a non-clean result that doesn't match a known shape (clean pass, known-clippy-fix-pattern, known-test-compile-pattern), it files a `validate-failed` DQ entry with the cargo output attached. The advisor reads it and queues a new Junior task with `[role:validate]` (a fourth subagent, Sonnet 4.6, mechanical) to triage.

The validate subagent reads the cargo output, decides one of:
- **Trivial fix:** apply it, commit, re-trigger validation. (Most clippy lints, missing imports, mechanical errors.)
- **Real surprise:** file a DQ pending entry surfacing to advisor with a one-paragraph summary. Advisor decides whether to relay to user or queue an impl-task fix.

This is the model-escalation lever. The model is invoked only when its judgment adds value.

## 4. Files this design will touch (when implemented)

This brief is design-only. The implementation in a future session will need to author plans for the following changes:

### 4.1 New artifacts

- `scripts/brehon/validate-runner.sh` — the cron-driven validator. Reads `.claude/decision-queue.json`, takes the next `validate-pending` entry, runs the cargo commands in a screen session, writes the result back. Idempotent. Lives at the homeserver level (cross-cutting infra), not at brehon-fork (single-repo-coupled would limit reuse if other Rust repos join Brehon).

- `systemd/validate-runner.timer` + `.service` — drop-in for the homeserver. Runs validate-runner.sh every 60s. Stoppable for forbidden windows / Brehon stand-down.

- `.claude/agents/validate.md` — the new subagent definition. Sonnet 4.6 (matches impl-task), `effort: medium`, narrow tool list (Read/Edit/Write/Bash/Glob/Grep — no LSP, no Agent), description matches `[role:validate]` dispatch.

- `.claude/commands/validate-triage.md` — the slash command form, used inside the validate subagent for known-pattern triage logic.

- `.claude/lessons/feedback_validate_subagent_known_patterns.md` — the catalogue of "known clippy fixes" and "known test-compile shapes" that Layer C uses to decide pass-fail-binary vs escalate-to-A.

- `.claude/PRPs/templates/validate-brief.template.md` — sibling to existing impl/planning brief templates.

### 4.2 Existing artifacts to extend

- `.claude/rules/decision-queue.md` — new `kind: "validate"` schema (extends current blocker/log/clarify enumeration). Validate entries have specific shape: `commit_sha`, `dod_commands` (array), optionally `validate_result` once filled. Per `feedback_update_rule_doc_with_schema_additive` — schema add must update the rule doc in the same commit.

- `.claude/rules/advisor-orchestrator.md` — add a "Validation queue" stage to the polling loop. The "stage-shape orchestration" §13 sub-section gains an item: "after impl-task complete → validate-pending DQ entry filed → Layer C runs out-of-band → on validate-result, advance." Refusal cases for catch-fire: validate-result missing after 90min (cron stalled), validate-failed escalation arriving without a Junior `[role:validate]` task on the queue (Layer A wiring broken).

- `.claude/agents/impl-task.md` — modify task-7 closing step. Currently impl-task runs §15 DoD itself. Under the new shape, impl-task commits, files a `validate-pending` DQ entry naming the commit + the §5 commands, and exits. The §15 DoD self-run is removed (or kept as an opt-in "fast-path" for trivial tasks that the planner has marked `validate: inline`).

- `.claude/agents/planning.md` — plan §13 task entries gain a new optional field per task: `validate_path: out-of-band | inline`. Default out-of-band. Planner must pick `inline` for tasks where the model genuinely needs to read cargo output as part of its work (rare — usually only for tasks that conditionally branch on test results).

- `homeserver/CLAUDE.md` — update the Junior daemon description with the new validate-runner timer + the §"Junior" sub-section's "Active daemons" list (validate-runner is technically not a Junior daemon; document in its own section).

### 4.3 NOT changing

- Junior's executor (`/opt/junior-src/src/core/executor.ts`) — the new shape works at the Brehon brief/agent layer, not Junior internals. Junior continues to dispatch `[role:*]` tasks identically.
- The four-role model frontmatter contract — adding a fifth role (validate) doesn't break the four; it adds a pattern-matched-pre-escalation layer between roles.
- The cgroup memory cap on `junior@brehon-fork` — it stays at 10 GB. cargo running outside the daemon doesn't hit it.
- The forbidden-execution-windows table in advisor-orchestrator.md — validate-runner respects the same windows (no cargo during 02:55-04:15 UTC etc).

## 5. Decision-queue schema additions

The `validate-pending` entry shape:

```json
{
  "id": <next>,
  "kind": "validate",
  "from": "impl-task",
  "status": "pending",
  "phase": "v1-JM-e",
  "commit_sha": "<7-char>",
  "branch": "phase-v1-JM-e",
  "dod_commands": [
    "bash scripts/brehon/cargo-check.sh --workspace --features full",
    "bash scripts/brehon/cargo-clippy.sh --workspace --features full --no-deps -- -D warnings",
    "bash scripts/brehon/cargo-test.sh --test e2e --no-run -p lemmy_server"
  ],
  "expected_exit_codes": [0, 0, 0],
  "context": "post-impl-task <N> validation per plan §15"
}
```

The `validate-result` entry (filled by Layer C):

```json
{
  "id": <pending-id>,
  "kind": "validate",
  "status": "resolved",
  "result": "pass" | "fail-known" | "fail-unknown",
  "exit_codes": [0, 0, 0],
  "log_path": ".claude/PRPs/debug/validate-<phase>-<sha>.log",
  "answered_by": "validate-runner",
  "answer": "<one-paragraph summary>"
}
```

Layer A escalation: `result: "fail-unknown"` triggers the advisor to queue `[role:validate]` Junior task. The validate subagent reads the log, attempts a fix or escalates per its own brief.

## 6. Layer C "known pattern" catalogue (initial — to seed at implementation time)

The validate-runner needs a small DSL or jq-pattern set to decide pass vs known-fail vs unknown-fail. Initial seed list (extracted from existing `feedback_*.md` lessons):

- **Pass:** all exit codes match `expected_exit_codes` exactly. Result = "pass".
- **Known-fail patterns** (output contains, in order of priority — first match wins):
  - `"error: could not compile"` followed by a `^error\[` block — known compile error. Result = "fail-known", suggested_role = "impl-task" (re-fix).
  - `"warning: unused"` only, no `error:` — known clippy lint, mechanical. Result = "fail-known", suggested_role = "impl-task" with brief "apply suggested clippy fix".
  - `"check_for_backend(diesel::pg::Pg)"` — known schema-regen mismatch per `feedback_pq_sys_stale_cache` neighbour. Result = "fail-known", suggested_role = "impl-task" with brief "re-run schema regen via diesel print-schema".
  - `Killed` or `signal: killed` — OOM cascade per `project_elitedesk_hung_2026_04_27`. Result = "fail-known", surface to advisor as catch-fire (don't auto-retry — file a DQ pending entry, advisor decides).
- **Unknown-fail:** anything that didn't match the above. Result = "fail-unknown", escalate to Layer A.

The catalogue starts small (5 patterns) and grows by retro harvesting. Each new known pattern requires:
1. A `feedback_*.md` lesson explaining the pattern + fix
2. An entry in `feedback_validate_subagent_known_patterns.md`
3. Cited in the validate-runner's pattern check

This is the same canonical-first + dogfood discipline already in place for slash commands.

## 7. Wall-clock budget and forbidden-window interaction

- **Layer C runtime per validation:** ~3–35 minutes depending on cache state and command set. The 35-minute upper bound is `cargo check + cargo clippy + cargo test --no-run` cold cache. Hot cache: ~3–8 minutes.
- **Layer A escalation runtime:** ~5–15 minutes (Sonnet 4.6 reading log + applying fix or filing DQ).
- **Concurrency:** Layer C is concurrency=1 (cargo can't run in parallel without doubling memory). The cron timer must serialise — locking via flock on `/var/run/brehon-validate.lock`.
- **Forbidden windows:** validate-runner.sh checks the same forbidden-window table as advisor-orchestrator. If a `validate-pending` entry arrives during a forbidden window, the runner defers (writes "deferred until <window-end>" to a transient file the advisor's polling loop reads). After window closes, picks up where it left off.

## 8. Failure modes to design against

- **Validate-runner stalled:** advisor's polling loop notices `validate-pending` entry > 90min old, no `validate-result`. Catch-fire: surface to user, suggest manual `systemctl status validate-runner.timer`.
- **Layer C false-pass:** known-pattern matched something that wasn't actually a known pattern. Mitigated by the catalogue requiring a citation for each entry. Caught by the JM-e first-trial retro.
- **Layer C false-escalate:** clean cargo pass mistakenly matched a fail pattern. Mitigated by exit-code check happening *before* output-pattern matching. If exit_codes match expected, never escalate.
- **DQ schema drift:** validate entries get into the queue but advisor's rule doc still enumerates only blocker/log/clarify. Mitigated by `feedback_update_rule_doc_with_schema_additive` discipline — schema update + rule doc update in same commit.
- **Concurrency between Layer C and a running impl-task on the same worktree:** the `validate-pending` entry names the commit SHA, not the worktree path. Layer C runs cargo against the *committed code* (checkout into a tmp dir, or against the worktree if no other task is using it). The implementing session will need to decide: cheaper-but-fragile (use the existing worktree) vs robust-but-slower (clone to /tmp/validate-<sha>/).

## 9. What the implementing session needs to do

A future session takes this brief and:

1. **Authors a planning brief** at `.claude/PRPs/briefs/validate-agent-planning-1.md` that translates §3–§7 into a sub-phase plan. The plan should fit the v1-JM template (§1..§20 sections) and target `governance-v0` directly (this is meta-work, not impl-work — per `feedback_pr_per_phase`, meta-work commits direct to governance-v0, not phase-branch + PR).

2. **Runs `/brehon-clarify` on this design brief** before queueing the planning task per the clarify gate. Likely clarify questions:
   - Does Layer A's `[role:validate]` subagent get its own systemd memory cap? (Recommend: same `junior@brehon-fork` cgroup since it's a Junior subagent.)
   - Is the validate-runner a homeserver-level systemd unit or a brehon-fork-level cron? (Recommend: homeserver-level — cross-cutting infra. Document under `homeserver/systemd/`.)
   - When a `validate-failed` requires fix, does the impl-task that authors the fix run as a *new* `[role:impl-task]` brief, or does the validate subagent write the fix itself? (Recommend: validate subagent writes trivial fixes; escalates non-trivial to impl-task. Threshold: ≤3 file edits = validate subagent; >3 = impl-task.)
   - Layer C concurrency: tmp-dir clone vs worktree-reuse? (Recommend: tmp-dir clone — robustness wins; cargo cache penalty is one-time per phase since /tmp/validate-* dirs persist within a sub-phase.)

3. **Surface to user** before queueing planning. This is sub-phase-scope architecture and falls under "judgment-heavy DQ" gate per advisor-orchestrator.md.

4. **Plan target:** v1-JM-e or later. Do not retrofit JM-d — that's already mid-flight.

5. **Pre-existing impl-tasks** continue to self-validate inline until the planning subagent rewrites `.claude/agents/impl-task.md` task-7 closing step. The new shape applies to plans authored after the validate-agent ships.

## 10. References

- **Originating evidence:** PMD #109 (impl-task running Opus), PMD #110 (telemetry blind-spot during cargo). The 33+ minute cargo wait that motivated this design is captured there.
- **Goal alignment:** `feedback_brehon_autonomy_goals` goal #4 (model-efficient).
- **Schema discipline:** `feedback_update_rule_doc_with_schema_additive`, `feedback_one_system_memory_in_repo`.
- **Subagent contract reference:** existing `.claude/agents/{planning,impl-task,bm-task}.md` are the canonical examples to mirror when authoring `validate.md`.
- **DQ schema reference:** `.claude/rules/decision-queue.md` — the kind:clarify precedent for adding a new kind value.
- **Brief shape reference:** `.claude/PRPs/briefs/jm-d-impl-2.md` is the most recent fully-formed impl brief; mirror its frontmatter shape and §1–§7 section ordering when authoring the planning brief that comes from this design.

## 11. Out of scope (explicit)

- Changes to BM verbs (bm-cut, bm-pr, bm-merge, etc) — validate is a sibling stage, not a BM verb.
- Cross-provider model trials (MiniMax, Gemini for cheaper validate) — Brehon stays Anthropic-only per `feedback_brehon_anthropic_only`. Validate subagent is Sonnet 4.6.
- Replacing the §15 DoD definition itself — we're changing where DoD runs (out-of-band vs in-band), not what it tests.
- Auto-retry of failed validation — `max_retries: 0` discipline in junior config.yaml stays. A validate-failed entry surfaces; advisor decides next steps; no auto-retry.
- The investigation of PMD #109 (impl-task on Opus) — that's a separate diagnostic. Independent from this design but informs the urgency: if impl-task is paying Opus rates for cargo wall-clock, the savings from this design double.

## 12. Acceptance criteria

The implementing session ships this when:

1. `validate-runner.sh` ships at `homeserver/scripts/brehon/` and is registered in `homeserver/library.yaml`.
2. The systemd timer + service are at `homeserver/systemd/` with a deployment note in CLAUDE.md.
3. `.claude/agents/validate.md` exists in brehon-fork, mirrors the four-role frontmatter shape, and is dispatched by `[role:validate]`.
4. `.claude/rules/decision-queue.md` enumerates `kind: "validate"` with the schema from §5 of this brief.
5. `.claude/rules/advisor-orchestrator.md` "Stage-shape orchestration" sub-section adds the validate stage between impl-task complete and bm-cut.
6. The first sub-phase (v1-JM-e) runs end-to-end with Layer C handling at least 4 of its 6 impl-tasks' validation, and Layer A engaging at least once for a real escalation.
7. JM-e retro section §5 reports per-task wall-clock savings (target: ≥20 minutes per impl-task on cargo-heavy tasks).
8. Library registration of new scripts/agents/lessons happens in the same commit that ships them — per the now-3rd-recurrence library-registration-lag pattern in PATTERNS.md.

## 13. Estimated effort (rough)

- Planning brief author + clarify pass: 0.5 days
- Plan authoring (one Opus planning task): 0.25 days (one Junior dispatch)
- Layer C implementation (validate-runner.sh + systemd unit + DQ schema): 1 day
- Layer A implementation (validate.md subagent + commands + lessons): 0.5 days
- Rule + agent doc updates (decision-queue.md, advisor-orchestrator.md, impl-task.md): 0.25 days
- Dogfood trial on v1-JM-e: opportunistic (no extra effort if JM-e ships under the new shape)
- Total: ~2.5 days of implementing-session time, spread across 1–2 sub-phases of opportunity.

This is too large to dogfood as a single Junior impl-task. Likely shape: 4–6 impl-tasks under a v1-validate-agent sub-phase, with the planner authoring the breakdown.