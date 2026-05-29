# Plan — Author v1 data-model/API doc from live code, retire v0 `04`, repoint all live wiring

**Status:** READY TO EXECUTE (authored 2026-05-29; execute in a fresh session)
**Type:** Documentation + reference-repoint (NOT Rust code; no `crates/`/`migrations/`/`tests/` changes)
**Branch:** `governance-v0` direct (meta-work per `.claude/rules/phase-branch.md` "Direct on governance-v0" — docs + `.claude/commands` + `.pi` prompts; CR review would be net-noise on prose). NOT a phase branch, NOT a PR.
**Author role:** advisor session (canonical `brehon-fork` checkout) OR a dedicated subagent it dispatches. This is content-authoring, so if the advisor delegates, brief the subagent per `advisor-orchestrator.md` §6.3 (full context, trust-but-verify).

---

## 0. Why this plan exists (problem statement)

`CLAUDE.md` is injected into **every** session. Anything it links must be safe to act on at face value — a pointer that needs a "but it's only v0" caveat is itself a hazard, because the caveat doesn't reliably travel into a planning/impl subagent's head; the link does.

`docs/brehon-law-inspired-network/04-data-model-and-api.md` is a **v0-scoped** document (4 migrations, v0 enums, 11-endpoint route table, "Aggregation rules (v0, deliberately simple)"). It is **silent on ~18 additional v1 migrations** and the v1 schema/API that actually shipped (rule-set versions, sponsor-allowlist, sponsor-liability grace window + `CaseStatus::SponsorLiabilityPending`, `person.membership_state`, jury-mechanics columns/enums, appeals-v1 columns, reputation-event v1 columns, federation-inbound tables). Yet the PRP command corpus calls `04` "authoritative table/struct/enum/DTO definitions" and instructs planners to verify field shapes against it.

**Net effect:** every planning/impl session is pointed at a v0 doc as if it covered v1 — the exact "historical reference confusing ongoing implementation" the user flagged.

**User decisions (2026-05-29, locked):**
1. **Build a v1 data-model/API doc FROM LIVE CODE** (`migrations/` DDL + `crates/db_schema/src/source/governance/` Diesel models + enums + route registrations), cross-checked against PRDs — NOT assembled from PRDs (design intent drifts from ship reality; briefs already cite phantom `04 §17`/`§8.4` sections that don't exist).
2. **Delete `04` entirely** once the v1 doc exists — subject to the Bucket-C history-rewrite constraint in §3 below (the user's "delete" intent applies to the live doc + live wiring; frozen historical records are NOT rewritten — see §3 RESOLVED-AT-AUTHORING decision).
3. **Repoint the PRP command corpus + CLAUDE.md together** in the same pass — the PRP commands are where v0 `04` actually misdirects implementation.

---

## 1. The new document

**Path:** `docs/brehon-law-inspired-network/04-data-model-and-api.md` — **REUSE THE SAME PATH.**

> **AUTHORING DECISION (resolved 2026-05-29):** "Delete `04` and create a v1 doc" is best executed as **rewrite-in-place at the same path**, NOT new-path + delete-old. Rationale:
> - ~80 files link `docs/brehon-law-inspired-network/04-data-model-and-api.md`. If we keep the path, **Bucket C historical links stay valid** (they point at "the data-model doc" — now the v1 one — which is acceptable for frozen records; the alternative is rewriting 33 historical files = forbidden history-rewrite).
> - Bucket A/B links keep working with zero path churn; we only edit their *surrounding prose* to drop v0-scoping language, not the path.
> - The v0 content is preserved as git history of this file (the v0 version is `04` @ `HEAD~` at execution time) + already-banner-marked `IMPLEMENTATION-PLAN-v0.md` carries the v0 narrative.
>
> This satisfies the user's "delete 04 entirely" intent in substance: **the v0-scoped content is gone from the live tree**, replaced by current content, at the path everything already points to. If the user on review insists on a literal filename change (`04-data-model-and-api-v1.md` + `git rm` the old), that's a follow-up — but flag the 33-historical-link breakage cost first.

**Header (replaces the v0 "Status: stable starting point" framing):**

```markdown
# 04 — Data Model & API

**Audience:** Backend engineers (daily-driver reference)
**Status:** LIVING — current as of <ship-state at execution>; reflects v0 base + all merged v1 schema. Derived from live code (migrations/ + crates/), not design intent. Remaining v1 lanes (RT-r4/r5, quality-r2) update this doc as they ship.
**Source of truth:** the `migrations/` DDL + `crates/db_schema/src/source/governance/*.rs` Diesel models + route registrations are authoritative; this doc is their readable synthesis. On any discrepancy, CODE WINS — fix the doc.
```

**Structure** — mirror the v0 `04`'s proven section layout (it's a good shape; planners know it), extended to current reality:

| § | Content | Live source to derive from |
|---|---|---|
| 1. Migrations | ALL ~22 governance migrations in dependency order (not just the v0 four). Group: v0 core (enums/core/jury/reputation/log/config/pseudonym), then v1 (membership_state, federation_attestations, rule_set_versions, sponsor_allowlist + r1 extension, jury_mechanics enums+columns, appeals-v1, sponsor-liability variants+grace, reputation-event-v1, federation-inbound-v1). | `migrations/2026-*` dirs (22 governance migrations confirmed at authoring) — read each `up.sql` |
| 2. Enums | Current enum set incl. v1 additions: `CaseStatus` (+ `SponsorLiabilityPending`, + any AdminReview), jury-mechanics enums, `appeal_requester_role`, sponsor-liability variants, jury-constraint-relaxation-reason. | `crates/db_schema/src/source/governance/` enum defs + `add_*_enums` migrations |
| 3. Diesel models | All 25 governance model files (v0 had ~16). New since v0: `federation_inbox_dropped_log`, `federation_inbox_nonce`, `federation_peer`, `jury_constraint_violation_log`, `remote_moderation_label`, `rule_set_version`, `sponsor_allowlist`, + v1 columns on existing models (`reputation_event` v1 cols, `appeal` v1 cols, `moderation_case` jury-mechanics cols). | `crates/db_schema/src/source/governance/*.rs` (25 files, listed in §4 below) |
| 4. Read models | db_views crates — current query set (v0 had governance_case/jury_queue/reputation/governance_modlog; verify no v1 additions/changes). | `crates/db_views/{governance_case,governance_modlog,jury_queue,reputation}/src/` |
| 5. API DTOs | Current request/response types incl. v1 endpoints (revoke_endorsement, admin_config, admin_create_rule_set, admin_sponsor_allowlist when RT-r4 lands, flag-bad-faith, appeal-vote). | `crates/api/api_common/src/governance.rs` |
| 6. Handlers | Current handler set. | `crates/api/api*/src/governance/` + **LOCATE the routes file first** (see §2 GOTCHA — `crates/api/routes/src/governance.rs` did NOT exist at authoring; find where routes actually register) |
| 7. Routes | Current route table — v0 11 endpoints + v1 admin/governance endpoints. | the located routes file |
| 8. Aggregation rules | Current rules — NOT "v0 deliberately simple"; reflect v1 quorum/threshold/deadlock/grace-window/sponsor-liability-pending lifecycle (the `submit_jury_vote` 9-step shape per jury-mechanics PRD §9.1). | `crates/api/api/src/governance/submit_jury_vote.rs` |
| 9–12. Federation | Current AP objects/activities/inbox-outbox/server-wiring incl. inbound (federation-inbound lane shipped a–e). | `crates/apub/**/governance/` + `crates/server/src/governance.rs` |
| 13. Entry-kind registry | Cross-link to `.claude/rules/governance-log-entry-kind-registry.md` as the authoritative entry-kind list (do NOT duplicate it — point at it). Also cross-link `governance-log-kinds-jsonl.md` (the harness-observability sidecar, NOT product). | the registry rule file |
| 14. Cross-references | Update to current docs. | — |

**The 25 governance Diesel model files** (authoritative §3 list, confirmed at authoring):
`actor_pseudonym, appeal, case_evidence, endorsement, federation_attestation, federation_inbox_dropped_log, federation_inbox_nonce, federation_peer, governance_config, governance_log, jury_assignment, jury_constraint_violation_log, jury_pool, jury_vote, moderation_case, public_case_log, redaction, remote_moderation_label, remote_sanction_notice, reputation_event, reputation_snapshot, rule_set_version, sanction, sponsor_allowlist, surety`.

---

## 2. Authoring method (CODE-FIRST — load-bearing per user decision #1)

1. **Read every governance `up.sql`** under `migrations/2026-*` (22 dirs). Extract: table name, columns + types, constraints, indexes, FKs, enum types created. This is the table/enum ground truth.
2. **Read every `crates/db_schema/src/source/governance/*.rs`** (25 files). Extract: struct fields + Diesel types, `*InsertForm` shapes, enum `#[derive(DbEnum)]` variants. Cross-check against the SQL — flag any divergence (the doc records the Rust model shape as the API-facing contract).
3. **Read `crates/api/api_common/src/governance.rs`** for DTOs.
4. **LOCATE the routes registration** (GOTCHA: `crates/api/routes/src/governance.rs` did not exist at authoring — `grep -rn "api/v4/governance" crates/api/` or check `crates/api/routes/src/lib.rs` for where the governance route tree mounts). Derive the current route table from reality.
5. **Read `submit_jury_vote.rs`** for the current aggregation/lifecycle shape (§8).
6. **Cross-check against PRDs** (`.claude/PRPs/prds/v1-*.prd.md`) — but PRDs are the *secondary* check; where PRD and code disagree, **CODE WINS** and the doc reflects code. Note any PRD-vs-code drift found in a `kind: "log"` DQ entry for later PRD reconciliation (do NOT edit PRDs in this pass).
7. **Fan out the reads with subagents** (22 migrations + 25 models + DTOs/routes is too many files to read serially in one context) — see §2a for the dispatch shape + mandatory model selection. The advisor/author session **synthesizes the doc itself**; subagents return structured extracts (column/type/constraint/variant lists), never prose.

**Trust-but-verify (per `feedback_verify_files_with_read.md` + §6.3):** after authoring, spot-check 3–4 tables by re-reading their `up.sql` against what the doc claims. A schema doc that's wrong is worse than an honestly-labelled v0 doc. **This verification is non-negotiable BECAUSE the extraction is fanned out to subagents** — the synthesizing session owns correctness, not the extractors (§2a).

### 2a. Subagent dispatch & model selection (MANDATORY — per `feedback_subagent_model_and_effort.md`)

The project's canonical rule for ad-hoc `Agent`-tool subagents in this repo: **set `model: "opus"` on every Agent call AND include an explicit max-effort directive in the prompt** (`"Run at maximum effort — deepest reasoning, most thorough exploration"`). Default to lower-capability models produces shallow work out of step with the parent's rigour. This is a HARD default, not a suggestion — cite the lesson in the dispatch.

Apply it here as follows:

| Subagent work | `subagent_type` | `model` | Effort directive | Rationale |
|---|---|---|---|---|
| **Migration extraction** (read N `up.sql`, return structured table/column/constraint/enum lists) | `Explore` | **`opus`** | yes — "max effort; report EVERY column, type, constraint, index, FK, enum value verbatim — omissions corrupt the schema doc" | Accuracy is load-bearing (a wrong schema doc is worse than none) AND Explore reads *excerpts not whole files* — so do NOT tier down. The `feedback_subagent_model_and_effort.md` "scripted/bright-line" exception does NOT apply (this is not a scripted dispatcher with encoded refusals; it's open-ended extraction where shallowness silently drops fields). |
| **Diesel-model extraction** (read N `*.rs`, return struct fields + Diesel types + `InsertForm` shapes + `DbEnum` variants) | `Explore` | **`opus`** | yes — same verbatim-completeness directive | Same reasoning. Cross-checking SQL-vs-Rust divergence is judgment work. |
| **Route/DTO/aggregation reads** (locate routes file, read `submit_jury_vote.rs` lifecycle) | `Explore` or `general-purpose` | **`opus`** | yes | Locating the routes registration (it's NOT at the path `04` claims) + reading the 9-step lifecycle is investigative, not mechanical. |
| **Doc synthesis + PRD cross-check + final verification** | (the advisor/author session itself — NOT delegated) | n/a (inherits the Opus parent session) | n/a | The synthesizing session owns the doc and its correctness. Per `feedback_subagent_model_and_effort.md`, never delegate the judgment core to a cheaper tier. |

**Dispatch discipline:**
- **Parallel, single message** — when fanning out the migration reads and the model reads, issue them as multiple `Agent` calls in ONE message so they run concurrently (per the lesson's "prefer multiple agents in a single message" + `feedback_parallel_cohort_dispatch` spirit). Don't serialise independent reads.
- **Brief each subagent like a colleague who just walked in** (per `advisor-orchestrator.md` §6.3): tell it the goal (extracting ground-truth schema for a doc rewrite), the exact files/glob, the output shape wanted (a structured list, not prose), and that completeness is the priority (every field, no summarising-away). Terse prompts produce shallow extracts.
- **Trust-but-verify the extracts** (§2 step + §6.3): the synthesizing session spot-re-reads a sample of `up.sql` against what the subagent returned before trusting it. Subagent reports describe what they *intended* to extract, not necessarily what's in the file.
- **Cost note:** Opus-on-extraction is more expensive than Sonnet, but per `feedback_brehon_autonomy_goals` this project optimises reliability over token-thrift on correctness-critical work, and a schema doc is exactly that. The lesson's tier-down exception is explicitly scoped to *scripted dispatchers with bright-line refusals* (e.g. `bm-*` verbs) — this task is neither, so no tier-down.

---

## 3. Repoint inventory (the blast radius — bucketed)

`04-data-model-and-api.md` is referenced in **~80 files** (full `grep -rl "04-data-model"` at authoring). Because §1 REUSES the same path, **only prose that v0-scopes the reference needs editing** — the path links themselves stay valid. Three buckets:

### Bucket A — LIVE WIRING (edit surrounding prose to drop v0-scoping; path unchanged)

Confirmed at authoring (file → ref count):

| File | Refs | Action |
|---|---|---|
| `CLAUDE.md` | 1 | The "Where to look next" schema line (already edited this session to "canonical schema/API"). With the v1 rewrite, the line is now TRUE as-is — verify wording: "Schema / DTO / route reference: `04-data-model-and-api.md` (current; tables, enums, routes — derived from live code)." No caveat needed. |
| `AGENTS.md` | 1 | Same check — drop any v0-scoping language. |
| `.claude/brehon-reference.md` | 1 | Line 37 P0 table row already says "Tables, enums, Diesel structs, DTOs, routes" — verify it doesn't call it v0-specific elsewhere. |
| `.claude/commands/prp-core/prp-plan.md` | 11 | **Primary repoint target.** All "authoritative field definitions / §3 Phase N" language assumes v0 phase structure. Reword: `04` is the current schema reference; v1 work extends it; phase-N references become "the relevant § N table/DTO". |
| `.claude/commands/prp-core/prp-prd.md` | 5 | Drop "already decomposed to tasks in `04`" v0 framing. |
| `.claude/commands/prp-core/prp-implement.md` | 3 | "verify table shape matches `04`" — now TRUE for v1; verify no v0-only caveat. |
| `.claude/commands/prp-core/prp-review.md` | 2 | "indexes match `04 §1.5`" — re-verify the § number against the rewritten doc. |
| `.claude/commands/prp-core/prp-codebase-question.md` | 1 | One-line; verify. |
| `.pi/prompts/prp-plan.md` | 11 | **`.pi/` IS A VERBATIM MIRROR of `.claude/commands/prp-core/`** (identical counts: 11/11, 5/5, 3/3, 2/2, 1/1). Apply the SAME edits to both copies. Per `CLAUDE.md` "Custom orchestration disclaimer" + the pi/claude dual-harness contract. |
| `.pi/prompts/prp-prd.md` | 5 | mirror of claude copy |
| `.pi/prompts/prp-implement.md` | 3 | mirror |
| `.pi/prompts/prp-review.md` | 2 | mirror |
| `.pi/prompts/prp-codebase-question.md` | 1 | mirror |
| `.pi/PROJECT_CONTEXT.md` | 1 | verify scoping language |

**`.pi/` mirror discipline (load-bearing):** the section *headings* and path references in `.pi/prompts/*` must stay in lockstep with `.claude/commands/prp-core/*` (dual-harness contract — `.pi/` is the pi-harness copy of the same commands). Every edit to a `.claude/commands/prp-core/X.md` ref gets the identical edit in `.pi/prompts/X.md`. After editing, `diff <(grep -n 04-data-model .claude/commands/prp-core/prp-plan.md) <(grep -n 04-data-model .pi/prompts/prp-plan.md)` should show matching ref lines.

### Bucket B — ACTIVE SIBLING DESIGN DOCS (update cross-refs as needed; path unchanged)

| File | Refs | Action |
|---|---|---|
| `docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md` | 60 | Already SHIPPED-bannered this session. Its 60 refs to `04` are part of the v0 plan narrative — **leave as-is** (they're correct in the v0 context; the banner tells readers the file is historical). Do NOT mass-rewrite. |
| `05-mvp-and-delivery-plan.md` | 7 | These are v0 MVP-scope cross-refs. Leave unless actively misleading; this is a v0 design doc too. |
| `99-decisions-and-open-questions.md` | 8 | ADR/OQ cross-refs — leave (living doc, refs are to specific § that the rewrite preserves by keeping section numbering compatible where possible). |
| `02-domain-model.md` (4), `03-architecture.md` (3), `00-README.md` (3), `06-security-and-threat-model.md` (2), `AGENT-PROMPT-*.md` (3), `V2/messaging.md` (2), `governance-log-kinds-jsonl.md` (1), `expert-review-suite/00-README.md` (1) | — | Spot-check each: if the ref points at a § that the rewrite renumbers/removes, fix that one ref. Otherwise leave. **Keep §-numbering stable in the rewrite where possible** to minimise this. |

### Bucket C — HISTORICAL RECORDS (DO NOT TOUCH — frozen)

**33 files** under `.claude/PRPs/**` (completed plans, ralph-archives, retros, reports, DQ archives, old briefs) + `.github/ai-review-prompts/` reference `04`. These are frozen audit records.

> **RESOLVED-AT-AUTHORING (history-rewrite constraint):** The user said "delete `04` entirely." Literal deletion + `git rm` would break 33 frozen historical links and require rewriting them — which violates the project's append-only / no-history-rewrite discipline (`decision-queue.md` "Forward-only consistency", retro-as-record convention). **The §1 same-path-rewrite resolves this:** the path keeps resolving, so historical links remain valid (they point at "the data-model doc" — acceptable for a frozen record to resolve to the current version). **Bucket C is LEFT ENTIRELY UNTOUCHED.** If the user on review wants literal filename deletion, surface the 33-file breakage cost and get explicit sign-off before rewriting any historical record.

---

## 4. Execution sequence

1. **Pre-flight:** `git fetch origin governance-v0`; confirm clean tree; confirm CWD = canonical `brehon-fork` on `governance-v0` (per multi-lane §Layout, meta-edits happen here).
2. **Fan out the source-extraction subagents** (§2a) — dispatch the migration-read + model-read + route/DTO-read subagents **as parallel `Agent` calls in a single message, each with `model: "opus"` and a max-effort + verbatim-completeness directive** (MANDATORY per `feedback_subagent_model_and_effort.md`; see §2a table). Collect their structured extracts.
3. **Author the doc** (§2 method) — the advisor/author session synthesizes `docs/brehon-law-inspired-network/04-data-model-and-api.md` in place from the §2 extracts (NOT delegated — the synthesizing session owns correctness). Bulk of the work (~870-line doc; budget accordingly).
4. **Verify the doc** (§2 trust-but-verify spot-check — re-read a sample of `up.sql` against what the subagents extracted AND what the doc claims; the extraction was fanned out, so this verification is non-negotiable).
5. **Bucket A repoints** — edit live wiring prose, both `.claude` and `.pi` copies in lockstep.
6. **Bucket B spot-fixes** — only § refs that the rewrite invalidated.
7. **Bucket C** — confirm untouched (`git status` shows no `.claude/PRPs/**` history files modified).
8. **`.pi` mirror diff check** (§3 Bucket A discipline).
9. **Commit** — single `docs:` commit (or split doc-rewrite from repoints if cleaner). Subject e.g. `docs(data-model): rewrite 04 from live code (v0 base + all v1 schema); repoint PRP corpus + CLAUDE.md`. Direct to `governance-v0`. Co-author trailer per harness.
10. **Do NOT push** unless user asks.
11. **Optional `kind: "log"` DQ** for any PRD-vs-code drift found in §2 step 6 (for later PRD reconciliation — not this pass).

---

## 5. Out of scope (do NOT do in this pass)

- **No PRD edits.** PRDs are design intent; reconciling them to code is a separate task. Record drift in a `kind: "log"` DQ only.
- **No Rust code changes.** This is docs + command-prose only.
- **No `migrations/` or `crates/` edits.** Code is the source of truth being read, not changed.
- **No history-rewrite** of Bucket C frozen records (see §3 RESOLVED).
- **No new `04-*-v1.md` filename** unless user overrides §1 same-path decision (flag breakage cost first).
- **No phase branch / PR.** Meta-work, direct to trunk.

---

## 6. Acceptance criteria

- `docs/brehon-law-inspired-network/04-data-model-and-api.md` documents all 22 governance migrations + 25 Diesel models + current enums/DTOs/routes/aggregation, derived from code, header marked LIVING with "CODE WINS" source-of-truth note. No "v0 deliberately simple" / "stable starting point" framing remains.
- Spot-check: 3–4 tables in the doc match their `up.sql` verbatim (columns/types/constraints).
- **Subagent dispatch followed §2a:** every source-extraction `Agent` call used `model: "opus"` + a max-effort/verbatim-completeness directive (per `feedback_subagent_model_and_effort.md`); the doc synthesis + verification was NOT delegated to a cheaper tier; extraction subagents ran in parallel (single message). No tier-down was applied (the lesson's scripted-dispatcher exception does not cover open-ended schema extraction).
- Bucket A live wiring (6 `.claude` files + 6 `.pi` files + CLAUDE.md + AGENTS.md + brehon-reference.md) no longer v0-scopes `04`; `.claude` and `.pi` copies identical.
- Bucket C (33 historical files) byte-unchanged.
- CLAUDE.md schema pointer is TRUE at face value with no caveat (the user's core requirement: injected-everywhere links must be current).
- Commit on `governance-v0`, not pushed.

---

## 7. Open question for the executing session to confirm with user (gate before commit)

The §1 decision (rewrite-in-place at the same `04` path vs. literal `04-v1.md` + `git rm` old) substantively delivers "delete v0 `04`" while avoiding 33-file history-rewrite. **Surface this to the user at the start of execution** — confirm same-path rewrite is the accepted interpretation of "delete entirely" before investing the ~870-line authoring effort. If they want a literal new filename, the plan still holds but Bucket B/C link-fixing grows (and the history-rewrite constraint must be re-litigated).
