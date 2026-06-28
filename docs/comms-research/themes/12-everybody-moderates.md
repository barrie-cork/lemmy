# 12 — Who does the moderating? Everybody — a little

Founder point (2026-06-27): the obvious question any organiser asks is **"who
does the moderating?"** Brehon Consensus's answer is its quiet masterstroke:
**everybody does, a little.** The work is shared across the whole membership, so
no one carries it alone — and with enough people, each person's share is tiny.

## The core claim

> **There's no moderator class here. When a serious case comes up, members are
> drawn to decide it — like jury duty. The responsibility *and* the workload are
> shared across everyone. The bigger the community, the lighter the load on any
> one person.**

## Why this is the answer to the deepest organiser pain

The audience's #1 burnout from [`05-audience-grassroots.md`](05-audience-grassroots.md):
> *"I'm burned out being the unpaid referee."*
> *"If I step back, it all falls apart, because everything's in my head."*

Every community runs on the unpaid emotional labour of one or a few people who
become the de-facto moderators — and they burn out, get resented, become a
single point of failure, or quietly become the unaccountable clique. **This is
the structural disease of community moderation.** Brehon Consensus's cure is to
**distribute the role itself**, not just give the same overworked admin better
tools.

## The counter-intuitive scaling (the surprising, memorable bit)

Normal platforms: **more members = more moderation load = more burnout.** The
work piles onto the same few people, who drown.

Brehon Consensus: **more members = *lighter* load per person.** Because cases are
shared out across the eligible pool, the more people there are to share it, the
less often any individual is called, and the smaller everyone's slice. The system
gets *more* sustainable as it grows, not less.

> One-liner: **"No one is the moderator, because everyone is — a little. And the
> more of you there are, the less each of you has to do."**

This is genuinely surprising — it inverts the assumption that growth makes
moderation harder. Lead with the surprise; it earns attention.

## Built-in fairness of the draw (founder, 2026-06-27)

The distribution isn't incidental — **it's built into the app to be fair on
purpose:**

- **Randomly selected** — jurors are drawn at random from the eligible pool, so
  it's not the same handful of people every time.
- **You get a break** — after serving, you're rested for a period before you can
  be called again, so the load genuinely rotates and no one is over-tapped.

The design goal stated plainly: **make it as fair as possible** — both fair in
*who decides* (random, not a clique) and fair in *who carries the work* (rotated,
with rest). This is the mechanism that makes the "everybody, a little" promise
real rather than aspirational.

> Honesty: ✅ verified live in v0 (config.rs + admin_assign_jury.rs, 2026-06-27).
> Jurors are drawn by SQL `ORDER BY random()` (genuinely random, re-rolled each
> case), and the cooldown/rest is real (`jury.constraints.juror_cooldown_days`,
> default 7) — both safe to state present-tense. Bonus: the draw is also
> *diversity-aware* (a 3-phase filter that re-rolls if one sponsor-cluster would
> dominate a panel), so "random, not a clique" is literally enforced, not just
> aspirational.

## What this delivers (the benefits to spell out)

- **No burnout.** The job isn't dumped on one exhausted volunteer.
- **No single point of failure.** If anyone steps back, the community doesn't
  collapse — the role was never in one person's head.
- **No unaccountable clique.** Power can't concentrate in a permanent mod team,
  because the role rotates through the membership.
- **Shared ownership.** When everyone takes a turn deciding, everyone has a stake
  in the rules being fair — moderation stops being "them vs us."
- **Minimal per-person effort.** In a healthy-sized community, you might be called
  occasionally, briefly. Not a job — a civic turn.

## How it connects to the other themes

- **It's the *engine* behind the juries** (Pillar 2, "decide together") — this
  file is the *who and how-much*, the jury analogy is the *feel*.
- **It's why standing is earned by jury service** ([`10`](10-reputation-is-earned-by-helping.md)):
  taking your turn *is* the helping. Shared work and earned standing are the same
  loop seen from two sides.
- **It reinforces "it's yours"** — the community moderates itself, literally, with
  no outside moderators and no internal bosses.

## Honesty guard-rails

- **"Minimal *if there are enough people*."** Be honest about the dependency: in a
  *small* community the pool is small and turns come round more often. The "load
  is tiny" claim is true at healthy scale; say "as the community grows" rather
  than implying it's effortless from day one.
- **Don't imply zero effort.** It's *shared* effort, not *no* effort — serving on
  a panel means actually reading the case and deciding in good faith. The promise
  is "a fair, light, shared turn," not "moderation does itself."
- **v0 status:** ✅ the jury-draw + eligibility-pool + cooldown mechanics are
  **live in v0** (verified 2026-06-27) — random selection, a 7-day default rest,
  and concurrent-assignment caps all real and tunable. The "everybody, a little"
  promise rests on shipped code, not just design.
