# Brief: v1-quality-r2a fix-impl-2

`[role:impl-task] v1-quality-r2a fix-impl-2 — cr-6 plan-text update — see .claude/PRPs/briefs/v1-quality-r2a-fix-impl-2.md`

## 1. Role + dispatch

Junior `impl-task` subagent (Sonnet 4.6) executes a single-file plan-text update on `phase-v1-quality-r2`. Resolves blocker DQ `745da950edad-001` raised by fix-impl-1 worker (Junior #498). fix-impl-1's brief §2.4 Edit C.3 had a wrong `old_string` — the worker correctly refused to improvise. This brief provides the correct `old_string` per advisor verification of plan file at `phase-v1-quality-r2 @ 3bff55041`.

## 2. Scope

**One file to edit, one Edit, two lines changed.**

- `.claude/PRPs/plans/v1-quality-r2a.plan.md` — update risks-table row at line 672 (the `dq-lint-durations.sh regex misses` row) to align mitigation text with the actual script behaviour after Edit A's ValueError handler shipped in commit `8621a84bb`.

**Zero changes outside the plan file.** No code edits. No new tests. No new DQ entry beyond the impl-raised validate-pending-laptop at end + this brief's resolving the existing blocker DQ.

### 2.1 §G4 NON-allowlist

This is a CR-triage continuation, not a §G4 workflow-fail classifier match. No verbatim §G4 row required. The recipe in §2.2 is the contract; if the Edit anchor diverges, raise `kind: "blocker"` DQ — do NOT improvise.

### 2.2 Edit A — `.claude/PRPs/plans/v1-quality-r2a.plan.md` line 672 risks-table row

**Anchor verified by advisor 2026-05-29 against `bash scripts/brehon/git-show-json.sh 8621a84bb .claude/PRPs/plans/v1-quality-r2a.plan.md` lines 670-672.** The risks table at §18 (line 666 H2 + line 668 header + line 669 separator) has three data rows at lines 670, 671, 672. Edit C.2 from fix-impl-1 already corrected line 671 "4 entries" → "3 entries". This Edit targets line 672 to reflect that the script now has explicit ValueError handling (per Edit A from fix-impl-1: `parse()` wraps `datetime.fromisoformat()` in `try/except ValueError`, returning `None` on malformed input).

Use `old_string` / `new_string` (exact match, 1 occurrence):

`old_string`:

```
| `dq-lint-durations.sh` regex misses an edge-case timestamp format | LOW | MED | Python `fromisoformat` handles all valid ISO-8601; non-ISO timestamps surface as `None` and skip the comparison |
```

`new_string`:

```
| `dq-lint-durations.sh` regex misses an edge-case timestamp format | LOW | MED | Python `fromisoformat` handles all valid ISO-8601; malformed timestamps raise `ValueError` which the `parse()` wrapper catches and returns `None`, so the entry is silently skipped rather than crashing the lint |
```

Two changes from the existing text:
1. "non-ISO timestamps" → "malformed timestamps" (more precise — `fromisoformat` raises `ValueError` for any unparseable input, not strictly "non-ISO")
2. Spell out the `parse()` wrapper's `try/except ValueError` mechanism so the mitigation text **describes the actual script behaviour as shipped in `8621a84bb`** (cr-6's core ask).

### 2.3 Verification steps (worker runs after Edit)

Worker self-tests in order:

1. **Anchor verification**: `rg "regex misses an edge-case timestamp format" .claude/PRPs/plans/v1-quality-r2a.plan.md` returns exactly **1** match on line 672.
2. **Post-edit text verification**: `rg "the \`parse\(\)\` wrapper catches and returns \`None\`" .claude/PRPs/plans/v1-quality-r2a.plan.md` returns exactly **1** match (confirms new text is present).
3. **Old-text absent verification**: `rg "non-ISO timestamps surface as" .claude/PRPs/plans/v1-quality-r2a.plan.md` returns **0** matches (confirms old text replaced, not appended).
4. **No script changes**: `git diff phase-v1-quality-r2 -- scripts/brehon/dq-lint-durations.sh scripts/brehon/precheck.sh` returns empty (no accidental scope creep).
5. **Plan-only diff**: `git diff --stat phase-v1-quality-r2` shows exactly **1** file changed in `.claude/PRPs/plans/` plus the DQ blocker resolution + the new validate-pending-laptop entry in `.claude/decision-queue.json`.

All five must exit cleanly. Any failure → raise `kind: "blocker"` DQ, do NOT push.

### 2.4 DQ blocker resolution — resolve `745da950edad-001`

Within the same commit that applies Edit A, mutate the pending blocker DQ to resolved with the advisor's option-a answer:

1. Read `.claude/decision-queue.json` fresh.
2. Locate entry with `id: "745da950edad-001"` in `pending[]`.
3. Mutate fields:
   - `answer`: `"option-a: brief old_string was wrong; advisor 2026-05-29 verified actual line 672 risks-table row reads 'dq-lint-durations.sh regex misses an edge-case timestamp format ... Python fromisoformat handles all valid ISO-8601; non-ISO timestamps surface as None and skip the comparison'. fix-impl-2 brief §2.2 provides the correct old_string targeting this row. Updated mitigation text spells out the parse() ValueError wrapper added by Edit A in fix-impl-1 commit 8621a84bb."`
   - `answered_by`: `"impl-self-resolved"` (per DQ Hard refusal #1 — worker MUST NOT write `"answered_by": "advisor"`; the brief carries the advisor's reasoning, but the resolution attribution is the worker's because it's resolving a blocker it raised — analogous to fix-impl-1's pattern. See DQ Recipe 3 in `.claude/refs/dq-recipes.md`.)
   - `resolved_at`: ISO 8601 UTC string for the time of mutation
4. Move the entry from `pending[]` to `resolved[]`.
5. Re-verify the JSON: `python -c "import io, json; dq = json.load(io.open('.claude/decision-queue.json', encoding='utf-8')); assert all(e['id'] != '745da950edad-001' for e in dq['pending']), 'still in pending'; assert any(e['id'] == '745da950edad-001' for e in dq['resolved']), 'not in resolved'; print('OK')"`

This mutation happens **in the same commit as Edit A**, not a separate commit.

## 3. Required reading

Per §2.3 of `.claude/rules/advisor-orchestrator.md`, mandatory file-class lesson injection:

- `.claude/lessons/feedback_falsifiable_hypothesis_before_structural_fix.md` — the blocker DQ was the worker correctly refusing to improvise on a wrong anchor; this fix-impl-2 reflects the brief author having verified the actual file state.
- `.claude/lessons/feedback_dq_v3_append_via_helper_script.md` — for the DQ mutation, fragments + helpers eliminate the heredoc-EOF class.

The Hard refusal #1 DQ attribution rule applies — see `.claude/rules/decision-queue.md` §"Attribution integrity".

## 4. Constraints

### 4.1 File-ownership

- ALLOWED: `.claude/PRPs/plans/v1-quality-r2a.plan.md`, `.claude/decision-queue.json`.
- FORBIDDEN: anything else. Specifically no edits to `scripts/brehon/`, no edits to `crates/`, no edits to `crates/server/tests/`. Zero scope creep.

### 4.2 Worker self-tests in §2.3 + §2.4 step 5

Run all 5 §2.3 + the §2.4 step 5 verification BEFORE `git add`. If ANY exit non-zero or returns unexpected count, raise `kind: "blocker"` DQ — do NOT push.

### 4.3 validate-pending-laptop DQ entry at end

After committing the Edit + DQ mutation, raise a `kind: "validate-pending-laptop"` DQ entry signalling that the laptop advisor should run the §15 commands. Schema:

- `kind`: `"validate-pending-laptop"`
- `from`: `"impl"`
- `branch`: `"phase-v1-quality-r2"`
- `phase_task`: `"v1-quality-r2a-fix-impl-2"`
- `commands`: `["bash scripts/brehon/dq-lint-durations.sh", "bash scripts/brehon/precheck.sh"]` (NO cargo — zero Rust changes)
- `result`: `null`
- `log_slice`: `null`
- `failed_commands`: `null`

Generate the composite v3 id via `bash scripts/brehon/dq-v3-new-entry.sh`. Append via `bash scripts/brehon/dq-v3-append-fragment.sh <fragment> --pending`.

### 4.4 Commit subjects

TWO commits, in order:

1. `feat(quality): fix-impl-2 — cr-6 plan text aligns with shipped Edit A — resolves DQ 745da950edad-001`
2. `chore(decision-queue): impl raised validate-pending-laptop for v1-quality-r2a fix-impl-2`

Per `.claude/rules/decision-queue.md` Attribution integrity — `chore(decision-queue):` is the impl session's signal; do NOT use `chore(advisor)`.

Body for commit 1 ends with the canonical HANDOVER trailer per `.claude/PRPs/templates/impl-task-brief.template.md` §3a discipline:

```yaml
HANDOVER:
  task: v1-quality-r2a-fix-impl-2
  filesCreated: []
  filesModified:
    - .claude/PRPs/plans/v1-quality-r2a.plan.md
    - .claude/decision-queue.json
  keyDecisions:
    - "Option-a per advisor: cr-6 mitigation text updated to describe parse() ValueError wrapper shipped in fix-impl-1 8621a84bb"
    - "DQ 745da950edad-001 resolved with answered_by=impl-self-resolved (worker resolves blocker it raised)"
  notes: "Edit A anchor at line 672 of plan; 2 lines changed (text replacement); resolves last open finding from PR #161 6-finding bundle. After this commit lands and laptop validate-pending mutates to pass, PR #161 is ready for bm-poll-cr re-poll → addressed_in fields update → bm-merge."
```

### 4.5 No cargo

Zero Rust code changes. No `cargo check`, no `cargo clippy`, no `cargo test`. The validate-pending-laptop in §4.3 lists ONLY the bash gates.

### 4.6 Push discipline

After the two commits, `git push origin <worker-branch>` so the advisor's polling loop can observe the validate-pending-laptop entry on next fetch. Standard impl-task push pattern.
