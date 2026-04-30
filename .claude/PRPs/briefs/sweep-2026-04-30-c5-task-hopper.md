# [role:impl-task] sweep-2026-04-30 C5 — task-hopper script + schema (issues #71, #47)

## 1. Dispatch line

`[role:impl-task] sweep-c5-task-hopper — see .claude/PRPs/briefs/sweep-2026-04-30-c5-task-hopper.md`

## 2. Scope

Two issues bundled into one cluster because both touch the task-hopper subsystem:

### #71 — `scripts/brehon/task-hopper.sh:210` dead-code path

The owner file is `$lock_owner_file` (a sibling file to `$lock_dir`), not inside `$lock_dir`. The current line at `task-hopper.sh:210` reads:

```bash
current="$(cat "$staging/owner" 2>/dev/null || cat "$lock_owner_file" 2>/dev/null || true)"
```

The `cat "$staging/owner"` always fails because no file is ever written at that path — it's dead code. **Fix:** drop the dead `cat "$staging/owner" 2>/dev/null ||` part, keeping only `cat "$lock_owner_file" 2>/dev/null || true`. Verify by reading the script's lock-creation path (where `$lock_owner_file` is written) and the lock-recovery path (where it's now read).

### #47 — `.claude/task-hopper.schema.json` lifecycle invariants

The JSON-Schema currently allows `status: "completed"` with `attempts: []` — schema-valid but logically invalid. Encode the state-machine invariants from `.claude/rules/task-hopper.md` (or `.claude/archived-rules/task-hopper.md` if the active rule has moved) §"The four verbs":

- **`status == "completed"`** → `attempts.length >= 1` AND final attempt has `result == "completed"` with non-null `ended_at` and non-null `commit_sha`.
- **`status == "escalated"`** → `issue_url != null` (with rule-noted exception: gh-issue-creation may race an API outage and leave it null pending manual backfill — encode as conditionally relaxed via the rule's documented note).

Use JSON-Schema's `if`/`then`/`else` or `oneOf` to express conditional requirements based on `status` value.

**Out of scope:**
- Do NOT modify `.claude/rules/task-hopper.md` content (issue #52 separately concluded the active rule is archived).
- Do NOT modify any task-hopper-consuming script beyond the dead-code fix in `task-hopper.sh:210`.
- Do NOT modify `.claude/task-hopper.json` data files.

**Boundaries:**
- Edit only `scripts/brehon/task-hopper.sh` (one-line dead-code fix) and `.claude/task-hopper.schema.json` (invariants).
- Two file edits, one commit subject combining both: `chore(scripts,schema): task-hopper.sh dead-code fix + schema lifecycle invariants (closes #71, closes #47)`.

## 3. Required reading

- **`scripts/brehon/task-hopper.sh`** entire file — to understand `$lock_owner_file` and `$staging/owner` paths
- **`.claude/task-hopper.schema.json`** entire file — current schema shape
- **`.claude/archived-rules/task-hopper.md`** — the rule the schema must encode (the rule itself is archived, but its state-machine spec is still the source-of-truth for the schema)
- **GitHub issue #71** body: `gh issue view 71 --repo barrie-cork/lemmy --json body --jq .body`
- **GitHub issue #47** body: `gh issue view 47 --repo barrie-cork/lemmy --json body --jq .body`

## 4. Constraints

**HARD FORBIDS:**
- `cargo *` of any kind on the worker.
- Editing `crates/**`, `migrations/**`, `.github/workflows/**`.
- Editing `.claude/task-hopper.json` (data file, not schema).
- Running the task-hopper helper script (`./scripts/brehon/task-hopper.sh ...`).

**Required behaviour:**
- Single commit subject: `chore(scripts,schema): task-hopper.sh dead-code fix + schema lifecycle invariants (closes #71, closes #47)`.
- Trailers: `Closes: barrie-cork/lemmy#71` and `Closes: barrie-cork/lemmy#47`.
- Push branch and exit.
- DO NOT raise validate-pending DQ — pure shell + JSON-Schema edits, no cargo cycle. Validate the schema by inspecting it against existing `.claude/task-hopper.json` data shapes (read-only, do not modify the data file).

**File-locality:** `scripts/brehon/task-hopper.sh` + `.claude/task-hopper.schema.json` — no overlap with any wave-1 cluster.

**Mid-task DQ push:** schema-encoding ambiguity (e.g. how to express "issue_url null is allowed only when API outage flagged in attempts[]") → `kind: "clarify"`, push, continue.
