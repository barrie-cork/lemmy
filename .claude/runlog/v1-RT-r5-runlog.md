# v1-RT-r5 runlog

Lane: RT (Reputation Tuning) — final sub-phase. Mode A (dedicated lane worktree `C:/Users/barri/Developer/brehon-fork-rt-r5`).
Scope: `reputation_rollup_cron` in `scheduled_tasks.rs` + `GET /admin/reputation/rollup` + `ENTRY_KIND_ROLLUP_RECOMPUTED` emit + e2e. Fires the last pre-landed RT-r1 const.

---

## advisor: /auto-roadmap RT-r5 driven 2026-05-31

- **Phase 0 recovery:** roadmap entry `v1-RT-r5` was `unstarted` despite bm-cut having run (`560eda9b1`); `/roadmap-next` flip never completed. Flipped to `in_flight` inline + cleaned stale `what_remains` (RT-r4 done). Commit `f06d23cf5` on governance-v0.
- **Plan-gap:** no plan file existed. Auto-authored planning brief `.claude/PRPs/briefs/rt-r5-planning-1.md` from PRD §5.5/§11-row-5 + entry-kind registry. User-approved (AskUserQuestion). Committed `5fff92faa`, pushed.
- **Clarify gate (advisor-mode):** 1 clarify-DQ `81ae24440317-001` — rollup compute path. Resolved: `reputation_rollup_cron` MUST use a dedicated weighted-average-of-per-community-snapshots helper, NOT reuse `recompute_snapshot(person_id, None)` (which event-sums instance-scoped events — categorically different from PRD §5.5 weighted average). Verified against `reputation_snapshot.rs:222/682/736/770`. Commit `293b57666`, pushed.
- **Daemon-local branch fix:** planning task #544 failed instantly — daemon `/srv/brehon-fork` had no `phase-v1-RT-r5` ref (`fatal: invalid reference`). bm-cut created the branch in the lane worktree + pushed to origin, but the daemon's separate clone never got it. Fixed: `ssh homeserver "cd /srv/brehon-fork && git fetch origin && git branch phase-v1-RT-r5 origin/phase-v1-RT-r5"` (no checkout — daemon HEAD stayed on phase-v1-quality-r3b). Bare name now resolves to `293b57666`.
- **Planning dispatched:** task **#545** (`[role:planning]`, base `phase-v1-RT-r5`) — running as of 2026-05-31 ~13:04 UTC. (#544 = failed pre-fix; do not retry.)

**Next:** await #545 → DoD smoke (§15) + watchpoint specificity gate → user gate 1 (plan approval) → pre-seed auto-state at `impl-cohort-1` → `/auto-phase v1-RT-r5`.
