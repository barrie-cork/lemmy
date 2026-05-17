# Session retro — 2026-05-17 — advisor-v1-ad-e-merge-closeout

**Harness:** claude-code
**Session window:** ~2026-05-17T17:42Z → ~2026-05-17T20:10Z (~150 min, heavily punctuated by ~33-min e2e + Junior waits — active orchestration ≪ wall-clock)
**Branch at start:** `ff35b64cc` (`phase-v1-AD-e`)
**Branch at end:** `497b7326e` (`governance-v0` — phase branch merged + deleted mid-session)
**Files touched:** ~9 (decision-queue.json, 2 runlog spans, pr-133-findings.yaml, verify report, bm-merge brief, 3 lesson files, retro, close-out handover) + 1 cherry-picked rs file
**Commits:** 14 (auto: 0, explicit: 14 — all advisor `chore(advisor)`/`docs(advisor)`/`docs(retro)`/`docs(lessons)`/`docs(handover)`/`chore(decision-queue)`)

## TL;DR

This session closed v1-AD-e end-to-end: rescued the 5th consecutive `#292 stale-base-self-merge` (fix-impl-2 #299), ran the 4-command laptop DoD (all green, full e2e 94/0/5), re-surfaced + cleared user gate 5, dispatched bm-merge (#301 false-failed at daemon finalize but the merge succeeded server-side — correctly diagnosed), fired the L14/L15/L16 belt-and-braces, authored + signed off the per-role phase retro, promoted 3 lessons, and handed the canonical-checkout-bound close-out (PMD sync / DQ archive / #292 issue) to the user as a tracked handover doc. The single most load-bearing finding: **the `#292` daemon bug is now the *norm* on multi-lane phases (5/5 workers this phase), not an exception** — the highest-leverage change is the daemon-side fix (fetch `origin/<phase>:<phase>` before worktree create), filed as a GitHub issue + lesson. Secondary: two self-inflicted Python-backslash-path bugs writing DQ #246 cost ~3 wasted tool calls — a recurring Windows trap now lessoned.

---

## What surprised us

- **bm-merge Junior #301 reported `failed` but the merge had actually succeeded.** The daemon's post-task finalize step (`git phase-v1-AD-e..HEAD --oneline`) choked because the merge had just *deleted* the phase branch — an ambiguous-revision error. `gh pr merge` is server-side, so it landed `486a24c68` on trunk regardless of the worker's stale local worktree. A surprising-but-benign decoupling: the Junior task status is NOT a reliable signal for whether the irreversible action happened. Verifying `gh pr view --json state,mergeCommit` FIRST was the right instinct; a naive "task failed → re-dispatch bm-merge" would have errored on an already-merged PR.
- **The `#292` stale-base pattern fired on 5/5 Junior workers this phase** (every single one — bm-pr, bm-poll-cr, bm-triage, fix-impl-1, fix-impl-2, + the bm-merge worktree). Surprising not that it happened (it was flagged 2-3× in prior retros) but the *100% hit rate* — it is no longer an exception to handle, it is the default path to budget for on a multi-lane phase. The implementation itself was clean one-pass; ~100% of this phase's orchestration cost was rescue overhead.
- **`governance-v0` has NO branch protection** (`gh api .../protection` → HTTP 404 "Branch not protected"). Surfaced while reconciling the bm-merge.md Phase-2.3 "PENDING check → STOP" rule against the pending CodeRabbit re-review. This means CodeRabbit is *advisory*, not a required check — which cleanly resolved what looked like a gate conflict. Mildly surprising that a constraint the merge-gate script treats as hard is actually soft on this repo's config.
- **Two consecutive Python backslash-escaping bugs** writing DQ #246's `commands[]` (`scripts\brehon` → `scriptsrehon` from `\b`→backspace; then `\'` SyntaxError aborting the whole script). Surprising how silent the first one was — valid JSON, file looked written, corruption only visible on a field-by-field re-read. The verify-script check was *itself* a third instance (`'scripts\brehon\'` is an unterminated literal).

## What to change

| # | Change | Expected effect | Cost | Recurrence |
|---|---|---|---|---|
| 1 | **File the daemon-side `#292` fix** (daemon must `git fetch origin <phase>:<phase>` before each `git worktree add`, OR fork from `origin/<phase>` not the local ref). Lesson `feedback_junior_292_stale_base_recover_recipe.md` authored; GitHub-issue body + homeserver-doc pointer drafted in the close-out handover. | Eliminates the root cause of 5/5 worker rescues on multi-lane phases — the single biggest time sink of the phase | major (daemon code change, owned upstream) | 5× this session, 3×+ prior phases |
| 2 | **Author DQ/JSON values containing Windows backslash paths via a Write-tool JSON fragment + read-merge, NEVER inline `python -c`/heredoc with the path in the source.** Lesson `feedback_windows_backslash_path_dq_via_write_fragment.md` authored. | Removes the silent `\b`→backspace path-mangle class entirely; saves the ~3-tool-call re-derivation that hit this session | minor (behavioral; lesson written) | 2× this session (#246 write + verify-script), 0× prior as a *distinct* lesson |
| 3 | **impl-task briefs enumerating N targets must add an all-N-or-blocker §4 constraint + a mechanical field→fn mapping table.** Lesson `feedback_impl_task_enumerated_transform_all_or_blocker.md` authored; fix-impl-2's table form is the model (byte-exact result vs fix-impl-1's silent-partial). | Prevents the cr-5-class silent-partial against a hard ADR; the advisor diff-vs-enumeration spot-check stays as the catch | minor (brief-template + lesson) | 1× this session (cr-5), 1× prior (retro #362 flagged) |
| 4 | **Amend `bm-merge.md` Phase 2.3 to distinguish required-check-pending (hard STOP) from advisory-check-pending + user-confirmed (proceed-with-DQ-log).** Recorded as DQ #247 + proposed in the phase retro §3. | Stops every future phase re-deriving the "is CodeRabbit a required check?" reconciliation at the merge gate | minor (one section edit in `.claude/commands/bm/bm-merge.md`) | 1× this session, recurring-latent (every phase hits the merge gate) |
| 5 | **On Windows, never pass two refs to one `git rev-parse --short A B`** when cwd may contain odd-named untracked files (the `_assist.txt`-with-`(` junk mangled the shell 3× this session). Use single-ref rev-parse or `git -C`. | Removes the recurring parens-mangle round-trip | trivial (behavioral) | 3× this session, recurring cross-platform class |

## What to carry forward

- **Never trust the daemon's "done/succeeded".** Every Junior completion this session was inspected with `git merge-base --is-ancestor` + `git diff --stat` + blob-SHA spot-check BEFORE advancing. This caught the #292 rescue AND disambiguated the #301 false-failure. This is now non-negotiable discipline on any Junior-dispatched phase.
- **Verify cargo via the explicit `CMDn_EXIT_0` marker + error/warning grep, never the bg-completion notification exit code.** All 4 DoD commands gated this way, strictly sequential (cmd-N+1 not started until cmd-N's marker confirmed). The render-path test (`admin_audit_html_returns_html_for_admin`) was the specific signal proving the `scrub_json().to_string()` shape correct — knowing *which* test proves *which* change is the discipline.
- **Checkpoint-reuse:** reuse the one full e2e run as the `/brehon-verify` §16a story checkpoint rather than re-running a 2nd ~33-min e2e. Saved ~33 min, zero signal loss (one binary covers all 3 stories + the regression guard).
- **Surface hard-ADR partials as a sharp user judgment call, not a vague question.** The cr-5 partial→gate-5→fix-impl-2 arc is the reference pattern: advisor catches via diff-vs-enumeration spot-check, does NOT adjudicate (never authors content), frames it precisely (which fields, what exposure, concrete options), user decides.
- **Multi-lane discipline held under pressure:** did the L14 runlog re-apply by ff-ing the lane worktree's local governance-v0 to origin (the canonical checkout was divergent-detached); did NOT `reset --hard` the divergent canonical checkout; handed canonical-checkout-bound close-out (PMD/DQ-archive/issue) to the user rather than risk the multi-lane DQ-divergence the rules forbid. Lane-safety reasoning was explicit at each branch point.
- **Atomic read-mutate-commit-push + origin-survival re-read** for every DQ/YAML/report write (multi-lane §6). No clobbered concurrent-session write this session.

---

## Three-signal scoring

Per `feedback_four_role_retro_signals.md`. Numbers defensible from the transcript, not exact.

| Skill / Agent / Command | Saved (min) | Wasted (min) | Surprise | Notes |
|---|---:|---:|---|---|
| #292 rescue recipe (cherry-pick clean commit) | ~40 | 0 | low | 5th application; reliable; would-have-deleted 2596 advisor lines if naively finalize-merged |
| DoD 4-cmd laptop validation (cargo-output-capture discipline) | ~10 | 0 | none | all green; explicit-marker gating worked; e2e proved scrub_json shape |
| `/brehon-verify` (checkpoint-reuse) | ~33 | 0 | none | reused #246 e2e; 3/3 ✓; S2.5 idiomatic-scope disposition unchanged (correct) |
| Gate 5 re-surface (AskUserQuestion) | ~5 | 0 | none | clean confirm; cr-5-now-complete framing |
| L15 inline merge-gate pre-checks | ~8 | 0 | medium | surfaced no-branch-protection → CR-advisory reconciliation; DQ #247 |
| bm-merge dispatch (#301) | ~3 | ~8 | high | merge succeeded server-side BUT task false-failed at daemon finalize; ~8 min diagnosing false-vs-real-failure |
| L14 belt-and-braces runlog re-apply | ~5 | ~6 | medium | BM skipped its runlog commit; canonical-checkout divergent → had to ff lane worktree's gov-v0 instead (~6 min detour) |
| Phase retro authoring (per-role, complexity scores) | ~15 | 0 | none | gate 6 signed off; 3 lessons promoted |
| Gate 6 sign-off (AskUserQuestion) | ~3 | 0 | none | clean approve + phase-transition |
| DQ #246 reconstruction (inline python -c) | 0 | ~10 | high | TWO self-inflicted backslash bugs (`\b`, `\'`) + a fragment-file workaround → ~10 min churn; → "What to change" #2 |
| Windows shell parens-mangle (rev-parse A B) | 0 | ~4 | low | 3× recovered round-trips; → "What to change" #5 |
| general-purpose subagent | — | — | — | NOT used this session (prior session did the 1.45M-char log analysis; noted for completeness) |

## Complexity scores (heavy tasks only)

Per `feedback_retro_task_complexity_score.md`. `<files>/<commits>/<runtime-min>/<max-log-silence-min>`. Flag >55min runtime / >40min silence / >8 files.

| Task | Files | Commits | Runtime (min) | Max log silence (min) |
|---|---:|---:|---:|---:|
| DQ #246 4-cmd DoD (laptop, incl ~33min full e2e) | 1 (DQ) | 1 | ~46 (4 cmds: 2+2+2+34 + verify) | ~34 (cmd-4 e2e bg, expected) |
| #299 rescue + DQ #246 reconstruction | ~3 | 4 | ~25 | n/a (advisor-interactive) |
| Phase retro + 3 lessons + close-out doc authoring | 5 | 4 | ~30 | n/a |
| bm-merge #301 (Junior, false-failed) + L14/L15/L16 | ~2 | 3 (advisor) | ~15 (incl #301 ~4min) | n/a |

No task exceeded the >55min-runtime / >8-files flags. The cmd-4 e2e ~34min silence is expected (bg full e2e), not a watchdog risk (advisor-laptop, not Junior).

## Decisions to revisit

- The **canonical `brehon-fork` checkout is in a divergent detached-HEAD state** (`e706cdefe`, couldn't ff governance-v0). Flagged to the user in the close-out doc; needs a deliberate non-destructive untangle before the next canonical-checkout operation. Worth its own short follow-up — not a `reset --hard`.
- **`/brehon-verify` route-literal descriptor false-phantom** (idiomatic `scope("/audit").route("/view")` trips the literal `.route("/audit/view"` grep). Carried from retro #362, reproduced this phase. Promote if it hits a 3rd phase — either the plan-template warns planners to write route descriptors as `.to(<handler>)`, or `/brehon-verify` inspects the routes file for a composed scope before classifying phantom.

---

## Promotion candidates (recurrence ≥ 2 in this session, or ≥ 1 here + ≥ 1 in prior memory)

Boxes UNCHECKED by default; user checks to authorise; the 3 lesson files are ALREADY committed on governance-v0 (`6e6d75b8c`) — these checkboxes track the PMD-index + MEMORY.md propagation, handed to the user in the close-out doc.

- [ ] #292 recover recipe: `feedback_junior_292_stale_base_recover_recipe.md` — **authored + committed**; PMD-sync + MEMORY.md index pending (canonical checkout, in close-out doc) — recurrence 5× this session + 3×+ prior
- [ ] enumerated-transform all-or-blocker: `feedback_impl_task_enumerated_transform_all_or_blocker.md` — **authored + committed**; PMD-sync pending — recurrence 1× here + 1× prior (#362)
- [ ] Windows backslash-path DQ via Write-fragment: `feedback_windows_backslash_path_dq_via_write_fragment.md` — **authored + committed**; PMD-sync pending — recurrence 2× this session
- [ ] bm-merge.md Phase-2.3 required-vs-advisory amendment — proposed in phase retro §3 + DQ #247; not yet applied to `.claude/commands/bm/bm-merge.md`
- [ ] PMD eval write — **SKIPPED**: `PROJECT_MEMORY_DB=unset` in the lane worktree (per skill Step 5: don't fabricate the path). The per-segment evals #358–#364 already capture the granular learnings; the phase retro `228a881f9` is the durable record. PMD-sync of the 3 lessons is in the close-out doc for the canonical checkout.

---

_Generated by `.claude/skills/session-retro/SKILL.md`. Lessons consulted:
`feedback_retro_not_report.md`, `feedback_four_role_retro_signals.md`,
`feedback_retro_task_complexity_score.md`. Auto-phase reliability section
omitted — no `/auto-phase` invocation or auto-state artifacts this session
(manual orchestration via direct tool calls)._
