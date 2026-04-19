# v0-polish execution plan

**Status:** ready to execute. Starting point `governance-v0` HEAD `08065e1a1` (PR #46 Phase 6 merge, 2026-04-19).

**Target:** tag `v0.0.0` after all polish PRs merge to `governance-v0`.

**Why this plan exists:** Phase 6 was the last implementation phase. Polish week closes production-code bugs, ships the README, and cleans up deferred items accumulated across Phases 1–6. Originally budgeted ~1 week per `docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md §9`.

## Source of truth

GitHub issues under `barrie-cork/lemmy` carry current state per item. This file is the **priority-ordered shipping plan**. If an issue's body conflicts with this plan, the issue wins for scope detail; this plan wins for priority + grouping.

## Priority ordering

### 🔴 Blocks v0 tag — production-code correctness

1. **GH #48** — `governance_log` atomicity + causal-ordering + hidden-write sweep (risk:high)
   - INSERT + UPDATE on `governance_log::append()` not atomic (Phase 5a-era bug; may leave `signature = NULL`)
   - `federation_sanction_sent` appended before its triggering `case_decided` (ADR-008 causal order)
   - `actor_pseudonym_helper::get_or_create` silent INSERT without matching log append (ADR-015)
   - **Shared fix strategy:** single in-flight transaction covers all writes + log append for each event boundary
   - **Acceptance:** `governance_log_hash_chain_holds` + documented invariant walk or crash-injection test
   - **Why first:** three related violations of ADR-006/008 on production code path. Must ship before v0 tag.

2. **GH #35** — NOTIFY trigger fires on INSERT before signature UPDATE (risk:high)
   - Subscribers observe unsigned rows. Related to #48 atomicity fix — likely one combined PR.
   - **Acceptance:** signature-first invariant observable to subscribers.

3. **GH #34** — `request_appeal` unreachable for Decided cases with `closed_at` in future (risk:high)
   - Pure bug with an easy fix — route/guard logic inverts. Low blast radius, high severity.

4. **GH #33** — declining juror can be picked as own replacement (risk:critical)
   - Selection-pool contamination bug. Likely `WHERE NOT IN (declined_jurors)` clause.
   - Grouping decision at polish-1 kickoff: bundle with #48/#35/#34 if fix is small; standalone if larger.

5. **GH #54 Critical #2p-7** — lock-race TOCTOU in `scripts/brehon/task-hopper.sh` release path
   - Advisor tooling, not production governance. Atomic-rename patch sketched in issue body.
   - **Severity note:** real race but v0 runs are short and single-worker; first fix (`0ff06abc9`) closed the guaranteed-race window; remaining TOCTOU is narrow.

### 🟠 Ship before v0 tag if cheap

6. **GH #54 Major #2p-2 / #2p-3** — `publish_sanction_notice.rs` defensive hardening
   - Reject remote actor at builder; enforce federated-scope invariant.
   - Belt-and-braces; no current caller violates. Unit tests on the builder verify.

7. **GH #54 Major #2p-6** — brief-doc rot
   - Agent briefs show `task-hopper.sh start 70` (bare numeric); validator requires `task-<id>`.
   - JSON shows agents invoked correctly — doc-only fix.

8. **GH #36** — 404 allowed in e2e route-registration gate masks missing route (risk:medium)
   - Test-hygiene bug; gate must assert 200/401/403, not accept 404.

9. **GH #47** — `task-hopper.schema.json` enforce lifecycle invariants (risk:low)
   - JSON-schema hardening for the hopper state file.

10. **GH #37** — `cargo-test-e2e` workflow silently overrides `rust-toolchain.toml` (ops / risk:low)
    - CI uses stable instead of 1.95. Known workflow bug.

11. **GH #49** — `task-hopper.sh` `TH_ISSUE_URL` env propagation (risk:low)
    - Already fixed in PR #46 at `9aa1a778a`. **Verify and close without reopening the file.**

12. **GH #38 / #39 / #50 / #51 / #52** — minor docs fixes
    - #38 fork-local design-doc citations + broken relative link in Phase 5c docs
    - #39 SUBSCRIPTIONS.md clarify serial-id gap semantics under rollback
    - #50 Phase 6 plan MD040 fences + stale enum type names
    - #51 DQ-6.7 wording stale (references fresh pool conn but code uses in-flight tx)
    - #52 task-hopper.md rule contradicts itself on agent-edit policy during crash recovery

### 🟡 Original cleanup items — no GH issue yet, file one if taking on

13. **ts-rs export regeneration** — skipped in Phases 3–5 due to upstream `DbUrl` debt. Two paths:
    - (A) Fix the `DbUrl` TS impl as a carry-patch and regenerate
    - (B) File a v1-deferral issue and drop from v0 scope

14. **Redaction regex harden** — `IMPLEMENTATION-PLAN-v0.md §7.1`. Focused test pass against real case data.

15. **OQ-006 threshold formula tuning** — Phase 4 placeholder replaced by Phase 5a config-driven multiplicative formula. Decide: keep 5a defaults or tune now that Phase 6 federation data exists.

16. **v0 announcement + README** — current README is Lemmy's upstream. Need: what Brehon is, what v0 proves, how to run/federate, v1/v2/v3 scope. Point at `IMPLEMENTATION-PLAN-v0.md §10` for non-goals.

17. **Full-workspace clippy sweep `-D warnings`** — every crate, not just governance-touched. Rust 1.95 lints surfaced during Phase 5→6 transition; confirm no dormant issues in untouched crates.

18. **Dead-code sweep** — `cargo +nightly udeps` or equivalent. Phase 5c §3 item 7 deferred NOTIFY-subscriber helper extraction to "if a second caller exists" — check if anything grew a second caller.

19. **CodeRabbit elevated-profile sweep** — `.coderabbit.yaml` currently `profile: assertive`. Consider one-time elevated pass across full `governance-v0` diff-from-upstream via meta-PR. **Note:** PR #46 already got two full CodeRabbit cycles; marginal value may be low. Decide at kickoff.

20. **Big renames / reorgs** — governance crate naming consistency audit; migration naming audit.

21. **DQ #37 layering revisit** — Phase 6 task 75 moved `governance_log` + `redaction` from `lemmy_api` → `lemmy_db_schema`. Worked but wasn't advisor-reviewed. Cleaner alternative (receiver free-function in `lemmy_api` called via `Data<LemmyContext>`) exists. SHAs: `c7f57bf0f` (refactor), `ec41597d1` (attribution correction). Evaluate if the layering grates in elevated-profile review (#19) — if so, revert and re-do.

### 🔵 v0-polish-deferred trivials

22. **GH #53** — micro-optimisations in `submit_jury_vote` + `governance_log::append` (enhancement / v1-tagged)
    - Dedup `map_decision_to_sanction` call; cache signing key via `OnceLock`. Non-blocking.

### 🟢 Tag step

23. **Tag `v0.0.0`** — after items 1–22 ship cleanly to `governance-v0`.

## Shipping strategy

Five PR groupings, chosen to balance CodeRabbit review load against integration velocity.

| PR | Items | Branch | Scope |
|---|---|---|---|
| **polish-1 critical bugs** | 1–3 (+ maybe 4) | `polish/critical-bugs` | GH #48 + #35 + #34. Three ADR-006/008 production bugs sharing transaction/signature infrastructure. Bundle because the fix strategies share code surface. |
| **polish-2 Bucket C** | 5–7 | `polish/bucket-c` | GH #54 subset: #2p-7 lock-race TOCTOU + #2p-2/#2p-3 defensive hardening + #2p-6 brief doc-rot. Confined to hopper script + AP activity builders. |
| **polish-3 docs** | 12 | `polish/docs-sweep` | GH #38 + #39 + #50 + #51 + #52. Markdown-only, zero code. Single `chore(docs): v0-polish docs sweep` commit. |
| **polish-N topic** | 13–21 | `polish/<topic>` per item | One PR per topic: ts-rs regen, redaction harden, OQ-006 tuning, README, clippy sweep, udeps, elevated-CodeRabbit, renames, DQ #37 layering revisit. Topics are unrelated; don't bundle. |
| **polish-tag** | 23 | `polish/v0-tag` | After all above ship, cut release commit + tag `v0.0.0`. |

**Items 8, 9, 10, 11 not yet grouped.** Decide at kickoff:
- Options: one `polish/ops-misc` PR, or absorb into polish-3 docs if they stay docs-only.
- Item 11 (#49) is already fixed in PR #46 — just verify and close without code changes.

## Rules for all polish PRs

- **Base branch:** `governance-v0` (current tip `08065e1a1`)
- **Merge method:** `--merge` (never `--squash`) per `feedback_pr_per_phase.md`
- **Critical-only merge discipline:** CodeRabbit findings beyond Critical get deferred to a follow-up polish PR rather than chased in the current one. PR #46 became a 30+-commit spiral; don't repeat that.
- **Each PR gets its own retro** at `.claude/PRPs/reports/polish-<n>-retro.md` per `feedback_retro_not_report.md`. Three H2 sections: what surprised us / what to change / carry-forward.
- **Execution model:** single impl session per PR (Phase 5a/5b/5c model, not Phase 6 layered-agents). PRs are small, sequential after polish-1, and don't have clean dependency boundaries warranting parallelism.

## Sequencing

1. **polish-1 first** (blocks v0 tag on its own merits — critical production bugs)
2. **polish-2 + polish-3 + polish-N in parallel** after polish-1 merges (they don't share code surfaces)
3. **polish-tag last**

## Completed during PR #46 (struck from scope)

- ✅ Critical actor-binding check on AP wrappers — `d70610980`
- ✅ Critical idempotency guard at submit_jury_vote post-decision — `41f1d0379`
- ✅ Critical lock-race compare-and-release v1 — `0ff06abc9` (v2 TOCTOU → #54 item 5)
- ✅ Major e2e no-auto-apply DB-state asserts — `455a7dbe4`
- ✅ Major task-hopper.json worktree path scrub — `addc0c9ab`
- ✅ Major TH_ISSUE_URL env export — `9aa1a778a`
- ✅ Major atomic insert + log append in **inbound** receivers (partial — send-path work still needed per #48) — `99dce6be2`
- ✅ Minor attribution-integrity pattern rule — `728659a24`

## What to skip in polish week

- Introducing new lints or formatters (plan §5.3 explicit)
- New abstractions / helper extractions beyond existing shipped ones — extract only when a second caller exists
- v1 scope items — all OQs marked v1 in `99-decisions-and-open-questions.md` stay v1
- v1-tagged GH issues (#22–#31, #40, #41, #53 and others labeled `v1`) — do not pull into v0 polish

## Cross-references

- `docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md §9` (effort table row for polish week)
- `docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md §7.1` (redaction regex, threshold formula risks)
- `docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md §10` (hard non-goals)
- `.claude/PRPs/reports/phase-6-complete-report.md` (Phase 6 retro + Bucket C amendments)
- GH #54 (PR #46 Bucket C follow-ups — detailed issue-body scope for items 5–7)
- GH #48 (governance_log sweep — items 1–2 in combined fix)
- governance-v0 HEAD at plan-write time: `08065e1a1`
