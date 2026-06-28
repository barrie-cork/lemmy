# 13 — The moderation journey, step by step

Founder point (2026-06-27): we'd described juries abstractly but never walked the
actual flow. This is the concrete "what happens when someone breaks a rule"
story a pamphlet needs. **Critically: cases are judged against the community's
RULES — not against whether someone is offended.**

## The flow (end to end)

1. **Someone posts or comments.** On whatever platform the community lives (e.g. a
   Lemmy server), a member writes a post or a comment.

2. **A member reports it — against a rule, not a feeling.** Any member who thinks
   it **breaks one of the community's rules** can report it. The report isn't
   *"I'm offended"* — it's *"this breaks rule X,"* and the reporter can attach
   **evidence**: quotes of other comments, context, links.

   > ★ The crucial correction (founder): **a case is judged by the rules, not by
   > whether one person finds it offensive and another doesn't.** Feelings are not
   > the standard; the community's agreed rules are. (If a behaviour genuinely
   > feels harmful but no rule covers it, that's not a report — that's a reason to
   > **propose a new rule**; see [`14-bottom-up.md`](14-bottom-up.md).)

3. **A jury is formed.** A panel of members is drawn to look at the claim. Their
   job is narrow and fair: **compare the reported content to the rule cited,**
   weighing the evidence gathered.

4. **The jury decides.** Found to break the rule, or not — by the community's
   agreed thresholds.

5. **Reputation moves on the outcome:**
   - **Found to break the rule →** the reported person's **standing goes down.**
   - **The reporter may gain a little standing** — for putting their neck out to
     protect the community. **This is customisable + controversial:**
     - It's **gated to *upheld* reports only** — you're rewarded for a *correct,
       rule-based* report, never for a personal vendetta. A rejected/bad-faith
       report earns nothing (and over time, crying wolf lowers reporting standing).
     - A community can **turn this off** if it worries about incentivising
       reporting. (→ [`15-yours-to-tune.md`](15-yours-to-tune.md).)

6. **Over time, low standing shrinks your role.** Repeated upheld findings →
   reputation falls lower and lower → **a smaller role in the community:**
   - **less likely to be called** for jury duty,
   - possibly a **minimum threshold** below which you're **not called at all**,
   - and maybe **can't vote on certain things.**

   > This is the downside mirror of "standing unlocks what you're trusted to do"
   > ([`10`](10-reputation-is-earned-by-helping.md)): standing earned by helping
   > unlocks roles; standing lost by breaking rules removes them. Same dial, both
   > directions.

## Why this flow is the fair version

- **Rules, not feelings** — no one is sanctioned on subjective offence; only
  against norms the community agreed in advance (consent, [`10`](10-reputation-is-earned-by-helping.md)).
- **Evidence-based** — the report carries its reasons and proof; the jury weighs
  them, it's not a vibe.
- **Peer-decided** — a drawn panel, not an admin ([`12`](12-everybody-moderates.md)).
- **Proportionate + restorative** — consequences are graduated and aim at repair,
  not a one-strike exile ([`09`](09-anti-cancel-culture.md)).
- **Self-correcting** — the role-shrinking happens *gradually* over repeated
  findings, not on a single mistake.

## Plain-language one-liner

> **"See something that breaks your community's rules? Report it, with your
> reasons. A panel of members checks it against the rules. If it breaks them,
> there's a fair, proportionate consequence — and it's all in the open."**

## Honesty guard-rails

- ✅ Verified live in v0 (2026-06-27): the full flow is shipped — random jury
  draw (`ORDER BY random()`), reporter-reward on reporting-accuracy
  (`reporter_upheld` +10 / dismissed −5), the minimum-standing gates
  (`DEFAULT_THRESHOLDS_*`), and the community-set decision threshold
  (`jury.quorum`). All safe to state present-tense. (The only governance knob
  that's design-not-default is reputation *decay* — irrelevant to this flow.)
- Don't imply the jury judges *people* — it judges **content against a rule.**
  Keep that distinction crisp; it's what separates this from a popularity trial.
