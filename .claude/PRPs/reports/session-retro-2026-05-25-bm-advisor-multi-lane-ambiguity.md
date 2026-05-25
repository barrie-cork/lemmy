# Session retro — 2026-05-25 — bm-advisor-multi-lane-ambiguity

**Harness:** claude-code
**Session window:** 2026-05-25T16:00 → 18:50 UTC (~170 min)
**Branch at start:** `e349c4df0` (`governance-v0`)
**Branch at end:** `dd08855e6` (`governance-v0`)
**Files touched:** 9
**Commits:** 4 (auto: 0, explicit: 4 — all `chore(advisor)` or `docs(rules,bm)`)

## TL;DR

Session opened as a `/start-brehon v1-RT-r3` resume, advanced cleanly through plan-approval gate → bm-cut (#464) → impl-0 brief dispatch (#465), then surfaced enough friction in the BM/advisor/multi-lane rule stack to justify a 3-file docs-only patch (`dd08855e6`) closing the gap. The most load-bearing finding: the rules and the practice diverged on three independent axes (bm-cut push behaviour, impl-task brief location, mobile-remote-control mode) and the session discovered each only by trial. Top change proposal: the three docs patches already shipped this session; the next-cycle proposal is a `bm-merge-forward` verb if Mode B recurs in 2+ more lanes.

---

## What surprised us

- **bm-cut.md self-contradicted on whether to push.** Preamble said "Phase 5 below pushes with -u"; Phase 3 said "Do NOT push yet"; Phase 5 was just an output block (no push). Every recent bm-cut brief (RT-r2, deps-r1, this session) overrode and instructed push. The script and practice had silently diverged for ~6 weeks without a retro flagging it.
- **No rule documented "mobile remote-control" lane mode.** The bootstrap handover for v1-RT-r3 mentioned "user is driving from canonical brehon-fork (mobile remote-control)" but `multi-lane-worktree.md` knew nothing about this mode. The CWD-check ritual (§Session-start ritual) assumed every active lane has a separate worktree — but in mobile mode, only canonical exists.
- **Daemon worktree state after bm-cut is a load-bearing side effect.** The bm-cut Junior task ran `git checkout -b phase-v1-RT-r3` against the daemon's main worktree — moving the daemon-side checkout from `governance-v0` to `phase-v1-RT-r3`. This wasn't documented anywhere, but it was *exactly* what made the trunk→phase brief merge possible from canonical (SSH-merge from the daemon worktree, which conveniently was on the phase branch). The pattern works; the documentation didn't.
- **`advisor-orchestrator.md` §1 and §2.1 directly contradicted each other** on where impl-task briefs live: §1 said canonical writes "briefs landed on trunk"; §2.1 said impl-task briefs go "on `phase-<X>`". Both true, but only via a procedure (trunk→phase sync) that wasn't named anywhere.
- **The pre-existing `brehon-fork-rt-r3` worktree claimed in bootstrap didn't exist on this laptop.** `git worktree list` showed only canonical. Per the rule, this should have prompted Mode A/B clarification at session-start; the rule didn't recognize Mode B existed.
- **SSH timeout on session start** (Tailscale DNS-side blip): first probe batch failed with `connect: timed out`; second attempt after user "try tailscale again" succeeded. Reminder that the homeserver path is a single point of failure for *all* state probes — the auto-memory MEMORY.md already flags "DNS/networking SPOF (homeserver-PMD)" as an emerging 2× pattern.

## What to change

| # | Change | Expected effect | Cost | Recurrence |
|---|---|---|---|---|
| 1 | **C: bm-cut.md push-step explicitness** — Phase 4 now does the push (was contradictory). Renumbered Phases 4→runlog → 5, 5→output → 6, +new 7 (daemon-worktree side effect), 8 (finalize hazard). **SHIPPED `dd08855e6`.** | Future bm-cut briefs no longer need to override the script. Reduces brief-authoring time + eliminates the "does it push?" question. | minor | 3× this lane stack (RT-r2, deps-r1, RT-r3 overrode the script) |
| 2 | **B: multi-lane-worktree.md §"Lane modes" (Mode A vs Mode B)** — explicit definitions, detection (`git worktree list`), tradeoffs. Session-start ritual now recognizes Mode B (only canonical exists + lane should be active). **SHIPPED `dd08855e6`.** | Sessions opening into Mode B can identify it from the rule alone; bootstrap handovers can name the mode explicitly. | minor | 1× this session (Mode B undocumented); prior phases (ship-3, deps-r1) likely also Mode B but never named |
| 3 | **B: multi-lane-worktree.md §"Brief location and trunk→phase sync"** — both modes' procedures explicit (Mode A = direct phase-branch commit; Mode B = SSH-merge from daemon worktree, 3 bash steps). **SHIPPED `dd08855e6`.** | Eliminates the "how do I get a trunk-committed brief into the phase branch?" research loop (cost this session: ~10 min). | minor | 1× this session |
| 4 | **A: advisor-orchestrator.md §2.1 mode-aware impl-brief location** — now defers HOW to multi-lane-worktree.md based on mode. Resolves the §1 / §2.1 contradiction. **SHIPPED `dd08855e6`.** | Future sessions reading §2.1 get the procedure, not a contradiction. | minor | 1× this session |
| 5 | **Future: `bm-merge-forward` verb** — if Mode B recurs in 2+ more lanes (≥ 4 total Mode-B uses), promote the SSH-merge alternative to a real BM verb so it can be queued as a normal Junior task instead of an SSH improvisation. The Mode B procedure currently names this as a future option. | Removes the dependency on the daemon-worktree side effect; lane parallelism no longer requires SSH access in the middle of a brief dispatch. | medium (new `bm-verb` script + agent updates) | 1× this session — defer until 2+ recurrences. Track in MEMORY.md under WATCH. |
| 6 | **Future: bm-cut.md script as ground truth for briefs** — the bm-cut brief in this session restated steps that are now fully in the script. Briefs could become 5-line "see `.claude/commands/bm/bm-cut.md`" stubs once we trust the script. | Briefs shrink from ~40 lines to ~10 lines; one source of truth. | minor | 1× this session — defer; first verify the new bm-cut.md doesn't drift. |

## What to carry forward

- **Surface friction-fix opportunities in-session, not at retro time.** The user's "review skills for BM and advisor" prompt mid-session caught friction patterns *while* the experience was fresh. Two days from now the surprise would have faded — this exact retro discipline depends on naming the surprise while the context is loaded.
- **The "ask before authoring patches" pattern from the auto-mode exit reminder worked cleanly.** I had 3 candidate fixes (C/B/A); the AskUserQuestion let the user pick All Three rather than me defaulting to "smallest" and leaving B/A to drift. Use this pattern when the fix scope is genuinely a user choice (size/cost trade-off, not a technical one).
- **Verifying daemon-state before SSH-merge is cheap and load-bearing.** The one-line `git symbolic-ref HEAD` probe on the daemon would have prevented blind merges if the daemon was on a different branch. Added as an explicit precondition to the Mode B sync procedure.
- **Read multiple file refs once, not iteratively.** When understanding the BM/advisor rule stack, batched-Read 4 files in one tool call (impl-task.md, bm-task.md, branch-manager.md, bm-cut.md) instead of one-at-a-time. The single batched-Read produced a complete picture in ~10s; four sequential reads would have taken ~40s + reasoning gaps.
- **roadmap.json updates are advisor responsibility at phase transitions.** Updating the roadmap during this session caught the 4 stale lane statuses (RT-r2 unstarted → done; ship-3 in_flight → done; deps-r1 in_flight → done; RT-r3 plan=null → plan exists). Future sessions should run a roadmap-staleness audit at session-start when the bootstrap's "shipped" list differs from `roadmap.json["lanes"][*]["status"]`.

---

## Three-signal scoring

| Skill / Agent / Command | Saved (min) | Wasted (min) | Surprise | Notes |
|---|---:|---:|---|---|
| `/start-brehon v1-RT-r3` | 25 | 1 | low | 9-probe synthesis caught SSH timeout cleanly; user retry resolved; full state lift in ~2 min |
| `/start-brehon --fast 465` (proposed; not invoked) | — | — | — | future poll candidate |
| AskUserQuestion (plan-approval gate) | 5 | 0 | none | clean fork; user picked "approve — queue bm-cut" without W2 patch |
| AskUserQuestion (fix scope C/B/A) | 8 | 0 | low | user picked All Three vs my expected "C only" — saved 2 separate retro cycles |
| bm-cut Junior task #464 | 15 | 0 | none | 2 min runtime; clean push + runlog + verification |
| impl-0 brief authoring + trunk→phase sync (Mode B) | -10 | 10 | medium | first time discovering Mode B procedure cost ~10 min of research; now codified |
| Plan-approval gate DoD smoke (§15.1 cargo check) | 0 | 0 | none | 4m 24s background; advisor blocked-other-work for full duration is wrong way to frame it (parallel work continued) |
| Watchpoint specificity grep | 3 | 0 | low | Python regex initially too strict (cited 4 fails); broadened pattern showed 9/13 pass; net useful |
| Docs-patch authoring (3 files) | -90 | 0 | none | ~90 min spent — net effect is recouped on next 2-3 BM/advisor sessions |
| `roadmap.json` update | 10 | 0 | none | caught 4 stale lane statuses + cleared confusion for future sessions |
| Roadmap-conflict autoresolve (linter modified `.json` mid-session) | 0 | 2 | low | system-reminder flagged user/linter modification; verified `last_updated_at: 2026-05-25T18:10:00Z` preserved — no clobber needed |

## Complexity scores (heavy tasks only)

| Task | Files | Commits | Runtime (min) | Max log silence (min) |
|---|---:|---:|---:|---:|
| Junior task #464 (bm-cut) | 1 | 1 | 2 | <1 |
| Junior task #465 (impl-task 0 audit) | 0 | 0 | running (started 18:24, retro at 18:55 ≈ 31 min so far) | unknown (still running) |
| Docs-patch (3 files) — advisor-side | 3 | 1 | ~30 | n/a (interactive) |
| Plan-approval gate (read + DoD + watchpoint regex) | 0 | 0 | ~7 (incl 4m24s background cargo) | n/a |

No tasks exceeded the watchdog envelope (60min runtime / 40min log silence / 8 files). Task #465 still running at retro time — if it exceeds the envelope it'll surface in the next retro cycle, but Probe 2 (workspace cargo check) is the dominant cost and should complete in ~5 min on the EliteDesk.

## Decisions to revisit

- **`bm-merge-forward` verb** — defer until 2+ more Mode-B lanes use the SSH-merge alternative. Track via MEMORY.md WATCH section.
- **Briefs-as-stubs migration** — bm-cut.md is now self-contained enough that the bm-cut brief could shrink from ~40 lines to ~10 lines (just dispatch line + scope reference). Defer one cycle; verify the new bm-cut.md ships cleanly through 1-2 more lanes before relying on it.
- **Roadmap-staleness audit at session-start** — pattern: bootstrap claims X is shipped, roadmap says X is in_flight. Could be a one-line addition to `/start-brehon` synthesis step. Defer; needs 2+ recurrence.
- **Mode B carve-out for canonical DQ writes** — the rule says "MUST NOT mutate phase-branch DQ entries from canonical" but in Mode B the canonical IS where the advisor lives. The rule's intent (avoid race conditions across lane sessions) still holds for plan-time DQ, but phase-branch DQ in Mode B has no laptop-side writer — Junior writes via the worker branch. The current §Layout text now correctly says "applies to both Mode A and Mode B" without further carve-out; verify after the next Mode B lane that this didn't accidentally orphan a legitimate phase-DQ-write case.

---

## Promotion candidates (recurrence ≥ 2 in this session, or ≥ 1 here + ≥ 1 in prior memory)

- [ ] **#5 `bm-merge-forward` verb** — promote to `.claude/commands/bm/bm-merge-forward.md` + `branch-manager.md` autonomy table. Wait for ≥ 2 more Mode-B uses (currently 1×).
- [ ] **Mobile-remote-control mode** — already shipped in `multi-lane-worktree.md`; consider promoting to a `.claude/lessons/feedback_mode_b_mobile_remote_control.md` if pattern recurs (1× this session — defer).
- [ ] **Daemon-worktree side effect (Phase 7 of bm-cut)** — already documented in script; if future sessions discover other downstream patterns dependent on this side effect, promote to a `.claude/lessons/feedback_daemon_worktree_state_post_bm_cut.md`.
- [ ] **Rule-script divergence audit** — single-occurrence noise: this session found 3 axes where rule/script and practice diverged. A quarterly audit of "do the scripts match what the briefs actually instruct?" might catch these earlier. Defer; needs 1+ more discovery to confirm the pattern.

---

## Lessons promoted this session

None new — this session's 3 fixes were patches to existing rules (`multi-lane-worktree.md`, `advisor-orchestrator.md`, `bm-cut.md`), not new lesson files. The discoveries were "the rules contradicted themselves" not "we learned something the lesson corpus didn't know."

---

_Generated by `.claude/skills/session-retro/SKILL.md`. Lessons consulted:
`feedback_retro_not_report.md`, `feedback_four_role_retro_signals.md`,
`feedback_retro_task_complexity_score.md`._
