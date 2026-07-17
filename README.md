<div align="center">

[![License](https://img.shields.io/github/license/barrie-cork/lemmy.svg)](LICENSE)

<h1>Brehon Consensus</h1>

<p>Community moderation by jury, not by admin — reputation-backed governance for civil society, online.</p>

</div>

## The problem this solves

Most online communities put moderation power in the hands of a small admin/mod team. That's a single point of failure: admins burn out, go rogue, or become a target for capture. It doesn't scale to groups that want to actually *self-govern* rather than be governed.

Most of all, Brehon Consensus is a place for people to associate and organise together while experimenting with different ways of self-governance — in the spirit of [*The Dawn of Everything*](https://en.wikipedia.org/wiki/The_Dawn_of_Everything), in which an archaeologist and an anthropologist summarised contemporary research to show that human civilisation has always been made up of many different ways of structuring society. The idea that There Is No Alternative is nonsense. The Brehon law tradition itself — one of those alternatives, and this project's namesake — is covered in depth by [Brehon Academy](https://www.youtube.com/@BrehonAcademy).

This is built for any group of people who organise together and need a fair, low-drama way to handle disputes without appointing a permanent boss: a **neighbourhood or residents' association**, a **local gardening or allotment group**, a **community advocacy or campaign group**, an **academic working group or research collective**, a **mutual-aid network**, a **club or co-op** — civil society generally, wherever people self-organise online. These are groups run by volunteers, with no HR department and no appetite for one person holding all the moderation power.

Brehon Consensus replaces admin-led moderation with a **procedure**: members report bad behaviour, a randomly-selected jury of peers reviews the case and votes, the decision (and the rule it's based on) is published to a public log, and sanctions are graduated — a warning before a restriction, a restriction before exclusion. No single person has unilateral moderation power, and every decision is auditable after the fact.

> **Trust is not given by the system — it is extended by other people, and risk is shared.**

It's a working fork of [Lemmy](https://github.com/LemmyNet/lemmy) (the federated, Reddit-style discussion platform) with this governance layer built in as new Rust modules alongside the existing codebase — not a separate app bolted on top. Each group runs its own instance (like a private forum), and instances can optionally federate with each other.

**Current stage:** solo-developer research build. Governance logic runs and is tested end-to-end, and an internal pilot is in progress — it hasn't yet run a real community. See [Roadmap](#roadmap) and [Status](#status).

## How this fits with other open-source platforms

Brehon Consensus is a **governance layer, not another platform competing for your community**. The open-source world already has good places to talk — Reddit-style forums, chat, video, microblogging. What none of them ship is a fair way to *run* the group that uses them: membership, rules, and dispute-handling that don't hang off one admin. That's the layer this project builds, and it's designed to travel across open platforms the way email travels between providers — you shouldn't need everyone on the same software to play fair together.

Concretely, the layer needs three things from a host platform: readable content (to gather evidence), a server-side surface to act through (to apply decisions), and stable member identities. Any open, self-hostable platform with those is a candidate host — and there are a lot to choose from.

| Status | Platform | What runs there |
| --- | --- | --- |
| **Native** | Lemmy (ActivityPub, Reddit-style) | The full governance layer, in-process — this repo. |
| **Live** | Matrix (Tuwunel homeserver + application-service bridge) | Jury deliberation rooms, appeal and emergency rooms — provisioned automatically from case state — plus 1:1 direct messages and town halls. Encrypted rooms are out of scope: governance needs readable evidence. |
| **Designed, not built** | PeerTube (ActivityPub, YouTube-style) | The planned next host — same federation model as Lemmy. |
| **Out of scope** | Serverless / end-to-end-encrypted P2P apps | No server-side evidence or enforcement surface to plug into. A boundary of the model, not a roadmap gap. |

The same three criteria fit much of the open web — federated forums such as Discourse and NodeBB, ActivityPub microblogging, label-based moderation on Bluesky's AT Protocol. Those are directions, not current claims: today, Lemmy and Matrix are what's wired up.

## Also relevant to DAOs

The same jury-and-reputation model applies to online-native groups too, including DAOs. DAOs are good at on-chain treasury and voting (tools like [Realms](https://realms.today) or [Squads](https://squads.so) already do that well on Solana). They're less good at **day-to-day social moderation** — deciding whether a specific post, comment, or member crossed a line, and doing it in a way the community trusts wasn't just "an admin's call." That's the gap Brehon Consensus targets:

- It's a **social/moderation governance layer**, not a treasury or token-voting mechanism — it complements on-chain tooling rather than competing with it.
- **No token exists and none is planned.** Voting weight comes from earned, decaying, multi-dimensional reputation — not from holding or staking anything.
- **No blockchain component today.** Case decisions are hash-chained and signed locally (tamper-evident, publicly auditable log). A narrow, later-stage evidential use — publishing hashes of case decisions to a public ledger — is envisaged for a future release, deliberately deferred and never load-bearing for governance itself. Nothing is anchored on any chain, Solana included, in this build — see [What this is not](#what-this-is-not).
- If useful to a DAO, the natural fit is as the **moderation/reputation layer sitting alongside** its existing on-chain treasury and voting stack, not a replacement for either.

We're sharing this as-is for evaluation, not claiming a Solana integration that doesn't exist yet.

## How it works

The core loop: **report → jury assignment → vote → published decision → reputation update**, with no direct admin action required on the golden path.

- **Sponsorship-gated onboarding** — new members need two existing members to vouch for them (or age in through a slower fallback path) before they can post, report, or vouch for others themselves. This is the anti-sybil / anti-brigading mechanism.
- **Jury panels, not permanent mods** — cases are decided by a small panel randomly drawn from eligible members, who vote and move on. No one holds standing moderator power.
- **Reputation held at risk** — members (and the sponsors who vouched for them) have reputation across four dimensions that can go down as well as up. Sponsoring someone who behaves badly costs you something, proportionally — which is what keeps vouching honest.
- **Graduated, restorative sanctions** — label → reduced visibility → temporary restriction → jury case → community exclusion → federation-wide signal. Exclusion is a last resort, not the default response, and reintegration paths exist below the top tier.
- **Deliberation happens in real rooms** — jurors deliberate in a private Matrix room provisioned automatically when their case needs one (appeals and emergencies get their own), and communities can hold town-hall meetings with chair-run mic-passing. Room lifecycle events go on the audit log; what's said in the room never does.
- **Public, auditable log** — every case decision is recorded in a tamper-evident, cryptographically signed log and published (redacted where needed) so outcomes can be checked by members and outside observers alike.
- **Federated** — built on ActivityPub, so instances running Brehon Consensus can share governance signals with each other while still talking normally to any regular Lemmy instance.

For the technical detail — schema, API surface, jury algorithm — see [docs/brehon-law-inspired-network/](docs/brehon-law-inspired-network/00-README.md).

## Where the model comes from

The mechanics are adapted from early Irish Brehon law, a system that maintained social order without police, prisons, or a central enforcer — using reputation, sponsorship, and community judgment instead.

| Brehon principle | Platform mechanic |
| --- | --- |
| Honor price — status you could lose | Reputation as stake at risk; higher trust means bigger consequences for bad-faith action |
| Sureties — guarantors backed your standing | Sponsorship: new members join via 2 sponsors, who share reputation risk if the sponsee misbehaves |
| Kin groups — layered loyalties | Moderation starts local (community) before escalating outward |
| Brehons — experts who interpret law, not rulers who enforce it | Randomly-selected jury panels, not a permanent moderator class |
| Restitution over punishment | Graduated, restorative sanctions before exclusion |
| Public, known laws | Every case cites the rule applied; decisions are logged publicly |
| Collective enforcement | Soft-enforcement ladder: label → reduced reach → restriction → exclusion |
| Inter-tribal law (túatha) | Federated instances recognise each other's governance signals, with local override |
| Status loss as deterrent | Visible trust signals, not hidden scores |

Full mapping and rationale: [docs/brehon-law-inspired-network/01-vision-and-principles.md](docs/brehon-law-inspired-network/01-vision-and-principles.md).

## Roadmap

Work runs in milestones on `governance-v0`. The order is committed; dates are not.

| Milestone | Scope | Status |
| --- | --- | --- |
| **Governance core** | The 11 `/api/v4/governance/*` endpoints: reports, jury cases, votes, appeals, sponsorship, reputation, admin config, federation-inbound. | ✅ Shipped |
| **M1 — Chat infrastructure** | Matrix homeserver + application-service bridge, 1:1 direct messages with rich media, admin-configured community rooms. | ✅ Shipped |
| **M2 — Governance-triggered rooms** | Jury, appeal, and emergency rooms provisioned automatically from case-state changes; room lifecycle recorded on the tamper-evident log (room *content* is never hashed). | ✅ Shipped |
| **M3 — Town halls** | Stage-mode voice rooms: chair-controlled mic-passing, raised-hand queue, cross-instance emergency mute, optional recording as a governance artefact. | ✅ Shipped |
| **Internal pilot** | First sustained live use with real accounts on a real deployment; go/no-go gate before any external community runs it. | ⏳ In progress |

After the pilot, the direction is broader platform reach (PeerTube first — see [How this fits with other open-source platforms](#how-this-fits-with-other-open-source-platforms)) and a member-facing rulemaking surface: members proposing rule changes and the community consenting to them, rather than admins editing config. Neither is scheduled yet; design detail lands in [docs/brehon-law-inspired-network/](docs/brehon-law-inspired-network/00-README.md) as it firms up.

## What this is not

- **Not token-based governance.** No on-chain voting, no "more tokens = more power", no token at all.
- **Not on-chain today.** The governance log is a local, cryptographically signed hash chain — tamper-evident and auditable, but not anchored to any blockchain in this build. A narrow, later-stage use is envisaged (publishing hashes of case decisions and rule changes to a public ledger as an evidential witness, not as judge or database) — deliberately deferred, and never the basis of everyday operation. See [docs/brehon-law-inspired-network/Brehn-Consensus-two-part-explainer.md](docs/brehon-law-inspired-network/Brehn-Consensus-two-part-explainer.md) §6-8.
- **Not a wholesale replacement for Lemmy moderation.** Existing admin/mod pathways still work during the transition — this adds a jury layer, it doesn't rip out the old one on day one.
- **Not a single reputation score.** Reputation is multi-dimensional and exposed as capabilities (can I sponsor, can I serve on a jury), not a leaderboard number.
- **Not permanent-ban-by-default.** Reintegration paths exist below the highest sanction tier.

## For engineers: stack

- **Base:** Lemmy 1.0-beta (Rust, Actix, Diesel, ActivityPub federation)
- **Governance hooks:** Extism plugin host
- **Auth:** Lemmy's existing JWT, optional passkey MFA (`webauthn-rs`)
- **Authorization:** hardcoded capability checks reading a `reputation_snapshot` — no external policy engine in v0
- **Governance log:** `sha2` hash chain via Postgres triggers, `rs_merkle` + `ed25519-dalek` signing
- **Messaging & RTC:** Matrix (Tuwunel homeserver) via an application-service bridge daemon (`services/bridge/`, workspace-excluded so the core build pulls zero Matrix deps); town-hall voice via LiveKit + Element Call
- **Privacy:** pseudonymised actor identities from day one (GDPR-aware by design)
- **License:** AGPL-3.0 (inherited from Lemmy)

v0 is deliberately a lean, solo-dev stack — no Keycloak, no OpenFGA, no Vault, no Kubernetes, no external log signer, no blockchain anchoring. Those are future considerations, not missing v0 features. 11 governance endpoints under `/api/v4/governance/*` make up the whole v0 API surface — see [docs/brehon-law-inspired-network/05-mvp-and-delivery-plan.md](docs/brehon-law-inspired-network/05-mvp-and-delivery-plan.md) §2.

## Status

Active development on the `governance-v0` branch. Design docs, ADRs, and the delivery plan live under [docs/brehon-law-inspired-network/](docs/brehon-law-inspired-network/00-README.md) — start with `00-README.md` for the full documentation suite and reading paths by role.

This repo is not accepting external contributions or issue reports at this stage. If you're a community or DAO evaluating this for real use, open a conversation rather than an issue.

## License

AGPL-3.0, inherited from upstream Lemmy — see [LICENSE](LICENSE). Every release honours the source-disclosure notice.

## Credits

Built on [Lemmy](https://github.com/LemmyNet/lemmy) by the LemmyNet team. Brehon law framing draws on historical scholarship on early Irish legal tradition (see `docs/brehon-law-inspired-network/` for sources), on [*The Dawn of Everything*](https://en.wikipedia.org/wiki/The_Dawn_of_Everything) by David Graeber and David Wengrow, and on [Brehon Academy](https://www.youtube.com/@BrehonAcademy).
