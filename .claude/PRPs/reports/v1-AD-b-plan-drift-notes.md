# v1-AD-b plan-drift notes

These notes flag drift between the plan text and what actually landed. To
be folded into the v1-AD-b completion report when it is written.

## §11 line 1101 — wrong key named in test-row description

**Plan claim:** `admin_set_config_community_scope_by_moderator` should
write `liability.regular_multiplier` at community scope via a community
moderator.

**Reality:** `liability.regular_multiplier` is declared
`ConfigScope::Instance` in `crates/api/api/src/governance/config.rs:1141-1152`,
so `check_policy` correctly rejects any community-scope write of it with
`DenialReason::ScopeMismatchInstanceKey` — the moderator-write path is
never reached.

**Why the plan was wrong:** The plan was authored before the metadata
was finalised in task 3 (commit `54b3d2491`). None of the liability keys
landed as `Both`-scope. The plan text was never reconciled.

**Fix applied (commit `<this commit>`):** Test swapped to `jury.quorum`
(Int, range 1-21, `ConfigScope::Both`) — the minimal correction that
exercises the moderator-write path against a real `Both`-scope key.
Float is unavailable today: no `Both`-scope key is declared
`ValueType::Float`.

**What this means for the completion report:**

> Plan §11 line 1101 named `liability.regular_multiplier` as the
> `Both`-scope key for the moderator-write test, but the metadata
> landed at task 3 declares all liability keys as
> `ConfigScope::Instance`. The test was corrected to `jury.quorum`.
> Future AD-c (and any other PRD that adds a community-tunable
> threshold) should cross-check against `CONFIG_KEY_METADATA` before
> authoring tests.

**RCA evidence:** `.claude/plans/twinkly-baking-wolf.md` (this session's
debug plan file).

**CI evidence of the failure:** GH Actions run
`24685261635` (`cargo-test-e2e` on `5ceffb52d`) — 28 pass / 1 fail.

**Local runtime evidence of the fix:**
`.claude/PRPs/debug/phase-v1-AD-b-rca-runtime.log` — 1 passed, 0 failed,
25.01s.
