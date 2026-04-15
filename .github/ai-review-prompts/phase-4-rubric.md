# Phase 4 rubric — First five endpoints (golden path)

**This is the highest-value review phase.** Be strict. Phase 4 delivers the
first end-to-end slice: report → threshold → jury selection → jury vote →
decision → sanction → modlog. The golden-path e2e test is the acceptance gate
for the whole v0 mechanic.

Endpoints in scope (from `IMPLEMENTATION-PLAN-v0.md` §3 Phase 4 and
`05-mvp-and-delivery-plan.md` §2):

1. `POST /api/v4/governance/report` — create_report (api_crud)
2. `GET /api/v4/governance/case/:id` — get_case (api)
3. `GET /api/v4/governance/jury/me` — list_my_jury_queue (api)
4. `POST /api/v4/governance/jury/vote` — submit_jury_vote (api) — **hardest
   single function**
5. `GET /api/v4/governance/modlog` — list_modlog (api)

Plus admin backstops: `POST /api/v4/governance/admin/assign-jury`,
`POST /api/v4/governance/admin/close-case`, and the illegal-content
`POST /api/v4/governance/admin/emergency-remove` per ADR-013.

## Focus areas beyond the ADR rubric

### Hash-chain emission (ADR-008) — the most common failure mode

- **Every governance write emits a `governance_log` row before the HTTP
  response returns.** Includes: report create, case open (threshold cross),
  jury assignment, jury accept/decline, jury vote, case decision, sanction
  create, emergency-remove. Flag any handler that writes DB state (Diesel
  insert/update/delete) without a corresponding `governance_log::append()`
  call in the same transaction.
- **`actor_pseudonym` lookup precedes the log write.** The log's actor field
  must be the pseudonym, never the raw `person_id`, `local_user_id`, or
  username. Flag handlers that pass `person.name` into the payload.
- **`scrub()` is applied to every user-supplied string in the payload.**
  Report rationale, vote justification, sanction reason — all must pass
  through the redaction service before insertion. ADR-015.

### `CaseStatus` exhaustiveness (ADR-013)

- Every `match` on `CaseStatus` handles `EmergencyRemove` explicitly. No
  wildcard `_ =>` arms. The ADR compliance workflow also checks this, but
  the AI review should flag it in prose as well — repeat offenders mean
  phase training is failing.
- The `emergency-remove` admin handler creates the case in `EmergencyRemove`
  status **post-facto** (the content is already removed at the moment the
  case is opened). Flag handlers that remove content based on a jury
  decision flow instead of the dedicated admin path.

### Permission shaping

- `get_case` returns different JSON shapes for public / juror / admin /
  target callers. The handler performs the shaping using the hydrated view
  from `read_case_detail`. Flag handlers that return raw view output to
  public callers, or that leak juror identities to the case target.

### Golden-path integration test

- The `report_to_modlog_golden_path` test under `tests/e2e.rs` runs the full
  flow against a real Postgres. Flag any mock DB. Flag any test that skips
  the final log-chain verification step (the test must re-walk the
  `governance_log` hash chain and assert integrity, mirroring the Phase 1
  `governance_log_hash_chain_holds` pattern).
- **Real users, real jury selection.** The test seeds at least 5 eligible
  jurors via direct DB inserts and lets the handler pick them — flag tests
  that hard-code juror IDs to sidestep the selection algorithm.

### v0 scope lockdown (ADR-010)

- No new handlers beyond the 5 endpoints + 3 admin backstops listed above.
- No Keycloak, OpenFGA, Vault, HSM, KMS, external signer, or blockchain
  anchoring imports — those are v2/v3. Flag any such dependency.
- No TypeScript/frontend changes — v0 is backend + API only. Flag any
  frontend edit.
