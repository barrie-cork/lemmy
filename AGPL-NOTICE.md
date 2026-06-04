# AGPLv3 Source Disclosure — Brehon Fork

This project is a **fork of [LemmyNet/lemmy](https://github.com/LemmyNet/lemmy)**, which is licensed under the [GNU Affero General Public License version 3](./LICENSE) (AGPL-3.0).

## Fork relationship

- **Upstream project:** Lemmy — `https://github.com/LemmyNet/lemmy`
- **Upstream pinned commit (initial fork point):** `811d0d09c66599a46b60e52c0a77d40728392c26`
- **`git describe` of that commit:** `1.0.0-alpha.12-167-g811d0d09c` (Lemmy 1.0-beta line)
- **Fork repository:** [barrie-cork/lemmy](https://github.com/barrie-cork/lemmy)
- **Working branch:** `governance-v0` (branched from `upstream/main` at the pinned SHA above)
- **Fork purpose:** adds governance, jury, reputation, and tamper-evident log primitives per the Brehon v0 design docs.
- **License:** inherited AGPL-3.0 per [ADR-011](docs/brehon-law-inspired-network/99-decisions-and-open-questions.md). See the full license text in [`LICENSE`](./LICENSE).

## What AGPLv3 requires

The AGPL is a strong copyleft license with a **network-use source-disclosure clause** (§13). If you, or anyone else, runs a modified version of this software to provide a service over a network — including an internal intranet or a federated fediverse instance — you **must** offer all users interacting with that service over the network the Corresponding Source of the running version, under the same AGPL-3.0 terms, free of charge.

In practice, any operator of a Brehon-fork-derived instance MUST:

1. Publish (or make available on request) the complete source code of the exact version they are running, including all governance-related modifications.
2. Preserve the copyright notices and this notice in any redistribution.
3. License their own modifications under AGPL-3.0 or a compatible license.
4. Link, from the running service UI or API response headers, to the repository where the source is published.

Operators running an unmodified release of this fork can satisfy (1) by pointing users to this GitHub repository at the specific commit or tag they are running. Operators who patch the fork further must publish their patches.

## Additional components — bridge daemon + Tuwunel homeserver

The M1 release introduces two additional components deployed alongside the Lemmy fork:

### `services/bridge/` — Brehon Matrix AS bridge

- **What it is:** a Matrix Application Service bridge daemon written in Rust (`crates`: `axum`, `matrix-sdk`, `ruma-appservice-api`).
- **License:** this component is original Brehon code, authored under AGPL-3.0 (the same license as this repository). Source is in `services/bridge/` in this repository.
- **AGPL §13 applicability:** the bridge daemon provides a network service (Matrix AS endpoint) and is distributed and deployed as part of the Brehon platform. Operators running this component are subject to the same AGPL §13 source-disclosure obligations as the Lemmy fork. Corresponding source for the bridge daemon is satisfied by publishing this repository at the running commit.

### Tuwunel (Matrix Conduit homeserver) — docker-compose dependency

- **What it is:** a Matrix homeserver (Tuwunel, a fork of [matrix-conduit/conduit](https://github.com/matrix-org/conduit)) used in the bridge docker-compose stack.
- **License:** Tuwunel / Conduit is licensed under the Apache License 2.0. Brehon does not modify the Tuwunel source; it is used as a pinned Docker image (`matrixconduit/matrix-conduit:v0.6.0`).
- **AGPL §13 applicability:** Tuwunel is Apache-2.0 licensed, not AGPL. However, operators deploying the Brehon bridge stack (which includes Tuwunel via docker-compose) should be aware that Tuwunel's own license terms govern the homeserver component. The Tuwunel/Conduit source is available at `https://github.com/girlbossceo/conduit` (Tuwunel fork) or `https://github.com/matrix-org/conduit` (upstream Conduit).
- **No Tuwunel source modifications:** Brehon does not patch the Tuwunel image. The `registration.yaml` and docker-compose configuration files in `services/bridge/` are original Brehon configuration, not Tuwunel source modifications.

## Weekly upstream rebase log

Per [IMPLEMENTATION-PLAN-v0.md §7.1](docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md) top-risk mitigations, we rebase `governance-v0` onto `upstream/main` weekly to pick up Lemmy 1.0-beta fixes. Each rebase records the new upstream SHA here so we can diff governance-touching changes across syncs.

| Date (YYYY-MM-DD) | Upstream SHA | `git describe` | Notes |
|---|---|---|---|
| 2026-04-14 | `811d0d09c66599a46b60e52c0a77d40728392c26` | `1.0.0-alpha.12-167-g811d0d09c` | Initial fork point — pre-implementation baseline |
| 2026-04-18 | `d1975776a491bc2f4085139bb50cbf89fe6d5e54` | `1.0.0-alpha.12-175-gd1975776a` | First weekly rebase (Phase 5b → 5c gap). 8 upstream commits absorbed: Rust toolchain 1.94 → 1.95, activitypub_federation 0.7.0-beta.9 → beta.10, DB pool timeout increase (#6448), federated post-removal fix (#6442), webfinger IP check (#6445), ARM build fixes. Zero governance-path overlap. All three validation gates (check / clippy / e2e compile) green. Conflict: upstream removed `cfg-if = "1"` from workspace deps (`cfg_if!` macro replaced by stdlib `cfg_select!` in Rust 1.95); fork drop accepted since no first-party code calls `cfg_if!`. |

## Questions

Questions about the fork's license obligations should be directed to the maintainer of this repository. Questions about the upstream Lemmy project should be directed to [LemmyNet/lemmy](https://github.com/LemmyNet/lemmy).
