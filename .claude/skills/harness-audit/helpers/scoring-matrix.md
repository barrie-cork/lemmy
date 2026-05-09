# Scoring matrix — composite score for compression candidates

Helper for `harness-audit` Phase 3. Read just-in-time when applying the formula. Do not improvise weights — they are calibrated against the 2026-05-09 trim outcome (commits `59a184ef6..18e4a29c4`, ~3.5k tokens saved) and changing them silently breaks audit-to-audit comparability.

## Inputs (per file)

- `chars`: total character count (from Phase 1).
- `class`: `ALWAYS` or `SCOPED` (from Phase 1).
- `redundancy_count`: number of canonical-source duplicates this file has against another auto-loaded file (from Phase 2 cross-reference grep). Examples of the pattern: validate-pending mutation flow lived in both `advisor-orchestrator.md` §5.2 and `decision-queue.md` "ci-watcher mutation pattern" before the 2026-05-09 collapse — that was `redundancy_count = 1`.
- `external_citations`: number of files outside `.claude/rules/` that cite this file by path or section name (from Phase 2 grep).
- `pi_shared`: boolean. True if any `.pi/` file or `AGENTS.md` references the file. (False is the common case in brehon-fork.)

## Formula

| Factor | Weight | Calculation | Rationale |
|---|---|---|---|
| Always-load | 0.40 | 1.0 if `class == ALWAYS`, else 0.0 | SCOPED files don't auto-load in meta-work sessions; not compression candidates |
| Size | 0.25 | `min(chars / 8000, 1.0)` | 8 KB ≈ 2k tokens; files over that scale are higher-impact compression targets |
| Redundancy | 0.20 | `min(redundancy_count / 2, 1.0)` | A file with 2+ canonical-source duplicates elsewhere is prime extract material |
| Citation drag | 0.10 | `1.0 - min(external_citations / 20, 1.0)` | Many citations = renaming risk; high citation count REDUCES the score (less safe to touch) |
| Pi-shared | 0.05 | `0.0 if pi_shared, else 1.0` | Pi-shared paths get composite ≈ 0 from this term; never compress |

**Composite = (sum of factor × weight) × 10**, range 0–10.

A file with `class == SCOPED` (factor weight 0.40 contributes 0.0) has a hard ceiling of composite 6.0. So all compression candidates with composite ≥ 6.0 are necessarily ALWAYS-load. SCOPED files top out at 6.0 even at max size + max redundancy + zero citations + Pi-safe — by design.

## Buckets

| Bucket | Composite range | Action |
|---|---|---|
| **High-confidence wins** | 6.0–10.0 | Recommend explicit action: Read-scope frontmatter / extract to `.claude/refs/` / fold into pattern. Surface in report Phase 4 with token-saving estimate. |
| **Watch items** | 3.0–5.9 | Defer. Cite the trigger that would promote (e.g. "if file grows past +50 lines OR gains a redundancy duplicate"). |
| **Out of scope** | 0.0–2.9 OR `pi_shared == true` OR already-minimum | Explicitly list with reason. |

## Worked example (calibrated against 2026-05-09 trim)

`pm-plugin-hooks-stable.md` (pre-trim, 116 lines, 2.9 KB):

- `class = ALWAYS` → 0.40 × 1.0 = 0.40
- `chars = 2900` → size factor 2900/8000 ≈ 0.36 → 0.25 × 0.36 = 0.09
- `redundancy_count = 0` → 0.20 × 0 = 0.00
- `external_citations = 39` (from the actual grep) → drag factor `1 - min(39/20, 1) = 0` → 0.10 × 0 = 0.00
- `pi_shared = false` (the `.pi/prompts/prp-plan.md` reference is a "see also" pointer, not a load) → 0.05 × 1.0 = 0.05

Composite = (0.40 + 0.09 + 0.00 + 0.00 + 0.05) × 10 = **5.4** — `Watch item`.

Yet the 2026-05-09 trim acted on it (Pass 1a, added `paths:` frontmatter). Why? Because the Pi-impact grep showed citations were "see also" pointers, not load deps — the action was low-risk. The skill's Phase 4 explicitly recommends a Pi-impact grep on every Watch row before deciding; on a 5.4 composite a Pi-safe Read-scope add is a defensible discretionary call. Document the discretion in the report.

## When the matrix is wrong

If the matrix consistently mis-scores against actual trim outcomes (audit recommends X, trim ships X but at wrong-order priority, OR audit misses a real win), file a `kind: log` DQ entry citing the audit run + the actual outcome. Recalibrate at retro time, not mid-audit.
