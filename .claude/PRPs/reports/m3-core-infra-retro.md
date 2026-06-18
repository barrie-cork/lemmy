# Retro: m3-core-infra — M3 town-hall RTC stack made deployable + fully optional

**Date:** 2026-06-18
**Phase branch:** `phase-m3-core-infra` (merged to `governance-v0` via PR #201 merge-commit — clean, gates honoured)
**PR:** #201 → `governance-v0`, MERGED `2026-06-18T19:43:59Z` @ `c3a13570beef` (merge-commit, not squash; phase branch deleted)
**Tasks:** 6 impl (Tasks 1–6) + fix-impl cohort (4a, 5a, cr-4/5, cr-6/7/8, cr-4b) + bm-cut + bm-pr + bm-poll-cr
**Trunk landing:** `c3a13570b` on `origin/governance-v0`; + 2 post-merge follow-ups (`1fb97d30c` retro bookkeeping, `ba4805a3b` Cargo.lock sync)

---

## §1 What shipped

The M3 town-hall RTC stack is now **deployable and fully optional** — a governance-only instance runs identically to before with the stack absent; `rtc_enabled=true` lights up the LiveKit/lk-jwt/Element Call sidecars and a pseudonym-preserving JWT path.

| # | Task | Files | Outcome |
|---|---|---|---|
| 1 | `rtc_enabled` config seed (default **false**) | `migrations/…seed_rtc_enabled_config/{up,down}.sql` | ✅ |
| 2 | `bridge_room` RTC columns (chair_id/queue_state/recording_config), PRAGMA-gated idempotent ALTER | `services/bridge/src/bridge_room.rs` | ✅ |
| 3 | LiveKit HS256 pseudonym-JWT mint (`sub: identity`, no person_id) + optional LiveKit config | `services/bridge/src/{livekit_jwt.rs,config.rs,main.rs}` | ✅ |
| 4 | `get_bridge_actor_pseudonym` endpoint + `/bridge/actor-pseudonym` route + e2e | `crates/api/api/src/governance/bridge_read.rs`, `crates/api/routes/src/lib.rs`, `crates/server/tests/e2e/governance.rs` | ✅ |
| 5 | 3 RTC Docker sidecars under `profiles: ["rtc"]` + AGPL-NOTICE rows | `services/bridge/docker-compose.yml`, `AGPL-NOTICE.md` | ✅ |
| 6 | `rtc_enabled=false` clean-posture e2e (governance unaffected) | `crates/server/tests/e2e/governance.rs` | ✅ |

**ADR pins held:** ADR-015 (JWT `sub` = pseudonym only; `BridgeActorPseudonym { pseudonym }` response carries no person_id/username/email — verified, see §3.1). ADR-011 (Element Call AGPL-3.0 → 3 AGPL-NOTICE rows). ADR-016 C-track (cross-app backplane RTC layer).

**Validation:** Windows `cargo check --workspace --features full` EXIT 0 (Task 4). Bridge Linux compile (`cargo-linux.sh check`, Docker rust:1.95) EXIT 0. e2e: 147 total (145 pre-phase + `m3_actor_pseudonym` ×2 + `m3_rtc_disabled` ×1), all scoped runs PASS. Deploy-smoke: all 3 sidecars boot (livekit `--dev` Up, lk-jwt :8085 200, element-call :8086 200); no-profile `up` starts none.

---

## §2 What went well

**Optionality was real, not aspirational.** Task 6's clean-posture e2e is the proof, not a claim — `rtc_enabled=false` exercises the full governance path with the RTC stack provably absent. Building the test that exercises the optional-off case (per `feedback_build_what_tests_exercise`) is what makes "fully optional" a verified property.

**ADR-015 false-positive was caught, not rubber-stamped.** CodeRabbit cr-3 claimed the actor-pseudonym endpoint needed an explicit `scrub()` of person identity. Falsified against the code before acting (per `feedback_verify_automated_reviewer_claims_against_compiler` + the falsifiable-hypothesis discipline): the `pseudonym` field **is** the redacted value, the sibling `get_bridge_status` doesn't scrub either, and there's no `scrub()` convention to mirror. Rebutted with evidence rather than authoring a spurious fix. One CR finding correctly rejected out of eight.

**Linux compile gate fired exactly where it should.** Task 3 (livekit-jwt, new `services/bridge` code) and Task 2 (bridge_room migration-shaped ALTER) raised `validate-pending-laptop-linux` DQs and got Docker-rust:1.95 proofs — Windows-green ≠ Linux-green is genuinely plausible for bridge code. The Lemmy-side pure-logic tasks correctly skipped it.

**Merge was clean and gated.** Unlike m3-core-entry-kinds (§3.1 of that retro — daemon finalize footgun bypassed gates 3+5), this phase's bm-pr deferred merge correctly, CR review completed, gate 3 (triage) and gate 5 (merge confirm) both ran interactively, and the advisor ran `gh pr merge --merge` inline per L15. The post-bm-pr OPEN-PR check (the §3.1 preventative from the prior retro) held.

---

## §3 What was rough

### 3.1 Fix-impl workers skip the validate-pending DQ-write (the dominant process signal)

**Observed twice this phase, same cohort.** The cr-fix briefs (#703 bridge cr-6/7/8, #704 lemmy cr-4/5) each instructed the worker to write a `validate-pending-laptop-{linux,e2e}` DQ entry, commit, push, then stop. Both workers made the code edit, committed, pushed, wrote their retro, and reported done — **neither raised the DQ**. Fix commits `2e4926322` (#703) and `d421fb61c` (#704) touched zero `decision-queue.json` lines.

**Falsified the framing before lessoning.** The symptom looked like "finalize-merge dropped the DQ" (the user's initial framing when asking for the lesson). Checked the fix commits' stats — they never touched `decision-queue.json` at all, so the worker never wrote the DQ in the first place. This is a *different* failure from the finalize-merge-drop family; wrote the lesson with the accurate mechanism rather than the assumed one.

**Why it's low-outcome / moderate-process:** the advisor already knew exactly what each fix needed (the brief specified the cargo/e2e/Linux command + filter), so the workaround — verify the fix landed via DoD grep + run the validation directly from the throwaway worktree — cost nothing and never blocked. But an advisor that *waited* for the DQ to surface would hang indefinitely.

**Fix applied:** lesson `feedback_fix_impl_workers_skip_validate_pending_dq.md` (committed `ac1b30859`). Belt-and-braces brief wording added ("push the DQ BEFORE writing your retro"); the durable mitigation is advisor verify-and-run-directly, treating the fix-impl validate DQ as best-effort not load-bearing. Promotion trigger: a 3rd fix-impl skip → `pattern_*` + a hard finalize-gate in `.claude/agents/impl-task.md`.

### 3.2 Daemon `status` field lags actual task completion

Multiple times the daemon reported a task `running` when the worker had in fact finished (verified via the worker log's terminal lines / the `jobs.updated_at` timestamp being stale-but-final). Not a new bug (`feedback_task_notification_exit_summary_unreliable` family) but worth re-noting: don't trust the daemon `status` field as the completion signal — confirm via log tail or the phase-branch tip advancing. No fix needed; the existing verify-the-output discipline already covers it.

### 3.3 Deploy-smoke caught two real config bugs (Task 5 → fix-impl-5a)

Element Call image tag was `0.6.0` (manifest-unknown; the real tag is `v0.6.0`, v-prefixed) and livekit's `--config /etc/livekit.yaml` exited with no mounted config. Both only surfaced by *actually booting* the sidecars (per `pattern_test_against_reality_not_syntax`) — a compose-file lint would have passed both. Fixed via fix-impl-5a (`cdf5a3b46`): livekit → `--dev`, element-call → `:v0.6.0`. Re-validated boot. Good catch by the §15.6 deploy-smoke gate; the lesson is that profile-gated service configs need a real boot, not just a YAML review.

### 3.4 Cargo.lock not shipped with its Cargo.toml change (post-merge loose end)

PR #201 shipped `services/bridge/Cargo.toml`'s promotion of `ed25519-dalek`/`hex` to direct deps (for the future task-12 link-claim handler) but not the matching lockfile entries — they sat as uncommitted canonical WIP across the session boundary. Caught at the post-merge orphan-WIP sweep, Linux-compile-proven (`cargo-linux.sh check` EXIT 0), and committed as a follow-up (`ba4805a3b`). Minor: the next bridge build would have regenerated it anyway, but committing keeps lock↔manifest in sync. Watch: when a task adds a direct dep, the lockfile sync should land in the same commit, not as a follow-up.

---

## §4 Four-role signals

- **Advisor:** Clean orchestration end-to-end through six impl tasks + a CR-fix cohort across a multi-hour, multi-session (compaction) span. Two judgment wins: (1) rebutting cr-3 with code evidence instead of authoring a spurious ADR-015 fix; (2) falsifying the "DQ lost at finalize" framing before writing the lesson — wrote the true mechanism (worker never raised it). Validation discipline held: ran all fix-impl validations directly from throwaway worktrees rather than waiting on DQs that never came. `retro_bypass` rate: N/A (no Stop-hook fail-opens this phase; the in-session Stop-hook prompts were the normal retro requirement, satisfied).
- **Planning:** Plan logic sound; §15 DoD commands executable as written (the `services/bridge` cargo commands correctly used `cargo-linux.sh --manifest-path`, not the Windows-local form). §16a stories were specific and checkpointable — all 4 verified with real commands, no phantoms. Planning ran in a prior session.
- **Impl (Junior #693–#699):** First-cycle clean on Tasks 1, 2, 3, 5, 6. Task 4 needed one fix-impl (E0432 Crud import nested in wrong crate → standalone `use lemmy_diesel_utils::traits::Crud;`) — a real import-path defect, not a process miss, fixed in 2 min (#697). The original impl-task workers DID raise their validate DQs (it's the *fix*-impl variant that skips — see §3.1). MIRROR discipline followed.
- **BM (Junior #692 bm-cut, #701 bm-pr, #702 bm-poll-cr):** All three verbs correct. bm-pr opened a non-draft PR and **deferred merge** (the §3.1-prior-retro footgun did NOT recur — the OPEN-PR check confirmed PR #201 stayed OPEN until the advisor merged). bm-poll-cr ingested 8 CR findings + flagged Copilot quota-block cleanly.

---

## §5 Per-task complexity

Per-task `<files>/<commits>/<runtime-min>/<max-log-silence-min>`. Runtimes from daemon `jobs` table (`(updated_at-created_at)/60`). The 01:31→12:26 gap between Task 2 and Task 3 was the overnight pause + gate-1 plan approval, NOT task runtime.

| Task (job) | Files | Commits | Runtime | Notes |
|---|---|---|---|---|
| planning (#691) | 1 plan | 1 | 16.8 min | Opus plan-shaping |
| bm-cut (#692) | — | 1 | 1.9 min | Clean |
| impl 1 rtc-enabled-seed (#693) | 2 | 1 (+DQ) | 21.9 min | config seed migration |
| impl 2 bridge-room-rtc (#694) | 1 | 1 (+DQ) | 23.6 min | PRAGMA-gated idempotent ALTER |
| impl 3 livekit-jwt (#695) | 3 | 2 | 30.5 min | HS256 JWT mint; new bridge module |
| impl 4 actor-pseudonym (#696) | 3 | 1 (+DQ) | 6.2 min | endpoint+route+e2e |
| fix-impl 4a Crud import (#697) | 1 | 1 | 2.0 min | E0432, 1-line import fix |
| impl 5 RTC sidecars (#698) | 2 | 1 | 2.5 min | compose profiles + AGPL rows |
| impl 6 clean-posture e2e (#699) | 1 | 1 | 5.3 min | optionality proof |
| fix-impl 5a deploy-smoke (#700) | 1 | 1 | 1.8 min | livekit --dev + element-call v0.6.0 |
| bm-pr (#701) | — | 1 | 3.3 min | PR #201 opened, merge deferred |
| bm-poll-cr (#702) | YAML | 1 | 4.6 min | 8 CR findings ingested |
| fix-impl cr-6/7/8 bridge (#703) | 3 | 1 | 3.8 min | PRAGMA/env/TTL guards; skipped validate-DQ (§3.1) |
| fix-impl cr-4/5 lemmy (#704) | 2 | 1 | 17.0 min | route test + SQLFluff; skipped validate-DQ (§3.1) |
| fix-impl cr-4b E0308 (#705) | 1 | 1 | 2.5 min | SessionMiddleware `**context` double-deref |

Phase complexity: **MEDIUM-HIGH** — 6 impl tasks across two crates + the bridge service, 1 real impl defect (E0432), 2 deploy-config bugs, 8 CR findings (1 rebutted, 7 fixed), 1 false-positive caught. No catch-fires; no cycle-3 §G4 loops.

---

## §6 Carry-forward actions

1. **(DONE this retro — structural gate shipped, did NOT wait for 3rd per user direction)** Added a finalize-gate to `.claude/agents/impl-task.md` §"Output discipline → Finalize-gate" + a matching hard refusal: the validate-DQ write is now a finalize-blocker (with a `git log -3 --stat | grep -q decision-queue.json` self-check + "push DQ before retro" ordering), applying to `fix-impl` explicitly. Lesson updated to record the gate. **The gate is a contract nudge, not daemon enforcement** — so advisor verify-and-run-directly STAYS the durable mitigation: after any fix-impl reports done, verify via DoD grep + run validation directly, never block on the DQ. **Owner: advisor. Trigger: next fix-impl dispatch — verify via DoD grep regardless of the gate. Escalation: if a fix-impl skips AGAIN despite the gate → `pattern_*` + real `executor.ts` finalize enforcement.**
2. **(this phase, done)** Lesson `feedback_fix_impl_workers_skip_validate_pending_dq.md` filed (`ac1b30859`); verify report + retro authored; Cargo.lock loose end closed (`ba4805a3b`, Linux-proven).
3. **(watch, low)** Direct-dep additions should ship their lockfile sync in the same commit (§3.4). Not worth a brief-template edit yet (1st occurrence); note for the M3-phase-3+ bridge work that will *use* `ed25519-dalek` (the task-12 link-claim handler).
4. **(cleanup, deferred)** Remove throwaway validation worktrees: `brehon-fork-validate-693`, `-694`, `brehon-fork-preexist-fix` (the `-validate-<id>` worktrees from this session's e2e/Linux runs). `git worktree remove` each.
