# 02 — What *is* this thing? (the category-noun problem)

The founder's genuine open question: *"Is it a system? Is it an app? I don't
know that yet."* This file works the problem. The category noun you pick is the
single most important word in the whole pitch — it tells the reader's brain
which shelf to put you on, and the wrong shelf kills comprehension instantly.

## Why this is hard

The product is genuinely cross-category:
- It's **software** (you install/run it) → "app" / "platform"
- It's a **set of rules and procedures** (membership, juries, sanctions) → "system" / "framework" / "constitution"
- It's meant to **plug into many other tools** (Lemmy, Matrix, PeerTube, MeshCore) → "layer" / "protocol" / "standard"
- It's a **way of doing things** → "method" / "practice" / "way of organising"

No single existing word is perfect. That's normal for genuinely new things
(cf. "search engine", "social network" — coined because nothing fit).

## Candidate nouns, ranked (with the trade-off)

| Noun | What it signals | Risk |
|---|---|---|
| **"a community jury for your group"** ★ | Concrete, civic, instantly fair-feeling. Research-backed: "jury" tests as *fairer than platform moderation* across the political spectrum (CHI 2020/2025). | Describes the *decide* mechanic, not membership + rule-making — surrounding copy must carry those. Needs the "verdict = fair collective decision, not adversarial judgment" reframe. |
| **"a way for groups to run themselves"** | Plain, human, benefit-first. No tech baggage. (Softened from "govern" per the derision finding below.) | Vague — needs a second sentence to become concrete. |
| **"self-governance layer"** | Captures the plug-into-anything truth. Honest to the architecture. | "Layer" is engineer-speak; organisers won't feel it. |
| **"governance protocol / standard"** | Most accurate to the cross-protocol vision; mirrors the email/ActivityPub story. | "Protocol" is cold and technical for a grassroots reader. |
| **"a constitution for online communities"** | Emotionally powerful, civic, memorable. Matches juries/rules/appeals. | Might sound heavy/legalistic; implies one fixed document (it's customisable). |
| **"app" / "platform"** | Familiar, low-friction. | *Wrong* — undersells it as one product, hides the portable/cross-protocol nature. Avoid as the primary noun. |

## Recommendation (founder decisions, 2026-06-27)

Two shifts from the research, then the same two-layer answer:

**Shift 1 — soften the greeting verb away from "govern."** Research (FrameWorks)
finds "govern / governance" triggers reflexive institutional derision in
non-technical readers. So the greeting verb becomes **"run your group / run things
your own way"** — warmer, plainer, same agency. "Govern" returns only one level
down (press / about-page).

**Shift 2 — promote "community jury" toward the primary greeting noun.** "Jury"
is concrete, civic, and tests as *fairer than platform moderation* across the
political spectrum (CHI 2020/2025) — where "governance" goes cold. Use it as the
lead category image, with the reframe that a **verdict here = a fair collective
decision by your peers, not an adversarial courtroom.**

Caveat to hold honestly: "jury" names the *decide* step, not membership or
rule-making — so the line around it must still carry those (the jury is *how
decisions get made*, inside a group that also writes its own rules and chooses
its own members).

The two-layer answer, updated:

- **For the organiser (pamphlet, hero line):**
  > "Run your group your own way — with a **community jury**, not a boss. Your
  > members write the rules, and your members (not an admin) decide fairly when
  > someone breaks them."

- **For the curious / press (one level down):**
  > "an open **self-governance layer** for online communities — the part that
  > decides, fairly and in the open, who belongs and what's allowed. It plugs into
  > the tools you already use."

- **For the technical reader (contributors, fediverse):**
  > "a portable, open-source **governance layer / emerging standard** for fair
  > self-government, hosted first on Lemmy, designed to work across ActivityPub,
  > Matrix, and other open protocols."

So: **"run your group / community jury" → "self-governance layer" → "standard"**
as you go deeper. The pamphlet leads with the jury image + the softened verb.

> ⚠ Testing note (research F): the *first* category label a reader meets dominates
> all downstream comprehension (Moreau 2001), so "community jury" is high-leverage
> and worth real A/B testing on actual organisers before it's locked in print.

## The protocol-agnostic truth (must be visible somewhere)

Per founder 2026-06-27: Brehon Consensus is meant to work with **all open-source
apps and protocols** — Lemmy (Reddit-style), Matrix (chat), PeerTube/YouTube
alternatives, MeshCore (mesh networking), and more. Lemmy is the *first* host,
not the definition.

This is the single fact that justifies "layer/protocol/standard" over "app." It
also unlocks the strongest analogy (see [`04-analogies.md`](04-analogies.md)):
governance that travels between communities the way **email travels between
providers** — you don't need everyone on the same software to play fair
together.

> **Working framing:** "It's to *fairness and self-rule* what email is to
> *messages* — an open standard that lets different communities, on different
> tools, govern themselves and still cooperate."

## How comparable projects solve the category-noun problem (research D)

The strongest comparable projects don't lead with *what they are* — they lead
with the **power relationship** ("not for sale", "stakeholders before
shareholders"), then introduce a precise category once the reader's engaged. That
two-layer move is the *industry pattern*, not a Brehon-specific problem — and it
validates leading with "run your group your own way / community jury" before
"layer" or "governance."

| Project | One-line pitch | Category noun | Power-contrast? | Lead analogy | Take for us |
|---|---|---|---|---|---|
| **Mastodon** | "Social networking that's not for sale." | social networking | Yes (commodity vs owned) | corporate-vs-people | power-contrast carries it without tech jargon |
| **Loomio** | "Transparent decisions your whole organization can trust." | decision-making platform | Implicit (email/chat gaps) | pain-naming | name the pain first; verb-led benefits |
| **Platform Coop** | "Stakeholders before shareholders." | cooperative | Yes (explicit) | 200-yr history | ownership language + historical anchor |
| **RJC** | "…those harmed into communication… play a part in repairing harm." | restorative practice | Implicit (repair vs punish) | human action, no system nouns | all-verb definition; drop system nouns |
| **Fediverse** | "Like a telephone network across providers." | federated network | No | telephone (lived experience) | analogy must be precise + end on "you won't notice" |
| **PolicyKit/Metagov** | "Open-source tool for evolving governance." | open-source tool | No | Ostrom / lab | caution: technical framing excludes lay users |

> Headline takeaway: **name the power relationship first** ("your rules, not the
> platform's"), then the noun. Brehon's lead should be the felt contrast + the
> jury image, with "layer/governance" as the destination words.

## Honesty flag

The cross-protocol reach is **vision, not shipped** (only Lemmy is wired today).
When using "works across Matrix/PeerTube/MeshCore," always frame as *designed
to / will*, never present tense. See
[`../source-notes/repo-essence-extract.md`](../source-notes/repo-essence-extract.md)
§Honesty note.

## Action

✅ Research done (Section F + D, 2026-06-27). Founder decisions locked: greeting
verb softened to "run your group"; "community jury" promoted as primary greeting
noun (with the verdict reframe); two-layer "jury → layer → standard" ladder kept.
**Remaining gate before print:** A/B-test "community jury" + "run your group
fairly" on real organisers — the first label dominates comprehension permanently
(Moreau 2001), so this one word is worth testing, not just deciding.
