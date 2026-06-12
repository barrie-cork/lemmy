#!/usr/bin/env bash
# check-excluded-lockfiles.sh — assert every workspace-EXCLUDED crate has a
# COMMITTED Cargo.lock.
#
# Why this exists (incident 2026-06-12, services/bridge):
#   A cargo workspace-excluded crate (root Cargo.toml `exclude = [...]`) is a
#   separate cargo project with its OWN dep resolution — the root Cargo.lock does
#   NOT cover it. If the excluded crate's Cargo.lock is absent, every cold
#   check/build/CI run re-resolves its deps against the live crates.io index and
#   can SILENTLY pick a breaking transitive bump (services/bridge got `time
#   0.3.48`, which broke `ruma-common 0.19.0` with 25 E0119 errors — no diff, no
#   commit, just a different resolution). The fix-the-incident pin worked, but
#   the bug CLASS (excluded crate + no committed lock) had no guard. This is it.
#
#   The prior mitigation was advisory ("commit it if created", m1-a-impl-8.md) and
#   got ignored — which caused the incident. This script makes it mechanical.
#
# What it checks: for each path in the root Cargo.toml `exclude = [...]` array
# that is a crate dir (has a Cargo.toml), assert `<dir>/Cargo.lock` is TRACKED in
# git (committed), not merely present on disk (an on-disk-but-untracked lock is
# the exact trap — it builds green locally, breaks on a clean CI checkout).
#
# Exit 0 = all excluded crates have a committed lock (or no excludes).
# Exit 1 = at least one excluded crate is missing a committed Cargo.lock.
# Exit 2 = could not determine state (not a git repo, no root Cargo.toml).
#
# Detection logic mirrors feedback_workspace_excluded_crate_must_have_lockfile.md
# §Detection: `git ls-tree HEAD <dir>/Cargo.lock` empty == NOT tracked (per
# pattern_verify_before_trusting_shell_output — empty output + exit 0 != tracked).
#
# Library crates are exempt (a lib legitimately may ship without a lock), but we
# cannot cheaply tell lib from bin without parsing every manifest; the
# conservative rule is "any excluded crate dir with a [[bin]] or src/main.rs
# needs a lock". We detect that and only enforce on apparent binaries; a crate
# with neither is treated as a library and skipped (warned, not failed).

set -euo pipefail

REPO_ROOT=$(git rev-parse --show-toplevel 2>/dev/null || true)
if [ -z "$REPO_ROOT" ]; then
  echo "check-excluded-lockfiles: not a git repo; skipping (exit 2)" >&2
  exit 2
fi

ROOT_TOML="$REPO_ROOT/Cargo.toml"
if [ ! -f "$ROOT_TOML" ]; then
  echo "check-excluded-lockfiles: no root Cargo.toml; skipping (exit 2)" >&2
  exit 2
fi

# Extract the `exclude = [ ... ]` array from the [workspace] table. Handles both
# single-line `exclude = ["a", "b"]` and multi-line array forms. Strips quotes,
# commas, brackets, comments. Pure text — no toml parser dependency.
excludes=$(
  awk '
    /^[[:space:]]*exclude[[:space:]]*=/ { inblk=1 }
    inblk {
      line=$0
      # capture everything; stop after the closing ]
      print line
      if (line ~ /\]/) { inblk=0 }
    }
  ' "$ROOT_TOML" \
  | tr ',' '\n' \
  | sed -E 's/.*exclude[[:space:]]*=[[:space:]]*\[?//; s/#.*$//; s/[][]//g; s/"//g; s/'\''//g; s/^[[:space:]]+//; s/[[:space:]]+$//' \
  | grep -vE '^[[:space:]]*$' || true
)

if [ -z "$excludes" ]; then
  echo "check-excluded-lockfiles: no workspace excludes; nothing to check."
  exit 0
fi

fail=0
checked=0
while IFS= read -r dir; do
  [ -z "$dir" ] && continue
  crate_toml="$REPO_ROOT/$dir/Cargo.toml"
  if [ ! -f "$crate_toml" ]; then
    # exclude entry that isn't a crate dir (e.g. a glob or stale path) — skip.
    continue
  fi

  # Is it a binary? bin if it has [[bin]] OR src/main.rs. Otherwise treat as lib.
  is_bin=0
  if grep -qE '^\[\[bin\]\]' "$crate_toml" 2>/dev/null; then is_bin=1; fi
  if [ -f "$REPO_ROOT/$dir/src/main.rs" ]; then is_bin=1; fi

  if [ "$is_bin" -eq 0 ]; then
    echo "check-excluded-lockfiles: $dir appears to be a library (no [[bin]]/src/main.rs) — lock not required, skipping." >&2
    continue
  fi

  checked=$((checked + 1))
  # Tracked == git knows the lock at HEAD. Empty ls-tree output == NOT tracked.
  if [ -n "$(git -C "$REPO_ROOT" ls-tree HEAD "$dir/Cargo.lock" 2>/dev/null)" ]; then
    echo "  ✅ $dir/Cargo.lock is committed."
  else
    echo "  ❌ $dir/Cargo.lock is NOT committed (workspace-excluded binary crate)." >&2
    fail=1
  fi
done <<< "$excludes"

if [ "$fail" -ne 0 ]; then
  echo "" >&2
  echo "FAIL: one or more workspace-excluded binary crates lack a committed Cargo.lock." >&2
  echo "      Without it, a cold CI build re-resolves deps to newest-in-range and can break silently." >&2
  echo "      Fix: cd <crate> (or cargo-linux.sh --manifest-path <crate>/Cargo.toml) to generate the lock, then git add + commit it." >&2
  echo "      Per .claude/lessons/feedback_workspace_excluded_crate_must_have_lockfile.md" >&2
  exit 1
fi

echo "check-excluded-lockfiles: OK ($checked excluded binary crate(s) checked, all locks committed)."
exit 0
