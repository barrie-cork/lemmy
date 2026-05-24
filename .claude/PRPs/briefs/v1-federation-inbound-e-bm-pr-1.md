> **[CLOSED — shipped 2026-05-24, advisory-lock phase]** This brief is from the
> completed `v1-federation-inbound-e` sub-phase (pg_advisory_xact_lock eviction fix).

# Brief: bm-pr — v1-federation-inbound-e

## 1. Role + dispatch

`[role:bm-task] bm-pr v1-federation-inbound-e — open PR into governance-v0`

## 2. Scope

Open a PR from `phase-v1-federation-inbound-e` into `governance-v0` with `--repo barrie-cork/lemmy`.

**Do NOT:** merge, post inline comments, or approve. PR open only.

**Commit log (governance-v0..phase-v1-federation-inbound-e):**
```
985147122 chore(merge): merge-forward governance-v0 into phase-v1-federation-inbound-e pre-PR
6c77b6566 docs(advisor): conformance-audit phase-diff v1-federation-inbound-e — 0 Tier-1/2/3 findings
754102e32 docs(advisor): brehon-verify v1-federation-inbound-e — all stories ✓
a4c5b5fe4 docs(retro): v1-federation-inbound-e — TOCTOU fix in evict_oldest_unreviewed_if_needed shipped
1f0af4f4c chore(decision-queue): advisor-laptop mutated DQ 364754a7bcbd-001 — pass validate-pending-laptop (task 2)
9ead828bb test(fed-in-e): concurrent receivers respect storage cap (task 2)
c31add660 fix(fed-in-e): acquire_evict_lock separator \x00 -> ':' to satisfy Postgres TEXT null-byte invariant (task 1 amend)
6961a915e feat(fed-in-e): serialise per-peer storage-cap eviction with pg_advisory_xact_lock (task 1)
```

**PR title:** `v1-federation-inbound-e — serialise per-peer inbox eviction via pg_advisory_xact_lock`

**PR body template:**
```
## Summary
- Closes Race A (over-eviction) + Race B (over-insertion) under concurrent inbound `PublishSanctionNotice` from the same peer.
- `acquire_evict_lock(conn, peer_domain, table_name)` acquires `pg_advisory_xact_lock(hashtextextended(peer_domain:table_name, 0))` inside each caller's `run_transaction`, serialising the COUNT-EVICT-INSERT sequence per `(peer_domain, table_name)`.
- `evict_oldest_unreviewed_if_needed_in_tx` (renamed from `evict_oldest_unreviewed_if_needed`) drops the inner `run_transaction` wrapper; all three callers now consolidate evict + insert in one outer transaction.
- Task 2 e2e: 8 concurrent receivers, cap 5 → `final_count == 5` (Race B) + `drop_log_count == 8` (Race A).
- Phase-2 full e2e: 104 passed, 0 failed.

## Verify report
`.claude/PRPs/reports/v1-federation-inbound-e-verify.md` — all 3 stories ✓

## Conformance audit
`.claude/PRPs/reports/conformance-audit-phase-diff-fed-in-e-2026-05-23.md` — 0 Tier-1/2/3 findings

## Plan reference
`.claude/PRPs/plans/v1-federation-inbound-e.plan.md`
```

## 3. Required reading

- `.claude/rules/branch-manager.md` — file-ownership, autonomy bounds
- `.claude/rules/gh-pr-fork-target.md` — `--repo barrie-cork/lemmy` mandatory
- `.claude/rules/phase-branch.md` — PR must target `governance-v0`, not `main`

## 4. Constraints

- `--repo barrie-cork/lemmy` on every `gh pr` command
- Base branch: `governance-v0`
- Not draft (CodeRabbit skips drafts)
- No force-push
- Write result (PR number + URL) to `.claude/runlog/bm-runlog.md`
