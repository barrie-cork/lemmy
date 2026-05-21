#!/usr/bin/env bash
# resolve-dq-canonical.sh — produce a canonical merged decision-queue
# view for an in-flight phase, spanning phase-branch + active worker
# branches.
#
# Why this exists:
#   During an in-flight Brehon sub-phase, a DQ entry can live in any
#   of three places:
#     1. origin/phase-v1-<phase>          — phase-branch tip (updates
#                                            only on daemon finalize-merge)
#     2. origin/junior/role-*-<task>-<id> — worker-branch tip (updates
#                                            on every Junior push)
#     3. EliteDesk live worktree          — daemon-managed checkout that
#                                            sees both #1 and #2 because
#                                            the daemon pulls worker refs
#
#   A reconciliation that reads only #1 misses entries that exist on #2.
#   This is the bug pattern that surfaced 2026-05-09 during c-2 resume:
#   the live advisor session saw pending=2 (DQ #164 + #165 on worker
#   branch); a meta-editor session reading origin/phase-v1-SL-c-2 saw
#   pending=0. Both "right" — different refs.
#
#   The canonical answer: phase-DQ ⋃ all open worker-DQs, deduped by
#   entry id, worker-branch wins on collision (most recent).
#
# Schema-v3 note (post-v1-dq-schema-r1, 2026-05-21):
#   Pre-v3 entry ids are integers; v3 native entries use composite
#   `<session_id>-<seq>` strings (see .claude/rules/decision-queue.md
#   §Schema (v3)). The dedup-by-id + sort keys below coerce via
#   `str(e['id'])` so mixed int/str ids sort deterministically without
#   a TypeError. Migration adds `id_v1: <int>` as a back-compat alias
#   on pre-v3 entries; the resolver does NOT use `id_v1` for dedup —
#   only the canonical `id` field, coerced to str.
#
# Usage:
#   ./scripts/brehon/resolve-dq-canonical.sh <phase>
#
#   <phase> = e.g. "v1-SL-c-2"
#
# Output:
#   stdout: absolute path to the canonical merged DQ JSON file (in
#           same tmpdir as git-show-json.sh)
#   exit 0 on success, non-zero on irrecoverable failure
#
# Sources walked (in order):
#   1. .claude/auto-state/<phase>.json — extract current_cohort.members[].junior_id
#      and previous-cohort .junior_id values (in case worker branches
#      not yet finalize-merged still carry useful DQ context)
#   2. origin/phase-<phase> — phase tip (always read, may be ahead of
#      laptop's local checkout)
#   3. For each junior_id from step 1: origin/junior/role-*-<id>
#      (best-effort; missing branches are silently skipped)
#
# Edge cases:
#   - auto-state JSON missing (pre-/auto-phase / post-archive): falls
#     back to phase-DQ only; emits a warning to stderr but exits 0.
#   - phase branch missing (pre-bm-cut): exits 0 with empty merged file.
#   - worker branch missing on origin (already cleaned up): silently skip.
#   - duplicate ids across refs: worker-branch entry wins; phase-branch
#     entry kept only if no worker-branch entry has that id.
#
# This script is read-only. It does not git fetch (caller's job, so
# the laptop polling cadence is not perturbed). It does not mutate
# the auto-state JSON. It does not push.

set -euo pipefail

if [ "$#" -ne 1 ]; then
  echo "usage: $0 <phase>" >&2
  echo "  example: $0 v1-SL-c-2" >&2
  exit 2
fi

PHASE="$1"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
GIT_SHOW="${SCRIPT_DIR}/git-show-json.sh"
STATE_FILE="${REPO_ROOT}/.claude/auto-state/${PHASE}.json"
PHASE_REF="origin/phase-${PHASE}"

# UTF-8 for Python (Windows defaults to cp1252).
export PYTHONIOENCODING=utf-8
export PYTHONUTF8=1

# Pick the same tmpdir convention as git-show-json.sh
if [ -n "${LOCALAPPDATA:-}" ]; then
  TMPDIR_NATIVE="${LOCALAPPDATA//\\//}/Temp"
else
  TMPDIR_NATIVE="/tmp"
fi
mkdir -p "${TMPDIR_NATIVE}"

OUT_PATH="${TMPDIR_NATIVE}/dq-canonical-${PHASE}.json"

# Collect junior_ids from auto-state, if present.
#
# Note: the Python heredoc emits ids one per line. On Windows MSYS Bash,
# Python's `print()` defaults to text mode → trailing \r\n. The bash
# `while IFS= read -r jid` loop downstream treats \r as part of the
# value, which corrupts the grep pattern `[-]<jid>$` (becomes
# `[-]162\r$` and never matches the \r-free input). Force LF-only via
# `sys.stdout.reconfigure(newline='\n')` so the read loop sees clean ids.
# Defence in depth: `tr -d '\r'` after the heredoc strips any stragglers
# (e.g. if PYTHONIOENCODING is overridden or the Python is older). Caught
# 2026-05-09 during c-2 cohort-2 resume; canonical resolver was reporting
# sources=['phase-branch'] only despite worker-branch entries existing.
JUNIOR_IDS=""
if [ -f "${STATE_FILE}" ]; then
  JUNIOR_IDS="$(python3 - "${STATE_FILE}" <<'PY' | tr -d '\r'
import io, json, sys
sys.stdout.reconfigure(newline='\n')
path = sys.argv[1]
with io.open(path, encoding='utf-8') as f:
    state = json.load(f)
ids = set()
cohort = state.get('current_cohort') or {}
for m in cohort.get('members', []) or []:
    jid = m.get('junior_id')
    if jid:
        ids.add(jid)
    for fa in m.get('fix_attempts', []) or []:
        fjid = fa.get('junior_id')
        if fjid:
            ids.add(fjid)
        ci = fa.get('ci_watcher_junior_id')
        if ci:
            ids.add(ci)
    ci = m.get('ci_watcher_junior_id')
    if ci:
        ids.add(ci)
for jid in sorted(ids):
    print(jid)
PY
)"
else
  echo "warn: auto-state file missing at ${STATE_FILE}; phase-DQ only" >&2
fi

# Helper: append a single ref's DQ entries to the accumulator JSON
# (passed via stdin, output to stdout). Tracks every consulted source
# in _canonical_sources_consulted so the audit trail survives even
# when later refs overwrite per-entry labels via the dedup-by-id
# rule. Also tracks (ref, src_label) per entry — the latest source
# wins on collision (per the canonical-merge rule), but the global
# sources list grows monotonically.
merge_ref() {
  local ref="$1"
  local label="$2"

  local src_path
  if ! src_path="$(${GIT_SHOW} "${ref}" .claude/decision-queue.json 2>/dev/null)"; then
    return 0  # ref missing or git-show failed; skip silently
  fi

  python3 - "${OUT_PATH}" "${src_path}" "${label}" <<'PY'
import io, json, sys
acc_path, src_path, label = sys.argv[1], sys.argv[2], sys.argv[3]
with io.open(acc_path, encoding='utf-8') as f:
    acc = json.load(f)
with io.open(src_path, encoding='utf-8') as f:
    src = json.load(f)

# Record this source globally — survives later overwrites of
# per-entry _canonical_source labels.
sources = set(acc.get('_canonical_sources_consulted', []))
sources.add(label)

# Build id -> existing-source map for collision tracking
existing_pending = {e['id']: e for e in acc.get('pending', [])}
existing_resolved = {e['id']: e for e in acc.get('resolved', [])}

# Phase-branch is added first; worker-branches added later overwrite
# (worker wins per the canonical merge rule).
for e in src.get('pending', []):
    e['_canonical_source'] = label
    existing_pending[e['id']] = e
for e in src.get('resolved', []):
    e['_canonical_source'] = label
    existing_resolved[e['id']] = e

# An entry resolved on a worker branch but still pending on phase-branch
# (or vice versa) → resolved wins (more recent state).
for eid in list(existing_pending.keys()):
    if eid in existing_resolved:
        del existing_pending[eid]

acc['_canonical_sources_consulted'] = sorted(sources)
acc['pending'] = sorted(existing_pending.values(), key=lambda e: str(e['id']))
acc['resolved'] = sorted(existing_resolved.values(), key=lambda e: str(e['id']))
acc['schema_version'] = max(
    acc.get('schema_version', 1),
    src.get('schema_version', 1),
)

with io.open(acc_path, 'w', encoding='utf-8') as f:
    json.dump(acc, f, indent=2, ensure_ascii=False)
PY
}

# Initialize accumulator
python3 - "${OUT_PATH}" "${PHASE}" <<'PY'
import io, json, sys
out_path, phase = sys.argv[1], sys.argv[2]
init = {
    "_canonical_phase": phase,
    "_canonical_sources_consulted": [],
    "schema_version": 1,
    "pending": [],
    "resolved": [],
}
with io.open(out_path, 'w', encoding='utf-8') as f:
    json.dump(init, f, indent=2, ensure_ascii=False)
PY

# 1. Phase-branch DQ (always tried first; worker entries override on conflict)
merge_ref "${PHASE_REF}" "phase-branch"

# 2. Each worker branch named in auto-state.
#    List ALL junior/* refs once, then filter per junior_id by suffix
#    match. (Per-id glob `refs/heads/junior/*-<id>` works at command-line
#    but not reliably inside a heredoc-fed subshell on Windows MSYS Bash —
#    list-once-filter-many sidesteps the issue and is faster anyway.)
if [ -n "${JUNIOR_IDS}" ]; then
  ALL_JUNIOR_REFS="$(git -C "${REPO_ROOT}" ls-remote origin 'refs/heads/junior/*' 2>/dev/null | awk '{print $2}' | sed 's|^refs/heads/||')"
  while IFS= read -r jid; do
    [ -z "${jid}" ] && continue
    # `|| true` avoids set -e tripping on grep no-match (exit 1 = no
    # match, distinct from exit 2 = error; we want both to mean "skip")
    branch="$(echo "${ALL_JUNIOR_REFS}" | grep -E "[-]${jid}$" | head -1 || true)"
    if [ -n "${branch}" ]; then
      merge_ref "origin/${branch}" "worker-${jid}"
    fi
  done <<<"${JUNIOR_IDS}"
fi

echo "${OUT_PATH}"
