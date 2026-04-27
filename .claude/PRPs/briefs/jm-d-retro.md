---
role: advisor
artifact: retro
phase: v1-JM-d
created: 2026-04-27
status: pre-seeded — flesh out post-merge
---

# Brief — v1-JM-d retro (forward-seeded)

This file is pre-seeded with carry-forward items the advisor surfaced *during* execution, before the phase has shipped. At retro time (after `bm-merge`), the advisor authors the full retro per `feedback_retro_not_report.md` ("short-form always" — what surprised, what to change, what to carry forward) and incorporates the items below.

## Pre-seeded carry-forward items

### Architectural deferral — decomposer subagent (from session 2026-04-27)

The advisor authored each impl-task brief by hand mid-flight (read plan §13 → write brief → commit → queue). This is the single biggest cost on the autonomy path. Defer to JM-e or later: introduce a **decomposer subagent** (Sonnet 4.6, mechanical translation) that runs after plan approval and before bm-cut, emitting all `.claude/PRPs/briefs/<phase>-impl-{1..N}.md` + `<phase>-bm-{cut,pr,merge}.md` files in one task. Advisor's runtime job collapses from "author-and-queue" to "fetch-and-queue." Open question for the retro: planning-subagent extension vs. new fifth role; option (2) cleaner, option (1) smaller.

### Other items observed during execution (to expand at retro time)

- Phase-branch routing: `bm-cut` should `git checkout phase-<suffix>` on the daemon's main checkout (not just inside its worktree), `bm-merge` should reset to `governance-v0`. Currently undocumented in `.claude/commands/bm/bm-cut.md` Phase 6. See DQ #53 + addendum at `fe682fbc1` ancestor commits.
- Trunk-vs-phase divergence: every advisor commit on `governance-v0` (briefs, DQs, cheatsheet) diverges the phase branch. Manual rebase required. Should be a documented step in the orchestrator rule, or automated via post-commit hook.
- Settings.json drift: Claude Code worker performed a settings consistency-write at 2026-04-26 14:25 UTC (project `model: claude-opus-4-7` → user-scope `opus[1m]`). Survived as uncommitted drift across worktrees. See DQ #54 resolution; user-scope audit on EliteDesk deferred.
- Telegram notifier sidecar deployed mid-phase (homeserver repo `842aa81` + `f4bd52f`): polls all junior@*.service DBs every 30s, posts done/failed transitions; brehon-fork → BREHON_CONSENSUS topic, others → INFRA. Live since 2026-04-27 09:56 UTC. Future autonomy gain — advisor session no longer needs to be active to surface task transitions.
- DQ-read priority: advisor should read `decision-queue.json` IMMEDIATELY on any task-status transition, not just when checking for new `pending`. Self-resolved entries (`impl-self-resolved`, `bm-self-resolved`) often contain plan-level lessons that affect the *next* task's brief. Missed during task #9 → task #2 transition: DQ #55's `lemmy_diesel_utils` CLI-args mismatch directly affects task 2's `print-schema` step (plan §13 task 2 assumes CLI-args, same wall hits at schema-regen time). Pattern fix: amend `.claude/rules/advisor-orchestrator.md` polling-loop discipline to "DQ read fires on every status transition AND covers resolved-by-self entries since last advisor read, not just pending."
- Plan-DoD validation gap: DQ #55's CLI-args mismatch was the kind of unexecutable-plan-step that `feedback_pre_phase_dod_smoke_test.md` warns about. Pre-bm-cut DoD smoke test missed it because §15 didn't include `cargo run -p lemmy_diesel_utils --features full -- revert --limit=2` (the plan's task-1 VALIDATE block did but §15 is the canonical smoke list). Either §15 should include every task-N VALIDATE command, or the DoD smoke should run task-N VALIDATE blocks too.
- **EliteDesk OOM-cascade incident, 2026-04-27 ~10:35 UTC**: task #10 (`[role:impl-task] v1-JM-d task 2`) ran `cargo check --workspace --features full` per plan §13 task 2 VALIDATE, and the EliteDesk soft-hung — kernel responsive (LAN ICMP), userspace dead (sshd, tailscaled, junior-monitor, notifier all silent). Hang persisted >14 min; no self-recovery via OOM-killer. Required out-of-band intervention to recover. Lesson: `cargo check --workspace --features full` on the Lemmy monorepo is **catastrophically memory-hungry** — needs a hard ceiling. Three concrete fixes for retro: (1) add `MemoryMax=` to `/etc/systemd/system/junior@brehon-fork.service.d/` drop-in so the daemon's worker tree is OOM-bounded; (2) consider switching workspace-wide checks to `--no-default-features` or per-crate `-p` invocations in plan VALIDATE blocks (where compatible — `feedback_features_full_p_crate_incompatible.md` is the gotcha); (3) add a pre-flight memory check to plan §13 task 0 probes that fails fast if available RAM < N GB. Server tuning: confirm swap is configured + sized; consider zswap. Captured before resolution because the box was unreachable when this lesson surfaced.
- **DNS chain SPOF on EliteDesk hang**: when the EliteDesk hung (above), the laptop lost ability to resolve external hostnames because Pi-hole (the laptop's primary DNS resolver) runs as a Docker container on the EliteDesk. Net effect: `git push origin governance-v0` failed with `Could not resolve host: github.com` — so the advisor session couldn't even commit *retro lessons about the hang* to GitHub during the hang itself. Fix candidates: (1) add 8.8.8.8 / 1.1.1.1 as a secondary DNS on the laptop's network stack so Pi-hole outages don't cascade; (2) document the SPOF in `docs/architecture.md` security/availability section so it's an explicit known fragility rather than a surprise; (3) consider running a redundant Pi-hole instance on a non-EliteDesk host (or a public resolver as automatic failover via `systemd-resolved`/`dnsmasq` priority); (4) on Windows, `tailscale set --accept-dns=false` is the cleanest emergency bypass — toggleable from PowerShell without admin, reversible with `--accept-dns=true`. Used during this incident at 11:45 UTC to unblock retro pushes. The autonomy goal is "advisor + daemons run while user is asleep" — that's incompatible with a DNS chain that breaks when one daemon-host hangs.

### Investigation findings (laptop-side, 2026-04-27 ~11:50 UTC, EliteDesk still hung)

- **EliteDesk RAM budget per `docs/architecture.md`**: 16GB total, ~13.25GB Docker, ~2.75GB headroom for OS + Junior workers, 4GB swap on data NVMe, `vm.swappiness=10`. **A `cargo check --workspace --features full` on Lemmy easily peaks at 8–12GB during type-check — far past the ~2.75GB headroom.** Combined with low swappiness (kernel resists paging until late), the runaway is hard and fast: OOM-cascade in seconds rather than gradual swap-thrash.
- **No `MemoryMax=` on `junior@brehon-fork.service`**: drop-ins inspected at `systemd/junior@brehon-fork.service.d/{output-tokens,rust-analyzer}.conf` — neither sets memory limits. Daemon currently inherits the system default (unbounded).
- **Concrete fix shape (proposed; ship after JM-d retro signs off)**: new drop-in `systemd/junior@brehon-fork.service.d/memory.conf` with `MemoryMax=6G`, `MemoryHigh=5G`, `MemoryAccounting=true`, `TasksMax=1024`. systemd cgroups propagate to children by default, so a worker's `cargo check --workspace --features full` would be cgroup-OOM-killed instead of taking the box down. Junior observes the kill as a non-zero exit → task transitions to `failed` → advisor sees it. **Tradeoff**: 6GB may be too tight for the workspace check to *succeed* — we'd convert an OOM-cascade into reproducible task failures. Two responses, retro should pick: (a) accept fail-fast, plan VALIDATE blocks switch to per-crate `-p` invocations (constrained by `feedback_features_full_p_crate_incompatible.md` — only `--workspace` composes with `--features full`, so per-crate needs custom feature subsets); (b) raise the cap to 10G, sacrifice some Docker headroom temporarily during cargo runs.
- **Per-daemon vs template-wide**: only `junior@brehon-fork` runs Lemmy-scale builds today; other daemons (agent-grey, food-producer, midleton-market, my-food-system, security, web-archive) have lighter codebases. Shipping the drop-in only for `brehon-fork` first is cheapest. Template-level (`/etc/systemd/system/junior@.service.d/`) drop-in would affect all instances — flagged as risk per `feedback_systemd_dropins_invisible.md` (cross-instance changes need explicit audit).

## Retro authoring at merge time

Per `feedback_retro_not_report.md`, the retro is distinct from the completion report. Three questions:

1. **What surprised us** — including the friction patterns above and any new ones from impl tasks 2–8 + CR cycle.
2. **What we'd change** — concrete edits to rule files, agent contracts, or skills.
3. **What we carry forward** — promote any retro-surfaced lesson to `.claude/lessons/feedback_*.md` and to PMD; cite by filename in the retro.

Per `feedback_four_role_retro_signals.md`, structure each `## What surprised us` and `## What we'd change` H2 with H3 sub-sections per role (advisor / planning / impl / BM) — otherwise the loudest role's signals dominate.
