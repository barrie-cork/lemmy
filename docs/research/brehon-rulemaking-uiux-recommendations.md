# Brehon Consensus — UI/UX recommendations for community rule configuration and consensus deliberation

Prepared from a read of the private `barrie-cork/lemmy` fork (governance crates, rule-set model, admin config handlers, the maud admin dashboard, and the project's own `expert-review-suite` specs). All file references below are real paths in that repository.

---

## 1. What exists today (grounded in the code)

### 1.1 Rules are an append-only version chain, authored top-down

A community's rules live in `rule_set_version` — an append-only table with a monotonically increasing `version`, a `parent_id` back-pointer, the `rule_text`, and a `text_sha256` over the canonical UTF-8 bytes (`crates/db_schema/src/source/governance/rule_set_version.rs`; migration `migrations/2026-04-22-000000-0000_add_rule_set_versions/up.sql`). The insert form deliberately does **not** derive `AsChangeset` — past rule text is immutable so in-flight juries stay grandfathered against `moderation_case.rule_set_version_id` (ADR-010 invariant, documented inline).

Creation flows through `admin_create_rule_set` (`crates/api/api/src/governance/admin_rule_sets.rs`). The capability gate is **community moderator OR instance admin** — nothing else. A single privileged write atomically (1) inserts the new version, (2) flips `rule_set.active_version_id` in `governance_config`, and (3) appends a signed `rule_set_version_created` entry to the hash-chained `governance_log`. There is no member-facing path into this function.

### 1.2 Config has a genuinely good UX primitive already: dry-run impact preview

`admin_set_config` (`crates/api/api/src/governance/admin_config.rs`) computes a **downstream impact preview** before any write, via `compute_downstream_impact`. For example, changing a reputation threshold returns `{current_eligible, proposed_eligible, delta}` by counting raw columns on `reputation_snapshot`; changing `jury.panel_size` returns `open_cases_needing_reassembly`; changing `report.case_threshold_micros` returns how many cases would open or stay closed. With `dry_run = true` the handler returns the preview and writes nothing. This "show me the blast radius before I commit" pattern is the single most reusable UX asset in the codebase and should anchor the new rulemaking UI.

Config metadata also already carries the hooks for procedural friction: `requires_step_up`, `requires_re_jury`, `apply_at_default` (`immediate` / `next_jury_cycle` / `next_snapshot_job` / `on_restart`), `valid_range`, and `valid_enum` per key (`build_config_entry`). These are surfaced over `GET /api/v4/governance/admin/config`.

### 1.3 The only "UI" is a server-rendered admin dashboard

`admin_dashboard_html.rs` renders a read-only maud page (active cases, recent config changes) plus an `EventSource` live audit tail. It is `is_admin`-gated and feature-flagged on `governance.dashboard.html_pages_enabled`. There is **no member-facing governance UI** at all — rule configuration today is effectively an admin/operator surface.

### 1.4 The decisive gap: no proposal / deliberation / consensus primitive exists

A full-tree search for `propos|delibera|poll|petition|amend|second|quorum|ratif` outside `.claude/` and `reports/` returns nothing in source. The API surface (`crates/api/api_common/src/governance.rs`) covers reports, cases, jury votes, assignments, appeals, emergency removal, config, and rule-sets — but there is no concept of a *proposed* rule that members discuss and converge on before it becomes a `rule_set_version`. Today the lifecycle is binary: a moderator decides, then writes version N+1.

### 1.5 The specs already ask for exactly this — and constrain how

The project's own design docs make the requirement explicit and bound the solution:

- `03-customization-and-rulemaking.md` separates a **policy layer** (numeric/enum settings — jury size, quorum, thresholds, appeal windows) from a **community rule-text layer** (rules, interpretations, *constitutional amendments*). It states the platform is "pluralist in policy, but not relativist in core safeguards," lists changes that must **not** be casually customizable (removing appeals, removing logging, unrestricted admin override), and in §6 ("Governance of customization itself") requires that important changes may need **step-up auth, multiple approvers, a delay before taking effect**, and that "communities should be able to distinguish between ordinary moderation and constitutional revision."
- `05-user-interaction-and-procedure.md` §3.4 defines the juror's procedural experience (notification → accept/decline → conflict checks → evidence → rule statement → decision → rationale) and §5.1 lists "rule-set editing" as an administrator interface that "should also expose the extra procedural burden attached to those powers." §4 sets the tone: governance is not arbitrary, jury service is civic work not "gamified point-collecting," decisions are procedural.

So the target is not "add a voting widget." It is: build a **rulemaking lifecycle** that mirrors the jury lifecycle's procedural legitimacy, distinguishes ordinary tuning from constitutional change, and reuses the impact-preview, step-up, and audit-log machinery already in the code.

---

## 2. Design principles for the new surfaces

1. **Mirror the case lifecycle, don't invent a new metaphor.** Members already (will) understand report → case → jury → decision → appeal. A rule change should feel like the same civic process applied to the rulebook: proposal → deliberation → consensus check → ratification → versioned enactment → (optional) sunset/review. This satisfies the §4 norm "jury service is civic work" and keeps one mental model.
2. **Two tiers, visibly distinct.** Per `03 §6`, separate **ordinary policy tuning** (a config-key change inside `valid_range`) from **constitutional revision** (rule-text amendment, or touching a key flagged foundational). Different friction, different colour, different quorum, different approver count. Never let them look the same.
3. **Show the blast radius before the vote, not after.** Reuse `compute_downstream_impact` so a proposal page renders "this change would make 41 fewer members jury-eligible" or "3 open cases would need re-paneling" *inline*, the moment a value is proposed. This is the highest-leverage UX move available and it already exists server-side.
4. **Consensus is a process with a visible threshold, not a 51% button.** Brehon's philosophy is anti-capture and procedural. Express agreement as a *consent/objection* model (see §4) with an explicit, configurable bar, a deliberation window, and a standing-down period for objections — not a raw majority tally that can be brigaded.
5. **Everything writes back to the governance log.** The proposal, every endorsement/objection, the impact snapshot at vote time, and the final enactment must append signed `governance_log` entries, exactly as `admin_create_rule_set` does today. The audit live-tail then covers rulemaking for free.
6. **Member-facing first, admin-gated only where the spec demands.** Today rule config is admin-only. The new deliberation surface must be a member surface; admins retain emergency/override powers but, per `05 §5.1`, the UI must "expose the extra procedural burden" when they use them.

---

## 3. Recommended UI/UX for configuring community rules

### 3.1 A "Community Rulebook" page (member-readable, mod-editable)

Replace the implicit "rule text is a blob only admins see" model with a first-class **Rulebook** view per community:

- **Rendered current version** at top, with a permanent version badge (`v{N}`, short `text_sha256` prefix, enacted date, enacting actor pseudonym). The hash is already computed and stored — surface it as a verifiability affordance ("this is the exact text in force").
- **Version timeline / diff viewer.** Because `rule_set_version` is an append-only `parent_id` chain, you can render a git-style history. Show a side-by-side or inline diff between any two versions. This directly serves `03 §3` ("a fair system should not judge a person under one rule and later quietly rewrite the operative rule text") and is trivial to build on existing data — `admin_list_rule_sets` already returns all versions ordered `version DESC` plus the active id.
- **"Rule in force at decision time" link from every case.** Cases pin `moderation_case.rule_set_version_id`; render that as a deep link so an observer (per `05 §3.7`) can see the exact rule version a person was judged under, not the current one.

### 3.2 Structured rules instead of one free-text blob (recommended evolution)

`rule_text` is currently a single `TEXT` column (max `rule_set.text_max_bytes`, default 64 KiB). For configuration UX, evolve toward a structured-but-still-hashable representation:

- Store rules as an ordered list of **rule items** (id, title, body, category, severity tier) serialized canonically (e.g. deterministic JSON or a fenced block format) so a single `text_sha256` still covers the whole set and the append-only/grandfathering invariant is preserved untouched.
- This unlocks per-rule editing, per-rule linking from a case ("alleged breach of Rule 4"), per-rule deliberation, and per-rule severity mapping into the existing sanction-kind layer (`sanction_kind_map.rs`) — without breaking ADR-010.
- Keep a "render to canonical text + hash" step so the verifiability story (`text_sha256`) is identical to today.

### 3.3 A guided rule editor that reuses config metadata

When a change touches the **policy layer** (a `governance_config` key, not rule text), the editor should be generated from the metadata the API already exposes:

- Render the right control per `value_type` (int/float/bool/enum), enforce `valid_range` / `valid_enum` client-side, and show the key's `description` and `doc_anchor` inline.
- Show `apply_at_default` ("takes effect: next jury cycle"), and badge `requires_re_jury` and `requires_step_up` so the consequence is visible *before* submission.
- Call the existing `dry_run=true` path on every value change and render the returned `downstream_impact` as a live "impact panel." This is the config UX equivalent of the diff viewer.

### 3.4 Make the two tiers visually and procedurally distinct

- **Ordinary tuning** (in-range policy change, no foundational flag): lighter chrome, shorter deliberation window, lower consent bar, single moderator can enact after the consent window if no sustained objection.
- **Constitutional revision** (rule-text amendment, or any key you tag `foundational` — extend `ConfigKeyMetadata` with a `tier` field): distinct colour/banner ("Constitutional change"), mandatory longer deliberation window, higher consent threshold, **multiple approvers**, mandatory delay-before-effect (`apply_at` forced away from `immediate`), and step-up auth enforced. This is the concrete UI realization of `03 §6` and §5's "what should not be casually customized."
- Surface a permanent legend so members can tell at a glance which kind of change they're looking at — the spec explicitly wants members to "distinguish between ordinary moderation and constitutional revision."

---

## 4. Recommended UI/UX for member deliberation and consensus

This is the missing primitive. Model it as a **Rule Proposal lifecycle** that parallels the case lifecycle.

### 4.1 Proposal lifecycle (new state machine)

```
Draft → Discussion → Consent check → (Ratified → Scheduled → Enacted=new rule_set_version)
                          │
                          └→ Objected/Revise → back to Discussion   or   Withdrawn/Failed
```

Persist this as a new `rule_proposal` aggregate (proposal, target = rule-text amendment or config-key change, author pseudonym, tier, deliberation window, consent threshold, status) plus child tables for `proposal_comment`, `proposal_position` (consent / consent-with-reservation / stand-aside / object), and a captured `impact_snapshot`. On ratification, the enactment step calls the *existing* `admin_create_rule_set` / `admin_set_config` machinery — the proposal layer sits **in front of** the privileged write, it does not replace the append-only invariant.

### 4.2 Use a consent/objection model, not raw majority

Brehon's anti-capture philosophy fits **consent-based decision-making** better than majority voting. For each proposal, a member can register one of:

- **Consent** — "I can live with this."
- **Consent with reservation** — agree, with a noted concern (captured, surfaced, non-blocking).
- **Stand aside** — "not for me, but I won't block."
- **Object** — a *reasoned* block. Objection requires a written rationale (mirrors the juror "rationale submission" step in `05 §3.4`) and must be addressable.

Enactment requires meeting a configurable bar (e.g. quorum reached AND consent ratio ≥ threshold AND all objections either resolved or formally overridden by the higher-tier process). Expose `quorum` and the consent threshold as policy-layer config keys (the spec's §2.2 already lists `quorum` as a policy setting), so each community tunes its own bar — pluralism in policy.

Weight positions by **standing**, not by a single score. The system deliberately has no single reputation number (`03 §2.1`, "no single reputation score"); positions can be qualified by the relevant standing dimension (e.g. jury-eligibility for jury-mechanics proposals) and displayed transparently, but never collapsed into one capturable metric.

### 4.3 The deliberation surface itself

- **Threaded, structured discussion** anchored to the specific proposed change (and, with structured rules from §3.2, to the specific rule item). Show the proposed diff at the top of every discussion view so debate stays anchored to concrete text.
- **Live tally with the bar shown**, not just counts: a consent meter ("12 consent / 2 reservations / 1 stand-aside / 1 objection — threshold: 75% consent + 0 unresolved objections; quorum 10 ✓"). Always render *what is required*, per the §4.1 norm "people should be able to follow the process."
- **Objection register.** Each open objection is a visible work-item with its rationale and a thread; a proposal cannot ratify while an objection is open. This makes the block meaningful and discourages drive-by vetoes (the objector must reason, exactly as a juror must).
- **Inline impact panel** (from `compute_downstream_impact`) pinned beside the tally for policy-layer proposals.
- **Deliberation timer** showing minimum-window remaining; constitutional proposals get a longer mandated window. No ratification before the window closes.
- **Provenance & audit drawer**: every position and comment shows the actor pseudonym and links to the `governance_log` entry, consistent with how the rest of the system logs.

### 4.4 Channel split per the spec (Lemmy vs Matrix)

`05 §2` and §5.3 are explicit: authoritative governance state lives on the Lemmy-facing surface; Matrix is for real-time coordination. So:

- **Authoritative deliberation, positions, and ratification → the Lemmy-facing Rulebook/Proposal pages.**
- **Notifications and coordination → Matrix**: "a constitutional proposal entered Discussion in c/yourcommunity," "deliberation window closes in 24h," "an objection was filed on Proposal #7," "Proposal #7 ratified — v9 enacts in 48h." This reuses the existing Matrix bridge as a notification rail and keeps the design principle "communication may happen in Matrix, but authoritative governance state is written back to the formal system."

### 4.5 Lifecycle transparency for non-participants

Per `05 §3.7`, a public observer should be able to see *that* a proposal existed, what kind of rule it concerned, the outcome, and that the rationale wasn't fabricated after the fact — without unnecessary identifying material. The proposal's public page (pseudonymous positions, final impact snapshot, enacted version link, governance-log references) delivers this directly.

---

## 5. Concrete, staged implementation plan

**Phase 1 — Make rules legible (low risk, high value, no schema-invariant changes).**
- Member-readable Rulebook page rendering the active `rule_set_version`, with version badge + `text_sha256`.
- Version timeline and diff viewer over the existing `parent_id` chain (data already returned by `admin_list_rule_sets`; add a member-readable read endpoint).
- "Rule in force" deep links from case pages via `moderation_case.rule_set_version_id`.

**Phase 2 — Guided config editor with live impact.**
- Generate the policy editor from `GET .../admin/config` metadata (range/enum/description/doc_anchor/apply_at/requires_step_up/requires_re_jury).
- Wire the `dry_run=true` impact panel into the editor.
- Add a `tier` field to `ConfigKeyMetadata` and render ordinary-vs-constitutional chrome.

**Phase 3 — Structured rules.**
- Migrate `rule_text` to a canonically-serialized rule-item list (preserve single-hash + append-only). Per-rule linking from cases and sanction-kind mapping.

**Phase 4 — Proposal & deliberation engine.**
- New `rule_proposal` aggregate + position/comment/impact-snapshot tables and lifecycle state machine.
- Consent/objection UI with quorum + threshold config keys, objection register, deliberation timer.
- Enactment path that calls the existing `admin_create_rule_set` / `admin_set_config` writers (proposal sits in front of the privileged write).
- Matrix notifications for lifecycle transitions.

**Phase 5 — Constitutional safeguards.**
- Enforce multi-approver, mandated delay (`apply_at` ≠ `immediate`), step-up auth, and longer windows for constitutional-tier proposals.
- Public proposal-log surface for observers; full `governance_log` integration so the audit live-tail covers rulemaking.

---

## 6. Pitfalls to avoid (drawn from the project's own constraints)

- **Don't break the append-only / grandfathering invariant.** Never mutate a past `rule_set_version`; the proposal layer must always enact as a *new* version (ADR-010).
- **Don't collapse standing into one number** to drive a vote — the spec forbids a single reputation score and treats this as constitutional.
- **Don't let constitutional change look like a toggle.** The spec's §5 list (no removing appeals, logging, due process; no unrestricted admin override) must be unrepresentable as a casual in-range config flip — gate those behind the constitutional-tier flow or make them non-configurable.
- **Don't make objection costless or majority brute-forceable.** Reasoned objection + visible threshold is the anti-capture mechanism; a raw 51% button undermines the whole philosophy.
- **Don't strand authoritative state in Matrix.** Coordinate there, ratify and record on the Lemmy-facing governance surface.

---

### Source files referenced in `barrie-cork/lemmy`
- `crates/db_schema/src/source/governance/rule_set_version.rs` — append-only rule-set model.
- `migrations/2026-04-22-000000-0000_add_rule_set_versions/up.sql` — table + invariants.
- `crates/api/api/src/governance/admin_rule_sets.rs` — `admin_create_rule_set` / `admin_list_rule_sets` (mod/admin gate, atomic 3-write).
- `crates/api/api/src/governance/admin_config.rs` — `compute_downstream_impact`, dry-run preview, step-up, denial logging.
- `crates/api/api/src/governance/admin_dashboard_html.rs` — the only existing rendered UI (maud, audit live-tail).
- `crates/api/api_common/src/governance.rs` — governance API surface (no proposal/deliberation primitive present).
- `docs/brehon-law-inspired-network/expert-review-suite/03-customization-and-rulemaking.md` — policy vs rule-text vs constitutional layers; governance-of-customization requirements.
- `docs/brehon-law-inspired-network/expert-review-suite/05-user-interaction-and-procedure.md` — participant journeys, juror procedure, Lemmy/Matrix surface split, admin procedural burden.
