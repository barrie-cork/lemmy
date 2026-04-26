---
name: Cross-check plan test assertions against landed metadata
description: Tests authored from the plan can drift from what metadata tables actually declare — grep before trusting plan row text
type: feedback
originSessionId: e8d77f73-4996-415b-810b-3f0a24b2ddf1
---
When a plan enumerates integration tests against compile-time metadata
(CONFIG_KEY_METADATA, CASE_TARGET_TYPES, any static registry), cross-check
the plan's named keys against the implementation BEFORE writing the test.
The plan is written at plan-authoring time; metadata is finalised during
implementation. They can drift silently.

**Why:** v1-AD-b PR #76 shipped with `admin_set_config_community_scope_by_
moderator` targeting `liability.regular_multiplier` per plan §11 line 1101.
That key is `ConfigScope::Instance`. The test expected `Both`-scope
behaviour; `check_policy` correctly rejected with `ScopeMismatchInstanceKey`
and the test failed in CI with `NotAnAdmin`. Fix was a 4-line key swap to
`jury.quorum`. Cost: one CI round-trip + one RCA cycle that could have
been a `rg 'scope: ConfigScope::Both' crates/` grep before writing the test.

**How to apply:** Any time a plan row references a specific metadata key,
run a one-command grep against the corresponding `*_METADATA` / const
table in the implementation before writing the assertion. If the plan
names a specific enum value, confirm it's in the enum. If the plan
prescribes a specific scope or range, read the metadata row and verify.
Takes 30 seconds; saves a CI round-trip.

Load-bearing for any phase that tests a configurable metadata table —
v1-AD-c, v1-jury-mechanics, v1-reputation-tuning all have the same shape
and the same trap.
