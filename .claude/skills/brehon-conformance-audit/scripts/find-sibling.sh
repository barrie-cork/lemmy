#!/usr/bin/env bash
# find-sibling.sh — in-file/module/crate sibling locator for brehon-conformance-audit
#
# Walks three strategies (cheapest first):
#   1. In-file:   Grep + Read same target_file, rank by signature-type similarity
#   2. In-module: Glob sibling .rs files in the same directory
#   3. In-crate:  All .rs files under the crate root (walking up to Cargo.toml)
#
# V2 TODO (LSP path): rust-analyzer-mcp invocation via JSON-RPC over stdio
#   (`mcp__rust-analyzer__documentSymbol` + `mcp__rust-analyzer__workspace_diagnostics`).
#   MCP is JSON-RPC over stdio, not a CLI tool — bash cannot invoke it directly.
#   For v1 the per-axis sub-files (Task 2) invoke LSP via the agent's `LSP` tool.
#   This script uses Grep+Read (bash + Python) as the primary path.
#   Per plan brehon-conformance-audit.plan.md §10.4 + Task 3 GOTCHA.
#
# Output (stdout):
#   "<file>:<line>:<signature>"  — best-ranked sibling found (exit 0)
#   "NO_SIBLING_FOUND"           — no sibling found (exit 0; NOT exit 1)
#
# Errors (stderr, exit 2):
#   "ERROR: ..."                 — missing/invalid arguments
#
# Exit codes:
#   0  — success (including NO_SIBLING_FOUND — empty result is valid)
#   2  — bad arguments (missing args, target_file not found)
#
# Usage:
#   find-sibling.sh <target_file> <target_fn>
#
# Consumed by: compute-metrics.sh (Task 6) — output format is load-bearing:
#   printf '%s:%d:%s\n' "$sibling_file" "$sibling_line" "$sibling_sig"
#
# Per pattern_verify_before_trusting_shell_output: exit codes validated, not trusted.
set -euo pipefail

# --- Argument validation (exit 2 on bad args) ---
if [ "$#" -lt 2 ] || [ -z "${1:-}" ] || [ -z "${2:-}" ]; then
  printf 'ERROR: usage: find-sibling.sh <target_file> <target_fn>\n' >&2
  exit 2
fi

TARGET_FILE="$1"
TARGET_FN="$2"

if [ ! -f "$TARGET_FILE" ]; then
  printf 'ERROR: target_file not found: %s\n' "$TARGET_FILE" >&2
  exit 2
fi

# --- Python: signature extraction + similarity scoring ---
# Variables passed via env to avoid quoting/escaping issues in the heredoc.
# All three strategies run inside Python to keep the logic in one place.
result=$(TARGET_FILE="$TARGET_FILE" TARGET_FN="$TARGET_FN" python3 << 'PYEOF'
import os
import re
import sys
import glob as glob_mod

target_file = os.environ['TARGET_FILE']
target_fn   = os.environ['TARGET_FN']


def extract_fn_sigs(filepath):
    """Return list of (lineno_1based, name, full_sig_text) for top-level fns.

    'Top-level' means the fn keyword starts at column 0 (no leading whitespace),
    which excludes nested functions, closures, and method bodies in Rust.
    """
    try:
        content = open(filepath, encoding='utf-8').read()
    except Exception:
        return []
    lines = content.splitlines()
    results = []
    # Anchored at ^: only matches lines where pub/async/fn starts at col 0.
    fn_pat = re.compile(r'^(?:pub\s+)?(?:async\s+)?fn\s+(\w+)')
    i = 0
    while i < len(lines):
        m = fn_pat.match(lines[i])
        if m:
            name = m.group(1)
            lineno = i + 1  # 1-based
            # Gather signature lines: from this line until the opening '{'.
            sig_parts = []
            j = i
            while j < len(lines):
                sig_parts.append(lines[j].strip())
                if '{' in lines[j]:
                    break
                j += 1
            full_sig = ' '.join(sig_parts)
            # Trim at the first '{' to remove the function body.
            brace_idx = full_sig.find('{')
            if brace_idx >= 0:
                full_sig = full_sig[:brace_idx].rstrip()
            results.append((lineno, name, full_sig))
        i += 1
    return results


def type_tokens(sig):
    """Extract CamelCase type-like tokens from a function signature.

    Uses uppercase-first as a simple heuristic to identify type names
    (e.g. Data, LemmyContext, LemmyResult, PublishLabel) while excluding
    keywords (pub, async, fn, let) and variable names (activity, context).
    """
    tokens = re.findall(r'[A-Z][A-Za-z0-9_]*', sig)
    return set(tokens)


def score_similarity(target_tokens, candidate_sig):
    """Jaccard-like score: shared type tokens / union of type tokens."""
    cand_tokens = type_tokens(candidate_sig)
    if not target_tokens or not cand_tokens:
        return 0.0
    intersection = len(target_tokens & cand_tokens)
    union = len(target_tokens | cand_tokens)
    return intersection / union


# --- Step 1: find target_fn in target_file ---
target_sigs = extract_fn_sigs(target_file)
target_entry = None
for lineno, name, sig in target_sigs:
    if name == target_fn:
        target_entry = (lineno, name, sig)
        break

if target_entry is None:
    # Target function not found in target_file — no sibling to locate.
    print('NO_SIBLING_FOUND')
    sys.exit(0)

target_tokens = type_tokens(target_entry[2])

# Tracking best match across all strategies.
best_file  = None
best_line  = None
best_sig   = None
best_score = 0.0


def update_best(filepath, sigs):
    """Check sigs in filepath; update best_* globals if a better match is found."""
    global best_file, best_line, best_sig, best_score
    for lineno, name, sig in sigs:
        # Skip the target function itself.
        if filepath == target_file and name == target_fn:
            continue
        score = score_similarity(target_tokens, sig)
        if score > best_score:
            best_score = score
            best_file  = filepath
            best_line  = lineno
            best_sig   = sig


# --- Strategy 1: in-file (same file, excluding target_fn) ---
update_best(target_file, target_sigs)

if best_file is not None and best_score > 0:
    print(f"{best_file}:{best_line}:{best_sig}")
    sys.exit(0)

# --- Strategy 2: in-module (sibling .rs files in same directory) ---
target_dir   = os.path.dirname(os.path.abspath(target_file))
target_abs   = os.path.abspath(target_file)
sibling_abs  = set()

for sibling_file in sorted(glob_mod.glob(os.path.join(target_dir, '*.rs'))):
    abs_sib = os.path.abspath(sibling_file)
    if abs_sib == target_abs:
        continue
    sibling_abs.add(abs_sib)
    update_best(sibling_file, extract_fn_sigs(sibling_file))

if best_file is not None and best_score > 0:
    print(f"{best_file}:{best_line}:{best_sig}")
    sys.exit(0)

# --- Strategy 3: in-crate (all .rs files under crate root) ---
# V2 TODO (LSP): rust-analyzer-mcp documentSymbol + references.
# V1 fallback: walk up from target_file to find the dir containing Cargo.toml.
def find_crate_root(filepath):
    """Walk up directory tree until we find a dir with Cargo.toml."""
    d = os.path.dirname(os.path.abspath(filepath))
    while True:
        if os.path.exists(os.path.join(d, 'Cargo.toml')):
            return d
        parent = os.path.dirname(d)
        if parent == d:
            break
        d = parent
    return os.path.dirname(os.path.abspath(filepath))

crate_root = find_crate_root(target_file)
for rs_file in sorted(glob_mod.glob(os.path.join(crate_root, '**', '*.rs'), recursive=True)):
    abs_rs = os.path.abspath(rs_file)
    if abs_rs == target_abs or abs_rs in sibling_abs:
        continue
    update_best(rs_file, extract_fn_sigs(rs_file))

if best_file is not None and best_score > 0:
    print(f"{best_file}:{best_line}:{best_sig}")
    sys.exit(0)

print('NO_SIBLING_FOUND')
PYEOF
)

printf '%s\n' "$result"
