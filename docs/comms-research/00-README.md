# Brehon Consensus — Communications & Messaging Research

This folder is the **evidence base for how we talk about Brehon Consensus** to
non-technical people — the raw material for a leaflet/pamphlet first, then a
website, press copy, and launch posts.

It is *not* the product docs (those live in
[`../brehon-law-inspired-network/`](../brehon-law-inspired-network/)). This is
about **translation**: turning a procedurally-correct governance system into
something a grassroots organiser can grasp in 60 seconds and want to adopt.

## What we decided (2026-06-27, with the founder)

| Decision | Choice |
|---|---|
| **Primary reader** | Grassroots organisers — activists, mutual-aid, co-ops, movements. Non-technical. Care about power, fairness, and running their own group their own way. |
| **First deliverable** | This research + themes. No finished copy yet — founder reviews the foundation before a word of pamphlet gets written. |
| **Brehon framing** | *Support, don't lead* — but reframed. Brehon law is **not** sold as romantic Irish heritage. It's **one piece of evidence that alternatives exist** (see "anti-TINA" below). |
| **Analogies** | The spine of the explanation (email-for-communities, references-for-a-job, jury-duty). |
| **Open question to resolve** | **What *is* this thing?** App? System? Protocol? Tool? A "way of organising"? See [`themes/02-what-is-it.md`](themes/02-what-is-it.md). |

## The two ideas that reframe everything

1. **Anti-TINA.** Neoliberalism's slogan is *"There Is No Alternative"* (TINA) —
   the belief that modern thin representative democracy is the only way to run
   anything. Brehon Consensus is a living counter-argument: **There Is An
   Alternative.** Brehon law is one of *many* historical proofs (Graeber &
   Wengrow, *The Dawn of Everything*) that humans have organised
   non-hierarchically, by consent and explicit rules, again and again. It was
   even cited successfully in a modern court case by Indigenous people. The
   product is the alternative made usable.

2. **Protocol-agnostic governance.** Brehon Consensus is **not** "a Lemmy fork"
   in the marketing. Lemmy is just the *first* host. The real thing is a
   **portable layer of fair self-government** designed to plug into any
   open-source app or protocol — Lemmy (Reddit-style), Matrix (chat), PeerTube
   (YouTube-style), MeshCore (mesh networking), and more. It complements those
   protocols the way a constitution complements a town: they move the messages;
   it decides, fairly and transparently, who belongs and what's allowed.

## Folder map (OKF-style: human-readable + machine-readable side by side)

```
comms-research/
├── 00-README.md                 ← you are here
├── themes/                      ← human-readable messaging foundation
│   ├── 01-essence.md            ← the one-sentence core + the 3 pillars
│   ├── 02-what-is-it.md         ← the category-noun problem (app? protocol?)
│   ├── 03-anti-tina.md          ← the "alternatives exist" narrative spine
│   ├── 04-analogies.md          ← the sticky comparisons, ranked + tested
│   ├── 05-audience-grassroots.md← who we're talking to, their words, their fears
│   ├── 06-message-ladder.md     ← 7-word → 1-line → 1-para → 1-page versions
│   ├── 07-traps-and-tone.md     ← what to avoid (jargon, romance, utopia-creep)
│   ├── 08-the-word-for-what-it-does.md ← moderation/mediation/governance choice
│   ├── 09-anti-cancel-culture.md← restorative = accountability without exile
│   └── 10-reputation-is-earned-by-helping.md ← standing from jury service, not clout
├── research-prompts/
│   └── perplexity-deep-research.md ← the prompt to run for evidence-based comms
├── source-notes/
│   └── repo-essence-extract.md  ← what the product actually does, plain English
└── machine/
    ├── messaging.yaml           ← structured message bank (for site/CMS reuse)
    └── glossary-plain.yaml      ← jargon → plain-English swaps
```

## How to use this

1. Read [`themes/01-essence.md`](themes/01-essence.md) first — it's the spine.
2. Run the Perplexity prompt in [`research-prompts/`](research-prompts/perplexity-deep-research.md);
   drop its output back into this folder as `research-prompts/perplexity-results-<date>.md`.
3. With themes + research evidence agreed, *then* write the pamphlet.

## Status

Foundation draft — 2026-06-27. Awaiting founder review of themes before copy.
