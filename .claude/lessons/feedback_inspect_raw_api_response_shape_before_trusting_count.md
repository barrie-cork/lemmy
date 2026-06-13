---
name: Inspect the raw API response shape before trusting a parsed count or field
description: When verifying that a pipeline worked by parsing a JSON API response, a wrong assumption about the top-level key silently yields an empty/zero result that reads as "feature broken." Before concluding absence ("0 pending", "no rows", "not found"), dump the raw response (head -c 500) and confirm the actual key path. On 2026-06-13 a parser keyed registration_applications[] when the API returns items[], falsely reporting "0 pending" twice on a working register pipeline.
type: feedback
---

When you verify that something worked by **parsing a JSON API response and reading a count or field**, a wrong guess about the response *shape* (the top-level key, the nesting, the array name) produces a silent empty/zero result — and an empty result reads as **"the feature is broken"** when the feature is actually fine and only the parser is wrong. This is a `pattern_verify_before_trusting_shell_output` case applied to HTTP/JSON: the parser, like a pipe or a `wc`, can lie about the underlying reality.

## The incident (2026-06-13, pilot register pipeline)

Verifying that a UI signup landed in the admin queue. The check parsed the registration-applications list API for `registration_applications[]` and reported `pending count: 0` — twice — concluding the pipeline might be broken. The account was provably created (login returned `registration_application_is_pending`), so the "0" was contradicted by other evidence. A `head -c 500` of the **raw** response showed the real shape:

```json
{"items":[{"registration_application":{"id":13,"answer":"...","published_at":"..."}, ...}]}
```

The API returns `items[]`, not `registration_applications[]`. The parser keyed a field that doesn't exist → empty list → false "0". Two verification cycles were wasted before the raw dump exposed the mismatch.

## How to apply

1. **Before concluding absence from a parsed response** — "0 results", "no rows", "not found", "queue empty" — dump the raw response first and read the actual key path:
   ```bash
   curl -s "$URL" -H "$AUTH" | head -c 500          # see the real shape
   curl -s "$URL" -H "$AUTH" | python3 -m json.tool | head -30   # or pretty-print top of it
   ```
   Only trust a parsed `len(items)` once you've confirmed `items` is the right key on a sample with known-nonzero content.

2. **Cross-check a "zero" against independent evidence.** Here, the login attempt returning `registration_application_is_pending` flatly contradicted "0 pending applications". When a parsed count disagrees with another signal, trust the disagreement — it's almost always the parser, not reality.

3. **Don't author the key from memory.** API response shapes drift between versions (Lemmy `items[]` vs an older `registration_applications[]`; `data[]` vs `results[]` vs a bare array). The 5-second raw dump beats a confident wrong guess that costs a verification cycle and risks a false "broken" conclusion in a report or to the user.

## Generalises to

Any verification where the success signal is read out of a structured response you didn't author: REST/JSON APIs, `gh ... --json` output, `jq` over a tool's stdout, an MCP tool result. The discipline is the same — confirm the shape on a known-good sample before trusting the parse, and treat a "0/empty/absent" that contradicts other evidence as a parser bug until proven otherwise.

## See also

- `pattern_verify_before_trusting_shell_output` (PMD pattern) — exit codes, pipes, `wc`, `ps`, wrappers all lie; this lesson is the JSON-API-response-shape instance of the same discipline.
- `feedback_falsifiable_hypothesis_before_structural_fix.md` — the broader "the named premise is a hypothesis, falsify cheaply before acting" discipline; a false "0 results" is a premise worth one raw dump to falsify.
- `.claude/rules/universal-guards.md` §3 (Input validation) — "Search returns 0 results → try at least one alternative before concluding absence."
