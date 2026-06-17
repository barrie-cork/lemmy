---
role: bm-task
verb: bm-pr
phase: m3-core-entry-kinds
base_branch: governance-v0
created: 2026-06-18
---

# bm-task brief — m3-core-entry-kinds bm-pr

**Role:** `[role:bm-task]`
**Verb:** `bm-pr`
**Phase:** `m3-core-entry-kinds`
**Base branch:** `governance-v0`

## Dispatch line

```
[role:bm-task] m3-core-entry-kinds bm-pr — see .claude/PRPs/briefs/m3-core-entry-kinds-bm-pr-1.md
```

## Scope

Open a PR from `phase-m3-core-entry-kinds` into `governance-v0` on `barrie-cork/lemmy`.

- Title: `feat(db_schema,api): M3 town-hall chair/mute entry-kind consts + shim + ROOM_KINDS 10→13`
- NOT a draft (CodeRabbit must review)
- `--repo barrie-cork/lemmy` mandatory

## PR body

```
## Summary

- Registers 3 new governance-log entry-kind string consts for M3 town halls (Phase 2): `ENTRY_KIND_ROOM_CHAIR_TRANSFERRED` (`room_chair_transferred`), `ENTRY_KIND_ROOM_CHAIR_OVERRIDE` (`room_chair_override`), `ENTRY_KIND_ROOM_MUTE_ALL` (`room_mute_all`) in `crates/db_schema/src/source/governance/governance_log.rs`
- Re-exports all 3 through the api shim `pub use` block (alphabetical) in `crates/api/api/src/governance/governance_log.rs`
- Extends the `ROOM_KINDS` integrity-gate array 10 → 13 (the ADR-008 `append_room_event` allow-list); both doc comments bumped 10→13
- Zero-migration: `entry_kind` is TEXT (mirrors the M2 room-kinds pattern)
- Pre-landed consts: no emitters this phase. Call sites land bridge-side in M3 phase 3 (`_CHAIR_TRANSFERRED` / `_CHAIR_OVERRIDE`) and phase 4 (`_MUTE_ALL`) via `append_room_event` (pre-landed-const exemption)
- Entry-kind registry count 69 → 72 (advisor meta-work, landed direct on `governance-v0` @ `35b16907a`)

## Plan reference

`.claude/PRPs/plans/m3-core-entry-kinds.plan.md` — Task 1 (db_schema consts), Task 2 (api shim + ROOM_KINDS), Task 4 (advisor registry reconcile)

## Validation

- Windows cargo check (`--workspace --features full`): EXIT 0 @ `f377c9444` (Finished 17m58s, lemmy_server + shim + ROOM_KINDS(13) gate compile-accept)
- Linux-compile gate: N/A — pure-logic diff (3 string consts + shim + array; no Cargo.toml/migrations/cfg touch), Windows-green == Linux-green
- e2e: N/A — const-only SCHEMA phase, no test files in diff
- Level-5 registry invariants: db_schema distinct consts = 72, shim parity = 72, no duplicate literals, ROOM_KINDS len = 13

## Commit log (phase diff)

See `git log governance-v0..phase-m3-core-entry-kinds --oneline`
```

## Required reading

- `.claude/rules/branch-manager.md`
- `.claude/rules/gh-pr-fork-target.md`
- `.claude/commands/bm/bm-pr.md`

## Constraints

- Base MUST be `governance-v0` (never `main`)
- `--repo barrie-cork/lemmy` on all `gh pr` commands
- NOT a draft
- Do NOT touch `crates/**`, `migrations/**`, `tests/**`
- The registry doc edit already landed on `governance-v0` (advisor meta-work) — it is NOT part of this PR diff; do not try to add or move it
