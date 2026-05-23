# Session retro — 2026-05-23 — adr-016-cross-app-backplane

**Harness:** claude-code
**Session window:** ~2026-05-23T17:30Z → 2026-05-23T18:50Z (~80 min wall-clock)
**Branch at start:** `5285321a3` (`governance-v0`)
**Branch at end:** `28f0dc0ff` (`governance-v0`)
**Files touched:** 3 (1 explicit ADR edit + 2 incidentally pulled by rebase: `api_tests/package.json`, `api_tests/pnpm-lock.yaml`)
**Commits:** 1 explicit (`28f0dc0ff` docs(adr): add ADR-016), pushed; 1 upstream pulled via rebase (Dependabot #135).

## TL;DR

User asked for a V2 messaging PRD drift check; mid-session the user reframed Brehon from "Lemmy-fork-with-governance" to "governance backplane spanning Matrix/PeerTube/Lemmy-fork/etc". The reframing changed the load-bearing question from "is the PRD still accurate" to "what cross-app contract codifies this vision". ADR-016 was authored (federation-shaped: B-fetch / B-publish / B-actor), ADR-004 amended, V2a/V2b/V2c renamed to M1/M2/M3, four new OQs opened (`OQ-ADR016-NN` prefix to avoid collision with the resolved pre-existing OQ-016), committed and pushed in one clean commit. The most-load-bearing finding: **mid-session scope reframings are high-value but require deliberate AskUserQuestion re-anchoring — without it, the work product silently misses the new framing.** Top change proposal: codify the "user reframed scope" detection pattern (mid-session surface that the work product's scope has shifted, ask before continuing).

---

## What surprised us

- **The drift check found near-zero drift but the user re-opened the much bigger question anyway.** The three parallel Explore agents returned with "PRD well-anchored" verdicts. The expected next move was a small follow-up list (ADR-016, OQ-V2-08, OQ-V2-10). Instead the user reframed the entire purpose of the project mid-session: Brehon is a governance backplane, not a Lemmy-fork. The drift check was structurally sound but addressing the wrong question. This is a recurring class — `feedback_falsifiable_hypothesis_before_structural_fix` warns about it from the other side; this session's variant is "premise reframing during a check task".

- **AskUserQuestion was load-bearing — twice — but the second batch was rejected and re-asked.** The first AskUserQuestion (which open decision to address) was clean. The second (4-question batch on ADR-016 shape) was rejected with "clarify these"; reformulated to "what would you like to clarify" succeeded. The default AskUserQuestion batch-of-four assumes the user is ready to commit to all four; when scope is still shifting (as it was here after the reframing), the user needs to push back. **Surprise: the rejection-then-clarify cycle is the system working correctly, not a friction.**

- **Push race with Dependabot bot.** First `git push origin governance-v0` failed non-fast-forward — Dependabot had landed `99d304c4f` (#135 npm bump) on the trunk while I was authoring the ADR. Investigated the upstream commit (touched only `api_tests/`, zero overlap), rebased cleanly, pushed. **Surprise: low-medium.** This is the same class as `feedback_cross_session_commit_attribution_collision` but with a bot session instead of a human session — the trunk gets edits at unpredictable times; any commit-on-trunk session should expect a rebase round-trip.

- **No multi-lane warning fired despite two other active lanes.** `git worktree list` showed rt-r2 and ship-3 active. The session-start ritual per multi-lane-worktree.md §1 wasn't surfaced. Two-fold reason: (a) the SessionStart multi-lane-check hook fires on lane lanes (per the rule's threshold heuristic — recent-commit-window-detection), but the canonical checkout isn't a phase-v1-* branch so it wasn't classified as needing the WARN; (b) I did surface the "lanes:" line BEFORE the push, late in the session, when I noticed it pre-commit. **Surprise: low.** The hook is calibrated for lane-to-lane interference; canonical-checkout meta-edits don't interfere with lane Cargo work. But the surface-first ritual should fire earlier in any session that touches trunk — not after 70 min of work.

- **The "task tools haven't been used recently" reminder fired 4× during a session that genuinely didn't need TaskCreate.** This was a single-thread sequential session (drift check → ADR draft → push). Splitting into tasks would have added overhead with no parallelism gain. The reminder is calibrated for sessions with parallel/long-running work — short single-thread editing-and-asking sessions trip it falsely.

## What to change

| # | Change | Expected effect | Cost | Recurrence |
|---|---|---|---|---|
| 1 | Add a "scope-reframe check" to the advisor's drift-check / verify family: when a user asks "is X still accurate?" and the answer is "yes but [reframed scope question]", surface the reframe to the user before proceeding to fixes. Document in `.claude/lessons/feedback_scope_reframe_before_fix.md` as a sibling of `feedback_falsifiable_hypothesis_before_structural_fix.md`. | Catches the "drift-check found nothing, but the premise of the drift-check is now wrong" class. Without this, the session would have produced a small ADR-016 patch addressing PRD drift instead of the actual cross-app-backplane ADR. | minor (one lesson file + cite in advisor-orchestrator.md §5.4 DQ triage tree) | 1× this session; 0× prior; promote-if-recurs threshold not yet met but the value of catching it once is large enough to justify a watch entry |
| 2 | Surface the "lanes:" status line proactively whenever the canonical checkout (`brehon-fork`) is about to do a write-side action (commit on `governance-v0`, ADR/rule edit, PMD write) AND any other worktree is active. Currently the `session-start-multi-lane-check.sh` hook is calibrated for lane-to-lane interference; meta-edits on trunk should also surface lanes once before the first write. Proposal: extend `.claude/hooks/session-start-multi-lane-check.sh` to ALSO fire on canonical-checkout sessions when `git worktree list` shows ≥2 entries, with WARN text "canonical-checkout meta-edit session; other lanes active: <list>". | Catches the case where the user briefly forgets which lane is which. Today I surfaced the lane status pre-commit (correctly) but only because the surface-first ritual prompted it manually — the hook didn't help. | medium (hook extension; needs test on both canonical and lane CWDs) | 1× this session; 1× prior (2026-05-22 boundary incident per advisor-orchestrator.md §1 "Surface-first ritual"); RECURRENCE THRESHOLD MET |
| 3 | When committing on `governance-v0` from the canonical checkout, **always `git fetch origin` immediately before `git push`** (not after the push fails). Reason: Dependabot lands on trunk at unpredictable times; the rebase after a push-failure costs an extra round-trip. The fetch-first pattern is one extra second; the failed-push-then-rebase is ~30s + recovery overhead + cognitive disruption. | Eliminates the "push race with Dependabot" class. The recipe is mechanical: any commit on trunk in a long-running session should bracket the push with `git fetch origin && git push` or use `git push --force-with-lease` after a deliberate rebase. | minor (lesson + cite in phase-branch.md or the existing branch-manager.md "When pushing trunk") | 1× this session; the multi-lane-worktree.md "atomic protocol" already addresses cross-CC-session races but doesn't cite the bot-author race; ADD a sub-bullet to that section's "concurrent commits" recipe |

## What to carry forward

- **AskUserQuestion → AskUserQuestion → write pattern for ADR drafting.** Two rounds of clarifying questions before writing ADR-016 caught two real defects: (a) the four-question batch was too ambitious for the still-shifting scope; (b) the second-pass per-question explanation surfaced the "B-fetch vs B-publish vs B-actor" structure cleanly. The user's preferred sequence was: ask broadly → user reframes if needed → ask specifically → draft. Never draft from the first ambitious batch alone.

- **Parallel Explore agents for drift checks.** The 8-section drift checklist was completed by three parallel Explore agents in one tool round; each agent had clear scope (enums+hooks, ADRs+sec/ops, OQs+naming+refs). Total wall time ~3 min for what would have been ~30 min sequentially. Worked because the sections were genuinely independent and each agent had clear deliverable shape. **Carry forward: any future "drift check on PRD/spec/doc against current code" task is a strong candidate for parallel Explore dispatch.**

- **OQ-prefix pattern when promoting a sub-question family from a new ADR.** When ADR-016 opened 4 child OQs and the global `OQ-016` number was already taken (resolved 2026-04-17), I used `OQ-ADR016-NN` to disambiguate. This pattern matches the existing `OQ-V1-AD-NN` / `OQ-V1-JM-NN` family convention — a sub-question family named after its origin (sub-phase or ADR) avoids global integer collisions. Carry forward as a convention any new ADR with ≥2 child OQs should use.

- **Don't auto-skip retros after "wins."** This session was a clean win (drift check + ADR-016 + push, no incidents). The reflex would be to skip retro discipline. But the three changes above all came from this session — most retros from clean sessions still produce 2-3 real proposals because friction patterns hide in the smooth-feeling parts (the lanes line happening late, the push race being absorbed without flagging).

---

## Three-signal scoring

| Skill / Agent / Command | Saved (min) | Wasted (min) | Surprise | Notes |
|---|---:|---:|---|---|
| 3× parallel Explore agents (drift check) | ~25 | 0 | low | clean parallel dispatch; one agent's row-count was off-by-one but caught by my read-back |
| AskUserQuestion (4-question ADR-shape batch) | 0 | ~5 | medium | user rejected the batch ("clarify these"); 2-min recovery via "what would you like to clarify" + re-explain |
| AskUserQuestion (re-asked clarifications, narrower) | 5 | 0 | none | clean — three load-bearing decisions surfaced (binary isolation, log writer, phase rename) |
| AskUserQuestion (3-question cross-app shape) | 8 | 0 | none | clean — user-answered all 4 (apps independent, federation-shaped, etc); reframed the ADR scope cleanly |
| AskUserQuestion (3-question evidence/sanction/identity) | 0 | ~3 | medium | user rejected and asked "explain each one"; needed verbose option-by-option breakdown before answers |
| Verbose explanation of evidence/sanction/identity options | 12 | 0 | none | user's preferred mode confirmed: option-by-option trade-off prose, then narrower question → answer → next |
| ADR-016 drafting (single Edit + single insert + single OQ-block + 1 changelog) | — | 0 | none | one-shot draft; one self-caught fix (OQ-016 → OQ-ADR016 prefix in Consequences body) |
| git push race recovery (fetch → rebase → push) | — | ~2 | low | mechanical recovery; safe because upstream touched different files |
| Memory + MEMORY.md update at close | 3 | 0 | none | clean; future session has the cross-app reframing indexed |

## Complexity scores (heavy tasks only)

The session had one "heavy" task — the ADR-016 authorship + push.

| Task | Files | Commits | Runtime (min) | Max log silence (min) |
|---|---:|---:|---:|---:|
| ADR-016 draft + amend ADR-004 + 4 OQs + changelog + commit + push | 1 (explicit) + 2 (rebase-pulled) | 1 explicit | ~40 (from "yes, draft ADR-016" to push complete) | n/a (interactive session, no log-silence semantics) |

No watchdog risk — interactive sessions don't have the Junior log-silence envelope. Complexity is well within manual editing range (1 file, ~80 lines of additive structured ADR prose).

## Decisions to revisit

- **OQ-ADR016-NN convention.** Worth a one-line note in `99-decisions-and-open-questions.md`'s introduction that the `OQ-<scope>-NN` namespacing is intentional (V1-AD / V1-JM / ADR016). Not urgent; document organically next time a new sub-question family opens.
- **The PRD V2a→M1 rename.** Deferred to M1 schedule time per ADR-016's "Enacted in". Worth flagging in `MEMORY.md` so the future M1-scheduling session doesn't forget. Already done — `project_brehon_cross_app_governance_backplane.md` notes it explicitly.
- **08-cross-app-governance.md companion doc.** Listed in ADR-016 Enacted-in as deferred to M1 schedule time. Will need a small clarify pass when M1 is on the table — what protocol-level detail belongs there vs in M1 sub-PRD vs in per-app integration ADRs.

---

## Promotion candidates (recurrence ≥ 2 in this session, or ≥ 1 here + ≥ 1 in prior memory)

- [ ] Change #2 (canonical-checkout multi-lane surface): promote to `.claude/lessons/feedback_canonical_checkout_lane_surface.md` (cross-harness lesson — applies to any meta-edit session, not just ADR work) + extend `.claude/hooks/session-start-multi-lane-check.sh`
- [ ] Change #3 (fetch-before-push on trunk): add sub-bullet to `.claude/rules/multi-lane-worktree.md` §"Hard refusals" #6 (atomic read-mutate-commit) — currently scoped to lane-to-lane races; extend to cover bot-author races on trunk
- [ ] Change #1 (scope-reframe check): single-occurrence; record as a watch entry in `MEMORY.md` under "Watch / promote-if-recurs"; promote to full lesson if it recurs

---

_Generated by `.claude/skills/session-retro/SKILL.md`. Lessons consulted: `feedback_retro_not_report.md`, `feedback_four_role_retro_signals.md`, `feedback_retro_task_complexity_score.md`._
