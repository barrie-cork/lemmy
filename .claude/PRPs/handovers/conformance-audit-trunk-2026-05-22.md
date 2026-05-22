---
phase: governance-v0 (trunk)
scope: brehon-conformance-audit + cargo gates
date: 2026-05-22
session_id: 43cae6fc-3e14-41b7-b0e1-3092948d2ff6
status: complete (audit + roll-up shipped; test --no-run in flight at session save)
---

# Brehon conformance audit — trunk @ `7e6c4202f` — handover

## What this session did

Ran the "natural Brehon stack" (cargo §15 gates + brehon-conformance-audit
across all 6 axes) against `governance-v0` trunk in an isolated worktree so
v1-federation-inbound-d planning could continue undisturbed on the canonical
checkout.

## Where the work landed

**Audit worktree:** `C:/Users/barri/Developer/brehon-fork-audit-2026-05-22`
(detached HEAD at `7e6c4202f`, off `origin/governance-v0`).

Worktree is NOT to be deleted at session-end — its `target/` dir is the only
artifact distinguishing warm vs cold rebuilds, and the `.claude/PRPs/reports/`
+ `.claude/PRPs/audit-metrics/` + cargo logs are durable inside it. Either
keep it as-is for follow-up, or copy artifacts into the canonical checkout
and `git worktree remove` it.

**Canonical checkout (this CWD, `C:/Users/barri/Developer/brehon-fork`):**
the only file touched in this session is THIS handover file. All audit
artifacts live in the worktree.

## Audit results — headline

**Zero Tier-1 / catch-fire findings.**

| Axis | T1 | T2 | T3 | Notes |
|---|---:|---:|---:|---|
| 1 conn-type | 0 | 0 | 0 | 17 `.run_transaction(` callsites on `&mut DbConn<'_>`; 48 helper fns on `&mut AsyncPgConnection` (canonical closure-body helpers) |
| 2 append reborrow | 0 | 20 | 0 | 3-token `&mut conn.into()` vs canonical 4-token; identical runtime |
| 3 trait-bound | 0 | 1 | 1 | Phase-6 axis-3 fixed on trunk by `b9691c0ab`; F3-1: implicit `Send` via async_trait |
| 4 error idiom | 0 | 0 | 5 | Phase-6 Finding 6.1 defect class **absent**; 5 Tier-3 are URL/local-JSONB |
| 5 conn acquisition | 0 | 5 | 1 | 20 canonical / 6 variant; F1 + F5 are clean unnecessary divergences |
| 6 ADR-015 | 0 | 0 | 0 | ADR-015 / GDPR pseudonymisation contract uniformly held |
| **Total** | **0** | **26** | **7** | |

## Cargo gates

| Gate | Status | Wall clock |
|---|---|---|
| `cargo check --workspace --features full` (canonical, warm) | pass (exit 0) | 9m 13s |
| `cargo check --workspace --features full` (worktree, cold) | pass (exit 0) | 23m 26s |
| `cargo clippy --workspace --features full --no-deps -- -D warnings` | pass (exit 0) | 8m 41s |
| `cargo test --workspace --features full --no-run` | **IN FLIGHT at session save** | log path below |

**`test --no-run` resume check (first action for the next session):**

```bash
tail -5 C:/Users/barri/Developer/brehon-fork-audit-2026-05-22/.claude/build-test-norun-audit-worktree-2026-05-22.log
```

Expect either `CARGO_TEST_NORUN_EXIT_0` or `CARGO_TEST_NORUN_EXIT_NONZERO` at
the end. If still compiling (no exit marker), wait — typical wall clock from
clippy-warm cache is ~10–15 min. Background bash ID was `bj6xhwd7t` but the
shell session that owns it ends with this session; the cargo subprocess
itself continues to completion regardless (writes to the log file).

Once the exit lands, update the rollup's gate-table row 4
(`.claude/PRPs/reports/conformance-audit-trunk-rollup-2026-05-22.md`) by
editing `in flight` → the actual result.

## Audit artifacts (all in the audit worktree)

Reports (`.claude/PRPs/reports/`):

- `conformance-audit-trunk-axis-1-2026-05-22.md` (authored inline by advisor)
- `conformance-audit-trunk-axis-2-2026-05-22.md` (subagent a540c19dcaf18b544)
- `conformance-audit-trunk-axis-3-2026-05-22.md` (subagent a16fdd5ba4caa9bbd)
- `conformance-audit-trunk-axis-4-2026-05-22.md` (subagent accfd215d017c287a)
- `conformance-audit-trunk-axis-5-2026-05-22.md` (subagent a1357079f35dc0ce2)
- `conformance-audit-trunk-axis-6-2026-05-22.md` (subagent ad7dd57064fa7f1d4)
- `conformance-audit-trunk-rollup-2026-05-22.md` (advisor synthesis)

Metrics (`.claude/PRPs/audit-metrics/`):

- `trunk-axis-2-2026-05-22.json` … `trunk-axis-6-2026-05-22.json`
- `_axis2-evidence-snapshot.txt` (pre-subagent-pivot in-flight grep evidence)

Cargo logs (in audit worktree `.claude/`):

- `build-check-audit-worktree-2026-05-22.log`
- `build-clippy-audit-worktree-2026-05-22.log`
- `build-test-norun-audit-worktree-2026-05-22.log` ← check first on resume

Trunk-baseline log (canonical checkout `.claude/`):

- `build-check-trunk-2026-05-22.log`

## fed-in-c re-audit decision (locked)

`phase-v1-federation-inbound-c` was merged into `governance-v0` on 2026-05-21
(`7cfc21c23`) with `--delete-branch`. The local ref is absent. Its merged
code IS part of this trunk audit. No separate re-audit task remains.

## Recommendations for the next sub-phase plan §3 watch list

(From the rollup §Recommendations — restate so next session can read this
file standalone without opening the rollup.)

1. **Axis-2 mechanical back-fill (20 callsites)** — single `chore(lint):`
   commit can normalise all 20 to the canonical 4-token form. Low-risk,
   mechanical. Bundle candidate for any quiet-window cleanup phase.
2. **Axis-5 F1 + F5 clean unnecessary divergences** — `admin_dashboard.rs:68-69`,
   `get_my_reputation.rs:37-38`. Same shape as #1; bundle in the same commit.
3. **Axis-3 F3-1 documentation-parity fix** — add `+ Send` to
   `wrap_governance_inbound`'s generic bound (`inbox.rs:493`). One-line
   edit; bundle with #1.
4. Axis-5 F2/F3/F4 (low priority) — DON'T back-fill; partial functional
   justification (intermediate `get_bool` / `get_int` calls reuse `pool`
   before `get_conn`). Re-evaluate only if a future refactor touches the
   same files.
5. Axis-4 Tier-3 sites + axis-6 zero findings — no action.

## What is NOT in this session's scope

- No DQ entries written. The audit was advisor-self-driven; nothing
  judgment-heavy required user-relay.
- No commits to `governance-v0` other than this handover file.
- No Junior tasks dispatched. The 5 axis subagents were advisor-side
  `Agent` tool dispatches (per `.claude/rules/advisor-orchestrator.md`
  §6.1 parallel-dispatch pattern; second occurrence of the pattern
  worth noting in the next retro — first was 2026-05-22 parallel retro
  followups).
- v1-federation-inbound-d planning was NOT touched by this session.
  Brief `.claude/PRPs/briefs/v1-federation-inbound-d-planning-1.md`
  remains in the state the canonical-checkout session left it (with
  the modification noted in `git status` at session start).

## Resume the audit follow-up (if/when)

The 26 Tier-2 + 7 Tier-3 watch-list items become real work only when:

- A `chore(lint):` cleanup phase is queued (good fit for a quiet
  window between sub-phases) — bundle recommendations 1+2+3 above.
- The next sub-phase plan's §3 watch list pulls items in.

No timeline is encoded; this audit is preventive, not reactive.

## Cross-session notes for the next advisor session

- **MCP / PMD:** the audit worktree's `.mcp.json` was bootstrapped with
  the canonical absolute `PROJECT_MEMORY_DB`
  (`C:/Users/barri/Developer/brehon-fork/.project-memory/memory.db`)
  per pmd-invariants.md #1. `PROJECT_ROOT` is the canonical checkout
  path — fine for read-only audit, but if the next session writes
  `memory_write_eval` from inside the audit worktree, consider
  updating `PROJECT_ROOT` to the worktree path.
- **Surface-first ritual:** when the next session opens, run the
  ritual per `.claude/rules/advisor-orchestrator.md` §1
  (`pwd && git branch --show-current && git worktree list`). Expected
  output: canonical `brehon-fork` on `governance-v0`, audit worktree
  on detached HEAD `7e6c4202f`, tooling worktree on
  `tooling-local-validation`.
- **DQ hook:** if UserPromptSubmit DQ-pending hook reports pending > 0
  at session start, run the surface-first lane status line BEFORE any
  tool call.

## Session retro candidate

A `session-retro-2026-05-22-conformance-audit-trunk.md` retro would
record:

- §6.1 parallel-dispatch pattern: second occurrence (first was
  2026-05-22 retro followups), getting close to the 2nd-distinct-session
  promotion threshold per `feedback_principles_not_rules.md`.
- Worktree-for-audit isolation: clean separation; canonical-checkout
  planning + audit ran in parallel without interaction.
- Subagent brief shape: 5 self-contained briefs each ≤350 words,
  agent return summaries each ≤200 words. Total wall clock ~17 min
  for all 5 axes (vs ~50min serial). Strong demonstration.

Defer authoring until either: (a) test --no-run lands and the rollup
is finalised, or (b) the user explicitly asks for `/reflect`.
