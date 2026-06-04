# [role:bm-task] m1-a-bm-triage-1 — triage CR findings on PR #179

## 1. Role + dispatch

`[role:bm-task] m1-a-bm-triage-1 — apply user-approved CR triage to pr-179-findings.yaml`

Run `/bm-triage` per `.claude/commands/bm/bm-triage.md`.

## 2. Scope

Apply the following user-approved bucket assignments to
`.claude/PRPs/reviews/pr-179-findings.yaml` (on phase-m1-a @ 184958047).

The findings YAML lives in the phase branch worktree:
`brehon-fork-m1a/.claude/PRPs/reviews/pr-179-findings.yaml`
(or fetch via `git show phase-m1-a:.claude/PRPs/reviews/pr-179-findings.yaml`)

**IMPORTANT:** The findings YAML is in the phase-m1-a branch, NOT governance-v0.
Read it from the m1a worktree or via git-show on the phase branch.
After mutating, commit the updated findings YAML on phase-m1-a and push.

| Finding | Severity | New bucket | Rationale |
|---|---|---|---|
| cr-001 | major | `rebut` | Finding is in `.claude/decision-queue.json` (advisor-authored content, BM file-ownership hard boundary). DQ --ignored flag inconsistency is an advisor-side doc note, not a bridge source defect. |
| cr-002 | low | `rebut` | Finding is in `.claude/PRPs/briefs/m1-a-impl-12.md` (advisor-authored brief, already shipped). Markdownlint on a committed advisor brief is not a bridge source issue. |
| cr-003 | major | `fix-in-pr` | Real bug: registration.yaml uses `@brehon_.*` but bridge generates `@_brehon_.*` puppet MXIDs. Regex must match `@_brehon_.*` to avoid appservice user-query misses. |
| cr-004 | major | `rebut` | CR claims `github.com/matrix-org/conduit` is non-existent but it is the original upstream Conduit repo (exists). Current AGPL-NOTICE.md cites both the Tuwunel fork (`girlbossceo/conduit`) and upstream Conduit (`matrix-org/conduit`) accurately. Finding is incorrect. |
| cr-005 | major | `rebut` | "Bidirectional relay" comment: M1 bridge is intentionally outbound-only (Matrix→Brehon) at this scope; the doc wording correctly reflects M1 design. CR's clarification requests on polling behavior and as_token target are minor doc nits on advisor-authored design docs, not bridge source bugs. |
| cr-006 | low | `fix-in-pr` | Valid minor improvement: add Tuwunel issue #465 URL to docker-compose ip_source comment for operator reference. |
| cr-007 | low | `rebut` | The registration.yaml mount IS functional in the docker-compose stack (Tuwunel reads it at startup). CR's clarification request adds no correctness value. Wont-fix. |
| cr-008 | low | `fix-in-pr` | Valid: puppet user namespace should be exclusive=true to prevent other AS from claiming puppet MXIDs. |
| cr-009 | major | `fix-in-pr` | Valid: POST /admin/provision-room lacks capability checks against reputation_snapshot flags. Per ADR-014, hardcoded capability checks are required before admin operations. Add 403 guard. |
| cr-010 | low | `fix-in-pr` | Valid: missing room_alias should return HTTP 400, not default to brehon-default. Input validation at API boundary. |
| cr-011 | major | `fix-in-pr` | Valid: axum::serve errors should propagate; soft_pause JoinHandle should be retained for supervision; select! for coordinated shutdown. Real robustness issue. |
| cr-012 | major | `fix-in-pr` | Valid: reqwest::Client in provision.rs and relay.rs has no timeout. Must add explicit timeout to avoid hung tasks. |
| cr-013 | major | `fix-in-pr` | Valid: brehon_user fed directly into Matrix MXID localpart without sanitisation. Characters outside Matrix localpart set will produce invalid MXIDs. Need escape/unescape helpers. |
| cr-014 | major | `fix-in-pr` | Valid: ensure_puppet may cache stale MXID mappings on error paths; only cache on success or M_USER_IN_USE. |
| cr-015 | major | `fix-in-pr` | Valid: relay failure currently returns Ok, ACKing the transaction so Tuwunel won't retry. Must return Err on relay failure. |
| cr-016 | critical | `carry-forward` | Explicitly scoped M1 limitation: relay.rs line 116 has comment "M1: best-effort — only hits when state_key is a puppet MXID" and line 180 "M1: only checks state_key. Task 13 replaces with room-membership lookup." Known incomplete at M1 scope; Task 13 replacement is the intended fix. Carry to m1-b/next phase. |
| cr-017 | critical | `carry-forward` | Explicitly scoped M1 stub: relay.rs line 7 comment "M1: send_as_puppet is a stub; full implementation gated by Task 13." Intentionally incomplete at M1 scope. Carry to m1-b/next phase. |
| cr-018 | major | `fix-in-pr` | Valid: soft_pause.rs Client::new() has no timeout; must use ClientBuilder::timeout() with interval < poll_interval. |

**fix-in-pr count: 11** (cr-003, cr-006, cr-008, cr-009, cr-010, cr-011, cr-012, cr-013, cr-014, cr-015, cr-018)
**rebut count: 5** (cr-001, cr-002, cr-004, cr-005, cr-007)
**carry-forward count: 2** (cr-016, cr-017)
**done: 0 | wont-fix: 0**

**Remaining critical open: 0** (both critical findings are carry-forward, not fix-in-pr)
**Remaining major open: 9** (cr-003, cr-009, cr-011, cr-012, cr-013, cr-014, cr-015, cr-018 = 8 major fix-in-pr; cr-005 rebut = 0 major open)

Wait — recount: fix-in-pr majors are cr-003, cr-009, cr-011, cr-012, cr-013, cr-014, cr-015, cr-018 = 8 major fix-in-pr. Plus cr-006, cr-008, cr-010 = 3 low fix-in-pr. Total fix-in-pr = 11. ✓

Update `counters` block after applying buckets. Set `recommendation: approved`.

After updating the findings YAML on phase-m1-a:
- Commit with subject: `chore(bm): m1-a triage PR #179 findings (11 fix-in-pr, 5 rebut, 2 carry-forward)`
- Push `phase-m1-a` to origin
- Append triage action to `.claude/runlog/bm-runlog.md`

## 3. Required reading

- `.claude/commands/bm/bm-triage.md` — full bm-triage procedure
- `.claude/PRPs/reviews/SCHEMA.md` — findings YAML schema
- `.claude/rules/branch-manager.md` — autonomy bounds, file-ownership

## 4. Constraints

- Edit `.claude/PRPs/reviews/pr-179-findings.yaml` on `phase-m1-a` ONLY
- Do NOT touch `crates/**`, `migrations/**`, `tests/**`, `services/bridge/src/**`
- Do NOT post any PR comment
- Do NOT merge
- `--repo barrie-cork/lemmy` on every `gh` command
- Append triage action to `.claude/runlog/bm-runlog.md` (create if absent)
- The two carry-forward findings (cr-016, cr-017) must have `bucket: carry-forward`
  and `notes` updated to cite the explicit M1 scope comments in relay.rs
