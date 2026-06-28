# 04 — Analogies (the explanatory spine)

The founder chose: **analogies are the spine.** Good call — concrete-first
explanation is one of the most evidence-backed comprehension aids (the
Perplexity pass will harden the cognitive-science behind this). The rule:
**every abstract mechanic gets a familiar anchor.** Below, ranked, with the
trade-off and the failure mode of each.

## Tier 1 — anchor analogies (carry the whole pitch)

### A. Email for communities  ★ the lead analogy
> "You can email anyone, on any provider, with any app — because everyone agreed
> on one open standard underneath. Brehon Consensus does that for *governance*:
> different communities, on different tools, can each govern themselves fairly
> and still cooperate — because they share one open way of doing it."

- **Carries:** open standard, federation, no central owner, plug-into-anything,
  the cross-protocol vision.
- **Pedigree:** this is exactly how Lemmy/ActivityPub/Mastodon explain
  federation, so it's proven on this audience.
- **Failure mode:** email is about *moving messages*; we're about *making
  decisions*. Be explicit: "email moves the messages; Brehon Consensus decides,
  fairly, who belongs and what's allowed." Don't let the reader think it's a
  comms tool.

### B. References / vouching to join  ★ the trust analogy
> "Joining is like getting a reference for a job, or being signed in to a club by
> two members. Real people vouch for you — and they put a bit of their own
> reputation on the line by doing it."

- **Carries:** sponsorship/surety, shared risk, sybil/fake-account resistance,
  "trust is relational."
- **Failure mode:** could feel exclusionary ("gatekeeping"). Counter immediately
  with the time-based fallback: "no one to vouch for you yet? You can also earn
  your way in over time."

### C. Jury duty  ★ the fairness analogy
> "When something serious happens, it's not a boss who decides — it's a panel of
> ordinary members, picked like a jury. They look at it, they decide together,
> and the reasons are written down for everyone to see."

- **Carries:** juries, no permanent moderator class, peer judgment, transparency,
  consensus thresholds.
- **Pedigree:** near-universally understood; carries deep "fairness" connotation.
- **Failure mode:** juries evoke courts/punishment — exactly the punitive frame
  we're escaping. Pair every jury mention with the *restorative* outcome: "and
  the goal isn't to punish — it's to put things right."

## Tier 2 — supporting analogies (for specific mechanics)

### D. A community's own rulebook / constitution
> "Every group writes its own rules — and can change them. Like a club
> constitution or a co-op's bylaws, but living and visible to all."
- Carries: customisable rule sets, self-organisation, "you make your own rules."

### E. A repair café / restorative-justice circle, not a courtroom
> "Closer to mending something together than to a trial. The first question is
> 'how do we put this right and keep you in the group?', not 'how do we punish
> you?'"
- Carries: graduated/restorative sanctions, reintegration, inclusivity.

### F. Earning trust like a new co-worker (not a credit score)
> "Standing builds the way trust does with a new colleague — slowly, through what
> you do. It's not a number on a leaderboard; it just quietly unlocks what
> you're trusted to help with."
- Carries: reputation-as-capability, no single score, earned-slowly, **earned by
  helping not by activity** (posting/liking/sharing don't count).
- **Failure mode:** must actively *distance* from "social credit score" (China)
  and "credit rating" — both carry dystopian baggage. The positive fix beats the
  disclaimer: "you earn it by *helping* — today, by serving fairly on a jury — not
  by being active or popular." See
  [`10-reputation-is-earned-by-helping.md`](10-reputation-is-earned-by-helping.md).

### G. Neighbours warning each other (not a police force)
> "Communities can tip each other off — 'heads up, this account caused trouble
> here' — but no community can order another one around. Like neighbours sharing
> a warning, not a central authority issuing orders."
- Carries: federation = advisory not binding, local sovereignty.

## Analogies to AVOID (they import the wrong meaning)

| Tempting analogy | Why it backfires |
|---|---|
| **Blockchain / DAO / crypto-governance** | Imports tokens, speculation, "more money = more power" — the *exact opposite* of the project. Hard no. |
| **Social credit score** | Authoritarian surveillance connotation. Reputation here is earned, capability-based, no single number — say so by contrast, and name the *Black Mirror "Nosedive"* fear to kill it. (Note: decay is *designed-to*, off by default in v0 — don't lean on "decaying" as a present-tense contrast.) |
| **Supreme court / legal system** | Too heavy, too punitive, implies lawyers and permanent judges. We have *temporary peer juries*, not a judiciary. |
| **Wikipedia moderation** | Real but ambivalent reputation (edit wars, cabals) — a contested example, invites argument. |
| **"Reddit but nicer"** | Undersells to a feature tweak; hides that it's a governance layer, not a forum. |

## The test for any new analogy (before it ships)

1. **Does the reader already understand the source?** (job reference: yes.
   ActivityPub: no.)
2. **Does it carry the *right* connotation?** (jury = fairness ✔ but also
   punishment ✖ — needs a pairing.)
3. **What does it get wrong, and have we said so?** Every analogy leaks; name the
   leak in one clause so it doesn't mislead.

> Rule of thumb: **one anchor analogy per pillar** (email→travels,
> references→trust, jury→fairness), supporting analogies only when a specific
> mechanic needs unlocking. Don't stack five metaphors in one paragraph — that's
> noise, not clarity.

## Two research-backed craft rules (Section A)

1. **Use at least two analogies from different surface domains for the *core*
   idea** — not one. A single analogy transfers spontaneously only ~30% of the
   time; naming it explicitly raises that to ~76%, and a *second* structurally
   similar analogy from a different domain is what makes the reader extract the
   underlying principle rather than just the story (Gick & Holyoak). So the
   self-government idea should land via *both* "email" (it travels) *and* "jury"
   (it's fair) — two anchors, one schema. (The jury analogy now also doubles as
   the lead category noun — see [`02-what-is-it.md`](02-what-is-it.md) — so it
   carries extra weight; keep "email" beside it so the *portable* half isn't lost.)

2. **Concrete vignette before the abstract benefit** (Rawson: +40% when the vivid
   example leads). Open each rung/section with a 2-line scene, *then* the
   definition — not the reverse. See [`06-message-ladder.md`](06-message-ladder.md).

> Process note (the curse of knowledge, Section A): the only reliable test of an
> analogy is a *cold reader* — someone from the [`05`](05-audience-grassroots.md)
> audience who hasn't read these themes. Test the lead analogies on 3–5 real
> organisers before print; we cannot self-assess comprehension reliably.
