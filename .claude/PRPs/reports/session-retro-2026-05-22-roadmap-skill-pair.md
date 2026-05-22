# Session retro — 2026-05-22 — roadmap-skill-pair

**Harness:** claude-code (Opus 4.7, 1M context)
**Session window:** 2026-05-22 ~14:00 → ~16:15 UTC (~2h 15m wall-clock; pre-compaction session began earlier, ~5h+ end-to-end including verification work on v1 PRDs that fed the roadmap)
**Branch at start:** `e9caf8933` (`governance-v0`)
**Branch at end:** `efa3f77c4` (`governance-v0`)
**Files touched:** 4 (`.claude/PRPs/v1-roadmap.json`, `.claude/PRPs/specs/auto-roadmap-skill-pair.md`, `.claude/rules/auto-roadmap.md`, user-scope `~/.claude/commands/{roadmap-next,auto-roadmap}.md`)
**Commits:** 2 (`98cccc734` spec+roadmap, `efa3f77c4` rule+schema-bump; both explicit advisor)

## TL;DR

Shipped a two-skill chain (`/roadmap-next` + `/auto-roadmap`) that composes on top of `/auto-phase` to drive multiple sub-phases of a v1 PRD lane without re-deriving "which sub-phase next?" at every boundary. The pre-compaction phase verified v1 PRD lanes (JM/SL/AD done; RT/SR partial) and produced `.claude/PRPs/v1-roadmap.json` as the durable source of truth; the post-compaction phase composed the skill bodies + rule + schema bump from the spec already authored. Highest-leverage finding: **when a sub-agent claims a fact about git history (e.g. "JM-d unstarted"), the parent must verify via direct `git log` before propagating — sub-agents return their intent, not necessarily what they observed.** Single-occurrence this session but a known recurrence class — already covered by `pattern_verify_before_trusting_shell_output.md`.

---

## What surprised us

- **Two sub-agent factual conflicts in the pre-compaction phase, both resolved by direct git verification.** A general-purpose sub-agent claimed JM-d was unstarted (it wasn't — six direct-to-trunk commits + retro shipped). A second sub-agent claimed SL-c-2 was in-flight (it wasn't — retro 5b80186e7 + lane meta-retro f4140f3f1 confirmed done). Both surfaced as the parent advisor double-checked specific git refs before committing the roadmap. Cost: ~5 min each. Lesson is already in the corpus (`pattern_verify_before_trusting_shell_output.md`); the surprise was the re-occurrence, not the class.

- **Skill harness picked up `/auto-roadmap` immediately on Write, but `/roadmap-next` was indexed without its description for the first ~5 min.** Both files use first-line-as-description (no YAML frontmatter, matching `/auto-phase` precedent). The harness available-skills system-reminder showed `auto-roadmap` with description on first refresh; `roadmap-next` showed "(no description)" until the next system-reminder cycle. Cosmetic — no functional impact — but a small surprise about the indexing latency. Self-resolved by the next reminder cycle.

- **The roadmap mapped 23 GitHub Dependabot CVEs to a noisy push warning** that has nothing to do with this work (upstream Lemmy crate vulnerabilities the fork inherits). Already documented as part of the project-tier service constraints in PMD. Not surprising on its own, but worth flagging that the noise floor for push warnings is now permanent — future retros should filter these out of "what got logged at push time".

- **AskUserQuestion answered cleanly on all three pre-implementation decisions** (output scope / plan-gap source / `/auto-phase` invocation mechanism) without any "Other" branch needed. Three options the user picked the recommended one on each. Surprising effectiveness for a four-option question with technical detail — the option labels carried enough context that "Recommended" was a clear choice, not a guess.

- **The `/auto-phase` skill body explicitly contains a precedent for two of skill 2's hardest design questions** (option-1 vs option-2 invocation mechanism + pre-seed JSON shape). Reading `/auto-phase` Phase 0.5 + Phase 0.7 first saved authoring time — the skill 2 body just composes those existing primitives. Surprising effectiveness of the "read canonical sibling first" discipline; this session was almost entirely composition rather than authoring novel mechanics.

## What to change

| # | Change | Expected effect | Cost | Recurrence |
|---|---|---|---|---|
| 1 | When skill-creator skill is invoked for skills that **dispatch real Junior tasks or mutate state files**, surface this in the first AskUserQuestion as an "output scope" option — Don't run eval-subagent harness blind. Currently this is buried in the SKILL.md guidance ("eval-subagent runs would queue real Junior tasks; not safe"). Promote to explicit AskUserQuestion phrasing the first time the skill-creator detects an `mcp__junior-brehon__` reference in the proposed skill body. | Prevents accidentally launching a real Junior dispatch from an eval subagent in future skill-pair authoring. | minor | 1× this session (skill-creator surfaced it correctly because the spec explicitly said "not safe to eval"); could recur for any future skill that wraps Junior dispatch |
| 2 | Add a `_validation_probe` row to the `.claude/PRPs/v1-roadmap.json` schema documenting that `python -c "json.load(io.open(...))"` from canonical CWD is the canonical "is this file healthy" check. Cheap addition; gives future skill 1 invocations a one-line health probe instead of re-deriving the check. Or — alternative — extract Phase 0 step 4's python probe from skill 1 into a small `scripts/brehon/roadmap-validate.sh` so the same probe is reusable. | One source of truth for "is roadmap healthy"; one place to fix the probe if schema bumps in the future. | minor | 1× this session, but the probe will fire on every `/roadmap-next` invocation forever |
| 3 | When authoring a new spec/rule pair (spec at `.claude/PRPs/specs/<X>.md`, companion rule at `.claude/rules/<X>.md`), commit the spec FIRST as a standalone commit, then the rule + companion artifacts. This session bundled spec + roadmap into commit 1 and rule + roadmap-schema-bump into commit 2 — workable but the spec deserves its own atomic landing for retro/review. Codify in a new short lesson: `feedback_spec_then_rule_separate_commits.md`. | Cleaner audit trail; reviewer can read the spec in isolation before seeing how the rule encodes it. | minor | 1× this session, but every future advisor-spec-then-rule pair would benefit |
| 4 | Skill 1 (`/roadmap-next`) currently dispatches `bm-cut` as a Junior task, then walks the bootstrap checklist directly in the advisor session. This **mixes orchestration with mechanical setup** in one skill. Consider splitting: `/roadmap-cut` (Junior bm-cut + push) and `/roadmap-bootstrap` (worktree-add + checklist walk + roadmap flip). Or — alternative — accept the current shape but add a checkpoint mid-skill so the user can verify bm-cut succeeded before bootstrap begins. The current shape's risk: if bm-cut succeeds but bootstrap fails, the lane branch exists on origin but no worktree is set up, and the roadmap is still `unstarted`. Manual recovery is documented in `.claude/rules/auto-roadmap.md` §Resume semantics but is friction. | Either split or checkpoint reduces recovery cost on a partial-failure path that's probability-low but cost-high. | medium | 1× this session (resume semantics documented = the concern was acknowledged but not fully resolved) |
| 5 | `/auto-roadmap` skill body says "if `/auto-phase` skill version drifts between dogfood and ship, skill 2 may break" but **doesn't actually pin** the `/auto-phase` version. The spec §6 decision point 5 raised this; the implementation deferred it. Cheap fix: at skill 2 Phase 0 step 5, add a `git log --grep="auto-phase"` check to detect mid-flight `/auto-phase` skill edits and surface to user with the trunk SHA the skill body was loaded at. Cost: ~10 lines of bash + python in the skill body. | Surfaces `/auto-phase` skill-body drift during a long-running sub-phase, where the spec's stated risk applies. | minor | 0× this session, but the risk surface exists for every `/auto-roadmap` invocation |

## What to carry forward

- **Pre-implementation AskUserQuestion to lock decisions before spec-into-code translation.** The four-option AskUserQuestion at the start of this session ("what should the skill-creator produce?", "plan-gap source?", "invocation mechanism?") landed perfect first-try because each option carried enough technical context that the user could decide quickly. Pattern: when a spec has flagged open questions (this spec's §6 had five), surface them via AskUserQuestion in the first message after reading the spec — never wait for the user to bring them up.

- **Compose-not-author when an upstream skill already encodes the primitives.** `/auto-phase` Phase 0.5 (resume reconciliation), Phase 0.7 (lazy-load discipline), and the auto-state JSON template are all primitives. Skill 2 imports them by reference rather than re-implementing. The whole skill 2 body is ~28 KB and most of it is wrapper logic + dogfood — the heavy state-machine logic stays inside `/auto-phase`. Pattern is the right shape for any future skill that extends a multi-phase orchestrator.

- **Atomic-protocol commit sequence per `.claude/rules/multi-lane-worktree.md` Hard refusal #6.** Used twice this session: roadmap mutation in commit 1 (spec+roadmap) and rule+schema-bump in commit 2. Both went through `git fetch → read → mutate → add → status verify → commit → push` as single shell sequences. Zero race incidents. The discipline is becoming muscle memory; keep it.

- **First-line-as-description for user-scope slash commands matches `/auto-phase` precedent.** Both new skills `/roadmap-next` and `/auto-roadmap` use first-line-as-description (no YAML frontmatter). The harness picks them up correctly. Pattern to apply for any future user-scope `~/.claude/commands/*.md`.

- **TaskCreate at the start of multi-step composition work, not retroactively.** Six tasks created upfront (read prep / write rule / write skill 1 / write skill 2 / schema bump+dogfood / commit). Each completed in order with `TaskUpdate` status flips. The harness's gentle reminders fired correctly when I was mid-step too long; would have been more useful to dismiss them as I went (they triggered three times in this session, each correctly noting status was up-to-date).

---

## Three-signal scoring

| Skill / Agent / Command | Saved (min) | Wasted (min) | Surprise | Notes |
|---|---:|---:|---|---|
| AskUserQuestion (pre-implementation gate, 3-Q) | 15 | 0 | none | Clean lock-in on output-scope + plan-gap-source + invocation-mechanism before code; spec §6 open questions resolved in one message |
| Reading `/auto-phase` skill body before authoring skill 2 | 30 | 0 | medium | Found that Phase 0.5 + Phase 0.7 + auto-state JSON template already encode all the primitives skill 2 needs; saved re-deriving them |
| `bm-cut.md` reading for skill 1's BM-dispatch authoring | 10 | 0 | none | Three precondition gates + regex branch-name check pattern reused verbatim |
| `feedback_phase_lane_worktree_bootstrap_checklist.md` reading | 8 | 0 | none | 11-step checklist becomes skill 1 Phase 4.2 table verbatim; no re-derivation |
| `feedback_retro_not_report.md` + companion lessons (Step 0 of this retro) | 5 | 0 | none | Standard retro prep; cheap |
| sub-agent verification of v1 PRD lane status (pre-compaction) | — | 10 | medium | Two factual conflicts on JM-d + SL-c-2; both resolved by parent verification; counted on pre-compaction side (already covered) |
| TaskCreate + TaskUpdate flow | 5 | 0 | none | Six tasks completed in order; harness reminders fired correctly thrice but were no-ops |
| Direct git log verification vs. believing sub-agent output | 10 | 0 | low | Already in PMD as a pattern; this session reused it correctly |
| Atomic-protocol commit sequence (×2) | 5 | 0 | none | Zero races; becoming muscle memory |
| Writing skill 1 dogfood trace inline (matches `/auto-phase` Phase 6) | 5 | 0 | none | ~15-line walkthrough; trivial to author once the skill body is done |
| Writing skill 2 dogfood trace inline | 8 | 0 | none | Hypothetical RT-r2 walk; took longer than skill 1 because plan-gap handler has more branches |

**Net: ~95 min saved on a session that took ~135 min wall-clock (post-compaction).** Saved time concentrated on (a) reading-before-authoring discipline, (b) the AskUserQuestion lock-in, (c) composing rather than re-implementing. Wasted time was minimal — the only real cost was the 10-min on pre-compaction sub-agent verification, which is already a known pattern in the corpus.

## Complexity scores (heavy tasks only)

| Task | Files | Commits | Runtime (min) | Max log silence (min) |
|---|---:|---:|---:|---:|
| Write `.claude/rules/auto-roadmap.md` | 1 | 1 | ~25 | ~3 (per Edit response cycle) |
| Write `~/.claude/commands/roadmap-next.md` | 1 | 0 (user-scope, untracked) | ~25 | ~3 |
| Write `~/.claude/commands/auto-roadmap.md` | 1 | 0 (user-scope, untracked) | ~30 | ~3 |
| Roadmap schema v1 → v2 bump | 1 | 1 | ~5 | <1 |
| Commit + push 2 commits | 2 (staged) | 2 | ~3 | <1 |

No task exceeded the 55-min runtime threshold or the 40-min log-silence threshold. The advisor-side Write/Edit cadence kept all tasks well-paced; no Junior tasks were dispatched in this session (`/roadmap-next` + `/auto-roadmap` will dispatch Juniors when invoked for real, but this was authoring only).

## Decisions to revisit

- **Skill 1's `bm-cut` plan-missing override.** Skill 1's body documents that `bm-cut` will STOP on missing-plan (per `bm-cut.md` Phase 2), and skill 1 catches the BM DQ "cut anyway" option. This override path is documented but not dogfooded — the real-world test fires when `/roadmap-next` is first used on `v1-RT-r2`. Worth a clarify pass before that dogfood to ensure the BM subagent accepts the override flag cleanly.
- **Skill 2's plan-gap handler `/brehon-clarify` invocation.** Skill 2 Phase 0.5.4 invokes `/brehon-clarify <brief-path>` via Skill tool. The `/brehon-clarify` skill exists but its argument expectation should be verified against current shape before first dogfood.
- **Roadmap schema bump propagation.** Schema is now at v2. The hard refusal #4 in skill 1 expects `$schema_version in {1, 2}`. If a future schema v3 lands, the skill needs updating. Mention in the next phase retro if the bump cadence becomes a friction point.

---

## Promotion candidates (recurrence ≥ 2 in this session, or ≥ 1 here + ≥ 1 in prior memory)

For each item from "What to change" that meets the threshold, the user may approve promotion. Boxes UNCHECKED by default; user checks to authorise.

- [ ] Change #1 (skill-creator surfaces Junior-dispatch class in AskUserQuestion): Promote to `.claude/lessons/feedback_skill_creator_junior_dispatch_class.md` — currently 1× this session but is a known structural concern; could recur for any future skill that wraps a state-mutating dispatch.
- [ ] Change #2 (roadmap validation probe extraction): Modify existing `.claude/PRPs/v1-roadmap.json` to add `_validation_probe` shape doc, OR add `scripts/brehon/roadmap-validate.sh`. Either is a minor in-repo edit.
- [ ] Change #3 (spec-then-rule separate commits): Promote to `.claude/lessons/feedback_spec_then_rule_separate_commits.md`. Currently 1× this session; would benefit every future advisor-authored spec/rule pair (next likely candidate: ship-readiness sub-PRDs).
- [ ] Change #4 (skill 1 split or checkpoint): Update existing `~/.claude/commands/roadmap-next.md` — add a mid-skill checkpoint between bm-cut completion and bootstrap walk; surface to user via AskUserQuestion if checkpoint fails. ~20 lines of skill body.
- [ ] Change #5 (`/auto-phase` skill version pin in `/auto-roadmap`): Update existing `~/.claude/commands/auto-roadmap.md` Phase 0 step 5 — add `git log --grep="auto-phase"` check at skill 2 entry; surface trunk SHA the skill was loaded at.

---

_Generated by `.claude/skills/session-retro/SKILL.md`. Lessons consulted: `feedback_retro_not_report.md`, `feedback_four_role_retro_signals.md`, `feedback_retro_task_complexity_score.md`. The 10-category `feedback_auto_phase_retro_signals.md` section omitted per Step 0.5 trigger conditions — this session never invoked `/auto-phase` and never mutated any `.claude/auto-state/*.json` file (the two files on disk predate the session)._
