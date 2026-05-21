#!/usr/bin/env bash
# Usage: compute-metrics.sh <metrics-file.json> [<metrics-file.json>...]
# Reads one or more audit-metrics JSON files; computes per-axis precision/recall,
# lead-time, and latent-footgun catch rate. Writes summary to stdout.
# Does NOT mutate input files.
# NO cargo invocation (Watchpoint #3).
#
# Schema validation: uses the jsonschema Python package when available;
# falls back to lightweight manual key/enum/length checks if not installed.
#
# Lead-time calculation: requires git log. Works from any git worktree per
# multi-lane-worktree.md — the canonical .git/ is shared across all lanes.
#
# This is the first bash+python-wrapper script in scripts/brehon-conformance-audit/.
# Pattern: set SCRIPT_DIR in bash, export env vars for Python, delegate
# all computation to a Python3 heredoc for cross-platform portability.
set -euo pipefail

if [ "$#" -eq 0 ]; then
  printf 'Usage: compute-metrics.sh <metrics-file.json> [<metrics-file.json>...]\n' >&2
  exit 1
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
export BREHON_SCHEMA_FILE="$SCRIPT_DIR/../audit-metrics.schema.json"

python3 - "$@" <<'PYEOF'
import sys
import json
import os
import subprocess
import datetime
import statistics

SCHEMA_FILE = os.environ.get("BREHON_SCHEMA_FILE", "")
VALID_AXES = {"1", "2", "3", "4", "5", "6"}


def _manual_validate(data, path):
    required = {"schema_version", "scope", "head_sha", "run_at", "skill_version", "predictions"}
    missing = required - set(data.keys())
    if missing:
        raise ValueError(f"{path}: missing required keys: {missing}")
    if data.get("schema_version") != 1:
        raise ValueError(
            f"{path}: schema_version must be 1, got {data.get('schema_version')!r}"
        )
    for pred in data.get("predictions", []):
        axis = pred.get("axis")
        if axis not in VALID_AXES:
            raise ValueError(
                f"{path}: prediction axis {axis!r} not in {sorted(VALID_AXES)}"
            )
        evidence = pred.get("evidence", "")
        if len(evidence) > 120:
            raise ValueError(
                f"{path}: prediction evidence too long ({len(evidence)} > 120 chars)"
            )


def validate_schema(data, path):
    try:
        import jsonschema
        if os.path.isfile(SCHEMA_FILE):
            with open(SCHEMA_FILE, encoding="utf-8") as f:
                schema = json.load(f)
            jsonschema.validate(data, schema)
        else:
            _manual_validate(data, path)
    except ImportError:
        _manual_validate(data, path)


def git_author_date(sha):
    """Return tz-aware datetime for sha's author date, or None on failure."""
    try:
        result = subprocess.run(
            ["git", "log", "-1", "--format=%aI", sha],
            capture_output=True, text=True, timeout=10
        )
        date_str = result.stdout.strip()
        if not date_str:
            return None
        dt = datetime.datetime.fromisoformat(date_str.replace("Z", "+00:00"))
        if dt.tzinfo is None:
            dt = dt.replace(tzinfo=datetime.timezone.utc)
        return dt
    except Exception:
        return None


def compute_metrics(files):
    all_data = []
    for path in files:
        try:
            with open(path, encoding="utf-8") as f:
                data = json.load(f)
        except (OSError, json.JSONDecodeError) as e:
            print(f"Error loading {path}: {e}", file=sys.stderr)
            sys.exit(1)
        try:
            validate_schema(data, path)
        except Exception as e:
            print(f"Schema validation failed for {path}: {e}", file=sys.stderr)
            sys.exit(1)
        all_data.append((path, data))

    # Aggregate counts across all files
    axis_tp = {a: 0 for a in VALID_AXES}
    axis_fp = {a: 0 for a in VALID_AXES}  # human-confirmed false positives only
    axis_fn = {a: 0 for a in VALID_AXES}  # ground-truth entries with no matching prediction
    lead_times = []   # seconds; negative = skill flagged before event (good)
    latent_footgun_count = 0

    for _path, data in all_data:
        predictions = data.get("predictions", [])
        gt_compile = data.get("ground_truth_compile_caught", [])
        gt_runtime = data.get("ground_truth_runtime", [])
        false_positives = data.get("false_positives", [])

        # Per-axis precision/recall counts
        for axis in VALID_AXES:
            pred_targets = {p.get("target") for p in predictions if p.get("axis") == axis}
            gt_targets = {
                e.get("target")
                for e in (gt_compile + gt_runtime)
                if e.get("axis") == axis
            }
            axis_tp[axis] += len(pred_targets & gt_targets)
            fp_targets = {e.get("target") for e in false_positives if e.get("axis") == axis}
            axis_fp[axis] += len(pred_targets & fp_targets)
            axis_fn[axis] += len(gt_targets - pred_targets)

        # Lead time: wall-clock between skill run_at and each runtime GT event
        run_at = None
        run_at_str = data.get("run_at", "")
        if run_at_str:
            try:
                run_at = datetime.datetime.fromisoformat(run_at_str.replace("Z", "+00:00"))
                if run_at.tzinfo is None:
                    run_at = run_at.replace(tzinfo=datetime.timezone.utc)
            except ValueError:
                pass

        for gt_entry in (gt_compile + gt_runtime):
            sha = gt_entry.get("evidence_commit_sha")
            if sha and run_at:
                event_date = git_author_date(sha)
                if event_date:
                    lead_times.append((run_at - event_date).total_seconds())

        # Latent-footgun: axis-4 predictions confirmed by runtime GT but NOT by compile GT
        axis4_pred = {p.get("target") for p in predictions if p.get("axis") == "4"}
        axis4_rt = {e.get("target") for e in gt_runtime if e.get("axis") == "4"}
        axis4_cc = {e.get("target") for e in gt_compile if e.get("axis") == "4"}
        latent_footgun_count += len(axis4_pred & axis4_rt - axis4_cc)

    # --- Output ---
    print("=" * 60)
    print("Brehon Conformance-Audit Metrics")
    print("=" * 60)
    print(f"Files: {', '.join(p for p, _ in all_data)}")
    print()

    active_axes = [
        a for a in sorted(VALID_AXES)
        if axis_tp[a] + axis_fp[a] + axis_fn[a] > 0
    ]

    if active_axes:
        # Summary table
        print(f"{'Axis':<8}  {'TP':>4}  {'FP':>4}  {'FN':>4}  {'Precision':>9}  {'Recall':>7}")
        print("-" * 56)
        for axis in active_axes:
            tp = axis_tp[axis]
            fp = axis_fp[axis]
            fn = axis_fn[axis]
            prec_denom = tp + fp
            rec_denom = tp + fn
            prec = tp / prec_denom if prec_denom > 0 else float("nan")
            rec = tp / rec_denom if rec_denom > 0 else float("nan")
            prec_str = f"{prec:.3f}" if prec == prec else "  n/a"
            rec_str = f"{rec:.3f}" if rec == rec else "  n/a"
            print(f"axis-{axis:<3}   {tp:>4}  {fp:>4}  {fn:>4}  {prec_str:>9}  {rec_str:>7}")
        print()

        # Canonical key=value lines (format consumed by test validation + retro §5)
        for axis in active_axes:
            tp = axis_tp[axis]
            fp = axis_fp[axis]
            fn = axis_fn[axis]
            prec_denom = tp + fp
            rec_denom = tp + fn
            prec = tp / prec_denom if prec_denom > 0 else float("nan")
            rec = tp / rec_denom if rec_denom > 0 else float("nan")
            prec_str = f"{prec:.3f}" if prec == prec else "n/a"
            rec_str = f"{rec:.3f}" if rec == rec else "n/a"
            print(f"axis-{axis} precision: {prec_str}")
            print(f"axis-{axis} recall: {rec_str}")
        print()
    else:
        print("(no predictions or ground-truth entries found)")
        print()

    # Lead time
    if lead_times:
        median_lt = statistics.median(lead_times)
        abs_h = abs(median_lt) / 3600
        direction = "before" if median_lt < 0 else "after"
        print(
            f"Lead time (median): {abs_h:.1f}h {direction} ground-truth event "
            f"({len(lead_times)} data point(s))"
        )
    else:
        print("Lead time: n/a (no runtime ground-truth with evidence_commit_sha)")
    print()

    # Latent-footgun catch rate
    print(f"Latent-footgun catch rate (axis-4): {latent_footgun_count}")
    if latent_footgun_count > 0:
        print("  (axis-4 findings the skill caught but the compiler missed)")
    print()
    print("=" * 60)


if __name__ == "__main__":
    files = sys.argv[1:]
    if not files:
        print(
            "Usage: compute-metrics.sh <metrics-file.json> [<metrics-file.json>...]",
            file=sys.stderr,
        )
        sys.exit(1)
    compute_metrics(files)
PYEOF
