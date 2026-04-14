# AGPLv3 Source Disclosure — Brehon Fork

This project is a **fork of [LemmyNet/lemmy](https://github.com/LemmyNet/lemmy)**, which is licensed under the [GNU Affero General Public License version 3](./LICENSE) (AGPL-3.0).

## Fork relationship

- **Upstream project:** Lemmy — `https://github.com/LemmyNet/lemmy`
- **Upstream pinned commit (initial fork point):** `811d0d09c66599a46b60e52c0a77d40728392c26`
- **`git describe` of that commit:** `1.0.0-alpha.12-167-g811d0d09c` (Lemmy 1.0-beta line)
- **Fork repository:** [barrie-cork/lemmy](https://github.com/barrie-cork/lemmy)
- **Working branch:** `governance-v0` (branched from `upstream/main` at the pinned SHA above)
- **Fork purpose:** adds governance, jury, reputation, and tamper-evident log primitives per the Brehon v0 design docs.
- **License:** inherited AGPL-3.0 per [ADR-011](../homeserver/docs/research/brehon-law-inspired-network/99-decisions-and-open-questions.md). See the full license text in [`LICENSE`](./LICENSE).

## What AGPLv3 requires

The AGPL is a strong copyleft license with a **network-use source-disclosure clause** (§13). If you, or anyone else, runs a modified version of this software to provide a service over a network — including an internal intranet or a federated fediverse instance — you **must** offer all users interacting with that service over the network the Corresponding Source of the running version, under the same AGPL-3.0 terms, free of charge.

In practice, any operator of a Brehon-fork-derived instance MUST:

1. Publish (or make available on request) the complete source code of the exact version they are running, including all governance-related modifications.
2. Preserve the copyright notices and this notice in any redistribution.
3. License their own modifications under AGPL-3.0 or a compatible license.
4. Link, from the running service UI or API response headers, to the repository where the source is published.

Operators running an unmodified release of this fork can satisfy (1) by pointing users to this GitHub repository at the specific commit or tag they are running. Operators who patch the fork further must publish their patches.

## Weekly upstream rebase log

Per [IMPLEMENTATION-PLAN-v0.md §7.1](../homeserver/docs/research/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md) top-risk mitigations, we rebase `governance-v0` onto `upstream/main` weekly to pick up Lemmy 1.0-beta fixes. Each rebase records the new upstream SHA here so we can diff governance-touching changes across syncs.

| Date (YYYY-MM-DD) | Upstream SHA | `git describe` | Notes |
|---|---|---|---|
| 2026-04-14 | `811d0d09c66599a46b60e52c0a77d40728392c26` | `1.0.0-alpha.12-167-g811d0d09c` | Initial fork point — pre-implementation baseline |

## Questions

Questions about the fork's license obligations should be directed to the maintainer of this repository. Questions about the upstream Lemmy project should be directed to [LemmyNet/lemmy](https://github.com/LemmyNet/lemmy).
