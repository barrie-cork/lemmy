---
name: feedback_validate_pending_laptop_write_then_stop
description: impl-task workers must write validate-pending-laptop DQ entry and stop — never run cargo-check.sh themselves; laptop advisor runs validation
type: feedback
---

Cargo validation belongs on the laptop, not the daemon. When a brief names `validate-pending-laptop`, the worker's job is to **write the DQ entry and push — then stop**. The laptop advisor pulls the entry and runs `cargo-check.sh --workspace --features full` locally.

**Why:** Two or more workers sharing the daemon's single build target directory serialize on the cargo file lock, adding 10-30+ min of silent waiting per task. The laptop has faster hardware, a warm incremental cache, and runs cargo in isolation. This is why the `validate-pending-laptop` kind exists.

**How to apply:** In every `impl-task` brief, the validate-pending-laptop constraint must say explicitly:

> Write the `validate-pending-laptop` DQ entry with `commands: ["./scripts/brehon/cargo-check.sh --workspace --features full"]`, commit + push, then **stop**. Do NOT run `cargo-check.sh` yourself — validation is delegated to the laptop advisor session.

The advisor then runs `cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > <log> 2>&1"` (Windows) or `./scripts/brehon/cargo-check.sh --workspace --features full > <log> 2>&1` (Bash) against the phase branch after the finalize-merge lands, per the validate-pending-laptop handler in `.claude/rules/advisor-orchestrator.md`.

**Recurrence:** v1-RT-r5 Cohort A (#549/#550) both ran cargo on the daemon (15:25-15:50), causing ~45 min of file-lock contention across two concurrent workers.
