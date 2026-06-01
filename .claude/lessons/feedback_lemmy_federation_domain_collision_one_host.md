---
name: lemmy-federation-domain-collision-one-host
description: Lemmy keys federated instances by PORT-STRIPPED domain, so two instances on one host via host.docker.internal:PORT (different ports) collapse to the same domain and each treats the other as itself. Bidirectional federation testing needs distinct container hostnames (upstream docker/federation pattern), not same-host-different-port. Confirmed 2026-06-02 Phase 8.
metadata:
  type: feedback
---

Running two Lemmy instances on one host and pointing them at each other via
`host.docker.internal:<port>` **cannot achieve bidirectional federation**.
Lemmy keys every federated instance by its **port-stripped domain**, so two
instances that differ only by port are indistinguishable at the federation
layer — each computes the other's domain as its own and treats the remote
actor as local.

**The mechanism (code, not guesswork):**

- `Settings::get_hostname_without_port` (`crates/utils/src/settings/mod.rs:71-81`)
  splits on `:` and keeps only the first segment. The doc comment says it
  explicitly: *"removes the port and returns `lemmy-alpha` instead."*
- So `host.docker.internal:8536` and `host.docker.internal:8537` both reduce
  to domain `host.docker.internal`.
- Symptom: `resolve_object` for the remote actor fails **silently** with
  `resolve_object_failed` and **no fetch is logged** (even at
  `lemmy_apub=debug` / `activitypub_federation=debug`) — the instance
  short-circuits because it resolves the domain to its own local instance
  and finds no matching local user. The remote `instance` table shows only
  the instance's own domain.

**Two adjacent footguns surfaced while diagnosing this** (rule them out fast,
they are NOT the blocker):

1. **`tls_enabled` defaults to `true`** (`crates/utils/src/settings/structs.rs:39`,
   `#[default(true)]`). A dev instance behind a plain-HTTP nginx proxy must
   set `tls_enabled: false` in its hjson, or its `ap_id`s bake as `https://`
   and remotes fail to connect (no TLS listener). This is a real config bug,
   just not the federation-keying one.
2. **JWT `iss` is never validated** (`crates/api/api_utils/src/claims.rs:26-35`,
   `Validation::default()` checks only `sub` / signature / `exp` + the
   `login_token` table). A hostname change does NOT invalidate existing JWTs,
   and the `ap_id` data can be rewritten in-place with a surgical SQL UPDATE —
   **a full DB-volume wipe is never required** just to change the hostname.

**How to apply — bidirectional federation testing:** use the upstream
`docker/federation/` topology already in this repo: distinct **container
hostnames** (`lemmy-alpha:8541`, `lemmy-beta:8551`, `lemmy-gamma:8561`) on a
shared Docker network, each yielding a unique port-stripped domain. Do NOT try
same-host-different-port. For **outbound-only** verification (ADR-014: Brehon
sends AP, vanilla doesn't error on governance types) the one-host setup is
fine — outbound resolution works as long as the two domains differ (e.g.
`localhost` vs `host.docker.internal`), which is exactly why Phase 8.1/8.3-8.7
passed before anyone tried to "fix" the hostname.

**Related:** [[feedback_falsifiable_hypothesis_before_structural_fix]] (the
handover named `host.docker.internal:8536` as the BUG-15 fix; that premise was
a hypothesis — testing it against `get_hostname_without_port` revealed the
domain collision before committing the wrong change), `docker/federation/docker-compose.yml`,
`.claude/PRPs/reports/docker-smoke-test-plan.md` §"Phase 8 — BUG-15".
