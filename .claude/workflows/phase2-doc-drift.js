export const meta = {
  name: 'phase2-doc-drift',
  description: 'Brehon v1-closeout Phase 2 — verify 3 doc-drift claims against code/file ground-truth, adversarially check each proposed fix, return a ranked apply-list',
  whenToUse: 'v1-closeout Phase 2 doc-drift reconcile (scan → verify; apply stays advisor+user)',
  phases: [
    { title: 'Scan', detail: 'one doc-vs-code verifier per drift claim' },
    { title: 'Verify', detail: 'adversarial re-check of each proposed fix against ground-truth' },
  ],
}

// ---- Ground-truth inputs (gathered inline by the advisor before launch) ----
// These are FACTS established pre-flight; agents must still independently verify,
// but they start from the correct paths (not the stale ones in the roadmap/plan).
const REPO = 'C:/Users/barri/Developer/brehon-fork-closeout'

const CLAIMS = [
  {
    key: 'sl-entry-kind-markers',
    title: 'SL ENTRY_KIND registry markers (pending)→(active)',
    prompt: `You are a doc-vs-code drift verifier for the Brehon governance platform (a Lemmy fork).
Repo root: ${REPO}. All reads only — make NO edits.

CLAIM TO VERIFY:
The registry file .claude/rules/governance-log-entry-kind-registry.md annotates four v1-SL-a
entry kinds as "(pending)" in their "Emitting handler" column, but the code allegedly emits them
correctly now, so they should be flipped to "(active)" — mirroring the JM-e precedent at line 140
(ENTRY_KIND_APPEAL_DECIDED → process_appeal_vote → "(active)").

The four kinds + their CLAIMED emitting call sites (from registry rows ~171-174):
  1. ENTRY_KIND_SPONSOR_LIABILITY_PENDING  → v1-SL-d crates/api/api/src/governance/submit_jury_vote.rs::process_vote (Decided->SponsorLiabilityPending transition)
  2. ENTRY_KIND_SPONSOR_LIABILITY_FIRED    → v1-SL-c crates/api/api/src/governance/sponsor_liability_grace.rs::run_grace_check_batch (fire branch)
  3. ENTRY_KIND_SPONSOR_LIABILITY_ESCAPED  → (registry row ~173 — read it for the exact handler claim)
  4. ENTRY_KIND_ENDORSEMENT_REVOKED        → v1-SL-b crates/api/api_crud/src/governance/revoke_endorsement.rs

GROUND-TRUTH PROCEDURE (do ALL of these):
- Read .claude/rules/governance-log-entry-kind-registry.md lines 160-180 to get the exact current
  "(pending)" text for all four rows AND the exact &str values (e.g. "sponsor_liability_pending").
- Read line 138-140 to confirm the JM-e "(active)" precedent pattern verbatim (what exactly does an
  "active" row look like — is it "(active)" suffix, or absence of "(pending)"?).
- For EACH of the four kinds, grep the named handler file (and broaden with
  rg "<ENTRY_KIND_CONST>" crates/ to catch the real emit site if it moved) to determine whether the
  const is actually USED at an emit call site (look for governance_log writes / append calls passing
  that const), not merely declared. A const that is declared but has zero emit call sites is still
  "pending"; a const passed to a log-append/emit call is "active".
- Distinguish: declaration site (const ENTRY_KIND_... = "...";) vs emit site (the const used as an
  argument to a governance-log write). Only emit sites make a kind "active".
- For ESCAPED specifically, the registry row may name a different handler — read the row, don't assume.

RETURN (StructuredOutput) per the schema: for the claim overall, doc_says = current registry text
state, code_says = which of the 4 kinds have real emit sites (with file:line evidence) and which do
not, fix = the precise edit (which rows flip to active, which stay pending, mirroring the line-140
pattern verbatim), confidence 0-1, and evidence = the file:line citations you found.`,
  },
  {
    key: 'rt-r2-retro-path',
    title: 'RT-r2 retro path in roadmap',
    prompt: `You are a doc-vs-code drift verifier for the Brehon governance platform.
Repo root: ${REPO}. All reads only — make NO edits.

CLAIM TO VERIFY:
.claude/PRPs/v1-roadmap.json points the v1-RT-r2 entry's "retro" field at
".claude/PRPs/reports/session-retro-2026-05-25-v1-deps-r1.md" — a deps-r1 session retro, NOT an
RT-r2 retro. The roadmap's own what_remains note says "update if a dedicated RT-r2 retro file is
authored." PRE-FLIGHT FINDING (verify it): a dedicated retro file DOES appear to exist at
.claude/PRPs/reports/v1-RT-r2-retro.md (and v1-RT-r2-verify.md). If so, the drift is real and the
fix is to REPOINT the roadmap field to the real retro file (not merely annotate).

GROUND-TRUTH PROCEDURE:
- Read .claude/PRPs/v1-roadmap.json and find EVERY occurrence where a v1-RT-r2 record's "retro"
  (or "plan"/"verify") field points at the deps-r1 retro. There may be more than one (the
  pre-flight grep saw both line ~235 and line ~543). List each occurrence with its surrounding
  JSON-path context (which object/array it's in) so the apply step edits the right one(s).
- Confirm .claude/PRPs/reports/v1-RT-r2-retro.md exists and is a genuine RT-r2 retro (read its
  first ~30 lines — does it actually cover v1-RT-r2? title/scope check). Same quick check on
  v1-RT-r2-verify.md (is it the verify report for RT-r2?).
- Confirm session-retro-2026-05-25-v1-deps-r1.md exists and IS a deps-r1 retro (so we know the
  current pointer is genuinely mismatched, not just oddly named).
- Determine the correct fix: repoint each mismatched "retro" field to v1-RT-r2-retro.md. Note
  whether a separate "verify" field exists/should point at v1-RT-r2-verify.md.

RETURN (StructuredOutput): doc_says = current roadmap pointer(s) with JSON-path context,
code_says = what files actually exist + whether the dedicated retro is genuine, fix = the precise
repoint edit(s) (old value → new value, one per occurrence), confidence 0-1, evidence = file paths
+ line numbers + the first-line title of each retro file you read.`,
  },
  {
    key: 'data-model-schema-current',
    title: '04-data-model-and-api.md vs live Diesel schema',
    prompt: `You are a doc-vs-code drift verifier for the Brehon governance platform (Lemmy fork).
Repo root: ${REPO}. All reads only — make NO edits.

CLAIM TO VERIFY:
docs/brehon-law-inspired-network/04-data-model-and-api.md is the LIVING, code-derived schema doc
(491 lines). It should reflect ALL merged v1 governance schema. The doc's own header says "CODE
WINS on discrepancy." Verify whether the doc's table/enum/column claims match the live Diesel
source structs.

PRE-FLIGHT FINDING: crates/db_schema/src/schema.rs does NOT exist at that path (Lemmy lays schema
out differently). The live governance source structs are at crates/db_schema/src/source/governance/*.rs
(e.g. moderation_case.rs, endorsement.rs, jury_vote.rs, governance_log.rs, appeal.rs, ...). The
generated Diesel table! macros are likely under crates/db_schema/src/ somewhere else (find it:
rg "diesel::table!" crates/db_schema/src/ -l, or look for a schema module / migrations).

GROUND-TRUTH PROCEDURE (this is a SCAN, not exhaustive — bound it to governance tables):
- Locate the canonical Diesel schema definition(s): rg "table!" crates/db_schema/src/ and/or read
  crates/db_schema/src/lib.rs to see how schema is structured. Identify where the governance table
  column lists actually live.
- List the governance tables present in code (from source/governance/*.rs filenames + their struct
  fields): moderation_case, endorsement, jury_vote, jury_assignment, jury_pool, governance_log,
  public_case_log, appeal, actor_pseudonym, federation_* , redaction, remote_*, etc.
- Read 04-data-model-and-api.md and extract the tables/enums it documents.
- DIFF at the TABLE level first (does the doc list every governance table that exists in code? any
  table in code but absent from doc = drift; any table in doc removed from code = drift). Then for
  2-3 of the highest-signal governance tables (moderation_case + endorsement + jury_vote), spot-check
  COLUMN-level: do the doc's columns/enums match the source struct fields? Report mismatches with
  evidence; do NOT attempt a full 491-line column-by-column audit (out of scope for this bounded sweep
  — flag it as "needs deeper pass" if you find pervasive column drift).
- Because the doc says CODE WINS, every fix is "update the doc to match code."

RETURN (StructuredOutput): doc_says = what the doc claims (table list + any spot-checked columns),
code_says = the live schema (table list + spot-checked struct fields, with file:line), fix = the
specific doc edits needed (table additions/removals + the spot-checked column corrections), OR
"no drift found" if the doc is current, confidence 0-1, evidence = file:line citations. If you find
the doc is broadly stale (many tables/columns off), set fix to a SCOPED recommendation ("defer to a
dedicated schema-audit pass") rather than enumerating hundreds of edits — this Phase 2 sweep is
deliberately small.`,
  },
]

const SCAN_SCHEMA = {
  type: 'object',
  additionalProperties: false,
  required: ['claim_key', 'drift_present', 'doc_says', 'code_says', 'fix', 'confidence', 'evidence'],
  properties: {
    claim_key: { type: 'string', description: 'echo the claim key' },
    drift_present: { type: 'boolean', description: 'true if the doc and code disagree (a fix is needed)' },
    doc_says: { type: 'string', description: 'current documented state (verbatim or close paraphrase)' },
    code_says: { type: 'string', description: 'ground-truth from the code/files, with file:line evidence' },
    fix: { type: 'string', description: 'the precise edit(s) to reconcile doc to code; "none" if no drift' },
    confidence: { type: 'number', description: '0.0-1.0 confidence in the verdict + fix' },
    evidence: { type: 'array', items: { type: 'string' }, description: 'file:line citations supporting the verdict' },
    needs_deeper_pass: { type: 'boolean', description: 'true if drift is broader than this bounded sweep should fix inline' },
  },
}

const VERDICT_SCHEMA = {
  type: 'object',
  additionalProperties: false,
  required: ['claim_key', 'fix_is_correct', 'fix_is_safe', 'reasoning', 'corrected_fix'],
  properties: {
    claim_key: { type: 'string' },
    fix_is_correct: { type: 'boolean', description: 'does the proposed fix actually match code ground-truth?' },
    fix_is_safe: { type: 'boolean', description: 'is it a pure doc/registry/json edit with no crates/ logic change?' },
    reasoning: { type: 'string', description: 'why the fix is/ is not correct + safe, with independent evidence' },
    corrected_fix: { type: 'string', description: 'if the scan fix was wrong/imprecise, the corrected edit; else "fix stands"' },
  },
}

phase('Scan')
const scans = await pipeline(
  CLAIMS,
  (claim) => agent(claim.prompt, { label: `scan:${claim.key}`, phase: 'Scan', schema: SCAN_SCHEMA, agentType: 'Explore' }),
  // Adversarial verify runs per-item as soon as its scan completes (no barrier).
  (scan, claim) => {
    if (!scan) return null
    if (!scan.drift_present) {
      // No drift → nothing to adversarially verify; pass the scan through.
      return { scan, verdict: { claim_key: claim.key, fix_is_correct: true, fix_is_safe: true, reasoning: 'scan found no drift; no fix to verify', corrected_fix: 'none' } }
    }
    const verifyPrompt = `You are an ADVERSARIAL fix-checker for the Brehon governance platform.
Repo root: ${REPO}. Reads only — make NO edits. Your job is to REFUTE the proposed fix if you can.

A first agent verified this doc-drift claim and proposed a fix. Do NOT trust it — independently
re-derive the ground-truth and check whether the fix is (a) CORRECT (matches code) and (b) SAFE
(pure doc/.claude/registry/json edit, zero crates/ logic change — Phase 2 forbids logic change).

CLAIM: ${claim.title}
SCAN VERDICT:
  drift_present: ${scan.drift_present}
  doc_says: ${scan.doc_says}
  code_says: ${scan.code_says}
  proposed fix: ${scan.fix}
  scan evidence: ${(scan.evidence || []).join(' | ')}

Independently verify by re-reading the cited files yourself (do not rely on the scan's quoted
evidence — open the files). Default to fix_is_correct=false if you cannot independently confirm the
code_says claim. Confirm fix_is_safe=false if the fix would touch anything under crates/, migrations/,
tests/, or change code logic (registry .md / roadmap .json / docs/*.md edits are SAFE).
If the fix is imprecise (wrong line, wrong value, would break JSON, would mis-mirror the line-140
pattern), provide the corrected_fix.`
    return agent(verifyPrompt, { label: `verify:${claim.key}`, phase: 'Verify', schema: VERDICT_SCHEMA, agentType: 'Explore' })
      .then((verdict) => ({ scan, verdict }))
  }
)

const results = scans.filter(Boolean)
return {
  phase: 'v1-closeout Phase 2 — doc-drift reconcile',
  scanned: CLAIMS.length,
  results,
}
