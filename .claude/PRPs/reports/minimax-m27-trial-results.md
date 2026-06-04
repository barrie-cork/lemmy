# MiniMax M2.7 vs Sonnet 4.6 — impl-task A/B trial — RUNNING (m1-b)

**Status:** m1-b arms COMPLETE — T3 + T4 + T5 recorded (n=3 of 5 cumulative). PAUSED until next MIRROR-heavy sub-phase (2 more tasks → n=5). First arms ran 2026-06-04.

**Decision rule (from runbook §2.5):** analyse after **5 cumulative eligible tasks, cross-phase** (user-confirmed 2026-06-04). m1-b contributes 3 (Tasks 3/4/5); the remaining 2 come from the next MIRROR-heavy phase. **Do NOT decide at end of m1-b** — n=3 is below the threshold.

**Trial model (user-confirmed 2026-06-04):**
- Both arms run in **parallel** off the same phase-branch base, on throwaway `ab-test/{sonnet,minimax}-impl-<N>` branches.
- The **Sonnet (control) arm is canonical** — only it merges into `phase-m1-b`. The **MiniMax arm is comparison data only and NEVER merges.**
- MiniMax pinned to **M2.7** (not M2.5). Dispatched via `queue-minimax-task.sh` (or the inline `junior task add` SSH equivalent); control via MCP `create_task`.

**Prior status:** the v1-RT-r4 arming (2026-05-30) never ran — serial dispatch after an OOM left no parallel arm. That stub is superseded by this file.

---

## Comparison table

| Task | Metric | Sonnet 4.6 (control) | MiniMax-M2.7 (trial) | Notes |
|---|---|---|---|---|
| **T3** (DTOs) | Junior id | #576 | #577 | base `phase-m1-b@81762a740`; brief `m1-b-impl-3.md` |
| | Wall-clock | 2m22s (10:36:47→10:39:09) | **1m39s** (10:38:37→10:40:16) | MiniMax faster |
| | DQ blockers raised | 0 | 0 | tie — neither needed clarification |
| | Cargo first-attempt | **PASS** — `check --workspace --features full` Finished 3m22s, 0 err 0 warn; `e2e --no-run` Finished 21m57s, `E2E_COMPILE_EXIT_0` | **PASS (inferred)** — code byte-identical to #576; the Sonnet compile (both commands green) proves it | see note below |
| | Commit subject | `feat(api_common): … (task 3)` ✓ | `feat(api_common): … (task 3)` ✓ | both conform |
| | HANDOVER trailer | ✓ complete | ✓ complete (identical keyDecisions) | both conform |
| | LESSON trailer | none | ✓ (correct ts-rs/Eq note) | MiniMax added a useful lesson |
| | Diff quality | correct; terse docs; backtick route refs match sibling | correct; **richer docs** (explains value_type derivation + scope format); dropped route backticks (minor sibling-style drift) | both honored the load-bearing `{previous,new}` divergence (no governance_log_id/preview/applied); both correctly omitted `Eq` |
| **T4** (admin handler) | Junior id | #578 | #579 | base `phase-m1-b@2a2c127b7`; brief `m1-b-impl-4.md`; **first substantive divergence — NOT byte-identical** |
| | Wall-clock | n/a (different dispatch paths; not comparable this task) | n/a | T4 arms dispatched ~minutes apart via different mechanisms; wall-clock not a clean signal here |
| | DQ blockers raised | 0 | 0 | tie on count — BUT brief §4.5 said "raise a blocker if the GET-response shape is ambiguous"; Sonnet followed the fallback ("ambiguous → clean bare-value shape"), MiniMax **guessed wrong** instead of raising. Process-divergence even though raw count ties. |
| | Cargo first-attempt | **PASS** — `check --workspace --features full` Finished 19m12s, 0 err **1 warn** (`private_interfaces` on `GetMessagingConfigQuery`); `e2e --no-run` Finished 27m35s, `E2E_COMPILE_EXIT_0`, 0 err 2 warn (same warning) | **not re-run** — code substantively different (192 vs 163 lines); NOT merged, so not validated. The warning Sonnet hit, MiniMax avoided (declared `pub struct`). | only the canonical (Sonnet) arm is compile-validated per the merge-only-Sonnet policy |
| | Commit subject | `feat(api): add messaging-config admin handler (task 4)` ✓ | conforms ✓ | both conform |
| | HANDOVER trailer | ✓ complete | ✓ complete | both conform |
| | GET-response DTO | **CORRECT** — bare `Json<ConfigValueWithProvenance>` (the actual provenance shape) | **WRONG** — `Json<AdminSetMessagingConfigResponse>` with `previous==new` duplicated (a set-response shape returned from a getter) | **decisive quality gap — Sonnet correct, MiniMax incorrect** |
| | Error handling | `.ok_or_else(...)?` (no panic paths; clippy-safe under `-D warnings`) | `.expect("is_boolean was true")` panic paths (clippy-fragile; would fail the workspace clippy gate) | Sonnet better |
| | DQ-file hygiene | surgical `+31/-2` (just its own validate-pending entry) | **rewrote 534 lines** of `decision-queue.json` (gratuitous reformat) | Sonnet better |
| | Scope discipline | exactly the briefed surface | added **unbriefed** `scope: Option<String>` defaulting to `"instance"` | Sonnet better (no scope creep) |
| | `private_interfaces` warning | **present** (1-line `pub struct` fix; folding into T5 since T5 edits this file) | absent (correctly declared `pub struct AdminGetMessagingConfig`) | **MiniMax's one win** — minor lint, trivially fixable |
| **T5** (validator + routes) | Junior id | #580 | #581 | base `phase-m1-b@dc5b93ba8`; brief `m1-b-impl-5.md`; **arms CONVERGED — validator byte-identical** |
| | Wall-clock | n/a | n/a | both workers HUNG in the post-task-retro `memory_write_eval` tail AFTER committing (benign; retro item — see note). Wall-clock not measurable. |
| | DQ blockers raised | 0 | 0 | tie — and this time correctly so. The brief §4.1 **pre-resolved** the error-variant ambiguity that bit MiniMax on T4 (spelled out the exact validator + `Unknown(format!)` idiom). Neither arm needed to guess. |
| | Cargo first-attempt | **PASS** — `check --workspace --features full` Finished 1m33s, 0 err **0 warn** (the T4 `private_interfaces` warning is CLEARED by the pub-struct fold-fix); `e2e --no-run` Finished 3m23s, `E2E_COMPILE_EXIT_0`, 0 err 0 warn | **not re-run** (comparison-only policy) — but code is near-identical to Sonnet, so inferred-equivalent | only Sonnet compile-validated per merge-only-Sonnet policy; warm `target/` from T4 → fast builds |
| | `validate_identity_policy` | per §10.4 + `Unknown(format!)` idiom | **BYTE-IDENTICAL** to Sonnet (same guard, same error message, lines 69-80) | **convergence** — when the brief pre-specified the validator, both arms produced the same correct code |
| | `pub struct GetMessagingConfigQuery` fold-fix | **applied** (line 155) | **applied** (line 153) | both correctly cleared the T4 leftover warning |
| | Route registration | nested `scope("/messaging-config")` POST+GET, mirror `:478-512` | **identical** registration + import style | both mirrored the sibling correctly |
| | Diff quality | +55/-2 across 3 files | +51/-2 (slightly terser; added a `row_to_value` GET-path helper) | near-tie; MiniMax marginally more compact |

**Cargo-validation note (T3):** the two arms produced **byte-identical struct definitions** — the ONLY diff is in doc-comment prose, which does not affect compilation. The canonical (Sonnet) arm is validated authoritatively on the laptop (`cargo check --workspace --features full` + `e2e --no-run`); the MiniMax arm's compile result is inferred-identical rather than re-run, to avoid a second serial ~9-min cargo pass on the same `target/` for provably-equivalent code. This inference is recorded explicitly so the n=5 decision isn't built on a hidden assumption.

### T3 qualitative read

Both arms are **functionally equivalent and correct on the first attempt.** Both honored the brief's load-bearing divergence (simple `{ previous, new }` response, NOT the full sibling shape) and the `serde_json::Value`-is-not-`Eq` constraint. MiniMax was ~33% faster wall-clock and volunteered a correct `LESSON:` trailer; Sonnet's commit-body prose and doc-comment backtick style matched the codebase conventions marginally more closely. **No quality gap that would move the cutover decision either way at n=1.**

### T4 qualitative read

T4 is the **first task where the two arms diverged substantively** (192 vs 163 lines, different code — not a doc-only diff like T3). The brief was deliberately a step harder: a real ambiguity (the GET-response shape) with an explicit instruction in §4.5 — *"if ambiguous, raise a blocker; else default to the clean bare-value shape."*

**Sonnet (control) was clearly the better artifact on four independent dimensions:**
1. **Correct GET DTO** — returned the bare `ConfigValueWithProvenance` provenance shape. MiniMax returned a *set*-response struct (`AdminSetMessagingConfigResponse`) from a *getter*, duplicating `previous==new` — semantically wrong (a getter has no "previous").
2. **No panic paths** — `.ok_or_else(...)?` vs MiniMax's `.expect(...)`. MiniMax's panics would trip the workspace clippy `-D warnings` gate (unwrap/expect denied per `feedback_clippy_test_style.md`).
3. **DQ-file hygiene** — Sonnet touched only its own validate-pending entry (`+31/-2`); MiniMax gratuitously rewrote 534 lines of `decision-queue.json`.
4. **Scope discipline** — MiniMax invented an unbriefed `scope: Option<String>` field; Sonnet stayed exactly on the briefed surface.

**MiniMax's single win:** it declared `pub struct AdminGetMessagingConfig`, avoiding the `private_interfaces` warning Sonnet emitted on its `GetMessagingConfigQuery`. This is a one-character lint (`pub`), trivially fixable — and is being folded into the T5 brief (T5 edits this same file).

**Process signal worth weighting:** the brief explicitly anticipated the GET-shape ambiguity and instructed "raise a blocker if ambiguous." Sonnet took the documented fallback (clean shape, no blocker — defensible). MiniMax **guessed and guessed wrong**. The raw DQ-blocker count ties at 0/0, but the *quality* of the no-blocker decision differs: Sonnet's no-blocker was correct-by-fallback, MiniMax's was incorrect-by-guess. At a 5-juror-panel level of rigour, this is the kind of divergence the trial exists to surface.

**Net T4:** Sonnet wins decisively on correctness + safety + discipline; MiniMax wins one trivial lint. This is the first task that would move the cutover decision **toward staying on Sonnet** — but it is one task; the n=5 cross-phase rule stands.

**Cargo-validation note (T4):** unlike T3, the arms are NOT byte-identical, so the MiniMax arm's compile result cannot be inferred from the Sonnet compile. Per the merge-only-Sonnet policy, **only the canonical (Sonnet) arm was compile-validated** (`cargo check --workspace --features full` 0 err + `e2e --no-run` 0 err, both green). The MiniMax arm was read for comparison but never compiled or merged — its `.expect()` panic paths and wrong DTO are static-analysis observations, not compile failures. Recorded explicitly so the n=5 read isn't built on a hidden "MiniMax would have compiled" assumption.

### T5 qualitative read — the convergence case (and what it teaches about T4)

T5 is the **most informative task in the trial so far**, because it directly tests the T4 hypothesis. T4's divergence (MiniMax guessed wrong on the GET-response shape) could have meant either (a) MiniMax is a weaker model, OR (b) MiniMax handles brief ambiguity worse than Sonnet. T5 was authored to distinguish these: the T5 brief §4.1 **pre-resolved the one ambiguity** (it spelled out the exact `validate_identity_policy` body + the `Unknown(format!)` error idiom, explicitly citing the T4 failure as the reason).

**Result: the arms CONVERGED.** MiniMax's `validate_identity_policy` is **byte-identical** to Sonnet's (same guard, same error message, lines 69-80). Both applied the `pub struct GetMessagingConfigQuery` fold-fix. Both registered the routes identically. The only diff is a small MiniMax `row_to_value` GET-path helper (+51 vs +55 lines).

**The teaching:** T4's gap was **brief-ambiguity-driven, not capability-driven.** When the brief left a shape decision to the worker (T4 GET DTO), MiniMax guessed and guessed wrong while Sonnet took the documented fallback. When the brief removed the ambiguity (T5 validator), MiniMax matched Sonnet exactly. This is a **load-bearing finding for the n=5 decision and for how we brief**: MiniMax is viable for **fully-specified MIRROR-ref tasks** but is **more sensitive to under-specified briefs** than Sonnet — Sonnet degrades more gracefully on ambiguity (falls back to the documented default; MiniMax improvises). The mitigation is cheap (tighter briefs), which is exactly the regime the trial targets (MIRROR-ref-heavy, ≤2 files, well-specified).

**Net T5:** tie on correctness, safety, and discipline. No quality gap. The convergence, given the deliberate ambiguity-removal, is the signal — not a point for either arm, but evidence that the T4 Sonnet win is **recoverable by briefing**, not an intrinsic MiniMax deficiency.

**Operational note (retro item):** both T5 workers HUNG in the post-task-retro `memory_write_eval` step AFTER committing their code (Sonnet pushed; MiniMax committed, push hung). No `.git/index.lock`; clean worktrees. The code was fully preserved (Sonnet `c9e7d6b76` on origin + FF-promoted; MiniMax `12487b06a` advisor-pushed to `ab-test/minimax-impl-5-result`; both worktrees tar-preserved). This is the **2nd retro-step friction this phase** (the daemon's post-task-retro MCP call stalling the worker process) — distinct from the dispatch findings, worth a durable lesson if it recurs.

---

## m1-b MiniMax arms — COMPLETE (3 of 5 cumulative)

All three m1-b MiniMax-eligible tasks (T3, T4, T5) are recorded. m1-b's remaining Tree-B tasks (T6 bridge_notify, T7 e2e) are **not** MiniMax-eligible (T6 = fire-and-forget HTTP wiring with no MIRROR-sibling shape; T7 = e2e test authoring, explicitly excluded by the trial criteria). **The trial pauses here at n=3 and resumes on the next MIRROR-heavy sub-phase to reach n=5.**

## Pending (cross-phase, to reach n=5)

- 2 more eligible tasks from the next MIRROR-heavy sub-phase (≥2 files-or-fewer, MIRROR-ref-heavy, cargo-gated, not e2e).

## Decision (deferred until n=5)

Per runbook §2.5: M2.7 matches Sonnet on cargo-first-pass AND DQ-blocker-rate across the 5 tasks → switch impl-task to MiniMax (≈10× cost saving). M2.7 worse on either → stay Sonnet. Mixed at n=5 → extend to n=8-10 or stay Sonnet (conservative default).

**Running tally (n=3 of 5):**
- **Cargo-first-pass:** T3 tie (both pass; MiniMax inferred-identical). T4 — only Sonnet validated; PASS (MiniMax not compiled per policy). T5 — only Sonnet validated; PASS 0 err 0 warn (MiniMax near-identical → inferred-equivalent). → **no MiniMax compile failure ever observed; no positive MiniMax compile evidence either** (the merge-only-Sonnet policy means MiniMax is never compiled — a structural limit of this trial design, noted for the n=5 read).
- **DQ-blocker rate:** 0/0 on raw count all three tasks. The *quality-inside-the-tie* split: T3 both fine; **T4 MiniMax incorrect-by-guess** (the one real divergence); T5 both correct (ambiguity pre-removed).
- **Wall-clock:** T3 MiniMax-favoured (~33% faster); T4/T5 not comparable.
- **Quality:** T3 tie · **T4 Sonnet wins decisively** · T5 tie. **Net: 1 decisive Sonnet win, 2 ties.**
- **Read at n=3 — the load-bearing interpretation:** the single Sonnet win (T4) is attributable to **brief ambiguity**, and T5 demonstrated it's **recoverable by tightening the brief** (MiniMax then matched Sonnet byte-for-byte). So the honest n=3 read is: *MiniMax is competitive on fully-specified MIRROR-ref tasks but needs tighter briefs than Sonnet (Sonnet degrades more gracefully on ambiguity).* This is NOT yet a "stay Sonnet" verdict — it's a "MiniMax viable IF briefs are tight" signal. The next-phase 2 tasks should be authored at the **same tight-spec bar as T5** to test whether MiniMax holds parity under that regime; if it does, the ≈10× cost saving is real.

---

_Live results file; appended per eligible task. Authored by advisor during m1-b 2026-06-04._
