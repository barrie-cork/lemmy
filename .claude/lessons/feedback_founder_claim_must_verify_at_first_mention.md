---
name: Tag a founder's factual claim as must-verify at first mention — never ship it as settled fact
description: When founder/stakeholder framing for public-facing copy leans on a factual claim (a historical event, a court case, a statistic, a "this has been done before"), record it as a must-verify proof-point with a verification hook at the FIRST mention — do not let it propagate into draft copy as settled fact. One debunked fact discredits the whole narrative it supports. The content analogue of the verify-automated-reviewer-claims-against-compiler gate.
type: feedback
---

When a founder or stakeholder gives you framing for public-facing copy (pamphlet,
website, press, README) and it **leans on a factual claim** — a historical event,
a court case, a statistic, a "this has been done before / this won a case / X% of
people" — treat the claim as a **hypothesis to verify**, not a fact to repeat.
Record it as a **must-verify proof-point with a verification hook at the first
mention**, and keep it out of any draft copy until verified. Do not let it
propagate through the messaging files as settled fact just because the founder
stated it confidently.

**Why:** In the 2026-06-27 comms-research session the founder's narrative spine
("there are lots of alternatives — fair non-hierarchical societies have really
existed, and customary/tribal law even won a modern court case") depended on
specific factual claims. The court-case claim was an unverified recollection — no
citable case was confirmed. I correctly tagged it must-verify in most files, but
it had *already shipped once* in the README as confident prose ("It was even cited
successfully in a modern court case by Indigenous people") before I softened it.
For an evidence-based pitch to a savvy, sceptical audience (grassroots organisers,
journalists, FOSS folk), **one debunked fact discredits the entire narrative it
supports** — and the narrative spine is the most load-bearing part of the copy.
The cost of verifying first is minutes; the cost of shipping a wrong fact is the
credibility of the whole project.

This is the **content/marketing analogue** of
`feedback_verify_automated_reviewer_claims_against_compiler.md` and the
falsifiable-hypothesis gate (`feedback_falsifiable_hypothesis_before_structural_fix.md`):
a confidently-stated claim that names a specific verifiable thing carries an
implicit "this is true" that is a hypothesis. For code the falsification is a
`cargo check`; for a factual claim it's a source check (web research / primary
source). Cheaper than the damage either way.

**How to apply:**

1. **Spot the claim class at first mention.** Historical events, named court
   cases, named people/books (as evidence, not just attribution), statistics,
   "this has been done / this works / this won" assertions. Attribution-only
   mentions ("the founder believes…") are fine; *evidentiary* claims are the
   trigger.
2. **Record it as a must-verify proof-point, not prose.** In the machine bank /
   notes, log: the claim, the (recalled) source, and `status: must-verify` or
   `status: verify-before-public`. Add a concrete **verification hook** — e.g. a
   section in the research prompt that will pin down a citable source, or a
   primary-source check to run.
3. **Keep it out of draft copy until verified.** If it must appear in a
   foundation doc, frame it explicitly as "⚠ verify" / "the founder recalls X;
   needs a citable source" — never as a flat factual sentence.
4. **On verification:** either replace with the real, cited fact, or — if no solid
   source exists — say so plainly and **cut it from public copy**. "No verifiable
   case found" is a valid, honest outcome.
5. **Honesty rule carries through** (pairs with the alpha-status / "designed to
   not is" discipline): present-tense factual claims in public copy must be true
   and citable; aspirational or recalled-but-unverified claims are flagged, not
   asserted.

**Generalises to:** any public-facing or outward-visible deliverable built from
stakeholder framing — marketing copy, grant applications, press releases, launch
posts, About pages. The more rhetorically load-bearing the claim, the more it
must be verified before it ships, because that's exactly the claim a hostile
reader will fact-check first.
