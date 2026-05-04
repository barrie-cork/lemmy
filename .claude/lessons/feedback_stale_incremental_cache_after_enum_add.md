---
name: Pre-emptive `cargo clean` before phase-tip e2e on cross-cutting enum-add
description: Windows cargo target/ stale incremental cache after cross-cutting enum-add (3+ new variants × 5+ match sites) reports false-positive E0004/E0277 errors despite source files md5-equalling phase-tip blobs and CI Linux passing the same SHA. Pre-emptive `cargo clean` is symmetric in cost; pay it forward.
type: feedback
---
**Rule:** for sub-phases that add enum variants AND sweep match sites in 5+ files, run `cargo clean` on the laptop BEFORE the phase-tip e2e. Cost is symmetric — ~30 min cold rebuild now vs ~60 min stale-cache recovery later.

**Why this matters:**
- v1-SL-a's first laptop e2e (post-fix-impl-3) reported 9 false-positive E0004/E0277 errors against the 3 new `CaseStatus` variants across 8 files. Source files were verified-exhaustive vs the phase-tip blobs via md5sum, AND the same SHA workspace-check passed on CI Linux.
- Diagnosis: `lemmy_db_schema_file` rmeta rebuilt with new variants, but `lemmy_api` was typechecked against the stale enum view from before Task 5's sweep merged. Cargo's incremental cache mishandles cross-crate enum-add when match sites are touched by a separate task that lands later.
- Diagnostic command (~5 min): `git ls-tree origin/<branch> <file> | awk '{print $3}' | xargs -I {} git cat-file -p {} | md5sum` vs `md5sum <local-file>`. If md5sums match AND CI passed the same SHA, suspect stale incremental cache.
- Recovery: `cargo clean` (93.7 GB removed in SL-a's case) + cold rebuild (~30 min) + e2e exit 0. Total recovery ~64 min. Pre-emptive clean would have cost ~30 min and shifted that time forward, not added to it.

**How to apply:**
- **Trigger condition:** sub-phase plan §13 includes one task that adds enum variants (3+ new arms) AND another task that sweeps match sites (5+ files). The cross-task cargo cache is the failure surface.
- **Advisor gate at phase-tip e2e:** before kicking off `cargo test --test e2e --features full -- --test-threads=1` for the first time on the phase-tip, run `cargo clean` first. Pay the ~30 min cold rebuild upfront.
- **If e2e is already running and reports E0004/E0277 against new variants:** pause, run the md5sum diagnostic, AND check CI Linux on the same SHA. If they all agree the source is correct, kill the cargo run, `cargo clean`, and restart. Don't try to fix source.
- **Symmetric-cost discipline:** `cargo clean` before phase-tip e2e is rarely wasted. The amortised cost over a sub-phase with cross-cutting enum-add is the same whether you pre-clean or recover.
- **Forward gate:** advisor §G4 checklist (proposed v1-SL-a §7 follow-up #4) — add the trigger condition to the §G4 classifier's pre-e2e prep step.

**Generalises to:** any cross-crate cargo work where one crate's pub enum shape changes AND consumers in other crates re-typecheck against it. Not specific to enum-add — also applies to trait method addition, type alias change, struct field add. The Windows cargo cache is the most reproducible surface; Linux/macOS cargo on the laptop are less prone but not immune.

**Symptom to recognise:** local cargo errors with E0004 (non-exhaustive match) or E0277 (trait bound not satisfied) against newly-added enum variants or trait impls, BUT (a) source files md5-equal phase-tip blobs, AND (b) CI on the same SHA passes. The two-conditions-and gates are load-bearing — don't treat E0004 as definitive without the diagnostic.

**Retire when:** cargo's cross-crate incremental cache invalidation handles enum-add cleanly (upstream cargo issue), OR the laptop runs Linux as primary cargo runner. Until then, pre-emptive clean is mechanical discipline. Source: v1-SL-a retro §3.4.
