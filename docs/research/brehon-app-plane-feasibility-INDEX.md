# Brehon app-plane feasibility — INDEX / DIGEST

> Navigation digest for `brehon-app-plane-feasibility-brief.md` (~169 lines, ~39KB).
> Read this first; open only the sections you need. Line numbers are for the SOURCE file.

## Overview

The report assesses which external platforms the Brehon Consensus governance backplane
can integrate with as "app planes" over its thin federation contract (B-fetch = read
evidence from a public API; B-publish = a small local subscriber that applies sanctions;
B-actor = portable per-user identity mapping). Each platform is rated against requirements
A–E and given an overall Strong/Partial/Blocked rating. Top-line conclusion: expand in the
order **PeerTube → Discourse, NodeBB, Bluesky labelers → ActivityPub microblogging (as
local-subscriber/advisory only) → team chat (with private-room limits) → long-tail Fediverse**;
keep Nostr and serverless E2EE P2P apps (Keet/Holepunch/Pear) out of hard enforcement. The
decisive constraint is B-publish: most platforms can expose public evidence, but enforcing a
sanction needs a documented bot/appservice/plugin/labeler surface that runs locally without
Brehon holding admin credentials.

## Section map

| Lines | Heading | What's in it |
|---|---|---|
| 1–9 | Executive summary | Top-line: best new candidates (Discourse, NodeBB, Bluesky); reuse AP + AT-Proto rails; Keet/Pear blocked. |
| 11–15 | Rating method | Definitions of Strong/Partial/Blocked + requirements A–E. |
| 17–38 | **Ranked feasibility table** | The 18-platform master table (rank, A–E columns, overall, one-line). Source of the condensed table below. |
| 40–82 | Strong candidates + integration designs | Per-platform B-fetch/publish/actor designs. Sub-headings below. |
| 42–46 | — Lemmy | Native in-process path + cross-instance subscriber model. |
| 48–54 | — Matrix | Appservice/bot; redact/kick/ban via power levels; E2EE is the B-fetch blocker. |
| 56–60 | — PeerTube | REST API + plugin subscriber; video/comment/account sanctions; confirmed next target. |
| 62–70 | — Bluesky / AT Protocol | Labeler service (not admin bot); DIDs; labels ≠ deletion; Ozone. |
| 72–76 | — Discourse | Local plugin or scoped API key; direct sanction mapping; no handed-over credentials. |
| 78–82 | — NodeBB | Read/Write API + plugin; plugin preferred over external write token. |
| 84–126 | Partial candidates + constrained designs | Per-family constraints + recommended limited designs. Sub-headings below. |
| 86–92 | — Mastodon/GoToSocial/Pleroma/Akkoma | Public fetch strong; admin actions need local authority; reuse AP Flag/Block. |
| 94–98 | — Mbin and PieFed | Mbin: validate per-release; PieFed: alpha API, later target. |
| 100–106 | — Mattermost/Rocket.Chat/Zulip | Private-by-default content; workspace-local moderation; locally installed app/plugin. |
| 108–112 | — Nostr | Public relay fetch + pubkey identity; B-publish only advisory (relays/clients decide). |
| 114–118 | — Friendica/Hubzilla/streams | AP/Zot actors; less-standardized B-publish; store both local ID + actor URL. |
| 120–126 | — Mobilizon/Funkwhale/Castopod | Event/media communities; Castopod REST API off by default (limited). |
| 128–134 | **Blocked: Keet/Holepunch/Pear** | Serverless E2EE P2P — no server evidence API, no subscriber plane. Why blocked. |
| 136–154 | Ride existing rails: what to reuse | AP Flag/Block; shared Fediverse safety infra; Bluesky labelers/Ozone. Sub-headings below. |
| 138–142 | — ActivityPub Flag/Block/Undo | Report/advisory rail only; remote Block "not guaranteed". |
| 144–148 | — Shared Fediverse safety infra | IFTAS/FediCheck, Fediseer, The Bad Space as advisory inputs. |
| 150–154 | — Bluesky labelers and Ozone | Reuse labeler model; `com.atproto.label.*` + `report.createReport`. |
| 156–164 | Implementation recommendations | Four prioritized recommendations (forums+labelers; AP partial-by-design; chat evidence policy; Nostr/P2P out). |
| 166–168 | Bottom line | The full expansion-order sentence + "don't become an identity provider / delete button". |

## Condensed feasibility table (from §"Ranked feasibility table", lines 17–38)

| # | Platform | Overall | Deciding factor (≤15 words) |
|---|---|---|---|
| 1 | Lemmy | Strong | Native typed client + built-in moderator workflows; in-process or local subscriber. |
| 2 | Matrix (Synapse/Dendrite/Conduit/Tuwunel) | Strong* | Appservice can redact/kick/ban; *E2EE & private rooms block B-fetch. |
| 3 | PeerTube | Strong | Public REST API + plugin system + video/comment moderation primitives. |
| 4 | Bluesky / AT Protocol | Strong** | Labelers + DIDs; **strong for labels, partial for hard takedown. |
| 5 | Discourse | Strong | Mature API + plugin + stable IDs; local plugin avoids handed credentials. |
| 6 | NodeBB | Strong | Read/Write API + plugin maps sanctions to local forum primitives. |
| 7 | Mastodon/GoToSocial/Pleroma/Akkoma | Partial | Silence/suspend need admin authority or local extension, not a bot. |
| 8 | Mbin | Partial | Lemmy-like, but validate permission boundaries per release. |
| 9 | PieFed | Partial | API still alpha; moving target, not first-wave. |
| 10 | Nostr relays/communities | Partial | Public fetch + pubkey IDs, but relays/clients decide enforcement. |
| 11 | Mattermost | Partial | Evidence often in private channels; destructive actions need local privileges. |
| 12 | Rocket.Chat | Partial | Apps-Engine works, but enforcement is workspace-local; many rooms private. |
| 13 | Zulip | Partial | Great APIs, but bot visibility + admin actions are permission-bounded. |
| 14 | Friendica/Hubzilla/streams | Partial | Federate + expose APIs, but B-publish surfaces less standardized. |
| 15 | Mobilizon | Partial | Public events fetchable; sanctions are event/group moderation only. |
| 16 | Funkwhale | Partial | Moderator tools exist; subscriber API/plugin coverage needs per-version validation. |
| 17 | Castopod | Partial | OSS + AP, but REST API disabled by default; mostly advisory. |
| 18 | Keet/Holepunch/Pear | Blocked | Serverless E2EE P2P — no public API plane, no subscriber enforcement plane. |

## Key recommendations

- **Next target = PeerTube** (confirmed Strong); it's the planned next app plane after the native Lemmy/Matrix set.
- **Then prioritize Discourse, NodeBB, and Bluesky labelers** — each has a documented, non-invasive local subscriber model and clear user/content identity. (§Implementation recs, lines 156–158.)
- **Implement ActivityPub microblogging as "Partial by design"**: Brehon emits report/advisory/sanction events; each Mastodon/GoToSocial/Pleroma/Akkoma instance runs a local subscriber that decides how to apply them. Do NOT treat as hard-enforcement. (Lines 160.)
- **For chat (Matrix/Mattermost/Rocket.Chat/Zulip): standardize an evidence-visibility policy first.** Matrix strong only in non-E2EE governed rooms; the rest are workspace-private/permission-bounded. (Lines 162.)
- **Keep Nostr and P2P out of the hard-enforcement roadmap** — Nostr can publish signed advisories (NIP-56/NIP-72) but relays/clients enforce; Keet/Pear stay Blocked. (Lines 164.)
- **Reuse ActivityPub `Flag` for reports/advisories** and `Block`/`Undo` semantics — but treat AP as a report/advisory rail, not a remote-control plane (remote `Block` "not guaranteed"). (Lines 138–142.)
- **Reuse Bluesky's labeler model + Ozone** for AT Protocol — a Brehon labeler, not a custom account-suspension system. Endpoints: `com.atproto.label.subscribeLabels`/`queryLabels`, `com.atproto.report.createReport`. (Lines 150–154.)
- **Interoperate with shared Fediverse safety infra** (IFTAS/FediCheck, Fediseer, The Bad Space) as advisory inputs/outputs, not automatic sanctions. (Lines 144–148.)

## Where to look for X

- **"How do I integrate Mastodon (and the AP microblog family)?"** → lines 86–92 (constrained design) + recommendation at line 160.
- **"How do I integrate PeerTube / what's the next target?"** → lines 56–60.
- **"How do Bluesky labels / Ozone work for sanctions?"** → lines 62–70 (design) + 150–154 (rails/endpoints).
- **"Which ActivityPub activities matter (Flag/Block/Undo)?"** → lines 138–142.
- **"Why is Keet/Holepunch/Pear blocked?"** → lines 9 (summary), 38 (table row), 128–134 (full rationale).
- **"How do I integrate a forum (Discourse / NodeBB)?"** → lines 72–76 (Discourse), 78–82 (NodeBB).
- **"What about team chat (Matrix/Mattermost/Rocket.Chat/Zulip) and the E2EE/private-room problem?"** → 48–54 (Matrix), 100–106 (chat trio), 162 (policy rec).
- **"What shared safety services should Brehon consume (IFTAS/Fediseer)?"** → lines 144–148.
- **"What do Strong/Partial/Blocked and requirements A–E mean?"** → lines 11–15.

## Coverage notes

- The report's own master table (lines 17–38) is the authority for ratings; this index condenses it but does not add platforms.
- Not covered in the report: concrete schema/wire-format for B-fetch/publish/B-actor, code, ADR cross-references, or effort estimates — it is a platform-feasibility survey only.
- Every external claim in the source is footnoted with a doc URL; this index drops the URLs to stay compact — follow the cited line range in the source for links.
