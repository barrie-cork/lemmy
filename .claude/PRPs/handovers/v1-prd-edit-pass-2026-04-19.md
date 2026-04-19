# Handover — v1 PRD Edit Pass (session 2026-04-19 afternoon)

**Session predecessor:** `v1-prep-2026-04-19.md` (morning) created 5 DRAFT v1 sub-PRDs + triage.
**This session's work:** PRD coherence audit → 6 B-decisions resolved → edit pass started.
**Status at handover:** **B1 + B2 edits DONE** (admin-dashboard + jury-mechanics); B3–B6 + N1–N5 + NOT1/NOT3/NOT4/NOT5 PENDING.
**Context budget:** stopped at user's direction due to high context usage.

---

## Critical guardrails (re-read before resuming)

The Phase 6 session is actively implementing on `phase-6` branch in an advisor worktree. These were flagged mid-session 2026-04-19:

1. **Do NOT write to `.claude/decision-queue.json`** — that's Phase 6's advisor/impl channel. This session's findings live at `.claude/PRPs/v1-planning-queue.json` (different file, distinct schema).
2. **Do NOT touch `phase-6` branch** or any worktree at `brehon-fork-advisor-phase6` / `brehon-fork-agent-*-phase6`.
3. **Do NOT write `resolved_by: "advisor"`** in ANY queue file. Only the advisor session uses that label. This session uses `resolved_by: "user"` or `resolved_by: "v1-planning-session"`.
4. **Do NOT edit homeserver memory files** for Phase 6 (`project_brehon_phase_6_notes.md`, `project_brehon_governance_platform.md`, etc.).
5. **Planning-only scope** — no `crates/` edits, no migrations, no Cargo.toml changes, no `cargo build`/`cargo check` from primary worktree while Phase 6 agents run.
6. **4 unpushed `wave1-*` branches exist locally** from earlier in this session (wave1-cr-mech, wave1-cr-probes, wave1-cr-db-index, wave1-filter-dto). Leave them where they are (no push, no delete). They'll activate when Wave 1 launches post-Phase-6.

Full guardrail text: see user message at session ~start ("V1 planning work can proceed in parallel with Phase 6 implementation, but three guardrails apply...").

---

## What got done this session

### 1. Audit complete

Read all 5 PRDs end-to-end. Cross-PRD coherence audit produced 15 findings:

- **6 blocking** (B1–B6): cross-PRD contradictions/ownership gaps
- **5 non-blocking** (N1–N5): within-PRD drift
- **4 noted** (NOT1, NOT3, NOT4, NOT5) + NOT2 (governance_log entry_kind registry, deferred to v1-impl prep)

All captured in `.claude/PRPs/v1-planning-queue.json` with schema distinct from Phase 6's decision-queue. Read this file first on resume.

### 2. All 6 B-decisions resolved (user-authorised)

| # | Decision | Option picked | Key outcome |
|---|---|---|---|
| **B1** | `appeal.*` namespace ownership | **A** — jury-mechanics owns fully | admin-dashboard delegates; `appeal.original_jurors_excluded` = hard code rule, not config |
| **B2** | `jury.panel_size` shape | **B** — dotted matrix + single-key fallback | OQ-026 cascade operationalised; v0 read sites preserved |
| **B3** | Reputation-tuning namespaces | **A-modified** (user adjustment) | 10 keys renamed; `bounds.*` + `feature.*` new 1st-class; `cron.*` collapsed into `job.*`/`participation.*`/`deltas.*`; 15 namespaces total |
| **B4** | sponsor-liability `liability.*` | **A** — flat matches v0 | 10 keys renamed from `sponsor.liability.*` to `liability.*` |
| **B5** | Phase 6 dependency on federation-inbound | **A-modified** — hard gate + SQL DO block preamble; timestamp invariant only ("must sort after Phase 6"), no specific date | Fails-loud if migration runs before Phase 6 lands |
| **B6** | `submit_jury_vote` ownership | **A** — jury-mechanics owns combined 9-step handler | User provided full pseudocode; sponsor-liability §9.3 reduces to pointer |

Full context + rationale + edit-scope-per-decision: see queue file `resolved` array.

### 3. Edits completed — B1 + B2

**admin-dashboard (`v1-admin-dashboard.prd.md`) — DONE for B1/B2:**
- §3.1 `appeal.*` row updated (5 v1 additions, jury-mechanics-owned annotation)
- §3.5 cascade sentence added (B2 reader-cascade order)
- §5.1 `jury.panel_size = 7` row kept with cascade-precedence note
- §5.2 3 `appeal.*` rows deleted + cross-ref line
- §10.2 issue #15 row annotated (ownership shift to jury-mechanics)
- §11 "Resolutions applied (2026-04-19)" added at end of file — tracks B1/B2 done, B3/B4/B5/N1 pending

**jury-mechanics (`v1-jury-mechanics.prd.md`) — DONE for B1/B2:**
- §3.5 new "Cascade fallback" subsection added
- §9.2 directive to use `config::get_int_cascade` helper (not direct dotted-key read)
- §15 "Resolutions applied (2026-04-19)" added at end of file — tracks B1/B2 done, B6 pending

### 4. v1-planning-queue.json created

`.claude/PRPs/v1-planning-queue.json` — distinct schema from Phase 6's queue. Contains:
- 6 resolved entries (B1–B6 with full resolution details + edit_scope per PRD + key_renames tables for B3/B4)
- 5 non-blocking entries (N1–N5 with proposed resolutions flagged)
- 5 noted entries (NOT1/NOT2/NOT3/NOT4/NOT5 with proposed resolutions or defer decisions)

Resumable state for the rest of the edit pass.

---

## What's PENDING — edits still to apply

**Read this from the queue file.** `pending` fields are explicit per-PRD edit_scope. Below is the TaskList-level summary of remaining work:

### TaskList on resume

```
#11 [pending] B3 edits: admin-dashboard + reputation-tuning
#12 [pending] B4 edits: sponsor-liability + admin-dashboard
#13 [pending] B5 edits: federation-inbound + admin-dashboard
#14 [pending] B6 edits: jury-mechanics + sponsor-liability
#15 [pending] Apply 🟡 non-blocking fixes (N1–N5)
#16 [pending] Apply 🟢 noted fixes (NOT1, NOT3, NOT4, NOT5)
```

(Tasks #1–#10 are completed — review, audit, summary, gap-check, queue, B1+B2.)

### B3 (task #11) — namespace collapse

**Files:** `v1-admin-dashboard.prd.md`, `v1-reputation-tuning.prd.md`

**reputation-tuning — 10 key renames per this table (from queue `B3.key_renames`):**

| Old | New |
|---|---|
| `cron.participation.weekly_increment` | `deltas.participation_weekly_active` |
| `cron.participation.activity_threshold_comments` | `participation.activity_threshold_comments` |
| `cron.participation.lookback_days` | `participation.lookback_days` |
| `cron.participation.weekly_dormancy_decrement` | `deltas.participation_dormant` |
| `cron.participation.dormancy_window_days` | `participation.dormancy_window_days` |
| `cron.participation_interval_days` | `job.participation_interval_days` |
| `cron.rollup_interval_days` | `job.rollup_interval_days` |
| `rollup.equal_weights` | `job.rollup_equal_weights` |
| `reputation.v1.decay_enabled` | `feature.reputation_v1_decay_enabled` |

**Sites to edit in reputation-tuning:**
- §5.2 table (new keys listed)
- §5.3 body (Source 1/2/3/4 sub-sections reference these keys)
- §8 defaults matrix (full enumeration)
- §13 cross-references
- Add §14 "Resolutions applied (2026-04-19)" with B3 entry

**admin-dashboard — §3.1 namespace table overhaul:**
- Prose: "62 keys across 7 namespaces" → "138 keys across 15 namespaces (v1 inventory crossing all 5 sub-PRDs)" with per-PRD contribution sub-table
- New rows: `bounds.*` (0 v0, 8 v1), `feature.*` (0 v0, 1 v1)
- Extend existing rows: `participation.*` (+2), `deltas.*` (+3), `job.*` (+3)
- Update §11 Resolutions: mark B3 done

### B4 (task #12) — sponsor-liability `sponsor.*` → flat `liability.*`

**Files:** `v1-sponsor-liability.prd.md`, `v1-admin-dashboard.prd.md`

**sponsor-liability — 10 key renames per queue `B4.key_renames`:**
All `sponsor.liability.grace_window.<thing>_hours` → `liability.grace_window_<thing>_hours`
All `sponsor.liability.<thing>` → `liability.<thing>`

**Sites to edit in sponsor-liability:**
- §4.2 table (6 grace-window key rows)
- §5.1 body (prose mentions sponsor.liability.* prefix — purge throughout §5)
- §7.3 table (2 restoration key rows)
- §9.3 body (~5 prose references — NB: §9.3 will be reduced to pointer in B6 edit anyway; coordinate)
- §10 full defaults matrix (10 entries)
- §12.1 rate-limit key row
- §13 OQ-V1-SL-01 context mentions the multi_sponsor key
- §16 Decisions Log "grace-window severity source" row
- Add §18 "Resolutions applied (2026-04-19)" with B4 entry

**admin-dashboard:**
- §3.1 `liability.*` row: v0 keys: 3, v1 additions: 2 → **v1 additions: 10**
- §5.2: add 7 new rows for sponsor-liability-owned keys NOT yet listed (restoration escapes x2, multi-sponsor rule x1, revoke rate-limit x1, grace_window minimum/maximum/alert_threshold x3)
- §11 Resolutions: mark B4 done; note partial N1 collateral

### B5 (task #13) — federation-inbound Phase 6 hard gate

**Files:** `v1-federation-inbound.prd.md`, `v1-admin-dashboard.prd.md`

**federation-inbound — insert new §8.0 "Dependencies" subsection before §8.1:**

Content: explicit preconditions list per queue `B5.edit_scope.federation-inbound`:
- Phase 6 PR merged to `governance-v0`
- `federation_attestation` table with Phase 6's columns exists
- `remote_sanction_notice` table with Phase 6's columns (esp. `received_at` per DQ-6.1) exists
- Phase 6 enum types (`attestation_type_enum`, `sanction_action_enum`, `sanction_scope_enum`) registered
- Phase 6 entry_kind constants (`federation_sanction_received`, `federation_attestation_received`) present in `governance_log.rs`
- Schema assumption: `remote_sanction_notice.source_instance` is TEXT (peer domain), NOT FK to `instance.id` — v1 joins via `JOIN instance ON instance.domain = source_instance`

**§8.2 SQL preamble DO block:**
```sql
-- Verify Phase 6 tables present before ALTER
DO $$
BEGIN
  IF NOT EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = 'federation_attestation') THEN
    RAISE EXCEPTION 'federation-inbound-v1 requires Phase 6 federation_attestation table';
  END IF;
  IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'remote_sanction_notice' AND column_name = 'received_at') THEN
    RAISE EXCEPTION 'federation-inbound-v1 assumes Phase 6 DQ-6.1 resolved with received_at column';
  END IF;
END $$;
```

**§15.2 DQ-6.1/DQ-6.2 paragraphs:** rewrite prescriptive — reference §8.0 as canonical preconditions list.

**§8 heading rename:** "Database & Migration Changes" → "Dependencies, Database & Migration Changes"

**§1 frontmatter:** scheduling line → "**Hard-gated on Phase 6 PR merge to `governance-v0`**; no implementation before gate satisfied. Plan-level review (this PRD) can proceed in parallel."

**Migration timestamp:** use invariant only — "must sort after Phase 6's `add_federation_attestations`". Do NOT reserve `2026-04-22-…`; leave exact slot for impl time.

**Add §16 "Resolutions applied (2026-04-19)"** — B5 entry with partial N5 collateral note.

**admin-dashboard:**
- §10.1 cross-references — add one line under federation-inbound consumer row: "depends on Phase 6 merge — see `v1-federation-inbound.prd.md` §8.0 for hard-gate conditions."
- §11 Resolutions: mark B5 done.

### B6 (task #14) — combined `submit_jury_vote` handler

**Files:** `v1-jury-mechanics.prd.md`, `v1-sponsor-liability.prd.md`

**jury-mechanics §9.1 — expand to full 9-step pseudocode** (user-provided 2026-04-19):

```
submit_jury_vote(vote, context):
  1. Load case row (FOR UPDATE per v0 concurrency guard)
  2. Validate juror is assigned + hasn't already voted + case is in InReview
  3. Insert jury_vote row
  4. Tally: read all jury_vote rows for this case
  5. Threshold check (v1 new):
     - quorum = case.quorum_snapshot
     - threshold = case.threshold_count_snapshot
     - if any decision has >= threshold votes: winning_decision = that decision; continue to step 6
     - if all jurors voted + no threshold met (deadlock):
         → flip status to CaseStatus::AdminReview; emit governance_log; return case_decided: false
     - if not all voted + no threshold met yet:
         → return case_decided: false (case stays InReview)
  6. Winning decision reached. Insert sanction rows per winning_decision.
  7. Sponsor-liability branch (sponsor-liability §9.3):
     - if sanction has liability implications AND target has active sponsors:
         - compute_sponsor_liability(...) → deltas
         - grace_expires_at = now + grace_window_for_severity(...)
         - update case: status = SponsorLiabilityPending, grace_expires_at = ..., decided_at = now
         - emit sponsor_liability_pending governance_log
         - notify_sponsor_of_pending_liability(...) for each delta
         - return case_decided: true, decision = winning_decision
           (public_case_log + juror reputation_events DEFERRED to scheduler fire/escape)
  8. No-sponsor / NoAction path (v0 lifecycle preserved):
     - update case: status = Decided, decided_at = now, closed_at = now
     - write public_case_log entry
     - write juror reputation_events (aligned/outlier)
     - write reporter reputation_event (upheld/dismissed)
     - emit case_decided governance_log
     - return case_decided: true, decision = winning_decision
  9. Appeal window bound (v1 new — appeal.window_days):
     - set appeal_window_expires_at = decided_at + config.appeal.window_days * 1 day
     - (fires on BOTH SponsorLiabilityPending and Decided paths — composes with sponsor
        grace window per sponsor-liability §6.5 min(...) semantics)
```

**Also add:**
- Test-strategy sub-note: "v0 `report_to_modlog_golden_path` uses unsponsored target → step 8 no-sponsor path → test assertions preserved. New sponsored-target test is a v1-impl follow-up."
- Cross-reference: "SponsorLiabilityPending branch specified by sponsor-liability-v1 §9.3; this section owns the integrated lifecycle shape."
- Mark §15 B6 row as done.

**sponsor-liability §9.3 — reduce from ~50 lines Rust pseudocode to ~10-line pointer:**

Text: "For the combined post-v1 `submit_jury_vote` handler shape, see jury-mechanics-v1 §9.1 (9-step pseudocode). This section's branch semantics contribute: (a) the `SponsorLiabilityPending` status transition with grace-window computation; (b) the deferral of `public_case_log_appended` + juror `reputation_event` writes to scheduler fire/escape time; (c) the `apply_sponsor_liability` → `compute_sponsor_liability` + `fire_sponsor_liability` compute/fire split documented in §9.1 of THIS PRD."

Keep §9.1 (compute/fire split) and §9.4 (scheduler grace-window helper module) intact — those are sponsor-liability's concerns.

**Add §18 "Resolutions applied (2026-04-19)"** with B6 entry.

### Task #15 — 🟡 non-blocking (N1–N5)

Per queue `non_blocking` array. Each has `proposed_resolution` field — use those.

- **N1** (admin-dashboard §5.2 phantom count): drop "4 reserved-for-v1.5 keys" prose from §5.2 count line. Collateral with B4 edit pass.
- **N2** (reputation-tuning §5.3 evidence-cited threshold): add placeholder `256 chars` + flag as v1-impl verification task for `jury.rationale` column existence.
- **N3** (jury-mechanics §5.3 constraint-relaxation algorithm): rewrite flowchart per queue N3 Option A — cooldown at pool-build filter, sponsor-cluster at panel-sample re-roll, geographic as soft score. Relaxation cascade: drop cooldown first (expand pool), then geographic scoring, then sponsor-cluster as last resort with warning log.
- **N4** (sponsor-liability §11.4 modlog reader regression): add paragraph documenting `public_case_log` fires at scheduler fire/escape time (not vote-tally) for liability cases; v1 clients should poll both case endpoint AND modlog.
- **N5**: already collateraled in B5 §8.0 — enumerate in the Resolutions log as resolved-via-B5.

### Task #16 — 🟢 noted (NOT1, NOT3, NOT4, NOT5)

- **NOT1** (OQ renumbering): rename admin-dashboard OQ-027..031 → OQ-V1-AD-01..05; jury-mechanics OQ-027..032 → OQ-V1-JM-01..06. Per queue `NOT1.renames`. Update cross-refs in each PRD's §Open Questions and §Resolutions.
- **NOT2** (governance_log entry_kind registry): DEFER to v1-impl prep. No PRD edit this session.
- **NOT3** (reputation-tuning ADR-010 compliance via feature flag): add paragraph to §9 framing feature-flag posture as weaker than snapshot-column posture but acceptable for reputation (rolling derived view, not in-flight procedural invariant).
- **NOT4** (admin-dashboard §3.2 ConfigKeyMetadata ambiguity): commit explicitly to compile-time — `&'static [ConfigKeyMetadata]` array. §6.1 reference updated.
- **NOT5** (admin-config-write.sh deprecation gating): §8.4 rewrite — deprecation gated on OQ-018 HTTP endpoint shipping, not on v1.1 milestone alone.

---

## Files on disk at handover

### Edited this session (B1+B2 only)
- `.claude/PRPs/prds/v1-admin-dashboard.prd.md` — 5 edits applied, §11 Resolutions section added
- `.claude/PRPs/prds/v1-jury-mechanics.prd.md` — 3 edits applied, §15 Resolutions section added

### Created this session
- `.claude/PRPs/v1-planning-queue.json` — full audit findings + B-decisions + pending edits (~400 lines)
- `.claude/PRPs/handovers/v1-prd-edit-pass-2026-04-19.md` — this file

### Untouched this session (edits pending)
- `.claude/PRPs/prds/v1-reputation-tuning.prd.md` — B3 pending
- `.claude/PRPs/prds/v1-sponsor-liability.prd.md` — B4 + B6 pending
- `.claude/PRPs/prds/v1-federation-inbound.prd.md` — B5 pending

### Git state
- Branch: `governance-v0` (local 3 commits behind origin — intentionally not fast-forwarded to avoid racing Phase 6)
- Working tree: 2 modified PRD files (admin-dashboard, jury-mechanics) + 2 new files (queue + this handover) + 4 unpushed local branches (`wave1-*` — leave alone)
- Nothing staged. Nothing committed this session.

---

## Resume plan (next session)

**Bootstrap:**
```
Read .claude/PRPs/handovers/v1-prd-edit-pass-2026-04-19.md
Read .claude/PRPs/v1-planning-queue.json (full — all B-decisions + pending fields carry edit instructions)
Read the Resolutions-applied section at the end of .claude/PRPs/prds/v1-admin-dashboard.prd.md and .claude/PRPs/prds/v1-jury-mechanics.prd.md to see what's done
```

**Sequencing recommendation (same as current TaskList #11–#16):**
1. **B3** (reputation-tuning renames + admin-dashboard §3.1 registry) — largest single edit; takes admin-dashboard §3.1 to its authoritative 15-namespace state
2. **B4** (sponsor-liability flat `liability.*` + admin-dashboard §5.2 additions) — coordinates with B6 since both touch sponsor-liability §9.3 prose
3. **B5** (federation-inbound §8.0 gate + admin-dashboard §10.1 cross-ref) — self-contained, minimal coordination
4. **B6** (jury-mechanics §9.1 expansion + sponsor-liability §9.3 reduction) — DO B4 BEFORE B6 so §9.3 rename lands before §9.3 reduction
5. **N1–N5** — small edits, mostly paragraph additions; can batch with the B-edit pass they collateral with (N1 with B4, N5 already in B5)
6. **NOT1, NOT3–NOT5** — OQ renumbering is mechanical find-replace; paragraph additions elsewhere

**Estimated context budget:** ~60k tokens if done in-session without subagents. Consider spawning Explore or general-purpose agents for mechanical find-replace work (OQ renumbering) to protect main context.

**Verification at end:** cross-check each PRD file has a "Resolutions applied (2026-04-19)" section listing every B/N/NOT item touching it. That's the completion signal.

**Do NOT commit or push** — PRDs stay DRAFT-status on `governance-v0`. User decides when to commit.

---

## Open questions for next session to surface

None blocking. All 6 B-decisions resolved; edits are mechanical application.

If ambiguity arises during a specific edit, check the queue file's `edit_scope` field for that ID — it's the canonical instruction. If still ambiguous, ask the user; do not guess.

---

*Generated: 2026-04-19 end-of-session. Trust but verify: the decisions in `v1-planning-queue.json` are user-authorised and correct; the edits I applied to admin-dashboard + jury-mechanics reflect them. Edits I did NOT apply (B3/B4/B5/B6/N1–5/NOT1/3/4/5) are pending per the pending-tasks section above.*
