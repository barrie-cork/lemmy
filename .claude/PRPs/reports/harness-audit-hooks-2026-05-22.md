# Harness audit — `.claude/hooks/` — 2026-05-22

**Branch:** `governance-v0`
**Run by:** `harness-audit` skill shape (adapted for hook-directory audit per `watch_hook_dir_audit_pending.md` trigger)
**Scope:** Hook scripts + their wiring in `settings.json` / `settings.local.json`. Pi-Coding excluded (Pi has its own `.pi/hooks/`).

## Headline counts

| Metric | Value |
|---|---|
| Total hook scripts in `.claude/hooks/` | **14** (excluding `README.md`) |
| Documented in `README.md` Inventory table | **7** |
| Documentation drift | **7** (50% undocumented) |
| Wired in tracked `.claude/settings.json` | **10** distinct scripts (some wired in multiple arrays) |
| Wired only in `.claude/settings.local.json` (gitignored) | **2** (`pmd-canonical-guard.sh`, `session-start-multi-lane-check.sh`) |
| Unwired in any `settings*.json` (laptop side) | **2** (`allow-prp-deliverables.sh`, `test-hooks.sh`) |
| Latency outliers (>500ms on Windows P50 laptop) | **1** (`validate-memory-search.sh` at ~964ms; `pmd-canonical-guard.sh` borderline at 926ms) |

## Inventory

| Script | Lines | Event | Matcher | Class | Wired in | Latency (ms) | In README |
|---|---:|---|---|---|---|---:|:---:|
| `prp-ralph-stop.sh` | 142 | `Stop` | (any) | LOOP-CONTROL | settings.json | (not timed — Stop hook may mutate) | ✓ |
| `retro-check.sh` | 198 | `Stop` | (any), timeout 10000 | LOOP-CONTROL + BLOCK | settings.json | (not timed — Stop hook may mutate) | ✗ |
| `check-cargo-pipe.sh` | 87 | `PreToolUse` | `Bash` | BLOCK | settings.json | ~515 | ✓ |
| `refuse-ssh-reset-hard-shared-checkout.sh` | 130 | `PreToolUse` | `Bash` | BLOCK | settings.json | ~683 | ✓ |
| `validate-memory-search.sh` | 69 | `PreToolUse` | `mcp__project-memory__memory_search\|mcp__vault-memory__memory_search`, timeout 5000 | BLOCK (deny JSON) | settings.json | **~964** | ✗ |
| `worktree-guard.sh` | 98 | `PreToolUse` | `Bash\|Edit\|Write\|MultiEdit`, timeout 5000 | BLOCK (deny JSON) — daemon-only via CWD gate | settings.json | ~617 (laptop no-op path) | ✗ |
| `allow-prp-deliverables.sh` | 119 | `PreToolUse` | `Write\|Edit\|MultiEdit` (intended) | MUTATE (allow JSON) — daemon-only via CWD gate | **UNWIRED** on laptop; deployed on EliteDesk daemon | ~780 (laptop no-op path) | ✗ |
| `inject-dq-state.sh` | 78 | `UserPromptSubmit` | (any) | MUTATE (additionalContext JSON) | settings.json | ~348 | ✓ |
| `pre-phase-audit.sh` | 76 | `SessionStart` | `startup` + `resume` | MUTATE (additionalContext JSON) | settings.json (two entries — one per matcher) | ~152 | ✓ |
| `pmd-canonical-guard.sh` | 147 | `SessionStart` | `.*` | WARN | **settings.local.json** | ~926 | ✓ |
| `session-start-multi-lane-check.sh` | 98 | `SessionStart` | `.*` | WARN | **settings.local.json** | ~447 | ✓ |
| `edit-readback-reminder.sh` | 45 | `PostToolUse` | `Edit\|Read`, timeout 5000 | MUTATE (additionalContext stdout) | settings.json | ~243 | ✗ |
| `observation-capture.sh` | 84 | `PostToolUse` | `.*`, timeout 5000 | MUTATE (writes JSONL to `~/.cache/tw-observations/`) | settings.json | ~652 | ✗ |
| `test-hooks.sh` | 128 | n/a — manual test harness | n/a | (developer tool) | **UNWIRED** | n/a | ✗ |

**Totals:** 14 scripts, 1,499 lines. ALWAYS-fire hooks on the laptop (per session start + per Bash + per UserPromptSubmit + per PostToolUse): 11. Two SessionStart hooks chain (~1.4s ceremony per session start). One BLOCK hook (`validate-memory-search.sh`) is the lone >500ms outlier in the hot path.

## Class breakdown

- **WARN-not-FAIL** (stderr only, exit 0 always; per `pmd-invariants.md` §5 + `advisor-orchestrator.md` §1): `pmd-canonical-guard.sh`, `session-start-multi-lane-check.sh`. Both wired only in gitignored `settings.local.json` — every fresh lane bootstrap must re-add them per `feedback_phase_lane_worktree_bootstrap_checklist.md` step 6.
- **BLOCK** (exit 2 with fix-message OR deny JSON): `check-cargo-pipe.sh`, `refuse-ssh-reset-hard-shared-checkout.sh`, `validate-memory-search.sh`, `worktree-guard.sh`, `retro-check.sh` (Stop block ≤3 attempts, fail-open after).
- **MUTATE** (writes state or emits JSON Claude Code consumes): `inject-dq-state.sh`, `pre-phase-audit.sh`, `edit-readback-reminder.sh`, `observation-capture.sh`, `allow-prp-deliverables.sh`.
- **LOOP-CONTROL** (Stop hook for autonomous loops): `prp-ralph-stop.sh`, `retro-check.sh`.

## Wiring map

### Tracked (`.claude/settings.json`) — propagates to every lane

| Event | Matcher | Hooks (in array order) | Notes |
|---|---|---|---|
| `Stop` | (any, first entry) | `prp-ralph-stop.sh` | Ralph loop continuation |
| `Stop` | (any, second entry) | `retro-check.sh` (timeout 10000) | Post-task retro enforcement |
| `PreToolUse` | `Bash` | `check-cargo-pipe.sh`, `refuse-ssh-reset-hard-shared-checkout.sh` | In array order — cargo-pipe runs first |
| `PreToolUse` | `mcp__project-memory__memory_search\|mcp__vault-memory__memory_search` | `validate-memory-search.sh` (timeout 5000) | Substring matcher — hits `_hybrid` too, hook exempts internally |
| `PreToolUse` | `Bash\|Edit\|Write\|MultiEdit` | `worktree-guard.sh` (timeout 5000) | Daemon-only via CWD gate; laptop fast-exit |
| `UserPromptSubmit` | (any) | `inject-dq-state.sh` | Fires every prompt |
| `SessionStart` | `startup` | `pre-phase-audit.sh` | Phase-branch audit reminder |
| `SessionStart` | `resume` | `pre-phase-audit.sh` | Same hook, second entry (duplicated wiring) |
| `PostToolUse` | `Edit\|Read` | `edit-readback-reminder.sh` (timeout 5000) | Edit counter at threshold 5 |
| `PostToolUse` | `.*` | `observation-capture.sh` (timeout 5000) | Shadow-mode JSONL detector |

### Lane-local (`.claude/settings.local.json`) — gitignored, must be re-bootstrapped per lane

| Event | Matcher | Hooks | Status |
|---|---|---|---|
| `SessionStart` | `.*` | `pmd-canonical-guard.sh`, `session-start-multi-lane-check.sh` | **Drift risk:** invariant lives in `pmd-invariants.md` §5 but the wiring is gitignored. A new lane worktree without these hooks ships without the WARN guard. Documented bootstrap step exists. |

### Unwired

| Script | Status | Disposition |
|---|---|---|
| `allow-prp-deliverables.sh` | Tracked in working tree (gitStatus: `??`), but not committed AND not wired in laptop settings. Deployed only on EliteDesk daemon (per `v1-ship-1-runlog.md` provenance: untracked on daemon as bootstrap-§3 hook). | **Daemon-side hook** — laptop copy is a reference / staging artifact. Confirm it lives canonically on EliteDesk and decide whether to track it in repo as documentation. |
| `test-hooks.sh` | Tracked in repo; intentionally never wired. | **Developer test harness** — runs `check-cargo-pipe.sh` against fixed payloads. Legitimately unwired, not unused. |

## Documentation drift

`README.md` Inventory table documents 7 of 14 scripts. Missing from README:

1. `retro-check.sh` — load-bearing Stop hook enforcing post-task retro discipline; complex Junior-vs-trunk branch routing logic; entirely absent from inventory.
2. `validate-memory-search.sh` — FTS5 word-cap enforcer on legacy `memory_search` tool.
3. `worktree-guard.sh` — Junior worktree sandbox enforcement (sudo block, symlink block, path-escape block); daemon-only via CWD gate.
4. `allow-prp-deliverables.sh` — daemon-side `.claude/PRPs/{plans,briefs,reports}/` write-allow shim (CC v2.1.119 sensitive-file gate workaround).
5. `edit-readback-reminder.sh` — PostToolUse advisory at 5-edit threshold.
6. `observation-capture.sh` — PostToolUse shadow-mode JSONL detector (4 detector rules, log at `~/.cache/tw-observations/`).
7. `test-hooks.sh` — developer test harness (acceptable to omit from operator inventory, but should be noted as "not wired by design").

The README has a self-aware drift note at line 20 acknowledging this audit was pending — that note can be retired once the inventory is updated.

## Latency outliers (>500ms)

Measured on this Windows P50 laptop in a single timing pass (warm shell). Variance ±100ms expected.

| Script | Latency | Notes |
|---|---:|---|
| `validate-memory-search.sh` | **~964ms** | Five sequential `jq -r` shell-outs over the same `$INPUT` (lines 22, 31, 32, 33, plus `echo \| wc -w \| tr`). On Windows each shell-out via Git Bash incurs ~150-200ms process spawn. The script is wired on every `memory_search*` call (substring matcher); five jq calls dominate the runtime. Hot path for PMD-heavy sessions. |
| `pmd-canonical-guard.sh` | ~926ms | SessionStart only (once per session). Acceptable but borderline — the script does multiple `git rev-parse` calls + path-resolution helper functions; if it gains additional work, watch the ceiling. |
| `allow-prp-deliverables.sh` | ~780ms | Laptop no-op path (CWD doesn't match `/srv/*/.junior/worktrees/*`). Hook still spawns + jq-shells before exiting. Negligible on laptop (unwired); on the Junior daemon this fires per Write/Edit/MultiEdit. |
| `refuse-ssh-reset-hard-shared-checkout.sh` | ~683ms | Per-Bash-call. Three sequential `printf \| grep -Eq` passes — each costs a subshell. |
| `observation-capture.sh` | ~652ms (realistic payload) | PostToolUse on `.*` — fires after **every** tool call. Most expensive hot-path hook by volume because of matcher breadth. Multiple jq shell-outs + `sed -E` heredoc stripping. |
| `worktree-guard.sh` | ~617ms | Laptop no-op path (CWD gate exits fast). Daemon-side runtime not measured here. |
| `check-cargo-pipe.sh` | ~515ms | Per-Bash-call. Python3 JSON parse + two `printf \| grep -Eq` passes. |

**Cumulative per-Bash-call cost** (laptop hot path, sequential): `check-cargo-pipe.sh` (~515ms) + `refuse-ssh-reset-hard-shared-checkout.sh` (~683ms) + `worktree-guard.sh` (~617ms) = **~1.8s overhead per Bash tool call** before observation-capture (~650ms) fires PostToolUse. Per `UserPromptSubmit`, `inject-dq-state.sh` adds ~348ms. Per session start, the two `settings.local.json` SessionStart WARN hooks chain to ~1.4s.

These costs are individually modest but compound. A session that does 30 Bash calls eats ~75s in hook latency before any tool work — invisible because hooks run between tool dispatch and tool execution.

## Ranked recommendations

### Tier 1 — ship now (high leverage, low cost)

#### 1. **Update `README.md` inventory table** to cover all 14 scripts (or 13 + "test-hooks.sh — developer test harness, not wired").
Cost: ~15 min of doc work. Leverage: the inventory drift note at README line 20 references this audit; closing that loop restores the README as a faithful map for the next operator. Future hook adds should be gated on README row-update.

#### 2. **Reduce `validate-memory-search.sh` shell-out cost** by collapsing 5 `jq -r` calls into one.
Current: 5 sequential `echo "$INPUT" | jq -r '...'` invocations (lines 22, 31, 32, 33, plus the wc pipe). Each `jq` cold-start on Windows Git Bash is ~150-200ms, dominating the ~964ms runtime. Proposed shape:

```bash
eval "$(echo "$INPUT" | jq -r '@sh "TOOL_NAME=\(.tool_name // "") QUERY=\(.tool_input.query // "") TAGS=\(.tool_input.tags // "") MEMORY_TYPE=\(.tool_input.memory_type // "")"')"
```

Estimated savings: ~600-700ms per `memory_search*` call. Hot-path impact: PMD-heavy advisor sessions (~10-30 `memory_search` calls per phase) save ~6-20s cumulative.

#### 3. **Document the lane-bootstrap wiring requirement** for `pmd-canonical-guard.sh` + `session-start-multi-lane-check.sh` in the README.
Both are wired only in `settings.local.json` (gitignored). The `pmd-invariants.md` §5 invariant is paper-only unless every new lane re-adds the wiring. Lesson `feedback_settings_local_json_worktree_bootstrap.md` covers the general case but the README is where an operator looks. One-line entry: "These two SessionStart WARN hooks live in `settings.local.json` because they reference per-lane paths; bootstrap-checklist step 6 re-wires them per lane."

#### 4. **Consolidate the duplicated `pre-phase-audit.sh` SessionStart wiring.**
Currently appears twice in `settings.json` (once per matcher `startup`, once per `resume`). Claude Code supports multi-matcher entries: `"matcher": "startup|resume"` (per docs). Collapses two entries to one. Trivial — pure config cleanup, no behavior change. ~5 lines saved in settings.json.

### Tier 2 — ship after discussion (medium cost, real impact)

#### 5. **Reorder `PreToolUse: Bash` array** to put cheaper hook first.
Current order: `check-cargo-pipe.sh` (~515ms) then `refuse-ssh-reset-hard-shared-checkout.sh` (~683ms). The cheaper hook already runs first, so order is correct. **No change needed.** Listed here for completeness — the audit's reordering recommendation is a no-op on the current ordering. (Verified by reading settings.json lines 49-62.)

#### 6. **Consider merging `pmd-canonical-guard.sh` + `session-start-multi-lane-check.sh` into one SessionStart dispatcher.**
Both are WARN-not-FAIL, both fire on the same matcher (`.*`), both share the same git-root discovery boilerplate. Their respective git probes (`rev-parse --git-common-dir`, `worktree list`, `log -1 --format=%ct`) could share a single git invocation lookup phase. Saves ~300-500ms of session-start ceremony.

Cost: ~30 min refactor + dogfood-test against the two existing trigger paths (canonical-PMD divergence + multi-lane recent-commit). Risk: dispatcher introduces one more failure point (single hook erroring affects both checks). Mitigation: dispatcher uses `set +e` between checks, prints WARNs independently.

Discuss before shipping — these two hooks intentionally exist as independent enforcers per the falsifiable-hypothesis principle. Merging may obscure attribution when one of them fires.

#### 7. **Re-evaluate `observation-capture.sh` matcher scope.**
The hook is wired on `.*` (every tool call) and runs ~650ms per realistic Bash payload. Per its header it's "shadow-mode" awaiting a 7-day precision measurement before any downstream summarizer wiring. Two paths:

- **A.** If the precision-measurement window has elapsed and no downstream consumer exists, retire the hook entirely (and the `~/.cache/tw-observations/` JSONL trail).
- **B.** If still shadow-mode, narrow the matcher from `.*` to `Bash` only (the hook's four detectors all key off `tool_input.command`, not file paths). Cuts firings from every tool call (~30/hour) to every Bash call (~10/hour) → ~15s saved per advisor session.

Audit-trail note: the hook header references "Idea PMD #914, plan PMD #918 Phase 4" — search PMD before retiring to confirm the precision measurement has concluded.

#### 8. **Audit `allow-prp-deliverables.sh` deployment provenance.**
The script is in the laptop working tree as untracked, deployed on the EliteDesk daemon as untracked-preserved (per `v1-ship-1-runlog.md`). For a load-bearing CC v2.1.119 sensitive-file gate workaround, untracked-deployment is fragile (a daemon repo reset would wipe it).

Recommendation: track in repo with a clear "DAEMON-SIDE ONLY — wire in daemon's `settings.local.json`" header. Existing internal CWD gate (`/srv/*/.junior/worktrees/*`) keeps it a no-op for laptop sessions; tracking the file only documents what's there.

### Tier 3 — defer (speculative)

#### 9. **Compile bash hooks to faster language.**
Not yet warranted. Per-hook costs are dominated by `jq` and Python3 shell-out — moving from bash to a faster shell wouldn't help. Rewriting to compiled Go binaries with embedded JSON parsing could drop per-hook cost by ~80%, but the engineering cost vs. the cumulative ~5-10s/session savings is not yet favorable. Revisit if hot-path hook count grows past 6 per Bash call.

#### 10. **Hook execution telemetry.**
Could write per-hook latency to a `~/.cache/cc-hook-perf/` JSONL trail at the end of each hook for empirical audit. Defer until a second audit (~6 months) confirms recurring latency drift; one-shot measurement (this report) is sufficient for now.

## Unused hooks (candidates for retire)

- **`test-hooks.sh`** — **Keep**. Developer test harness, intentionally unwired. README should note its purpose ("not wired by design — run manually to verify `check-cargo-pipe.sh` behavior after edits").
- **`allow-prp-deliverables.sh`** — **Keep, but track explicitly**. Daemon-side hook deployed via untracked-preserved path; the laptop working-tree copy is the canonical authoring location. Per Tier 2 #8 above, track it in repo with a header noting its daemon-only scope.

No genuine retirement candidates — every hook either fires on the laptop, fires on the daemon, or is a developer tool.

## Defence-in-depth notes (do NOT retire)

Per the task constraint "don't propose deprecating block-class hooks even if rarely hit":

- **`refuse-ssh-reset-hard-shared-checkout.sh`** prevents the 2026-05-21 daemon-finalize race (per its header). Single-incident provenance, but the prevented class is "orphaned plan-merge + ~25 min recovery" — high cost, low frequency, exactly the case for a block hook.
- **`worktree-guard.sh`** prevents the agent-grey PMD-destruction incident (PMD #998). Daemon-only via CWD gate; laptop overhead is negligible.
- **`check-cargo-pipe.sh`** prevents silent cargo failures masked by `| tail` exit-code masking. Per `cargo-output-capture.md`, every `cargo` exec is potentially destructive of correctness signal.

All three stay as-is.

## See also

- Trigger: `watch_hook_dir_audit_pending.md` PMD memory (2026-05-22 watch entry)
- Skill: `.claude/skills/harness-audit/SKILL.md` (audit shape source)
- Companion lessons: `feedback_phase_lane_worktree_bootstrap_checklist.md` (settings.local.json hook re-wiring), `feedback_settings_local_json_worktree_bootstrap.md`
- Invariants: `.claude/rules/pmd-invariants.md` §5 (SessionStart canonical-PMD guard rationale)
- Recent runlog provenance for `allow-prp-deliverables.sh`: `.claude/runlog/v1-ship-1-runlog.md` lines 162-262

## Recommendation summary

- **Top 3 Tier-1 wins (verbatim from recommendations above):**
  1. Update `README.md` inventory table to cover all 14 scripts (or 13 + "test-hooks.sh — developer test harness, not wired").
  2. Reduce `validate-memory-search.sh` shell-out cost by collapsing 5 `jq -r` calls into one. Estimated savings: ~600-700ms per `memory_search*` call.
  3. Document the lane-bootstrap wiring requirement for `pmd-canonical-guard.sh` + `session-start-multi-lane-check.sh` in the README.
- **One-liner Tier-1 #4:** Collapse duplicated `pre-phase-audit.sh` `SessionStart` wiring into one entry with matcher `startup|resume`.
- **Latency hot path:** `validate-memory-search.sh` (~964ms) is the lone meaningful outlier. Per-Bash-call cumulative hook overhead ~1.8s.
- **Documentation drift:** 7 of 14 scripts undocumented (50%); the README has a self-aware drift note acknowledging this audit was pending.
- **Pi-boundary check:** Not applicable — `.claude/hooks/` is Claude-Code-specific; Pi has its own `.pi/hooks/` (per harness-audit skill Phase 0).
- **No genuine retirement candidates.** Every hook either fires (laptop or daemon) or is a developer tool.

The user reviews this report and decides which (if any) recommendations to action. This audit does not edit hooks, settings, or README.
