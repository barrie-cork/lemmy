---
name: Run the exact failing test locally before push — 25s vs 4min CI round-trip
description: When RCA fixes a specific CI test, local runtime verification of that one test is cheap insurance against a second false-green push
type: feedback
originSessionId: e8d77f73-4996-415b-810b-3f0a24b2ddf1
---
When fixing a CI test failure, run the exact failing test locally with
Docker before push — even if L1 (cargo check) and L4 (cargo test --no-run)
are green. The L4 gate only proves the test compiles; it doesn't prove
the assertion holds. For a single targeted test, local runtime is
~25 seconds vs. ~4 minutes for a CI round-trip; the math is always in
favour of running it locally once.

**Why:** On PR #76's `admin_set_config_community_scope_by_moderator` RCA,
L1 + L4 both exit 0 after the `jury.quorum` key swap — but those pass
without running a single assertion. Without the user's explicit
"run the failing test locally before push" instruction, a mis-keyed
swap (e.g. wrong range, wrong type) could have shipped and taken another
CI cycle to catch. The 25s local runtime matched CI behaviour
(29 passed; 0 failed; 3 ignored) and gave the confidence to push once.

**How to apply:** After any RCA fix whose scope is a specific named test
(or small set of named tests), run:

```
cmd //c "scripts\\brehon\\cargo-test.bat --test e2e -p lemmy_server \
  <test_name> -- --test-threads=1 > .claude/PRPs/debug/<slug>.log 2>&1"
```

before `git push`. Docker-per-test spins in 20-30s for simple tests. For
RCA fixes, this is cheap insurance. For broader feature commits
(many tests), the full e2e locally is too expensive and CI is the right
gate. Rule of thumb: if you can name the exact test that was red, run
it locally.

Pattern tie-in: Complements `feedback_test_target_compile_validation.md`
(L4 compile gate); this adds the runtime gate for targeted RCA scenarios.
Also ties to `feedback_cold_build_gate_layered_agents.md` — both are
about catching things CI would catch later for cheaper locally.
