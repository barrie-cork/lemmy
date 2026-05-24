---
name: Verify a handover's named-but-untested system-property assumptions before writing the patch
description: When a handover proposes a "smallest change" gated on an untested assumption about a system property (env var existence, CLI behavior, file presence on a remote machine, deploy shape), the next session MUST empirically verify the assumption BEFORE the patch. One curl/ssh/grep beats a smoke task that ships in 25 min wasted. Recurrence with `feedback_falsifiable_hypothesis_before_structural_fix.md` (DQ #338 — same defect class, different artifact: a DQ's RCA there; a handover's prescription here).
type: feedback
---

## TL;DR

A handover file is a confidence-loaded artifact: the previous session wrote it after closing-stage thinking, and the next session inherits it as a near-prescription. When a handover proposes a "smallest change" (Option A / first iteration / minimum patch) that is **gated on an untested assumption about a system property** — e.g. "env var `X` exists in `claude -p` mode", "directory `/opt/foo` is a git checkout", "CLI tool `rsync` is on PATH" — the next session MUST empirically verify the assumption BEFORE writing the patch. One curl/ssh/grep beats a smoke task that ships in 25 min wasted.

The assumption-verification gate fires AFTER reading the handover's resume sequence and BEFORE committing the first patch the handover prescribes. Treat the handover's "smallest change" as the leading hypothesis, not a closed case.

## Why this matters (session-4 role-customization incident 2026-05-24)

Session-3 handover §3.3 confidently proposed Option A as the smallest-change form for fixing the Junior-side role-signal Stop hook: gate role detection on `[[ "$CLAUDE_PROMPT" =~ \[role: ]]`. The env var simply doesn't exist — Claude Code's `-p` mode is env-minimal by design (only `CLAUDE_PROJECT_DIR` + plugin paths). The handover never tested the assumption, so the patch encoded it and shipped broken.

- Smoke task #448 was the first reality check (sentinel row 533 landed via HTTP MCP, but no queue file, no role-signal sibling, no signal anywhere).
- The third `claude-code-guide` query confirmed the docs say the prompt is ONLY available via the `transcript_path` JSONL.
- Pivoted to parse `transcript_path` JSONL for the role tag (`729b13312`); verified end-to-end via smoke #449.

**Cost of the wrong-premise patch: ~25 min of smoke-task displacement** before the falsification gate would have inverted the recommendation. Two sibling untested-assumption traps in the same session:

- `/home/barrie/MCPs/project-memory-mcp/` on EliteDesk is NOT a git checkout (session-3 handover §3.3 step 1 confidently said "cd … && git pull"; exited 128 with "not a git repository" — the directory is a scp'd deployment from March).
- Drain script's `rsync` doesn't exist on Git-Bash Windows (handover §3.3 phrasing said "rsync queue file back"; failed instantly with `rsync: command not found` despite the existing `pattern_cross_platform_divergences.md`).

The pattern recurs across `feedback_falsifiable_hypothesis_before_structural_fix.md` (DQ #338, 2026-05-21): there the artifact was a DQ entry's RCA; here it's a handover's prescription. Same defect class — confidence-loaded artifact + untested premise + wrong-shaped patch.

## Why this matters (session-5 T4a incident 2026-05-24 — scope-vs-locality variant)

Session-5 inherited session-4 handover's §3.2 and §4.1, which BOTH explicitly prescribed user-scope (`~/.claude/commands/check-role-health.md`) for the new `/check-role-health` consumer slash command. Session 5 authored to that path and was already mid-spec when the user asked **"should this be project level rather than user level?"** — the question instantly falsified the handover's prescription.

Every dependency the command consumes is brehon-fork-specific:

- Role manifests live at `.claude/roles/<role>/` (project-scope, not in any user-scope dir).
- Canonical PMD is at `C:/Users/barri/Developer/brehon-fork/.project-memory/memory.db` (pmd-invariants invariant #1 — absolute path tied to ONE checkout).
- Drain script lives at `scripts/brehon/<drain-script>` (project-scope).
- The `[role:X]` tag schema, the four-role model, the config_version subtree SHA — all project-scope concepts.

A user-scope command would have installed globally but produced empty results from every other repo. The handover's recommendation carried forward unchallenged across 3 sessions (session 2 → session 3 → session 4 handovers all said "user-scope") before the user falsified it on first read. **The handover author's session never tested the assumption against the command's dependency locality.**

**Cost of detection: ~4 min of write churn** before the user-as-falsifier inverted the scope. Cost of a missed detection (had the user not asked): a globally-installed command useless from anywhere except brehon-fork, plus user-scope contamination of a project-bound concept (advisor sessions in unrelated repos seeing the command).

The defect class generalises beyond env vars + CLI tools to **any handover-prescribed configuration-locality decision** (scope, path, install location, host machine). The new mandatory check: when a handover prescribes a config decision, list the dependencies that decision constrains and verify the decision is consistent with them.

**Threshold met (2×):** session-4 incident (env var existence) + session-5 incident (scope-vs-locality). Pattern is no longer hypothesis — promoted to canonical lesson row.

## When to apply

Every time a handover file or bootstrap brief proposes a "smallest change" / "first iteration" / "minimum patch" that names a specific system property as the gate. The gate fires AFTER reading the handover in full and BEFORE writing the first patch.

Triggering signatures in the handover entry:

- Prescription cites an env var by name (`$CLAUDE_PROMPT`, `$ANTHROPIC_MODEL`, `$X`).
- Prescription cites a CLI tool by name (`rsync`, `gh`, `jq`) and assumes presence on the target host's PATH.
- Prescription cites a remote directory shape (git checkout, deploy method, on-disk artifact).
- Prescription cites a hook event, log line, file path on a remote machine, or any system property whose state the handover author may have inferred without testing.
- Prescription cites a **configuration locality** (user-scope vs project-scope, install path, host machine, scope of a slash command / hook / settings entry). Variant from session-5 T4a 2026-05-24: handover said "user-scope at `~/.claude/commands/`" for a command whose every dependency was project-bound.

Does NOT fire when:

- The prescription cites a property the handover author empirically verified in their own session (look for "verified by …" / "smoked via …" / SHA-cited test commit).
- The prescription is purely on-laptop work where the next session would catch the mismatch within seconds of execution (cheap to discover; no smoke task lost).
- The prescription is purely judgment-shaped (deciding between two design directions, neither of which is implementation-ready).

## How to apply

Three mechanical steps, total ≤5 min per assumption:

1. **Identify the named system property.** Scan the handover's "smallest change" / "first iteration" / Option A section for: env var names, CLI tool names, remote directory paths, deploy shapes, hook event names. List them with a one-line "what's assumed" note.

2. **Verify each assumption with the cheapest possible check.**
   - Env var: `ssh <host> "env | grep ^<VAR>="` or check docs (`claude-code-guide` agent for `claude -p` env contract).
   - CLI tool: `ssh <host> "command -v <tool>"` or `which <tool>` on the worktree host.
   - Remote directory shape: `ssh <host> "ls -la <path> && cd <path> && git status 2>&1 | head -3"` (catches non-git-checkout instantly).
   - Hook event / log line: read the relevant rule/docs OR ask `claude-code-guide` (1 min round-trip vs 25-min smoke task).
   - **Configuration locality (scope-vs-locality check):** list the dependencies the prescribed scope constrains (data store paths, sibling files, peer scripts, the things the artifact reads/writes). If ANY dependency is more-local than the prescribed scope, the prescription is structurally wrong. Mechanical check: for a user-scope prescription, every dependency should also be user-scope (`~/.claude/`, `~/.config/`, `$HOME/...`). For a project-scope prescription, dependencies can be project-scope OR user-scope (project includes both). Mixed dependencies + user-scope prescription = falsify.

3. **If verification falsifies any assumption: surface to user via `AskUserQuestion` with the failed assumption + a revised options list.** Do NOT proceed with the handover's named "smallest change". The handover's higher-level intent is still valid; the prescribed path is not.

If verification confirms all assumptions: proceed with the handover's smallest change.

## Hard refusals

- **NEVER commit the handover's "smallest change" patch before verifying its named system-property assumptions.** The patch may compile + look correct locally and still ship broken end-to-end. The verification step is the cheap discovery gate that exists for exactly this asymmetry.

- **NEVER assume a handover's confidence-loaded prose ("we'll just …", "smallest change is …", "Option A is …") means the assumption was tested.** Detail in the prose is correlated with the handover author's *belief*, NOT with the assumption's truth. Session-3 handover §3.3 had ~200 chars of detailed prose naming `CLAUDE_PROMPT` env + step-by-step bash; every detail was internally consistent; the env var simply doesn't exist.

- **NEVER expand the verification window past 5 min per assumption without surfacing to the user.** The gate exists to bound cost; if 5 min of probing produces ambiguous results, that's itself a signal — surface "assumption unverified, here's what I found" rather than committing to the patch.

## Cross-references

- `.claude/lessons/feedback_falsifiable_hypothesis_before_structural_fix.md` — sibling pattern; DQ #338 incident 2026-05-21 (the artifact there was a DQ's RCA, here it's a handover's prescription). Same defect class: confidence-loaded artifact + untested premise + wrong-shaped patch.
- `.claude/lessons/feedback_verify_automated_reviewer_claims_against_compiler.md` — sibling pattern for CR / Copilot claims (compile-check the proposed change BEFORE triaging).
- `.claude/lessons/feedback_advisor_dryrun_process_rule_preconditions_at_brief_author.md` — sibling pattern for brief preconditions (dry-run a transcribed process-rule's preconditions at brief-author time).
- `.claude/lessons/pattern_cross_platform_divergences.md` — Git-Bash on Windows lacks rsync; covers the cross-platform leg of this incident.
- `.claude/commands/handover/handover-advisor.md` — handover-advisor template includes a "What's pending" / "Pending inventory" section; the verification step is now wired into the brief shape per Phase 3.5's bullet list (see "verify-before-prescribe" note added 2026-05-24).
- `.claude/rules/handover.md` — handover-shared invariants. The "What never goes in a brief" section is the closest existing gate; this lesson adds the dual: what next-session MUST do BEFORE applying a brief.
- Session retro 2026-05-24 role-signal-hook-fix Change #1.
