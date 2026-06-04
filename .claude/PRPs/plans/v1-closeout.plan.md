# v1 Close-Out — Cleanup & Refactor Plan (hybrid /workflows + four-role)

> **Status:** APPROVED 2026-06-04. Execution handed to the user in a dedicated `brehon-fork-closeout` worktree session. Author: advisor session (canonical `brehon-fork`, `governance-v0`).
> **Plan-file home:** `C:\Users\barri\.claude\plans\steady-seeking-lampson.md` (this file) + tracked copy at `.claude/PRPs/plans/v1-closeout.plan.md` on `governance-v0`.
>
> **Approval decisions (2026-06-04):**
> 1. **Phase 4 / deps-r2** — RUN NOW, gate only T2. T1 (webmention inline), T3 (tar dispute), T4 (wasmtime log) proceed immediately in the close-out lane; T2's `[patch.crates-io]` Cargo.toml edit waits for M1 merge.
> 2. **Phase 5 / wasmtime** — KEEP DEFERRED + document x86_64-not-vulnerable rationale + suppress all 13 in Dependabot + **set a `/schedule` watch on `extism/extism` PR #847** to adopt the upstream release the moment it lands.
> 3. **Promotion** — promote to tracked `.claude/PRPs/plans/v1-closeout.plan.md` (done at Phase 0).
> 4. **First workflow** — Phase 3 carry-patch audit is the small cost-gauge run before broader sweeps.
>
> **Execution owner:** the USER, in a fully-harnessed `brehon-fork-closeout` session (PMD + MCP + hooks wired per Phase 0). This advisor session only prepares the handoff.

---

## Session progress & state  (LIVING — newest entry on top)

> Append a dated block each working session. Keep it factual: what shipped, what's gated, what the next concrete action is. This is the resume surface — a fresh session reads this section + the bootstrap handover and knows where it stands.

### State snapshot (as of 2026-06-04, session post-M1 — M1 MERGED, trunk integrated, Phases 6–8 UNBLOCKED)

| Axis | Value |
|---|---|
| Lane / branch | `brehon-fork-closeout` / `phase-v1-closeout` (HEAD `060bcac15` — the post-M1 merge commit; **all pushed**, 0 behind / 20 ahead origin/gov-v0) |
| Mode | Mode A (dedicated lane worktree) — own CC session, laptop-local |
| Harness | **GREEN — SIGNED OFF** (unchanged; flag `.claude/audit-phase-v1-closeout-complete.flag` SET). |
| M1 gate | **🔓 m1-b MERGED — but ⚠️ SEQUENCING OPEN (see below).** PR #177 (`phase-m1-b`, Tree B) merged into `governance-v0` at `d6d027794`; `git log origin/phase-m1-b ^origin/governance-v0` = **EMPTY**. The plan's literal M1-isolation gate (which keyed on `phase-m1-b`) is satisfied → Phases 6–8 are **code-conflict-clear** (M1's Tree-B files are now on trunk + merged into this lane). Daemon currently **FREE** (0 active/0 queued, PID 2120848); **no `phase-m1-a` branch exists yet, no open M1 PR.** **⚠️ BUT MEMORY.md HARD RULE (set by a prior session) says M1-a (Tree A bridge crate, Tasks 8–14) is the *user-chosen NEXT phase* and "M1 still owns the daemon until m1-a ships too."** → **NO-ELITEDESK may NOT be fully lifted.** This session's task said "Execute Phases 6–8"; the HARD RULE says close-out resumes *after* m1-a. **UNRESOLVED — surface to user next session (see §"⚠️ OPEN SEQUENCING QUESTION").** |
| Phases done | Phase 0 ✅ · harness ✅ · Phase 1 ✅ · Phase 2 ✅ · Phase 3 ✅ · Phase 5 ✅ · **post-M1 trunk integration ✅ (this session)** |
| In progress | — (clean handoff; HEAD `060bcac15`, working tree clean, pushed). |
| Next action | **FIRST: resolve the §"⚠️ OPEN SEQUENCING QUESTION" with the user** — is the next move (a) close-out Phases 6–8 now, or (b) hold for m1-a (Tree A) to ship first per the MEMORY.md HARD RULE? Trunk integration is DONE either way (do NOT re-merge/re-rebase — lane is current with post-M1 trunk at `f56039e62`). **IF Phases 6–8 cleared:** start Phase 6 per plan §"Phase 6": 🟦 Workflow plan-the-split (read-only line-range map of the 18,582-line `crates/server/tests/e2e.rs` per test domain) → 🟩 four-role (or laptop-local) behaviour-preserving execute. Then Phase 7 (type-state, 6 sites), Phase 8 (TODO sweep + capstone retro). Task list seeded (TaskList: #2/#3/#4 — unblocked, #1 done). |

**Targets verified present on the working branch (post-merge, 2026-06-04):** `crates/server/tests/e2e.rs` = **18,582 lines**; **6** `TODO(type-state)` sites (accept_jury_assignment.rs:110, admin_assign_jury.rs:122, admin_close_case.rs:65, admin_trigger_appeal_rejury.rs:68, sponsor_liability_grace.rs:135, submit_jury_vote.rs:271); **17** `TODO(brehon-fork)` markers; M1's `messaging_config.rs` + `governance_messaging_config.rs` merged in cleanly.
**Executable NOW (M1 unblocked):** Phases 6, 7, 8 + Phase 4-T1/T2 (deps-r2). NO-ELITEDESK lifted → four-role Junior OK; OR laptop-local per `project_laptop_canonical_cargo_runner.md` (cargo still laptop-only — that rule survives M1).
**Pending USER actions (not blocking, carried from prior session):** file Phase 3 PR1 (windows-signal) upstream + Docker-clippy for PR2; 3 retro promotion candidates in `session-retro-2026-06-04-closeout-phases-2-and-5.md`; rotate MiniMax key after m1-b trial (MEMORY.md pending obligation).

### ⚠️ OPEN SEQUENCING QUESTION (resolve with user before any Phase 6 work)

**The conflict:** this session was told *"M1 shipped. Execute: Phases 6–8."* But the **MEMORY.md HARD RULE** (`project_closeout_no_elitedesk_m1_active.md`, set by a prior session) says: *"phase-m1-b MERGED, BUT M1-a (Tree A bridge crate) is the user-chosen NEXT phase, so M1 still owns the daemon until m1-a ships too. Close-out resumes after m1-a."*

**Verified facts (2026-06-04):**
- `phase-m1-b` (M1 Tree B, Tasks 0–7) **IS merged** to `governance-v0` (`d6d027794`). The plan's literal M1-isolation gate keyed on `phase-m1-b` → satisfied.
- **No `phase-m1-a` branch exists on origin; no open M1 PR.** m1-a is *planned, not started*. Daemon is free *right now*.
- M1 was deliberately **split** (DQ `a192dbab1de8-001`, user gate): m1-b first (shipped), then m1-a (Tree A `services/bridge/` daemon + Tree C docs, Tasks 8–14).

**Why it matters:** m1-a (Tree A) is workspace-*excluded* (`services/bridge/`), so it likely does NOT touch the Phase 6/7/8 files (`crates/server/tests/e2e.rs`, `crates/api/api/src/governance/`). **IF that holds, Phases 6–8 are code-conflict-clear regardless of m1-a** and the "execute Phases 6–8 now" instruction is safe. The only real contention would be **daemon ownership** if m1-a dispatches Junior while close-out also wants four-role — but that's avoidable (run close-out cargo laptop-local; or sequence). **The trunk integration done this session is correct and needed either way.**

**Two clean resolutions for the next session to put to the user:**
- **(a) Proceed with Phases 6–8 now** (matches this session's explicit task). Confirm m1-a's file-set is disjoint from Phase 6/7/8 targets (`services/bridge/` only) — if disjoint, no code-conflict; run cargo laptop-local to avoid daemon contention with any m1-a dispatch. Update the MEMORY.md HARD RULE to "lifted for Phases 6–8 (m1-a is workspace-disjoint)."
- **(b) Hold Phases 6–8 until m1-a ships** (matches the MEMORY.md HARD RULE). Do m1-a next (separate effort, not this plan), then return to close-out. Pre-M1-a close-out is essentially done (Phases 0/1/2/3/5 + integration).

**Do NOT auto-pick** (per `feedback_advisor_instruction_mismatch_stop_and_ask.md` — explicit instruction conflicts a recorded rule). Trunk integration is committed/pushed; nothing is lost by pausing here.

### 2026-06-04 (cont. 5) — post-M1 trunk integration (M1-b shipped; merge not rebase)

- **M1-b GATE FLIPPED — M1 Tree B SHIPPED.** PR #177 (`phase-m1-b`) merged into `governance-v0` at `d6d027794` (`99ad26dc0` = bm-merge-complete tip). `git log origin/phase-m1-b ^origin/governance-v0` empty. Daemon free (`daemon_status`: 0 active/0 queued). The plan's literal M1-isolation gate (keyed on `phase-m1-b`) is satisfied → Phases 6–8 are **code-conflict-clear**. **⚠️ Whether NO-ELITEDESK is fully lifted is OPEN** — m1-a (Tree A) is the user-chosen next phase per the MEMORY.md HARD RULE; see §"⚠️ OPEN SEQUENCING QUESTION" above. (m1-a not started: no branch, no PR.)
- **Integration = MERGE, not rebase (user decision via AskUserQuestion).** The plan said "rebase onto post-M1 trunk," but all 19 closeout commits are **meta-only** (`.claude/`+`docs/`, zero `crates/` — the direct-commit-on-gov-v0 file set, never a PR). Rebase would rewrite pushed SHAs + force-push (against `no-destructive-defaults.md`). **Chose `git merge origin/governance-v0` → `phase-v1-closeout`** (commit `060bcac15`): resolves the collision once, preserves both histories, no force-push. Lane now 0 behind / 20 ahead origin/gov-v0, pushed.
- **Cross-session collision resolved (canonical `brehon-fork` M1-advisor session authored overlapping meta-files same day).** 5 files overlapped; 2 identical (no conflict: `feedback_daemon_telegram_completion_hook.md`, `v1-closeout-lane-setup-report.md`); **3 genuinely conflicted**:
  - **2 lessons** (`feedback_advisor_watchpoint_specificity.md`, `feedback_verify_automated_reviewer_claims_against_compiler.md`) — both sessions **independently reconstructed the same broken-citation lessons** on 2026-06-04. **Resolution (user): took gov-v0's versions** (`--theirs`; richer — they carry worked bad→good examples + the PR#132 `.get(0)`→`.first()` Diesel incident). Closeout's shorter parallel reconstructions dropped. Verified staged blobs == gov-v0 blobs.
  - **`decision-queue.json`** — gov-v0 had **282** resolved (M1's full history + un-archived legacy int-ids 225–282); closeout had **167** (it archived the legacy ints in Phase 1) + **1 unique** entry `466deb2e9332-001` (Phase-5 wasmtime keep-deferred log). **Resolution: UNION** — gov-v0 as authoritative base (proper schema-v3 shape) + the 1 closeout-unique entry = **283 resolved, 0 pending, 0 duplicate ids**, valid JSON (written `ensure_ascii=False`). The 1 common id with substantive diff (`a192dbab1de8-001`, the M1 split decision) → took gov-v0's (trunk-authoritative, well-formed).
- **Post-merge sanity GREEN:** working tree clean, no conflict markers, M1's messaging crates present, all Phase 6/7/8 targets confirmed on the working branch (counts above).
- **⚠️ Cargo is STILL laptop-only** even with NO-ELITEDESK lifted — `project_laptop_canonical_cargo_runner.md` is independent of the M1-daemon-block and survives M1 ship. Phase 6/7 cargo+e2e runs on the laptop (64 GB); Junior may orchestrate but workers write `validate-pending-laptop[-e2e]` + STOP.
- **NEXT SESSION:** (1) **FIRST resolve the §"⚠️ OPEN SEQUENCING QUESTION"** with the user — Phases 6–8 now vs hold for m1-a. (2) Integration is DONE — **do NOT re-merge/re-rebase** (lane current at `f56039e62`). (3) IF Phases 6–8 cleared: re-verify gate empty as a 5-sec sanity (`git log origin/phase-m1-b ^origin/governance-v0`), confirm m1-a file-set disjoint from Phase 6/7/8 targets, then launch the Phase-6 🟦 plan-the-split workflow (read-only line-range domain map of e2e.rs) per plan §"Phase 6". Task list (#2/#3/#4) seeded + unblocked.

### 2026-06-04 (cont. 4) — session `f2240f74` (Phases 2+5 shipped; `/auto-phase` correctly refused)

- **`/auto-phase v1-closeout` was invoked but does NOT apply** — it hard-refuses off `governance-v0` (we're on `phase-v1-closeout`) AND only orchestrates the four-role Junior pipeline (🟩), which is M1-gated under NO-ELITEDESK. The prior session's "next session runs /auto-phase" was a reasonable-but-wrong shorthand. **Resolution (surfaced to user, chosen):** drive the 🟦 lanes via the **`Workflow` tool** per the plan's own execution model (line 103). This is the close-out's primary engine while M1 holds the daemon. **Correction for future sessions: do NOT re-attempt `/auto-phase` for this lane** — the snapshot's Next-action now says so.
- **Phase 2 (doc-drift) SHIPPED** — commit `bb8a92ddd`. Workflow `wdg2fylk0` (5 agents, ~234K tok, 3.5 min): scan + adversarial-verify per claim. Results: (1) SL ENTRY_KIND markers — flipped 5 `(pending)`→`(active)` across 4 v1-SL-a rows (all emit sites verified shipped; RESTORATION_COMPLETED correctly left pending); (2) RT-r2 retro path — **plan said "annotate", reality was "repoint"** (dedicated `v1-RT-r2-retro.md` exists → roadmap line 235 repointed off the deps-r1 placeholder); (3) 04-data-model schema — **NO drift** (24 tables + 30 migrations verified matching source). Ground-truth-gather-before-fanout caught (2)+(3) before the workflow ran.
- **DQ archive — verified ALREADY COMPLETE** (no commit). Live 311 KB / 166 resolved / 0 pending. `dq-archive.sh --dry-run` showed **0 eligible** (30 remaining legacy int-ids all cited→kept-live; 136 v3 composite ids structurally never-eligible, incl. live M1 entries). The plan's "545 KB / 273" snapshot was pre-the-cont.2-archive. 200 KB trigger is structurally unmeetable (v3 entries alone ~215 KB) — resolves naturally when v1→M1 closes.
- **Phase 5 (wasmtime deps-r3) SHIPPED** — commit `cfcb49175`. User decision (ADR-012 gate): **KEEP DEFERRED**. Verify caught **two stale plan premises**: (a) "13 aarch64-only criticals" → actually **12** alerts, only 2 criticals aarch64-only, rest medium/low incl. one x86_64; (b) "watch PR #847 (open)" → **#847 CLOSED/stale/wrong-direction**; extism 1.21.0 (our pin) already shipped wasmtime-41 via #897; no live upstream PR to ≥42. Deliverables: decision record `v1-deps-r3-wasmtime-deferral-decision.md`, `kind:log` DQ `466deb2e9332-001`, durable release-watch (`watch_extism_wasmtime_42_adopt.md` + MEMORY.md — NOT a `/schedule` cron, which auto-expires at 7d vs the months-long horizon).
- **Phase 4-T4 DONE** + **wasmtime Dependabot dismissals DONE** — commit `e719081b2`. All 12 wasmtime alerts (#37–#47, #60) dismissed `tolerable_risk` via `gh api`. Live: **18 → 7 open** (0 wasmtime). **Phase 4-T3 premise was inverted** — `astral-tokio-tar 0.6.0` is NOT the fix release (0.6.1/0.6.2 are; we pin vulnerable 0.6.0), but it's **dev-only** (via `testcontainers`) so exposure is nil; **user chose to skip dismissal + bump post-M1** over dismissing-as-dev-only. The 7 still-open (astral-tokio-tar ×3 + rustls-webpki ×3 + idna ×1) are deliberate: tar = post-M1 bump; webpki+idna = Phase 4-T1/T2 *fixes*, M1-gated (dismissing the rustls-webpki HIGH #55 would hide a real fixable alert). Reusable fact: `gh api dismissed_comment` caps at **280 chars**.
- **Session retro** — `session-retro-2026-06-04-closeout-phases-2-and-5.md` (commit `fbfb448ca`). Dominant finding (5× this session): plan-stated EXTERNAL facts (alert counts, upstream PR/release state, file existence) are hypotheses — verify live before acting. Augments PMD lesson id 199 (eval id 785). 3 promotion candidates await user approval.
- **`.claude/workflows/` now tracked** — committed `phase2-doc-drift.js` as a reusable scan→adversarial-verify harness (Phase 8 governance-TODO sweep can reuse the pattern); resolves retro change #4.
- **M1 gate re-check: +39** (was +29 session-start; M1 advanced ~10 commits, still mid-flight, no trunk merge). Gate CLOSED, NO-ELITEDESK live.
- **Next session:** re-check M1 gate first. Pre-M1 close-out is essentially DONE (Phases 0/1/2/3/5 + DQ archive). When gate flips → rebase onto post-M1 trunk, then Phases 6/7/8 + 4-T1/T2.

### 2026-06-04 (cont. 3) — session `d3e7ac99` (handoff to /auto-phase; M1 MERGE in progress)

- **Decision (user):** stop driving phases manually; **next session runs `/auto-phase`** to execute the 🟦 workflow lanes (Phase 2 doc-drift, and post-M1 Phases 6/8) under the proper machinery. This session's manual runs covered Phases 0/1/3 + harness — clean handoff at HEAD `49c3e145c`, working tree clean.
- **Phase 2 NOT started** — interrupted at the first drift-verify step (SL ENTRY_KIND registry markers). Ground-truth for the 3 drift items (SL `(pending)`→`(active)` markers, RT-r2 retro path, 04-data-model schema verify) still to gather. `/auto-phase` picks this up.
- **⚠️ M1 gate STILL CLOSED at +29; M1 mid-Task-6 fix (NOT a trunk merge).** User clarified "it is merging" → actually **`lemmy_api_utils` compiling** — i.e. M1 is running a cargo validation of its Task-6 fix (the E0599 reqwest-middleware json-feature fail on `phase-m1-b` tip `ca61393af`), NOT merging to `governance-v0`. Verified on origin: `governance-v0` tip `8d3d2276b`, gate `phase-m1-b ^ gov-v0` = **+29** (unchanged), no open M1 PR. **Gate is closed; treat Phases 6–8 + Phase 4-T1/T2 as still BLOCKED.**
  - **⚠️ CARGO CONTENTION FLAG for the next session:** M1 is actively running cargo (`lemmy_api_utils` compiling) at handoff. If that compile is on the **laptop**, a fresh `/auto-phase` session must NOT start heavy local cargo concurrently (Phase 2 is doc-only so it's fine; but any validate-pending or harness re-run would contend). Check `Get-Process cargo,rustc` before launching cargo work.
  - **Next session re-check FIRST:** `git fetch origin --prune && git log origin/phase-m1-b ^origin/governance-v0 --oneline | wc -l` — **0 = M1 merged → gate flips**, Phases 6–8 + Phase 4-T1/T2 UNBLOCK, NO-ELITEDESK lifts. When it flips: (1) rebase close-out lane onto post-M1 `governance-v0` before any Phase 6 work (M1-isolation invariant + plan §99); (2) verify daemon free via `list_tasks` before lifting NO-ELITEDESK. Until 0, everything stays laptop-local and Phases 6–8 stay gated.

### 2026-06-04 (cont. 2) — session `d3e7ac99` (Phase 1 hygiene COMPLETE + Phase 3 drafts delivered)

- **Phase 3 carry-patch audit DONE.** Workflow `wym9sz191` (6 agents, ~531K tok) verified all 4 upstreamable candidates governance-free + still-needed-upstream. 2 PR drafts at `.claude/PRPs/reports/v1-closeout-phase3-carry-patch-audit.md`: **windows-signal = file-now** (clean, conf 0.90-0.95); **clippy-`#[expect]` = verify-first** (run the Docker `rust:1.95` clippy check before filing — the unstable lint may fire on upstream's CI). Key catch: fork working-tree diffs are NOT cherry-pickable (governance-coupled fixture arg + unrelated bundled edits) — the report's `## Changes` isolate clean hunks. **User action pending:** file PR1 upstream (advisor can't push to LemmyNet); in-tree `#___` backfill stays M1-gated.
- **Phase 1 hygiene COMPLETE** (all sub-parts):
  - **1a** — 6 orphaned worktree dirs removed (`rm -rf`, user-confirmed, re-inspected at delete-time). The re-inspect guard CAUGHT that `brehon-fork-scratch/sl-b/` was NOT empty (plan misdescribed it) — held 12 ephemeral SL-b files; user-confirmed separate delete. Only canonical + validate (M1) + closeout remain.
  - **DQ archive** — `dq-archive.sh` (lane `BREHON_REPO=$(pwd)`, cutoff 340, slug `v1-closeout-legacy-int-ids`): 545,916 → 311,894 B (−43%), 273 → 166 resolved. 107 legacy integer-id entries archived; citation guard kept 30 cited-by-rule entries LIVE; 136 composite v3 entries stay live by policy. Commit `865642e8e`.
  - **1b** — 46 shipped-sub-phase plans `git mv`'d to `completed/`. Active plans **55 → 9** (M1 + close-out + 7 cited-by-live-docs). All quality-r* verified done; deps-r2/r3 have no plan file. Commit `c715d6e7d`.
  - **1c** — MEMORY.md milestone-prune (user-gated `memory-prune` skill). 25,119 → **24,366 B** (under the 24.4 KB budget) / 203 → **199 lines** (under 200). 7 shorten-in-place edits + CI-section consolidation; same-codebase caveat honored (all Cargo/Rust/Windows/daemon/git lessons KEPT, zero content lost, 160 links intact). *(Outside repo — no commit.)*
  - **1d** — 2 missing always-load-rule citations reconstructed (`feedback_advisor_watchpoint_specificity.md`, `feedback_verify_automated_reviewer_claims_against_compiler.md`); Step-7 sweep now **0 TIER-1 broken citations**. Commit `b60d99a3a`.
- **Skill-worth flags** appended to lane-setup report (user request): memory-prune + dq-archive.sh keep-as-is; candidate `carry-patch-upstream-audit` skill (defer to 2nd rebase sweep); citation-integrity GAP (fold into memory-prune Step 3.5); exit-code traps = discipline miss not missing skill. Commit `d7df62046`.
- **M1 gate re-verified: now +25** (was +22; tip `dc5b93ba8`, M1 advancing). Gate firmly closed, NO-ELITEDESK live.
- **Phases now DONE:** 0 (bootstrap), harness sign-off, **1 (all hygiene)**, **3 (carry-patch audit + drafts)**.
- **Executable-now remaining:** Phase 2 (doc-drift reconcile), Phase 4-T3/T4 (laptop-local), Phase 5 (wasmtime decision). **Still M1-gated:** Phase 4-T1/T2, Phases 6–8.

### 2026-06-04 (cont.) — session `d3e7ac99` (harness sign-off + Phase 3 launch)

- **Harness signed off — all 4 probes green.** Probe 3 (full e2e `--no-run` compile) finished exit 0 in 26m47s (caught by a still-armed Monitor across the `/compact`). Probe 4 (negative test, `--features nonexistent_xyz`) ran directly → **exit 101**, confirming the `cargo-test.bat` shim propagates non-zero exit codes (the whole point of P4). *Two exit-capture traps hit + avoided en route:* `| tail` masked the wrapper exit (`feedback_pipes_mask_exit_codes.md`), and `& echo %errorlevel%` evaluated before the backgrounded cmd finished — the authoritative read is a bare, unpiped, non-backgrounded invocation. Flag `.claude/audit-phase-v1-closeout-complete.flag` set (gitignored runtime artifact).
- **M1 gate re-verified** at session resume: still **+22**, tip `dc5b93ba8` unchanged → gate firmly closed, NO-ELITEDESK still live.
- **Phase 3 (carry-patch audit) STARTED** — the plan's designated first 🟦 cost-gauge workflow. Launched `wym9sz191` (laptop harness, daemon-free, M1-safe): 4 candidate-verifier agents (one per upstreamable hunk: 2× Windows-signal in `server/src/lib.rs:55,284`, 2× clippy-`#[expect]` removal in `pagination.rs:236` + `vote/impls.rs:130`) → 2 PR-draft agents (Windows-signal bundle + clippy-expect bundle).
- **Marker-count drift caught** (`feedback_runbook_audit_drift_post_event_check.md`): the plan's "11 `TODO(brehon-fork)` markers" is now **17 in-tree** (`grep -rn "TODO(brehon-fork)" crates/`). The extra 6 are all `membership_state`/Phase-5a-task-51 governance-coupled (fork-local-forever, the ❌ class the plan named conceptually as "×7") **plus** 2 governance-internal TODOs the plan didn't list (`admin_emergency_remove.rs:74,120` "wire to canonical remove helper", `admin_reputation_stats.rs:12` "audit-log admin queries") — neither is an upstreaming candidate. **The upstreamable set is unchanged: 4 markers → 2 PRs.** The post-M1 annotation step's in-tree count is 17, not 11.
- **Pre-launch ground truth gathered** (read-only, daemon-free): `upstream` remote confirmed (`LemmyNet/lemmy`); fetched `upstream/main` (tip `159911a37`); **verified upstream/main STILL has bare `tokio::signal::unix` with no `#[cfg(not(windows))]` guard** (lines 56/285/286/289) → the Windows-signal carry-patch is genuinely still-needed upstream, not already-fixed. Toolchain pinned `1.95` (the clippy-`#[expect]` candidates are tied to this → the workflow flags the upstream-toolchain-mismatch caveat).

### 2026-06-04 — session `d3e7ac99` (harness verification + lane-setup audit)

- **Harness verified GREEN before any plan work** (per `pre-phase-harness-audit.md` + user instruction "first ensure the harness system is fully working"). Bootstrap artifacts (submodules, `.mcp.json`, `.env`, `settings.local.json`), PMD HTTP daemon (hybrid search round-trips), Junior daemon healthy, all SessionStart hooks wired. Cargo probes 1+2 pass; 3+4 in flight at session close.
- **GAP 1 fixed — Telegram completion hook was broken.** `list_hooks` showed hook ID 1 `active` but with `LAST ERROR: BuildMessage: ModuleNotFound`, never triggering (daemon restart May 29 left a stale check_fn). Removed ID 1 → recreated as **ID 2** (`junior-task-terminal-notify`), clean. *(Read-only-equivalent daemon op — no task dispatch; honors NO-ELITEDESK.)*
- **GAP 2 fixed — missing lesson reconstructed.** `feedback_daemon_telegram_completion_hook.md` was cited by always-load `advisor-orchestrator.md` §1 + 4 handovers but absent on disk. Authored it (3-state decision table, recreate-via-natural-language recipe, hook-ID-not-stable note, junior.db-vs-MCP-store topology caveat).
- **NO-ELITEDESK directive captured** (user, 2026-06-04). Recorded in PMD (`project_closeout_no_elitedesk_m1_active.md`) + MEMORY.md HARD RULE + folded into Phase 4 / cross-cutting guardrails of this plan (commit `1c7084622`).
- **Lane-setup report authored** — `.claude/PRPs/reports/v1-closeout-lane-setup-report.md` (forward-looking learnings for future isolated-lane bootstraps; TL;DR table, the 2 gaps, daemon-topology finding, cargo-probe table, hygiene baseline, improvement checklist).
- **Hygiene debt quantified** for Phase 1: DQ **545 KB / 273 resolved** (over both archive triggers), **55 active plan files**, 8 worktree dirs.
- **Plan validated self-consistent** with the live NO-ELITEDESK directive (Phase 4 lines 151–159 + guardrail line 241 already encode it). Phase 4 is *partially* executable: T3/T4 are laptop-local/daemon-free; only T1/T2 are gated.
- **Deferred (not done):** MEMORY.md is at 203 lines (+1 from the HARD-RULE add) — a proper prune is **Phase 1c's job** (milestone-prune skill, user-gated), not an ad-hoc trim. Flagged as hygiene item in the setup report.

---

## Context — why this plan exists

v1 has shipped. Every lane in `.claude/PRPs/v1-roadmap.json` is `done` — JM, SL, AD, RT (r1–r5), SR (ship-1→3), federation-inbound (a–e), redaction-r1, quality (r1→r3c), plus the meta-lanes (validate-agent, rls-r1, dq-schema-r1). All 11 v0 endpoints are implemented and e2e-tested. The codebase is now a mature Lemmy fork (`governance-v0`) carrying a 91-file governance layer woven across 8 crate sub-trees.

Shipping at this pace left three classes of accumulated debt that a deliberate close-out should retire **before** the platform takes on the next milestone's weight:

1. **Code-side debt** — an 18,526-line `crates/server/tests/e2e.rs` monolith (a repeated Junior Edit-hang hazard), 6 `TODO(type-state)` retrofit sites in `governance/`, 38 governance `TODO/FIXME` markers.
2. **Repo & meta hygiene** — 6 orphaned worktree directories on disk, 54 active plan files (most for shipped sub-phases), 217 reports, a 194-lesson corpus, and roadmap/registry documentation drift.
3. **Deferred security/deps** — `v1-deps-r2` (6 Dependabot alerts) and `v1-deps-r3` (13 wasmtime alerts via extism), both explicitly parked "post-v1-ship."
4. **Carry-patch upstreaming** — 11 `TODO(brehon-fork)` markers; 4 are generic, upstreamable Lemmy fixes that could be contributed and then deleted from the tree.

**The hard constraint:** the next milestone, **M1 (chat infrastructure / "v2")**, is *in flight right now* in the `brehon-fork-validate` worktree (`phase-m1-b`, Tasks 0–7). M1 implements/edits `crates/server/tests/e2e.rs`, `crates/api/api/src/governance/{mod,messaging_config,admin_config}.rs`, and `crates/db_schema/{newtypes,…/governance/…,schema}`. **This overlaps the e2e-split and type-state-retrofit surfaces directly.** Per the user's decision, this plan **isolates in its own worktree and defers every `crates/` edit until M1 merges.** Meta/hygiene and deps/upstreaming *analysis* run now; `crates/` *refactoring* is staged behind the M1 landing.

**Intended outcome:** a clean v1 baseline — empty stale-worktree set, archived shipped plans, reconciled docs, a triaged+filed upstreaming set, a green security posture (or documented suppressions), and (post-M1) a decomposed e2e suite + type-state-hardened governance handlers — without ever racing the live M1 lane.

---

## Execution model — the hybrid (this is the core design)

Per the user: *not all tasks suit `/workflows`; the plan is hybrid.* The Workflow tool runs a background JS script that fans out subagents — its sweet spot is **parallel read/audit + adversarial verification** where intermediate results stay out of the main context. Its hard limits (confirmed against the Claude Code guide): the **script itself has no file/shell access** (only the subagents it spawns do), and a workflow **cannot pause for mid-run user input** — only permission prompts. So anything needing a **user gate mid-flight** (CR triage, merge confirm, ADR-risk calls) must live *outside* a workflow.

Every phase below is tagged with one **execution lane**:

| Lane | When it applies | Mechanism |
|---|---|---|
| 🟦 **workflow** | Parallel fan-out over many files/sites; audit + adversarial verify; read-heavy classification. No mid-run user gate. | `Workflow` tool (a JS script per phase). User opt-in already given ("optimise it for /workflows"); launch still shows the per-run approval prompt. |
| 🟩 **four-role** | `crates/` code changes that need plan→approval→Junior-impl→CR→merge gates. | Brehon `/auto-phase` pipeline (planning/impl-task/bm-task/ci-watcher) — the existing machinery. |
| 🟨 **manual/advisor** | Single careful sequential actions, inspect-then-confirm deletes, ADR-risk decisions, or launching/gating the other two lanes. | Advisor session directly (this session or a successor), with explicit user confirmation where flagged. |

**Why hybrid beats all-workflow:** the audits (which files, which sites, what state) are embarrassingly parallel and benefit from a workflow's fan-out + verify; the *mutations* of shipped governance code are exactly the high-stakes, gated, one-careful-change-at-a-time work the four-role pipeline was built for — and the destructive filesystem ops need a human's eyes before they run.

**Workflow cost gate (applies to every 🟦 phase):** the first workflow run in this close-out should be scoped small (e.g. the 11-file carry-patch audit, not a whole-repo sweep) to gauge token burn before the broader sweeps. Caps: 16 concurrent / 1000 total agents per run; the user can stop any run without losing completed work and approves each launch.

---

## Phase map (by conflict-risk, not by topic)

```
NOW (zero M1 conflict — no crates/ mutation)
├─ Phase 0  Bootstrap the isolated close-out worktree          🟨 manual/advisor
├─ Phase 1  Repo & meta hygiene                                🟦 workflow (audit) → 🟨 (apply)
├─ Phase 2  Documentation-drift reconcile                      🟦 workflow (scan) → 🟨 (apply)
├─ Phase 3  Carry-patch upstreaming triage + filing            🟦 workflow (audit) → 🟨 (PRs)
├─ Phase 4  Deferred security — deps-r2                         🟩 four-role (gated)
└─ Phase 5  Deferred security — deps-r3 (wasmtime) DECISION     🟨 manual/advisor (ADR-risk gate)

GATED ON M1 MERGE (crates/ mutation — must not race phase-m1-b)
├─ Phase 6  e2e.rs decomposition                               🟦 workflow (plan-the-split) → 🟩 four-role (execute)
├─ Phase 7  Type-state retrofit (6 sites)                      🟩 four-role (gated)
└─ Phase 8  Governance TODO/FIXME sweep + close-out retro      🟦 workflow (audit) → 🟩/🟨 (apply)
```

**Sequencing rule:** Phases 0–5 may proceed immediately in the close-out worktree. Phases 6–8 are *authored* now (plans/briefs drafted) but **blocked** — their impl tasks do not dispatch until `git log origin/governance-v0 ^phase-m1-b` confirms M1 has merged to trunk. The close-out worktree rebases onto post-M1 `governance-v0` before Phase 6 begins.

---

## Phase 0 — Bootstrap the isolated close-out worktree  🟨 manual/advisor

Goal: a dedicated lane so close-out work never shares a working tree or DQ with the live M1 lane (per `multi-lane-worktree.md`).

1. From canonical `brehon-fork`: `git worktree add ../brehon-fork-closeout governance-v0` → branch `phase-v1-closeout` (cut via `bm-cut` brief, or directly for the meta-only early phases).
2. Run the **lane bootstrap checklist** (`feedback_phase_lane_worktree_bootstrap_checklist.md`): `git submodule update --init --recursive`; copy `.mcp.json`, `.env`, `.claude/settings.local.json`; verify the canonical-PMD path guard.
3. Open a new CC session with CWD = `brehon-fork-closeout`. This session drives Phases 1–5. Document **lane mode A** in a bootstrap handover at `.claude/PRPs/handovers/v1-closeout-bootstrap.md`.
4. Confirm no DQ cross-contamination: the close-out lane writes its own `.claude/decision-queue.json`; canonical stays on `governance-v0` meta-edits only.

**Output:** `brehon-fork-closeout/` worktree on `phase-v1-closeout`, bootstrapped, session open.
**User confirm:** none beyond the standard worktree-create (this is a local, non-destructive add).

---

## Phase 1 — Repo & meta hygiene  🟦 workflow (audit) → 🟨 (apply)

Goal: empty the stale-worktree set, archive shipped plans, and right-size the meta-corpus. **No `crates/` touch — zero M1 conflict.**

### 1a. Stale worktree teardown  🟨 manual/advisor (inspect-then-confirm)
Six orphaned dirs on disk, **all with no `.git`** (already-removed worktrees whose dirs survived) and **all empty or near-empty** (verified: 0–1 entries, 8–64 KB; `brehon-fork-scratch/` holds one empty `sl-b/`):
`brehon-fork-deps-r1`, `brehon-fork-phase-v1-AD-a`, `brehon-fork-redaction-r1`, `brehon-fork-scratch`, `brehon-fork-ship-1`, `brehon-fork-ship-2`.

- `git worktree prune` is already clean (git knows they're gone) — so these are pure filesystem cleanup, NOT `git worktree remove`.
- **Per `no-destructive-defaults.md`:** re-inspect each (`ls -A`) at run time, confirm still empty, then `rm -rf` **one at a time with the dir name echoed**. This is the one destructive step — it requires explicit user go-ahead listing the six paths.
- Also reconcile any *registered-but-stale* worktrees the roadmap notes (`brehon-fork-rt-r3/r4/r5`, `quality-r1`, `fed-in-d`) — these may still be registered; use `git worktree remove` (NOT `rm -rf`) for those, `--force` only if a submodule worktree blocks (`feedback_worktree_remove_force_for_submodules.md`).

### 1b. Plan-file archival  🟦 workflow → 🟨 apply
54 active plan files in `.claude/PRPs/plans/`; only 7 in `completed/`. Most active files are for `done` sub-phases.
- **Workflow:** fan out a classifier — one agent per plan file (or batched) cross-references the filename against `v1-roadmap.json` lane status and returns `{file, sub_phase, status, safe_to_archive}`. A completeness-critic agent verifies no plan cited by an *active* rule/brief is moved (grep each candidate's basename across `.claude/{rules,commands,PRPs/briefs,lessons}`). Output: a structured archive manifest.
- **Apply (🟨):** `git mv` the confirmed-done plans into `.claude/PRPs/plans/completed/`; commit `chore(plans): archive N shipped sub-phase plans`. M1 plans (`m1*.plan.md`) stay active.

### 1c. MEMORY.md milestone-prune  🟨 manual/advisor (skill)
MEMORY.md is over budget (202 lines, limit 200; SessionStart WARN fires). A v1→M1 milestone boundary is the highest-yield prune trigger.
- Invoke the **`memory-prune`** skill (Step 2.5 milestone sweep). Surfaces findings → user confirms → applies. Retires prior-milestone phase-texture (the archived-2026-05-31 class, shipped-fix cards) in one pass.

### 1d. Lesson-corpus sweep (optional, advisory)  🟦 workflow
194 lessons in `.claude/lessons/`. Not pruning code lessons (they're load-bearing), but a workflow can surface **superseded/duplicate** pairs (e.g. the historically-cited-but-never-authored slugs the rules already note) for the user to decide. Read-only audit → report; no auto-delete.

**SEED — broken citations in ALWAYS-LOAD rules (verified 2026-06-04, via the `new-lane` skill's Step-7 citation-integrity sweep).** This is the concrete starting point for the lesson-corpus audit — distinct from dedup. The sweep (`grep -rhoE '(feedback|reference)_[a-z0-9_]+\.md' .claude/rules/ | sort -u | while read f; do [ -f ".claude/lessons/$f" ] || echo "$f"; done`) found **3 always-load-rule citations resolving to files that do NOT exist in `.claude/lessons/`** — every session inherits these broken pointers (the GAP-2 class from `v1-closeout-lane-setup-report.md`, now generalised). All three are cited by the always-load rule `.claude/rules/advisor-orchestrator.md`:

| Missing lesson | Cited at (always-load) | Status / action |
|---|---|---|
| `feedback_advisor_watchpoint_specificity.md` | `advisor-orchestrator.md` §3.5 (watchpoint-specificity gate — load-bearing plan-approval gate) | **Author it** — the gate's behaviour is documented inline in §3.5 + indexed in MEMORY.md; reconstruct the lesson from those + git history of watchpoint DQs. |
| `feedback_verify_automated_reviewer_claims_against_compiler.md` | `advisor-orchestrator.md` §5.4 (falsifiable-hypothesis CR-finding variant — compile-check before triaging a CR trait/type claim) | **Author it** — content sketched inline in §5.4 + the PR#132 `.get(0)`→`.first()` Diesel incident; reconstruct from there. |
| `feedback_daemon_telegram_completion_hook.md` | `advisor-orchestrator.md` §1 (Telegram completion hook check) | **Likely already resolved** — reconstructed as a draft this session (GAP 2 fix in the close-out worktree); confirm it's committed to `governance-v0`, else author from the draft. |

Remediation is **authoring 2–3 short lessons** (≤ TIER-2 risk, `.claude/lessons/` only — meta-work, direct-commit per `phase-branch.md`), NOT a workflow fan-out — but the Phase-1d workflow should *also* re-run the Step-7 sweep at completion to confirm zero TIER-1 breaks remain, and surface the ~14 TIER-2 (handover) breaks as a separate lower-priority list. The `new-lane` skill Step 7 splits TIER-1/TIER-2 and carries this ~3+14 baseline so future lanes flag only *new* drift.

**Outputs:** empty stale-worktree set; shipped plans archived; MEMORY.md under budget; (optional) lesson-dedup report; **3 TIER-1 broken always-load-rule citations resolved (lessons authored); Step-7 sweep re-run shows 0 TIER-1 breaks.**

---

## Phase 2 — Documentation-drift reconcile  🟦 workflow (scan) → 🟨 (apply)

Goal: flip the known doc-drift the roadmap itself flags, so the design docs match the shipped code. **No `crates/` logic change** (registry/comment edits only).

Known drift (from `v1-roadmap.json` → `what_remains.documentation_drift_followups` + lane `known_followups`):
- **SL ENTRY_KIND markers** still say `(pending)` in `.claude/rules/governance-log-entry-kind-registry.md` for `SPONSOR_LIABILITY_PENDING/FIRED/ESCAPED` + `ENDORSEMENT_REVOKED` — code emits them correctly; flip to `(active)` mirroring JM-e's line-140 pattern.
- **RT-r2 retro path** in the roadmap points at the deps-r1 retro (placeholder) — correct or annotate.
- **04-data-model-and-api.md** (the LIVING schema doc) — verify it reflects all merged v1 schema (it's code-derived; a workflow can diff doc claims vs `schema.rs`).

- **Workflow:** fan out doc-vs-code verifiers — each takes one drift claim, greps the code for ground truth, returns `{claim, doc_says, code_says, fix}`. Adversarial verify (a second agent compile/grep-checks each proposed fix before it's trusted — per `feedback_verify_automated_reviewer_claims_against_compiler.md`).
- **Apply (🟨):** registry/doc edits are `.claude/` + `docs/` only → direct-commit on `phase-v1-closeout` (or canonical `governance-v0`) per `phase-branch.md` "direct" set. `docs(registry):` / `docs(rules):` subjects.

**Output:** registry markers flipped; roadmap placeholders corrected; LIVING schema doc verified current.

---

## Phase 3 — Carry-patch upstreaming triage + filing  🟦 workflow (audit) → 🟨 (PRs)

Goal: contribute the generic, non-governance carry-patches to upstream LemmyNet, then track their PR numbers in-tree per the existing convention (`feedback_carry_patch_todos.md`). **Analysis is read-only; PR filing is user-driven; the only in-tree edit is filling `#___` placeholders — deferred until M1 merges since 2 candidates live in `crates/`.**

Audit complete (this planning pass): 11 `TODO(brehon-fork)` markers → **4 upstreamable**, **7 fork-local-forever**.

| Candidate | File | What it does | Verdict |
|---|---|---|---|
| Windows signal handling ×2 | `crates/server/src/lib.rs:55,284` | `#[cfg(not(windows))]`-gates `tokio::signal::unix`; ctrl-c fallback on Windows | ✅ upstreamable (generic portability) |
| clippy `#[expect]` removal | `crates/diesel_utils/src/pagination.rs:236` | drops a no-longer-firing `multiple_bound_locations` expectation under `-D warnings` | ✅ upstreamable (toolchain drift) |
| clippy `#[expect]` removal | `crates/db_views/vote/src/impls.rs:130` | same `#[expect]` drift on `paginate_vote_response` | ✅ upstreamable (toolchain drift) |
| `membership_state` ×7 | `db_schema/{lib,source/person,impls/person}.rs`, `api_crud/user/create.rs`, `db_views/registration_applications/impls.rs` | adds/threads the governance `MembershipState` column | ❌ fork-local (governance-coupled) |

- **Workflow (audit refinement, optional):** the 4 upstreamable candidates can be re-verified in parallel — one agent per candidate confirms it's *genuinely* governance-free (no `membership_state` / governance-type leakage in the diff), drafts an upstream-PR description, and checks whether upstream Lemmy `main` has already fixed it. Output: 2 ready-to-file PR drafts (the 2 Windows signal hunks naturally bundle into one PR; the 2 clippy `#[expect]` removals into a second).
- **File PRs (🟨 user):** the user files the upstream PRs (against `LemmyNet/lemmy`, NOT the fork) using the drafts. I cannot push to upstream — I prepare the branches/diffs and PR bodies; the user submits.
- **Backfill `#___` (deferred behind M1):** once a PR number exists, fill it into the marker. The 2 Windows-signal markers are in `crates/server/src/lib.rs` and the 2 clippy ones in `crates/` — these are `crates/` edits → **wait for M1 merge** (Phase 6+ window) before committing the placeholder fills. The 7 fork-local markers get their TODO text updated to "fork-local (governance) — do not upstream" to stop re-triaging them every rebase (also a `crates/` edit, same deferral).

**Output now:** 2 upstream-PR drafts + branches ready for the user to file. **Output post-M1:** `#___` placeholders filled; fork-local markers annotated.

---

## Phase 4 — Deferred security: deps-r2  🟩 four-role (gated) — ⛔ BLOCKED: NO-ELITEDESK while M1 active

> **NO-ELITEDESK directive (user, 2026-06-04 — `project_closeout_no_elitedesk_m1_active.md`):** while the close-out lane runs, the EliteDesk daemon (`junior@brehon-fork`) is **owned by M1** (`phase-m1-b`) — **zero close-out Junior dispatch**. Phase 4 as specced is 🟩 four-role (Junior `bm-cut`/planning/impl-task/ci-watcher on the daemon) → **that is blocked.** Read-only daemon queries (`list_tasks`, `daemon_status`) are still fine. Holds until `phase-m1-b` merges to `governance-v0`.
>
> **When Phase 4 comes up, SURFACE to user — do NOT auto-pick** (per the directive):
> - **(a) Defer Phase 4 entirely** until M1 ships and the daemon frees (cleanest; security fixes land later). Phase 4 then runs normal four-role.
> - **(b) Run deps-r2 laptop-local** — advisor/subagent writes the Rust, laptop cargo+e2e validates (per `project_laptop_canonical_cargo_runner.md`), CR review on the PR, NO Junior dispatch. Honors no-EliteDesk but loses the four-role gating granularity.
>
> Either way, T2's `[patch.crates-io]` Cargo.toml edit also stays gated behind M1 merge (overlap risk, per the approval decision). T3 (Dependabot dispute, GitHub UI) and T4 (wasmtime log DQ) are laptop-local and unaffected by the daemon block — they can run anytime.

Goal: retire 6 of 18 open Dependabot alerts. Precondition `v1-ship-complete` is satisfied (v1 shipped). Fully scoped greenfield brief exists: `.claude/PRPs/briefs/v1-deps-r2-planning-1.md`. **This is a `crates/`-touching, cargo-gated lane** (needs CR review + merge confirm; touches `Cargo.toml`/lockfile/`crates/api/api_utils/src/utils.rs`). Originally specced 🟩 four-role, but the NO-ELITEDESK directive above blocks daemon dispatch — see the two options.

Four tasks (from the brief):
- **T1** — inline webmention replacement (~70 LOC reqwest impl in `crates/api/api_utils/src/utils.rs`), drops the rustls-webpki 0.101.7 chain → closes 3 alerts (1 High).
- **T2** — fork `rlidwka/mdurl.rs` → `barrie-cork/mdurl`, bump idna 0.3→1.1, add the first `[patch.crates-io]` entry to root `Cargo.toml` → closes 1 medium.
- **T3** — file a "dispute advisory" on `astral-tokio-tar 0.6.0` (it IS the fix release) → closes 2 via Dependabot dismissal (🟨 user action in the GitHub UI).
- **T4** — record the wasmtime deferral as a `kind: "log"` DQ entry + dismiss all 13 wasmtime alerts with documented reasons → hands off to Phase 5.

- **M1-overlap check before dispatch:** T1 touches `api_utils/src/utils.rs`; M1 touches `api_utils/src/{notify,plugins,bridge_notify,lib}.rs` — *different files in the same crate*. Low collision risk, but **confirm via `git diff` of M1's actual touch-set at dispatch time** and prefer to sequence Phase 4 *after* M1 merges if any doubt. The `[patch.crates-io]` addition to root `Cargo.toml` is the riskier overlap (M1 may edit Cargo.toml) → safest to gate T2 behind M1.
- **Watchpoints (from brief):** re-enumerate webmention callsites (`rg "webmention::|send_webmention" crates/` must = 6); re-verify the 18-alert total at lane-cut (Dependabot may have re-counted).
- **Pipeline (option a, post-M1 four-role):** `bm-cut` → planning (brief already exists; clarify pass) → impl cohort (T1, T2 serial — both edit build config) → laptop cargo+e2e validation → CR triage → merge confirm.
- **Pipeline (option b, laptop-local, no daemon):** advisor/subagent writes T1 Rust on `phase-v1-closeout` → laptop `cargo-check.bat`/`cargo-test.bat` validate → open PR → CR review → merge confirm. No `bm-cut`/Junior; T2 still gated behind M1.

**Output:** 6 Dependabot alerts retired; `[patch.crates-io]` established; wasmtime deferral logged.

---

## Phase 5 — Deferred security: deps-r3 (wasmtime) DECISION  🟨 manual/advisor (ADR-risk gate)

Goal: decide the wasmtime path. **No brief exists; high risk against ADR-012 (Extism plugin host).** This phase is a **user decision point, not an execution lane** — the 13 wasmtime alerts are aarch64-only criticals and Brehon deploys x86_64, so the real-world exposure is low and the right call may be "keep deferred."

The situation (verified): extism 1.21.0 pins wasmtime 41.0.4 via internal wiggle-macro APIs, so a bare `[patch.crates-io]` override won't compile — closing these alerts needs **forking extism** (1–3 days, high ADR-012 risk) OR adopting an upstream extism release (tracking `extism/extism` PR #847, open since 2026-04-07).

**Surface to user — three options:**
1. **Keep deferred** (recommended default) — document the x86_64-not-vulnerable rationale, suppress all 13 in Dependabot, revisit when upstream extism ships wasmtime ≥42. (Already the roadmap's `deferred_until` posture.)
2. **Fork-and-patch now** — author a deps-r3 brief, fork extism, four-role pipeline it. Only if the user wants a fully-green security tab before pilot.
3. **Wait-and-watch** — set a `/schedule` check on extism PR #847 so we adopt the upstream release the moment it lands.

This phase produces a **decision + (if option 1/3) documented suppressions**, not code, unless the user picks option 2.

---

## Phase 6 — e2e.rs decomposition  🟦 workflow (plan-the-split) → 🟩 four-role (execute) — **GATED ON M1 MERGE**

Goal: break the 18,526-line `crates/server/tests/e2e.rs` monolith into per-domain modules. **This is the single biggest code-side refactor and the one with the hardest M1 conflict** — M1 actively edits `e2e.rs` (adds `m1_*_fixtures` modules). **Does not start until M1 has merged to `governance-v0` and the close-out worktree has rebased onto it.**

Why it matters: the monolith is a documented, recurring Junior Edit-hang hazard (`feedback_junior_worker_e2e_edit_hang.md`, `feedback_fix_impl_pre_locate_e2e_anchors.md`) — every phase that edits it pays an anchor-collision + hang tax. Decomposing it removes that tax for all future milestones (M2, M3).

- **Workflow (plan-the-split, read-only):** fan out analyzers — each maps one logical test domain (jury, sponsor-liability, appeal, reputation, federation-inbound, admin, redaction, ship, m1-messaging) to its line ranges and its shared-fixture dependencies. A synthesis agent proposes a module boundary set (`tests/e2e/{jury,sponsor,appeal,…}.rs` + a shared `common/fixtures.rs`) and flags every cross-domain helper that must move to `common`. Output: a precise, line-ranged split manifest. **This is exactly the fan-out-then-synthesize shape workflows excel at**, and it keeps the 18K-line file out of the main context.
- **Execute (🟩 four-role):** the split itself is a careful, compile-gated, test-must-still-pass refactor → four-role pipeline. Critical guardrail: **behaviour-preserving only** — same tests, same assertions, just relocated; the full e2e suite must show identical pass-count before/after (`feedback_emit_added_requires_full_e2e_gate.md`). Likely several sub-phases (one module-group per sub-phase) to keep each diff reviewable and each cargo run green.
- **Risk:** Rust test-module visibility + shared-fixture re-exports (`feedback_rust_visibility_cross_crate.md`); `mod` declarations and `#[path]` if needed. The workflow's split manifest de-risks this by enumerating every move up front.

**Output:** `e2e.rs` decomposed into per-domain modules; identical test coverage; future e2e edits no longer hang Junior.

---

## Phase 7 — Type-state retrofit (6 sites)  🟩 four-role (gated) — **GATED ON M1 MERGE**

Goal: replace ad-hoc `case.status` match-guards with the `GovernanceCase<S>` phantom type-state wrapper at the 6 annotated sites. **Deferred behind M1** — the sites live in `crates/api/api/src/governance/` alongside M1's `mod.rs`/`admin_config.rs`/`messaging_config.rs` edits; sharing the directory + `mod.rs` risks merge conflict.

The 6 `TODO(type-state)` sites (canonical pattern: `feedback_governance_type_state_handlers.md` — `GovernanceCase<S>` + `TryFrom<ModerationCase>` centralising the `CaseStatus` guard):
- `accept_jury_assignment.rs:110` — dual-role match → `GovernanceCase<JurySelection>`
- `admin_assign_jury.rs:122` → `GovernanceCase<PreJuryAssignable>`
- `admin_close_case.rs:65` — inverted guard (reject Closed only)
- `admin_trigger_appeal_rejury.rs:68` — single-variant guard (cleanest candidate)
- `sponsor_liability_grace.rs:135` — filter-query pattern → exhaustive match
- `submit_jury_vote.rs:271` — terminal-state idempotency guard → `GovernanceCase<Active>`

- **Four-role:** each site is a focused, compile-gated handler change with e2e coverage → plan → impl cohort (sites are independent → `[P]` parallel-eligible, but they share `governance/mod.rs` for the wrapper definition, so Task 0 defines the type-state scaffold non-`[P]`, then the 6 retrofits fan out). CR + merge gates apply.
- **Not a workflow:** these are governance-semantic changes to shipped code with real behaviour-equivalence stakes — four-role gating, not background fan-out.

**Output:** 6 type-state TODOs resolved; `CaseStatus` guards centralised; the type-state pattern's retrofit debt cleared.

---

## Phase 8 — Governance TODO/FIXME sweep + close-out retro  🟦 workflow (audit) → 🟩/🟨 (apply) — **GATED ON M1 MERGE**

Goal: triage the remaining ~38 governance `TODO/FIXME/XXX/HACK` markers (the non-type-state, non-carry-patch residue) and write the v1 lane-meta retro.

- **Workflow (audit):** fan out — one agent per marker cluster reads the TODO + context and classifies `{marker, file, category(bug|defer-to-v2|stale|actionable), recommended_action}`. A completeness-critic confirms none is a latent correctness bug hiding as a TODO (silent-failure lens). Output: a triaged TODO ledger.
- **Apply:** actionable + low-risk → 🟩 four-role fix cohort (or 🟨 direct for comment-only). Defer-to-v2 → annotate + log. Stale → delete.
- **Close-out retro (🟨):** author the v1 capstone meta-retro (`.claude/PRPs/reports/v1-closeout-retro.md`) — the four-role retro signals across the whole v1 arc, the `retro-bypass.jsonl` rate trend, and what carries into M1+. Gate 6 (retro sign-off) → optional `/brehon-phase-transition` to mark v1 formally closed.

**Output:** governance TODO debt triaged + retired; v1 formally closed with a capstone retro.

---

## Cross-cutting guardrails (apply to every phase)

- **M1-isolation invariant (the prime directive):** no Phase touches a file in M1's implement-set (`e2e.rs`, `governance/{mod,messaging_config,admin_config}.rs`, `db_schema/{newtypes,schema, …/governance/…}`, `api_utils/{notify,plugins,bridge_notify,lib}.rs`, `Cargo.toml`) until `git log origin/governance-v0 ^phase-m1-b` is empty (M1 merged). Phases 0–5 are structured to respect this *by construction* (they touch `.claude/`, `docs/`, filesystem, and non-overlapping `crates/` only — with the T1/T2 caveat flagged in Phase 4).
- **NO-ELITEDESK invariant (2026-06-04 user directive — `project_closeout_no_elitedesk_m1_active.md`):** while close-out runs, M1 owns the EliteDesk daemon — **zero close-out Junior dispatch** (no `bm-cut`/planning/impl-task/ci-watcher tasks on `junior@brehon-fork`). Close-out is **laptop-local only**: 🟦 workflow phases run via the laptop `Workflow` tool (subagents in the laptop harness, not the daemon); 🟨 manual/advisor phases are laptop-local; **Phase 4 (the one 🟩 four-role phase) is BLOCKED → surface options to user** (defer vs laptop-local impl). Read-only daemon queries (`list_tasks`, `daemon_status`, `list_hooks`) are fine. Holds until `phase-m1-b` merges. Reinforces (does not replace) the laptop-canonical-cargo-runner rule.
- **Lane discipline:** all close-out work in `brehon-fork-closeout`; its own DQ; canonical `brehon-fork` does meta-edits only (`multi-lane-worktree.md`). One session, one CWD, one lane.
- **No-destructive-defaults:** the only `rm -rf` is Phase 1a (six verified-empty dirs) — inspect-then-confirm, one dir at a time, with explicit user go-ahead.
- **Workflow opt-in & cost:** user opt-in is given; each 🟦 launch still shows its approval prompt; first run scoped small to gauge burn; user can stop any run.
- **Direct-vs-PR:** meta/docs/registry edits commit direct on the branch per `phase-branch.md`; every `crates/` change goes through the PR flow with CodeRabbit.

---

## Verification — how we'll know each tier is done

**Hygiene tier (Phases 1–2):**
- `ls -d C:/Users/barri/Developer/brehon-fork-*` shows only canonical + `brehon-fork-validate` (M1) + `brehon-fork-closeout` (this lane).
- `ls .claude/PRPs/plans/*.plan.md | wc -l` drops to the active-only set (M1 + close-out); shipped plans in `completed/`.
- `wc -l MEMORY.md` ≤ 200; SessionStart WARN no longer fires.
- `grep "(pending)" .claude/rules/governance-log-entry-kind-registry.md` returns no SL markers that code actually emits.

**Security tier (Phases 4–5):**
- GitHub Dependabot alert count drops by 6 (deps-r2); the remaining 13 wasmtime alerts are either closed (option 2) or documented-suppressed (option 1/3).
- `grep -A2 "patch.crates-io" Cargo.toml` shows the mdurl fork pin (deps-r2 T2).
- `rg "webmention::" crates/` returns 0 external-crate references (T1 inlined); full e2e suite green via laptop cargo.

**Upstreaming tier (Phase 3):**
- 2 upstream PRs filed against `LemmyNet/lemmy` (user-submitted from the prepared drafts).
- Post-M1: `grep "TODO(brehon-fork)" crates/` markers carry either a real PR number or "fork-local — do not upstream"; no bare `#___` on the 4 upstreamed hunks.

**Code-refactor tier (Phases 6–8, post-M1):**
- `wc -l crates/server/tests/e2e.rs` drops dramatically; `ls crates/server/tests/e2e/` shows the per-domain modules; full e2e pass-count identical to pre-split (run via laptop `cargo-test.bat --workspace --test e2e --features full`).
- `grep "TODO(type-state)" crates/` returns 0.
- `grep -ic "TODO\|FIXME" governance code` materially reduced; triage ledger shows each marker resolved/deferred/deleted.
- v1 close-out retro on trunk; `/brehon-phase-transition` (optional) marks v1 closed.

---

## Open decisions — RESOLVED at approval (2026-06-04)

1. **Phase 4 sequencing** — ✅ RUN NOW, gate only T2. T1/T3/T4 proceed in the close-out lane immediately; T2's Cargo.toml `[patch.crates-io]` edit waits for M1 merge.
2. **Phase 5 wasmtime** — ✅ KEEP DEFERRED + document + suppress 13 alerts + `/schedule` watch on `extism/extism` PR #847.
3. **Promotion** — ✅ promoted to `.claude/PRPs/plans/v1-closeout.plan.md` at Phase 0.
4. **First workflow** — ✅ Phase 3 carry-patch audit is the cost-gauge run.

## Handoff model (2026-06-04)

This advisor session (canonical `brehon-fork`) **prepares** the close-out lane and hands off. The **user drives execution** in a separate, fully-harnessed `brehon-fork-closeout` session. Phase 0 deliverables from this session:
- Tracked plan committed to `governance-v0`.
- `brehon-fork-closeout` worktree created on `phase-v1-closeout`, **fully bootstrapped** (submodules + `.mcp.json` + `.env` + `.claude/settings.local.json` so PMD MCP, Junior MCP, and all SessionStart hooks are live — per `feedback_phase_lane_worktree_bootstrap_checklist.md` + `pmd-invariants.md` #1/#5).
- Cold-start bootstrap handover at `.claude/PRPs/handovers/v1-closeout-bootstrap.md` (committed) — the user's next session reads this with zero conversation context.
- **Isolation guarantee:** the close-out lane never shares a working tree, branch, or DQ with the live M1 lane (`brehon-fork-validate` / `phase-m1-b`). PMD is intentionally shared (HTTP daemon, cross-lane by design — not a race surface).
