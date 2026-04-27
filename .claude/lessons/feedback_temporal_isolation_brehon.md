---
name: Temporal isolation > spatial isolation for Brehon cargo workloads
description: When Brehon cargo workloads share the EliteDesk with cron-driven backups/crawls, schedule them outside the cron windows; raising memory caps doesn't fix contention.
type: feedback
---

When `cargo check`/`cargo test --workspace --features full` workloads on the EliteDesk run concurrently with cron-driven jobs (NAS backups 03:00–03:15, web-archive crawls 03:00–07:00, weekly review Sun 02:30, restore drill Sun 04:00), they OOM-cascade the box even with `MemoryMax=10G` on `junior@brehon-fork`. The cap protects sshd/tailscaled but the system OOM-killer still picks off other workloads (e.g. `brehon-jmd-pg`).

**Why:** the EliteDesk has 16 GB total. With 5–6 GB consumed by always-on infra plus side-project stacks (when not standed-down), Brehon has <8 GB headroom; cargo check on Lemmy peaks at 8–12 GB. Two incidents on 2026-04-27 (task #10 cascade at 10:35 UTC, task #11 squeeze at 12:30 UTC) confirmed the pattern.

**How to apply:**

- Forbidden windows (UTC) — `impl-task` agent's task-0 pre-flight refuses to start cargo work in:
  - Daily 02:55–04:15 (NAS backup chain + web-archive govie-search)
  - Sunday 01:55–02:35 (HSE crawl + weekly review)
  - Sunday 03:55–04:30 (restore drill)
- Recommended Brehon execution windows (UTC): primary 16:00–02:30, secondary 04:30–14:59.
- Two-layer enforcement:
  1. Advisor's polling loop self-defers without filing a DQ for routine deferrals (per `.claude/rules/advisor-orchestrator.md` "Forbidden execution windows").
  2. `impl-task` agent's task-0 pre-flight bash refuses + files a DQ catch-fire if the advisor missed the gate.
- Don't just raise the cap. Raising `MemoryMax` past 10G eats infra headroom. Cap stays; scheduling fixes the contention.
- Stand-down for the duration of a heavy phase: pause side-project Junior daemons + Docker stacks (agent-grey, midleton-market, my-food-system) to free ~3.5 GB. See `homeserver/docs/troubleshooting-laptop-elitedesk.md` "Stand-down log".

**Source-of-truth for the cron table:** `homeserver/docs/troubleshooting-laptop-elitedesk.md` "Temporal isolation" section. If new cron jobs are added, update the doc, the orchestrator rule, and the brief template (`.claude/PRPs/templates/impl-task-brief.template.md`).
