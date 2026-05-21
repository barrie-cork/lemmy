---
name: Advisor authors meta-work tasks directly when daemon contamination + worker hangs accumulate
description: For phases shipping zero Rust (all .claude/** + docs/** edits), the cohort/serial Junior dispatch model only pays off when worker independence + daemon stability + worker-side PMD all hold. When 2+ of those fail in one phase, advisor-authoring the remaining tasks beats continued dispatch on round-trip cost.
type: feedback
---

# Advisor authoring under daemon stress

## The decision criterion

For a Brehon sub-phase that ships **zero Rust** (all `.claude/**` rule/lesson edits, `docs/**` non-product additions, retro authoring), evaluate after each Junior dispatch:

- **First worker hang post-DQ-raise** — note it; recovery cost ~10 min advisor toil. Acceptable.
- **Second worker hang post-DQ-raise** OR **first cross-lane daemon-ref contamination detected** — surface to user with the advisor-authoring option as Recommended. Subsequent tasks ship advisor-side.

The threshold is intentionally low. Junior dispatch's payoff comes from parallelism (cohorts) and cheap per-task context; when both are compromised by daemon stress, the residual value (clean attribution to `feat()` commits authored by `bm-task` author identity) is not worth the cost.

## How to apply

- **Plan-time:** the planning brief identifies the phase's Rust-impact volume. Zero-Rust phases get an explicit "advisor-authoring fallback authorised at criterion X" line in §0 PRECONs.
- **Run-time:** advisor's polling-loop catch-fire ticks `cross_lane_contamination_count` and `worker_hang_count`. At 1+1 OR 0+2, AskUserQuestion offering advisor-authoring path.
- **Attribution:** advisor-authored task commits use `feat(<phase>): <description> (task N) — advisor-authored` subject, body cites the originating user gate, body cites the daemon-stress decision point. Co-Authored-By trailer per repo convention.

## Trade-offs

- **Gives up:** Junior parallelism (cohorts), task-author identity (bm-task → claude-advisor-laptop), worker-side cargo pre-push validation.
- **Keeps:** All §13 task IMPLEMENT specs (advisor follows them verbatim), all VALIDATE probes (advisor runs them on lane worktree), all DQ entries (atomic raise+resolve in same commit).
- **Per-task cost ratio:** ~2-5 min advisor authoring vs ~3-13 min Junior wall-clock + ~5-10 min recovery on hang.

## See also

- `feedback_four_role_model.md` — the canonical four-role model this lesson modulates under stress
- `feedback_brehon_autonomy_goals.md` — autonomy + reliability + slow-OK; advisor-authoring serves "reliability" at the cost of "autonomy"
- `feedback_worker_hang_post_dq_raise.md` — the trigger pattern
- `feedback_cross_lane_daemon_ref_contamination.md` — the other trigger pattern
- `.claude/PRPs/reports/v1-rls-r1-retro.md` — first phase where this pattern was formally authorised mid-phase
