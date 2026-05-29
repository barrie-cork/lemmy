#!/usr/bin/env bash
# verify-rule-anchors.sh — enforce the Pi-citation freeze map.
#
# WHAT: scans every cross-harness prose pointer that cites a NAMED SECTION of a
# Claude `.claude/rules/*.md` file, and verifies the cited section name still
# resolves to a heading (or at least literal text) in the target file. A citation
# whose named anchor no longer exists is a SILENT dangling reference — the citing
# harness (Pi subagent, Claude skill/command) sends a reader to a section that
# isn't there.
#
# WHY: the brehon-fork rules corpus is a DUAL-HARNESS contract. `.pi/` prompts +
# skills cite Claude rule sections BY PROSE NAME (e.g. brehon-clarify.md cites
# advisor-orchestrator.md "Stage-shape orchestration"; ci-watcher SKILL cites the
# decision-queue.md "ci-watcher mutation pattern" ×11). Moving or renaming a cited
# heading during a context-budget relocation breaks the other harness with zero
# runtime signal. This guard makes the freeze map ENFORCED, not just documented —
# the highest-leverage protection during any rules-corpus relocation.
# Per .claude/PRPs/reports/harness-redesign-session-profiles-2026-05-29.md §5 rec 3
# + retro-harvest eval O11 (guard flagged absent).
#
# USAGE:
#   bash scripts/brehon/verify-rule-anchors.sh            # scan all, exit 1 on any dangle
#   bash scripts/brehon/verify-rule-anchors.sh --list     # print the citation->anchor map, exit 0
#   VERIFY_ANCHORS_STRICT=0 bash ...                      # warn-only (exit 0 even on dangle)
#
# EXIT: 0 = all cited anchors resolve (or --list / non-strict). 1 = ≥1 dangling
# citation found in strict mode (default).
#
# SCOPE: the 5 big dual-harness rule files. Citations are sourced from .pi/ +
# .claude/{skills,commands,agents,refs} + ~/.claude/{commands,skills}. Report prose
# (.claude/PRPs/reports/**) and lesson WATCH-notes are EXCLUDED — they discuss
# anchors descriptively, they don't depend on them at runtime.

set -o pipefail

REPO_ROOT="${CLAUDE_PROJECT_DIR:-$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)}"
cd "$REPO_ROOT" || { echo "verify-rule-anchors: cannot cd to repo root" >&2; exit 2; }

STRICT="${VERIFY_ANCHORS_STRICT:-1}"
MODE="scan"
[ "$1" = "--list" ] && MODE="list"

# The dual-harness rule files whose section names are cited cross-harness.
RULE_FILES=(
  ".claude/rules/advisor-orchestrator.md"
  ".claude/rules/decision-queue.md"
  ".claude/rules/multi-lane-worktree.md"
  ".claude/rules/branch-manager.md"
  ".claude/rules/pmd-invariants.md"
)

# Citation source trees. ~/.claude is user-scope (commands + skills the advisor uses).
SOURCE_DIRS=(
  ".pi"
  ".claude/skills"
  ".claude/commands"
  ".claude/agents"
  ".claude/refs"
  "$HOME/.claude/commands"
  "$HOME/.claude/skills"
)

# Excluded path fragments: eval/test fixtures + report prose discuss anchors
# DESCRIPTIVELY (as examples / proposals), they do not depend on them at runtime.
# A dangle inside an eval fixture is not a broken cross-harness contract.
EXCLUDE_FRAGMENTS='/reports/|/retro-harvest-workspace/|/iteration-[0-9]|/eval-[0-9]|verify-rule-anchors'

# Build a regex alternation of the rule basenames (without .md) for citation matching.
basenames_re=""
for f in "${RULE_FILES[@]}"; do
  b="$(basename "$f" .md)"
  basenames_re="${basenames_re}${basenames_re:+|}${b}"
done

# Extract candidate (file, quoted-anchor) citations.
# A citation looks like:  <basename>.md ... "Quoted Section Name"
# or                      <basename>.md ... §"Quoted Section Name"
# We capture the FIRST quoted string that appears on the same line as the file ref.
# This is a heuristic; --list lets a human eyeball the full map.
collect_citations() {
  local d
  for d in "${SOURCE_DIRS[@]}"; do
    [ -d "$d" ] || continue
    # grep lines mentioning a rule basename; exclude report/eval prose + the guard's own doc.
    grep -rn -E "(${basenames_re})\.md" "$d" 2>/dev/null \
      | grep -vE "$EXCLUDE_FRAGMENTS"
  done
}

# Given a citing line, emit "FILE<TAB>ANCHOR" pairs for any quoted section name in
# ANCHOR POSITION relative to the file reference. Anchor position = the quoted name
# appears AFTER the `<basename>.md` token, optionally separated by a `§`, the word
# "section", or punctuation — but on the SAME side as the file ref. A quote that
# floats elsewhere on the line (a concept label, a DQ field value, an example
# phrase) is NOT an anchor. This adjacency rule kills the false positives from
# lines that merely mention a rule file and also happen to quote something.
parse_line() {
  local line="$1"
  local b file=""
  for b in advisor-orchestrator decision-queue multi-lane-worktree branch-manager pmd-invariants; do
    case "$line" in *"$b.md"*) file=".claude/rules/$b.md";; esac
    [ -n "$file" ] && break
  done
  [ -z "$file" ] && return 0
  local bn; bn="$(basename "$file")"

  # Take only the substring AFTER the first occurrence of "<basename>.md".
  # Anchors are cited downstream of the file token: `<bn>.md "Name"`, `<bn>.md §"Name"`,
  # `<bn>.md §Name`, `<bn>.md "X" + "Y"` (verbs that cite two sections).
  local after="${line#*${bn}}"

  # Pull quoted strings from the AFTER-substring only, AND only the first 3 (a line
  # citing >3 quoted sections after one file ref is almost always prose, not pointers).
  printf '%s\n' "$after" \
    | grep -oE '"[^"]+"' \
    | head -3 \
    | while IFS= read -r q; do
        local a="${q#\"}"; a="${a%\"}"
        [ "${#a}" -le 4 ] && continue
        # Reject anything that is clearly not a section name.
        case "$a" in
          *"/"*|*"("*|*"json"*|*"import "*|*".claude"*|*"chore("*|*"DQ #"*|*"§13"*) continue;;
          *"  "*) continue;;  # multi-space => sentence/quote, not a heading
        esac
        # An anchor name is short-ish (headings are ≤~6 words). Reject long sentences.
        local words; words=$(printf '%s' "$a" | wc -w)
        [ "$words" -gt 7 ] && continue
        # Skip bare DQ field-value words.
        case "$a" in
          advisor|user|planner|blocker|clarify|impl|bm|ci-watcher|log) continue;;
          auto-fixable|surface-to-user) continue;;  # concept labels in ci-watcher SKILL, not headings
          "bm-self-resolved"|"impl-self-resolved") continue;;
        esac
        printf '%s\t%s\n' "$file" "$a"
      done
}

# Does ANCHOR resolve in FILE? Resolution test, in priority order:
#  1. Exact heading match: a line `## <anchor>` or `### <anchor>` (anchor may be a
#     prefix of the heading, e.g. cite "Stage-shape orchestration" vs heading
#     "### 3.1 Stage-shape orchestration").
#  2. Heading contains the anchor text (case-insensitive substring on a heading line).
#  3. Literal text anywhere in the file (a cited phrase that's prose, not a heading —
#     weaker but not a dangle; flagged as TEXT-ONLY in --list).
# Returns: "HEADING" | "TEXT" | "MISSING"
resolve_anchor() {
  local file="$1" anchor="$2"
  [ -f "$file" ] || { echo "MISSING"; return; }
  # 1+2: heading lines containing the anchor (case-insensitive)
  if grep -iE '^#{2,4} ' "$file" | grep -iqF "$anchor"; then
    echo "HEADING"; return
  fi
  # 3: literal text anywhere
  if grep -iqF "$anchor" "$file"; then
    echo "TEXT"; return
  fi
  echo "MISSING"
}

# ---- main ----
declare -a DANGLES=()
declare -a TEXTONLY=()
declare -a OK=()
seen=""

while IFS= read -r raw; do
  # raw = "path:lineno:content"; strip the path:lineno prefix for parsing,
  # but keep it for reporting.
  src="${raw%%:*}"
  rest="${raw#*:}"
  lineno="${rest%%:*}"
  content="${rest#*:}"
  while IFS=$'\t' read -r file anchor; do
    [ -z "$file" ] && continue
    key="${file}::${anchor}"
    case "$seen" in *"|$key|"*) continue;; esac
    seen="${seen}|$key|"
    verdict="$(resolve_anchor "$file" "$anchor")"
    case "$verdict" in
      HEADING) OK+=("$file  §\"$anchor\"  (cited ${src}:${lineno})");;
      TEXT)    TEXTONLY+=("$file  §\"$anchor\"  → TEXT-ONLY (no heading; cited ${src}:${lineno})");;
      MISSING) DANGLES+=("$file  §\"$anchor\"  → MISSING  (cited ${src}:${lineno})");;
    esac
  done < <(parse_line "$content")
done < <(collect_citations)

if [ "$MODE" = "list" ]; then
  echo "=== Pi/Claude-side section-name citations of the 5 dual-harness rule files ==="
  echo ""
  echo "-- RESOLVES TO HEADING (frozen, safe) --"
  printf '%s\n' "${OK[@]}" | sort -u
  echo ""
  echo "-- TEXT-ONLY (cited phrase exists but is NOT a heading; relocation must preserve the phrase) --"
  printf '%s\n' "${TEXTONLY[@]}" | sort -u
  echo ""
  echo "-- MISSING (DANGLING — cited section does not exist in target file) --"
  if [ "${#DANGLES[@]}" -eq 0 ]; then echo "(none)"; else printf '%s\n' "${DANGLES[@]}" | sort -u; fi
  exit 0
fi

# scan mode
if [ "${#DANGLES[@]}" -gt 0 ]; then
  echo "verify-rule-anchors: ${#DANGLES[@]} DANGLING cross-harness citation(s) found:" >&2
  printf '  %s\n' "${DANGLES[@]}" | sort -u >&2
  echo "" >&2
  echo "A cited section name no longer resolves to a heading or text in its target rule file." >&2
  echo "Either restore the anchor, or fix the citing prose. Run with --list for the full map." >&2
  [ "$STRICT" = "1" ] && exit 1
fi
echo "verify-rule-anchors: OK — all ${#OK[@]} heading-resolved + ${#TEXTONLY[@]} text-only citations resolve. 0 dangling."
exit 0
