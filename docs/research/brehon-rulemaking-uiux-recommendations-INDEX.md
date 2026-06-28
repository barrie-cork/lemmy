# Navigation digest for brehon-rulemaking-uiux-recommendations.md (180 lines, ~9.5KB)

> Read this first; open only the sections you need. Line numbers are SOURCE-file hints.

## Overview

This report recommends UI/UX patterns for Brehon Consensus's community rulemaking layer. The source identifies a critical gap: while the code has append-only rule versioning, dry-run impact previews, and case/jury lifecycle UIs, there is **no member-facing proposal/deliberation surface** for how rules get created and adopted by consent. The recommendations build a staged rulemaking lifecycle (Proposal → Discussion → Consent check → Ratification → Enactment) that mirrors the jury service model, reuses existing impact-preview and audit-log machinery, and enforces the design principle that constitutional changes must feel procedurally distinct from ordinary config tuning.

## Section map

| Lines | Heading | What's in it |
|---|---|---|
| 1–6 | Title + provenance | Report source: barrie-cork/lemmy fork; grounded in code reads |
| 7–38 | §1. What exists today | Current state: append-only rule chains, dry-run previews, admin-only config UI, zero deliberation surface |
| 9–13 | 1.1 Rules are append-only version chain | `rule_set_version` table + migration; top-down admin write; ADR-010 grandfathering invariant |
| 15–19 | 1.2 Config has dry-run impact preview | `compute_downstream_impact` UX primitive; existing config metadata + procedural-friction hooks |
| 21–23 | 1.3 Only UI is admin dashboard | Server-rendered maud page + live audit tail; member-facing governance UI absent |
| 25–27 | 1.4 Decisive gap: no proposal/deliberation | Zero API surface for proposed/discussed/consensual rules; today binary: moderator writes, then rule exists |
| 29–37 | 1.5 Specs already require this | Design docs 03 + 05 explicitly ask for rulemaking lifecycle, step-up auth, distinct constitutional tier, procedural transparency |
| 40–48 | §2. Design principles | Six anchoring principles: mirror jury lifecycle, two-tier visuals, show blast radius, consent model not majority, audit-log writes, member-facing first |
| 51–82 | §3. UI/UX for configuring rules | Rulebook page + version timeline/diffs, structured-rules evolution, guided config editor, ordinary vs constitutional visual tiers |
| 53–59 | 3.1 Rulebook page | Member-readable version history, side-by-side diffs, "rule in force" links from cases, verifiability affordances |
| 61–67 | 3.2 Structured rules | Evolve `rule_text` to canonical list format; preserve single hash + append-only; unlocks per-rule editing/linking/deliberation |
| 69–75 | 3.3 Guided config editor | Dynamically generated from API metadata; live impact panel; cost-of-change visibility |
| 77–81 | 3.4 Tier-distinct visuals | Ordinary tuning = lighter, single mod enact; constitutional = distinct colour, multi-approver, mandated delay, step-up auth |
| 85–130 | §4. UI/UX for deliberation | New proposal lifecycle state machine, consent/objection model, threaded discussion, live tally, objection register, Matrix notifications |
| 89–97 | 4.1 Proposal lifecycle | New state machine: Draft → Discussion → Consent check → (Ratified → Scheduled → Enacted); new aggregate + child tables; enacts via existing privileged writers |
| 99–110 | 4.2 Consent/objection model | Four positions (Consent, Consent-with-reservation, Stand-aside, Object); reasoned blocks; quorum + threshold config; standing-based weight, not single score |
| 112–119 | 4.3 Deliberation surface | Threaded discussion anchored to diff, live tally + bar visualization, objection register, inline impact panel, deliberation timer, audit drawer |
| 121–126 | 4.4 Channel split (Lemmy/Matrix) | Authoritative state on Lemmy; Matrix for notifications + coordination only; reflects §05 architecture principle |
| 128–130 | 4.5 Observer transparency | Public proposal page (pseudonymous positions, impact, enacted-version link, governance-log references) for non-participants |
| 134–158 | §5. Staged implementation | Five phases: (1) rules legible, (2) config editor+impact, (3) structured rules, (4) proposal engine, (5) constitutional safeguards |
| 161–168 | §6. Pitfalls to avoid | Five design-invariant traps: break append-only, collapse standing to score, make constitutional look casual, make objection costless, strand state in Matrix |
| 171–180 | Source files referenced | Crate paths, migration, design docs, API surface, existing UI |

## Recommendations summary

| Component | Stage | Key recommendation | Deciding factor | Source |
|---|---|---|---|---|
| Rules legibility | Phase 1 | Render Rulebook page with version badge, hash, timeline, diffs | "Fair system should not rewrite operative rule text secretly" | L53–59, 03§3 |
| Config editing | Phase 2 | Generate guided editor from API metadata; wire dry-run impact panel | Existing UX asset reusable; highest-leverage move available | L69–75, 1.2 |
| Rule structure | Phase 3 | Migrate to canonically-serialized rule-item list; preserve single hash | Unlocks per-rule editing/linking/deliberation without breaking ADR-010 | L61–67 |
| Rulemaking lifecycle | Phase 4 | Proposal aggregate → Discussion → Consent check → Ratification → Enactment | Mirrors jury lifecycle procedural legitimacy; sits *in front of* privileged write | L89–97 |
| Consensus model | Phase 4 | Consent/objection + configurable quorum + reasoned blocks; weight by standing | Anti-capture philosophy; objection register forces discourse not drive-by veto | L99–110 |
| Tier distinction | Phase 2–5 | Visual + procedural split: constitutional = longer window, multi-approver, delay, step-up | Spec explicitly requires members distinguish ordinary moderation from constitutional revision | L77–81, 03§6 |
| Deliberation UI | Phase 4 | Threaded discussion + live tally (with bar shown) + objection register + impact panel | Transparency of process is civic work norm; matches case/jury UX language | L112–119 |
| Audit integration | Phase 4–5 | Every proposal/position/enactment appends signed governance_log entry | Proposal layer sits in front of existing writers; audit live-tail covers rulemaking for free | L45, 4.1 |
| Channel split | Phase 4 | Matrix notifications only; authoritative state on Lemmy Rulebook/Proposal pages | Enforces design principle "communication may happen elsewhere, authoritative state recorded formally" | L121–126, 05§2+5.3 |
| Safeguards | Phase 5 | Enforce multi-approver, mandated delay, step-up for constitutional-tier; gate via tier not toggle | Spec's non-customizable list (appeals, logging, due process) must be unrepresentable as casual flip | L155, L165 |

## Key recommendations (≤8 bullets with source line cites)

1. **Build a Rulebook page as the anchor.** Render the active `rule_set_version` with version badge, `text_sha256` hash, and a side-by-side diff viewer over the parent_id chain. This serves §1.5 spec requirement ("fair system should not judge under one rule and quietly rewrite it") and reuses existing data from `admin_list_rule_sets`. (L53–59)

2. **Reuse the dry-run impact preview UX as the highest-leverage move.** Wire `compute_downstream_impact` into the guided config editor so members see "this change makes 41 fewer eligible" *before* voting, not after. This primitive already exists in production code. (L69–75, L15)

3. **Separate ordinary tuning from constitutional revision with visually distinct tiers.** Per §03§6 spec requirement, ordinary in-range policy changes get lighter UI + single moderator enactment; constitutional amendments (rule-text, foundational-flagged keys) get distinct colour + multi-approver + mandated delay + step-up auth. Members must be able to tell them apart at a glance. (L77–81, L43)

4. **Mirror the jury lifecycle procedural legitimacy with a proposal state machine.** Design a Proposal lifecycle (Draft → Discussion → Consent check → (Ratified → Scheduled → Enacted)) as a new aggregate that sits *in front of* the existing `admin_create_rule_set` / `admin_set_config` writers. This preserves ADR-010 (append-only grandfathering) and satisfies §04 norm that governance is civic work, not a toggle. (L89–97, L42)

5. **Use consent/objection model with reasoned blocks, not raw 51% voting.** Members register Consent / Consent-with-reservation / Stand-aside / Object; objections must include rationale (mirrors juror rationale requirement). Enactment bar: configurable quorum + consent threshold + all objections either resolved or formally overridden. Weight by standing dimension, never collapse to one score. (L99–110, L4)

6. **Render live tally with the bar shown and objection register visible.** On every proposal discussion page, show "12 consent / 2 reservations / 1 stand-aside / 1 objection — threshold: 75% consent + 0 unresolved objections; quorum 10 ✓" + a work-item per open objection with its rationale thread. This makes the process followable and the block meaningful. (L112–119, L115)

7. **Write every proposal/position/enactment to governance_log as signed entries.** The proposal layer appends to the same hash-chained audit trail as cases/decisions. Matrix carries notifications; authoritative state is recorded formally on the Lemmy surface. This enforces "communication happens elsewhere, but authoritative governance state is written back to the formal system." (L45, L121–126)

8. **Never let constitutional safeguards be casually flipped.** The spec's non-customizable list (removing appeals, removing logging, unrestricted admin override) must be unrepresentable as a `valid_range` in-range config flip. Gate behind the constitutional-tier rulemaking flow or make non-configurable. This is a design invariant, not a suggestion. (L165, 03§6)

## Where to look for X

| If you need to know | Look here | Line range |
|---|---|---|
| What rule/config state already exists and what's missing | §1. What exists today | L7–38 |
| What the existing code already does well (impact preview) | §1.2 Config has dry-run preview | L15–19 |
| Why today's zero deliberation surface is the critical gap | §1.4 Decisive gap | L25–27 |
| What the design specs say about rulemaking (must have phases, multi-tier, audit) | §1.5 Specs already ask for this | L29–37 |
| The six design anchors that justify all UI recommendations | §2. Design principles | L40–48 |
| How to build the member-facing Rulebook page and version diffs | §3.1 Rulebook page | L53–59 |
| Why/how to migrate rules from free-text blob to structured items | §3.2 Structured rules | L61–67 |
| How to generate policy config UI from existing metadata | §3.3 Guided editor | L69–75 |
| How to make constitutional vs ordinary changes look/feel different | §3.4 Two-tier visuals | L77–81 |
| What the new proposal state machine and lifecycle look like | §4.1 Proposal lifecycle | L89–97 |
| How consent/objection model works vs majority voting | §4.2 Consent model | L99–110 |
| What controls/affordances the deliberation page needs | §4.3 Deliberation surface | L112–119 |
| Why Lemmy hosts authoritative state and Matrix carries notifications | §4.4 Channel split | L121–126 |
| What observers (non-participants) should see | §4.5 Observer transparency | L128–130 |
| Five concrete phases to build (legibility → config → rules → proposal → safeguards) | §5. Staged implementation | L134–158 |
| Design traps that would break the spec or ADR-010 | §6. Pitfalls to avoid | L161–168 |

## Coverage notes

- **Source audit path**: All file references are cited as real paths in barrie-cork/lemmy fork (crate files, migrations, design docs). This index does not verify those paths are correct — it flags where the recommendations cite them.
- **No master table in source**: The source has no ranked/scored comparison table. The recommendations table above is built from the per-section verdicts (what each phase delivers) and the design-principle anchors (why each choice matters).
- **ADR references**: The source cites ADR-010 (append-only / grandfathering invariant) and ADR-011+ (not named explicitly, derived from design doc §03 and §05 text). Citations preserve the spec-document names (03, 05) as the source uses them.
- **No ratings or verdicts to preserve**: The document is prescriptive (recommendations, not evaluation). All "verdicts" (e.g., "Phase 1 is low-risk, high-value, no schema change") are inline descriptors preserved verbatim.
- **Design-principle language**: Phrases like "mirror the jury lifecycle," "consent model," "procedural legitimacy" are the report's own framing. This index uses them unchanged to avoid paraphrase.
- **Future work implicit**: The report assumes v1-rulemaking implementation (Phase 1–5 staged); it does not address post-ship maintenance, federation of rule proposals across instances, or multi-community rulemaking federates. Those are out of scope.

---

**Index verification (Line citation spot-check):**

- L9: "### 1.1 Rules are an append-only version chain, authored top-down" ✓
- L40: "## 2. Design principles for the new surfaces" ✓
- L51: "## 3. Recommended UI/UX for configuring community rules" ✓
- L85: "## 4. Recommended UI/UX for member deliberation and consensus" ✓
- L134: "## 5. Concrete, staged implementation plan" ✓
- L161: "## 6. Pitfalls to avoid (drawn from the project's own constraints)" ✓

All citations verified against source file.
