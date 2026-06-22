---
phase: role-customization-T4b
plan: C:/Users/barri/.claude/plans/i-am-interesting-assessing-ethereal-shore.md   # the role-customization initiative plan (§T4b)
phase_branch: (none — role-customization is direct-on-governance-v0 meta-work per phase-branch.md; no PR flow)
worktree: C:/Users/barri/Developer/brehon-fork (canonical, governance-v0)
authored: 2026-06-22
authored_by: advisor (canonical brehon-fork / governance-v0 session)
purpose: Bootstrap the role-customization-T4b advisor session. Read the RESUME block first.
---

# ⏩ RESUME — read this block first

**You are the advisor for Brehon role-customization-T4b.** This is the role-customization initiative (role-customized Claude Code harness; laptop advisor vs EliteDesk Junior; RLS-driven evolution). M3-core-e2e-pilot is CLOSED (shipped 2026-06-22, PR #208). T1a/T1b/T2/T3a/T3b/T3c/T4a are all SHIPPED. **T4b is the next pending task.**

role-customization is `.claude/`/`scripts/`/`.claude/hooks/` meta-work → **direct-on-governance-v0, NO PR flow** (per `phase-branch.md`). No bm-cut, no phase branch.

## Session-start ritual (do these first)

1. `pwd && git -C C:/Users/barri/Developer/brehon-fork branch --show-current && git -C C:/Users/barri/Developer/brehon-fork worktree list` — confirm CWD = canonical `brehon-fork` on `governance-v0`.
2. `git -C C:/Users/barri/Developer/brehon-fork fetch origin && git -C C:/Users/barri/Developer/brehon-fork rev-parse --short governance-v0` — should be `6c751199a` or later; if drifted, `git log --oneline 6c751199a..governance-v0`.
3. Read the role-customization running-state record `workflow_state_role_customization.md` (the lane's living scratchpad — T4b status, T5 hold condition, check-role-health findings). It auto-loads via MEMORY.md.
4. Read the initiative plan `C:/Users/barri/.claude/plans/i-am-interesting-assessing-ethereal-shore.md` §T4b for the task contract, and the handover `.claude/PRPs/handovers/role-customization-2026-05-24-session3.md` (the authoritative session-3 handover the lane record cites).
5. `.claude/decision-queue.json` — DQ pending should be 0 at handoff.

## Next concrete action

**T4b: extend the `weekly-review` skill with a role-health step + a JSONL-queue drain call.** Concretely: add a step to the weekly-review skill that (a) runs the role-signal queue drain (`drain-role-signal-queue.sh` — the scp-based version fixed 2026-05-25 in `ea99b6acf`), then (b) runs the `/check-role-health` report (per-role strip-candidate analysis). This wires role-health into the existing weekly cadence rather than ad-hoc. Author the change against the weekly-review skill, dogfood it, commit direct-on-governance-v0.

NOTE: T5 (end-to-end verify) is PENDING but **HELD** — it needs ≥5 non-sentinel dispatches per role, and the corpus is still thin (ci-watcher×0, impl-task×1 as of 2026-05-25). Do NOT start T5 until that traffic threshold is met. T4b has no such gate.

---

## 1. role-customization-T4b in one paragraph

The role-customization initiative gives each Brehon role (advisor / planning / impl-task / bm-task / ci-watcher) its own startup context + tools, evolving via RLS-driven feedback. The substrate shipped (T1-T3); T4a added `/check-role-health` (drain + PMD query + per-role strip-candidate report). **T4b** makes role-health a *recurring* step inside the weekly-review skill (drain the role-signal JSONL queue + run the health report), so strip-candidate signals accumulate on a cadence instead of ad-hoc. DoD: the weekly-review skill runs the drain + health step; dogfooded once.

## 2. Why T4b is easier than M3-core-e2e-pilot

**Easier:** T4b is a single skill-extension (weekly-review) + a drain-call wiring — `.claude/` meta-work, no Rust, no e2e stack, no Junior dispatch, no PR/CR flow. Direct-commit on governance-v0. The drain script (`drain-role-signal-queue.sh`) + `/check-role-health` already exist and are verified — T4b composes them into the weekly cadence. **Not easier:** the weekly-review skill is a real skill with its own structure; the role-health step must slot in without breaking the existing weekly-review steps — read the skill end-to-end before editing (`feedback_read_canonical_before_writing_spec.md`).

## 3. Lessons from M3-core-e2e-pilot that apply

- **Advisor-side:** the bm-pr daemon-finalize gate-bypass (`feedback_bm_pr_daemon_finalize_merges_phase_into_trunk.md`) — but T4b is direct-on-governance-v0 with NO PR, so this does not apply here; noted for the next PR-flow phase.
- **Falsifiable-hypothesis before flagging** (`feedback_falsifiable_hypothesis_before_structural_fix.md`) — verify a suspected miss against the actual artifact before catch-fire (caught two false alarms this phase).
- **Stale-index discipline:** the MEMORY.md "Next: T4a" line was stale (T4a had shipped). Verify ship-status against the authoritative workflow-state record, not the index line (`feedback_runbook_audit_drift_post_event_check.md`).
- **Dogfood slash-command/skill specs before commit** (`feedback_dogfood_slash_command_specs.md`) — T4b edits a skill; walk it through once.

## 4. role-customization-T4b watchlist

- **The weekly-review skill file** (locate via `ls ~/.claude/skills/weekly-review/` or `.claude/commands/` — confirm path before editing) — the role-health step must be added WITHOUT disturbing the existing weekly-review steps; read the full skill first.
- **`drain-role-signal-queue.sh`** (laptop + EliteDesk `dist/scripts/`) — uses scp not rsync (Git Bash Windows workaround, `ea99b6acf`); the weekly-review drain call must invoke the scp version, not re-introduce rsync.
- **`/check-role-health`** at `~/.claude/commands/check-role-health.md` — the report command T4b calls; confirm it still runs (27-day-old record; verify live).
- **PMD HTTP transport** — the role signals flow to the canonical PMD via the HTTP daemon on homeserver (`100.81.145.58:11435`, per `pmd-invariants.md` #1; NOT the laptop `100.104.171.26`). A drain that targets the wrong store strands rows.
- **T5 hold gate** — do NOT advance to T5 (end-to-end verify) until ≥5 non-sentinel dispatches per role exist; the corpus is thin.

## 5. Operational rules

- role-customization = `.claude/`/`scripts/`/`.claude/hooks/`/skill meta-work → **direct-on-governance-v0, no PR, no bm-cut** (`phase-branch.md`).
- Commit subjects: `chore(role-custom):` or `feat(weekly-review):` as appropriate; direct push to governance-v0.
- No Junior dispatch needed for a skill edit — the advisor authors it inline (this is laptop-side `Agent`-tool / direct-edit work, not a four-role Junior task). If a verification run needs the EliteDesk (e.g. the EliteDesk drain side), SSH `homeserver`.
- Dogfood the skill change before commit (`feedback_dogfood_slash_command_specs.md`).
- DQ attribution: `chore|docs(advisor|decision-queue):` for any DQ writes.
- The 6 user gates mostly don't bind here (no PR/merge); plan-approval is moot (the plan §T4b is the contract). The judgment-heavy-DQ gate still applies if T4b surfaces an ADR/scope question.

## 6. What changed from M3-core-e2e-pilot's rule set

- No e2e stack, no docker, no daemon cargo, no PR flow, no CR triage, no merge gate. T4b is a single direct-commit skill edit. The whole heavy orchestration apparatus (cohort dispatch, validate-pending DQ, bm-* verbs) is dormant for this task.

## 7. Catch-fire procedures

- Universal triggers per `.claude/rules/advisor-orchestrator.md` §5.6.
- Phase-specific: if T4b's drain call targets the laptop PMD file (`100.104.171.26`) instead of the homeserver HTTP daemon (`100.81.145.58:11435`), STOP — that's the store-split hazard (`feedback_pmd_retro_check_http_store_split.md`).
- If the weekly-review skill edit would break an existing weekly-review step, STOP and re-read the skill end-to-end.

## 8. Archive after role-customization-T4b

Standard close: run `/brehon-phase-transition role-customization-T4b <next>`. Since role-customization uses ONE lane record (not per-task records), the transition updates `workflow_state_role_customization.md`'s status line (T4b SHIPPED) rather than closing/creating a per-task file. The `<next>` is likely T5 — but ONLY if the ≥5-non-sentinel-dispatches-per-role hold condition is met by then; otherwise role-customization parks and the next phase is a different track.

---

## Git state at handoff (captured literally — do not paraphrase)

- governance-v0 HEAD: `6c751199a` (captured 2026-06-22) — `chore(decision-queue): resolve 10 superseded earlier-cycle validate fails at phase close (advisor-laptop)`
- Phase branch HEAD: (none — role-customization is direct-on-governance-v0; no phase branch)
- Recent governance-v0 commits (`git -C C:/Users/barri/Developer/brehon-fork log --oneline -5 governance-v0`):

  ```
  6c751199a chore(decision-queue): resolve 10 superseded earlier-cycle validate fails at phase close (advisor-laptop)
  e3a051b84 fix(bm-pr): guard against daemon-finalize leaking phase content into trunk
  abd004006 docs(lessons): bm-pr daemon-finalize merges phase into trunk — gate-bypass root cause (PR #208)
  5c6481880 chore(merge): finalize m3-core-e2e-pilot bm-pr (job-772)
  7a4a22169 Merge branch 'governance-v0' into phase-m3-core-e2e-pilot
  ```

## Decision-queue snapshot at handoff

```decision-queue-snapshot
(empty at handoff — DQ pending = 0; 10 superseded M3 cycle entries resolved at 6c751199a)
```

## Stop-and-ask tripwires

- Stop and ask if: T4b's scope grows beyond the weekly-review skill edit + the drain-call wiring (e.g. it wants to touch the role manifests or propose strips) — strip proposals are gated on traffic data, not part of T4b.
- Stop and ask if: the weekly-review skill cannot be located at a `~/.claude/skills/weekly-review/` or `.claude/commands/` path — confirm the actual weekly-review entry point before editing anything.
- Stop and ask if: `/check-role-health` or `drain-role-signal-queue.sh` no longer runs (27-day-old record; the harness may have drifted) — verify both live before wiring them into weekly-review.
- Stop and ask if: the role-signal queue drain would write to the laptop PMD file rather than the homeserver HTTP daemon — that's the store-split hazard, not a T4b decision to make unilaterally.
