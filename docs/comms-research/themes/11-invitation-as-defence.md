# 11 — Invitation + surety as a defence against bad actors

Founder point (2026-06-27): the invitation/vouching model isn't only about
building trust — it's a **defence**. Because the person who invites you **stakes
their own reputation on you**, getting in is costly and traceable. That single
mechanic raises the wall against a whole class of threats organisers know and
fear.

## The core claim

> **You get in because someone vouches for you — and they put their own standing
> on the line to do it.** That makes a community very hard to flood, infiltrate,
> or astroturf. Bad actors can't just sign up by the thousand; every member is
> someone a real, accountable member chose to back.

## The threats it defends against (name them — organisers feel these)

This is the part to make concrete. These aren't abstract "bad actors" — they're
specific, recognisable menaces:

| Threat | What it looks like | Why surety stops it |
|---|---|---|
| **Bots / AI spam** | Mass fake accounts, automated posting, scams, "buy now" floods, AI-generated noise | Bots have no one to vouch for them, and no real member will stake reputation on a thousand throwaways |
| **Agents provocateurs** | People with an agenda — sometimes *paid* — who join to inflame, divide, derail, and discredit a group from inside | Infiltration leaves a trail: someone vouched for them, and that voucher's standing is on the line |
| **Brigades / pile-ons** | Coordinated outside mobs arriving to swarm a target | They can't get *in* at scale; outsiders have no vouching path |
| **Sock-puppets / sybils** | One person running many fake identities to fake consensus or stack a vote | Each identity needs a separate, reputation-staking sponsor — expensive and exposing |
| **Disrespectful drive-by users** | People who don't care about the group's rules and won't stick around | Vouchers won't risk their standing on someone who'll embarrass them |

## Why it works — the mechanism in one breath

**Trust is relational and risk is shared.** Your inviter isn't just saying "I
know them" — they're saying "I'll take a hit to my own standing if they behave
badly." That:

1. **Raises the cost of entry for bad actors** (you need a real, willing,
   reputation-bearing sponsor — money can't manufacture that at scale).
2. **Makes infiltration traceable** (every member is connected to who vouched for
   them; bad behaviour ripples back up the chain).
3. **Aligns incentives** (members self-select who they bring in, because they
   share the consequences).

This is the repo's principle 2 (sureties) and principle 4 (bad actions propagate
through relationships) — see
[`../source-notes/repo-essence-extract.md`](../source-notes/repo-essence-extract.md).

## Why this is a *strong* current selling point — the exhaustion is the demand

Founder (2026-06-27): **these are all major issues with social media platforms
today, and people are sick of it — they want an alternative.**

That's the strategic core of this whole pitch. Bots, AI spam, astroturfing, paid
provocateurs, coordinated inauthentic behaviour, and pile-ons aren't fringe
worries — they're **the defining, daily experience of mainstream social media in
2026.** People are *exhausted*. The demand isn't hypothetical curiosity about a
"better platform"; it's active, felt fatigue and a wish to get *out*.

This connects the defence directly to the **anti-TINA spine**
([`03-anti-tina.md`](03-anti-tina.md)): people don't need to be *convinced* there's
a problem — they're living it. The job of the copy is not to argue the problem
exists; it's to show that **a real, usable alternative is here**, and to make the
relief tangible. Meet exhaustion with relief, not with a lecture.

Most platforms "defend" against these threats with opaque, top-down "trust &
safety" algorithms the community can't see or control — which is part of *why*
people distrust them. Brehon Consensus defends with a **human, visible,
community-owned mechanism**: who's willing to vouch for you. That's a more
trustworthy answer *and* a more democratic one.

> Framing: **"You're not imagining it — and you don't have to put up with it."**

> One-liner: **"Bots can't get a reference. Provocateurs can't hide who let them
> in."**

## Balance — the inclusivity tension (handle honestly)

Invitation-only can read as **exclusionary / gatekeeping / a clique** — the exact
opposite of "inclusive," one of our six core words. Always pair the defence with
the openness valve:

- **There's a time-based way in too** — you don't *need* a sponsor; you can also
  earn your way in over time with clean conduct. ✅ Live in v0 (verified
  2026-06-27): a `MembershipState` of `provisional` plus account-age thresholds
  (`onboarding.sponsor_min_account_age_days` 30, `provisional_membership_cooldown_days`
  14) is the shipped time-based path — not just design.
- **It's not about keeping people out** — it's about keeping *bad-faith floods*
  out, so genuine newcomers land in a community that isn't already on fire.
- **The community sets its own openness** — a group can be more or less open
  depending on its own rules (ties to consent / community-set-stakes, theme 10).

Never sell the gate without the valve. The honest frame: **welcoming to people,
hostile to floods.**

## Where it goes in the copy

- A natural **fourth "sharp hook"** alongside cancel-culture and
  earned-by-helping ([`00-README.md`](../00-README.md) sharp-hooks section).
- Pairs with the **job-reference analogy** ([`04-analogies.md`](04-analogies.md) B)
  — the analogy already carries it; this file gives it the *security* payload.
- Strong **FAQ** material: "Won't bots/trolls just take over?" → "Much harder
  here — here's why." (added to [`06-message-ladder.md`](06-message-ladder.md)).

## Honesty guard-rail

Don't claim it's *impossible* to abuse — claim it's *much costlier and traceable*.
A determined, patient, well-resourced attacker could still cultivate standing;
the point is that casual floods, drive-by bots, and cheap sock-puppet armies —
the 99% — are stopped, and the rest leave a trail. Proportionate claim, not magic.
