# 15 — It's yours to tune (customisability as a core promise)

Founder point (2026-06-27): **a lot of this is customisable** — the system is a
neutral, configurable layer, and each community tunes it to fit how *they* want
to run. This has come up across so many points that it deserves to be one clear
promise rather than scattered footnotes.

## The core claim

> **Brehon Consensus doesn't impose one way of running a community. It's a layer
> your group tunes to fit — the rules, the stakes, how reputation works, how
> juries are drawn. It bends to your community, not the other way round.**

This is the practical face of "bottom-up" ([`14`](14-bottom-up.md)) and "consent"
([`10`](10-reputation-is-earned-by-helping.md)): self-governance isn't just
*choosing rules*, it's tuning the **whole machine**.

## What a community can tune (the knob list)

Everything founder has flagged as customisable, in one place:

| Knob | What it controls | Notes |
|---|---|---|
| **The rules** | what's allowed / what breaks them | the foundation; bottom-up, by consent ([`14`](14-bottom-up.md)) |
| **The stakes** | how much a violation costs your standing | community sets it, all consent ([`10`](10-reputation-is-earned-by-helping.md)) |
| **Reputation decay rate** | how fast standing fades over time | **tune it down to encourage engagement** — see below |
| **Reporter reward** | whether a successful reporter gains a little standing | controversial → off-by-default-able, upheld-reports-only ([`13`](13-the-moderation-journey.md)) |
| **Jury thresholds** | how much agreement a decision needs | *(verify live vs design)* |
| **Jury cooldown / rest** | how long after serving before you're called again | the fairness-of-the-draw dial ([`12`](12-everybody-moderates.md)) |
| **Minimum standing to be called / to vote** | the floor below which your role shrinks | *(verify live vs design)* |
| **Openness of entry** | how much to rely on vouching vs. time-based entry | the inclusivity valve ([`11`](11-invitation-as-defence.md)) |

## The decay example (founder's own)

Reputation **decays over time** — but *how fast* is tunable. A community that
**wants more people engaging** can **dial the decay down**, so members don't feel
they're constantly at risk of losing standing. The logic:

> Fast decay = "use it or lose it" pressure (can discourage casual members). Slow
> decay = a more welcoming, lower-anxiety community. **The group picks the balance
> that fits its culture.**

This is a good, concrete illustration of *why* tunability matters: the same
mechanic serves a high-trust activist cell and a relaxed gardening club
differently, because each sets it for itself.

## Why this is a selling point (not just a feature)

- **Answers "is this one-size-fits-all?"** → No. It's yours to shape.
- **It's the anti-SaaS pitch:** mainstream platforms make *you* fit *their*
  moderation model. This fits you. ([`05`](05-audience-grassroots.md) — what
  organisers want: "this is *ours*").
- **It's experimentation made safe:** tunable knobs + low stakes = a community can
  try a configuration, see how it feels, and adjust. (*Dawn of Everything*
  experimentation, [`03`](03-anti-tina.md) / [`14`](14-bottom-up.md).)

## Honesty guard-rail (load-bearing — read before any public copy)

"**Customisable**" is becoming a heavily-leaned-on promise. Before the pamphlet
says "your community can tune X," **verify against the repo which knobs are
actually config-driven in v0 vs. designed-for-later.** The docs confirm the
*principle* is real (e.g. the sponsor-liability floor is config-driven; values
are documented as "tuneable" with "per-community override"), but the *full knob
list above is not all confirmed live.* Frame unconfirmed knobs as "designed to be
tunable," not "is tunable," until checked. A wrong customisation claim is an easy
credibility loss.
