#!/usr/bin/env bash
# Phase 5c task 68 probe — verifies three things:
#
# (1) The signature `pub fn config(cfg: &mut ServiceConfig,
#     rate_limit: &RateLimit)` exists at
#     crates/api/routes/src/lib.rs (DQ #19's answer).
#
# (2) The /governance scope block at the file's tail exists and has
#     ≥ 6 routes wired today (Phase 4a+4b shipped 6; 5c adds 6 more).
#
# (3) `cargo check -p lemmy_api_routes --features full` exits 0 on
#     current tree. Covers the actix wiring with the real feature flags
#     task 68 will activate.
#
# Run from repo root. Writes log to .claude/tmp/probe-68.log.

set -euo pipefail
mkdir -p .claude/tmp

ROUTES_LIB="crates/api/routes/src/lib.rs"

# (1) signature check
if ! grep -q 'pub fn config(cfg: &mut ServiceConfig, rate_limit: &RateLimit)' "$ROUTES_LIB"; then
  echo "PROBE_68_FAIL_1: config(cfg, rate_limit) signature missing from $ROUTES_LIB"
  exit 1
fi
echo "PROBE_68_OK_1: config(cfg, rate_limit) signature present"

# (2) route count — governance scope block. Phase 4a+4b = 6 routes:
# /report, /endorsement, /case, /modlog, /jury/me, /jury/vote,
# /admin/assign-jury, /admin/close-case. That's 8. Phase 5c task 68
# adds 6 more, bringing total to 14. Pre-task-68 count must be ≥ 8.
# We grep for the .route( calls inside the /governance scope block.
AWK_BLOCK='/scope\("\/governance"\)/{inside=1} inside{print} /^      \),$/{if(inside){inside=0; exit}}'
ROUTES=$(awk "$AWK_BLOCK" "$ROUTES_LIB" | grep -c '\.route(' || true)
echo "PROBE_68_GOV_ROUTES: $ROUTES"
if [ "$ROUTES" -lt 8 ]; then
  echo "PROBE_68_FAIL_2: governance scope has fewer than 8 routes (pre-5c baseline)"
  exit 1
fi
echo "PROBE_68_OK_2: governance scope has $ROUTES routes (≥8 baseline)"

# (3) cargo check — NOTE: lemmy_api_routes does not declare a `full`
# feature (same plan-drift class as lemmy_server; surfaced by this
# probe 2026-04-18). So we use --workspace --features full (which
# lemmy_utils DOES declare and which unifies full across every dep).
# This is the canonical pattern from feedback_api_crud_oauth_feature_quirk.md.
set +e
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/tmp/probe-68.log 2>&1"
STATUS=$?
set -e
tail -5 .claude/tmp/probe-68.log
if [ "$STATUS" -ne 0 ]; then
  echo "PROBE_68_FAIL_3: cargo check --workspace --features full exit $STATUS"
  exit "$STATUS"
fi
echo "PROBE_68_OK_3: cargo check exit 0"
echo "PROBE_68_OK: all 3 checks pass — actix wiring verified for task 68"
echo ""
echo "PROBE_68_NOTE: plan §9 task 68 DoD specifies"
echo "  'cargo-check.bat -p lemmy_api_routes --features full'"
echo "but lemmy_api_routes does NOT declare the full feature. The plan"
echo "needs updating to either (a) drop --features full for -p lemmy_api_routes,"
echo "or (b) use --workspace --features full as above. Flag this for"
echo "the parallel advisor session's next plan revision."
