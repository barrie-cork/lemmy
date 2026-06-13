---
name: pilot-resilience-test-blast-radius
description: Phase-7 resilience seed scripts (and any destructive/stateful pilot test) have side effects WIDER than the single case they exercise — three faces that no per-case verify catches — lane-boundary (testing script controls an infra-owned container), monitor-collision (a persistent health-monitor pages on a deliberate test-induced stop), and instance-scoped config as a global kill-switch (messaging_enabled=false pauses the WHOLE instance). Coordinate all three in the cross-session shared-state file; none are code bugs.
type: feedback
---

# Pilot resilience tests have blast radius beyond their own case

Surfaced reviewing the testing lane's `scripts/brehon/pilot-seed/seed-resilience.sh`
(phase 7) on 2026-06-13. A destructive/stateful pilot test scenario almost never
fails because "the code is broken" — it fails because its side effects reach
WIDER than the single case it means to exercise, and a per-case verify can't see
that. Three concrete faces, all coordination gaps, none code bugs:

## 1. Lane-boundary — testing-lane script controlling an infra-owned container

A testing-lane script that does `docker stop/start brehon-bridge` (or
`docker compose restart bridge`) is mutating an **infra-lane-owned** resource. Per
`pilot-internal-SHARED-STATE.md` §6 hazards, the bridge + Tuwunel containers and
the lemmy container ENV are infra-owned. This is **not a hard refusal** — phase-7
resilience testing legitimately needs to stop the bridge; stopping it IS the test.
But it MUST be coordinated in shared state first, not run unilaterally. The review
move: flag it, don't edit the other lane's script to remove the stop (that would
defeat the test); ask them to coordinate timing.

## 2. Monitor-collision — a passive health-monitor fights a destructive test

When a persistent container-health Monitor (e.g. a pilot watch polling `docker ps`
every 90s) coexists with a resilience test that intentionally stops a container,
the monitor's `CONTAINER not-Up` alert is **indistinguishable from a real crash**.
The monitoring session may "investigate" or restart the container out from under
the running test. The two activities actively fight each other absent a handshake:

- The test driver MUST announce the deliberate stop in shared state FIRST
  ("running resilience sub-case 3 now, bridge-down intentional, ~30s").
- The monitoring session MUST check for that announcement before treating a
  down-container as an incident, and mute/expect the alert for the duration.

Generalises beyond pilot: any time monitoring and destructive resilience-testing
share a target, you need this announce-then-mute handshake.

## 3. Instance-scoped config is a global kill-switch, not per-case

`governance_messaging_config` rows with `scope='instance'` (e.g.
`messaging_enabled`) apply to the **whole instance**. A soft-pause test that flips
`messaging_enabled=false` to confirm "no room provisions while paused" disables
Matrix propagation for **EVERY** case during the window — if any *other* case
transitions then, its room silently won't provision and the transition is LOST
(push-only hook, no replay). The blast radius is instance-wide though the test
only means to pause its own case. Keep the window short, run it when no other
governance traffic is in flight, and coordinate.

This generalises to ALL instance-scoped governance config (the panel-size cascade,
quorum/threshold fractions, reputation deltas all live at `scope='instance'` as of
2026-06-13 — none are seeded at `community:<id>` scope yet, but the config reader
supports community overrides, so a future per-community knob would narrow the
blast radius). See `feedback_pilot_governance_workflow_seeding_order.md` and the
config-aware seeding fix (`vote_to_threshold` reads the frozen per-case snapshots
precisely because config is not fixed).

## The unifying principle

Before running a stateful/destructive pilot test, ask **"what is the blast radius
beyond this one case?"**:

1. **Ownership** — which lane owns the resources I'm mutating? Coordinate before
   touching infra-owned containers/config from a testing lane.
2. **Watchers** — what passive monitors will react to my deliberate disruption?
   Announce it so they don't treat the intended state as an incident.
3. **Scope** — which config am I toggling, instance-global or case-local? An
   instance-scoped flip has instance-wide reach.

All three are resolved in the cross-session shared-state file
(`pilot-internal-SHARED-STATE.md` §5 Cross-session asks), not in code. The
resilience-script fixes that came out of this review were coordination notes plus
two cosmetic script fixes (dead baseline var, health-aware bridge-restart settle —
commit `72c407771`); the resilience logic itself was sound.

PMD: pattern memory 994.
