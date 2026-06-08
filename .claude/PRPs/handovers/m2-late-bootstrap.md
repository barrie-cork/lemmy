---
phase: m2-late
plan: .claude/PRPs/plans/m2-late.plan.md   # (not yet authored — gated on OQ resolution)
phase_branch: phase-m2-late                  # not yet created; cut at bm-cut after plan approval
worktree: C:/Users/barri/Developer/brehon-fork-m2late   # created at bm-cut; until then canonical brehon-fork
lane_mode: A                                 # A = dedicated lane worktree (default); B = mobile remote-control. Read by session-start-multi-lane-check.sh to detect declared-vs-actual drift.
authored: 2026-06-06
authored_by: advisor (canonical brehon-fork / governance-v0 session)
purpose: Bootstrap the m2-late advisor session. Read the RESUME block first; it is the entry point.
---

# ⏩ RESUME — read this block first

**You are the advisor for Brehon m2-late.** This is a fresh session (or a resumed one). The Brehon advisor runs inside the brehon-fork checkout — CWD `C:/Users/barri/Developer/brehon-fork` for governance-v0 meta-work, or `C:/Users/barri/Developer/brehon-fork-m2late` once `bm-cut` creates the lane worktree (per `.claude/rules/multi-lane-worktree.md`). There is no homeserver session.

## Session-start ritual (do these first)

1. `pwd && git -C C:/Users/barri/Developer/brehon-fork branch --show-current && git -C C:/Users/barri/Developer/brehon-fork worktree list` — confirm CWD/lane.
2. `git -C C:/Users/barri/Developer/brehon-fork fetch origin && git -C C:/Users/barri/Developer/brehon-fork rev-parse --short governance-v0` — must equal `9a4d702d5` (see §"Git state at handoff"); if drifted, `git -C C:/Users/barri/Developer/brehon-fork log --oneline 9a4d702d5..governance-v0` and update mental model before acting.
3. Read `.claude/decision-queue.json` for pending entries; compare against §"Decision-queue snapshot" below.
4. The brehon-fork `MEMORY.md` auto-loads; `workflow_state_m2_late.md` is the running-state scratchpad.

## Next concrete action

> **⚠️ STALE-FRONTMATTER NOTICE (updated 2026-06-07):** the frontmatter above predates phase start and is NO LONGER accurate. The phase has STARTED: branch `phase-m2-late-1` exists, the plan `.claude/PRPs/plans/m2-late.plan.md` is authored, and Task 1 (sanction_event migration + SanctionKind enum) is COMMITTED. The OQ-gating described below was resolved before bm-cut. Ignore the "not yet created / not yet authored" frontmatter.

**CURRENT STATE: validating Task 1.** There is ONE pending DQ: `001f1c47c5dc-001` (`validate-pending-laptop`). 

**→ Read `.claude/PRPs/handovers/m2-late-1-t1-validate-resume.md` FIRST — it is the verified, turnkey resume plan.** It contains the 4 remaining steps (hand-edit schema.rs → migrate-roundtrip → workspace cargo-check → mutate the DQ to pass) with grep-verified verbatim insertions. Two critical do-NOTs baked into that handover: (1) do NOT run `diesel print-schema` — the fork convention is HAND-EDIT schema.rs (fed-in-a/v1-AD-a/v1-SL-a precedent); the `diesel_ltree.patch` path is a red herring for this task. (2) do NOT run the DQ's literal `cargo run ... -- migration run` command — `diesel_utils/main.rs` bails on any arg.

(Original gating note, now historical: this phase WAS gated on OQ-ADR016-02 + OQ-ADR016-04; those were resolved before the plan was authored. No action needed.)

---

## 1. m2-late in one paragraph

m2-late is M2 Phases 6–7 of the governance-triggered Matrix rooms milestone. Phase 6 adds B-publish sanction propagation: when a Brehon governance decision produces a sanctionable outcome (case resolved → sanction), the binary publishes a structured event to external Matrix subscribers via a webhook-style delivery. Phase 7 (optional) adds B-actor portable-ID linkage, replacing the bridge-local puppet map with Brehon-issued portable IDs. The DoD: Phase 6 ships a sanction event route (`POST /governance/sanction-event`) + subscriber registration + at-least-once delivery. Phase 7 ships the `B-actor` link-flow (user-initiated linkage of their Brehon identity to a Matrix actor). Both phases are gated — see §"Next concrete action" above.

## 2. Why m2-late is harder than m2-rooms-a

**Harder:** m2-rooms-a was self-contained (bridge + binary API contract was internal; test infrastructure was `#[ignore]` + docker-compose). m2-late requires resolving open design questions (OQ-ADR016-02/04) with cross-system scope — the B-publish schema must be stable across Lemmy-fork + Matrix + any future Brehon-governed app. The subscriber delivery model (webhook vs Matrix room vs polling) isn't settled.

**Not harder:** the bridge codebase is familiar (m2-rooms-a established patterns: `bearer middleware`, `append_room_event`, room provisioner pattern). The `governance_case_after_transition` hook is already wired. The crate boundary discipline (bridge ≠ workspace) is established. The plan template + four-role model is well-exercised.

## 3. Lessons from m2-rooms-a that apply to m2-late

### Advisor-side
- **OOM recovery via tar**: pre-cancel SSH tar pattern confirmed effective (`feedback_cancel_before_oom_tar_recovery.md` pending authorship — see retro). Carry forward for any cargo-heavy task.
- **bm-merge brief must distinguish `gh pr merge` from daemon-finalize**: the BM brief must say "call `gh pr merge <N>` — this is NOT the daemon-finalize-merge pattern; the daemon-finalize applies to worker branches, not to phase PRs." Enforce in brief template.
- **Positive cargo-scope instruction insufficient**: any impl-task brief touching `services/bridge/` must include an EXPLICIT prohibition: "do NOT run `cargo check --workspace --features full`; ONLY `cd services/bridge && cargo check`."
- **UNSTABLE `--admin` bypass confirmed (3rd use)**: `feedback_bm_merge_unstable_admin_bypass.md` is load-bearing. No new lesson needed.

### Planning-side
- **Pre-impl data-flow trace for novel seams**: when the plan proposes a data-flow across a crate/binary boundary, trace the actual reachability before dispatching the planning Junior. `feedback_trace_data_flow_precedent.md` covers this. m2-rooms-a T2 required an inline re-plan (DQ -055) because the plan assumed `api → api_utils` reachability that doesn't exist.
- **§16a stories required**: the m2-rooms-a plan was authored before the §16a convention; `/brehon-verify` had to fall back to manual reconciliation. m2-late plan must include §16a stories.

### Impl-side
- **`api_utils/Cargo.toml` dep pre-check**: before dispatching any impl-task that adds diesel/diesel_async imports to `crates/api/api_utils/src/**`, verify the deps are in `api_utils/Cargo.toml`. E0432 recurred twice (T1w + T4a). Add to §4 Constraints in the brief.
- **NO-CARGO-ON-ELITEDESK** (`project_laptop_canonical_cargo_runner.md`) — absolute. Workers write `validate-pending-laptop` DQ + STOP; laptop advisor runs cargo.

### BM-side
- **Sensitive-file classifier**: `.claude/PRPs/reviews/*.yaml` + `.claude/runlog/*.md` block BM workers (3rd recurrence). **Action needed**: add these paths to BM agent allow-list before dispatching any BM task. See MEMORY.md "Watch / promote-if-recurs".

## 4. m2-late-specific watchlist

1. **`docs/brehon-law-inspired-network/99-decisions-and-open-questions.md` §"OQ-ADR016-02"** — B-publish event schema. The plan's §13 must reflect whatever schema is resolved here. Watchpoint: plan §13 T1 (schema) must name the exact JSON field set and `sanction_kind` enum variants; any field not named in §13 is out of scope.
2. **`crates/api/api/src/governance/` — new sanction-event route handler** — must use `conn.run_transaction()` if it writes ≥2 DB rows (per `feedback_multi_write_handlers_need_transactions.md`). Check at plan §13 IMPLEMENT list.
3. **`services/bridge/src/room_provisioner.rs`** — Phase 6 may require updating room provisioner to consume sanction events via the new route. Guard: `bridge_callback_secret` bearer middleware is already wired (cr-5 fix, `521e949e3`); any new bridge route must use it.
4. **`services/bridge/Cargo.toml`** — Phase 6/7 may add new deps. Check `api_utils/Cargo.toml` simultaneously for any workspace-side imports. E0432 class recurred twice in m2-rooms-a; pre-check both manifests before dispatching impl.
5. **ADR-015**: `actor_pseudonym.pseudonym` ONLY to bridge — never real usernames/emails. B-actor Phase 7 in particular: any portable-ID linkage must route through `actor_pseudonym`, not the `local_user` table. Verify at plan §13 IMPLEMENT list.

## 5. Operational rules

**Polling:** ~10 min cadence, `mcp__junior-brehon__list_tasks`; on transition `show_task + git fetch + read DQ` → triage → queue next.

**Briefs:** `.claude/PRPs/briefs/m2-late-<role>-<n>.md`. Planning/BM briefs commit to `governance-v0`; impl-task briefs must be visible on `phase-m2-late` (Mode A: author on phase branch directly). Pre-queue: `memory_search_hybrid` + `/precheck` + §2.4 mandatory file-class lesson injection. LemmyResult Case A override for any e2e brief.

**Cargo validation:** validate-pending-laptop (pre-Shape-G model). Laptop runs all cargo. Workers write DQ + STOP. `cd services/bridge && cargo check` for bridge; `./scripts/brehon/cargo-check.sh --workspace --features full` for workspace. NEVER mix; NEVER run either on daemon.

**Windows e2e:** `cmd //c "scripts\\brehon\\cargo-test.bat ..."` with explicit exit marker; never bare `cargo test`; never `-p lemmy_server --features full`.

**Model tiering:** Planning → Opus 4.8, Impl → Sonnet 4.6, BM/ci-watcher → Haiku 4.5.

**Clarify gate:** run `/brehon-clarify` on every planning brief before queueing Junior. Gate planning on all clarify-DQ resolved.

**6 user gates:** (1) plan approval, (2) judgment-heavy DQ, (3) CR triage, (4) Phase 2 e2e (local vs dispatch — never auto-pick), (5) merge confirm, (6) retro sign-off.

**DQ attribution:** advisor commits `^(chore|docs)\((advisor|decision-queue)\)`. Never write `answered_by: "advisor"` from a non-advisor session. Use `dq-v3-new-entry.sh` + `dq-v3-append-fragment.sh`.

**Shape G:** SUSPENDED (residual-only — use only for public green-check). validate-pending-laptop is the primary validation gate.

**Memory headroom:** no bulk `e2e.rs` reads. cargo-class tasks on daemon = OOM risk — validate-pending-laptop ONLY.

## 6. What changed from m2-rooms-a's rule set

- **m2-late is GATED** — cannot start until OQ-ADR016-02 + OQ-ADR016-04 resolve. All prior rules still apply once unblocked.
- **BM allow-list fix pending** — sensitive-file classifier blocks `.yaml` + runlog writes. Must be resolved before first BM dispatch (see §3 BM-side above + MEMORY.md action-needed).
- **bm-merge brief must have daemon-finalize distinction callout** — new template addition following m2-rooms-a incident.
- **`api_utils` dep pre-check** added to impl-task brief §4 Constraints mandatory items.

## 7. Catch-fire procedures

Universal triggers (`.claude/rules/advisor-orchestrator.md` §5.5):
- Junior subagent ignores hard refusals (impl writes `crates/**` without authorizing brief)
- `answered_by: "advisor"` in commit with non-`^(chore|docs)\((advisor|decision-queue)\)` subject
- Subagent commits directly to `governance-v0` or `main`
- `bm-task` opens PR into `main` instead of `governance-v0`
- Phase branch has uncommitted state when Junior reports complete
- ci-watcher `timed_out` (60-min cap exceeded) — do NOT auto-rerun
- §G4 cycle-count ≥3 same `(error_class, file_basename)` — HARD REFUSAL regardless of allowlist

Phase-specific additions:
- **OQ-ADR016 schema changes mid-impl** — if a new ADR decision changes the B-publish schema after impl tasks have started, stop loop and surface (scope change, gate 2 judgment-heavy DQ).
- **ADR-015 violation in sanction-event route** — if any planned handler would expose real `local_user` fields (email, username) to the bridge, STOP before dispatching (ADR-015 is load-bearing).
- **Cargo on daemon (NO-CARGO-ON-ELITEDESK)** — if a worker runs workspace cargo or bridge cargo on the daemon, cancel immediately, tar the worktree, apply advisor-direct.

## 8. Archive after m2-late

Run `/brehon-phase-transition m2-late <next>`. This skill will: close `workflow_state_m2_late.md`, delete `workflow_state_m2_rooms_a.md` (two-ago), create the next skeleton, write the next bootstrap file, update brehon-fork MEMORY.md, commit on governance-v0. This file stays in `.claude/PRPs/handovers/` as its own archive — git history is the archive.

---

## Git state at handoff (captured literally — do not paraphrase)

- governance-v0 HEAD: `9a4d702d5` (captured 2026-06-06) — `docs(retro): m2-rooms-a — 8-task phase, OOM recovery, bm-merge path correction`
- Phase branch HEAD: not yet created (branch `phase-m2-late` cut at bm-cut, after OQ resolution + plan approval)
- Recent governance-v0 commits:

  ```
  9a4d702d5 docs(retro): m2-rooms-a — 8-task phase, OOM recovery, bm-merge path correction
  1cc74ad50 chore(decision-queue): advisor-laptop PASS — governance-fix-med-1 cargo check exit 0 (b2fcddebf2f8-001)
  9e65cb9d6 chore(bm): merge PR #191 complete — m2-rooms-a
  205ba2399 Merge branch 'junior/role-impl-task-governance-fix-med-1-see-claude-prps-briefs-governance-v0-fix-impl-med-1-md-636'
  4a2adf619 Merge branch 'governance-v0' into junior/role-impl-task-governance-fix-med-1-see-claude-prps-briefs-governance-v0-fix-impl-med-1-md-636'
  ```

## Decision-queue snapshot at handoff

```decision-queue-snapshot
(empty at handoff — 0 pending entries as of 9a4d702d5)
```

## Stop-and-ask tripwires

- Stop and ask if: the OQ-ADR016-02/04 resolution changes the B-publish delivery model away from webhook (the PRD lean assumes webhook; a different model is a plan-scope change requiring user confirmation).
- Stop and ask if: a plan task introduces a new migration under `crates/db_schema/migrations/**` that the phase's DoD does not reference — check that the migration is intentional and not scope creep.
- Stop and ask if: any sanction-event route handler is designed to accept data from an unauthenticated external caller — all new binary routes must require `BRIDGE_CALLBACK_SECRET` bearer auth (`feedback_bm_merge_unstable_admin_bypass.md` plus the bridge_auth pattern from `521e949e3`).
- Stop and ask if: the plan's §13 IMPLEMENT list for Phase 7 (B-actor) does not explicitly route through `actor_pseudonym.pseudonym` — ADR-015 is load-bearing and the B-actor link-flow is the highest-risk ADR-015 surface in this milestone.
- Stop and ask if: the governance-fix-med-1 task (DQ `b2fcddebf2f8-001` — geographic_diversity_score denominator fix, commits `748a68049`+`8aa56e166`, merged `205ba2399`) introduced any API-visible regression you need to account for in m2-late's story coverage.
