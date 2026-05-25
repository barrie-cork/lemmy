---
name: Role-detection gate changes — smoke against ≥3 transcript shapes before commit
description: Every change to a Junior-vs-advisor role-detection gate (in role-signal-utilisation.sh or any future hook that classifies sessions by role) must run a smoke harness against three real transcript shapes — real Junior worker, finalize agent, advisor session — BEFORE commit. Four sequential prior fixes on the same hook shipped because each was test-cased against an incomplete shape set.
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
