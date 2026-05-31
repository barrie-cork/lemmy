//! `GET /api/v4/governance/admin/audit/stream` — hand-rolled Server-Sent
//! Events wrapper around Postgres `LISTEN governance_events`.
//!
//! The `governance_log_notify_trigger` (defined in migration
//! `2026-04-20-000100-0000_fix_governance_log_notify_trigger_after_sign`,
//! superseding the initial v1-AD-a trigger) fires
//! `pg_notify('governance_events', ...)` on `AFTER UPDATE OF signature`
//! when `signature` transitions `NULL → NOT NULL`. That is the moment a
//! row becomes a subscribable artifact (the `governance_log_signature_gate`
//! trigger permits exactly one such transition per row, so the notify
//! fires at most once per row). Subscribers observing the prior INSERT
//! would see `signature IS NULL` rows that the V2 messaging bridge
//! cannot verify.
//!
//! This handler holds a dedicated `tokio_postgres::Client` for the lifetime
//! of the HTTP response, filters notifications to the two `admin_config_*`
//! entry kinds, hydrates the full `governance_log` row from a pooled
//! diesel-async connection, projects via the shared
//! [`project_to_audit_entry`] helper, and emits SSE frames.
//!
//! Read-only: emits NO `governance_log` entry (ADR-008).
//!
//! Per-admin connection cap: one open stream per `PersonId`. Second
//! concurrent connection from the same admin returns 409 Conflict. The
//! [`SseGuard`] removes the entry from [`ACTIVE_SSE_ADMINS`] on drop so
//! the admin can reconnect after disconnecting.
//!
//! Hand-rolled per OQ-V1-AD-02: avoids the `actix-web-lab` dep for one
//! endpoint. Frame format per HTML5 §9.2.4: `event: X\ndata: Y\n\n`.
//! Keepalive comment (`: keepalive\n\n`) every 15 s prevents idle-proxy
//! timeouts.

use crate::governance::{
  audit_projection::project_to_audit_entry,
  governance_log::{ENTRY_KIND_ADMIN_CONFIG_CHANGE_DENIED, ENTRY_KIND_ADMIN_CONFIG_CHANGED},
};
use actix_web::{
  HttpResponse,
  web::{Bytes, Data},
};
use async_stream::stream;
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use lemmy_api_utils::{context::LemmyContext, utils::is_admin};
use lemmy_db_schema::{
  newtypes::GovernanceLogId, source::governance::governance_log::GovernanceLog,
};
use lemmy_db_schema_file::{PersonId, schema::governance_log as governance_log_schema};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_diesel_utils::connection::get_conn;
use lemmy_utils::error::{LemmyErrorType, LemmyResult};
use std::{collections::HashSet, pin::Pin, sync::OnceLock, task::Poll, time::Duration};
use tokio::{
  sync::{Mutex, mpsc},
  time::interval,
};
use tokio_postgres::{AsyncMessage, NoTls, Notification};

/// Per-admin SSE-connection tracker. `OnceLock` avoids the `once_cell`
/// dep; `Mutex` makes the insert + remove atomic; process restart clears
/// state (documented v2 limitation — see plan §4.1 load-bearing
/// decisions).
static ACTIVE_SSE_ADMINS: OnceLock<Mutex<HashSet<PersonId>>> = OnceLock::new();

/// Bounded capacity for the SSE notification channel. If a slow client
/// can't drain within 256 pending notifications, new notifications are
/// dropped (try_send on Full) rather than back-pressuring the
/// tokio-postgres LISTEN connection. Governance events are small; 256
/// gives a fast client comfortable headroom and a slow client a
/// diagnosable gap instead of memory growth.
const SSE_CHANNEL_CAPACITY: usize = 256;

fn active_sse_admins() -> &'static Mutex<HashSet<PersonId>> {
  ACTIVE_SSE_ADMINS.get_or_init(|| Mutex::new(HashSet::new()))
}

/// Drops the per-admin cap entry + aborts the driver task when the
/// stream body is dropped (client disconnect, server shutdown, panic).
///
/// `tokio::spawn` inside `Drop` is fire-and-forget; on runtime shutdown
/// the cleanup task may not run, leaving the HashSet entry — acceptable
/// because the next process restart clears the set (plan §4.1).
struct SseGuard {
  admin_id: PersonId,
  driver: Option<tokio::task::JoinHandle<()>>,
}

impl Drop for SseGuard {
  fn drop(&mut self) {
    if let Some(handle) = self.driver.take() {
      handle.abort();
    }
    let admin_id = self.admin_id;
    tokio::spawn(async move {
      let mut set = active_sse_admins().lock().await;
      set.remove(&admin_id);
    });
  }
}

pub async fn admin_audit_stream(
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<HttpResponse> {
  is_admin(&local_user_view)?;

  let admin_id = local_user_view.person.id;

  // Per-admin cap: return 409 Conflict via a direct response so the HTTP
  // status is correct without touching the shared `LemmyErrorType` enum.
  // See plan §16 acceptance: "Per-admin SSE cap returns 409 on second
  // concurrent connection from same PersonId".
  {
    let mut set = active_sse_admins().lock().await;
    if set.contains(&admin_id) {
      return Ok(
        HttpResponse::Conflict()
          .content_type("text/plain; charset=utf-8")
          .body("another SSE stream is already open for this admin"),
      );
    }
    set.insert(admin_id);
  }

  let db_url = context.database_url();
  let (pg_client, pg_conn) = match tokio_postgres::connect(db_url, NoTls).await {
    Ok(pair) => pair,
    Err(e) => {
      active_sse_admins().lock().await.remove(&admin_id);
      return Err(LemmyErrorType::Unknown(format!("tokio_postgres connect failed: {e}")).into());
    }
  };

  // Bounded channel: if a slow SSE client can't drain fast enough, drop
  // new notifications on Full instead of letting the backlog grow without
  // bound (cr-14). Back-pressure would propagate upstream to tokio-postgres,
  // which is the wrong direction — Postgres is not waiting for us.
  // Dropped events reach the client as a gap; the client can reconnect and
  // re-read `/admin/config/audit` for the backfill.
  let (tx, rx): (_, mpsc::Receiver<Notification>) = mpsc::channel(SSE_CHANNEL_CAPACITY);
  let driver = tokio::spawn(async move {
    let mut connection = pg_conn;
    std::future::poll_fn(move |cx| {
      loop {
        match Pin::new(&mut connection).poll_message(cx) {
          Poll::Ready(Some(Ok(AsyncMessage::Notification(n)))) => match tx.try_send(n) {
            Ok(()) => {}
            Err(mpsc::error::TrySendError::Full(_)) => {
              tracing::warn!(
                "admin_audit_stream SSE channel full (capacity {SSE_CHANNEL_CAPACITY}); \
                 dropping governance_events notification — slow client will see a gap"
              );
            }
            Err(mpsc::error::TrySendError::Closed(_)) => {
              return Poll::Ready(());
            }
          },
          Poll::Ready(Some(Ok(_))) => {}
          Poll::Ready(Some(Err(_))) | Poll::Ready(None) => {
            return Poll::Ready(());
          }
          Poll::Pending => return Poll::Pending,
        }
      }
    })
    .await;
  });

  if let Err(e) = pg_client.batch_execute("LISTEN governance_events").await {
    driver.abort();
    active_sse_admins().lock().await.remove(&admin_id);
    return Err(LemmyErrorType::Unknown(format!("LISTEN failed: {e}")).into());
  }

  let guard = SseGuard {
    admin_id,
    driver: Some(driver),
  };

  let context_for_stream = context.clone();

  let body = stream! {
    let _guard = guard;
    let _client = pg_client; // hold conn alive for request lifetime
    let mut rx = rx;

    // Top-level SSE `retry:` field per HTML5 §9.2.5 — sets EventSource's
    // reconnection interval to 10 s. An `event: retry\ndata: 10000\n\n`
    // frame would be a custom event named "retry" with data `10000`, which
    // browsers dispatch to any `addEventListener("retry", ...)` listener
    // but do NOT apply as the reconnect interval (cr-15).
    yield Ok::<Bytes, actix_web::Error>(Bytes::from("retry: 10000\n\n"));

    let mut heartbeat = interval(Duration::from_secs(15));
    heartbeat.tick().await;

    loop {
      tokio::select! {
        _ = heartbeat.tick() => {
          yield Ok(Bytes::from(": keepalive\n\n"));
        }
        maybe_note = rx.recv() => {
          let Some(note) = maybe_note else { break };
          if note.channel() != "governance_events" { continue }

          let Ok(parsed) = serde_json::from_str::<serde_json::Value>(note.payload()) else {
            continue;
          };
          let Some(kind) = parsed.get("kind").and_then(|v| v.as_str()) else { continue };
          if kind != ENTRY_KIND_ADMIN_CONFIG_CHANGED
            && kind != ENTRY_KIND_ADMIN_CONFIG_CHANGE_DENIED
          {
            continue;
          }
          let Some(entry_id) = parsed.get("entry_id").and_then(serde_json::Value::as_i64) else {
            continue;
          };

          let mut pool = context_for_stream.pool();
          let Ok(mut conn) = get_conn(&mut pool).await else { continue };
          let row: Option<GovernanceLog> = match governance_log_schema::table
            .filter(governance_log_schema::id.eq(GovernanceLogId(entry_id)))
            .select(GovernanceLog::as_select())
            .first(&mut conn)
            .await
            .optional()
          {
            Ok(opt) => opt,
            Err(e) => {
              tracing::warn!(
                "admin_audit_stream: error hydrating governance_log entry {entry_id}: {e}"
              );
              continue;
            }
          };
          let Some(row) = row else { continue };

          let entry = project_to_audit_entry(row);
          let Ok(json) = serde_json::to_string(&entry) else { continue };
          let kind_str = kind.to_string();
          yield Ok(Bytes::from(format!("event: {kind_str}\ndata: {json}\n\n")));
        }
      }
    }
  };

  Ok(
    HttpResponse::Ok()
      .content_type("text/event-stream")
      .insert_header(("Cache-Control", "no-cache, no-transform"))
      .insert_header(("X-Accel-Buffering", "no"))
      .streaming(body),
  )
}
