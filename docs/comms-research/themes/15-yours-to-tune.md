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

Status verified against config.rs 2026-06-27. **LIVE** = a config row a community
can override today; **design** = built but off/unwired in v0.

| Knob | What it controls | Status |
|---|---|---|
| **The rules** | what's allowed / what breaks them | LIVE — the foundation; bottom-up, by consent ([`14`](14-bottom-up.md)) |
| **The stakes** | how much a violation costs your standing | LIVE — `DEFAULT_DELTAS_*` config rows, per-community ([`10`](10-reputation-is-earned-by-helping.md)) |
| **Reputation decay rate** | how fast standing fades over time | **design, default-OFF** — `feature.reputation_v1_decay_enabled`=false. Frame as "switch on standing that fades", not "fades" — see below |
| **Reporter reward** | whether a successful reporter gains a little standing | LIVE — `reporter_upheld` +10 / dismissed −5; upheld-only ([`13`](13-the-moderation-journey.md)) |
| **Jury thresholds** | how much agreement a decision needs | LIVE — `jury.quorum` (3 of panel 5) |
| **Jury cooldown / rest** | how long after serving before you're called again | LIVE — `jury.constraints.juror_cooldown_days` (default 7) ([`12`](12-everybody-moderates.md)) |
| **Minimum standing to be called / to vote** | the floor below which your role shrinks | LIVE — `DEFAULT_THRESHOLDS_*` (jury-reliability 50, etc.) |
| **Openness of entry** | vouching vs. time-based / provisional entry | LIVE — `MembershipState` (member/provisional), `onboarding.*` account-age rows ([`11`](11-invitation-as-defence.md)) |

## The decay example (founder's own)

Reputation **can be set to decay over time** — the mechanic is built, though it
ships **off by default** in v0 (so frame it as "you can switch on standing that
fades", not "standing fades"). Once on, *how fast* is tunable. A community that
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

## Honesty guard-rail (load-bearing) — RESOLVED 2026-06-27

Verified against `config.rs`: **the knob list above is real.** Seven of the eight
knobs are LIVE config rows a community can override today (rules, stakes, reporter
reward, jury thresholds, jury cooldown, minimum-standing floors, entry openness via
membership-state + account-age). So "customisable" is a *safe present-tense*
promise for those.

**The single exception:** reputation **decay** is built but **off by default**
(`feature.reputation_v1_decay_enabled`=false). Say "you can switch on standing
that fades," never "standing fades over time." That one line is the difference
between an honest promise and an overclaim.
