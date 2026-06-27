# NEXT SESSION — what to do with the Perplexity research findings

You are picking up the Brehon Consensus comms-research work. The previous session
built the messaging foundation (themes 01–15 + machine bank + the Perplexity
prompt). The user is now bringing back the **Perplexity Deep Research output**.

## Step 0 — orient (5 min, don't skip)

1. Read [`00-README.md`](00-README.md) — the decisions, the master spine
   ("bottom-up, not top-down"), the folder map.
2. Skim the `themes/` files 01–15. They are the agreed foundation. Do NOT
   re-litigate decisions already locked there.
3. ✅ DONE (2026-06-27): the research output is already committed at
   [`research-prompts/perplexity-results-2026-06-27.md`](research-prompts/perplexity-results-2026-06-27.md)
   (372KB, immutable source). Read it — then go to Step 1. Do NOT re-fetch or
   re-run; the findings are in.

## Step 1.0 — DIGEST the report first, via subagent fan-out (DO NOT read it whole)

The report is **2,669 lines / 372KB (~95K tokens)**. Reading it into the main
context will blow your working window and leave no room to actually do the
reconcile. **Do not read the file end-to-end.** Instead, **fan out one subagent
per section** (the `Explore` or `general-purpose` agent), each reading ONLY its
bounded line-range and returning a tight, fixed-shape digest. The main agent then
holds eight ~1-page digests (~8–12K tokens total), not the raw 95K.

The report has clean `# Section A–G` boundaries (verified 2026-06-27). Dispatch
all eight in ONE message so they run concurrently:

| Agent | Section | Read ONLY lines | Reconciles into |
|---|---|---|---|
| 1 | A — explaining novel ideas | 26–357 | themes/04, 06 |
| 2 | B — framing / narrative / anti-TINA | 358–702 | themes/03 |
| 3 | C — trust / reputation / hype | 703–1015 | themes/10, 07 |
| 4 | C2 — cancel culture / restorative / consent | 1016–1305 | themes/09, 10 |
| 5 | D — competitor positioning | 1306–1858 | themes/02, 08 |
| 6 | E — history verification | 1859–2189 | themes/03 + proof_points |
| 7 | F — naming | 2190–2525 | themes/02, 08 |
| 8 | G — synthesis / rules / risks | 2526–2669 | new themes/16 |

**Prompt each subagent like a colleague who hasn't seen the project** (per
`.claude/rules/advisor-orchestrator.md` §6.3). Give it: the file path + its EXACT
line range (`Read` with `offset`/`limit`), one line on what Brehon Consensus is,
the theme file(s) its section maps to, and this **fixed digest schema** so the
eight outputs compose:

```
Return ONLY this structure (no preamble, ≤400 words):
## Section <X> digest
- KEY FINDINGS (3–6 bullets): the evidence-backed claims, each with its source/citation
- CONFIRMS: which of our existing theme positions this evidence supports (name the theme + claim)
- CHALLENGES/CORRECTS: anything that contradicts or complicates a theme — what changes and why
- VERIFY FLAGS: any claim the report itself marks contested/unverifiable (esp. Section E history)
- ACTION ITEMS: concrete edits to make to the named theme file(s)
```

Because each subagent's *raw* section text stays in the subagent and only the
digest returns, the main context never holds the 95K. Save the eight digests
concatenated to `research-prompts/digests-2026-06-27.md` (commit it) — that file,
not the raw report, is the working input for Step 1.

> Cost note: 8 subagents × one bounded read each is far cheaper than one main-agent
> 95K read, AND it parallelises. If a section is still too big for one agent (none
> here exceed ~550 lines), split that one agent's range in half.

> Smaller alternative (if you can't fan out): process **one section per turn**
> in the main context — read lines X–Y, reconcile into its theme, commit, drop it,
> next turn. Slower, serial, but bounded. The fan-out is strictly better.

## Step 1 — RECONCILE the findings against the themes (this is the main job)

Work from the **digests** (Step 1.0 output), not the raw report. The research is
*evidence to harden or correct the themes*, not a thing to admire. Go section by
section (the prompt maps each to specific files):

| Research section | Reconcile into |
|---|---|
| A (cognitive science: analogy, plain language, chunking) | `themes/04-analogies.md`, `06-message-ladder.md` — confirm or adjust the analogy spine |
| B (framing, narrative, anti-TINA) | `themes/03-anti-tina.md` — sharpen the "lots of alternatives" frame |
| C (reputation framing, credibility, hype) | `themes/10`, `07-traps-and-tone.md` |
| C2 (cancel culture, restorative, procedural justice, consent) | `themes/09`, `10` §community-set-stakes |
| C3 (bad-actor defence, distributed moderation, bottom-up rules, low-stakes adoption) | `themes/11`, `12`, `13`, `14`, `05` on-ramp |
| D (competitor positioning) | `themes/02-what-is-it.md`, `08` — new comparison table |
| **E (history verification)** | `themes/03` + `machine/messaging.yaml` `proof_points` — see Step 2 |
| F (naming) | `themes/02`, `08` — settle the category noun |
| G (synthesis, top-10 rules, risks) | new file: `themes/16-evidence-synthesis.md` |

For each: where the evidence **confirms** a theme, add the citation. Where it
**contradicts or complicates**, edit the theme and note what changed and why.
Keep edits surgical and commit per-theme with clear messages.

## Step 2 — RESOLVE the open verification flags (load-bearing — do before any copy)

Three things are tagged "must verify" across the folder. The research should
settle #1; you must check #2 and #3 against the repo:

1. **The court-case claim** (customary/tribal law winning in a modern court).
   Section E should give a real citable case (Mabo / tikanga / First Nations) or
   confirm no Brehon-specific one exists. Update `themes/03` + `messaging.yaml`
   `proof_points` accordingly. If unverified → it stays OUT of public copy.
2. **Which customisation knobs are actually live in v0** vs. design (decay rate,
   reporter-reward, jury cooldown, min-standing-to-be-called). Grep the repo
   (`crates/`, `docs/brehon-law-inspired-network/01-vision §5`, `04-data-model`).
   Mark each knob in `themes/15` live-or-design.
3. **The reporter-reward mechanic** (does reporting-accuracy reputation rise on an
   upheld report in v0?). Check `04-data-model` + the governance crates.

Honesty rule stays in force: present tense for shipped only; "designed to" for
vision. Alpha / solo-dev / Lemmy-only-so-far.

## Step 3 — only THEN write (gate: user confirms)

After reconcile + verification, surface a short summary to the user:
"themes hardened, N corrections, open flags resolved/remaining — ready to write?"
**Wait for the user.** Do not start drafting the pamphlet unprompted.

When greenlit, the first deliverable is **one pamphlet** (rung 3 in
`06-message-ladder.md`), written to grassroots organisers, bottom-up spine,
analogy-led, honest about alpha status. Save drafts under `drafts/`.

## Working agreement carried from last session

- **Notes-then-write mode.** Capture and summarise points back to the user before
  writing copy; don't ping-pong on wording mid-stream.
- This is **docs meta-work on `governance-v0`** — direct commits, no PR
  (per `.claude/rules/phase-branch.md`). Commit aggressively in small units.
- Tone: plain, warm, sober. Not startup hype, not manifesto, not Celtic romance.

## State at handover

- 15 theme files + `source-notes/` + `machine/` (messaging.yaml, glossary) + the
  Perplexity prompt, all committed on `governance-v0`.
- Last commits: `af1f8af01` (wiring pass), `c756ea601`, `dcf6f2d2d`, `ecc4ab9fb`.
- Nothing pending; folder is internally consistent. No pamphlet drafted yet.
