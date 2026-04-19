//! Phase 6 task 76 — federation outbound orchestrator.
//!
//! ## Purpose
//!
//! This module is the orchestrator wrapper that bridges
//! `submit_jury_vote.rs`'s in-flight transaction to Agent D's
//! [`lemmy_apub::governance::outbox`] builder/enqueuer split.
//! When the winning sanction has scope
//! [`SanctionScope::FederatedRecommendation`], `submit_jury_vote`'s
//! post-decision transaction calls
//! [`send_local_sanction_notice`] **on the in-flight conn**, which:
//!
//! 1. Looks up the local admin Person (ADR-010 single-admin v0) via
//!    [`PersonView::list_admins`] using a fresh pool conn (admin row
//!    has been committed since instance bootstrap — safe outside the tx).
//! 2. Calls Agent D's
//!    [`outbox::build_local_sanction_notice_plan`] **passing the
//!    in-flight conn**, so the build sees the just-inserted `sanction`
//!    row from `submit_jury_vote.rs:226-242`.
//! 3. Calls Agent D's [`outbox::enqueue_sanction_notice_activity`] on
//!    the in-flight conn — INSERTs a `sent_activity` row that commits
//!    or rolls back together with the rest of the post-decision logic.
//! 4. Calls
//!    [`crate::governance::governance_log::append`] (re-export of
//!    [`lemmy_db_schema::source::governance::governance_log::append`])
//!    on the in-flight conn — INSERTs the
//!    `federation_sanction_sent` log entry that commits or rolls back
//!    with the rest.
//!
//! Steps 3 + 4 share the in-flight tx, so they commit atomically with
//! each other AND with the case-decided log entry, public_case_log
//! INSERT, sanction INSERT, sponsor-liability deltas, etc. that
//! `submit_jury_vote::process_vote` produces. Plan §757 atomicity
//! requirement satisfied: if the federation publish fails, the entire
//! `submit_jury_vote` call rolls back — no half-decided cases where
//! local state says "Decided" but no peer was notified.
//!
//! ## DQ resolutions
//!
//! - **DQ-6.5** (resolved id 35) — branch on
//!   `winning_sanction.scope == SanctionScope::FederatedRecommendation`,
//!   not on `SanctionAction`. The call site uses
//!   [`map_decision_to_sanction`] to re-derive the scope from
//!   `winning_decision` since the local `(scope, action)` tuple is
//!   scoped to `submit_jury_vote.rs`'s sanction-insert block (lines
//!   226-269) which closes before the federation-publish hook (line
//!   396).
//! - **DQ-6.6** (resolved id 36) — outbound side: Agent D's
//!   builder/enqueuer split is preserved; this wrapper consumes both
//!   halves on the caller's conn.
//! - **DQ-6.6-inbound** (resolved id 37) — moved `governance_log::append`
//!   down to `lemmy_db_schema`, which simplified this wrapper: we call
//!   `governance_log::append` directly on the in-flight conn without
//!   any cycle concerns. Per `.claude/rules/decision-queue.md`
//!   "Attribution integrity" the original `answered_by: "advisor"`
//!   label on id 37 is a known process breach; the refactor itself
//!   stands and is correct, see runlog 2026-04-19 09:30Z entry.
//! - **DQ-6.7** (resolved id 38, `impl-self-resolved`) — brief specified
//!   the wrapper opens its own `conn.run_transaction()` with signature
//!   `(case_id, &context)`, but the call site already runs inside an
//!   outer `run_transaction` (submit_jury_vote.rs:109-116). The
//!   established codebase pattern (sponsor_liability.rs:34-35)
//!   explicitly forbids nesting `run_transaction`. Wrapper signature
//!   adapted to `(case_id, conn, context)` so the federation publish
//!   runs on the in-flight conn and is atomic with the outer tx per
//!   plan §757. The actor pseudonym is computed inside the wrapper
//!   from the loaded admin Person rather than passed in — the
//!   federation send is performed by the local admin (ADR-010), not
//!   the juror who happened to cast the deciding vote (ADR-015
//!   attribution).
//!
//! ## Future cleanup
//!
//! Per advisor's id 37 note, "the orchestrator wrapper becomes optional
//! rather than mandatory. Phase-6 carry-forward: future cleanup may
//! collapse Agent D's builder/orchestrator split". A v1 simplification
//! could inline this wrapper's three calls directly into
//! `submit_jury_vote.rs`. The wrapper exists today to keep the
//! `submit_jury_vote` handler readable and to give the brief's
//! `send_local_sanction_notice` name a home.

use crate::governance::actor_pseudonym_helper;
use activitypub_federation::config::Data;
use diesel_async::AsyncPgConnection;
use lemmy_api_utils::context::LemmyContext;
use lemmy_apub::governance::outbox;
use lemmy_apub_objects::objects::person::ApubPerson;
use lemmy_db_schema::{
  newtypes::ModerationCaseId,
  source::governance::governance_log::{
    self,
    ENTRY_KIND_FEDERATION_SANCTION_SENT,
  },
};
use lemmy_db_views_person::PersonView;
use lemmy_db_views_site::SiteView;
use lemmy_utils::error::{LemmyErrorType, LemmyResult};

/// Orchestrate a federated SanctionNotice publish: build the plan,
/// then enqueue + audit-log on the caller's in-flight conn.
///
/// Called from `submit_jury_vote::process_vote` when the winning
/// sanction has `scope == SanctionScope::FederatedRecommendation` (per
/// DQ-6.5 — branch on scope, not action). Runs inside the existing
/// `conn.run_transaction()` that wraps the entire post-decision block;
/// failure here rolls back the whole decision so local state never
/// outruns federation (plan §757).
///
/// # Arguments
///
/// - `case_id` — the moderation case being decided. Drives all DB
///   reads inside [`outbox::build_local_sanction_notice_plan`].
/// - `conn` — the **in-flight transaction conn** owned by the caller's
///   `conn.run_transaction()`. Both writes (sent_activity INSERT,
///   governance_log INSERT) run on this conn and share the outer tx.
///   Reborrowed as `&mut (&mut *conn).into()` for the calls that take
///   `&mut DbPool<'_>` (canonical pattern per
///   `sponsor_liability.rs:319-323`), so `conn` remains usable on
///   the caller side after this function returns (the caller still
///   needs to write the `case_decided` log entry on the same tx).
/// - `context` — needed by the builder for activity-id hostname
///   generation and by `Person::read`/etc. inside `resolve_target_url`.
///
/// The `actor_pseudonym` for the `federation_sanction_sent` log entry
/// is computed inside this wrapper from the loaded admin Person — the
/// federation send is performed by the local admin (ADR-010), not the
/// juror who happened to cast the deciding vote. Passing the juror's
/// pseudonym here would mis-attribute the federation action under
/// ADR-015.
///
/// # Errors
///
/// Returns `NotFound` if no local admin exists (ADR-010 violation in
/// production; in tests it means `seed_person` was not called with
/// `admin = true`). Propagates DB errors and AP-construction errors
/// from the builder. Any error here triggers `?` propagation up
/// through `submit_jury_vote::process_vote`, which rolls back the
/// entire post-decision tx.
pub async fn send_local_sanction_notice(
  case_id: ModerationCaseId,
  conn: &mut AsyncPgConnection,
  context: &Data<LemmyContext>,
) -> LemmyResult<()> {
  // Step 1 — load the local admin Person (ADR-010 single-admin v0).
  // Uses a fresh pool conn since the admin row has been committed
  // since instance bootstrap; safe outside the in-flight tx.
  let admin_person = load_local_admin(context).await?;
  let admin_id = admin_person.id;
  let apub_actor: ApubPerson = admin_person.into();

  // Step 1.5 — derive the admin's pseudonym for the federation_sanction_sent
  // log entry on the in-flight conn. ADR-015 requires the actor_pseudonym
  // column to identify the actor that performed the action; the federation
  // send is performed by the local admin, not by the deciding juror. The
  // helper insert is idempotent under the `(person_id)` unique constraint
  // (see actor_pseudonym_helper docs), so executing it inside the outer
  // tx is safe even if the admin pseudonym row already exists from prior
  // governance writes.
  let admin_pseudonym =
    actor_pseudonym_helper::get_or_create(&mut (&mut *conn).into(), admin_id).await?;

  // Step 2 — build the plan on the IN-FLIGHT conn so the build sees
  // the just-inserted `sanction` row from
  // `submit_jury_vote.rs:226-242`. If we used a fresh pool conn here,
  // the sanction wouldn't be visible (different tx, doesn't see
  // uncommitted writes).
  let plan = outbox::build_local_sanction_notice_plan(
    case_id,
    &apub_actor,
    conn,
    context,
  )
  .await?;

  // Step 3 — enqueue the AP activity on the in-flight conn. The
  // sent_activity INSERT participates in the outer tx, so it commits
  // (or rolls back) with the rest of the post-decision writes.
  outbox::enqueue_sanction_notice_activity(&plan, conn).await?;

  // Step 4 — append the federation_sanction_sent log entry on the
  // SAME in-flight conn. Sharing the conn means this INSERT also
  // shares the outer tx; either both writes commit or neither does.
  // ADR-008 hash-chain audit-trail invariant satisfied: every
  // governance write hits the log. Reborrow `conn` via
  // `&mut (&mut *conn).into()` so the caller can still use `conn`
  // after this call (e.g. to append the `case_decided` entry).
  governance_log::append(
    &mut (&mut *conn).into(),
    ENTRY_KIND_FEDERATION_SANCTION_SENT,
    plan.log_payload,
    Some(admin_pseudonym),
  )
  .await?;

  Ok(())
}

/// Load the local admin Person (ADR-010 single-admin v0).
///
/// Mirrors the canonical pattern used at `api_crud/src/site/read.rs:43`
/// and `api/src/local_user/add_admin.rs:30`: pull the local site,
/// list admins on its instance, take the first. v0 has exactly one
/// admin per ADR-010; if multiple admins ever exist we still want the
/// "oldest" admin (which `list_admins` orders by; see
/// `db_views/person/src/impls.rs:108`), so taking the first is
/// stable.
///
/// # Errors
///
/// Returns `NotFound` if no local admin exists. In production this is
/// an ADR-010 violation (the bootstrap should have created one); in
/// tests it means the test fixture forgot `LocalUser { admin: true }`.
async fn load_local_admin(
  context: &Data<LemmyContext>,
) -> LemmyResult<lemmy_db_schema::source::person::Person> {
  let site_view = SiteView::read_local(&mut context.pool()).await?;
  let admins = PersonView::list_admins(
    None,
    site_view.instance.id,
    &mut context.pool(),
  )
  .await?;
  let admin_view = admins
    .into_iter()
    .next()
    .ok_or(LemmyErrorType::NotFound)?;
  Ok(admin_view.person)
}
