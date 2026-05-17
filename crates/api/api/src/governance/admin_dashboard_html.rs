//! `GET /api/v4/governance/admin/dashboard/view` — server-rendered HTML page
//! for the admin governance dashboard.
//!
//! `GET /api/v4/governance/admin/audit/view` — server-rendered HTML page
//! showing recent config-change audit entries with a live `EventSource` tail.
//!
//! Both pages are instance-admin-gated (`is_admin`) and feature-flagged on
//! `governance.dashboard.html_pages_enabled` (default `true`). When the flag
//! resolves `false`, both routes return 404 (feature-off semantics per
//! R-html-3). Handlers live in the same crate as `admin_dashboard` so the
//! `pub(crate)` data-gathering fns are reachable without widening their API
//! surface (R-html-1). Engine: maud (single dep, zero new non-Rust files,
//! native actix `Responder`). Read-only; emits NO `governance_log` (ADR-008).

use crate::governance::{
  admin_dashboard::{gather_dashboard, list_recent_config_changes},
  config::{ConfigCache, Scope, get_bool},
};
use actix_web::{HttpResponse, web::Data};
use lemmy_api_common::governance::{AdminConfigAuditEntry, AdminDashboardResponse};
use lemmy_api_utils::{context::LemmyContext, utils::is_admin};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_diesel_utils::connection::get_conn;
use lemmy_utils::error::LemmyResult;
use maud::{DOCTYPE, PreEscaped, html};

const HTML_PAGES_KEY: &str = "governance.dashboard.html_pages_enabled";

/// Vanilla EventSource script for the audit live-tail page.
/// Named events per admin_audit_stream frame format:
///   event: admin_config_changed\ndata: <JSON>\n\n
///   event: admin_config_change_denied\ndata: <JSON>\n\n
/// EventSource.onmessage fires only on *unnamed* events; addEventListener is
/// required for named events (plan §13 Task 4 GOTCHA).
const AUDIT_SCRIPT: &str = "<script>
const es = new EventSource('/api/v4/governance/admin/audit/stream');
const statusEl = document.getElementById('audit-status');
const tbody = document.getElementById('audit-tbody');
function makeRow(e) {
  const d = JSON.parse(e.data);
  const tr = document.createElement('tr');
  const signed = d.signature ? 'yes' : 'no';
  const prevVal = d.previous_value !== undefined && d.previous_value !== null ? JSON.stringify(d.previous_value) : '';
  const denial = d.denial_reason || '';
  const actor = d.actor_pseudonym || '';
  [d.id, d.created_at, d.entry_kind, d.scope, d.key,
   prevVal, JSON.stringify(d.new_value), d.reason,
   actor, signed, denial].forEach(function(v) {
    const td = document.createElement('td');
    td.textContent = String(v);
    tr.appendChild(td);
  });
  return tr;
}
es.addEventListener('admin_config_changed', function(e) {
  tbody.insertBefore(makeRow(e), tbody.firstChild);
});
es.addEventListener('admin_config_change_denied', function(e) {
  tbody.insertBefore(makeRow(e), tbody.firstChild);
});
es.onerror = function() { statusEl.textContent = 'Reconnecting…'; };
es.onopen  = function() { statusEl.textContent = 'Live (EventSource connected)'; };
</script>";

// ── Dashboard page ────────────────────────────────────────────────────────────

pub async fn admin_dashboard_html(
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<HttpResponse> {
  is_admin(&local_user_view)?;
  let mut cache = ConfigCache::new();
  let mut pool = context.pool();
  let enabled = get_bool(&mut cache, &mut pool, Scope::Instance, HTML_PAGES_KEY).await?;
  if !enabled {
    return Ok(HttpResponse::NotFound().finish());
  }
  let conn = &mut get_conn(&mut pool).await?;
  let resp = gather_dashboard(conn, &mut cache, &context).await?;
  let html = render_dashboard(&resp);
  Ok(
    HttpResponse::Ok()
      .content_type("text/html; charset=utf-8")
      .body(html),
  )
}

fn render_dashboard(resp: &AdminDashboardResponse) -> String {
  html! {
    (DOCTYPE)
    html lang="en" {
      head {
        meta charset="utf-8";
        meta name="viewport" content="width=device-width, initial-scale=1";
        title { "Governance Admin Dashboard" }
        style {
          (PreEscaped(
            "body{font-family:sans-serif;margin:2rem}\
             table{border-collapse:collapse;width:100%}\
             th,td{border:1px solid #ccc;padding:.4rem .8rem;text-align:left}\
             th{background:#f4f4f4}\
             h2{margin-top:2rem}"
          ))
        }
      }
      body {
        h1 { "Governance Admin Dashboard" }
        p { "Calculated: " (resp.calculated_at.format("%Y-%m-%d %H:%M:%S UTC")) }

        h2 { "Active Cases" }
        p { "Total active: " (resp.active_cases.total_active) }
        table {
          thead { tr { th { "Status" } th { "Count" } } }
          tbody {
            @for (status, count) in &resp.active_cases.by_status {
              tr { td { (status) } td { (count) } }
            }
          }
        }

        h2 { "Jury Queue" }
        table {
          thead { tr { th { "State" } th { "Count" } } }
          tbody {
            tr { td { "Pending accept" } td { (resp.jury_queue.pending_accept) } }
            tr { td { "Accepted" } td { (resp.jury_queue.accepted) } }
            tr { td { "Submitted" } td { (resp.jury_queue.submitted) } }
          }
        }

        h2 { "Federation" }
        table {
          thead { tr { th { "State" } th { "Count" } } }
          tbody {
            tr { td { "Active" }  td { (resp.federation.active) } }
            tr { td { "Expired" } td { (resp.federation.expired) } }
            tr { td { "Total" }   td { (resp.federation.total) } }
          }
        }

        h2 { "Rule Sets" }
        p {
          "Communities with rule sets: " (resp.rule_sets.communities_with_rule_sets)
          " — Total versions: " (resp.rule_sets.total_versions)
        }
        @if !resp.rule_sets.per_community.is_empty() {
          table {
            thead { tr { th { "Community ID" } th { "Active Version" } } }
            tbody {
              @for rs in &resp.rule_sets.per_community {
                tr {
                  td { (rs.community_id.0) }
                  td {
                    @if let Some(v) = rs.active_version_id {
                      (v)
                    } @else {
                      "\u{2014}"
                    }
                  }
                }
              }
            }
          }
        }

        h2 { "Reputation" }
        h3 { "Buckets (0, 1\u{2013}30, 31\u{2013}80, 81\u{2013}200, 200+)" }
        table {
          thead {
            tr {
              th { "Dimension" }
              th { "0" }
              th { "1\u{2013}30" }
              th { "31\u{2013}80" }
              th { "81\u{2013}200" }
              th { "200+" }
            }
          }
          tbody {
            (bucket_row("Reporting accuracy",        &resp.reputation.buckets.reporting_accuracy))
            (bucket_row("Jury reliability",          &resp.reputation.buckets.jury_reliability))
            (bucket_row("Participation consistency", &resp.reputation.buckets.participation_consistency))
            (bucket_row("Endorsement strength",      &resp.reputation.buckets.endorsement_strength))
          }
        }

        h3 { "Thresholds (instance)" }
        table {
          thead { tr { th { "Threshold" } th { "Value" } } }
          tbody {
            tr { td { "Jury reliability" }     td { (resp.reputation.thresholds_current.jury_reliability) } }
            tr { td { "Reporting accuracy" }   td { (resp.reputation.thresholds_current.reporting_accuracy) } }
            tr { td { "Endorsement strength" } td { (resp.reputation.thresholds_current.endorsement_strength) } }
          }
        }

        h3 { "Capabilities" }
        table {
          thead { tr { th { "Capability" } th { "Count" } } }
          tbody {
            tr { td { "Jury eligible" }    td { (resp.reputation.capability_counts.jury_eligible_count) } }
            tr { td { "Trusted reporter" } td { (resp.reputation.capability_counts.trusted_reporter_count) } }
            tr { td { "Can sponsor" }      td { (resp.reputation.capability_counts.can_sponsor_count) } }
          }
        }

        h3 { "Founder events" }
        table {
          thead { tr { th { "State" } th { "Count" } } }
          tbody {
            tr { td { "Active" }  td { (resp.reputation.founder_event_stats.active_count) } }
            tr { td { "Expired" } td { (resp.reputation.founder_event_stats.expired_count) } }
          }
        }

        h2 { "Recent Config Changes" }
        @if resp.recent_config_changes.is_empty() {
          p { "No recent config changes." }
        } @else {
          (render_audit_table(&resp.recent_config_changes))
        }
      }
    }
  }
  .into_string()
}

fn bucket_row(label: &str, buckets: &[i64; 5]) -> maud::Markup {
  html! {
    tr {
      td { (label) }
      @for v in buckets.iter() {
        td { (v) }
      }
    }
  }
}

// ── Audit page ────────────────────────────────────────────────────────────────

pub async fn admin_audit_html(
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<HttpResponse> {
  is_admin(&local_user_view)?;
  let mut cache = ConfigCache::new();
  let mut pool = context.pool();
  let enabled = get_bool(&mut cache, &mut pool, Scope::Instance, HTML_PAGES_KEY).await?;
  if !enabled {
    return Ok(HttpResponse::NotFound().finish());
  }
  let conn = &mut get_conn(&mut pool).await?;
  let recent = list_recent_config_changes(conn).await?;
  let html_out = render_audit(&recent);
  Ok(
    HttpResponse::Ok()
      .content_type("text/html; charset=utf-8")
      .body(html_out),
  )
}

pub(crate) fn render_audit(entries: &[AdminConfigAuditEntry]) -> String {
  html! {
    (DOCTYPE)
    html lang="en" {
      head {
        meta charset="utf-8";
        meta name="viewport" content="width=device-width, initial-scale=1";
        title { "Governance Admin \u{2014} Config Audit" }
        style {
          (PreEscaped(
            "body{font-family:sans-serif;margin:2rem}\
             table{border-collapse:collapse;width:100%}\
             th,td{border:1px solid #ccc;padding:.4rem .8rem;text-align:left}\
             th{background:#f4f4f4}\
             #audit-status{color:#888;font-size:.85rem;margin-bottom:1rem}"
          ))
        }
      }
      body {
        h1 { "Governance Config Audit" }
        p id="audit-status" { "Live (EventSource connected)" }
        // Per-admin SSE cap: opening the audit page in a second tab will get
        // a 409 from /audit/stream; EventSource retries per retry:10000 frame.
        p style="font-size:.8rem;color:#aaa" {
          "Note: only one live-tail stream per admin is supported. "
          "A second tab will show reconnecting status."
        }
        table id="audit-table" {
          thead {
            tr {
              th { "ID" } th { "Time (UTC)" } th { "Kind" } th { "Scope" }
              th { "Key" } th { "Prev" } th { "New" } th { "Reason" }
              th { "Actor" } th { "Signed" } th { "Denial" }
            }
          }
          tbody id="audit-tbody" {
            @for e in entries {
              (audit_entry_row(e))
            }
          }
        }
        (PreEscaped(AUDIT_SCRIPT))
      }
    }
  }
  .into_string()
}

fn audit_entry_row(e: &AdminConfigAuditEntry) -> maud::Markup {
  html! {
    tr {
      td { (e.id) }
      td { (e.created_at.format("%Y-%m-%d %H:%M:%S")) }
      td { (e.entry_kind) }
      td { (e.scope) }
      td { (e.key) }
      td {
        @if let Some(v) = &e.previous_value { (v.to_string()) } @else { "" }
      }
      td { (e.new_value.to_string()) }
      td { (e.reason) }
      td {
        @if let Some(p) = &e.actor_pseudonym { (p) } @else { "" }
      }
      td {
        @if e.signature.is_some() { "yes" } @else { "no" }
      }
      td {
        @if let Some(d) = &e.denial_reason { (d) } @else { "" }
      }
    }
  }
}

fn render_audit_table(entries: &[AdminConfigAuditEntry]) -> maud::Markup {
  html! {
    table {
      thead {
        tr {
          th { "ID" } th { "Time (UTC)" } th { "Kind" } th { "Scope" }
          th { "Key" } th { "Prev" } th { "New" } th { "Reason" }
          th { "Actor" } th { "Signed" } th { "Denial" }
        }
      }
      tbody {
        @for e in entries {
          (audit_entry_row(e))
        }
      }
    }
  }
}
