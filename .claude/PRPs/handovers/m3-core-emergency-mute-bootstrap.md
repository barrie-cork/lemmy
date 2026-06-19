---
phase: m3-core-emergency-mute
plan: .claude/PRPs/plans/m3-core-emergency-mute.plan.md   # (not yet authored)
phase_branch: phase-m3-core-emergency-mute                 # not yet created until bm-cut
worktree: C:/Users/barri/Developer/brehon-fork             # canonical until bm-cut; lane worktree optional (Mode A/B)
authored: 2026-06-19
authored_by: advisor (canonical brehon-fork / governance-v0 session)
purpose: Bootstrap the m3-core-emergency-mute advisor session. Read the RESUME block first; it is the entry point.
---

# ⏩ RESUME — read this block first

**You are the advisor for Brehon m3-core-emergency-mute** (M3 town halls, Phase 4, bridge-side). This is a fresh session. The Brehon advisor runs inside the brehon-fork checkout — CWD `C:/Users/barri/Developer/brehon-fork` for governance-v0 meta-work, or `C:/Users/barri/Developer/brehon-fork-<lane>` once `bm-cut` creates the lane worktree (per `.claude/rules/multi-lane-worktree.md`). There is no homeserver advisor session.

## Session-start ritual (do these first)

1. `pwd && git -C C:/Users/barri/Developer/brehon-fork branch --show-current && git -C C:/Users/barri/Developer/brehon-fork worktree list` — confirm CWD/lane.
2. `git -C C:/Users/barri/Developer/brehon-fork fetch origin && git -C C:/Users/barri/Developer/brehon-fork rev-parse --short governance-v0` — must equal `5a58686d7` (see §"Git state at handoff"); if drifted, `git -C C:/Users/barri/Developer/brehon-fork log --oneline 5a58686d7..governance-v0` and update your mental model before acting.
3. Read `.claude/decision-queue.json` (and `scripts/brehon/resolve-dq-canonical.sh m3-core-emergency-mute` once a phase branch exists) for any pending entries; compare against §"Decision-queue snapshot" below (empty at handoff).
4. The brehon-fork `MEMORY.md` auto-loads; `workflow_state_m3_core_emergency_mute.md` is the running-state scratchpad. Read `workflow_state_m3_core_stage_mode.md` (CLOSED) ONCE for carry-forward.

## Next concrete action

Author `.claude/PRPs/briefs/m3-core-emergency-mute-planning-1.md` (scope per `.claude/PRPs/prds/m3-town-halls-rtc.prd.md` §"Phase 4: M3-core emergency-mute" + Decision D3) → `/brehon-clarify` → queue planning Junior → `/auto-phase m3-core-emergency-mute`.

---

## 1. m3-core-emergency-mute in one paragraph

Phase 4 of M3 delivers **federation-wide emergency mute-all** for town-hall rooms: a chair (or moderator) can drop ALL publishers across federated instances in <500ms, measured at the publisher client. Mute-all is enacted via **Matrix power-levels across instances** (OQ-V2-06 — unlike mic-passing which uses LiveKit publish-grants; emergency mute is the cross-instance hammer, so it rides the federation-level power-level mechanism, not per-room LiveKit grants). It emits the `room_mute_all` governance-log chain entry — **first EMISSION** of a const already REGISTERED in m3-core-entry-kinds (Phase 2). Depends on Phase 1 (RTC infra) + Phase 3 (stage mode). DoD: a cross-instance integration test drops all publishers <500ms at the publisher client + the `room_mute_all` chain entry lands with pseudonym payload. Bridge-side (`services/bridge/**`), Linux-compile only.

## 2. Why m3-core-emergency-mute is easier/harder than m3-core-stage-mode

- **Easier:** the entry-kind const (`ENTRY_KIND_ROOM_MUTE_ALL`) already exists (Phase 2) — no `governance_log.rs` const work, no count bump. The emit-intent→drain seam is already built (`Stage::pending_emits` + `drain_emits` + `post_room_event`) — `room_mute_all` likely pushes an EmitIntent the same way `room_chair_override` does. The bridge Linux-compile + validate-pending-laptop-linux discipline is now well-worn (4 phases deep).
- **Not easier / harder:** **cross-instance** is genuinely new — stage-mode was single-room/single-instance; emergency-mute must reach publishers on FEDERATED instances via Matrix power-levels. The <500ms latency budget is a real, measurable acceptance bar (stage-mode's grace was virtual-time deterministic; this one needs a publisher-client measurement). OQ-V2-06 (the power-level mechanism across instances) must be resolvable at plan time — confirm it's not parked. Power-levels are a Matrix-federation surface the bridge hasn't driven for *mute* before.

## 3. Lessons from m3-core-stage-mode that apply to m3-core-emergency-mute

Reference by filename, never duplicated:

- **Advisor-side:** `feedback_daemon_long_name_refspec_finalize.md` (NEW this phase, 4× — `git fetch origin <worker>:refs/heads/_fin<id>` for EVERY finalize-merge touching a `junior/role-*` branch; default, not fallback). `feedback_finalize_merge_where_to_look_first.md` (daemon-local-first finalize: daemon merges to daemon-local gov-v0/phase without pushing — advisor pushes; check daemon vs origin at every merge). `feedback_fix_impl_workers_skip_validate_pending_dq.md` (watch HELD last phase but keep verify-via-DoD-grep). `feedback_verify_automated_reviewer_claims_against_compiler.md` + `feedback_falsifiable_hypothesis_before_structural_fix.md` (CR findings are hypotheses — read the code before triaging as fix-in-pr; cr-4 was REAL, verified by reading `promote_next`).
- **Planning-side:** `feedback_advisor_watchpoint_specificity.md` (cite file:line, not concepts). The deterministic-unit-test DoD worked for stage-mode's timing boundary — but emergency-mute's <500ms is a *cross-instance* measurement, so consider whether a unit test suffices or a docker-gated integration test (like stage-mode's `#[ignore]` `stage_mode.rs`) carries the real signal.
- **Impl-side:** `feedback_bridge_validates_on_linux_not_windows.md` (`cargo-linux.sh --manifest-path services/bridge/Cargo.toml`; Windows ruma-common E0119). `feedback_linux_compile_proof_is_a_gate.md` (validate-pending-laptop-linux DQ at result:pass gates bm-pr). `feedback_clippy_test_style.md` (no unwrap/expect; `Result<()>` + `?`). `feedback_validate_pending_laptop_write_then_stop.md` (workers write the DQ and STOP — NO cargo on EliteDesk). The dead_code-scaffold-ahead-of-caller pattern recurred 3× in stage-mode — if emergency-mute builds an emitter before its drain wiring, the same `#[allow(dead_code)]`-then-remove-when-caller-lands flow applies.
- **BM-side:** `feedback_pr_review_triage_pattern.md` (four-bucket triage). Copilot was quota-limited on #202 — CR may carry the whole review again.

## 4. m3-core-emergency-mute-specific watchlist

1. **OQ-V2-06 resolvability (PRE-PLANNING gate):** before authoring the planning brief, scan `docs/brehon-law-inspired-network/99-decisions-and-open-questions.md` for OQ-V2-06 (cross-instance power-level mute mechanism). If parked, confirm the blocking condition is met or note it in the brief §4. Per `feedback_commit_aggressively_in_shared_repos.md` OQ-resolvability check. The PRD says OQ-V2-06 is referenced for the mute mechanism — verify its status.
2. **`room_mute_all` is EMIT-only, not register:** the const `ENTRY_KIND_ROOM_MUTE_ALL` already exists in `crates/db_schema/src/source/governance/governance_log.rs` (m3-core-entry-kinds, Phase 2). The plan §13 must NOT add a new const or bump the registry count (69→72 already done). A plan that re-registers it is a scope error. Per `feedback_entry_kind_runtime_allowlist_check.md` — verify the const is present BEFORE planning so the brief says "emit the existing const", not "register".
3. **Power-levels ≠ LiveKit grants:** mic-passing (Phase 3) used LiveKit publish-grants; emergency-mute uses Matrix power-levels across instances. The plan must NOT reach for `GrantSink`/`GrantCmd` (those are stage.rs LiveKit-grant machinery). The mute path is a Matrix-API call. Watch for a plan that conflates the two mechanisms in `services/bridge/src/`.
4. **ADR-015 pseudonyms-only on the `room_mute_all` payload:** the actor (who triggered the mute) is a pseudonym; no person_id/username/MXID into the chain entry. Brief §4 must make this load-bearing (`feedback_cheap_model_arm_drops_adr_constraints.md`).
5. **ADR-016 metadata-only:** the chain entry carries mute metadata (actor pseudonym, room, timestamp) — never speech/video content. `post_room_event` is metadata-only.
6. **<500ms latency is a measurable DoD, not a vibe:** the §16a marquee story must assert the cross-instance publisher-client measurement. Decide at plan time: deterministic unit test of the mute-emission path + a docker-gated `#[ignore]` integration test for the real <500ms cross-instance measurement (mirroring stage-mode's `stage_mode.rs` pattern). The unit DoD is the gate; the integration test is Phase-6-pilot-grade.

## 5. Operational rules

Polling cadence ~10 min (`mcp__junior-brehon__list_tasks`). Brief discipline: briefs at `.claude/PRPs/briefs/m3-core-emergency-mute-<role>-<n>.md`, committed to governance-v0 first (Mode B: sync to phase branch via daemon temp-worktree single-file `git checkout origin/governance-v0 -- <brief>`). Pre-queue `memory_search_hybrid` + `/precheck` + §2.4 mandatory file-class lesson injection (bridge files → `feedback_bridge_validates_on_linux_not_windows.md` + `feedback_linux_compile_proof_is_a_gate.md` + `feedback_clippy_test_style.md`). **NO CARGO ON ELITEDESK** — workers write `validate-pending-laptop-linux` DQ and STOP; laptop advisor runs `cargo-linux.sh` (`feedback_validate_pending_laptop_write_then_stop.md`). Shape G RESIDUAL-ONLY (local cargo-linux + CR/Copilot = full internal coverage; public green-check only). Model tiering: Planning→Opus, Impl→Sonnet, BM/ci-watcher→Haiku. Clarify gate before planning. 6 user gates (1 plan-approval, 2 ADR/scope-DQ, 3 CR-triage, 4 e2e-mode [likely n/a — bridge unit-test DoD], 5 merge-confirm, 6 retro). DQ attribution: `chore|docs(advisor|decision-queue):`. Cross-lane running cap ≤2; daemon trunk sync before every dispatch. MANDATORY finalize-hazard check after every merge. Drive end-to-end via `/auto-phase m3-core-emergency-mute`.

## 6. What changed from m3-core-stage-mode's rule set

- **NEW lesson in the corpus:** `feedback_daemon_long_name_refspec_finalize.md` — apply it proactively for finalize-merges (don't wait to discover `origin/<worker>` empty).
- **No entry-kind const work this phase** (it was a Phase 2 deliverable). The entry-kind count cross-check at the NEXT transition is skipped for this phase (no new consts).
- **Cross-instance is new** — stage-mode was single-instance. The watchlist gains the power-levels-vs-grants distinction (item 3) and the OQ-V2-06 pre-planning gate (item 1).
- Everything else carries forward unchanged (bridge Linux-compile, NO-CARGO-ON-ELITEDESK, emit-intent→drain seam, the 6 gates).

## 7. Catch-fire procedures

Universal triggers per `.claude/rules/advisor-orchestrator.md` §5.6: subagent ignores a hard refusal (writes `crates/**` from a bridge-only brief); attribution breach (`answered_by: advisor` in a non-`chore|docs(advisor|decision-queue):` commit); subagent commits directly to gov-v0/main; BM opens PR into main; phase branch dirty when Junior reports complete; ci-watcher exit-code surprise; EliteDesk daemon down. Phase-specific additions: a plan that re-registers `ENTRY_KIND_ROOM_MUTE_ALL` (scope error — already registered); a plan that reaches for `GrantSink`/`GrantCmd` for the mute path (mechanism confusion — mute is Matrix power-levels, not LiveKit grants); a `room_mute_all` payload carrying a real identity (ADR-015 breach — catch-fire, a real id in a hash-chained entry is permanent).

## 8. Archive after m3-core-emergency-mute

Standard close: run `/brehon-phase-transition m3-core-emergency-mute m3-core-recording`. The skill will close `workflow_state_m3_core_emergency_mute.md`, delete the two-ago record (m3-core-stage-mode), create the m3-core-recording skeleton, write its bootstrap, update MEMORY.md, commit on gov-v0. This bootstrap stays in `.claude/PRPs/handovers/` as its own archive (git history is the archive). Note: Phase 5 (recording) is **flag-gated** (`record_town_halls` default false) + adds MinIO — a bigger infra step than emergency-mute.

---

## Git state at handoff (captured literally — do not paraphrase)

- governance-v0 HEAD: `5a58686d7` (captured 2026-06-19) — `docs(retro): m3-core-stage-mode — stage mode shipped; cr-4 single-presenter caught by CR; daemon long-name refspec lesson (4x)`
- Phase branch HEAD: not yet created (branch `phase-m3-core-emergency-mute` cut at bm-cut)
- Recent governance-v0 commits (`git log --oneline -5 governance-v0`):

  ```
  5a58686d7 docs(retro): m3-core-stage-mode — stage mode shipped; cr-4 single-presenter caught by CR; daemon long-name refspec lesson (4x)
  592a1d3bf chore(bm): merge PR #202 complete — runlog entry
  dacbfddaa Merge branch 'governance-v0' into junior/role-bm-task-m3-core-stage-mode-bm-merge-...
  8ac58d36a chore(bm): merge bm-merge task #723 complete — PR #202 shipped
  63315f069 chore(bm): merge PR #202 complete — runlog entry
  ```

## Decision-queue snapshot at handoff

```decision-queue-snapshot
(empty at handoff)
```
Note: at transition time, 3 stale `validate-pending-laptop[-linux]` entries (stage-mode Tasks 1/2/3, ids `8f77bcfc296e-001`/`3a93f5eebf23-001`/`e1256574bda6-001`) were forward-merge artifacts on gov-v0 — the underlying code was validated + shipped in PR #202. They were moved to `resolved[]` at this transition (advisor, result:pass, "phase closed, no re-validation"). If they reappear in `pending[]` on a future fetch, they are still stale — do not re-validate shipped code.

## Stop-and-ask tripwires

- Stop and ask if: the plan §13 adds a new `ENTRY_KIND_*` const or bumps the registry count — `ENTRY_KIND_ROOM_MUTE_ALL` already exists (Phase 2); this phase EMITS it, does not register.
- Stop and ask if: the plan introduces a new migration under `crates/db_schema/migrations/**` or `services/bridge` schema — emergency-mute is mute-logic + chain-emission; a new column/migration is a scope question (the `bridge_room` RTC cols shipped in Phase 1).
- Stop and ask if: OQ-V2-06 (cross-instance power-level mute mechanism) is `parked` in `99-decisions-and-open-questions.md` with an unmet blocking condition — that gates the plan's mute mechanism; resolve or surface before authoring the brief.
- Stop and ask if: the `room_mute_all` chain payload would carry a person_id/username/MXID instead of a pseudonym — ADR-015 hard pin; a real id in a hash-chained entry is permanent and unscrubable.
- Stop and ask if: the <500ms latency DoD has no measurable test (the §16a marquee story must assert the publisher-client measurement, deterministic unit + optional docker-gated integration — not a prose claim).
