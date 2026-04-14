# Agent prompt — produce an MVP implementation plan

Paste the block below into a fresh Claude Code session (or hand to a subagent). The agent will read the relevant docs and write an implementation plan to a new file.

---

```
You are helping a solo developer plan the MVP implementation of a federated
grassroots-organisation governance platform. The platform is a fork of Lemmy
with a Brehon-law-inspired governance layer (sponsorship, reputation, juries,
graduated restorative sanctions, transparent case logs).

The design is already captured. Your job is to read it and produce a concrete,
executable implementation plan for v0 (MVP). You are NOT writing code yet.

## Working directory and repo

Primary working directory: c:\Users\barri\Developer\homeserver

The design docs live at:
  docs/research/brehon-law-inspired-network/

This is an infrastructure-as-code monorepo; the Brehon fork itself does not
exist yet. The implementation plan you produce will be the blueprint for
creating it (likely in a separate repo — flag that decision as a pre-flight
step in your plan).

## Required reading (in this order)

Read these completely before writing anything:

  1. docs/research/brehon-law-inspired-network/00-README.md
     — orientation, reading paths, traceability chain

  2. docs/research/brehon-law-inspired-network/01-vision-and-principles.md
     — the 9 Brehon principles, baseline policy parameters, non-goals

  3. docs/research/brehon-law-inspired-network/02-domain-model.md
     — glossary, actor state machine, case lifecycle, reputation model
     — note the EmergencyRemove case state (§3.1)

  4. docs/research/brehon-law-inspired-network/04-data-model-and-api.md
     — THIS IS YOUR PRIMARY REFERENCE. Tables, enums, Diesel structs, views,
       DTOs, routes, handler responsibilities. Treat the Rust code blocks as
       authoritative starting points.
     — note the ActorPseudonym table (§3)
     — note the EmergencyRemove variant in CaseStatus (§2)

  5. docs/research/brehon-law-inspired-network/05-mvp-and-delivery-plan.md
     — MVP scope (11 endpoints), v0 simplifications, 6-step implementation
       order, Monday-morning checklist, warnings. §7 shows the v1/v2/v3
       roadmap — anything flagged post-MVP is OUT OF SCOPE for your plan.

  6. docs/research/brehon-law-inspired-network/99-decisions-and-open-questions.md
     — all 15 committed ADRs and 12 open questions. The ADRs are hard
       constraints; you must not re-litigate them.

Reference as needed (do not need to read cover-to-cover):

  7. docs/research/brehon-law-inspired-network/03-architecture.md
     — crate layout, plane separation, flow diagrams

  8. docs/research/brehon-law-inspired-network/06-security-and-threat-model.md
     — §2.2.1 emergency-remove, §6.1 GDPR, §7 threat table

  9. docs/research/brehon-law-inspired-network/07-operations-and-federation.md
     — deployment shape (Docker Compose on one host for v0)

The raw chat transcripts (chat1.md, chat2.md) are the original source material
but you should NOT need them — the numbered docs are the distilled, authoritative
version. Do not edit them.

## Hard constraints (do not re-litigate)

From the ADRs in 99-decisions-and-open-questions.md:

- Base platform: fork from Lemmy 1.0-beta (ADR-012) — use the Extism plugin
  system where it simplifies governance hooks
- Licence: AGPLv3 inherited (ADR-011)
- v0 scope: exactly the 11 endpoints in 05 §2; nothing else from the API surface
- v0 simplifications in 05 §3 are mandatory: 5-juror panels, fixed quorum 3,
  simple majority, no diversity constraints, no severity thresholds,
  outbound-only federation, reputation-decay stub, no rule-set versioning,
  no blockchain anchoring (local hash chain only)
- Solo-dev tech stack: NO Keycloak, NO OpenFGA, NO Vault, NO Kubernetes,
  NO external log signer, NO blockchain anchoring. Those are v2/v3 work.
- Auth: Lemmy's existing JWT; optional passkey MFA via `webauthn-rs` crate
- Authz: hardcoded capability checks in Rust reading `reputation_snapshot` flags
- Governance log: sha2 hash chain via Postgres triggers; `rs_merkle` + `ed25519-dalek`
  for the signing layer; key lives in `.env` for v0
- Secrets: env vars / Docker secrets; no secrets manager in v0
- Deployment: single Docker Compose file on one host
- GDPR from day 1: pseudonymised actor IDs in the governance log (ADR-015);
  the `actor_pseudonym` table is mandatory
- Illegal content from day 1: admin emergency-remove path with post-facto
  jury review (ADR-013); `EmergencyRemove` case state is mandatory
- Federation interop: content-level federation with vanilla Lemmy works
  normally; governance signals are new AP types and are fork-only (ADR-014)

## What the plan must contain

Write a markdown implementation plan with these sections:

1. **Executive summary** — one paragraph, what v0 ships and when it's done
2. **Pre-flight** — fork setup, dev environment, repo decisions
   (separate repo vs. subtree in homeserver? flag this for the user)
3. **Phase-by-phase task breakdown**, following the 6 steps in 05 §4:
   - Step 1: Schema + Diesel foundation
   - Step 2: Read models (db_views)
   - Step 3: API common DTOs
   - Step 4: First five endpoints (end-to-end slice)
   - Step 5: Reputation & sponsorship
   - Step 6: Federation objects & activities (outbound-only)
   For each phase, list:
   - Concrete tasks (numbered, small enough to be one commit each)
   - Exact file paths for new files and modified files
   - References back to 04 sections for struct/enum definitions
   - Dependencies on prior phases
   - Definition-of-done (how you know the phase is complete)
4. **Cross-cutting requirements** (apply to every phase):
   - Hash-chain trigger on `public_case_log` and governance log entries
   - `actor_pseudonym` table and redaction-service integration
   - `EmergencyRemove` case state wired through where CaseStatus is touched
   - AGPLv3 `LICENSE` file and source-disclosure notice
5. **Test strategy** — solo-dev appropriate:
   - Integration tests only for the golden path (no unit tests until something
     breaks twice)
   - `tests/e2e.rs` that spins up Postgres in Docker and exercises the full
     report → case → jury → decide → log flow
   - CI: `cargo check --workspace` + `cargo clippy` + the e2e test
6. **Monday-morning checklist** — the first 5 concrete tasks to start coding,
   distilled from 05 §5 with exact file paths
7. **Risks and unknowns** — what could go wrong, what needs verification
   (e.g. Lemmy 1.0-beta API stability, Extism hook surface for governance)
8. **Open questions to escalate** — any questions from 99 (OQ-001 through
   OQ-012) that genuinely block Step 1 and must be resolved before coding
   starts. Do not list open questions that can be deferred.
9. **Estimated effort** — rough, optional; phase-by-phase order-of-magnitude
   (days / weeks, not hours). Solo-dev context.

## What NOT to do

- Do NOT write any Rust code. This is a plan, not an implementation.
- Do NOT re-litigate any ADR in 99. If you disagree with one, add it to §8
  "open questions to escalate" — do not contradict it in the plan body.
- Do NOT add v1, v2, or v3 scope to the MVP. If something is tempting but
  post-MVP, list it as "deferred to v1" and move on.
- Do NOT invent new tech choices. The stack is frozen at the solo-dev level.
- Do NOT produce UX or frontend work — not in scope.
- Do NOT create new numbered docs (01-xx). The suite is complete.
- Do NOT edit chat1.md, chat2.md, or any of the 00-99 numbered docs.

## Deliverable

Write the plan to a new file:

  docs/research/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md

Use markdown links in the form [04 §3](04-data-model-and-api.md) when
referencing source sections so the plan is navigable from the docs directory.

Once the plan file is written, report back with:
- Total task count across all phases
- Top 3 risks
- Any OQs from 99 that actually block starting (not just "should resolve
  eventually")

Do not start coding. Do not modify the numbered docs. Plan only.
```

---

## Notes on using this prompt

- **Fresh session recommended.** The agent starts cold and reads only what
  the prompt points it at — that's the most reliable way to get a plan that
  actually respects the ADRs rather than one that drifts on session context.
- **Expect the agent to want to explore.** It will use Glob/Grep/Read tools
  across the doc suite. That's fine and expected.
- **Review the output before trusting it.** The plan is a starting point;
  cross-check the file paths against [04](04-data-model-and-api.md) and make
  sure no v1 scope has leaked in.
- **If you want a subagent instead**, spawn via `Agent` with
  `subagent_type: "Plan"`, put this whole block in the `prompt` field, and
  let it run. The output will come back as a tool result rather than a
  written file, unless you explicitly ask it to write one (which this
  prompt does).
