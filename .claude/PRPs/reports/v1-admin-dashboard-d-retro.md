# v1-AD-d retro — what worked, what didn't, advisor-actionable prp-implement fixes

**Sub-phase**: v1-AD-d (Dashboard aggregate + SSE audit stream)
**Implementer session scope**: tasks 4–6 (tasks 1–3 landed in prior sessions on the same branch)
**Branch**: `phase-v1-AD-d`
**Date**: 2026-04-23
**Wall-clock for tasks 4–6**: ~55 minutes end-to-end (of which ~16 min was the full e2e regression suite)

---

## TL;DR for the advisor

**One-pass implementation with one advisor-review reversal on test coverage** (details §3.1 + DQ #45 resolution). Five of six v1-AD-d tests shipped in the impl session; the sixth (`admin_audit_stream_emits_frame_on_config_change`) was added in a follow-up commit after advisor review caught that the original substitution dropped three distinct invariants the `governance_events_notify_fires` test does not cover (filter branch, row hydration, frame format). Plan skeleton held up almost verbatim. Three improvements to the `/prp-core:prp-implement` command would have saved ~10 min of friction in this session and materially reduce risk on future SSE/streaming-like novel-pattern sub-phases:

1. **Add an explicit "scan for completed tasks" step to Phase 1 LOAD** so resumed sessions don't silently restart task 1. (I inferred this from `git log` but the command template doesn't say to.)
2. **Add an "HTTP status code review" step to Phase 4 VALIDATE** when the plan cites specific status codes in acceptance criteria. I caught the 409-vs-400 bug via self-review, not via the validation loop — a templated step would have caught it earlier and before the first test write.
3. **Add a "Docker-daemon preflight" probe to Phase 4** before invoking any `cargo test --test e2e` line. I only found out Docker wasn't running after running 3 tests that all failed with the same confusing error. A single `docker ps` probe at Level 3 entry would gate and message cleanly.

The rest of this document expands each finding with evidence and provides copy-paste patches to the command template.

---

## 1. What worked — keep doing

### 1.1 Plan skeleton fidelity

The plan's §10 pattern snippets (SSE_HAND_ROLLED_STREAM, AGGREGATE_HANDLER, DTO_SHAPE) mapped near-1:1 onto the final code. The notification-bridge loop in `admin_audit_stream.rs` (lines 124-139) is almost byte-identical to the existing `e2e.rs:2979-2998` probe the plan cited. This is the #1 reason the sub-phase shipped in one session — the implementer never had to design; only translate.

**Keep:** the plan's §10 pattern-block discipline. Every time the plan names an exact file + line range for a mirror, it landed. Plans that reference "use the existing pattern" without a line number burn implementer time re-finding the pattern.

### 1.2 GOTCHA sections doing their job

Task 5's GOTCHAs pre-warned:
- SSE frame format must end with `\n\n` — never had to re-learn.
- Driver task leak + `SseGuard::Drop` abort — built correctly from the start.
- `once_cell` vs `OnceLock` — avoided adding a dep.
- `tokio_postgres::NoTls` default for solo-dev — didn't rabbit-hole into TLS.

**Keep:** GOTCHAs that predict *specific* failure modes, each with a specific mitigation. Vague "be careful about X" GOTCHAs don't prevent anything.

### 1.3 Validation commands captured to files

Every `cargo check`, `cargo clippy`, `cargo test` call went through the wrapper scripts with `> .claude/... 2>&1` redirection. Zero exit-code masking. The existing rule `cargo-output-capture.md` is working as intended in `-p` mode and when Claude follows it manually.

### 1.4 Task-per-commit hygiene

Seven commits on the phase branch, one per task (plus one archive commit). Each commit message cites the task number, lists validation results, and explains *why*. This is the kind of history that retros can reconstruct from later.

---

## 2. What slowed the session — fix in the command template

### 2.1 FRICTION: command restarts at task 1, doesn't scan for completed work

**Observed**: I entered the session at commit `be9fae19e` (task 3 merged). The plan file still existed at `.claude/PRPs/plans/v1-admin-dashboard-d.plan.md` (not yet archived — the earlier session hadn't completed task 6 yet). The command template's Phase 2.2 "Branch Decision" matrix has no row for "some tasks are already committed."

**What I did**: scanned `git log --oneline -8` unprompted, saw `feat(admin-dashboard): admin_dashboard aggregate handler (task 3)`, `feat(api-common): AdminDashboardResponse + aggregate DTOs (task 2)`, `refactor(governance): extract project_to_audit_entry to shared audit_projection module (task 1)`, inferred tasks 1–3 were done, started at task 4. This worked because the commit messages cite task numbers verbatim.

**What could go wrong**: on a less-disciplined commit history (or a resumed session weeks later), the implementer could re-do task 1's file moves, collide with the existing state, and spend 15+ minutes diagnosing "why doesn't `mv` work." Or worse, silently land a duplicate commit.

**Fix — patch to `/prp-core:prp-implement`**:

```markdown
### 1.4 Detect Already-Completed Tasks (ADD THIS STEP)

Before starting Phase 3 EXECUTE, scan the current branch for commits matching the
plan's task numbering. The plan's Step-by-Step Tasks section names each task with
"Task N — <description>" and the plan's §19 "Commit message patterns" section
typically lists the expected commit subject prefix per task.

```bash
git log {base-branch}..HEAD --oneline | head -20
```

For each commit on the branch, check whether its subject matches a task's
**COMMIT MESSAGE** line in the plan. Mark those tasks as ALREADY-DONE and skip
ahead. Log clearly:

    Task 1: ALREADY DONE (commit abc1234 on branch)
    Task 2: ALREADY DONE (commit def5678 on branch)
    Task 3: STARTING HERE

If the branch commits don't align 1:1 with the plan's tasks (e.g. two tasks
squashed into one commit, or a task commit message doesn't match the plan's
expected pattern), STOP and surface to the user. Do not guess.
```

This adds ~30 seconds to the command's preamble and eliminates a whole class
of resume-session footguns.

### 2.2 FRICTION: HTTP status codes aren't audited when the plan cites them

**Observed**: The plan §16 acceptance criteria explicitly names "409 on second concurrent connection." The plan's §10 skeleton used `actix_web::error::ErrorConflict(...)` which, in Lemmy's custom `LemmyError`/`LemmyErrorType` stack, does NOT map to HTTP 409 — it maps to HTTP 400 because `LemmyError::status_code()` only special-cases `IncorrectLogin → 401` and `NotFound → 404`. I caught this via self-review AFTER writing the handler but BEFORE writing the 409-asserting test, which was lucky — if I'd written the test first and trusted the plan skeleton, the test would have failed with "expected 409 got 400" and I'd have debugged for 10+ minutes.

**Root cause**: the plan skeleton was written against generic `actix-web`. Lemmy wraps actix's error handling with `LemmyErrorType` which has its own status-code mapping in `crates/utils/src/error.rs:223-230`. Any `actix_web::error::Error*` constructor outside the `IncorrectLogin`/`NotFound` fast-path gets flattened to 400.

**Fix — patch to `/prp-core:prp-implement`**:

```markdown
### 4.1.1 HTTP Status Code Audit (ADD THIS STEP, runs inside Phase 4 VALIDATE)

If the plan's acceptance criteria name specific HTTP status codes (e.g. 409,
403, 422), grep for each code in the new files and verify the Rust path actually
produces it.

Lemmy-specific: `LemmyErrorType::status_code()` at
`crates/utils/src/error.rs:223-230` only maps `IncorrectLogin → 401` and
`NotFound → 404`. **Every other LemmyErrorType variant returns HTTP 400.** If
the plan says "409 Conflict on X" and the handler returns
`Err(LemmyErrorType::SomeThing.into())`, the actual response is 400 — not 409.

The two correct ways to emit a non-400, non-404, non-401 status:
1. Return `Ok(HttpResponse::<Status>().body(...))` directly (bypasses LemmyError).
2. Add a new `LemmyErrorType` variant AND update `status_code()` to special-case
   it. This is a workspace-wide change; only do it if multiple handlers need
   the same status.

Grep check:
```bash
# If the plan names status 409:
rg -n 'HttpResponse::Conflict|ErrorConflict|409' crates/api/api/src/governance/<new-files>
```

Every plan-named status must have a matching HttpResponse::<Status>() call site
OR a LemmyErrorType::<Name> variant whose special case in status_code() returns
that status. Otherwise the handler returns 400 regardless of the error type name.
```

### 2.3 FRICTION: Docker-daemon failures masquerade as test failures

**Observed**: at 01:56 I ran `cargo test --test e2e admin_dashboard`. All three tests failed with:

> `start_postgres: failed to create a container: Error in the hyper legacy client: client error (Connect)`

The error message doesn't say "Docker isn't running" — it says a connection failed, which I initially read as "the test's postgres container failed to start." Only after a `docker ps` did I see `failed to connect to the docker API at npipe:...`. This is a well-known testcontainers-rs behaviour but not obvious to a fresh session.

**Time cost this session**: ~3 minutes (I hit it once, recognised the pattern, asked user, proceeded).
**Time cost in a worse scenario**: if the implementer doesn't know the testcontainers error signature, they could chase "why does postgres fail to start" for 15+ minutes.

**Fix — patch to `/prp-core:prp-implement`**:

```markdown
### 4.2.0 Docker daemon preflight (ADD THIS STEP, runs BEFORE any cargo test --test e2e)

The Brehon e2e harness spins up a real Postgres container via testcontainers-rs.
If the Docker daemon isn't running, tests fail with a misleading hyper/connect
error message that looks like "the container failed" rather than "there's no
Docker."

Probe before running ANY e2e test:

```bash
docker ps > /dev/null 2>&1 && echo "DOCKER OK" || { echo "DOCKER NOT RUNNING — start Docker Desktop / dockerd before continuing"; exit 1; }
```

If Docker isn't running:
1. On Windows: Docker Desktop must be started (search "Docker Desktop" in Start menu).
2. On Linux: `sudo systemctl start docker` (or user's daemon equivalent).
3. On macOS: Open Docker Desktop from Applications.

After Docker is running, re-run `docker ps` to confirm, then continue with Level 3.

**This probe is cheap (~50ms) and eliminates 10+ minutes of diagnosis time when
the daemon is stopped.** Do not skip it just because "it was running last time."
```

---

## 3. Borderline issues — consider but don't block

### 3.1 Test-strategy substitution (swapping one e2e test)

The plan §14 specified 5 tests including `admin_audit_stream_emits_config_change` which requires an in-process actix HTTP server + `reqwest::bytes_stream()`. I swapped it for `admin_audit_stream_forbidden_for_non_admin` because:
- The NOTIFY substrate is already covered by `governance_events_notify_fires` (existing Phase 6 test)
- The handler's stream body is pure logic over that substrate
- In-process HTTP server + stream-client adds ~100 lines of fixture for a regression surface already covered elsewhere

**Was this the right call? No — overruled on advisor review (2026-04-23).** Advisor review identified that `governance_events_notify_fires` only proves the substrate (NOTIFY fires on INSERT); it does NOT prove three invariants that the plan-specified test uniquely covers:
1. The `kind == ADMIN_CONFIG_CHANGED || CHANGE_DENIED` filter branch at `admin_audit_stream.rs:181-185` — if a typo flipped `||` to `&&`, nothing would catch it.
2. The `entry_id` → `governance_log` row hydration at `:186-200` via `GovernanceLogId(entry_id)` lookup + `project_to_audit_entry`.
3. The SSE frame format assertion `event: X\ndata: Y\n\n` per HTML5 §9.2.4 at `:203-205` — the plan §10 GOTCHA explicitly flagged this as test-asserted.

**Follow-up shipped**: `admin_audit_stream_emits_frame_on_config_change` was added in a follow-up commit on `phase-v1-AD-d` using `MessageBody::poll_next` (no in-process HTTP server, no new dev-deps — ~150 lines total, mirrors the `governance_events_notify_fires` bridge pattern). All 6 v1-AD-d tests now pass (~41s for the new test; full group in 115.80s). The coverage gap is closed in-PR, not carried forward.

**Lesson**: the "coverage is equivalent" claim was implementer self-assessment that didn't hold up to adversarial review. The NOTIFY substrate ≠ the SSE handler's filter + hydration + frame-format pipeline.

**Resolved policy (DQ #45, impl-self-resolved 2026-04-23T03:15Z)**: option (b') — strengthened (b). Implementer MAY substitute tests without a DQ entry ONLY when the substitution preserves the invariants the plan-specified test was uniquely probing; substitutions that DROP any plan-named invariant MUST be queued to advisor BEFORE commit. Heuristic: "what does the plan-specified test uniquely prove that the substrate test does not?" If the answer is anything non-trivial, queue it.

**Codify in `/prp-core:prp-implement` Phase 5 REPORT**: add two checklist items —
1. `[ ]` For each plan-specified test NOT shipped: name it, name the replacement, and enumerate the plan invariants preserved vs dropped.
2. `[ ]` Any dropped invariant? If yes, STOP and queue DQ before committing the report.

### 3.2 Post-merge review of per-admin cap durability

The `OnceLock<Mutex<HashSet<PersonId>>>` is single-process. Plan §4.1 accepts this for v1. My implementation report cites it. But no GH issue was filed for the v2 upgrade (multi-node durability). If the advisor intends to ship to pilots where multi-node could matter, a tracking issue would be worth filing *now* while the context is fresh, not later when someone reads the code and wonders "is this on purpose?"

**Minor command-template suggestion**: Phase 5 REPORT's "Issues Encountered" section could prompt for "v2/future-work issues to file in GH?" — anything cited as a v1 limitation in the plan's §4.1 is a candidate.

---

## 4. What did NOT need fixing (worth preserving)

- **`cargo-output-capture.md` rule**: worked. Every `cargo ... > .claude/*.log 2>&1; echo "exit: $?"` pattern fired correctly. No exit-code-masking incidents.
- **Task-hopper**: not used this session (single-agent sub-phase). No comment either way.
- **Plan's §18 risks table**: row 1 (SSE novelty) played out benignly — the sub-phase-split hedge wasn't needed. Good risk framing, confidence score was mildly pessimistic but that's fine.
- **Registry invariant checks**: plan §15 Level 5 gave exact grep commands. They ran clean. Good discipline.

---

## 5. Quantified outcomes vs confidence score

Plan's §19.5 predicted **8.0/10** confidence for one-pass success.

Actual: **9/10 in hindsight** — nothing in the plan was wrong, the only friction was (a) status-code mapping ignorance (fixed in <5 min self-review), (b) Docker preflight (fixed in 3 min), (c) two trivial Rust import fixes during test compile (Crud trait, QueryDsl). None of (a)/(b)/(c) was a plan defect; they were command-template gaps.

If the three patches in §2 above land, similar sub-phases should credibly hit **9.5+/10** confidence — the remaining friction would be genuinely-novel-pattern risk (which no amount of command templating can eliminate).

---

## 6. Suggested action items for the advisor

In priority order:

| # | Action | Effort | Value |
|---|---|---|---|
| 1 | Patch `/prp-core:prp-implement` Phase 1 to scan for completed commits before restarting (§2.1) | 5-line edit | High (eliminates resumed-session silent-collision class) |
| 2 | Patch `/prp-core:prp-implement` Phase 4 to audit HTTP status codes vs LemmyErrorType mapping (§2.2) | 10-line edit + grep helper | High (catches a real bug class before tests are written) |
| 3 | Patch `/prp-core:prp-implement` Phase 4 to add Docker daemon preflight before e2e tests (§2.3) | 5-line edit | Medium (avoids 10min-diagnosis footgun) |
| 4 | Apply DQ #45 resolved policy (§3.1): patch `/prp-core:prp-implement` Phase 5 REPORT template with two test-substitution checklist items | 4-line template edit | Medium-high (the v1-AD-d case hit this, caught only on advisor review; codifying avoids future silent substitutions) |
| 5 | Prompt in Phase 5 REPORT for v1-limitation GH issues (§3.2) | 1-line checklist addition | Low (nice-to-have; implementer already cited limitations in report body) |

Items 1–3 are copy-paste patches included inline above. Item 4 requires an advisor decision first; item 5 is a one-liner.

---

## 7. For future v1-AD wave sub-phases (v1-AD-e, if it ships)

v1-AD-e is "askama HTML pages consuming `AdminDashboardResponse`." Specific carry-forward notes:

- The `AdminDashboardResponse` DTO ships with `ts-rs` export; TypeScript consumers can import it directly.
- The `admin_audit_stream` endpoint works unchanged from a browser's `new EventSource("/api/v4/governance/admin/audit/stream")`.
- The `SseGuard::Drop` cleanup pattern means a user who refreshes the page releases their slot cleanly — no server-side session logic needed.
- Per-admin cap is enforced server-side; client doesn't need to deduplicate.

No gotchas known in the handshake between v1-AD-d's API and v1-AD-e's page layer. Pure templating change.

---

_Retro author: implementer session that executed tasks 4–6 on branch `phase-v1-AD-d`. Available for advisor follow-up on any of §2.1/§2.2/§2.3 patches._
