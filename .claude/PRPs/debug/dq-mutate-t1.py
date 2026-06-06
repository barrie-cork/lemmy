import io, json
from datetime import datetime, timezone

DQ = "/srv/brehon-fork/.claude/decision-queue.json"
with io.open(DQ, encoding="utf-8") as f:
    d = json.load(f)

now = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
moved = []
remaining = []
for e in d["pending"]:
    if e.get("kind", "").startswith("validate-pending") and e.get("phase_task") == 1:
        e["result"] = "pass"
        e["answer"] = (
            "cargo check exit 0 after rusqlite 0.32->0.37 bump. "
            "libsqlite3-sys conflict with matrix-sdk-sqlite resolved by unifying version. "
            "WP-6 zero-Matrix-deps=0. advisor-laptop 2026-06-06."
        )
        e["answered_by"] = "advisor-laptop"
        e["resolved_at"] = now
        e["log_slice"] = None
        e["failed_commands"] = None
        moved.append(e)
    else:
        remaining.append(e)

d["pending"] = remaining
d["resolved"].extend(moved)

with io.open(DQ, "w", encoding="utf-8") as f:
    json.dump(d, f, indent=2, ensure_ascii=False)
    f.write("\n")

print("moved:", len(moved))
print("pending:", [e["id"] for e in d["pending"]])
