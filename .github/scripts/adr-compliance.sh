#!/usr/bin/env bash
# Brehon governance-v0 ADR red-flag scanner.
#
# Usage: adr-compliance.sh <path-to-pr.diff>
# Exit 0 if clean, 1 if any red flag hit, 2 on script error.
# Emits Markdown findings to stdout for the calling workflow to post as a PR
# comment. Scans only added lines (leading '+', not '+++').
#
# Red flags align with ADRs in docs/brehon-law-inspired-network/99-decisions-and-open-questions.md
# and the "no DB mocking / Selectable template" project rules under .claude/rules/.

set -euo pipefail

DIFF="${1:-/tmp/pr.diff}"
if [ ! -f "$DIFF" ]; then
  echo "::error::diff file not found: $DIFF"
  exit 2
fi

violations=0

emit() {
  printf '### %s\n**ADR / rule:** %s\n\n```diff\n%s\n```\n\n' "$1" "$2" "$3"
  violations=$((violations + 1))
}

added_lines() {
  grep -E '^\+[^+]' "$DIFF" | sed 's/^+//' || true
}

# Print added lines for files where forbidden dependency references would be
# executable/configuration scope rather than policy prose. This intentionally
# excludes PR templates, .claude/.pi runbooks, and design docs where the same
# terms often appear in "do not add Keycloak/OpenFGA/Vault" guardrails.
added_dependency_surface_lines() {
  awk '
    /^\+\+\+ b\// {
      file = substr($0, 7)
      active = (file ~ /(^Cargo\.(toml|lock)$|^crates\/.+\/Cargo\.toml$|^migrations\/|^docker\/|^docker-compose|^\.github\/workflows\/|^scripts\/|(^|\/)package(-lock)?\.json$|(^|\/)pnpm-lock\.yaml$|(^|\/)yarn\.lock$)/)
      next
    }
    active && /^\+[^+]/ { sub(/^\+/, ""); print }
  ' "$DIFF" || true
}

# 1. ADR-010: v2-scope dependencies forbidden in v0.
# Use an identifier-boundary regex rather than \b — underscore counts as a word
# character in ERE, so \bopenfga\b would miss `openfga_rs`, `openfga-sdk`, etc.
# The pattern [^[:alnum:]_]|^ before and [^[:alnum:]]|$ after matches the start
# or end of an identifier in typical code/config contexts.
for term in keycloak openfga vault hsm kms; do
  hits=$(added_dependency_surface_lines | grep -iE "(^|[^[:alnum:]_])${term}([^[:alnum:]]|$)" || true)
  if [ -n "$hits" ]; then
    emit "Forbidden v2 dependency reference: \`${term}\`" "ADR-010 (v0 scope, v2 hardening)" "$hits"
  fi
done

# 2. ADR-008: tampering with append-only governance_log.
hits=$(added_lines | grep -iE '(DROP +TRIGGER|DROP +TABLE).*governance_log|ALTER +TABLE +governance_log|TRUNCATE +governance_log|DELETE +FROM +governance_log|UPDATE +governance_log' || true)
if [ -n "$hits" ]; then
  emit "Destructive change to \`governance_log\`" "ADR-008 (append-only signed log)" "$hits"
fi
hits=$(added_lines | grep -iE 'DROP +FUNCTION +governance_log_hash_chain|CREATE +OR +REPLACE +FUNCTION +governance_log_hash_chain' || true)
if [ -n "$hits" ]; then
  emit "Change to hash-chain trigger function" "ADR-008 (hash chain is load-bearing)" "$hits"
fi

# 3. ADR-013: non-exhaustive CaseStatus match (wildcard arm).
# Two shapes to detect: (a) `match case.status { ... _ => ... }` where the
# discriminant is a field access (common Rust idiom), (b) `match x { CaseStatus::Variant => ... _ => ... }`
# where the match is keyed by a CaseStatus enum value. Awk tracks "are we in a
# match block whose discriminant or arms mention case/status/CaseStatus", and
# flags a wildcard within the next 40 added lines.
wild=$(awk '
  /^\+.*match[[:space:]]/ {
    line = $0
    if (line ~ /case\.status/ || line ~ /[Cc]ase[Ss]tatus/ || line ~ /case_status/) {
      in_match = 1
      line_counter = 0
      start_line = NR
      start_text = line
      next
    }
  }
  in_match == 1 {
    line_counter++
    if (line_counter > 40) {
      in_match = 0
      next
    }
    if ($0 ~ /^\+.*[[:space:]]_[[:space:]]*=>/) {
      print "line " start_line ": " start_text
      print "line " NR ": " $0
      in_match = 0
    }
  }
' "$DIFF" || true)
if [ -n "$wild" ]; then
  emit "Possible non-exhaustive match on \`CaseStatus\` (wildcard \`_ =>\` within 40 lines of a \`match case.status\` / \`CaseStatus\`)" "ADR-013 (\`EmergencyRemove\` must be handled explicitly)" "$wild"
fi
# Per-file token-occurrence balancing — only flag if a file's `+` lines mention
# `CaseStatus::EmergencyRemove` strictly fewer times than its `-` lines, i.e.
# the file lost net occurrences of the variant. This handles arm-split refactors
# (single-line | A | B | EmergencyRemove arm → multi-line arms) without false
# positives, while still catching real removals.
#
# Restricted to actual Rust source under `crates/` because policy/runbook/
# scanner files (this script, PR templates, .pi/.claude prompts, design docs)
# often quote the arm verbatim as documentation, and "removing" such a quote
# is not an ADR-013 violation.
removed_emergency=$(awk '
  function flush_file() {
    if (active && removed_total > added_total) {
      printf "%s", removed_lines
    }
    removed_total = 0
    added_total = 0
    removed_lines = ""
    active = 0
  }
  /^diff --git / { flush_file() }
  /^\+\+\+ b\// {
    file = substr($0, 7)
    active = (file ~ /^crates\/.*\.rs$/)
    next
  }
  active && /^-[^-]/ {
    n = gsub(/CaseStatus::EmergencyRemove/, "&")
    if (n > 0) {
      removed_total += n
      removed_lines = removed_lines $0 "\n"
    }
    next
  }
  active && /^\+[^+]/ {
    added_total += gsub(/CaseStatus::EmergencyRemove/, "&")
    next
  }
  END { flush_file() }
' "$DIFF" || true)
if [ -n "$removed_emergency" ]; then
  emit "Removal of \`CaseStatus::EmergencyRemove\` reference" "ADR-013 (variant is mandatory)" "$removed_emergency"
fi

# 4. ADR-015: bypassing scrub() / raw PII near governance_log writes.
hits=$(added_lines | grep -nE 'governance_log.*(person\.name|person\.email|display_name|\.bio\b|\.matrix_user_id)' || true)
if [ -n "$hits" ]; then
  emit "Direct PII field referenced near \`governance_log\` write" "ADR-015 (pseudonymised actor IDs only)" "$hits"
fi
hits=$(added_lines | grep -nE '#\[allow\([^)]*\)\].*scrub|//[[:space:]]*skip[[:space:]]*scrub|scrub_disabled|scrub[[:space:]]*=[[:space:]]*false' || true)
if [ -n "$hits" ]; then
  emit "Redaction / \`scrub()\` bypass marker" "ADR-015 (all user-visible strings must be scrubbed)" "$hits"
fi

# 5. Project rule: no mocking the DB in integration tests.
hits=$(added_lines | grep -nE '\b(mock_db|MockDb|mockito.*postgres|in_memory_pool|InMemoryPgPool|FakeDbPool)\b' || true)
if [ -n "$hits" ]; then
  emit "Integration test appears to mock Postgres" "Project rule: real-DB integration tests only" "$hits"
fi

# 6. v0 scope creep: new routes in governance router modules (advisory).
# File-scoped to actual route registration files under crates/api/routes/.
# An unscoped grep for `\.route(` + `governance` false-positives on Markdown
# briefs that quote `.route(...)` as prose, and false-negatives on real route
# additions where `governance` is in the surrounding `scope("/governance")`
# wrapper rather than on the same line as `.route(`.
hits=$(awk '
  /^\+\+\+ b\// {
    file = substr($0, 7)
    active = (file ~ /^crates\/api\/routes\/.*\.rs$/)
    next
  }
  active && /^\+.*\.route\(/ { print }
' "$DIFF" || true)
if [ -n "$hits" ]; then
  emit "New governance route added — verify it maps to one of the 11 v0 endpoints" "ADR-010 / 05-mvp-and-delivery-plan.md §2 (advisory)" "$hits"
fi

# 7. Phase 2a rule: derive(Selectable) forbidden in governance view crates.
# The added line with #[derive(..., Selectable, ...)] does not contain the
# filename — we need awk to track the current hunk's file header and only flag
# derive lines inside governance view crates.
hits=$(awk '
  /^\+\+\+ b\// {
    file = substr($0, 7)
    next
  }
  /^\+.*#\[derive\([^)]*Selectable/ {
    if (file ~ /^crates\/db_views\/(governance_case|jury_queue|governance_modlog|reputation)\//) {
      print file ": " $0
    }
  }
' "$DIFF" || true)
if [ -n "$hits" ]; then
  emit "\`derive(Selectable)\` on a governance view struct" "Phase 2a \`view-crate-selectable-template\` rule (bare-scalar fields forbid Selectable)" "$hits"
fi

# 8. Archived design history is immutable.
hits=$(grep -E '^\+\+\+ b/docs/brehon-law-inspired-network/(chat1|chat2)\.md' "$DIFF" || true)
if [ -n "$hits" ]; then
  emit "Edit to archived design-history document" "Project rule: \`chat1.md\` / \`chat2.md\` are immutable history" "$hits"
fi

if [ "$violations" -eq 0 ]; then
  exit 0
fi

# Push and workflow_dispatch events have no PR context — there is nothing to
# attach an "acknowledge" comment to, so they cannot satisfy the bypass gate.
# Treat findings on those triggers as advisory: print and exit 0. The PR-event
# run on the eventual PR is the actual gate.
if [ -z "${PR_NUMBER:-}" ]; then
  printf '\n---\n_%d red flag(s) detected — advisory only (no PR context: push or workflow_dispatch). The PR for this branch is the gate; a maintainer must acknowledge each finding there before merge._\n' "$violations"
  exit 0
fi

# Advisory-bypass: if the repo OWNER has posted a PR comment containing
# "acknowledge" (case-insensitive), the advisory flag is deemed cleared.
# Set GITHUB_ACKNOWLEDGE=true in the workflow env to activate this check.
# OWNER_ID defaults to the numeric GitHub user ID of barrie-cork (15565016).
if [ "${GITHUB_ACKNOWLEDGE:-false}" = "true" ]; then
  repo="${GITHUB_REPOSITORY:-}"
  if [ -n "$repo" ]; then
    owner_id="${OWNER_ID:-15565016}"
    ack_body=$(gh api "repos/$repo/issues/$PR_NUMBER/comments" \
      --jq ".[] | select(.user.id == $owner_id) | .body" 2>/dev/null)
    if echo "$ack_body" | grep -qi acknowledge; then
      printf '\n---\n_Bypass: advisory red-flag acknowledged by maintainer comment. Merging permitted._\n'
      exit 0
    fi
  fi
fi

printf '\n---\n_%d red flag(s) detected. Advisory — a maintainer must acknowledge each finding before merge._\n' "$violations"
exit 1
