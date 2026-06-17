---
phase: m3-core-infra
plan: .claude/PRPs/plans/m3-core-infra.plan.md   # (not yet authored)
phase_branch: phase-m3-core-infra                  # not yet created until bm-cut
worktree: C:/Users/barri/Developer/brehon-fork     # canonical until bm-cut creates a lane (likely Mode B given infra scope)
authored: 2026-06-18
authored_by: advisor (canonical brehon-fork / governance-v0 session)
purpose: Bootstrap the m3-core-infra advisor session. Read the RESUME block first; it is the entry point.
---

# ⏩ RESUME — read this block first

**You are the advisor for Brehon m3-core-infra** (M3 town halls — Phase 1: deployable, fully-optional RTC stack). Fresh or resumed session. The advisor runs inside the brehon-fork checkout — CWD `C:/Users/barri/Developer/brehon-fork` for governance-v0 meta-work, or `C:/Users/barri/Developer/brehon-fork-<lane>` once `bm-cut` creates a lane worktree. No homeserver session.

## Session-start ritual (do these first)

1. `pwd && git -C C:/Users/barri/Developer/brehon-fork branch --show-current && git -C C:/Users/barri/Developer/brehon-fork worktree list` — confirm CWD/lane.
2. `git -C C:/Users/barri/Developer/brehon-fork fetch origin && git -C C:/Users/barri/Developer/brehon-fork rev-parse --short governance-v0` — must equal `71d89ed50` (see §"Git state at handoff"); if drifted, `git log --oneline 71d89ed50..governance-v0` and update your model.
3. Read `.claude/decision-queue.json` (DQ was empty at handoff). Once a phase branch exists, use `scripts/brehon/resolve-dq-canonical.sh m3-core-infra`.
4. brehon-fork `MEMORY.md` auto-loads; `workflow_state_m3_core_infra.md` is the running scratchpad. Read `workflow_state_m3_core_entry_kinds.md` ONCE for carry-forward.

## Next concrete action

Author `.claude/PRPs/briefs/m3-core-infra-planning-1.md` (scope per `.claude/PRPs/prds/m3-town-halls-rtc.prd.md` Phase 1, line 225-228) → `/brehon-clarify` → queue planning Junior. **Before that:** resolve the infra-shape questions in §4 (this is a deploy-heavy phase, not pure-code — the plan must account for sidecars + a bridge-side migration + AGPL-NOTICE rows, and Shape-G/cargo-on-laptop discipline differs for bridge crates which compile on Linux only).

---

## 1. m3-core-infra in one paragraph

Phase 1 of the M3 town-halls cluster: make the real-time-communication stack **deployable and fully optional**, and give the bridge the ability to mint LiveKit JWTs. Scope: deploy LiveKit Server + lk-jwt-service + Element Call sidecars; add an `rtc_enabled` config row (default **false**) to `governance_messaging_config`; bridge LiveKit JWT integration with **identity→pseudonym mapping at JWT-issue time** (ADR-015 pin — the JWT identity must be the room pseudonym, never person_id/username); `bridge_room` RTC-state columns + a **bridge-side migration**; AGPL-NOTICE rows for the new sidecars. **DoD:** `rtc_enabled=false` runs a clean governance-only instance (RTC stack absent, zero side-effects); `rtc_enabled=true` mints a valid LiveKit JWT carrying a pseudonym identity for an `always_pseudonym` room. No emitters of the M3 chair/mute consts here (those land Phase 3/4).

## 2. Why m3-core-infra is easier/harder than m3-core-entry-kinds

- **Harder:** m3-core-entry-kinds was a 3-const pure-logic diff. m3-core-infra is **infra + deploy + bridge-integration + a migration** — multiple moving parts, new external services (LiveKit/lk-jwt/Element Call), and the bridge compiles on **Linux only** (Docker `rust:1.95`, not Windows — `ruma-common` E0119 on Windows). Expect `validate-pending-laptop-linux` DQ entries for bridge changes.
- **Harder (config surface):** the `rtc_enabled` flag must produce a genuinely-clean instance when false — that's a "build only what the flag exercises" discipline (`feedback_build_what_tests_exercise.md`); the off-path must be tested, not assumed.
- **Easier:** the four primitives M3 builds on already shipped (bridge daemon + `bridge_room` table + `append()` writer + `governance_messaging_config` KV + pseudonym allocator). This phase EXTENDS them, doesn't invent them.
- **Not easier:** the marquee M3 risk (federation-wide emergency-mute <500ms) is Phase 4, NOT here — but the JWT-issue pseudonym mapping is the ADR-015 load-bearing seam for the whole cluster; get it right here.

## 3. Lessons from m3-core-entry-kinds that apply to m3-core-infra

- **Advisor-side (THE big one):** the daemon finalize-merge footgun bit on the bm-pr — it pushed phase code to origin trunk, bypassing gates 3+5 (PR #200 auto-closed MERGED, CR review killed mid-flight). See `feedback_junior_finalize_merges_bm_cut_branch.md` (recurrence #3, the bm-pr variant). **MANDATORY guard until the structural daemon fix lands:** on every `done` bm-pr task, BEFORE advancing to cr-wait, verify the PR is OPEN (not MERGED) AND `origin/governance-v0` has not absorbed the phase code. A MERGED-within-minutes PR = the footgun → catch-fire. Also: the bm-pr brief must forbid the worker from `git merge governance-v0 into phase-<X>` + pushing the phase branch.
- **Advisor-side:** verify cargo completion the hard way (log `Finished` marker + exit sentinel + a known-heavy crate compiling), NOT the task-notification exit summary (`feedback_task_notification_exit_summary_unreliable.md`). Held up across a `/compact` mid-build this phase.
- **Advisor-side:** daemon-local trunk sync before every dispatch (`feedback_daemon_local_trunk_stale_multi_lane.md`) — bit twice this phase via push-without-sync divergence.
- **Planning-side:** validate the plan's DoD command-form at gate 1, not just its logic — m3-core-entry-kinds' plan used `rg` (absent) + line-counting greps. For an infra phase, the DoD will include Docker/deploy commands + bridge `cargo-linux.sh` — dry-run them.
- **Impl-side:** bridge cargo runs on Linux via `scripts/brehon/cargo-linux.sh ... --manifest-path services/bridge/Cargo.toml`, NOT Windows-local (`feedback_bridge_validates_on_linux_not_windows.md`). The bridge migration here needs the Linux-compile-proof gate (`feedback_linux_compile_proof_is_a_gate.md`).

## 4. m3-core-infra-specific watchlist

1. **`governance_messaging_config` typed-KV** — the `rtc_enabled` row goes here (the table that already holds `record_town_halls`/decay knobs). Plan §13 must add the row via the existing typed-KV insert path, NOT a new column on a different table. Cite the table + the existing config-key seed pattern.
2. **`bridge_room` RTC-state columns** — this needs a **bridge-side migration** under `services/bridge/migrations/**` (SQLite), NOT a `crates/db_schema/migrations/**` Postgres migration. Plan §13 must name the correct migration dir. A Postgres migration here is a scope error.
3. **ADR-015 pin at JWT issue** — the LiveKit JWT `identity` claim MUST be the room pseudonym (the `always_pseudonym` allocator output), never `person_id`/username/email. Plan §4 must make this a load-bearing DoD: `grep` for the pseudonym→JWT mapping fn AND its callsite, per `feedback_cheap_model_arm_drops_adr_constraints.md` + the §2.4a ADR-load-bearing clause.
4. **`rtc_enabled=false` clean-posture** — the off-path must be a tested success signal (the RTC stack absent, zero side-effects), not an untested assumption. Plan §16a must include a clean-false story.
5. **AGPL-NOTICE rows** — new sidecars (LiveKit, lk-jwt, Element Call) inherit the source-disclosure obligation (ADR-011). Plan must add the NOTICE rows; do not let this slip.
6. **Bridge compiles on Linux only** — every bridge `crates`/`services/bridge` change needs a `validate-pending-laptop-linux` DQ (Docker `rust:1.95`), per `feedback_bridge_validates_on_linux_not_windows.md`. Windows-local bridge cargo will fail with `ruma-common` E0119.

## 5. Operational rules

- Polling cadence ~10 min (`mcp__junior-brehon__list_tasks`). Briefs in `.claude/PRPs/briefs/m3-core-infra-<role>-<n>.md`, committed to governance-v0 first.
- Pre-queue: `memory_search_hybrid` + `/precheck` + §2.4 mandatory file-class lesson injection. Daemon trunk sync before EVERY dispatch (`advisor-orchestrator.md` §4).
- NO-CARGO-ON-ELITEDESK: workers write `validate-pending-laptop` (or `-linux` for bridge) DQ and STOP; laptop runs all cargo.
- Shape G: RESIDUAL-ONLY (public green-check only) — local `cargo-linux.sh` + CR/Copilot = full internal coverage. validate-pending-laptop per `advisor-orchestrator.md` §5.2.
- Windows e2e: bat-wrapper + explicit-exit-marker reads (`feedback_windows_e2e_requires_bat_wrapper.md`); never bare cargo (libpq.dll).
- Model tiering: Planning→Opus, Impl→Sonnet, BM/ci-watcher→Haiku.
- The 6 user gates + clarify gate hold. DQ attribution: `chore|docs(advisor|decision-queue):`.
- **bm-pr OPEN-PR check is now MANDATORY** (see §3) — the daemon-finalize footgun makes this non-optional.

## 6. What changed from m3-core-entry-kinds's rule set

- **New mandatory advisor step:** post-bm-pr OPEN-PR verification (PR is OPEN + origin trunk has not absorbed phase code) before advancing to cr-wait. Added because the m3-core-entry-kinds bm-pr finalize-footgun bypassed gates 3+5.
- **Bridge-Linux discipline is back in scope** (was dormant in the const-only m3-core-entry-kinds): bridge changes → `validate-pending-laptop-linux` + the Linux-compile-proof gate.
- **Migration discipline returns:** this phase has a bridge-side (SQLite) migration; the m3-core-entry-kinds zero-migration shortcut does not apply.

## 7. Catch-fire procedures

- Universal triggers per `.claude/rules/advisor-orchestrator.md` §5.6 (subagent hard-refusal, attribution breach, non-allowlist §G4 fail, ci-watcher exit-code surprise, daemon down).
- **Phase-specific:** a `done` bm-pr task whose PR shows MERGED within minutes = the daemon-finalize footgun → catch-fire, assess the landed diff, do NOT treat as a quiet daemon-local cleanup (it reached origin). A new Postgres migration under `crates/db_schema/migrations/**` (this phase is bridge-SQLite-migration-only) = scope violation → catch-fire.

## 8. Archive after m3-core-infra

Run `/brehon-phase-transition m3-core-infra m3-core-stage-mode` (Phase 1 → Phase 3; Phase 2 already shipped). This skill closes `workflow_state_m3_core_infra.md`, deletes the two-ago record, creates the next skeleton + bootstrap, updates MEMORY.md, commits on governance-v0. This bootstrap file stays as its own archive (git history).

---

## Git state at handoff (captured literally — do not paraphrase)

- governance-v0 HEAD: `71d89ed50` (captured 2026-06-18) — `docs(retro): m3-core-entry-kinds — clean impl + bm-pr finalize-footgun catch-fire (accept+harden)`
- Phase branch HEAD: not yet created (branch `phase-m3-core-infra` cut at bm-cut)
- Recent governance-v0 commits (`git log --oneline -5 governance-v0`):

  ```
  71d89ed50 docs(retro): m3-core-entry-kinds — clean impl + bm-pr finalize-footgun catch-fire (accept+harden)
  e8a7117e3 docs(lessons): finalize-merge bug recurrence 3 — bm-pr variant pushes to origin, bypasses gates 3+5
  088a8be43 Merge remote-tracking branch 'origin/governance-v0' into governance-v0
  a5fc60a2d chore(merge): finalize m3-core-entry-kinds bm-pr task (job-690)
  bbcf7eee9 Merge branch 'governance-v0' into phase-m3-core-entry-kinds
  ```

## Decision-queue snapshot at handoff

```decision-queue-snapshot
(empty at handoff)
```

## Stop-and-ask tripwires

- Stop and ask if: the plan introduces a new migration under `crates/db_schema/migrations/**` — this phase's only migration is bridge-side SQLite under `services/bridge/migrations/**`; a Postgres migration is a scope violation.
- Stop and ask if: the LiveKit JWT `identity` claim is wired to anything other than the room pseudonym (person_id/username/email) — ADR-015 breach.
- Stop and ask if: a bm-pr task reports `done` and its PR is MERGED rather than OPEN — the daemon-finalize footgun fired (gates 3+5 bypassed); catch-fire and assess the landed diff.
- Stop and ask if: the plan ships `rtc_enabled=true` as the default — it must default false (clean-posture-first).
- Stop and ask if: the new sidecars land without AGPL-NOTICE rows — ADR-011 source-disclosure obligation.
