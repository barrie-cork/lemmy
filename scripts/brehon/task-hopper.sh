#!/usr/bin/env bash
# task-hopper.sh — per-task execution ledger for parallel-agent phases.
#
# Four verbs: start, retry, complete, escalate. Reads/writes
# .claude/task-hopper.json under a mkdir-lock. Auto-escalates to a GitHub
# issue when a retry pushes attempts past the configured cap.
#
# See plan: C:\Users\barri\.claude\plans\create-a-low-level-eager-hopper.md
# Rule:     .claude/rules/task-hopper.md
# Schema:   .claude/task-hopper.schema.json
#
# JSON edits go through Python (stdlib json). This keeps the helper
# portable across Linux CI and Windows-bash local envs without requiring
# jq. Python 3.8+ required.

set -euo pipefail

# ----- Locate hopper state, independent of CWD -----
script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "$script_dir/../.." && pwd)"
state_file="$repo_root/.claude/task-hopper.json"
lock_dir="$state_file.lock"
lock_owner_file="$state_file.lock.owner"
tmp_file="$state_file.tmp.$$"

# Per-process lock ownership token. PID + high-resolution timestamp gives
# uniqueness across retries in the same shell. Sibling owner file (not
# inside lock_dir) keeps rmdir semantics intact on the lock directory.
lock_token="$$-$(date +%s%N 2>/dev/null || date +%s)"

# Issue-creation target (mirrors .claude/rules/gh-pr-fork-target.md).
gh_repo="barrie-cork/lemmy"

# ----- Python interpreter -----
PY="${PYTHON:-python}"
if ! command -v "$PY" >/dev/null 2>&1; then
  if command -v python3 >/dev/null 2>&1; then PY=python3; else
    echo "task-hopper: Python interpreter not found on PATH" >&2
    exit 1
  fi
fi

err()  { printf 'task-hopper: %s\n' "$*" >&2; }
warn() { printf 'task-hopper: WARN: %s\n' "$*" >&2; }

usage() {
  cat >&2 <<'EOF'
Usage:
  task-hopper.sh start    <id> --agent <a> --kind <k> --layer <n> \
                          --label "<text>" --worktree <path>
  task-hopper.sh retry    <id> --reason "<text>" [--log <path>]
  task-hopper.sh complete <id> --commit-sha <sha> [--log <path>]
  task-hopper.sh escalate <id> --reason "<text>" [--log <path>]

Verbs
  start      Register a new attempt. Creates the task entry on first call;
             appends a new in_progress attempt on subsequent calls ONLY if
             the prior attempt is finalised (e.g. after retry).
  retry      Finalise the current attempt as 'failed' with the given
             reason. If attempts < cap, leaves the task in_progress. If
             attempts == cap, auto-escalates (files a GH issue, sets
             status=escalated, exits 4).
  complete   Finalise the current attempt as 'completed', record the
             commit SHA, set status=completed.
  escalate   Voluntary early escalation. Finalise current attempt as
             'abandoned', file a GH issue, set status=escalated, exit 4.

Environment
  CLAUDE_SESSION_ID            Recorded on each attempt for cross-session
                               race detection.
  CLAUDE_TRANSCRIPT_PATH       Recorded so escalation issues link back to
                               the agent's context.
  TASK_HOPPER_LOCK_TIMEOUT     Seconds to wait for the mkdir-lock (def 30).
  TASK_HOPPER_STALE_LOCK_AGE   Seconds after which a lock is assumed stale
                               and removed (default 60).
  TASK_HOPPER_NO_GH            If set, skip gh issue creation on escalate.
  PYTHON                       Override Python interpreter (default python,
                               falls back to python3).

Exit codes
  0   Verb succeeded (start/retry under cap/complete).
  1   Usage error.
  2   Validation / state-transition error.
  3   Lock acquisition failed.
  4   Escalation triggered (status=escalated). Caller should surface.
  5   Hopper state file missing or malformed.
EOF
}

# ----- Argument parsing -----
if [ $# -lt 2 ]; then usage; exit 1; fi

verb="$1"; shift
task_id="$1"; shift

agent=""
kind=""
layer=""
label=""
worktree=""
reason=""
log_path=""
commit_sha=""

while [ $# -gt 0 ]; do
  case "$1" in
    --agent)       agent="$2"; shift 2 ;;
    --kind)        kind="$2"; shift 2 ;;
    --layer)       layer="$2"; shift 2 ;;
    --label)       label="$2"; shift 2 ;;
    --worktree)    worktree="$2"; shift 2 ;;
    --reason)      reason="$2"; shift 2 ;;
    --log)         log_path="$2"; shift 2 ;;
    --commit-sha)  commit_sha="$2"; shift 2 ;;
    -h|--help)     usage; exit 0 ;;
    *) err "unknown flag: $1"; usage; exit 1 ;;
  esac
done

# Validate task_id shape
if ! printf '%s' "$task_id" | grep -Eq '^task-[a-z0-9-]+$'; then
  err "invalid task id '$task_id' — must match ^task-[a-z0-9-]+$"
  exit 1
fi

# ----- State-file existence -----
if [ ! -f "$state_file" ]; then
  err "hopper state file missing at $state_file"
  err "expected initial content: {\"schema_version\":1,\"phase\":\"<n>\",\"retry_caps\":{...},\"tasks\":[]}"
  exit 5
fi

# ----- Lock acquisition -----
lock_timeout="${TASK_HOPPER_LOCK_TIMEOUT:-30}"
stale_age_sec="${TASK_HOPPER_STALE_LOCK_AGE:-60}"

acquire_lock() {
  local waited=0
  local interval_ms=200
  local max_iters=$((lock_timeout * 1000 / interval_ms))
  while ! mkdir "$lock_dir" 2>/dev/null; do
    # Stale-lock detection: compare mtime to now.
    if [ -d "$lock_dir" ]; then
      local age
      age="$(
        "$PY" - "$lock_dir" "$stale_age_sec" <<'PY' 2>/dev/null
import os, sys, time
p = sys.argv[1]
threshold = int(sys.argv[2])
try:
    mtime = os.path.getmtime(p)
    age = time.time() - mtime
    print(int(age))
    sys.exit(0 if age >= threshold else 1)
except OSError:
    sys.exit(2)
PY
        echo "$?"
      )"
      # Last line of $age is the exit code; everything before it is age in sec.
      local rc="${age##*$'\n'}"
      if [ "$rc" = "0" ]; then
        warn "removing stale lock at $lock_dir (older than ${stale_age_sec}s)"
        rmdir "$lock_dir" 2>/dev/null || rm -rf "$lock_dir"
        # Clear the orphaned owner file from the dead process so our
        # own token (written after mkdir below) is the sole authority.
        rm -f "$lock_owner_file"
        continue
      fi
    fi
    sleep 0.2
    waited=$((waited + 1))
    if [ "$waited" -ge "$max_iters" ]; then
      err "could not acquire lock $lock_dir within ${lock_timeout}s"
      return 3
    fi
  done
  # Record our ownership token AFTER mkdir succeeds. mkdir is the atomic
  # acquire; the owner file is a compare-and-release marker so we never
  # remove a lock that was reclaimed out from under us.
  printf '%s\n' "$lock_token" > "$lock_owner_file"
}

release_lock() {
  # Compare-and-release. If another process reclaimed our lock as stale
  # and took it for themselves, the owner file will either be missing
  # (they cleared it during stale-reclaim) or hold their token, not ours.
  # In both cases we must leave the lock dir alone so we don't release a
  # lock that no longer belongs to us (CodeRabbit PR #46 critical).
  if [ -f "$lock_owner_file" ]; then
    local current
    current="$(cat "$lock_owner_file" 2>/dev/null || true)"
    if [ "$current" = "$lock_token" ]; then
      rm -f "$lock_owner_file"
      rmdir "$lock_dir" 2>/dev/null || true
    fi
  fi
  rm -f "$tmp_file"
}

# ----- Python edit helper -----
# Runs a Python snippet against the state file. The snippet is given
# CONTEXT via env vars and writes the mutated state JSON to stdout.
py_edit() {
  local op="$1"
  TH_OP="$op" \
  TH_STATE_FILE="$state_file" \
  TH_TASK_ID="$task_id" \
  TH_AGENT="$agent" \
  TH_KIND="$kind" \
  TH_LAYER="$layer" \
  TH_LABEL="$label" \
  TH_WORKTREE="$worktree" \
  TH_REASON="$reason" \
  TH_LOG="$log_path" \
  TH_COMMIT="$commit_sha" \
  TH_SESSION="${CLAUDE_SESSION_ID:-}" \
  TH_TRANSCRIPT="${CLAUDE_TRANSCRIPT_PATH:-}" \
  "$PY" - <<'PY'
import json, os, sys
from datetime import datetime, timezone

op        = os.environ["TH_OP"]
path      = os.environ["TH_STATE_FILE"]
task_id   = os.environ["TH_TASK_ID"]
agent     = os.environ.get("TH_AGENT", "")
kind      = os.environ.get("TH_KIND", "")
layer_s   = os.environ.get("TH_LAYER", "")
label     = os.environ.get("TH_LABEL", "")
worktree  = os.environ.get("TH_WORKTREE", "")
reason    = os.environ.get("TH_REASON", "") or None
log_path  = os.environ.get("TH_LOG", "") or None
commit    = os.environ.get("TH_COMMIT", "") or None
session   = os.environ.get("TH_SESSION", "") or None
transcript= os.environ.get("TH_TRANSCRIPT", "") or None

def now_iso():
    return datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")

with open(path, "r", encoding="utf-8") as f:
    state = json.load(f)

if state.get("schema_version") != 1:
    print("schema_version != 1", file=sys.stderr)
    sys.exit(5)

tasks = state.setdefault("tasks", [])
task = next((t for t in tasks if t["id"] == task_id), None)

def cap_for(k):
    by_kind = state.get("retry_caps", {}).get("by_kind", {})
    default = state.get("retry_caps", {}).get("default", 3)
    return int(by_kind.get(k, default))

def last_attempt(t):
    return t["attempts"][-1] if t.get("attempts") else None

def new_attempt(n):
    return {
        "n": n,
        "started_at": now_iso(),
        "ended_at": None,
        "result": "in_progress",
        "commit_sha": None,
        "log_path": None,
        "session_id": session,
        "transcript_path": transcript,
        "reason": None,
    }

def finalise(t, result, status, *, with_reason=None, with_log=None, with_commit=None):
    att = last_attempt(t)
    att["ended_at"] = now_iso()
    att["result"] = result
    if with_reason is not None: att["reason"] = with_reason
    if with_log is not None:    att["log_path"] = with_log
    if with_commit is not None: att["commit_sha"] = with_commit
    t["status"] = status
    t["updated_at"] = now_iso()

# Emit structured result on stdout as JSON so bash can branch.
def emit(outcome, **extra):
    result = {"outcome": outcome, **extra}
    # Serialise STATE then the trailer: state + "\n" + trailer-json
    print(json.dumps(state, indent=2, ensure_ascii=False))
    print("---TH-TRAILER---")
    print(json.dumps(result))
    sys.exit(0)

if op == "start":
    if task is None:
        # First-time creation — required flags checked in bash.
        if not all([agent, kind, layer_s, label, worktree]):
            print("first start requires all flags", file=sys.stderr); sys.exit(1)
        try:
            layer = int(layer_s)
        except ValueError:
            print("--layer must be integer", file=sys.stderr); sys.exit(1)
        task = {
            "id": task_id,
            "label": label,
            "layer": layer,
            "agent": agent,
            "kind": kind,
            "worktree": worktree,
            "status": "in_progress",
            "attempts": [new_attempt(1)],
            "issue_url": None,
            "created_at": now_iso(),
            "updated_at": now_iso(),
        }
        tasks.append(task)
        emit("started", attempt=1)
    else:
        status = task["status"]
        if status in ("completed", "escalated"):
            print(f"task {task_id} already {status}; refusing to re-start terminal task", file=sys.stderr)
            sys.exit(2)
        last = last_attempt(task)
        if last and last["result"] == "in_progress":
            # A start is already running. Refuse so we don't clobber an
            # in-flight attempt (double-start / concurrent-agent race).
            print(f"task {task_id} has an in-flight attempt; call retry or escalate first", file=sys.stderr)
            sys.exit(2)
        # Legitimate re-start: previous attempt finalised as failed/abandoned
        # (via retry); append a fresh in-progress attempt.
        task["attempts"].append(new_attempt(len(task["attempts"]) + 1))
        task["status"] = "in_progress"
        task["updated_at"] = now_iso()
        emit("started", attempt=len(task["attempts"]))

elif op == "retry":
    if task is None:
        print(f"task {task_id} not found; call start first", file=sys.stderr); sys.exit(2)
    if task["status"] != "in_progress":
        print(f"task {task_id} is not in_progress (current: {task['status']})", file=sys.stderr); sys.exit(2)
    if not reason:
        print("retry requires --reason", file=sys.stderr); sys.exit(1)
    finalise(task, "failed", "in_progress",
             with_reason=reason, with_log=log_path, with_commit=None)
    n = len(task["attempts"])
    cap = cap_for(task["kind"])
    if n >= cap:
        task["status"] = "escalated"
        task["updated_at"] = now_iso()
        emit("escalated_auto", attempts=n, cap=cap)
    emit("retried", attempt=n, cap=cap)

elif op == "complete":
    if task is None:
        print(f"task {task_id} not found; call start first", file=sys.stderr); sys.exit(2)
    if task["status"] != "in_progress":
        print(f"task {task_id} is not in_progress (current: {task['status']})", file=sys.stderr); sys.exit(2)
    if not commit:
        print("complete requires --commit-sha", file=sys.stderr); sys.exit(1)
    finalise(task, "completed", "completed",
             with_reason=None, with_log=log_path, with_commit=commit)
    emit("completed", commit=commit)

elif op == "escalate":
    if task is None:
        print(f"task {task_id} not found; call start first", file=sys.stderr); sys.exit(2)
    if task["status"] != "in_progress":
        print(f"task {task_id} is not in_progress (current: {task['status']})", file=sys.stderr); sys.exit(2)
    if not reason:
        print("escalate requires --reason", file=sys.stderr); sys.exit(1)
    finalise(task, "abandoned", "escalated",
             with_reason=reason, with_log=log_path, with_commit=None)
    emit("escalated_voluntary", attempts=len(task["attempts"]))

elif op == "set_issue_url":
    if task is None:
        print(f"task {task_id} not found", file=sys.stderr); sys.exit(2)
    url_env = os.environ.get("TH_ISSUE_URL", "").strip()
    task["issue_url"] = url_env or None
    task["updated_at"] = now_iso()
    emit("issue_url_set", url=(url_env or None))

else:
    print(f"unknown op: {op}", file=sys.stderr); sys.exit(1)
PY
}

# Split Python output (state + trailer) into $tmp_file and echo the trailer
# to stdout. Python on Windows emits \r\n line endings which break naive
# bash partitioning; we normalise line endings before splitting.
split_py_output() {
  local full="$1"
  TH_FULL="$full" TH_TMP="$tmp_file" "$PY" - <<'PY'
import os, sys
full = os.environ["TH_FULL"]
tmp  = os.environ["TH_TMP"]
# Normalise Windows-style line endings that Python's print() emits on this platform.
full = full.replace("\r\n", "\n").replace("\r", "\n")
marker = "---TH-TRAILER---"
marker_line = "\n" + marker + "\n"
idx = full.find(marker_line)
if idx == -1:
    # Try without leading \n (edge case: marker at start of output).
    idx2 = full.find(marker + "\n")
    if idx2 == 0:
        body, rest = "", full[len(marker) + 1:]
    else:
        sys.stderr.write("trailer marker missing in python output\n")
        sys.stderr.write(f"captured ({len(full)} bytes):\n{full!r}\n")
        sys.exit(1)
else:
    body = full[:idx]
    rest = full[idx + len(marker_line):]
with open(tmp, "w", encoding="utf-8", newline="\n") as f:
    f.write(body)
    if not body.endswith("\n"):
        f.write("\n")
sys.stdout.write(rest.strip() + "\n")
PY
}

# Atomic publish: rename tmp to state file.
publish_state() {
  mv "$tmp_file" "$state_file"
}

# ----- gh issue create (on escalation) -----
file_github_issue() {
  local attempts_total="$1"
  if [ -n "${TASK_HOPPER_NO_GH:-}" ]; then
    warn "TASK_HOPPER_NO_GH set; skipping gh issue create"
    return 0
  fi
  if ! command -v gh >/dev/null 2>&1; then
    warn "gh not on PATH; skipping issue creation (issue_url stays null)"
    return 0
  fi

  local phase_label title labels body head_sha
  phase_label="$("$PY" -c 'import json,sys;print(json.load(open(sys.argv[1],encoding="utf-8"))["phase"])' "$state_file")"
  head_sha="$(cd "$repo_root" && git rev-parse --short HEAD 2>/dev/null || echo unknown)"

  # Render the issue body via Python so the JSON lookup is robust.
  body="$(
    TH_STATE_FILE="$state_file" TH_TASK_ID="$task_id" TH_HEAD_SHA="$head_sha" TH_PHASE="$phase_label" \
    "$PY" - <<'PY'
import json, os
path = os.environ["TH_STATE_FILE"]
tid  = os.environ["TH_TASK_ID"]
sha  = os.environ["TH_HEAD_SHA"]
phase= os.environ["TH_PHASE"]
with open(path, encoding="utf-8") as f:
    state = json.load(f)
t = next((x for x in state["tasks"] if x["id"] == tid), None)
if t is None:
    raise SystemExit(1)
cap = state["retry_caps"]["by_kind"].get(t["kind"], state["retry_caps"]["default"])
lines = [
    "## Task",
    "",
    f"- **ID:** `{t['id']}`",
    f"- **Label:** {t['label']}",
    f"- **Layer:** {t['layer']}",
    f"- **Agent:** {t['agent']}",
    f"- **Kind:** {t['kind']}",
    f"- **Worktree:** `{t['worktree']}`",
    "",
    f"## Attempts ({len(t['attempts'])}/{cap})",
    "",
]
for a in t["attempts"]:
    lines.append(
        f"{a['n']}. **{a['started_at']}** → {a.get('ended_at') or '—'} — "
        f"{a['result']}: {a.get('reason') or '(no reason)'} "
        f"(log: {a.get('log_path') or '—'})"
    )
lines += [
    "",
    "## Last transcript",
    "",
    f"`{t['attempts'][-1].get('transcript_path') or '—'}`",
    "",
    "## Plan reference",
    "",
    f"`.claude/PRPs/plans/phase-{phase}-federation.plan.md` § {t['id']}",
    "",
    "## Hopper snapshot",
    "",
    f"`.claude/task-hopper.json` at commit `{sha}`",
    "",
    "---",
    "",
    "Auto-filed by `scripts/brehon/task-hopper.sh` on retry-cap exhaustion.",
    "Per `.claude/rules/task-hopper.md`.",
]
print("\n".join(lines))
PY
  )"

  title="Phase ${phase_label} ${task_id} blocked after ${attempts_total} attempts"
  # Pull agent + kind for labels.
  local agent_l kind_l
  agent_l="$("$PY" -c 'import json,sys;s=json.load(open(sys.argv[1],encoding="utf-8"));print(next(t["agent"] for t in s["tasks"] if t["id"]==sys.argv[2]))' "$state_file" "$task_id")"
  kind_l="$("$PY" -c 'import json,sys;s=json.load(open(sys.argv[1],encoding="utf-8"));print(next(t["kind"] for t in s["tasks"] if t["id"]==sys.argv[2]))' "$state_file" "$task_id")"
  labels="phase-${phase_label}/agent-blocked,agent:${agent_l},kind:${kind_l}"

  local url
  if url="$(gh issue create --repo "$gh_repo" --title "$title" --label "$labels" --body "$body" </dev/null 2>&1)"; then
    url="$(printf '%s\n' "$url" | grep -E '^https?://' | head -n1)"
    if [ -n "$url" ]; then
      # Write issue_url back into state (re-enter the python editor in set_issue_url mode).
      local out
      TH_ISSUE_URL="$url" out="$(py_edit set_issue_url)"
      split_py_output "$out" >/dev/null
      publish_state
      err "filed issue: $url"
    else
      warn "gh issue create exit 0 but no URL parsed; issue_url stays null"
    fi
  else
    warn "gh issue create failed: $url"
  fi
}

# ----- Main flow -----
trap 'release_lock' EXIT

acquire_lock || exit 3

# Run the requested op through Python, split output, publish atomically.
full_output="$(py_edit "$verb" || { rc=$?; rm -f "$tmp_file"; exit "$rc"; })"
trailer="$(split_py_output "$full_output")"
publish_state

# Parse trailer JSON to decide exit code / issue-filing.
outcome="$("$PY" -c 'import json,sys;print(json.loads(sys.argv[1]).get("outcome",""))' "$trailer")"

case "$outcome" in
  started)
    n="$("$PY" -c 'import json,sys;print(json.loads(sys.argv[1]).get("attempt",""))' "$trailer")"
    err "$task_id started (attempt $n)"
    exit 0
    ;;
  retried)
    n="$("$PY" -c 'import json,sys;d=json.loads(sys.argv[1]);print(d.get("attempt",""))' "$trailer")"
    cap="$("$PY" -c 'import json,sys;d=json.loads(sys.argv[1]);print(d.get("cap",""))' "$trailer")"
    err "$task_id retry recorded (attempt $n/$cap failed; will re-attempt on next start)"
    exit 0
    ;;
  completed)
    sha="$("$PY" -c 'import json,sys;print(json.loads(sys.argv[1]).get("commit",""))' "$trailer")"
    err "$task_id completed at $sha"
    exit 0
    ;;
  escalated_auto)
    n="$("$PY" -c 'import json,sys;d=json.loads(sys.argv[1]);print(d.get("attempts",""))' "$trailer")"
    cap="$("$PY" -c 'import json,sys;d=json.loads(sys.argv[1]);print(d.get("cap",""))' "$trailer")"
    err "$task_id escalated after $n attempts (cap=$cap)"
    file_github_issue "$n"
    exit 4
    ;;
  escalated_voluntary)
    n="$("$PY" -c 'import json,sys;print(json.loads(sys.argv[1]).get("attempts",""))' "$trailer")"
    err "$task_id escalated voluntarily after $n attempts"
    file_github_issue "$n"
    exit 4
    ;;
  issue_url_set)
    # internal reentry; caller already logged
    exit 0
    ;;
  *)
    err "unknown outcome from python: $outcome"
    exit 2
    ;;
esac
