---
phase: role-customization-T5
plan: C:/Users/barri/.claude/plans/i-am-interesting-assessing-ethereal-shore.md   # §Verification (T5 = end-to-end verify)
phase_branch: (none — role-customization is direct-on-governance-v0 meta-work per phase-branch.md; no PR flow)
worktree: C:/Users/barri/Developer/brehon-fork (canonical, governance-v0)
authored: 2026-06-22
authored_by: advisor (canonical brehon-fork / governance-v0 session)
purpose: Bootstrap the role-customization-T5 session — BUT T5 IS HELD. Read the RESUME block; the first action is a GATE CHECK, not T5 work.
---

# ⏩ RESUME — read this block first

**You are the advisor for Brehon role-customization-T5 — but T5 is HELD.** Do NOT
start T5 work on resume. T5 (end-to-end verify of the role-customization
substrate) is gated on a traffic threshold that was not met at handoff. Your
first action is to **check whether the gate has lifted**, not to author a T5
brief.

T1a/T1b/T2/T3a–c/T4a/**T4b are all SHIPPED.** role-customization is
`.claude/`/`scripts/`/`.claude/hooks/`/skill meta-work → **direct-on-governance-v0,
NO PR flow** (per `phase-branch.md`). No bm-cut, no phase branch, no Junior
dispatch for the substrate work itself.

## The hold gate (the ONLY thing that matters on resume)

**T5 stays HELD until ≥5 non-sentinel role-signal dispatches exist PER role.**
At the last measurement (2026-05-25 `check-role-health` first run) the corpus was
thin: `bm-task×7` (4 sentinel → 3 real), `planning×3`, `impl-task×1`,
`ci-watcher×0`. Far below the ≥5-per-role bar — `ci-watcher` and `impl-task`
especially. T5 verifies the substrate against *real* per-role traffic; running it
on a near-empty corpus measures nothing.

**2026-06-22 update:** the T4b ship surfaced **13 undrained role-signal rows from
the last ~24h** (via the drain dry-run). That's encouraging volume growth, but
the rows are unfiltered — sentinel-vs-real and per-role counts are unknown until
drained + aggregated. The gate is NOT presumed lifted.

## Session-start ritual (do these first)

1. `pwd && git -C C:/Users/barri/Developer/brehon-fork branch --show-current && git -C C:/Users/barri/Developer/brehon-fork worktree list` — confirm CWD = canonical `brehon-fork` on `governance-v0`.
2. `git -C C:/Users/barri/Developer/brehon-fork fetch origin && git -C C:/Users/barri/Developer/brehon-fork rev-parse --short governance-v0` — should be `3915b0fe2` or later; if drifted, `git log --oneline 3915b0fe2..governance-v0`.
3. Read `workflow_state_role_customization.md` (the ONE lane record — T4b SHIPPED, T5 HELD, check-role-health findings). It auto-loads via MEMORY.md.
4. `.claude/decision-queue.json` — DQ pending should be 0 at handoff.

## Next concrete action — GATE CHECK (not T5 work)

Run the role-health gate measurement:

1. `bash scripts/brehon/drain-role-signal-queue.sh` — drain the queue (ingests the ~13 pending rows + any new since). Per `feedback_pmd_retro_check_http_store_split.md`, confirm it targets the canonical store, not the laptop `100.104.171.26`.
2. `/check-role-health` (no arg = all four roles) — read the per-role `n_tasks` (non-sentinel) counts.
3. **Decision:**
   - **All four roles ≥5 non-sentinel dispatches** → the gate has lifted. NOW author the T5 brief: read plan §Verification (5 steps), `.claude/PRPs/briefs/role-customization-T5-*.md` → run the 5 verify steps → record results in the lane record. T5 is itself a verification task (no new code), likely advisor-driven inline.
   - **Any role < 5** → gate still held. Update the lane record's T5 line with the fresh per-role counts + date, and STOP. role-customization parks here; the next track is a different initiative (not role-customization). Surface to user: "T5 still held: <role>×<n> below the ≥5 gate."
   - **User waives the gate** (per `feedback_minimax_trial_run_below_threshold_on_user_override.md` — the user may waive a ≥N threshold) → proceed to T5 with the waiver noted in the lane record.

---

## 1. role-customization-T5 in one paragraph

T5 is the **end-to-end verification** of the role-customization substrate: run the
5 verification steps from the initiative plan §Verification against *real* per-role
traffic to confirm (a) the `role-signal-utilisation.sh` Stop hook fires + writes
correct rows for each of the 4 Junior roles, (b) the drain + `/check-role-health`
pipeline aggregates them correctly, (c) the per-role strip-candidate analysis
produces actionable signal. It's a verification phase — no new substrate code; the
substrate shipped in T1–T4b. DoD: the 5 verify steps run green against a corpus
with ≥5 non-sentinel dispatches per role.

## 2. Why T5 is held (not "easier/harder")

T5 is **gated, not blocked-by-difficulty.** The work itself is light (run 5 verify
steps, read the report). What's missing is the *input*: enough real per-role Junior
traffic for the verification to be meaningful. The corpus has been thin because
role-customization shipped during a stretch of advisor-side / meta-work phases
(M3-core was Junior-heavy but those dispatches predate parts of the instrumentation,
and the recent phases — pilot, T4b — were advisor-inline with little Junior
dispatch). The gate exists precisely so T5 doesn't produce a false "verified" on an
empty corpus (same false-confidence class as the store-split incident).

## 3. Lessons from T4b that apply to T5

- **Dry-run the dependency before acting** (`feedback_dogfood_slash_command_specs.md`): the T4b drain dry-run was the highest-value 3 seconds of the phase — it both validated the script and quantified the gap. For T5, run the drain + `/check-role-health` FIRST to measure the gate before authoring any T5 brief.
- **Don't start a held/gated phase on momentum** — the T5 hold is the whole point of this bootstrap. Per `feedback_falsifiable_hypothesis_before_structural_fix.md`: "the gate is lifted" is a hypothesis; measure it (step 2 above) before believing it.
- **Verify against reality, not the index line** (`feedback_runbook_audit_drift_post_event_check.md`): the MEMORY.md "Next: T4a" line was stale at the start of T4b (T4a had shipped). Re-measure per-role counts live; don't trust the 2026-05-25 findings as current.
- **Match ceremony to task size** (`feedback_principles_not_rules.md`): T5 is a verification task, advisor-inline — no cohort dispatch, no bm-* flow.

## 4. role-customization-T5 watchlist

- **`scripts/brehon/drain-role-signal-queue.sh`** — must scp from `homeserver` and ingest into the canonical store, NOT the laptop file at `100.104.171.26` (store-split hazard, `feedback_pmd_retro_check_http_store_split.md`). The drain dry-run at T4b confirmed it runs; verify the ingest target at T5.
- **`.claude/hooks/role-signal-utilisation.sh`** — the Stop hook that writes signals. The Step-5b dispatch-vs-signal-rate WARN in `~/.claude/commands/check-role-health.md` catches the "hook silently exit-0 on role gate" defect class (4× historical recurrence). If T5's measurement shows dispatches but no new signals, run `bash .claude/hooks/test-role-signal-utilisation.sh`.
- **Sentinel filtering** — `check-role-health` excludes smoke-test sentinel rows; the ≥5 gate is on *non-sentinel* counts. Don't count sentinels toward the gate.
- **`.claude/roles/<role>/rules.allowlist` + `mcp.json`** — T5 may surface strip candidates (rules/MCPs never used). Strip PROPOSALS are user-gated, NOT a T5 deliverable — T5 only verifies the pipeline produces them; it does not apply strips.
- **Plan §Verification** at `C:/Users/barri/.claude/plans/i-am-interesting-assessing-ethereal-shore.md` — the authoritative 5-step T5 contract. Read it before authoring the T5 brief (only after the gate lifts).

## 5. Operational rules

- role-customization = `.claude/`/`scripts/`/`.claude/hooks/`/skill meta-work → **direct-on-governance-v0, no PR, no bm-cut** (`phase-branch.md`).
- ONE lane record (`workflow_state_role_customization.md`), not per-task records — the transition skill updates its status line rather than closing/creating per-task files.
- Commit subjects: `chore(role-custom):` or `docs(role-custom):` as appropriate; direct push to governance-v0.
- The drain + `/check-role-health` measurement runs **laptop-side** (scp from homeserver + canonical-DB ingest); cannot be delegated to the daemon.
- DQ attribution: `chore|docs(advisor|decision-queue):` for any DQ writes.
- The 6 user gates mostly don't bind (no PR/merge). Judgment-heavy-DQ gate applies if T5 surfaces an ADR/scope question.
- Model tiering, clarify gate, memory headroom: standard advisor-orchestrator rules apply but most are dormant for a verification task.

## 6. What changed from T4b's rule set

- T4b was a single direct-commit skill edit; T5 is a verification task gated on traffic. The big difference: **T5's first action is a gate check, not phase work.** If the gate is held (likely), T5 does not start — role-customization parks and the next session works a different track.
- No new substrate code in T5 (T4b added a skill step; T5 verifies the whole substrate). The drain/health pipeline T4b wired into weekly-review IS part of what T5 verifies.

## 7. Catch-fire procedures

- Universal triggers per `.claude/rules/advisor-orchestrator.md` §5.6.
- Phase-specific: if the drain targets the laptop PMD file (`100.104.171.26`) instead of the canonical store, STOP — store-split hazard (`feedback_pmd_retro_check_http_store_split.md`).
- If `/check-role-health` shows dispatches-but-no-new-signals over 24h, STOP and run the hook smoke harness — the instrumentation is broken and T5 would verify a dead pipeline.
- If tempted to start T5 with any role < 5 non-sentinel dispatches and no user waiver, STOP — that's running a held phase, the exact thing this bootstrap exists to prevent.

## 8. Archive after role-customization-T5

If T5 runs and ships: standard close via `/brehon-phase-transition role-customization-T5 <next-track>`. Since role-customization uses ONE lane record, the transition updates the status line (T5 SHIPPED) and the lane initiative completes. If T5 stays held, there is no transition — role-customization parks; the next track is a separate initiative bootstrapped on its own.

---

## Git state at handoff (captured literally — do not paraphrase)

- governance-v0 HEAD: `3915b0fe2` (captured 2026-06-22) — `feat(weekly-review): add role-signal drain + role-health step (role-customization-T4b)`
- Phase branch HEAD: (none — role-customization is direct-on-governance-v0; no phase branch)
- Recent governance-v0 commits (`git -C C:/Users/barri/Developer/brehon-fork log --oneline -5 governance-v0`):

  ```
  3915b0fe2 feat(weekly-review): add role-signal drain + role-health step (role-customization-T4b)
  d2ad8d88a fix(advisor): correct bm-pr gate-bypass RCA + ship interim trunk-state guard + daemon spec
  89be5b8d7 chore(brehon): close m3-core-e2e-pilot, bootstrap role-customization-T4b
  6c751199a chore(decision-queue): resolve 10 superseded earlier-cycle validate fails at phase close (advisor-laptop)
  e3a051b84 fix(bm-pr): guard against daemon-finalize leaking phase content into trunk
  ```

  (This bootstrap's own transition commit — `chore(brehon): close role-customization-T4b, bootstrap role-customization-T5` — lands on top of `3915b0fe2` when Step 4 commits.)

## Decision-queue snapshot at handoff

```decision-queue-snapshot
(empty at handoff — DQ pending = 0)
```

## Stop-and-ask tripwires

- Stop and ask if: any role has < 5 non-sentinel dispatches after the drain + `/check-role-health` measurement AND the user has not waived the gate — T5 does not start; role-customization parks.
- Stop and ask if: the drain or `/check-role-health` no longer runs (the substrate may have drifted; verify both live before any T5 work).
- Stop and ask if: the drain would write to the laptop PMD file (`100.104.171.26`) rather than the canonical store — store-split hazard, not a unilateral fix.
- Stop and ask if: T5's scope grows beyond running the plan §Verification 5 steps (e.g. it wants to APPLY strips or edit role manifests) — strip application is user-gated and not part of T5.
