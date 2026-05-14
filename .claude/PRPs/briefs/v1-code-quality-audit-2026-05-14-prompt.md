---
purpose: Bootstrap prompt for a fresh advisor session to produce the v1 code-quality audit
authored: 2026-05-14
authored_by: governance-v0 session (companion to other session parked on v1-ship-1 plan approval)
target_session: fresh Claude Code session in C:/Users/barri/Developer/brehon-fork on governance-v0
target_output: .claude/PRPs/reports/v1-code-quality-audit-2026-05-14.md
related:
  - .claude/PRPs/reports/v0-endpoint-coverage-2026-05-14.md (substrate inventory; required reading)
  - .claude/lessons/feedback_concurrent_advisor_session_collision.md (PMD id 302; concurrent-session discipline)
  - .claude/PRPs/plans/v1-ship-1.plan.md (parked plan awaiting approval; this audit may revise sequencing)
---

# v1 code-quality audit — fresh-session bootstrap prompt

Paste everything below the `---PROMPT---` marker as the first message to the new session. The session will read CLAUDE.md auto, pre-fetch upstream, dispatch parallel Explore subagents, and produce the audit report. It then goes idle.

---PROMPT---

You are a fresh Claude Code session in brehon-fork (governance-v0 branch). Your single task is to produce a code-quality audit of all Brehon-authored code on this fork. When the audit ships, you go idle — no follow-up work in this session.

Read CLAUDE.md before anything else. The 15 ADRs, 11 v0 endpoints, and v0 simplifications are hard constraints. The four-role model + governance-v0 trunk discipline apply.

Read .claude/PRPs/reports/v0-endpoint-coverage-2026-05-14.md before dispatching subagents. It enumerates what's shipped and references ADRs 011/013/014/015 + hash chain. Treat it as the substrate inventory.

---

PRE-AUDIT — upstream fetch + drift baseline

Capture the upstream baseline so divergence findings are anchored to a real reference, not a stale local one.

  git fetch upstream
  UPSTREAM_TIP=$(git rev-parse upstream/main)
  LOCAL_MAIN=$(git rev-parse main)
  DRIFT=$(git rev-list --count main..upstream/main)
  echo "audit baseline: brehon-fork main @ ${LOCAL_MAIN:0:9}, upstream/main @ ${UPSTREAM_TIP:0:9}, drift = ${DRIFT} commits"

Record these three values verbatim in §1 Method of the audit report. They are the anchor for every Axis-4 divergence finding.

For Lens-1 parity reference files: every Explore subagent reads its representative upstream file from upstream/main, NOT from local main. Example: when contrasting a Brehon governance handler against upstream's post/like.rs, the read command is `git show upstream/main:crates/api/api/src/post/like.rs`, not `Read crates/api/api/src/post/like.rs`. Local main is stale.

Do NOT rebase onto upstream/main. That is a separate decision the divergence catalog from this audit will inform.

If DRIFT > 200 commits, surface this in §1 and ask the user whether to fast-forward main before continuing. <=200 is safe to proceed.

---

GOAL

Audit all Brehon-authored code on governance-v0 for production-level quality. The concern is spaghetti, workarounds, short-term fixes that need refactoring before further phases stack on top. Shipping order is not the question — code health is.

---

LENSES (apply in parallel, rank by remediation cost)

Lens 1 — Lemmy-upstream parity. Does the code read like the rest of Lemmy 1.0-beta? Same idioms, same error patterns (LemmyResult<T>, LemmyError variants), same test fixture conventions (AsyncPgConnection, DbPool::Conn), same trait usage, same module structure (db_schema/source/ vs api/src/), same handler shape (Json<X> -> LemmyResult<Json<Y>>), same Diesel insert/update patterns. Anything that looks bolted on rather than native to Lemmy is a parity-fail.

Lens 2 — Rust idiomatic best practices. No unwrap()/expect() in non-test code. No #[allow(...)] escape hatches without a // SAFETY: or // MIRROR: justification. Error types are typed (not stringy format!("{e}") chains beyond what LemmyError demands). ? propagation correct, no .ok() swallowing errors. Async patterns don't block. No premature Arc<Mutex<>> where &mut self would do. No clone-storms in hot paths.

Lens 3 — Long-term maintainability. Future-maintainer-readable. Naming reflects intent (no tmp, helper, do_thing). Comments explain WHY not WHAT, only where non-obvious. Functions have single responsibilities. Modules have clear boundaries. No 600-line files that should be 4 modules. No copy-paste-with-tweaks where extraction is obvious. No dead code paths or commented-out blocks.

Axis 4 — Lemmy-upstream divergence (separate axis, not a fail by default). Every finding tagged: quality-fail / divergence-from-lemmy / both / N/A. The fork's plan is to stay separated, so divergence is allowed — but every divergence should be intentional + visible. Surprises during a future upstream-merge are the real cost.

---

SCOPE

Includes (all Brehon-authored only — verify via git blame):
  - crates/api/api/src/governance/**/*.rs
  - crates/api/api_crud/src/governance/**/*.rs
  - crates/db_schema/src/source/governance/**/*.rs
  - crates/db_schema_file/src/source/governance/**/*.rs (if Brehon-authored)
  - crates/db_schema_file/src/enums.rs (just Brehon-added enum variants)
  - crates/db_schema_file/src/schema.rs (just Brehon-added sql_types + table! extensions)
  - All migrations under migrations/2026-*/ (up.sql + down.sql)
  - All Brehon-fork test fns in crates/server/tests/e2e.rs (anything inside `mod *_fixtures` whose author is Brehon, plus phase1_migrations_round_trip)
  - crates/tools/seed_founders/ (Brehon-authored entirely)
  - crates/api/api_common/src/governance/ if it exists
  - Any other governance/ subdirectory across crates

Excludes:
  - Upstream Lemmy code (use git log + git blame to confirm Brehon authorship; anything authored by Lemmy maintainers stays out of scope)
  - Cargo.toml dependency-list deltas (separate concern; future deps audit)
  - .github/workflows/ (CI is its own audit lane)

---

METHOD

Dispatch parallel Explore subagents, one per scope area:
  - Explore A: crates/api/api/src/governance/**
  - Explore B: crates/api/api_crud/src/governance/**
  - Explore C: crates/db_schema/src/source/governance/** + db_schema_file/src/source/governance/** + enums.rs/schema.rs Brehon adds
  - Explore D: migrations/2026-*/
  - Explore E: crates/server/tests/e2e.rs (Brehon-authored fns only) + crates/tools/seed_founders/

Each Explore returns ~50 candidate findings raw. You synthesise + dedupe + rank.

For Lens-1 parity comparison: each Explore picks one representative upstream Lemmy file (NOT Brehon-authored) as a reference shape — e.g. for handlers, contrast a Brehon governance handler against upstream `crates/api/api/src/post/like.rs` or similar. Cite the upstream file in any parity-fail finding so the reviewer can see the contrast.

For divergence tagging: assume divergence is allowed (per user constraint "keep our implementation separated"). Tag it for visibility, don't flag it as a fail unless it ALSO triggers Lens 2 or Lens 3.

Read-only audit. Do NOT modify code. Do NOT propose specific edits beyond the "recommended fix" line per finding.

---

OUTPUT

Single document at .claude/PRPs/reports/v1-code-quality-audit-2026-05-14.md.

Required structure:

§1 Method — what subagents ran, what they read, total file count, total LOC of Brehon-authored code, the three upstream-drift values from pre-audit (LOCAL_MAIN, UPSTREAM_TIP, DRIFT).

§2 Headline counts table — findings per (lens × severity) cell.

§3 Detailed findings — each finding has fields:
  * location (file:line range)
  * lens(es) — which of Lens 1/2/3 it fails
  * divergence axis tag — quality-fail / divergence-from-lemmy / both / N/A
  * anti-pattern name (concise label, e.g. "unwrap in handler", "copy-pasted helper")
  * description (1-2 sentences)
  * why it's wrong (1-2 sentences; cite a lesson or Lemmy convention)
  * recommended fix (1 sentence + line range for the new shape)
  * estimated effort (XS = <30 min / S = 30-120 min / M = half-day / L = day+)
  * frequency count (how many distinct sites this pattern appears at — same finding type counted once with frequency, not N times)

§4 Ranked refactor backlog — top-20 findings ranked by (severity × frequency × inverse-effort). Each entry tagged:
  - "fix-before-next-phase" (strict gate per user 2026-05-14: blocks v1 PRD resumption until shipped as a refactor PR, regardless of whether the next PRD touches this code)
  - "fix-during-relevant-sub-phase" (handle when the touching phase comes up)
  - "accept-and-document" (out-of-scope debt; log it and move on)
  - "intentional-divergence-accept" (Axis-4 only; intentional + safe)

§5 Cross-cutting observations — module-boundary issues, naming inconsistencies, recurring anti-patterns that span multiple files (not just per-file findings).

§6 Divergence catalog — separate appendix listing every (b)+(c) tagged finding so future rebase work has the full inventory. For each: what Lemmy does, what Brehon does, why Brehon does it differently if knowable.

---

POST-AUDIT WORKFLOW (downstream sequence for context, not your task)

After this audit ships, the user-side sequence is:

Step 4 — User reviews §4 ranked refactor backlog. Decides which findings are fix-before-next-phase (block PRD resumption) vs fix-during-relevant-sub-phase vs accept-and-document.

Step 5 — Strict gate (per user 2026-05-14): every fix-before-next-phase finding ships as its own refactor PR before any v1 PRD work resumes. Even if the refactor is in code not touched by the next PRD. Refactor commits go in clean, separate PRs (phase-v1-refactor-<area>) so CR can review them in isolation. NO bundling refactor work with feature PRDs.

Step 6 — Once all fix-before-next-phase refactors are merged into governance-v0, resume v1 PRD implementation. Current PRD landscape (verify status per PRD §11 phase table when planning):
  - v1-sponsor-liability.prd.md — SL-a through SL-e shipped
  - v1-jury-mechanics.prd.md — JM-d + JM-e shipped; JM-f pending
  - v1-reputation-tuning.prd.md — RT-r1 shipped; r2 critical-path (gates r3 + r5); r4 + r6 parallel-eligible
  - v1-admin-dashboard.prd.md — partial (check git refs for phase-v1-AD-*)
  - v1-federation-inbound.prd.md — status unknown; check git refs
  - v1-ship-readiness.prd.md — 384 lines, authored 2026-05-14 by parallel session; v1-ship-1 plan currently awaiting approval. Whether this PRD comes before or after others is a user-side ordering decision informed by your audit findings.

Hard rule for the resume-PRD sequence: every PRD planning Junior task MUST read this audit report as required reading. If a PRD's tactical phases would re-create an anti-pattern flagged here, the planner MUST cite the audit finding and either (a) plan the refactor as a pre-condition phase, or (b) explicitly accept the divergence in the plan body with rationale. No silent re-creation of flagged anti-patterns.

Hard rule for code-quality maintenance going forward: this audit becomes the baseline. Every subsequent retro (per phase ship) checks whether the shipped phase introduced new instances of the audit's flagged anti-patterns. If yes, retro logs as a regression and the next phase's planning gets an extra constraint: "avoid X" citing this audit + the retro.

You do NOT execute steps 4-6. You only produce the audit report. Note these steps in §0 Context-for-reviewer at the top of the report so future readers see the framing.

---

WHEN DONE

Surface a one-paragraph summary in this session's final response:
  - Total findings + count of "fix-before-next-phase" tier
  - The single highest-leverage refactor (highest severity × frequency, lowest effort) — the one to consider tackling first
  - One sentence on the divergence picture: is the fork drift mostly intentional or mostly accidental?

Then this session goes idle. No follow-up work in this session. The parent will open a new session to plan the first refactor PR once they've reviewed your report.

Concurrent-session discipline (per feedback_concurrent_advisor_session_collision.md, PMD 302): another advisor session is parked on the v1-ship-1 plan-approval gate. Do not write to .claude/decision-queue.json. Do not edit files outside the audit report path. Do not dispatch Junior tasks. Single-purpose session. If you find anything outside scope that needs surfacing, put it in §5 Cross-cutting observations of the report, not in a sideloaded write.
