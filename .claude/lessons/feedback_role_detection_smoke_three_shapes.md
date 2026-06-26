---
name: Role-detection gate changes — smoke against ≥3 transcript shapes before commit
description: Every change to a Junior-vs-advisor role-detection gate (in role-signal-utilisation.sh or any future hook that classifies sessions by role) must run a smoke harness against three real transcript shapes — real Junior worker, finalize agent, advisor session — BEFORE commit. AND the harness must assert every field the hook emits, not just the role string. Five sequential fixes on the same hook shipped: four from incomplete transcript-shape coverage, one (fix #5, 2026-06-26) from the harness asserting role detection but never the array-population jq.
type: feedback
---

When changing any role-detection logic that gates a hook on
`[role:planning|impl-task|bm-task|ci-watcher]` (currently
`.claude/hooks/role-signal-utilisation.sh`; future hooks too), run the
smoke harness at `.claude/hooks/test-role-signal-utilisation.sh`
against ≥3 real transcript shapes BEFORE committing the change:

- **Real Junior worker** — Junior CLI dispatch (framework prefix
  ~552 bytes + `Task:\n[role:X]` on its own line). MUST match the
  expected role.
- **Finalize agent** — Per-task git-only sub-agent
  (`You are a git finalize agent...` + `Task that was completed:
  [role:X]` as a SAME-line prose value). MUST NOT match.
- **Advisor session** — Free-form interactive laptop prose, system
  reminders, slash-command output. MUST NOT match.

**Why:** The role-signal Stop hook has had FOUR sequential root-cause
fixes (`f6088a83d` → `feaa75db9` → `729b13312` → `b80c16dcf` →
`10b29ec4d`) — every prior fix shipped because the test cases didn't
cover the full transcript shape space:

1. **`f6088a83d` (initial):** gated on `git rev-parse --abbrev-ref
   HEAD` matching `^junior/`. Test case: assumption that hook cwd
   would be the worker worktree. Reality: cwd is the daemon's MAIN
   checkout. **Untested shape:** the actual Stop-hook process cwd
   under `claude -p`.
2. **`feaa75db9` (fix #1):** switched to `CLAUDE_PROMPT` env var.
   Test case: assumption that env carries the prompt. Reality: env
   doesn't exist in `claude -p` mode. **Untested shape:** a real
   Junior worker's actual env at Stop time.
3. **`729b13312` (fix #2):** fell back to `head -50 transcript |
   grep '[role:X]'` anywhere. Test case: real Junior worker
   transcript (passed). **Untested shapes:** finalize-agent
   transcripts that quote `[role:X]` in their "Task that was
   completed:" prose (silently false-positive'd → PMD rows 561+562
   with no real utilisation data); advisor transcripts mid-content
   `[role:X]` mentions (extra CPU per Stop event).
4. **`b80c16dcf` (fix #3, T4a):** narrowed to `^\[role:` anchored at
   byte 0 of `head -c 200`. Test cases: advisor session (passed —
   correctly rejected) + finalize agent (passed — correctly
   rejected). **Untested shape:** a real Junior CLI dispatch whose
   framework prefix wraps the tag at byte ~553. EVERY production
   Junior worker since 2026-05-24 12:06Z silently exit-0'd in the
   role gate.
5. **`10b29ec4d` (fix #4, this lesson's trigger):** match
   `^Task:\s*\n\[role:X\]` multi-line. All three shapes asserted via
   the smoke harness BEFORE commit.

Four shipped misses on the same defect class. The cost: 24h+ of
lost signal data + two PMD pollution rows + a 25-min session to
diagnose. The fix is process, not code: every gate change runs
the harness against the full shape space.

**How to apply:**

- The harness at `.claude/hooks/test-role-signal-utilisation.sh`
  takes transcript paths via env vars
  (`TRANSCRIPT_POSITIVE_JUNIOR`, `TRANSCRIPT_NEGATIVE_FINALIZE`,
  `TRANSCRIPT_NEGATIVE_ADVISOR`, plus `_2` siblings for a second
  Junior role and a second finalize variant). On EliteDesk with
  env vars unset, the harness auto-discovers the most-recent
  Junior worker transcript.
- For every role-detection block change, the smoke harness
  output MUST be pasted into the commit body OR the commit body
  MUST cite the SHAs of the transcripts asserted against. The
  harness exits non-zero if any FAIL or if 0 transcripts were
  available; a green commit cannot land otherwise.
- The harness contains its own inline copy of the role-detection
  logic (`detect_role()` function). When the production hook's
  regex changes, mirror the change into the harness in the same
  commit. A diverged harness is a bigger trap than no harness.
- If a new transcript shape emerges (e.g. a future Junior CLI
  rewrite changes the dispatch wrapper), add a new test case
  to the harness in the SAME commit that updates the hook. Don't
  let the next "fix N+1" cycle start.

**Symptom to recognise:**

- The hook's role-detection block is being edited and the commit
  body doesn't cite the harness output → red flag, ask for it.
- A new fix narrows the matcher to fix a false-positive class
  without checking that the existing positive cases still pass.
- A new fix broadens the matcher to fix a false-negative class
  without checking the prior false-positive cases stay rejected.
- Signal corpus stops growing for >24h despite Junior dispatches
  happening — the gate is silently exit-0'ing somewhere.

**Generalises to:** any harness-side hook that classifies sessions
by their dispatch prompt. The three-shape rule (one positive, one
adjacent-but-must-reject, one ambient-but-must-reject) generalises:
test the contract AND each known false-positive class AND the
ambient laptop case. For non-role gates, substitute equivalent
shapes — e.g. for a "finalize-vs-impl gate" you'd want a finalize
agent (positive), an impl-task that mentions finalize in prose
(must-reject), and an advisor session discussing finalize-agent
behaviour (must-reject).

**Fix #5 (`53e03a93a` → `67d22f227`, 2026-06-26) — the field-coverage
extension.** The first four fixes all concerned *role detection* (does the
gate fire for the right session?). Fix #5 was a different defect entirely:
detection worked perfectly, but the hook's OTHER output — the
`rules_read` + `mcp_tools_invoked` arrays — was extracted with a flat
`jq 'select(.type? == "tool_use")'`. In Claude Code JSONL, `tool_use`
blocks are NESTED inside `assistant.message.content[]`; top-level `.type`
is `"assistant"`, never `"tool_use"`. So both arrays were `[]` for all 197
rows across a full month (2026-05-24..06-26), masking real MCP usage on
Opus planning tasks. Proven on job-768: flat path → `[]`; nested path →
`["Bash","Read","ToolSearch","Write","mcp__project-memory__memory_write_eval"]`.

**Why the harness missed it:** the harness asserted ONLY `detect_role`.
It had no assertion for the array-population jq — the very field that broke.
detect_role kept passing; the rows kept shipping empty. The three-shape
rule was satisfied and the bug was still invisible.

**The generalisation (now mechanical):** a smoke harness for a hook that
emits a STRUCTURED record must assert EACH independently-computed field,
not just the headline one (the role string). A field with its own
extraction block is its own regression surface. Fix #5 added
`assert_mcp_nonempty()` (mirrors the production array jq) which FAILs on a
flat-select regression. Going forward, for every distinct jq/grep/parse
block in `role-signal-utilisation.sh`, there must be a harness assertion
that fails when that block returns wrong/empty.

**Fixture selection must match the asserted property.** Fix #5 also hit two
false-reds from naive auto-discovery: `ls -t | head -1` grabbed a role-less
weekly-review task and asserted `role=planning`; and the MCP case grabbed a
Haiku bm-task (which uses zero MCP by design — pure Bash/Edit/Read) and
asserted non-empty. The harness now scans newest-first for a transcript that
*exhibits the property being asserted* (role-prefixed for detection;
≥1 `mcp__` call for the MCP case), not merely the newest file. PMD #350 (bug),
#351 (pattern).

**Related lessons:**

- `feedback_falsifiable_hypothesis_before_structural_fix.md` —
  each prior fix's RCA was a hypothesis. Verifying against the
  full shape space catches hypotheses that hold for some inputs
  but not all.
- `feedback_test_against_reality_not_syntax.md` — bash -n / lint
  / prose review didn't catch this; a multi-line regex change
  parses fine and reads sensible without matching any real input.
- `feedback_lesson_must_pair_with_structural_fix_when_fixable.md`
  — this lesson ships paired with the harness file, not as
  process-only prose.
- Session retro `.claude/PRPs/reports/session-retro-2026-05-25-role-signal-hook-gate-fix.md`.
