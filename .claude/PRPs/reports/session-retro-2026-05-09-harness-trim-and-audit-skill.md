# Session retro — 2026-05-09 — harness-trim-and-audit-skill

**Harness:** claude-code
**Session window:** ~2026-05-09 17:00 UTC → ~2026-05-09 18:30 UTC (~90 min)
**Branch at start:** `c8c8d0dd2` (`throwaway-test-concurrent-probe2`, no-op label on `governance-v0` tip)
**Branch at end:** `a8cc61380` (`governance-v0`)
**Files touched:** 9 (3 trimmed rules, 1 new refs/, 4 new skill files, 1 user-scope MEMORY.md)
**Commits:** 4 (auto: 0, explicit: 4) — `59a184ef6`, `6a4caf9fd`, `18e4a29c4`, `a8cc61380`. Pushed in two batches (3 trim commits + 1 skill commit).

## TL;DR

Two-pass session: first compressed ~3.5–4k tokens off Claude Code auto-load via three direct `chore(rules):` commits (PM-rule path-scope, DQ recipe extract, advisor-orchestrator §5.2 collapse), then converted the methodology into a reusable `harness-audit` project-scope skill so future trim attempts don't re-derive from scratch. The audit subagent on the first pass missed pre-existing `paths:` frontmatter on `governance-log-entry-kind-registry.md`, costing ~4.5k of mis-targeted plan padding before mid-execution reframe — that miss is now codified into the new skill's Phase 1 Explore subagent prompt as a verbatim instruction. **Highest-leverage takeaway:** when delegating read-heavy probes to a subagent, the prompt must explicitly direct first-N-line reads for frontmatter detection; "list large files by size" is the classic shape that silently includes already-scoped files.

---

## What surprised us

- **Audit-subagent miss class.** The Explore subagent reported `governance-log-entry-kind-registry.md` (19.7 KB, 4.9k tok) as the #1 compression target without noticing the file already had `paths:` frontmatter from commits `0ff419cfc` + `dbc0fecad` (Phase B of a prior trim). The token cost was already saved months ago. The miss surfaced only when I read the file directly mid-execution to author Pass 1 edits. Cost: ~10 min of plan-padding + reframe overhead. **Class:** subagent prompt didn't direct frontmatter detection → audit reported on raw file size only.
- **Branch label drift.** Session opened on `throwaway-test-concurrent-probe2` rather than `governance-v0`, both pointing at the same SHA `c8c8d0dd2`. No divergent commits — the throwaway was a stale label from a prior session's probe. Caught via `git rev-parse HEAD governance-v0 origin/governance-v0` returning identical SHAs. Surprise was low (no harm done), but a real divergence at this point would have caused a clean-branch refusal cascade.
- **Concurrent-session activity visible in `git log`.** While my session was working, another concurrent session committed `b8225be3e`, `e75d6a814`, `479408a98` directly on `governance-v0` (8/6/3 minutes ago at audit-time). Push of my second batch (`a8cc61380`) showed those commits already on origin (`479408a98..a8cc61380` push range). The session-awareness rule predicts this scenario; this is an empirical confirmation that the dual-session pattern is in active use today.
- **`harness-audit` skill auto-registered immediately on Write.** After writing `.claude/skills/harness-audit/SKILL.md` with `user-invocable: true`, the very next system reminder listed `harness-audit` in the skill catalogue alongside `code-audit` etc. No reload required. Confirms the project-scope skill discovery is genuinely live-fs — useful to know for future skill authoring.
- **MEMORY.md compaction worked first try.** RESUME paragraph (1,997 chars run-on) → 5 structured fields (~1,000 chars), 204 → 190 lines. No 200-line truncation regression. Counter-evidence to the "compaction always loses signal" prior — when the source is run-on prose, structured-field rewrite is pure win.

## What to change

| # | Change | Expected effect | Cost | Recurrence |
|---|---|---|---|---|
| 1 | Promote "subagent prompts for read-heavy probes MUST direct first-N-line reads for frontmatter detection" to a new lesson `feedback_subagent_frontmatter_detection.md`; update `feedback_subagent_delegation_for_multi_probe_commands.md` to cross-link | Future audit subagents skip already-scoped files; saves the ~10-min reframe per audit | minor (one new lesson + cross-link edit) | 1× this session, 0× prior — but the harness-audit skill itself codifies the fix so the lesson is the durable record |
| 2 | Run `/harness-audit` against the post-trim state in a fresh session and confirm Phase 1 correctly classifies `governance-log-entry-kind-registry.md` as SCOPED (regression test the new skill against the lesson it encodes) | Validates the skill body works as designed before relying on it for the next trim | minor (one fresh-session run, ~5 min) | 0× prior; first dogfood run |
| 3 | Add a session-awareness warning to the harness-audit skill body: "if `git log governance-v0..HEAD` shows commits in the last 60 min that aren't yours, surface concurrent-session activity in the report's environment snapshot" | Prevents the audit from silently reporting on stale state when another session is concurrently mutating | minor (Phase 0 prose addition) | 1× this session, ≥3× in prior memory (`feedback_parallel_agents_one_worktree_per_agent`, `feedback_parallel_agent_diff_collision_detection`) |
| 4 | Delete the stale `throwaway-test-concurrent-probe2` branch label (`git branch -D throwaway-test-concurrent-probe2`) so future sessions don't open onto it accidentally | Removes one source of branch-label confusion | trivial | 1× this session |
| 5 | Validate the new harness-audit skill's `skillListingBudgetFraction: 0.005` envelope: confirm the multi-line frontmatter description is ≤6 lines (ours is 7 — slightly over) | Avoids the skill listing crowding out task budget on every turn | trivial (frontmatter trim) | 1× this session, frontmatter authored by me |

## What to carry forward

- **Plan-then-revise pattern when an audit subagent miss surfaces.** Mid-execution, I caught the `paths:` frontmatter miss and updated the plan file in-place via `Edit` rather than re-planning from scratch. The plan stayed the audit trail of what was attempted; the rewritten Pass 1 became the actual work. Pattern: **the plan file is a living document during execution, not a contract**.
- **Direct-commit on `governance-v0` for pure meta-work.** All four commits were `chore(rules):` / `feat(skills):` — no `crates/`, no PR. Per `phase-branch.md` direct-commit policy. Saves the PR overhead when CodeRabbit review value is net-noise (advisor prose, not Rust).
- **Helper-file split for skill bodies.** The `harness-audit` skill split into `SKILL.md` (121 lines, the entry point) + 3 helpers under `helpers/` (87+51+94 lines, read just-in-time). Mirrors the `test-write` skill pattern. Keeps SKILL.md scannable at frontmatter-load time; defers the prose-heavy reference material to Phase 3 / Phase 5 read events.
- **Empirical verification before trusting subagent inventory.** When a subagent reports "X is the biggest target," the parent should spot-read 2-3 of the top entries before acting. Caught the frontmatter miss; would also catch other classes of subagent error (wrong glob scope, file size measured pre-trim, etc.).
- **Compactable RESUME paragraph pattern for MEMORY.md.** When a project's RESUME header drifts to >1,500-char run-on prose, restructuring into 5 single-line fields (STATUS / IN-FLIGHT / DQ / NEXT / CARRY) is reliable signal-preserving compaction. Reuse for future MEMORY.md health passes.
- **The `.claude/refs/` convention.** First populated this session via `dq-recipes.md`. Naming convention establishes: `.claude/lessons/` (PMD-indexed, search-on-demand), `.claude/refs/` (path-cited, Read-on-demand), `.claude/rules/` (auto-load). Three-tier disclosure model is now visible to future authoring sessions.

---

## Three-signal scoring

Per `.claude/lessons/feedback_four_role_retro_signals.md`. Numbers defensible from this session's transcript.

| Skill / Agent / Command | Saved (min) | Wasted (min) | Surprise | Notes |
|---|---:|---:|---|---|
| Initial audit Explore subagent | 15 | 10 | medium | Saved 15 min vs me reading every rules file inline; wasted 10 min via missed `paths:` frontmatter — net +5 min, but the miss-class informs change #1 |
| Pi-Coding boundary Explore subagent | 12 | 0 | low | Self-contained prompt, returned single ~1.5 KB synthesis; confirmed `.pi/` invariants in one shot |
| MEMORY.md structure Explore subagent | 8 | 0 | none | Catalogued sections + char-counted entries; output drove the compaction with no rework |
| `AskUserQuestion` (3 questions) | 5 | 0 | none | Pi scope / harness scope / location — clean architectural fork before commit |
| `AskUserQuestion` (commit decision) | 2 | 0 | none | Standard "commit + push?" gate after harness-audit skill written |
| Mid-execution plan reframe | — | 10 | high | Reframe cost ~10 min after audit miss surfaced; recovered the plan file in-place |
| `Bash` git operations (status, diff, log, push) | — | 0 | none | Standard git probes; all clean |
| `Edit` on advisor-orchestrator.md §5.2 | 8 | 0 | none | Single targeted Edit; content-preserving compaction |
| `Edit` on decision-queue.md (recipe extract) | 12 | 0 | none | One large Edit replacing 117 lines with a 4-line pointer |
| `Write` on `.claude/refs/dq-recipes.md` (new) | — | 0 | none | New file; bonus archive-span fix added during write |
| `Write` MEMORY.md (compaction) | 8 | 0 | none | Full-file rewrite; 204→190 lines |
| `Write` `harness-audit` SKILL.md + 3 helpers | 25 | 0 | none | Mirrored `code-audit` precedent; 4 files in one batch |

**Aggregate:** ~95 min saved across delegations and writes; ~20 min wasted (audit miss + reframe). Net positive ~75 min. Surprise level: one medium (audit miss), one high (reframe needed), rest none/low.

## Complexity scores (heavy tasks only)

Per `.claude/lessons/feedback_retro_task_complexity_score.md`. Format: `<files>/<commits>/<runtime-min>/<max-log-silence-min>`. Flag if >55min runtime, >40min log-silence, or >8 files touched.

| Task | Files | Commits | Runtime (min) | Max log silence (min) |
|---|---:|---:|---:|---:|
| Three-pass trim (PM rule scope + recipe extract + §5.2 collapse) | 4 | 3 | ~35 | <2 |
| MEMORY.md compaction | 1 | 0 (user-scope) | ~8 | <1 |
| harness-audit skill authoring | 4 | 1 | ~25 | <2 |

No watchdog-envelope flags. Largest single task (skill authoring) at 4 files / 1 commit / 25 min — comfortably within the Sonnet impl-task envelope, though this session ran on Opus 4.7 (1M).

## Decisions to revisit

- **Pi-Coding sibling skill.** Out-of-scope this session (locked at clarification stage). When `/harness-audit` proves valuable on Claude Code, consider authoring `.pi/skills/harness-audit/` mirror. Trigger: 2+ Claude Code audit runs that cite Pi-shared paths in the "Out of scope" bucket — that's the signal Pi has its own load surface worth auditing.
- **Subagent-prompt regression library.** The "subagent missed frontmatter" miss-class is one of N classes that subagent prompts can fail. Worth a separate session to compile a prompt-shape checklist (frontmatter-aware reads / glob scope verify / pre-trim baseline check / etc.) — would feed into a `subagent-prompt-shape-audit` lesson.

---

## Promotion candidates (recurrence ≥ 2 in this session, or ≥ 1 here + ≥ 1 in prior memory)

- [ ] Change #1 (subagent frontmatter detection): promote to `.claude/lessons/feedback_subagent_frontmatter_detection.md` (cross-harness lesson) — recurrence 1× here + 0 prior, but the harness-audit skill body codifies it so promotion lifts it from skill-internal to general lesson
- [ ] Change #2 (regression test): execute as a follow-up `/harness-audit` run; not a lesson promotion, just a verification task
- [ ] Change #3 (concurrent-session warning in skill): edit `harness-audit/SKILL.md` Phase 0 — minor skill body update; recurrence ≥3× in prior memory makes this a promote-quickly candidate
- [ ] Change #4 (delete throwaway branch): one-liner `git branch -D throwaway-test-concurrent-probe2`; not a lesson, just hygiene
- [ ] Change #5 (skill description ≤6 lines): one Edit on `harness-audit/SKILL.md` frontmatter — trim from 7 lines to ≤6; trivial fix
- [ ] PMD eval write — `PROJECT_MEMORY_DB=unset` per Step 5 check; **skipped** (no fabricated path)

---

_Generated by `.claude/skills/session-retro/SKILL.md`. Lessons consulted: `feedback_retro_not_report.md`, `feedback_four_role_retro_signals.md`, `feedback_retro_task_complexity_score.md`. Auto-phase reliability section omitted — no `/auto-phase` invocation this session and no `.claude/auto-state/` mutation (leftover `v1-SL-c-2.json` is from a concurrent session, does not trigger Step 0.5)._
