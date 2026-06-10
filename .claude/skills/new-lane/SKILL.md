---
name: new-lane
description: Bootstrap a Brehon lane worktree. Triggered explicitly by /new-lane.
---

# new-lane — bootstrap a fully-harnessed isolated Brehon lane

Brings a dedicated `brehon-fork-<lane>` worktree online with the **full harness live** (PMD MCP,
Junior MCP, SessionStart/Stop/PreToolUse/PostToolUse hooks) and a cold-start handover, so a fresh
CC session in the new worktree can drive a sub-phase with every safety net wired. This is the
**executable** form of `.claude/lessons/feedback_phase_lane_worktree_bootstrap_checklist.md`,
hardened with the gaps the `v1-closeout-lane-setup-report.md` field test found (error-state hook
checks, gitlink-file submodule verification, citation-integrity sweep).

## When the lane isolation matters

Two human-side CC sessions on the same checkout race the shared `.claude/decision-queue.json`
(per `.claude/rules/multi-lane-worktree.md`). A dedicated worktree makes the DQ a per-worktree file
with one writer. Use this skill whenever a NEW phase will run concurrently with an existing live
lane — the canonical example is the v1 close-out lane running while M1 (`phase-m1-b`) is in flight.

## Inputs (gather before starting)

- **`<lane>`** — short lane slug (e.g. `closeout`, `rt-r6`). Worktree dir = `brehon-fork-<lane>`.
- **`<phase-branch>`** — the branch the worktree tracks (e.g. `phase-v1-closeout`). Either it
  already exists (created by `bm-cut`) or this skill creates it off `governance-v0` with `-b`.
- **`<base>`** — what to fork from when creating the branch (default `governance-v0`).
- Whether the lane will touch `crates/` (decides if the cargo-wrapper audit is needed — Step 8).

## Steps

### 1. Pre-flight: canonical clean + concurrency surface

From the **canonical** checkout (`C:/Users/barri/Developer/brehon-fork`):

```bash
cd C:/Users/barri/Developer/brehon-fork
git branch --show-current          # must be governance-v0
git status --short                  # must be clean before worktree add
git worktree list                   # see ALL active lanes — note any live phase-v1-*/phase-m1-* lane
git fetch origin
git log governance-v0..origin/governance-v0 --oneline   # empty = in sync; else pull first
```

Note any **concurrent live lane** (another `brehon-fork-<x>` on a `phase-*` branch) — the new lane
must stay isolated from it. If a lane is actively driven (its tip advanced recently), surface a
one-line lane status before proceeding (surface-first ritual, `advisor-orchestrator.md` §1).

### 2. Create the worktree

If `<phase-branch>` already exists on origin (bm-cut created it):
```bash
git worktree add ../brehon-fork-<lane> <phase-branch>
```
If it does not exist yet (meta-only early lane, branch this skill creates):
```bash
git worktree add -b <phase-branch> ../brehon-fork-<lane> <base>
```
Verify: `git worktree list` shows `brehon-fork-<lane>` on `<phase-branch>`.

### 3. Submodule init (the #1 lane footgun)

`git worktree add` does NOT init submodules. `crates/email/translations` is an empty gitlink until
initialized — any cargo touching `lemmy_email` fails `Os { code: 3, kind: NotFound }`.

```bash
cd ../brehon-fork-<lane>
git submodule update --init --recursive
```

**Verify correctly (report GAP — gitlink is a FILE in a worktree, not a dir):** do NOT test
`[ -d crates/email/translations/.git ]` — in a worktree the submodule `.git` is a *pointer file*,
so that gives a false negative. Use:
```bash
git submodule status crates/email/translations   # no leading '-' = initialized
ls crates/email/translations | wc -l             # >0 = populated
```

### 4. Wire the harness (3 gitignored files — copy, don't author)

`.mcp.json`, `.env`, `.claude/settings.local.json` are gitignored (per-worktree) and do NOT
propagate via git. Copy them from canonical (fastest, carries the correct HTTP-PMD block + hook
wiring):
```bash
cp C:/Users/barri/Developer/brehon-fork/.mcp.json            .mcp.json
cp C:/Users/barri/Developer/brehon-fork/.env                 .env
cp C:/Users/barri/Developer/brehon-fork/.claude/settings.local.json .claude/settings.local.json
```

**PMD verification — HTTP topology (2026-05-30+):** there is NO `PROJECT_MEMORY_DB` line; the
`project-memory` server is HTTP. Verify with:
```bash
grep '"url".*11435' .mcp.json    # must match http://localhost:11435/mcp
```
**Absence of `PROJECT_MEMORY_DB` is CORRECT, not a failure** (per `pmd-invariants.md` #1 — the HTTP
daemon manages the DB server-side). Do not add one. Required servers in `.mcp.json`:
`project-memory`, `junior-brehon`, `ref-context`, `tavily` (NOT `rust-analyzer-mcp` — not installed).

### 5. Verify SessionStart + PreToolUse hooks wired

The copied `settings.local.json` should carry these. Verify programmatically (per the checklist
steps 8/10/11 — `io.open(..., encoding='utf-8')` mandatory on Windows):

```bash
python -c "
import io, json
s = json.load(io.open('.claude/settings.local.json', encoding='utf-8'))
ss = s.get('hooks', {}).get('SessionStart', [])
for name in ('pmd-canonical-guard.sh', 'session-start-multi-lane-check.sh'):
    ok = any(name in h.get('command','') for e in ss for h in e.get('hooks', []))
    print(('OK  ' if ok else 'MISS') + ' SessionStart: ' + name)
t = json.load(io.open('.claude/settings.json', encoding='utf-8'))
pre = t.get('hooks', {}).get('PreToolUse', [])
ok = any('refuse-ssh-reset-hard-shared-checkout.sh' in h.get('command','') for e in pre for h in e.get('hooks', []))
print(('OK  ' if ok else 'MISS') + ' PreToolUse: refuse-ssh-reset-hard-shared-checkout.sh')
"
```
All three must print `OK`. Any `MISS` → re-apply the missing snippet from
`feedback_phase_lane_worktree_bootstrap_checklist.md` steps 6/9/11. Also confirm the scripts exist
on disk: `ls .claude/hooks/{pmd-canonical-guard,session-start-multi-lane-check,refuse-ssh-reset-hard-shared-checkout,retro-check}.sh`.

### 6. Verify the Junior completion hook by ERROR-STATE (report GAP 1)

Daemon restarts wipe/break hooks. The check is **not** "does a hook exist" — it's "does a working
hook exist." Call `mcp__junior-brehon__list_hooks` and apply the 3-state table:

| State | Action |
|---|---|
| Hook present, no `last_error` | ✅ OK — nothing to do |
| Hook absent | Recreate via `mcp__junior-brehon__create_hook` (✅/❌ on done/failed) |
| Hook present **with `last_error`** (e.g. `BuildMessage: ModuleNotFound`) | `remove_hook(<id>)` then `create_hook(...)` — the broken `check_fn` won't fire |

**Cite the hook by PURPOSE, not ID** — hook IDs are NOT stable across recreation (a recreate may
yield ID 2, ID 3…). **Trust `list_hooks` (live MCP), never cross-check against an on-disk
`junior.db`** — the MCP-connected daemon and the SSH-queried DB file can be different stores
(report's daemon-topology finding). Detail home: `feedback_daemon_telegram_completion_hook.md`.
This is daemon state, not repo state — it does NOT travel with the worktree.

### 7. Citation-integrity check (report GAP 2)

A new lane inherits the always-load rule corpus verbatim — **a broken `feedback_*.md` citation in
an always-load rule is inherited too** (GAP 2: a rule cited a lesson file that did not exist
anywhere). Two tiers — **the `.claude/rules/` subset is the dangerous one** (always-loaded, so a
broken citation silently misleads every session); handover citations are point-in-time and lower
priority.

```bash
# TIER 1 (always-load rules — the dangerous subset, fix these):
echo "=== broken citations in ALWAYS-LOAD rules ==="
grep -rhoE '(feedback|reference)_[a-z0-9_]+\.md' .claude/rules/ 2>/dev/null | sort -u \
  | while read f; do [ -f ".claude/lessons/$f" ] || echo "  RULE-BROKEN: $f"; done
# TIER 2 (handovers — informational, often references lessons that existed then):
echo "=== broken citations in handovers (lower priority) ==="
grep -rhoE '(feedback|reference)_[a-z0-9_]+\.md' .claude/PRPs/handovers/ 2>/dev/null | sort -u \
  | while read f; do [ -f ".claude/lessons/$f" ] || echo "  HANDOVER-BROKEN: $f"; done
```

**Baseline note (2026-06-04):** on the current corpus this surfaces ~3 TIER-1 (rule) + ~14 TIER-2
(handover) known-broken citations — pre-existing drift the close-out plan's Phase 1d already
targets. So: **surface any NEW TIER-1 break to the user immediately** (a rule the lane just
inherited cites a missing lesson — that's GAP 2 recurring); record TIER-2 / known-baseline items as
a seed for the lesson-corpus audit, don't block on them. Do not silently ignore a TIER-1 break.

### 8. (Optional — only if the lane touches `crates/`) Cargo-wrapper audit — REFERENCE

If the lane will compile Rust, the wrapper must be proven green before any cargo-gated phase. This
skill does **not** run the audit — it points to the gate. Run the 4-probe audit per
`.claude/rules/pre-phase-harness-audit.md` (Probe 1 fast `-p` smoke, Probe 2 `--features full`,
Probe 3 e2e compile, Probe 4 negative feature-propagation). **Budget ~20–25 min** for a cold target
dir (Probes 2+3 are the long pole); overlap it with Steps 9–10. On all-pass:
`touch .claude/audit-phase-<phase-branch>-complete.flag`. Skip this step entirely for doc/meta-only
lanes (close-out Phases 0–3, lesson edits, etc.).

### 9. Quantify inherited baseline (report hygiene items)

A fresh lane inherits the trunk's full state. Capture the baseline so the first phase knows what to
clean:
```bash
echo "DQ: $(wc -c < .claude/decision-queue.json) bytes, pending=$(python -c "import io,json;print(len(json.load(io.open('.claude/decision-queue.json',encoding='utf-8'))['pending']))")"
echo "active plans: $(ls .claude/PRPs/plans/*.plan.md 2>/dev/null | wc -l)"
echo "lessons: $(ls .claude/lessons/*.md 2>/dev/null | wc -l)"
ls -d C:/Users/barri/Developer/brehon-fork-* 2>/dev/null    # worktree dirs (live + stale)
```
**If DQ > 200 KB or > 100 resolved entries**, flag `dq-archive.sh` for early in the lane's first
phase (over the archive-policy trigger; per `decision-queue.md`). The lane writes its OWN DQ —
never mutate phase-branch DQ from the canonical checkout.

### 10. Write the cold-start handover

Author `.claude/PRPs/handovers/<phase-branch-or-lane>-bootstrap.md` — self-contained, readable with
zero conversation context. MUST include: current state, the lane's purpose, what's safe-now vs
gated, any **prime-directive gates verbatim** (e.g. an M1-merge gate `git log origin/<other-lane>
^origin/<gate-branch>` must be empty; a NO-ELITEDESK directive), resolved decisions, the verified
harness state (this skill's Step 4–7 results), the isolation guarantee, and the next concrete
action with a `VERIFIED_AT: <SHA>` line. Mirror `v1-closeout-bootstrap.md` (the gold-standard
example the report names). Commit it on `<phase-branch>` (`.claude/` meta-work → direct commit per
`phase-branch.md`); push the branch so the handover survives worktree loss.

### 11. Report

Surface a compact lane-up summary in the conversation:
```
Lane up: brehon-fork-<lane> on <phase-branch> (pushed to origin)
Harness: worktree ✓ | submodules ✓ | .mcp.json/.env/settings.local.json ✓ | hooks ✓(2 SS + PreToolUse) | PMD HTTP ✓ | Junior hook <state>
Citations: <all resolve | N broken — surfaced>
Cargo audit: <ran/passed | deferred (meta-only) | referenced>
Baseline: DQ <size>, <N> plans, <M> lessons — <archive flag if over threshold>
Isolation: dedicated worktree + branch + DQ; PMD shared (HTTP, by design). Other live lane: <name or none>.
Next: open a CC session with CWD = brehon-fork-<lane>; read .claude/PRPs/handovers/<...>-bootstrap.md
```

## Hard refusals

- **Never `rm -rf` a worktree dir** to recreate — use `git worktree remove` so `.git/worktrees/<name>/`
  admin state is cleaned (`multi-lane-worktree.md` hard refusal #4).
- **Never write phase-branch DQ from the canonical checkout** — the lane owns its DQ
  (`multi-lane-worktree.md` hard refusal #2).
- **Never author `.mcp.json` from `.mcp.json.example`** — the example has `/path/to/` placeholders
  that cause `-32000` MCP failures. Copy from canonical instead.
- **Never assume a hook is healthy because it exists** — check `last_error` (Step 6).
- **Never trust an on-disk `junior.db` for hook state** — trust `list_hooks` (the live MCP).
- **Never commit another session's uncommitted/draft files** when committing the handover — stage
  only the handover path explicitly.

## See also

- `.claude/lessons/feedback_phase_lane_worktree_bootstrap_checklist.md` — the canonical 12-step
  checklist this skill operationalizes (the authoritative snippet source for any MISS).
- `.claude/PRPs/reports/v1-closeout-lane-setup-report.md` — the field test that found the GAPs this
  skill hardens against.
- `.claude/PRPs/handovers/v1-closeout-bootstrap.md` — the gold-standard handover to mirror (Step 10).
- `.claude/rules/multi-lane-worktree.md` — lane lifecycle + hard refusals.
- `.claude/rules/pmd-invariants.md` #1 — canonical/HTTP PMD topology (Step 4).
- `.claude/lessons/feedback_daemon_telegram_completion_hook.md` — Junior hook 3-state table +
  ID-instability + daemon-store caveat (Step 6).
- `.claude/rules/pre-phase-harness-audit.md` — the cargo 4-probe gate (Step 8, referenced).
