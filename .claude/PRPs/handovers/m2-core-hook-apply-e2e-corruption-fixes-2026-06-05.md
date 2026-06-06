# RESUME: apply 4 e2e.rs corruption fixes on phase-m2-core-hook

**Authored:** 2026-06-05, end of diagnosis session (advisor, m2-core-hook lane).
**For:** a FRESH session whose sole task is to apply the 4 fixes diagnosed this session.
**Read with zero conversation context.** Everything you need is here + the full report.

---

## RESUME block (read this first)

- **Lane / mode:** m2-core-hook, Mode A. Worktree: `C:/Users/barri/Developer/brehon-fork/brehon-fork-m2`, branch `phase-m2-core-hook`.
- **CWD to open the fresh session in:** `C:/Users/barri/Developer/brehon-fork/brehon-fork-m2` (the temp validation worktree — still present; do NOT remove it until the fixes pass + m2 PR is open).
- **State-machine stage:** m2-core-hook Task 8 e2e gate is OPEN (failed). 8 of N tasks done; Task 8 (e2e tests) blocked on a pre-existing trunk corruption that must be fixed before the e2e suite compiles.
- **Phase-branch tip:** `fcc57549e` (`docs(retro): session retro`). Last code commit: `91c646514` (Task 8 merge). e2e.rs is at **pristine corrupt HEAD** (my partial fix was reverted).
- **NEXT CONCRETE ACTION:** apply the 4 fixes below, re-run m2_ e2e, mutate DQ `1dcd6a201003-002` to pass.
- **Cross-session deps:** DQ pending 1 (`1dcd6a201003-002`, validate-pending-laptop-e2e, `result: fail`). `governance-v0` local tip `40e64a459` (Task-8 DQ merge) **not yet pushed to origin** — push it. No concurrent lanes active.

---

## WHY (one paragraph)

m2-core-hook Task 8 added 3 e2e tests; validating them surfaced that `crates/server/tests/e2e.rs` does not compile. Root cause is a **pre-existing trunk corruption** (NOT Task 8): the botched BUG-1 cherry-pick `6f4947b48` (2026-06-05 13:43) re-injected ~2,100 lines of pre-decomposition test content into `e2e.rs` after that file had been decomposed into a 156-line `include!` host (`0f3531c81`). `governance-v0` has been e2e-uncompilable since 13:43. There is ALSO a separate Task-8 code bug (`CaseStatus::Active` — no such enum variant). **Full diagnosis:** `.claude/PRPs/debug/e2e-rs-botched-cherry-pick-corruption.md`. **Lesson:** `feedback_cherry_pick_onto_restructured_file_reinjects_content.md`. **PMD #831.**

11 compile errors = **9× E0428** (3 corruption layers) + **2× E0599** (Task-8 bug).

---

## THE 4 FIXES (compiler-verified this session; all on `phase-m2-core-hook`)

### Fix #2 — restore e2e.rs to last-good (resolves all 9× E0428)
`0f3531c81` is a clean ancestor of HEAD; the ONLY commit to touch `e2e.rs` since is the corrupting `6f4947b48`, and its entire diff is additive duplicates (every `+async fn` already lives in `governance.rs` via `include!`). Zero legit changes lost.
```bash
cd C:/Users/barri/Developer/brehon-fork/brehon-fork-m2
git checkout 0f3531c81 -- crates/server/tests/e2e.rs
# verify: file is 156 lines, contains ONLY postgres_container_boots + template_dump_capture
#         + the mod common / include! block. NO can_insert_moderation_case / migration tests / MIGRATIONS const.
```
**Restore ONLY e2e.rs.** Do NOT restore governance.rs or anything else to that SHA — governance.rs has legit later changes (M1-b, Task 6 hook wiring, Task 8 tests, BUG-1 emergency-remove test).

### Fix #3 — add BUG-1's migration to the canonical revert list (resolves phase1_revert_list_matches_disk #[test])
BUG-1's revert-list entry landed in the dead duplicate, never reaching `governance.rs`. The canonical list has 20 entries, **missing `2026-06-01-000000-0000_backfill_author_defendant` at index 1** (disk newest-2 = messaging_config 06-03 + backfill_author_defendant 06-01; list has only the first).
- File: `crates/server/tests/e2e/governance.rs`, `const MIGRATIONS_TO_REVERT_PHASE_1` (~line 202-204).
- Insert `"2026-06-01-000000-0000_backfill_author_defendant",` as the **second** entry (right after `add_governance_messaging_config`, before `add_federation_inbound_v1`). List goes 20 → 21 = disk newest-21, contiguous.
- Update the leading count comment (currently "M1-b governance-messaging (1 migration, bump 19 → 20)") to reflect the BUG-1 backfill migration too. Re-run `phase1_revert_list_matches_disk` to confirm parity.

### Fix #4 — Task 8 bug: CaseStatus::Active → Open (resolves both E0599)
`CaseStatus` has no `Active` variant (real: `Open` default, `ThresholdMet`, `JurySelection`, `InReview`, `Decided`, `Appealed`, `Closed`, `EmergencyRemove`, `AdminReview`, `SponsorLiability*`). See `crates/db_schema_file/src/enums.rs:393`.
- File: `crates/server/tests/e2e/governance.rs`, test `m2_hook_suppressed_when_messaging_disabled`, ~lines 5364 and 5377.
- Change both `CaseStatus::Active` → `CaseStatus::Open` (the `status:` field and the `from_status` arg to `governance_case_after_transition`).
- **Four-role note:** advisor must not author `crates/**`. CLEAN option = re-dispatch this 2-token fix as a Junior `fix-impl-task` (brief it, push brief on phase branch, dispatch with `base_branch=phase-m2-core-hook`). Pragmatic option = fix inline given it's 2 tokens and bundled with the corruption repair. **User decided this is a fresh-session task; confirm the role-boundary call at the start of that session.**

(Fix #1 — delete e2e.rs lines 110-893 — is SUPERSEDED by #2's full restore. Do not do it separately.)

---

## AFTER THE FIXES

1. **Re-validate** (Docker must be up; from the brehon-fork-m2 worktree):
   ```bash
   cd C:/Users/barri/Developer/brehon-fork/brehon-fork-m2
   cmd //c "scripts\brehon\cargo-test.bat --workspace --test e2e m2_ > %LOCALAPPDATA%\Temp\m2-e2e-fix.log 2>&1 && echo EXIT_0 >> %LOCALAPPDATA%\Temp\m2-e2e-fix.log || echo EXIT_NONZERO >> %LOCALAPPDATA%\Temp\m2-e2e-fix.log"
   ```
   **TRUST THE IN-LOG MARKER (`EXIT_0`/`EXIT_NONZERO`), NOT the task-notification exit code** — the wrapper's `&& echo … || echo …` makes the notification report 0 even on cargo failure (bit us twice this session). Per `cargo-output-capture.md`.
   Expect: compile clean + 3 m2_ tests pass (`m2_append_room_event_writes_chain_entry`, `m2_append_room_event_rejects_non_room_kind`, `m2_hook_suppressed_when_messaging_disabled`). NOTE: a clean compile also re-runs `phase1_revert_list_matches_disk` (#[test], runs by default) — fix #3 must make it green.

2. **Commit the fixes** on `phase-m2-core-hook` (one commit or split #2/#3 from #4):
   `fix(e2e): restore e2e.rs from botched BUG-1 cherry-pick + revert-list + CaseStatus::Active→Open`
   Body must explain it repairs the `6f4947b48` corruption + the Task-8 enum bug; reference the debug report.

3. **Mutate DQ `1dcd6a201003-002` → pass** (currently `result: fail`, in `pending[]`):
   set `result: "pass"`, refresh `answer`/`log_slice`, `answered_by: "advisor-laptop"`, `resolved_at: <now>`, **move from `pending[]` to `resolved[]`**. Atomic read-mutate-write (multi-lane Hard refusal #6).

4. **Push** `phase-m2-core-hook`. Also **push `governance-v0`** local tip `40e64a459` to origin (`git -C C:/Users/barri/Developer/brehon-fork push origin governance-v0`, or from the daemon — it was unpushed at end of the diagnosis session).

5. **Decide the trunk question (surface to user):** `governance-v0` carries the same corruption (the fix is only on the phase branch). It propagates to trunk at m2 PR merge. Other lanes can't run e2e until then — ask the user whether a **direct trunk hotfix** is warranted or m2-PR-merge is soon enough. (See debug report §"Blast radius" note 1.)

6. **Then resume normal m2 flow:** Task 8 done → next task or `/brehon-verify` → bm-pr. Remove the `brehon-fork-m2` temp worktree only AFTER the e2e passes and you no longer need a local validation runner (or keep it if m2 has more e2e tasks). The retro proposes a trunk compile-only CI gate (change #3) — that's a separate follow-up, user-gated against the Shape-G suspension.

---

## VERIFIED_AT

- Corruption root cause + 4 fixes verified at HEAD `91c646514` (code) / report committed `1a086e631` / retro `fcc57549e`.
- `0f3531c81` confirmed ancestor of HEAD; its e2e.rs has 0 of the 9 duplicated definitions.
- `CaseStatus` variants read from `crates/db_schema_file/src/enums.rs:393` (no `Active`).
- Revert-list drift: disk newest-2 = [messaging_config 06-03, backfill_author_defendant 06-01]; canonical list missing the 06-01 entry.
- DQ `1dcd6a201003-002` mutated to `result: fail`, stays pending, this session.

---

## DECISIONS BAKED IN (user-confirmed 2026-06-05, fresh session pre-flight)

A short canonical session opened in `brehon-fork` (governance-v0) by mistake, surfaced the lane + Fix-#4 questions to the user, recorded the answers here, then stopped (did NOT touch the phase branch). The reopened `brehon-fork-m2` session does NOT need to re-ask:

1. **Lane / CWD:** user will reopen the working session with **CWD = `C:/Users/barri/Developer/brehon-fork/brehon-fork-m2`** (Mode A, dedicated lane worktree). Do all work there. Do NOT drive from canonical.
2. **Fix #4 (`CaseStatus::Active` → `Open`):** apply **INLINE** (user chose pragmatic), bundled with Fixes #2/#3 in the repair — no Junior fix-impl-task dispatch. Rationale: 2-token change, and #2/#3 are also `crates/tests` edits in the same cohesive corruption-repair commit; routing one line through a full Junior cycle is disproportionate. (Four-role note acknowledged + user-waived for this bundled repair.)

**VERIFIED current phase tip (re-checked this pre-flight): `a12838ab5`** (`chore(decision-queue): advisor-laptop FAIL DQ …`) — two commits past the doc's stated `fcc57549e` (the retro + the DQ-fail commit, both authored after the doc body). `e2e.rs` confirmed still corrupt at **2256 lines** (last-good target = 156). Nothing else moved. The 4 fixes + the `0f3531c81` restore SHA are all still valid against this tip.
