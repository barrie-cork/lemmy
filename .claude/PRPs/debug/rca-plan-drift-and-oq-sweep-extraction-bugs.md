---
issue_slug: plan-drift-and-oq-sweep-extraction-bugs
created: 2026-04-18
mode: deep
---

# Root Cause Analysis — plan-drift `code_route_count=0` and oq-sweep OQ-006 garbled verdict

**Issue 1**: `plan-drift.yml` reports `code_route_count: 0` and 11 disagreement rows even though the governance routes ARE wired in code.
**Issue 2**: `oq-sweep.yml` aggregate report cell for OQ-006 shows `Verdict: ###` and `First line: Date: 2026-04-14`, while the `unknown` counter correctly increments to 1.
**Severity**: Medium — both are workflow extraction bugs that produce false-red signals; neither corrupts on-disk governance state. They will, however, generate spurious GitHub Issues (plan-drift) and degrade the operational signal of the sweep.
**Confidence**: High for both. Both reproduced locally against the current `phase-5b` HEAD.

---

## Issue 1 — plan-drift extracts no routes

### Symptom

`docs/reports/plan-drift/<date>.md` reports `code_route_count: 0` with 11 disagreement rows; every plan endpoint shows ✅ in the "In plan" column and `—` in "In code". Reproduced locally:

```
=== PLAN (11 lines) ===
GET /api/v4/governance/case
GET /api/v4/governance/cases
... (9 more, all from 05 §2)
=== CODE GREP TARGETS ===
0 /tmp/routes.raw
```

### Evidence chain (5 Whys)

**WHY 1**: Why does `code_route_count: 0`?
→ Because `/tmp/routes.raw` is empty after the `Extract routes wired in code` step.
→ Evidence: `.github/workflows/plan-drift.yml:67-75` greps three paths into `/tmp/routes.raw`:
```yaml
grep -rnE '\.route\s*\(\s*"[^"]+"' \
  crates/api/api/src/governance/ \
  crates/api/api_crud/src/governance/ \
  crates/server/src/governance.rs 2>/dev/null || true
```
Local reproduction shows zero matches across these three paths.

**WHY 2**: Why is `/tmp/routes.raw` empty?
→ Because none of the three grep targets contain `.route(...)` calls. Handler files in `crates/api/api/src/governance/` and `crates/api/api_crud/src/governance/` define handler functions; they do not register routes. `crates/server/src/governance.rs` is a 28-line composition stub that only logs at startup.
→ Evidence:
- `crates/server/src/governance.rs:22-28` — only contains `pub fn schedule_governance_jobs(_context: &LemmyContext) { info!(...) }`.
- `ls crates/api/api/src/governance/` — 14 handler files; none register routes.
- `ls crates/api/api_crud/src/governance/` — 3 files (`create_endorsement.rs`, `create_report.rs`, `mod.rs`); none register routes.

**WHY 3**: Why don't those paths contain route registration?
→ Because Lemmy 1.0-beta concentrates all route wiring in a single file: `crates/api/routes/src/lib.rs`. The governance scope is registered there at lines 505–522 inside the top-level `/api/v4` scope.
→ Evidence: `crates/api/routes/src/lib.rs:505-522`:
```rust
.service(
  scope("/governance")
    .wrap(rate_limit.post())
    .route("/report", post().to(create_report))
    .route("/endorsement", post().to(create_endorsement))
    .route("/case", get().to(get_case))
    .route("/modlog", get().to(list_modlog))
    .service(
      scope("/jury")
        .route("/me", get().to(list_my_jury_queue))
        .route("/vote", post().to(submit_jury_vote)),
    )
    .service(
      scope("/admin")
        .route("/assign-jury", post().to(admin_assign_jury))
        .route("/close-case", post().to(admin_close_case)),
    ),
),
```

**WHY 4**: Why does the workflow grep the wrong paths?
→ Because the workflow author assumed there would be a `crates/api/routes/src/governance.rs` file (matching the per-feature route-file pattern that exists in some other Lemmy crates) or that wiring would live in `crates/server/src/governance.rs`. Neither assumption holds: `routes` is a single-file crate (`Glob crates/api/routes/**/*.rs` returns only `lib.rs`), and the server-side governance file is a stub.
→ Evidence: `crates/api/routes/` contains only `lib.rs`. CLAUDE.md's expected paths section names `crates/api/routes/src/governance.rs` as an "expected" path (line 87), reinforcing the mistaken assumption.

**WHY 5 / ROOT CAUSE**: The grep target list in `plan-drift.yml:69-72` does not include `crates/api/routes/src/lib.rs`, where routes are actually registered.
→ Evidence: `.github/workflows/plan-drift.yml:69-72`. Adding `crates/api/routes/src/lib.rs` to the grep target list (or replacing the three current targets with just that file) would make the extraction find all 7 currently-wired routes.

### Secondary issue — even if the grep target were fixed, the prefix-stripping is missing

The plan in `05 §2` lists endpoints as `POST /api/v4/governance/report`. The router declares them as nested scopes: outer `/api/v4` (line 203), then `/governance` (line 506), then `/jury` or `/admin` (lines 513, 518), then `.route("/report", ...)`. The current awk extractor on `plan-drift.yml:77-84` would emit only `POST /report`, never `POST /api/v4/governance/report`. So fixing the grep target is necessary but not sufficient — the extractor must also reconstruct the full path by walking enclosing `scope("...")` frames.

### Fix specification

Two changes to `.github/workflows/plan-drift.yml`:

**Change A — point grep at the real route file** (lines 64-87):

```yaml
- name: Extract routes wired in code
  id: code_routes
  run: |
    # Lemmy concentrates routes in crates/api/routes/src/lib.rs as a single
    # nested actix-web scope tree. Walk that file with awk so we can
    # reconstruct the full path from the enclosing scope("...") frames.
    python3 - <<'PY' > /tmp/code.txt
    import re
    routes_file = "crates/api/routes/src/lib.rs"
    scope_re   = re.compile(r'scope\(\s*"([^"]+)"\s*\)')
    route_re   = re.compile(r'\.route\(\s*"([^"]+)"\s*,\s*(get|post|put|delete|patch)\(')

    def strip_strings_and_comments(line: str) -> str:
        # Remove `// ...` line comments and "..." string contents so
        # paren counting only sees structural parens.
        ...

    stack: list[tuple[str, int]] = []  # (path_segment, paren_depth_when_opened)
    depth = 0
    with open(routes_file, encoding="utf-8") as fh:
        for line in fh:
            sanitised = strip_strings_and_comments(line)
            for ch in sanitised:
                if ch == "(":
                    depth += 1
                elif ch == ")":
                    depth -= 1
                    while stack and stack[-1][1] > depth:
                        stack.pop()
            for m in scope_re.finditer(line):
                stack.append((m.group(1), depth))
            for m in route_re.finditer(line):
                full = "".join(seg for seg, _ in stack) + m.group(1)
                # Restrict to governance for this audit
                if full.startswith("/api/v4/governance"):
                    print(f"{m.group(2).upper()} {full}")
    PY
    sort -u -o /tmp/code.txt /tmp/code.txt
    echo "Code routes:"
    cat /tmp/code.txt
    echo "count=$(wc -l < /tmp/code.txt)" >> "$GITHUB_OUTPUT"
```

The Python walker gives correct nested-scope reconstruction in ~25 lines; the original awk would need an explicit stack of its own and is harder to read. (If Python is unavailable, the same logic in awk is acceptable but ~3× longer.)

**Change B — leave the awk plan extractor alone**. It already produces fully-qualified paths because `05 §2` writes them out in full.

### Verification

After the fix:
1. Manually run the workflow via `workflow_dispatch`.
2. Inspect the next `docs/reports/plan-drift/<date>.md` — `code_route_count` should be `7` (current Phase 4b state: report, endorsement, case, modlog, jury/me, jury/vote, admin/assign-jury, admin/close-case = 8 actually — recount: report, endorsement, case, modlog, jury/me, jury/vote, admin/assign-jury, admin/close-case = **8** routes).
3. The disagreement count should drop from `11` to `3` (the three Phase 5+ endpoints not yet wired: `/cases`, `/jury/accept`, `/jury/decline`, `/appeal`, `/reputation/me` minus the 8 wired = `11 - 8 = 3` … recount: 11 plan endpoints − 8 wired = 3 still missing if all 8 wired ones are in the plan. Actually plan has `case` not `cases` for Phase 4 mapping; verify against §2 list at audit time.)

The exact target count depends on which routes are wired at audit time. The signal is "extraction now finds the wired routes", not a specific number.

### Files to modify

- `.github/workflows/plan-drift.yml:64-87` — replace grep+awk extractor with the Python scope-walker above.

---

## Issue 2 — oq-sweep aggregate cell for OQ-006 shows raw header

### Symptom

Aggregate report row:
| OQ | Verdict | First line |
|---|---|---|
| OQ-006 | ### | Date: 2026-04-14 |

Counter row: `unknown: 1` (correct).

### Evidence chain (5 Whys)

**WHY 1**: Why does the OQ-006 cell show `Verdict: ###`?
→ Because `aggregate.steps.report.run` extracts the verdict from the artefact with `head -n 1 "$result_file" | tr -d '\r' | awk '{print $1}'`. The first whitespace-delimited token of the artefact's line 1 is `###`.
→ Evidence: `.github/workflows/oq-sweep.yml:341` reads from the artefact directly; the artefact for OQ-006 begins with `### ADR-supersede — OQ-006 resolution`, whose first awk-token is `###`.

**WHY 2**: Why is line 1 of the OQ-006 artefact `### ADR-supersede — OQ-006 resolution` instead of one of the verdict tokens?
→ Because the model produced the supersede ADR markdown block at the top of its response without prefacing it with the required `IMPLICITLY-RESOLVED` line-1 token. The system prompt and output contract both require line 1 to be exactly `STILL-OPEN`, `IMPLICITLY-RESOLVED`, or `CONTRADICTED-BY-CODE`, but the model elided the verdict and dove straight into the supersede block.
→ Evidence: `.github/workflows/oq-sweep.yml:147-164` — the prompt asks for line 1 to be a verdict, then "If IMPLICITLY-RESOLVED or CONTRADICTED-BY-CODE, emit a supersede-ADR block exactly:" followed by a fenced markdown template starting with `### ADR-supersede — ${OQ} resolution`. A small model (default `mistral-ai/ministral-3b`) often skips the verdict line and emits the template body verbatim.
→ Corroborating context: OQ-006 was resolved in code yesterday (commit `1682a544f`, `feat(governance): task 58 — OQ-006 threshold formula in create_report`). The grep evidence step would have surfaced concrete file:line matches in `crates/api/api_crud/src/governance/create_report.rs`, so the model's *substantive* judgment of IMPLICITLY-RESOLVED was correct — only the format compliance failed.

**WHY 3**: Why didn't the `infer` step's `case` validator turn this into a `STILL-OPEN` artefact?
→ Because the `infer` step normalizes only the GitHub-output `verdict` variable, not the artefact file. After the `case` block sets `verdict=STILL-OPEN` in the GitHub output, `oq-result.md` is left untouched on disk and uploaded as-is.
→ Evidence: `.github/workflows/oq-sweep.yml:214-223`:
```bash
verdict=$(head -n 1 /tmp/oq-result.md | tr -d '\r' | awk '{print $1}')
case "$verdict" in
  STILL-OPEN|IMPLICITLY-RESOLVED|CONTRADICTED-BY-CODE) ;;
  *)
    echo "::warning::${OQ}: no valid verdict on line 1 — got: '$verdict'. Defaulting to STILL-OPEN."
    verdict=STILL-OPEN
    ;;
esac
echo "verdict=$verdict" >> "$GITHUB_OUTPUT"
```
The `verdict` shell var is updated, but `/tmp/oq-result.md` is not rewritten. Meanwhile the `Open PR on IMPLICITLY-RESOLVED` step (lines 244-300) gates only on the GitHub-output `verdict` (so no spurious PR was created — the bug is contained).

**WHY 4**: Why does the aggregate use the artefact's line 1 instead of the GitHub-output verdict?
→ Because matrix-job outputs are awkward to fan out across `max-parallel: 4` shards (each shard's `outputs.verdict` is keyed by job index, not OQ id). The aggregate avoids that mess by re-parsing artefacts uniformly. That design is fine; the bug is just that the *artefact* never reflects the normalization the `infer` step performed.
→ Evidence: `.github/workflows/oq-sweep.yml:86-87` — the `sweep` job declares only `outputs.oq: ${{ matrix.oq }}`, not `verdict`. The aggregate has no access to the per-shard `infer.outputs.verdict`.

**WHY 5 / ROOT CAUSE**: The `infer` step's normalization is one-sided. It sanitizes the verdict variable (which gates the IMPLICITLY-RESOLVED PR step) but does not sanitize the artefact (which feeds the aggregate). The aggregate then re-parses the un-sanitized artefact, getting whatever the model emitted — `###` in this case.
→ Evidence: `.github/workflows/oq-sweep.yml:214-223` mutates `$verdict`; nothing rewrites `/tmp/oq-result.md`.

### Why the counter is right but the cell is wrong

The aggregate's `case` block (`oq-sweep.yml:344-349`) classifies anything not in the three valid tokens as `unknown`, so `###` correctly increments `unknown` to 1. The cell rendering, however, uses `${verdict:-UNKNOWN}` — the `:-` default applies only when `$verdict` is **empty or unset**. `###` is non-empty, so the literal `###` is rendered.

### Fix specification

Two changes, either of which alone is sufficient. Both is best (defence in depth):

**Change A — sanitize the artefact in the `infer` step.** Replace lines 214-223 in `.github/workflows/oq-sweep.yml`:

```yaml
verdict=$(head -n 1 /tmp/oq-result.md | tr -d '\r' | awk '{print $1}')
case "$verdict" in
  STILL-OPEN|IMPLICITLY-RESOLVED|CONTRADICTED-BY-CODE) ;;
  *)
    echo "::warning::${OQ}: no valid verdict on line 1 — got: '$verdict'. Defaulting to STILL-OPEN."
    verdict=STILL-OPEN
    # Persist the normalisation to the artefact so the aggregate sees the
    # same verdict token. Without this, aggregate re-parses raw line 1 and
    # renders garbage like '### ADR-supersede' as the verdict cell.
    {
      echo "$verdict"
      echo "(verdict auto-normalised by infer step on $(date -u +%Y-%m-%dT%H:%M:%SZ); model produced no valid line-1 token; original output below)"
      echo
      cat /tmp/oq-result.md
    } > /tmp/oq-result.normalised.md
    mv /tmp/oq-result.normalised.md /tmp/oq-result.md
    ;;
esac
echo "verdict=$verdict" >> "$GITHUB_OUTPUT"
```

This keeps the original model output (now starting at line 4) for diagnosis while making line 1 a valid verdict token. The aggregate will see `STILL-OPEN` as line 1 and `(verdict auto-normalised…)` as line 2 — clear, machine-parseable, human-readable.

**Change B — harden the aggregate cell render** (`oq-sweep.yml:350`):

```diff
-              table+="| ${oq} | ${verdict:-UNKNOWN} | ${first_line} |\n"
+              # Belt-and-braces: only render verdict if it is a known token;
+              # otherwise show UNKNOWN to avoid leaking raw markdown headings.
+              case "$verdict" in
+                STILL-OPEN|IMPLICITLY-RESOLVED|CONTRADICTED-BY-CODE) display="$verdict" ;;
+                *) display="UNKNOWN" ;;
+              esac
+              table+="| ${oq} | ${display} | ${first_line} |\n"
```

This protects the aggregate even if a future change to the `infer` step regresses Change A.

### Verification

1. Fire `workflow_dispatch` with `only_oq: OQ-006` to re-run the sweep against the same OQ that exposed the bug.
2. Inspect the next aggregate report:
   - Cell should read `| OQ-006 | STILL-OPEN | (verdict auto-normalised…) |` if the model again skips the verdict, OR `| OQ-006 | IMPLICITLY-RESOLVED | <real second line> |` if the model now complies.
   - `unknown` counter should be `0`; if the model skips the verdict, Change A rewrites line 1 to `STILL-OPEN` so the aggregate increments `still_open` (not `unknown`). The diagnostic text surfaces on line 2 via the sanitised artefact.
3. Confirm no `Open PR on IMPLICITLY-RESOLVED` PR is created when the model produces garbled output (already correctly gated; this just confirms no regression).

### Files to modify

- `.github/workflows/oq-sweep.yml:214-223` — Change A (sanitize artefact).
- `.github/workflows/oq-sweep.yml:350` — Change B (defensive cell render).

---

## Git history (both issues)

```
$ git log --oneline -- .github/workflows/plan-drift.yml .github/workflows/oq-sweep.yml
e3f335a78  …(CI review stack live, per memory `project_ci_review_stack.md`)
```

Both workflows were introduced in the CI review stack commit (`e3f335a78`, 2026-04-16). They have not been touched since. The plan-drift issue would have shown up on the very first run because the grep targets never matched anything; the oq-sweep issue surfaces only when a model emits a non-compliant response, which happens stochastically with `mistral-ai/ministral-3b` at temperature 0.1.

**Type**: Both are original bugs in the audit workflows, not regressions. Both are extraction-logic bugs as the user already diagnosed.

---

## Hypotheses ruled out

| Hypothesis | Why ruled out |
|---|---|
| `governance-v0` is checked out at a stale commit that genuinely has no routes | Routes confirmed at `crates/api/routes/src/lib.rs:505-522` on current `phase-5b` HEAD; phase-5b descends from governance-v0 with route work landed in Phase 4a (commit `e82534667`). |
| The `awk` regex `/\.route\s*\(/` is malformed | Tested locally against `crates/api/routes/src/lib.rs` with the same regex via grep; it matches all 8 governance routes. The regex is fine; the problem is the file isn't in the grep target list. |
| OQ-006 verdict was correctly emitted as IMPLICITLY-RESOLVED but the aggregate's `head -n 1` is broken | `head -n 1` works correctly; the bug is upstream — the model's output truly does start with `### ADR-supersede`. The infer step's normalisation never reaches the artefact. |
| The aggregate's `${verdict:-UNKNOWN}` should produce `UNKNOWN` for `###` | `:-` only kicks in when the variable is unset or empty; `###` is non-empty, so the literal renders. Fix is the explicit `case` in Change B. |

---

## Cross-cutting observation

Both bugs share a structural pattern: **a workflow step normalises a value into a GitHub output but does not normalise the on-disk artefact that downstream steps re-read.** plan-drift's authors did the same thing in spirit (they assumed a file path that doesn't exist instead of grepping the file that does). When designing audit workflows, treat the artefact as the source of truth — anything that gates downstream behaviour on a value should also rewrite the artefact so the value survives the marshalling boundary between jobs.

This is worth a `feedback_*.md` memory entry once the fixes land: *"In matrix → aggregate workflows, normalise the artefact, not just the GitHub output."*
