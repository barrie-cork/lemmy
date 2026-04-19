# Post-Phase-6 polish — intake runlog

**Started:** 2026-04-19T17:20Z
**Base:** `governance-v0` @ `08065e1a1` (PR #46 merge)
**Source memory:** `project_brehon_post_phase6_cleanup.md` (advisor-side homeserver)
**Target:** ship v0.0.0 tag after all polish PRs land on `governance-v0`

---

## Shipping strategy (from memory)

| PR | Items | Branch | Priority |
|---|---|---|---|
| **polish-1 critical bugs** | 1–3 (+ maybe 4) | `polish/critical-bugs` | 🔴 blocks tag |
| **polish-2 Bucket C** | 5–7 | `polish/bucket-c` | 🟠 ship if cheap |
| **polish-3 docs** | 12 | `polish/docs-sweep` | 🟠 low risk |
| **polish-N topic** | 13–21 | `polish/<topic>` per item | 🟡 evaluate |
| **polish-tag** | 23 | `polish/v0-tag` | 🟢 last |

Rules: base `governance-v0`, merge (not squash), CR critical-only discipline.

---

## Phase 0 — triage (in progress)

Running parallel research agents to answer:

1. **Which issues are still open?** Cross-reference memory's issue list (#33, #34, #35, #36, #37, #38, #39, #47, #48, #49, #50, #51, #52, #53, #54) against `gh issue list` current state. Some may already be closed (memory notes #49 should be closeable).
2. **What's the current code state for polish-1 items (#48, #35, #34, #33)?** Read the files, assess fix difficulty, confirm memory's scope against what's actually there.
3. **What trivial docs items (#38, #39, #50, #51, #52) can be bundled?** Confirm each exists + check cost.

Agents will return triage notes; advisor will then pick a starting PR and sequence the work.

---

## Agent assignments

- **Agent-T1 (issues)** — `gh issue list --state open --repo barrie-cork/lemmy` full dump + verify claims in memory
- **Agent-T2 (polish-1 code)** — read `governance_log::append`, `submit_jury_vote` post-decision block, `federation_outbox::send_local_sanction_notice`, `request_appeal` guard, `admin_assign_jury` pool filter
- **Agent-T3 (docs bundle)** — read each of #38/#39/#50/#51/#52, confirm scope, estimate combined diff size

## Next

Once triage agents return, the advisor writes a concrete PR sequence
(starting with polish-1) and spawns impl agents per PR.
