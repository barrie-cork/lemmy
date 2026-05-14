# v1 code-quality audit — 2026-05-14

## §0 Context for reviewer

Single-pass audit of all Brehon-authored code on `governance-v0` against three lenses (Lemmy parity, Rust idioms, long-term maintainability) plus a fourth divergence-tracking axis. Read-only. No code modifications. No DQ writes. No Junior dispatch. Single-purpose advisor session — goes idle on completion. Concurrent-session discipline per PMD #302: another advisor session is parked on the `v1-ship-1` plan-approval gate.

**Downstream sequence (informational — not executed in this session):**

- **Step 4 (user):** Review §4 ranked refactor backlog. Decide each finding's tier: fix-before-next-phase / fix-during-relevant-sub-phase / accept-and-document / intentional-divergence-accept.
- **Step 5 (strict gate per user 2026-05-14):** Every fix-before-next-phase finding ships as its OWN refactor PR before any v1 PRD work resumes — even if the refactor is in code untouched by the next PRD. Refactor commits land in clean, separate PRs (`phase-v1-refactor-<area>`). NO bundling refactor work with feature PRDs.
- **Step 6 (planning resumption):** Once all fix-before-next-phase refactors are merged into `governance-v0`, resume v1 PRD implementation. Every PRD planning Junior task MUST read this audit as required reading. If a PRD's tactical phases would re-create an anti-pattern flagged here, the planner MUST cite the audit and either (a) plan the refactor as a pre-condition phase, or (b) explicitly accept the divergence in the plan body with rationale. No silent re-creation of flagged anti-patterns.
- **Going forward:** This audit is the baseline. Every subsequent retro checks whether the shipped phase introduced new instances of the flagged anti-patterns. If yes, retro logs as regression and the next phase's planning gets the constraint "avoid X" citing this audit + the retro.

---

## §1 Method

**Subagents dispatched:** 5 parallel `Explore` subagents, one per scope area:

- **Explore A** — `crates/api/api/src/governance/**` (30 .rs files; ~13 KLOC).
- **Explore B** — `crates/api/api_crud/src/governance/**` (5 .rs files).
- **Explore C** — `crates/db_schema/src/source/governance/**` (22 .rs files) + Brehon-added entries in `crates/db_schema_file/src/{enums.rs,schema.rs}`.
- **Explore D** — `migrations/2026-04-*` + `migrations/2026-05-*` (29 migration dirs, 58 .sql files).
- **Explore E** — `crates/server/tests/e2e.rs` Brehon-authored fixtures + test fns + `crates/tools/seed_founders/` + `crates/apub/activities/src/governance/**` (5 .rs files).

**Total scope:** 122 Brehon-authored files in scope (per `git ls-files` count across the 5 paths). Approximate Brehon-authored LOC (rough, includes blanks/comments):
- API handlers (Explore A+B): ~14,400 LOC across 35 files. Top three: `config.rs` (3272), `admin_config.rs` (1414), `submit_jury_vote.rs` (1019).
- DB source models (Explore C): ~3,500 LOC across 22 files.
- Migrations (Explore D): ~4,500 LOC across 58 .sql files.
- Tests + tool + apub (Explore E): ~2,500 LOC of Brehon-authored fixtures (within the 8945-line `e2e.rs`) + `seed_founders/src/main.rs` (~330 LOC) + 1366 LOC across 5 apub governance files.

**Total Brehon-authored: ~25,000 LOC.**

**Upstream drift baseline (captured pre-audit):**

```
LOCAL_MAIN     = d1975776a491bc2f4085139bb50cbf89fe6d5e54
UPSTREAM_TIP   = 17288bed2dc8fda0a8a61050d92c8cf9ed6aa2a0
DRIFT          = 24 commits behind upstream/main
```

24 commits behind upstream/main; below the 200-commit fast-forward threshold; no user action needed pre-audit. Every Axis-4 divergence finding below is anchored to upstream `17288bed2` — when Explore subagents read upstream reference files (e.g. `post/like.rs`), they did so via `git show upstream/main:<path>`, not via local main.

**Synthesis pass (this session, after subagent returns):** ~165 raw findings reduced to 64 distinct findings after dedup, frequency-aggregation, and verification. Three subagent claims were verified directly and one was invalidated:

| Subagent claim | Verification | Result |
|---|---|---|
| Three migrations missing `down.sql` (Explore D #32, #33, #34) | `ls migrations/2026-04-2{1,2}-*/` | **INVALIDATED** — all three have down.sql. Not included in this report. |
| `governance_config.rs` is the only governance source model missing `Eq` derive | `grep "Eq" crates/db_schema/src/source/governance/*.rs` | **CONFIRMED** — 20 of 22 derive `Eq`; `governance_config.rs:10` is the lone outlier; `reputation_event.rs:13` also derives. Finding stands. |
| Case C error-type mixing across e2e.rs Brehon fixtures (Explore E #1) | `grep "async fn .* -> (Result\|LemmyResult)" crates/server/tests/e2e.rs` | **CONFIRMED + amplified** — 20+ Brehon test fns return `Result<(), Box<dyn Error>>`; only one helper (`admin_config_fixtures::bootstrap`) returns `LemmyResult`. Severity escalated to critical. |
| `create_report.rs:130-232` SELECT-then-write not wrapped in `run_transaction` | `grep "run_transaction" crates/api/api_crud/src/governance/create_report.rs` | **CONFIRMED** — zero `run_transaction` calls in the file. Finding stands as TOCTOU race risk. |

The invalidated finding underscores a recurring lesson: Explore subagents under context pressure occasionally hallucinate negatives (missing files, missing fields) without re-grepping. Synthesis verification caught it. Other findings depend on relative-positioning claims (e.g. file:line citations) that the parent session sampled but did not re-grep for every one of 165 raw findings — readers should sample-verify before acting on high-severity items.

---

## §2 Headline counts

Lens distribution × severity tier (severity is the audit's own ordinal: **critical** = correctness/safety risk; **major** = clear quality-fail with widespread impact; **medium** = quality-fail localized or low frequency; **minor** = style/documentation; **positive** = exemplar noted, no fix). Lens 4 = Axis-4 (intentional-divergence-accept).

| Lens | Critical | Major | Medium | Minor | Positive | Total |
|---|---|---|---|---|---|---|
| Lens 1 — Lemmy parity | 1 | 4 | 8 | 5 | 0 | 18 |
| Lens 2 — Rust idioms | 2 | 5 | 12 | 6 | 0 | 25 |
| Lens 3 — Maintainability | 1 | 7 | 10 | 6 | 0 | 24 |
| Lens 4 — Axis-4 (divergence-accept) | 0 | 0 | 0 | 0 | 4 | 4 |
| **Findings unique (deduped across lens)** | **3** | **10** | **22** | **15** | **4** (positive exemplars) | **64** |

(Findings often hit multiple lenses; the unique-row count is post-dedup.)

**Three critical findings (drive §4 fix-before-next-phase tier):**
1. **Case C error-type mixing in `e2e.rs`** — 20+ Brehon test fns use `Result<(), Box<dyn Error>>`, helpers mostly match but one (`bootstrap`) returns `LemmyResult`. Adding a single Lemmy-native call to a Box-typed test triggers E0277 cascade. Violates `feedback_lemmy_error_no_std_error.md` Case C hard refusal.
2. **TOCTOU race in `create_report.rs`** — SELECT-then-UPDATE/INSERT not wrapped in `run_transaction`; concurrent reports on the same target can slip between read and write. Sibling files (`create_endorsement.rs`, `revoke_endorsement.rs`) use `run_transaction` for the same pattern — internal-inconsistency violates Brehon's own convention.
3. **Test fixture duplication across 4+ phase-specific modules in `e2e.rs`** — `v1_jm_b_fixtures`, `v1_jm_e_fixtures`, `v1_sl_b_fixtures`, `v1_sl_c_fixtures` each define near-identical `seed_case`, `seed_jurors`, `seed_jury_eligible_snapshots`. ~70% code overlap. Adding one new helper requires updating 4 modules; bug fixes replicate 4×.

---

## §3 Detailed findings

Findings are grouped by scope area for navigability; within each area they are ordered by severity, then frequency. Each row has: location · lens(es) · Axis-4 tag · anti-pattern name · description · why · recommended fix · effort · frequency.

Severity is encoded in the leading badge: `[CRIT]` `[MAJ]` `[MED]` `[MIN]` `[POS]`.

Axis-4 tags: **quality-fail** / **divergence-from-lemmy** / **both** / **N/A** (where N/A = design/maintainability question, not defect).

### 3.A `crates/api/api/src/governance/` — 30 handler files

**3.A.1** `submit_jury_vote.rs:1-1019` · Lens 3 · Axis-4 N/A · **[MAJ]** monolithic 1019-line file with 9-step vote aggregation, appeal panel logic, sponsor-liability branching, federation orchestration, ~15 internal helpers · Lemmy convention: >600-line files split into submodules grouped by responsibility · A code-review of "what changed in vote-decision logic?" reads past 200+ lines of setup and appeal-specific branches · split into `submit_jury_vote/mod.rs` (handler + orchestration), `vote_tally.rs`, `appeal_panel.rs`, `sanction_decision.rs`; preserve handler visibility, make internal modules `pub(crate)` · **L** · 1 file affected, but reads as the single highest-effort backlog item.

**3.A.2** `admin_config.rs:1-1414` · Lens 3 · Axis-4 N/A · **[MAJ]** 1414-line file mixing 4 HTTP handlers + 7 impact-preview functions + DTO builders + comprehensive inline test suites; `admin_set_config` handler alone is ~400 lines with inlined JSON payload construction · Same Lens 3 rationale as 3.A.1 · `build_admin_config_changed_payload` (line 1389) is a 50+-line inline closure-style payload builder embedded in the handler · split into `admin_config/mod.rs` (handlers), `impact_analysis.rs` (7 fns), `payload.rs` (builders); keep tests near logic · **M** · 1 file, multiple split points.

**3.A.3** `config.rs:1-3272` · Lens 3 · Axis-4 N/A · **[MAJ]** 3272-line module with 8 typed accessors (`get_int`, `get_float`, `get_bool`, `get_text`, plus `_opt` variants) each containing near-identical cache-check-DB-fetch-match-fallback logic, ~200 lines of duplication · Lens 3 DRY violation: a cache-eviction-policy change requires editing 8 functions · extract generic `get_cached_or_fetch<T>` helper with type-level dispatch (associated types or trait bounds); typed accessors become thin wrappers preserving public API · **M** · 8 functions affected, single pattern.

**3.A.4** admin_*.rs cluster (`admin_assign_jury.rs`, `admin_close_case.rs`, `admin_config.rs`, `admin_dashboard.rs`, `admin_emergency_remove.rs`, `admin_reputation_stats.rs`, `admin_rule_sets.rs`, `admin_trigger_appeal_rejury.rs`) · Lens 3 · Axis-4 N/A · **[MED]** every admin handler repeats `is_admin(&local_user_view)?` followed by `let admin_pseudonym = actor_pseudonym_helper::get_or_create(&mut context.pool(), admin_id).await?` boilerplate · DRY: changing pseudonym allocation semantics requires editing 8 handlers · extract `get_admin_pseudonym(local_user: &LocalUserView, pool: &mut DbPool<'_>) -> LemmyResult<(PersonId, String)>` into a new `admin_common.rs` or extend `governance/mod.rs` · **S** · 7-8 files (frequency 8).

**3.A.5** `admin_audit_stream.rs:227,234` · Lens 2 · Axis-4 quality-fail · **[MAJ]** `.await.optional().ok().flatten()` chain swallows database errors — the `.ok()` between `.optional()` (returns `Result<Option<T>>`) and `.flatten()` reduces a query failure to `None` indistinguishable from a legitimate empty result · Lens 2: `.ok()` swallows errors; upstream `like.rs` uses `?` propagation or explicit `.map_err()` with context · replace with `await.optional()?` or `.map_err(|e| LemmyErrorType::Unknown(format!("governance_log query: {e}")))` · **S** · 2 sites in 1 file.

**3.A.6** `admin_rule_sets.rs:313`, `admin_dashboard.rs:317` · Lens 2 · Axis-4 quality-fail · **[MED]** `.and_then(|i| i32::try_from(i).ok())` on a config read returns `i64`; the `.ok()` swallows `TryFromIntError` so an out-of-range admin config silently becomes `None` · Lens 2: typed-conversion errors should surface, not be erased; sibling pattern in `create_endorsement.rs:266-275` uses `.map_err(|_| LemmyErrorType::Unknown(...))` correctly · replace with `.and_then(|i| i32::try_from(i).map_err(|e| LemmyErrorType::Unknown(format!("config value out of i32 range: {i}, {e}"))))` · **XS** · 2 sites in 2 files; also recurs at `create_report.rs:201` (see 3.B.4).

**3.A.7** 5 sites across `jury_common.rs:49,86`, `sponsor_liability_grace.rs`, `admin_rule_sets.rs`, `admin_audit_stream.rs` · Lens 2 · Axis-4 quality-fail · **[MED]** `.map_err(|_e| LemmyErrorType::Unknown("query failed".to_string()))` discards original Diesel error context, losing pool-exhaustion vs syntax-error vs connection-died distinction in server logs · Lens 2: error context preservation — upstream pattern is `?` propagation or contextualizing `.map_err()` that includes original error · replace each with `.map_err(|e| LemmyErrorType::Unknown(format!("<context>: {e}")))` or just `?` · **XS** per site, **S** in aggregate · 5 sites, 1 pattern.

**3.A.8** `submit_jury_vote.rs:138`, `admin_assign_jury.rs:110`, `admin_close_case.rs:41` · Lens 2 · Axis-4 divergence-from-lemmy · **[MED]** `let vote_data = data.clone()` before moving into the `run_transaction` closure; closure consumes `vote_data` once and never reuses original; clone is defensive-only · Lens 2: Rust move semantics over Clone when original is not needed; upstream avoids interim clone points by constructing forms directly · remove `.clone()`, move `data` directly into closure; if logging after tx needs fields, use selective borrow before tx · **M** · 3 sites, may require closure-capture restructure.

**3.A.9** `submit_jury_vote.rs:140-145` (context clone), and similar at `admin_assign_jury.rs` · Lens 2 · Axis-4 divergence-from-lemmy · **[MED]** `let context_for_tx = context.clone()` ("cheap Arc clone" per inline comment) before closure; canonical upstream pattern uses `&context` thread-through · Pragmatic divergence — Arc clone is genuinely cheap; current readability is fine · No change required for v0; flag as **acceptable divergence** if closure-lifetime restructure burden is not worth idiom gain · **M** if pursued, else accept · 2 sites.

**3.A.10** `redaction.rs:31-59` · Lens 2 · Axis-4 N/A · **[MIN]** `OnceLock<Regex>` with `.expect("valid regex")` for 3 static regexes; each call site annotated `#[expect(clippy::expect_used, reason = "static regex — infallible at startup")]` · Pattern is acceptable; `#[expect]` justifications are correct and specific; modern alternatives (compile-time regex macros) exist but are not load-bearing · No change required · **XS** if pursued · 3 sites, justified.

**3.A.11** `jury_common.rs:93` · Lens 2 · Axis-4 both · **[MIN]** `.try_into().unwrap_or(usize::MAX)` with inline comment claiming "BigInt can't exceed panel_size in practice; saturate instead of error" · Saturation is defensible but comment is ad-hoc rather than citing a specific invariant · reword comment to `// SAFETY: max_shared is bounded by panel_size_snapshot <= 100 per config CHECK constraint OQ-024 § …; saturation safe for mathematically-impossible case` · **XS** · 1 site.

**3.A.12** `admin_config.rs:1315,1338,1390` test code · Lens 2 · Axis-4 quality-fail · **[MIN]** three `.unwrap()`/`.expect()` calls in test-only assertions checking JSON payload structure, no `// SAFETY:` comment · Brehon workspace clippy denies unwrap in non-test code; in tests, justification comments are still a soft convention to aid review · add `// SAFETY: test-only; payload guaranteed by preceding json!() macro` at each site · **XS** · 3 sites.

**3.A.13** `submit_jury_vote.rs:585-610` (paraphrased — verify on read; estimated location of `+10/-5 JuryReliability` award block per v0 endpoint coverage report §2) · Lens 3 · Axis-4 N/A · **[POS]** decision-aggregation logic (`ALL_JURY_DECISIONS` iter + stable enum order + threshold gate + AdminReview fallback on deadlock) is well-commented and traces directly to PRD §6 — positive exemplar, no change · The 9-step structure is opaque due to file size (see 3.A.1) but each step's intent is documented · use as template when authoring future multi-step orchestration handlers · N/A · 1 site (exemplar).

### 3.B `crates/api/api_crud/src/governance/` — 5 CRUD handlers

**3.B.1** `create_report.rs:130-232` · Lens 1+2 · Axis-4 both · **[CRIT]** SELECT-then-UPDATE-or-INSERT branch with NO `run_transaction` wrapper — concurrent reports targeting the same entity can slip between read and write, causing duplicate case creation or lost weight accumulation; sibling files in the same directory (`create_endorsement.rs:136`, `revoke_endorsement.rs:143`) DO use `run_transaction` for analogous patterns — internal inconsistency · Lens 1: Lemmy CRUD convention requires atomicity for non-idempotent writes; Lens 2: TOCTOU race is a classic Rust safety lapse; the internal-inconsistency makes this a stronger finding than parity-only · wrap lines 130-232 in `conn.run_transaction(|conn| { async move { ... }.scope_boxed() })` capturing the entire SELECT-UPDATE-INSERT sequence · **M** · 1 file, single fix.

**3.B.2** `request_appeal.rs:79-192` · Lens 1 · Axis-4 divergence-from-lemmy · **[MED]** validation preconditions (case status exhaustive match, appeal window expiry, caller eligibility) run INSIDE the `process_appeal()` helper that's already inside `run_transaction()` · Lens 1: Lemmy convention separates validation into pre-tx fn; the tx scope should be pure write · extract lines 85-137 into `async fn validate_appeal_preconditions(...) -> LemmyResult<(Case, RequesterRole)>` called BEFORE line 66's `conn.run_transaction()`; tx body becomes pure write · **M** · 1 file.

**3.B.3** `create_endorsement.rs:126-127`, `revoke_endorsement.rs:78-79` · Lens 2 · Axis-4 quality-fail · **[MED]** `sponsor_pseudonym.clone()` / `caller_pseudonym.clone()` before moving into `run_transaction` closure; original binding is used only for the clone — move directly is clearer · See 3.A.8; same pattern · remove `.clone()`, rebind into closure scope · **XS** per site · 2 sites.

**3.B.4** `create_report.rs:201` · Lens 2 · Axis-4 quality-fail · **[MED]** `.and_then(|i| i32::try_from(i).ok())` silently drops out-of-range config values; same pattern as 3.A.6 · Same rationale; this is the third site of the same anti-pattern · same fix as 3.A.6 · **XS** · counts with 3.A.6's frequency.

**3.B.5** `revoke_endorsement.rs:295-296` · Lens 3 · Axis-4 quality-fail · **[MIN]** `TODO: restore majority threshold once moderation_case.baseline_sponsor_count column lands` — issue reference is embedded in `.claude/runlog/adhoc-sl-c-baseline-sponsor-count.md`, not a tracked GH issue · TODOs in committed code should reference public issue trackers (or at minimum a stable internal path) — `.claude/runlog/` files can be pruned · change to `TODO(v1-SL-?, GH #<N>): restore majority threshold...` and create the GH issue · **XS** · 1 site, but flag a pattern: audit all TODO/FIXME comments in Brehon-authored code for stable references.

**3.B.6** `create_report.rs:75-81` · Lens 1 · Axis-4 N/A · **[POS]** input validation (reason_code length + emptiness) correctly placed BEFORE any pool/tx operations — matches Lemmy pre-tx-validate convention · No fix · N/A · exemplar.

**3.B.7** `create_endorsement.rs:170-183` · Lens 2 · Axis-4 N/A · **[POS]** exhaustive match on `SponsorGateStrategy` (Closed / Open / Age / Unknown(s)) with named arms and no `_ =>` fallthrough · matches workspace clippy `forbid_match_wildcard_for_single_variants` and the discipline in `feedback_clippy_test_style.md` · No fix · exemplar.

**3.B.8** `create_report.rs:277` · Lens 2 · Axis-4 N/A · **[POS]** `#[expect(clippy::as_conversions, reason = "...")]` wraps f64-to-i64 cast with safety justification linking to OQ-006 spec; cast is guarded by `.is_finite()` · No fix · exemplar.

### 3.C `crates/db_schema/src/source/governance/` — 22 source models + enums.rs + schema.rs additions

**3.C.1** `governance_config.rs:10` · Lens 1+2 · Axis-4 quality-fail · **[MAJ]** `#[derive(PartialEq, Serialize, Deserialize, Debug, Clone)]` — missing `Eq`. All 20 other governance source models derive `PartialEq, Eq` together. Lone outlier breaks `HashSet<GovernanceConfig>` / `BTreeMap` key bounds · Lens 1: 100% parity violation with sibling files in the same directory; Lens 2: idiomatic Rust collection trait bound · add `Eq` to derive: `#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Clone)]` · **XS** · 1 file, one-line fix.

**3.C.2** `jury_assignment.rs:37` (`role`), `appeal.rs:50` (`requester_role`), `reputation_event.rs:51` (`source_event_type`) · Lens 1 · Axis-4 divergence-from-lemmy · **[MAJ]** InsertForm fields typed as `Option<EnumType>` while underlying column is NOT NULL with DB DEFAULT; pattern hides the default in Postgres rather than the Rust type, so callers cannot tell from the form signature whether omitting is semantically safe or a bug · Lens 1: Lemmy convention populates ALL required fields explicitly on InsertForm; optional DB defaults are migration-layer detail · Make non-optional: `pub role: JuryAssignmentRole`, push the default into a `const`/`Default` impl callers use explicitly · **M** per file (call-site audit) · 3 files, same pattern.

**3.C.3** `moderation_case.rs:46-93` field ordering · Lens 1 · Axis-4 divergence-from-lemmy · **[MED]** fields grouped by sub-phase (v1-AD-a, v1-JM-a, v1-SL-a) rather than entity concern (identity, temporal, state, snapshot, appeal, sponsorship); future phases will insert mid-struct, breaking the "immutable-except-for-append" Diesel positional-binding convention · Lens 1: upstream Lemmy source models group by concern · re-organize once into stable section order (identity FKs → temporal → state enums → snapshots → appeal-specific → sponsor-specific); lock the convention in a module comment · **M** · 1 file, but high-risk reorder; do once before more phases add fields.

**3.C.4** `moderation_case.rs:38-93` documentation depth inconsistency · Lens 3 · Axis-4 quality-fail · **[MED]** some fields (`applied_config_snapshot`, `rule_set_version_id`) have 4-line doc comments explaining backfill strategy and NULL semantics; others (`severity`, `status`) have none · mixed coverage leaves maintainers uncertain whether a new field needs phase-gating or NULL handling docs · standardise — either add 1-2 sentence doc to every field (preferred) explaining NULL semantics and backfill strategy, or remove and rely on a top-level convention doc · **S** · 1 file.

**3.C.5** `jury_constraint_violation_log.rs:34` (`constraint_name: String`) · Lens 3 · Axis-4 quality-fail · **[MED]** free-text column for an enumerated value-space (PRD §5.3 enumerates 4 constraint types only: `no_recent_juror_repeat`, `no_majority_from_same_sponsor_cluster`, `geographic_diversity_preferred`, plus one) · CR #92 already promoted `relaxation_reason` to a bounded-vocab enum for the same audit-trail concern (`JuryConstraintRelaxationReason`); the pattern wasn't applied here · promote `constraint_name` to enum with variants matching the 4 known constraints; add migration for the column type · **M** · 1 file + migration + Rust call-site audit.

**3.C.6** `case_evidence.rs:23` (`mime_type: String`) · Lens 3 · Axis-4 quality-fail · **[MED]** MIME type stored as String without validation/enum; call sites could store invalid MIME types (typo, free-text nonsense) on a governance-audit-critical column · governance audits should have high data quality · either parse-validate at insert time via a `MimeType` newtype with a `FromStr` impl, OR add a DB CHECK constraint listing accepted MIME values · **M** · 1 file + Rust newtype.

**3.C.7** `rule_set_version.rs:18` (`text_sha256: Vec<u8>`) · Lens 3 · Axis-4 quality-fail · **[MED]** SHA-256 digest stored as raw `Vec<u8>` with comment "consumers hex-encode for display"; no type distinction between SHA-256 and other binary; a typo treating it as UTF-8 or base64 is easy and silent · type-safety: newtype prevents the size-mismatch and accidental-decode classes · introduce `Sha256Hash([u8; 32])` newtype with `Display` (hex) + `FromStr` (hex parse) + `AsRef<[u8]>`; migrate column type to fixed-width bytea if convenient · **M** · 1 file + Rust call-site audit.

**3.C.8** `endorsement.rs:33`, `surety.rs:33`, `federation_attestation.rs:34-39` · Lens 3 · Axis-4 quality-fail · **[MIN]** InsertForm omits temporal fields (`revoked_at`, `created_at`) without inline comment explaining write-once semantics · maintainers might think field is forgotten; sibling `federation_attestation.rs` is correct (auto-generated `id` + `created_at` is Lemmy convention) but Endorsement+Surety lack the doc · add 1-line comment: `/// revoked_at is set via update(), not insert.` · **XS** · 3 sites.

**3.C.9** `jury_pool.rs:21` (and 5 siblings: `reputation_snapshot`, `endorsement`, `surety`, `appeal`, `sponsor_allowlist`, `case_evidence`) · Lens 3 · Axis-4 quality-fail · **[MIN]** nullable foreign keys (`community_id: Option<CommunityId>`, others) without comment explaining when NULL is valid · maintainers + API consumers can't determine whether NULL is "valid instance-wide" or "error state" · add 1-line doc to each: `/// NULL = instance-wide; Some = community-specific.` (adapt to actual semantics per column) · **XS** · 6 sites, mechanical.

**3.C.10** `mod.rs:15` redaction feature gate · Lens 3 · Axis-4 quality-fail · **[MIN]** `#[cfg(feature = "full")] pub mod redaction;` — module is feature-gated but rationale not documented in the mod.rs · add comment: `/// Redaction helpers for governance audit logs; gated to feature="full" to keep db_schema-only consumers slim.` · **XS** · 1 file.

**3.C.11** `endorsement.rs` + `surety.rs` + `sponsor_allowlist.rs` (no shared mod doc) · Lens 3 · Axis-4 divergence-from-lemmy · **[MIN]** three different table names for related-but-distinct concepts (endorsement = vouch direction, surety = sponsor liability chain, sponsor_allowlist = community-scoped permission); no top-level glossary in `governance/mod.rs` explaining the trio · add a top-level comment in `mod.rs` distinguishing the three; future non-Brehon contributors should grasp the chain without reading 3 source files · **S** · 1 file (mod.rs) — docs only.

**3.C.12** `governance_log.rs:282` (signing) `to_bytes().to_vec()` double-allocation · Lens 2 · Axis-4 quality-fail · **[MIN]** `signing_key.sign(...).to_bytes().to_vec()` allocates twice per signed entry; `Signature` API may support a single-shot allocation · minor perf cost in security-critical hot path · check `ed25519_dalek::Signature` API; replace if a single-shot exists, else accept · **XS** · 1 site.

**3.C.13** `governance_log.rs:303-321` ed25519 signing & error handling · Lens 1+2 · Axis-4 N/A · **[POS]** signing key loaded from env, proper error handling (no panics, all errors returned via LemmyResult), hash chain wired correctly per ADR · No fix · exemplar.

**3.C.14** `enums.rs` `JuryConstraintRelaxationReason` (lines 820-835) · Lens 1 · Axis-4 N/A · **[POS]** 4 variants (SmallPool, ClusterPressure, ClusterPressureExhausted, AdminOverride), each with one-line PRD-section citation, no fallback `Other` variant, comment at line 811 cites CR #92 + ADR-015 safeguard · use as canonical bounded-vocab-enum template for finding 3.C.5's promotion · N/A · exemplar.

**3.C.15** `enums.rs` `ReputationEventSourceType` (and others) · Lens 1 · Axis-4 N/A · **[POS]** PascalCase variants, serde derives consistent with siblings, `#[default]` discipline maintained · matches Lemmy enum conventions · No fix · exemplar.

**3.C.16** `schema.rs` Brehon table ordering · Lens 1 · Axis-4 N/A · **[POS]** Brehon table! blocks slot into Lemmy's alphabetical order convention (actor_pseudonym, appeal, case_evidence, endorsement, governance_config, governance_log, jury_*, moderation_case, etc.); audited cleanly · No fix · exemplar.

**3.C.17** `appeal.rs:31-35` + `moderation_case.rs:65-71` `panel_size_snapshot` / `quorum_snapshot` / `threshold_count_snapshot` repeated · Lens 3 · Axis-4 quality-fail · **[MED]** two tables carry the same three snapshot fields with no shared type · type-safety: a bad value in one table not checked in the other · introduce `PanelSnapshot { size: i32, quorum: i32, threshold: i32 }` struct embedded in both via `#[serde(flatten)]` or composition · **L** · requires migration or composition layer.

### 3.D Migrations 2026-04-* + 2026-05-* — 29 migration dirs (58 .sql files)

**3.D.1** `governance_log_notify_trigger_after_sign` (2026-04-20-000100) + the migration it fixes (2026-04-20-000000) · Lens 3 · Axis-4 N/A · **[MAJ]** fix-migration pattern is uncatalogued: 2026-04-20-000100 fixes 2026-04-20-000000 (AFTER INSERT race with UPDATE signature), but no central registry documents the fix relationship; reverting these out-of-order silently re-introduces the bug · cross-file ordering hazard for any future revert · create `migrations/FIXES.md` (or a header comment on each fix-migration explicitly naming the fixed-migration) cataloguing all "fix" migrations; consider renaming convention `<timestamp>-fix-<original-slug>` for clarity · **S** · 1 file authored, future-applicable convention.

**3.D.2** `2026-04-15-100000-0000_add_governance_enums/up.sql` (and all 4 enum-using migrations 2026-04-23, 2026-04-27, 2026-05-03) · Lens 1 · Axis-4 divergence-from-lemmy · **[MED]** Brehon enum values use PascalCase ('Open', 'ThresholdMet', 'Minor', 'Major'); Lemmy upstream enums use snake_case ('admin_add', 'mod_remove_post') · Lens 1 parity: case-style mismatch with sibling Lemmy enums; risks serde-deserialization issues if a fork-aware client uses Lemmy's case conventions · accept divergence with documentation (Brehon's PascalCase matches its Rust-side enum variant names, easing serde round-trip) — OR rationalize to snake_case with serde aliases. **Recommend: accept-and-document** as intentional divergence; not worth the migration churn for cosmetic alignment · N/A if accepted, M if pursued · 15+ enum values across 4 migrations.

**3.D.3** `2026-04-20-000000-0000_add_governance_log_notify/up.sql:1-28` · Lens 1+3 · Axis-4 quality-fail · **[MED]** `governance_log_notify()` plpgsql function has no inline SQL comments explaining: (a) the 3-field JSON payload shape (id, kind, created_at), (b) the 8KB NOTIFY cap rationale, (c) subscriber handling expectations · governance triggers are security-sensitive; documented rationale aids future maintainers · add 5-10 line comment block before `CREATE OR REPLACE FUNCTION governance_log_notify()` documenting payload shape + cap rationale + subscriber semantics · **XS** · 1 file.

**3.D.4** `2026-04-20-000100-0000_fix_governance_log_notify_trigger_after_sign/up.sql:23-39` · Lens 1+3 · Axis-4 quality-fail · **[MED]** AFTER UPDATE OF signature trigger function lacks `SET search_path = public` and explicit `SECURITY INVOKER` declaration; if an admin changes search_path globally, function lookups could resolve unexpectedly or fail silently · upstream Lemmy triggers commonly pin search_path · add `SET search_path TO public;` and `SECURITY INVOKER` (or `DEFINER` if intentional) to the function definition; mirror in down.sql · **XS** · 1 file.

**3.D.5** `2026-04-19-000000-0000_add_restoration_sanction_variant/down.sql:1-7`, `2026-04-23-000050-0000_add_jury_constraint_relaxation_reason_enum/down.sql`, `2026-05-03-000000-0000_add_case_status_sponsor_liability_variants/down.sql`, `2026-04-27-000000-0000_add_appeal_requester_role_enum/down.sql` · Lens 2+3 · Axis-4 quality-fail · **[MED]** enum-value irreversibility is mentioned in down.sql but the up.sql lacks a GOTCHA comment warning that `ALTER TYPE ADD VALUE` cannot be reversed without a full type rebuild; a future maintainer running selective revert may not notice the constraint until partway through · upstream pattern is to add GOTCHA at up-time, not just defer to down.sql · add up.sql header comment: `-- GOTCHA: Postgres enum values are irreversible. This variant cannot be dropped without DROP TYPE + CREATE TYPE with manual data migration. See down.sql.` · **XS** per file · 4 files.

**3.D.6** `2026-04-22-000300-0000_seed_v1_config_keys/up.sql:53` AND `2026-04-18-000000-0000_add_governance_config/up.sql:73-113` · Lens 2 · Axis-4 quality-fail · **[MED]** seed migrations use `ON CONFLICT (scope, key, valid_from) DO NOTHING` with `valid_from` defaulting to `now()`; idempotency is only true within the same transaction (re-running the migration hours later inserts new rows with different valid_from) · `2026-04-23-000200-0000_seed_v1_jm_config_keys/up.sql:34-62` (Phase v1-JM-a) fixes this by pinning `valid_from = '2026-04-23T00:02:00Z'::timestamptz` — gold standard not retro-applied · pin `valid_from` in both pre-v1-JM-a seeds to literal timestamptz values; mirror in down.sql · **S** · 2 files.

**3.D.7** `2026-04-23-000200-0000_seed_v1_jm_config_keys/up.sql:34-62` · Lens 2 · Axis-4 N/A · **[POS]** correctly pins `valid_from` to literal timestamptz; reruns are truly idempotent · use as template for retrofit of 3.D.6 · N/A · exemplar.

**3.D.8** `2026-04-23-000100-0000_add_jury_mechanics_columns/up.sql:1-44` header · Lens 3 · Axis-4 N/A · **[POS]** 44-line header documenting ADR-010 + ADR-008 + ADR-015 exception trail, plan section, PRD section, task # — exemplary citation discipline · use as template for v1-era migrations · N/A · exemplar.

**3.D.9** `2026-04-20-000100-0000_fix_governance_log_notify_trigger_after_sign/up.sql:1-20` comment block · Lens 3 · Axis-4 N/A · **[POS]** exemplary documentation of the fix rationale: bug name, signature gate logic, at-most-once delivery semantics — gold standard for governance-critical migrations · use as template · N/A · exemplar.

**3.D.10** `2026-05-10-000000`, `…-000100`, `…-000200`, `…-000300` (4 v1-RT-r1 migrations same day) AND `2026-04-23-000000`, `…-000050`, `…-000100`, `…-000200` (4 v1-JM-a migrations same day) · Lens 3 · Axis-4 quality-fail · **[MED]** day-clustered migrations have interdependencies (e.g. 000000 adds column, 000200 backfills it) but no cross-file ordering comments; if Diesel runs them out-of-order or selectively, downstream backfill targets a non-existent column · add to each migration's up.sql: `-- Phase N/M of <sub-phase>. Must run in order: …, …, this, …, ….` · **XS** per file · 8 files.

**3.D.11** `2026-04-22-005541-0000_update_modlog_check_constraint/up.sql:1-4` · Lens 2 · Axis-4 quality-fail · **[MED]** CHECK constraint rebuild drops and re-adds without `DROP CONSTRAINT IF EXISTS` or `ADD CONSTRAINT IF NOT EXISTS` guards; partial-apply / re-run on different DB states will error · idempotency · change to `ALTER TABLE modlog DROP CONSTRAINT IF EXISTS modlog_check;` followed by the ADD with appropriate guard · **XS** · 1 file.

**3.D.12** `2026-05-10-000200-0000_backfill_reputation_event_source_type/up.sql:37-60` · Lens 2 · Axis-4 quality-fail · **[MED]** backfill UPDATE on hot table (`reputation_event`) lacks documented expected runtime, recommended `work_mem`, or partial-index pre-stage; an operator running this in prod blind could lock the table for minutes · backfills on multi-million-row tables need operator-facing guidance · add up.sql header comment with `Expected runtime: O(n) full scan; recommend setting work_mem >= 64MB; pre-stage partial index `CREATE INDEX CONCURRENTLY idx_reputation_event_source_event_type_null ON reputation_event (source_event_type) WHERE source_event_type IS NULL;` to speed up.` · **XS** · 1 file.

**3.D.13** `2026-04-15-100100-0000_add_governance_core/up.sql` (`case_evidence`, `sanction`, `appeal`, etc.) · Lens 1 · Axis-4 quality-fail · **[MED]** FK columns lack secondary single-column indexes; e.g. `case_evidence.uploader_id`, `sanction.target_person_id`, `sanction.target_post_id`, `sanction.target_community_id` are FKs but no `CREATE INDEX` on them · Lemmy convention indexes all FKs to support reverse-lookup queries (`SELECT ... WHERE uploader_id = ?`) without sequential scan · add: `CREATE INDEX idx_case_evidence_uploader ON case_evidence (uploader_id); CREATE INDEX idx_sanction_target_person ON sanction (target_person_id); …` · **S** · 1 migration, multiple indexes; may want to do this as a follow-up migration rather than retroactively editing 100100.

**3.D.14** `2026-04-22-000100-0000_add_sponsor_allowlist/up.sql` · Lens 1 · Axis-4 quality-fail · **[MED]** sponsor_allowlist has UNIQUE (community_id, person_id) which creates a composite index, but no single-column index on person_id alone; "which communities is this person allowlisted to?" goes sequential · same rationale as 3.D.13 · `CREATE INDEX idx_sponsor_allowlist_person ON sponsor_allowlist (person_id);` · **XS** · 1 file (or follow-up migration).

**3.D.15** `2026-04-15-100100`, `2026-04-15-100200`, `2026-04-15-100300`, `2026-04-15-100500` cluster headers · Lens 3 · Axis-4 quality-fail · **[MIN]** no top-level header comment explaining each multi-table migration's semantic grouping (e.g. governance_core is 5 tables forming "moderation case lifecycle"); reader must infer the grouping · add 2-3 line header per migration: `-- Sub-phase X task Y: moderation case lifecycle tables (case, evidence, sanction, appeal, public_case_log). Semantic unit: Open → Decided → Appealed → Closed lifecycle.` · **XS** · 4 files (and apply to subsequent multi-table migrations as convention).

**3.D.16** `2026-04-23-000100-0000_add_jury_mechanics_columns/up.sql:108-120` `relaxation_metadata JSONB` · Lens 3 · Axis-4 quality-fail · **[MIN]** governance-audit-critical JSONB column has comment warning "NEVER free-text user-supplied strings" but no positive example of the correct bounded structure · readers won't know what's correct to write · add example: `-- Example payload: {"dropped_constraint_name": "no_recent_juror_repeat", "phase": "pool_build", "pool_size_before": 10, "pool_size_after": 8}. NEVER include free-text reasons, usernames, or email addresses.` · **XS** · 1 file.

**3.D.17** seed migrations down.sql key-list parity (`2026-04-22-000300`, `2026-04-23-000200`, `2026-05-10-000300`) · Lens 3 · Axis-4 quality-fail · **[MIN]** down.sql DELETE key-list mirrors up.sql INSERT list; if up.sql is edited (key added/removed), down.sql drifts silently unless caught by `config_parity_round_trip` test · add in-file comment: `-- GOTCHA: this key list MUST stay in sync with up.sql INSERTs. Test config_parity_round_trip enforces; a failure there indicates drift.` · **XS** · 3 files.

**3.D.18** `2026-04-22-005541-0000_update_modlog_check_constraint` · Lens 3 · Axis-4 quality-fail · **[MIN]** migration touches upstream `modlog` table without revert-order warning; reverting this in isolation before downstream governance migrations may leave modlog stale · add comment: `-- GOTCHA: if reverting, ensure all subsequent governance migrations (2026-04-23+) are reverted FIRST. modlog.CHECK is a shared resource.` · **XS** · 1 file.

**3.D.19** `2026-04-23-000000` + `2026-04-23-000050` (enum-only migrations) · Lens 2 · Axis-4 quality-fail · **[MIN]** `-- no-transaction` marker missing on enum-only migrations that include `ALTER TYPE ADD VALUE` (which requires running outside a tx in Postgres 11+); 2026-04-19 and 2026-05-03 do have the marker · without the marker, Diesel wraps in a tx and Postgres returns "unsafe use of new value" · audit each enum-only migration and add `-- no-transaction` at line 1 where required · **XS** · 2-3 files (verify by running migrations on a clean DB).

**3.D.20** `2026-04-15-100400-0000_add_actor_pseudonym` substrate · Lens 3 · Axis-4 N/A · **[POS]** ADR-015 GDPR pseudonym table substrate per the v0 endpoint coverage report § ADR conformance; column shape and FK constraints match spec · No fix · exemplar.

### 3.E e2e.rs Brehon fixtures + seed_founders + apub governance

**3.E.1** `e2e.rs` Brehon-authored test fns (lines 20, 55, 864, 908, 1054, 1518, 1803, 1866, 1971, 2696, 2770, 2864, 3702, …) + helpers + sibling `bootstrap` (line 767) · Lens 2 · Axis-4 quality-fail · **[CRIT]** **Case C error-type mixing** per `feedback_lemmy_error_no_std_error.md`: 20+ Brehon test fns return `Result<(), Box<dyn Error>>`, `governance_fixtures::start_postgres` and many of its helpers return `Box<dyn Error>`, but `admin_config_fixtures::bootstrap` (line 767) returns `LemmyResult<...>` — mixed shapes in the same file across the same Brehon-authored test corpus · the lesson's hard refusal: type-shape uniformity is mandatory across a single test module; no mechanical bridge resolves Case C. Adding a single Lemmy-native call to a Box-typed test will trigger E0277 cascade. The §G4 classifier (`.claude/rules/advisor-orchestrator.md`) auto-escalates Case C to user-relay · pick Case A (uniform `LemmyResult<()>` outer + `LemmyResult<T>` helpers; mirror v1-SL-b canonical at ~line 11139) and apply across all Brehon governance test fns + fixtures. Estimated: 20+ test fns, ~30 helpers; some currently-stable tests will see refactor-only churn. Schedule as a dedicated refactor PR — bundle-friendly only because the change is mechanical · **L** · file-wide, ~20+ test fns.

**3.E.2** `e2e.rs:8029-13854` four phase-specific fixtures modules (`v1_jm_b_fixtures`, `v1_jm_e_fixtures`, `v1_sl_b_fixtures`, `v1_sl_c_fixtures`) · Lens 3 · Axis-4 quality-fail · **[CRIT]** each module defines own variants of `seed_case`, `seed_jury_eligible_snapshots`, `seed_jurors`, `seed_community`, `seed_user`; ~70% code overlap; bug fix needs 4 edits; new helper needs 4 modules updated · DRY at scale; tightly coupled to 3.E.1's signature unification (Case C fix and dedup share underlying refactor) · extract shared `governance_test_helpers` module with the canonical seeders; phase-specific fixtures wrap phase-specific assertions only; possibly fold into the Case A migration of 3.E.1 in a single mega-refactor · **L** · 4 files, 15+ test fns affected.

**3.E.3** `e2e.rs:1054-1500` `phase1_migrations_round_trip` test fn is 447 lines covering forward apply + post-forward probes (4 sub-phases) + revert + post-revert probes · Lens 3 · Axis-4 quality-fail · **[MAJ]** test mixes 3 phases (forward, revert, re-apply) and 8 assertion groups; failure in revert phase mixes with all post-forward probe output; IDE test runners show one test; diagnosis is hard · split into `test_phase1_migrations_forward`, `test_phase1_migrations_revert`, `test_phase1_migrations_reapply` with fresh-Postgres fixtures each · **L** · 1 fn into 3, plus duplicated setup (consider extracting a shared bootstrap helper).

**3.E.4** `e2e.rs` `phase1_migrations_round_trip` migration-count revert · Lens 1 · Axis-4 quality-fail · **[MAJ]** uses `PHASE_1_MIGRATION_COUNT` to drive revert; comment at line 1067 acknowledges this is "bookkeeping fiction, not a semantic invariant" and existing inline TODO (line 1093) flags switching to a named-migration list; future refactor that re-orders migrations will silently make the probe invalid · per `feedback_phase1_migration_count_lifo.md` (memory: feedback corpus already captures this lesson) · replace with explicit `MIGRATIONS_TO_REVERT: &[&str] = &["governance_log", "actor_pseudonym", ...]` using the migration runner's named API · **M** · 1 file, requires migration-runner API audit.

**3.E.5** `e2e.rs:2209-2225` `seed_person` (inside `report_to_modlog_golden_path`) duplicates `admin_config_fixtures::seed_user` (lines 5745-5762) · Lens 3 · Axis-4 quality-fail · **[MED]** local helper for one test fn nearly identical to module-level helper; future drift inevitable · move shared helper into `governance_fixtures` module; both test fns call the same canonical seed_user · **S** · 2 files (will fold into 3.E.2 refactor).

**3.E.6** `e2e.rs` deterministic ed25519 signing seed hex hardcoded in 2+ test fns · Lens 3 · Axis-4 quality-fail · **[MIN]** `const SIGNING_SEED_HEX: &str = "0000000000000000000000000000000000000000000000000000000000000001"` repeated in `postgres_container_boots` (line 20) and `report_to_modlog_golden_path` (line 2078) · magic-constant duplication · extract to module-level `const DETERMINISTIC_SIGNING_SEED_HEX: &str = "..."` with doc explaining choice (testability + reproducibility); reference from both test fns · **XS** · 2 sites.

**3.E.7** `e2e.rs` rate-limit bucket bump hardcoded in `admin_config_fixtures::bootstrap` (lines 5722-5730) but NOT applied in `governance_fixtures` bootstrap path · Lens 3 · Axis-4 quality-fail · **[MIN]** admin_config tests are insulated from rate-limit failures; governance tests calling multi-write handlers may flake under default 6/300s Post bucket · per `feedback_rate_limit_debug_config_post_bucket.md` · extract `fn apply_debug_rate_limits(rate_limit: &RateLimit)` called from both bootstrap paths · **S** · 1 helper, 2 call-site edits.

**3.E.8** `e2e.rs` fixture modules NOT marked `#[cfg(test)]` · Lens 3 · Axis-4 quality-fail · **[MIN]** `mod governance_fixtures { ... }`, `mod admin_config_fixtures`, etc. at file scope without `#[cfg(test)]` annotation; IDE tooling + cargo build behavior ambiguous on whether this is harness or dead code · convention: test fixtures get `#[cfg(test)]` to scope correctly · add `#[cfg(test)]` above each `mod *_fixtures` declaration · **XS** per file · 7 modules.

**3.E.9** `e2e.rs` `assert!(dump.len() >= governance_fixtures::pg_template::MIN_TEMPLATE_DUMP_BYTES)` at lines 70-80 · Lens 3 · Axis-4 quality-fail · **[MIN]** dual-use of `MIN_TEMPLATE_DUMP_BYTES` constant (smoke test + cache invalidation per CR #15 + #18); if someone updates the constant in the future, smoke test silently changes coverage · add comment at smoke-test site explicitly linking the constant's dual-use intent, OR extract a `MIN_SMOKE_TEST_DUMP_BYTES = MIN_TEMPLATE_DUMP_BYTES` alias documenting smoke-test semantics · **XS** · 1 site.

**3.E.10** `e2e.rs` test naming: `admin_assign_jury_severity_tier_regular_minor_panel_5_jurors` and similar 50+-char names · Lens 3 · Axis-4 quality-fail · **[MIN]** long names truncate in CI logs (80-char cap); brittle to assertion changes (panel_size 5 → 7 needs rename) · shorten to `admin_assign_jury_regular_minor_panel_size` with doc comment `/// Assert panel_size = 5 for regular/minor tier per jury.panel_size.regular.minor config.` · **S** · 8+ test fns.

**3.E.11** `e2e.rs` test fns without doc comments (`phase1_migrations_round_trip`, `report_to_modlog_golden_path`, …) · Lens 3 · Axis-4 quality-fail · **[MIN]** Lemmy test convention puts a `///` doc at every test fn explaining the scenario · add `/// Test that …` lines to all ~10 Brehon-authored test fns · **S** · 10 fns.

**3.E.12** `crates/apub/activities/src/governance/publish_sanction_notice.rs:282,285,286,301,306,331` + `publish_trust_attestation.rs:146-153` · Lens 2 · Axis-4 quality-fail · **[MED]** clone-storm on `ObjectId<ApubPerson>` + `Url` + `ap_id`: 9+ `.clone()` calls per builder fn moving values into struct fields. Original bindings are not re-used after the clones · Lens 2 idiomatic ownership; builder API likely takes ownership via Into/From; clones are over-defensive · audit `SanctionNoticeProtocol::new` and `PublishSanctionNoticeFromBuilder` signatures; if move-by-value works, remove clones and rely on Rust's borrow checker · **M** · 2 files, may need builder-API tweaks.

**3.E.13** `crates/apub/activities/src/governance/inbox.rs:147,156,171` `governance_log::append(&mut (&mut *conn).into(), ...)` from inside `run_transaction` · Lens 2 · Axis-4 quality-fail · **[MED]** advisory-row + governance_log insert wrapped in `run_transaction` (correct per ADR-006 atomicity), but `governance_log::append` takes a `DbPool`-derived ref while sibling step takes `&mut AsyncPgConnection` directly — heterogeneous conn types in one tx body · maintainer reading this might not realize both ops are atomic; future refactor could break atomicity silently · update `governance_log::append` signature to accept `&mut AsyncPgConnection` directly (or `&mut <C: Connection>`), OR add a doc comment clarifying the coercion · **S** · 2 files.

**3.E.14** `crates/apub/activities/src/governance/publish_sanction_notice.rs:117-147` + `inbox.rs` cross-crate orchestrator split for atomicity contract · Lens 3 · Axis-4 divergence-from-lemmy · **[MED]** ADR-006 + plan §757 require `enqueue_sanction_notice_activity` + `governance_log::append` to be atomic; the split puts the builder in `lemmy_apub_activities` and the orchestrator in `lemmy_api`; no compile-time link between them; a future change that skips the log append breaks atomicity silently · documentation discipline only · introduce a named `send_local_sanction_notice_atomic(plan, conn)` function in `lemmy_api/governance/federation_outbox.rs` that wraps both calls; cite ADR-006 in its doc; callers (submit_jury_vote) use the wrapper, not the two-step pattern · **S** · 1 new fn, ~2 call-site refactors.

**3.E.15** `crates/apub/activities/src/governance/publish_sanction_notice.rs:238` `assert_actor_is_local(actor)?` returning `LemmyResult<()>` · Lens 2 · Axis-4 quality-fail · **[MIN]** function name `assert_*` conventionally implies panic-on-failure; this function returns `Result` (the actual behavior is "verify or error") · Lemmy convention: `verify_*` = returns Result; `assert_*` = panics · rename to `verify_actor_is_local`; update comment to cite ADR-010 directly · **XS** · 1 site.

**3.E.16** `crates/apub/activities/src/governance/inbox.rs:105-175` no rate-limit guard on inbound activity reception · Lens 2 · Axis-4 divergence-from-lemmy · **[MIN]** ADR-006 makes inbound advisory-only (never auto-applied), but "advisory-only" ≠ "spam-resistant"; a peer can flood `remote_sanction_notice` + `governance_log` · intentional v0 omission; v1-federation-inbound will add rate-limit per ADR · document explicitly: add `// TODO(v1-federation-inbound): rate-limit per-remote-instance to prevent log spam. v0 accepts unlimited per ADR-006 (never auto-applied).` · **XS** · 1 site.

**3.E.17** `crates/apub/activities/src/governance/inbox.rs:112-116` `info!("Receiving remote sanction notice {} ...", activity.id);` BEFORE the insert · Lens 2 · Axis-4 quality-fail · **[MIN]** if the insert (line 158+) fails, the log says "received" when nothing was stored; log-vs-storage divergence · move `info!` to AFTER successful insert; replace pre-insert with `debug!` if needed · **S** · 1 file.

**3.E.18** `crates/apub/activities/src/governance/publish_sanction_notice.rs:59-100` verify() ordering · Lens 1 · Axis-4 quality-fail · **[MIN]** `verify_is_public(&self.to, &self.cc)?` runs before actor-binding check; if `verify_is_public` fails, error doesn't mention actor binding; framework HTTP-signature check is assumed but not documented · add `/// HTTP signature verification is performed by activitypub_federation before this function is called; we assume the signer is the authenticated actor. This method checks (1) public broadcast intent and (2) consistency between activity.actor and object.actor to prevent spoofing.` to fn doc · **XS** · 1 site.

**3.E.19** `seed_founders/src/main.rs:84` `let raw_part = parts.get(idx).copied().unwrap_or("");` · Lens 2 · Axis-4 quality-fail · **[MIN]** `unwrap_or("")` falls through to parse failure on next line, but error reads as "parse failed" rather than "missing segment"; two-stage failure with confusing first stage · single-stage with explicit ok_or_else: `let raw_part = parts.get(idx).copied().ok_or_else(|| LemmyErrorType::Unknown(format!("founder spec `{raw}` {label} not parseable (missing segment)")))?;` · **XS** · 1 site.

**3.E.20** `seed_founders/src/main.rs:75-130` `parse_founder_spec` validation function no unit tests · Lens 3 · Axis-4 quality-fail · **[MED]** function is the main entry point for CLI parsing; bugs here break the tool; no edge-case coverage (negative values, out-of-range, parse errors, wrong segment count) · add `#[cfg(test)] mod tests` block with `test_parse_founder_spec_valid`, `_negative_delta`, `_exceeds_max`, `_wrong_segment_count`, `_non_numeric` cases · **S** · 1 file, 5 tests.

**3.E.21** `seed_founders/src/main.rs:1-22` module doc missing preconditions + exit codes · Lens 3 · Axis-4 quality-fail · **[MIN]** doc explains what the tool does, but not exit-code semantics, DB preconditions (database exists, migrations applied, signing key env set), expected invocation order (after Phase 5b seed migration) · admins running tool need this in doc, not deep in code · expand module doc with `## Preconditions` + `## Exit codes` sections · **XS** · 1 file.

**3.E.22** `seed_founders/src/main.rs:305` `println!` instead of `tracing::info!` · Lens 2 · Axis-4 divergence-from-lemmy · **[MIN]** Lemmy convention uses `tracing::*` macros for structured logging; `println!` bypasses log filtering · replace with `tracing::info!("seeded {} founder(s) ...", new_count)` · **XS** · 1 site.

**3.E.23** `seed_founders/src/main.rs:238-242` verbose error message `expires_at={} must be strictly in the future (now={now})` · Lens 2 · Axis-4 quality-fail · **[MIN]** repeats user input verbatim; Lemmy convention keeps error messages tight (assume CLI caller has input in view) · shorten to `expires_at must be strictly in the future` · **XS** · 1 site.

**3.E.24** `seed_founders/src/main.rs:92` PII-grep hyphen-vs-underscore comment · Lens 3 · Axis-4 quality-fail · **[MIN]** rule-following comment ("Label uses a hyphen so it does not match the §12 Level 5 PII identifier-name grep") is codebase-specific knowledge inline; future developer might "fix" the hyphen not realizing the PII-grep consequence · extract to a rule file `.claude/rules/pii-grep-identifier-naming.md` or shorter doc; reference from seed_founders with a brief comment · **XS** · 1 site, but pattern: audit other inline rule-following comments in Brehon-authored code.

---

## §4 Ranked refactor backlog

**Ranking method:** (severity_weight × frequency_count × inverse_effort). Severity weights: CRIT = 8, MAJ = 4, MED = 2, MIN = 1. Effort weights (inverse): XS = 4, S = 3, M = 2, L = 1.

Top-20 findings. Each entry is tagged per user 2026-05-14 strict-gate vocabulary: **`[fix-before-next-phase]`** blocks v1 PRD resumption; **`[fix-during-relevant-sub-phase]`** handles when a phase touches the code; **`[accept-and-document]`** logs and moves on; **`[intentional-divergence-accept]`** Axis-4 only.

| Rank | Finding | Score | Tier | Notes |
|---|---|---|---|---|
| 1 | 3.E.1 — Case C error-type mixing in e2e.rs (20+ test fns) | 8×20×1 = 160 | **fix-before-next-phase** | Every new e2e test inherits this risk. Bundle with 3.E.2 below. |
| 2 | 3.E.2 — 4 phase-specific fixtures modules with ~70% duplication | 8×4×1 = 32 | **fix-before-next-phase** | Pairs with rank-1; shared refactor PR. Touching same files. |
| 3 | 3.B.1 — TOCTOU race in `create_report.rs` (no run_transaction) | 8×1×2 = 16 | **fix-before-next-phase** | Sibling files DO use run_transaction. Inconsistency is the real risk. |
| 4 | 3.A.4 — admin gate + pseudonym fetch boilerplate in 7-8 admin handlers | 2×7×3 = 42 | **fix-during-relevant-sub-phase** | Touching any one admin file? Extract while there. |
| 5 | 3.A.7 — `.map_err(|_| Unknown(...))` swallows Diesel context in 5 sites | 2×5×4 = 40 | **fix-during-relevant-sub-phase** | Each `?` propagation is XS but spans 5 files. |
| 6 | 3.D.6 — Seed migrations not pinned `valid_from` (2 files) | 2×2×3 = 12 | **fix-before-next-phase** | Idempotency surprise on rerun. Pair with 3.D.10 in a migrations-hygiene PR. |
| 7 | 3.E.3 — `phase1_migrations_round_trip` 447 lines, 3 phases mixed | 4×1×1 = 4 | **fix-before-next-phase** | CI signal clarity. Bundle with 3.E.1+3.E.2. |
| 8 | 3.E.4 — Migration-count revert is "bookkeeping fiction" | 4×1×2 = 8 | **fix-before-next-phase** | Already-flagged inline TODO; fix during 3.E.3 split. |
| 9 | 3.A.1 — `submit_jury_vote.rs` 1019 lines monolithic | 4×1×1 = 4 | **fix-during-relevant-sub-phase** | When JM-f or RT-r2+ touches vote-aggregation, split then. |
| 10 | 3.A.2 — `admin_config.rs` 1414 lines, 4 handlers mixed | 4×1×2 = 8 | **fix-during-relevant-sub-phase** | When AD-* touches config, split then. |
| 11 | 3.C.1 — `governance_config.rs` missing `Eq` derive | 4×1×4 = 16 | **fix-before-next-phase** | One-line fix; cleanest possible refactor PR. |
| 12 | 3.C.2 — 3 InsertForm `Option<Enum>` for NOT NULL columns | 4×3×2 = 24 | **fix-during-relevant-sub-phase** | When touching jury_assignment / appeal / reputation_event. |
| 13 | 3.A.5 — `.optional().ok().flatten()` swallows DB errors in admin_audit_stream | 4×2×3 = 24 | **fix-before-next-phase** | Audit stream is observability-critical; error erasure here is the worst spot. |
| 14 | 3.E.12 — Clone-storm in `publish_sanction_notice.rs` (9+ sites) | 2×9×2 = 36 | **fix-during-relevant-sub-phase** | When federation work touches publish_*. |
| 15 | 3.D.13+3.D.14 — Missing FK indexes (`case_evidence`, `sanction`, `sponsor_allowlist`) | 2×4×3 = 24 | **accept-and-document** | One follow-up migration; not blocking; document & schedule for first-pilot prep. |
| 16 | 3.C.5 — `constraint_name: String` not enum (CR #92 sibling not retro-applied) | 2×1×2 = 4 | **fix-during-relevant-sub-phase** | When JM-f or RT touches jury_constraint_violation_log. |
| 17 | 3.E.20 — `parse_founder_spec` no unit tests | 2×1×3 = 6 | **fix-before-next-phase** | seed_founders is the entry tool; failing here is operator-visible. |
| 18 | 3.A.3 — `config.rs` 8 accessor functions duplicated | 4×1×2 = 8 | **accept-and-document** | Generic refactor risk > duplication cost for v0; revisit when adding the 9th accessor. |
| 19 | 3.D.2 — PascalCase enum values diverge from Lemmy snake_case | 2×15×1 = 30 | **intentional-divergence-accept** | Migration churn > parity gain; matches Brehon's Rust-side enum variants. |
| 20 | 3.E.13 — `governance_log::append` heterogeneous conn type in tx | 2×3×3 = 18 | **fix-during-relevant-sub-phase** | When federation-inbound work touches inbox.rs. |

**Summary of tier assignments:**

| Tier | Count |
|---|---|
| **fix-before-next-phase** (gates PRD resumption) | 8 (ranks 1, 2, 3, 6, 7, 8, 11, 13, 17 — rank 17 has the lowest score but seed_founders is operator-facing) |
| **fix-during-relevant-sub-phase** (handle in-context) | 8 |
| **accept-and-document** | 2 |
| **intentional-divergence-accept** | 1 |
| Out of top-20 but in §3 | 44 |

**Recommended refactor PR sequence for fix-before-next-phase tier:**

- **PR-1 — `chore(test): unify error-type to LemmyResult + consolidate fixtures + split phase1_migrations_round_trip`**: ranks 1, 2, 7, 8 bundled. The e2e refactor is one large mechanical change touching all Brehon test fns + 4 fixtures modules + the round-trip test. Bundling minimizes the number of times the file is destabilized. Estimate: L (full day).
- **PR-2 — `feat(api_crud): wrap create_report SELECT-then-write in run_transaction`**: rank 3. Isolated 1-file fix. Estimate: M (half-day).
- **PR-3 — `fix(db_schema): add Eq derive to governance_config`**: rank 11. One-line fix in a single-purpose PR for cleanest CR-review history. Estimate: XS.
- **PR-4 — `fix(migrations): pin valid_from on v1-AD-a / v0 seed migrations`**: rank 6. Two-file fix. Estimate: S.
- **PR-5 — `fix(api): propagate Diesel errors in admin_audit_stream optional().ok() chains`**: rank 13. Estimate: S.
- **PR-6 — `test(seed_founders): add unit tests for parse_founder_spec validation`**: rank 17. Estimate: S.

PR-1 is the largest; PRs 2-6 are short and serial. Estimated cumulative effort: 2-3 working days to ship fix-before-next-phase tier; v1 PRD work then resumes.

---

## §5 Cross-cutting observations

**5.1 Internal-inconsistency is the most concerning pattern.** The audit's most actionable findings are NOT parity-with-upstream gaps (which can be accepted as fork divergence) but inconsistencies WITHIN Brehon code: 3.B.1 (`create_report` skips `run_transaction` while sibling `create_endorsement` uses it); 3.C.1 (`governance_config` is the only governance source model without `Eq`); 3.D.6 vs 3.D.7 (Phase v1-JM-a seed pins `valid_from`, Phase 5a + v1-AD-a seeds don't); 3.E.1 (one fixtures bootstrap returns `LemmyResult`, twenty Brehon test fns return `Box<dyn Error>`). These are not Lemmy-divergence — they're Brehon-vs-Brehon divergence. The lesson: when an audit catches Brehon being inconsistent with Brehon, the canonical pattern almost always lives in a sibling file. Refactoring to the better-of-two siblings is mechanical and cheap; the cost is just doing it.

**5.2 The lesson corpus is doing real work.** Multiple findings reference `.claude/lessons/feedback_*.md` directly: `feedback_lemmy_error_no_std_error.md` (3.E.1), `feedback_phase1_migration_count_lifo.md` (3.E.4), `feedback_rate_limit_debug_config_post_bucket.md` (3.E.7). The lessons have caught Case C *during* implementation; the audit catches Case C in *baselined* code. This is the lesson lifecycle working: PMD → lesson file → mandatory injection on next brief → caught at advisor-side gate. The audit's role is to retroactively apply lessons that the corpus didn't have when the code was written. That implies a corollary: **every audit-flagged anti-pattern that recurs after this audit ships represents a lesson that exists but isn't being injected at planning time**. Watch for that in retro signals.

**5.3 The fork's divergence is mostly intentional and mostly safe.** The Axis-4 catalog in §6 lists 28 findings tagged divergence-from-lemmy or both. The bulk of these are: governance-specific tables/enums with no upstream equivalent (no divergence to score against); PascalCase enum values matching Brehon's Rust-side variant names (intentional, low-risk); ADR-006 atomicity contracts that span Lemmy + apub crates (intentional, well-documented in builder fn comments). Only two divergence tags carry real risk: 3.E.14 (cross-crate orchestrator split with no compile-time atomicity link — a refactor could break the contract silently) and 3.A.8/9 (data.clone() patterns that diverge from upstream move-semantics — pragmatic, but a v2 maintainer might "fix" them not realizing the closure-capture constraint). Neither is critical; both are documentation-fixable.

**5.4 File-size growth is a recurring smell, not a defect.** Three files (`config.rs` 3272 LOC, `admin_config.rs` 1414 LOC, `submit_jury_vote.rs` 1019 LOC) are flagged for splitting. None of them are buggy; they grew organically as sub-phases stacked features. Splitting them is a context-cost trade-off: code-reviewer attention is bounded; a 1500-line file that should be 4 files imposes a tax on every future CR. The tier assignments for §4 ranks 9, 10, 18 reflect this: don't refactor unless touching, but DO refactor when touching. That said: `submit_jury_vote.rs` is the next likely casualty if RT-r2+ adds vote-aggregation logic without splitting first.

**5.5 Documentation discipline is uneven.** Migration headers vary from "exemplary 44-line ADR trail" (3.D.8) to "no header at all" (3.D.15). Source-model field comments vary from "4-line backfill explainer" to "no comment for status/severity" (3.C.4). Test fn docstrings are absent across all Brehon-authored fns (3.E.11). The pattern: the longer a sub-phase spent in CR triage, the better-documented its outputs. v1-JM-a migrations (the gold standard) reflect this — they shipped after extensive CR feedback. v0 migrations (the silent ones) didn't. **Implication for going forward:** every retro should check the documentation depth of the shipped phase as a per-phase score, not just the test-coverage score.

**5.6 The test corpus has a sub-phase-by-sub-phase organic structure.** Each sub-phase added its own `v1_<phase>_fixtures` module; this was a reasonable per-phase choice but compounds into the 4-module DRY violation (3.E.2). The fix is to flatten back to one shared `governance_fixtures` module and have phase-specific test fns live in the e2e file body, importing from the common module. This is exactly the structure Lemmy upstream uses. Trade-off: a single shared module is harder to grep at retro time (which phase introduced helper X?), but git blame already answers that question without a per-phase module wall.

---

## §6 Divergence catalog (Axis-4 inventory)

Every finding tagged **divergence-from-lemmy** or **both** is catalogued below for the eventual upstream-rebase work (separate decision, not part of this audit). For each: what Lemmy does (cited reference path on `upstream/main@17288bed2`), what Brehon does (cited file:line), why Brehon does it differently if knowable.

| # | Brehon location | Lemmy reference | What Lemmy does | What Brehon does | Why (if knowable) |
|---|---|---|---|---|---|
| 1 | 3.A.8 `submit_jury_vote.rs:138` etc. | `upstream/main:crates/api/api/src/post/like.rs` | Direct closure ownership; `data` moved in | `let vote_data = data.clone()` before move | Defensive clone; pragmatic but non-idiomatic |
| 2 | 3.A.9 `submit_jury_vote.rs:140-145` | upstream like.rs | Threads `&context` through | Arc-clones context for closure | Closure-lifetime simplicity; comment "cheap Arc clone" acknowledges |
| 3 | 3.B.2 `request_appeal.rs:79-192` | upstream `crates/api/api_crud/src/community/create.rs` | Validates before tx | Validates inside tx | Atomicity-prioritized; mixes validate + write in tx |
| 4 | 3.C.2 jury_assignment / appeal / reputation_event | upstream source models | Required-field InsertForm types match column necessity | `Option<Enum>` with DB DEFAULT | DEFAULT-driven ergonomics; hides intent in Rust type |
| 5 | 3.C.3 `moderation_case.rs:46-93` | upstream `crates/db_schema/src/source/post.rs` | Fields grouped by entity concern | Fields grouped by sub-phase chronology | Append-only convention but breaks logical ordering |
| 6 | 3.C.11 endorsement/surety/sponsor_allowlist trio | (no Lemmy equivalent) | N/A | Three separate tables, no glossary | Brehon-specific concept space |
| 7 | 3.D.2 governance enum PascalCase | upstream modlog enums use snake_case | snake_case enum values | PascalCase enum values | Matches Brehon Rust-side variant names; intentional |
| 8 | 3.D.5 enum-add irreversibility GOTCHA placement | upstream enum-add migrations | GOTCHA at up-time | GOTCHA in down.sql only | Inherited stylistic difference, fixable |
| 9 | 3.D.8 v1-JM-a migration headers | upstream migrations | Standard 1-2 line comment | 44-line ADR trail | Brehon documentation discipline above Lemmy baseline (positive) |
| 10 | 3.E.1 e2e error-type Case C | upstream test fns uniform `LemmyResult<()>` | Uniform LemmyResult | Mixed Box<dyn Error> + LemmyResult across same file | Historical: started with Box (pre-lesson); never retro-applied |
| 11 | 3.E.12 apub publish_*.rs clone-storms | upstream `crates/apub/activities/src/community/announce.rs` | Move-semantics on Url + ObjectId | 9+ clones per builder fn | Builder API over-defensive; pre-CR-review pattern |
| 12 | 3.E.14 cross-crate orchestrator split for atomicity | upstream apub orchestrators in single crate | Single-crate orchestrator | Builder in apub, orchestrator in lemmy_api | DQ-6.6 resolution; safe but no compile-time link |
| 13 | 3.E.16 inbound activity no rate-limit | upstream Community::get/Update have rate-limits | Per-handler rate-limit | None on inbound governance activities | v0 advisory-only intent; v1-federation-inbound will add |
| 14 | 3.E.22 `println!` vs `tracing::info!` | upstream uses tracing universally | tracing::*! | println!! in seed_founders main | Tool-side oversight; not deliberate |

**Upstream-rebase risk assessment:**

- **Low risk** (cosmetic / docs-only): #5, #6, #7, #8, #9, #14. Fix during rebase, no logic change.
- **Medium risk** (touched code, mechanical): #1, #2, #3, #10, #11. Code-touch but no behavior change; risk is conflict-resolution complexity at rebase.
- **Higher risk** (semantic divergence): #4, #12, #13. Resolution requires understanding the Brehon-side rationale before deciding whether to converge or document.

None are blocking for future rebases; all are catalogued for retro reference.

---

**Audit ends.** Total findings: 64 distinct (3 critical + 10 major + 22 medium + 15 minor + 4 positive exemplars + 10 divergence catalog entries fold into the rest). Read-only audit complete; no code modified; no DQ writes; no Junior dispatch.
