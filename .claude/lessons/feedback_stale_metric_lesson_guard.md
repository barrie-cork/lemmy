---
name: stale-metric-lesson-guard
description: Lessons that cite numeric counts derived from source files need a source+date annotation so the count can be re-verified before being cited in a brief or plan. Two confirmed recurrences in brehon-fork.
type: feedback
---

# Stale metric guard for lesson files

When a lesson cites a numeric count, rate, or size that was derived by
reading a source file (e.g. grep -c, wc -l, test count, migration count),
that number will silently go stale as the source file changes.

## Rule

Any lesson that contains a metric of the form "N <things> in <file>" MUST
include a maintenance annotation immediately after the number:

    <!-- verified: grep -c '<pattern>' <path>, <YYYY-MM-DD> — re-verify before citing -->

Example:
    41 tests in `crates/server/tests/e2e.rs`
    <!-- verified: grep -c '#\[tokio::test\]' crates/server/tests/e2e.rs, 2026-06-01 -->

## Before citing a lesson metric in a brief or plan

Run the verification command from the annotation. If the count has
changed, update the lesson before using it.

    grep -c '#\[tokio::test\]' crates/server/tests/e2e.rs   # e.g. for test count

If no annotation is present, grep the source yourself before trusting
the number.

## Why

**Why:** Two confirmed recurrences in brehon-fork:
1. `feedback_local_validation_cycle_2026_05_02.md` cited "67 tests" in e2e.rs
   (written 2026-05-03). By 2026-06-01, e2e.rs had 41 tests. The wrong count
   propagated into advisor-orchestrator.md's "~26 min baseline" context.
2. `feedback_e2e_nextest_filter_groups.md` cited "41 tests as of 2026-06-01"
   with no re-verify guard. Same class, caught proactively.

**How to apply:**
- When authoring a new lesson with a count: add the annotation inline.
- When reading a lesson in a brief-authoring context: grep before citing.
- When updating a lesson: update the annotation date.

**Generalises to:** any Rust project, any doc that derives a number from
running a command against a file. The pattern is: metric → source → date →
re-verify command.

**Symptom to recognise:** a brief cites "N tests" or "M migrations" and
the actual run fails or produces unexpected results — count was stale.
