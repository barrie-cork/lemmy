#!/usr/bin/env bash
# Dependabot alert dismissal commands — v1 close-out, prepped 2026-06-04
#
# READ THIS HEADER BEFORE RUNNING ANYTHING. These are OUTBOUND, state-changing
# GitHub actions (they mutate the repo's security tab, visible to anyone with
# repo access). Run section by section; each `gh api` call dismisses ONE alert.
#
# Token: your gh auth has `repo` scope → sufficient to dismiss Dependabot alerts
# on a repo you admin (the dedicated `security_events` scope is for code-scanning,
# not Dependabot). Verified 2026-06-04.
#
# ⚠️ API LIMIT (learned 2026-06-04, the hard way): `dismissed_comment` caps at
# 280 chars. Over-length → HTTP 422 "Invalid request" and the alert is NOT mutated
# (safe — rejected before any state change). Keep comments terse; the full rationale
# lives in the linked report, not the comment. Measure with `echo ${#VAR}` before sending.
#
# ── EXECUTED 2026-06-04: SECTION A done (12 wasmtime alerts dismissed, verified
#    0 wasmtime open). SECTION B SKIPPED (user chose post-M1 testcontainers bump).
#    SECTION C never applies (M1-gated fixes). 18 open → 7 open (tar×3 + webpki×3 + idna×1). ──
#
# ── LIVE-VERIFIED STATE (2026-06-04, gh api repos/barrie-cork/lemmy/dependabot/alerts) ──
# 18 open alerts in 4 clusters:
#   • wasmtime  ×12  → DISMISS (Phase 5 decision: keep-deferred)        ← SECTION A
#   • astral-tokio-tar ×3 → DISMISS as dev-only (CORRECTED — see note)  ← SECTION B
#   • rustls-webpki ×3 + idna ×1 → DO NOT DISMISS (M1-gated code fix)   ← SECTION C (no commands)
#
# ⚠️ TWO PLAN PREMISES WERE STALE (verify-against-reality caught them):
#   1. Plan Phase 5 said "13 aarch64-only criticals" → actually 12 alerts, only 2
#      criticals are aarch64-only; rest medium/low. Dismissal still correct (decision
#      was keep-deferred regardless), reason text below reflects the TRUE picture.
#   2. Plan Phase 4-T3 said "astral-tokio-tar 0.6.0 IS the fix release; dispute it"
#      → WRONG. We pin 0.6.0; fixes are 0.6.1 (#57,#58) and 0.6.2 (#65). 0.6.0 is
#      VULNERABLE. BUT astral-tokio-tar is a DEV-ONLY dep (via testcontainers 0.27.3,
#      the e2e Postgres harness) — NOT in the production tree. So the honest remedy
#      is "dismiss as not-used-in-production" (Section B) OR bump testcontainers when
#      M1 frees the lockfile. Do NOT "dispute" — 0.6.0 is genuinely affected.

set -euo pipefail
REPO="barrie-cork/lemmy"

# ──────────────────────────────────────────────────────────────────────────────
# SECTION A — wasmtime ×12  (Phase 5 keep-deferred decision)
# ──────────────────────────────────────────────────────────────────────────────
# Reason enum: tolerable_risk. Rationale (TRUE picture): 2 criticals (#46,#43) are
# aarch64-only sandbox escapes — Brehon deploys x86_64 only. Rest are medium/low
# requiring attacker-controlled WASM; Brehon's WASM guests are operator-authored
# governance plugins (ADR-012), not attacker-supplied. extism 1.21.0 (our pin)
# pins wasmtime 41.0.4 via internal wiggle-macro APIs — bare [patch.crates-io]
# won't compile. Deferred to deps-r3; adopt upstream extism release w/ wasmtime>=42.
# Full record: .claude/PRPs/reports/v1-deps-r3-wasmtime-deferral-decision.md

WASMTIME_REASON="Extism (ADR-012 plugin host) pins wasmtime 41.0.4 via internal wiggle-macro APIs; bare patch override won't compile. WASM guests are operator-authored governance plugins, not attacker-supplied. Criticals #43/#46 are aarch64 sandbox escapes; Brehon deploys x86_64 only. Rest medium/low needing attacker-controlled WASM. Deferred to deps-r3; will adopt upstream extism release with wasmtime>=42. See .claude/PRPs/reports/v1-deps-r3-wasmtime-deferral-decision.md"

for N in 60 47 46 45 44 43 42 41 40 39 38 37; do
  echo ">>> dismissing wasmtime alert #$N"
  gh api -X PATCH "repos/${REPO}/dependabot/alerts/${N}" \
    -f state="dismissed" \
    -f dismissed_reason="tolerable_risk" \
    -f dismissed_comment="${WASMTIME_REASON}"
  echo
done

# ──────────────────────────────────────────────────────────────────────────────
# SECTION B — astral-tokio-tar ×3  (CORRECTED: dev-only, not "dispute")
# ──────────────────────────────────────────────────────────────────────────────
# astral-tokio-tar 0.6.0 is pulled ONLY by testcontainers 0.27.3 (e2e test harness,
# dev-dependency). It is NOT in the production/deployed dependency tree. The 3 alerts
# (#57 GHSA-fp55 medium, #58 GHSA-xx64 low, #65 GHSA-3cv2 medium) are tar-parsing
# issues that only execute in CI/local e2e parsing trusted container layers → zero
# production exposure. Reason enum: not_used (accurate — not used in the shipped binary).
#
# ALTERNATIVE (preferred long-term): bump testcontainers to a release depending on
# astral-tokio-tar >= 0.6.2 — a Cargo.lock change, M1-GATED (lockfile overlap). If you
# bump instead of dismiss, SKIP this section and do it post-M1. Dismissing now is fine
# (dev-only) and reversible if the bump later closes them.

TAR_REASON="astral-tokio-tar 0.6.0 is a DEV-ONLY dependency (via testcontainers 0.27.3, the e2e Postgres test harness) — not in the production/deployed dependency tree. Tar-parsing issue only executes in CI/local e2e against trusted container layers; zero runtime exposure. Will close transitively when testcontainers bumps to astral-tokio-tar >= 0.6.2 (M1-gated lockfile change)."

for N in 65 58 57; do
  echo ">>> dismissing astral-tokio-tar alert #$N (dev-only)"
  gh api -X PATCH "repos/${REPO}/dependabot/alerts/${N}" \
    -f state="dismissed" \
    -f dismissed_reason="not_used" \
    -f dismissed_comment="${TAR_REASON}"
  echo
done

# ──────────────────────────────────────────────────────────────────────────────
# SECTION C — rustls-webpki ×3 + idna ×1  →  DO NOT DISMISS (no commands here)
# ──────────────────────────────────────────────────────────────────────────────
# These are Phase 4-T1/T2 work — they get FIXED, not dismissed:
#   • rustls-webpki #55 (HIGH, fix 0.103.13), #50 (low), #49 (low) → T1 drops the
#     rustls-webpki chain by inlining the webmention reqwest impl (crates/api/
#     api_utils/src/utils.rs). Production code change. M1-GATED.
#   • idna #33 (medium, fix 1.0.0) → T2 forks mdurl + bumps idna 0.3->1.1 via the
#     first [patch.crates-io] entry. M1-GATED (Cargo.toml overlap).
# Dismissing these would hide a real, fixable HIGH (rustls-webpki #55). Leave them
# OPEN until Phase 4-T1/T2 land post-M1. (Plan Phase 4 lines 236-237.)

echo "=== DONE. Sections A+B dismissed 15 alerts (12 wasmtime + 3 tar). ==="
echo "=== Section C (rustls-webpki x3 + idna x1) left OPEN — they get FIXED in Phase 4-T1/T2 post-M1, not dismissed. ==="
echo
echo "Verify result (expect 4 open remaining: rustls-webpki #55/#50/#49 + idna #33):"
echo "  gh api repos/${REPO}/dependabot/alerts --paginate --jq '[.[] | select(.state==\"open\")] | length'"
echo "  # 18 total - 15 dismissed (12 wasmtime + 3 tar) = 4 remaining (the M1-gated T1/T2 fixes)"
