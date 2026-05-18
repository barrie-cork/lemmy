# EliteDesk recovery paste — daemon-local ref fix (2026-05-17)

Context: an advisor `reset --hard` hit a TOCTOU race (daemon rotated the
shared /srv/brehon-fork checkout mid-command), corrupting daemon-LOCAL
refs `phase-v1-AD-e` and leaving `phase-v1-ship-1` unsynced. ZERO data
loss; nothing pushed; origin + PR #133 intact. Daemon is STOPPED.

Run this whole block in a terminal ON the EliteDesk (or via your other
session that has EliteDesk shell access). It is CAS-guarded — if a guard
aborts, STOP and report the actual SHA; do not force.

---

```bash
cd /srv/brehon-fork

# 0. Confirm daemon is down before touching anything
systemctl is-active junior@brehon-fork.service
# expect: failed  (cosmetic SIGTERM; app exited 0/SUCCESS = stopped). If "active", STOP.

# 1. Pre-state snapshot
echo "PRE: AD-e=$(git rev-parse --short phase-v1-AD-e) ship1=$(git rev-parse --short phase-v1-ship-1) checkout=$(git branch --show-current)"
HOOK_PRE=$(sha256sum .claude/hooks/allow-prp-deliverables.sh | cut -d' ' -f1); echo "HOOK_PRE=$HOOK_PRE"
git fetch origin phase-v1-AD-e phase-v1-ship-1 --quiet
echo "origin AD-e=$(git rev-parse --short origin/phase-v1-AD-e) origin ship1=$(git rev-parse --short origin/phase-v1-ship-1)"

# 2. (a) Restore AD-e daemon-local -> authoritative origin
#        CAS-guarded from the contaminated 8f6387e9a
git update-ref refs/heads/phase-v1-AD-e "$(git rev-parse origin/phase-v1-AD-e)" 8f6387e9a6896ddf068e38fee1e112e246e50e99

# 3. (b) Sync ship-1 daemon-local -> origin (original goal)
#        CAS-guarded from 0decd9971
git update-ref refs/heads/phase-v1-ship-1 "$(git rev-parse origin/phase-v1-ship-1)" 0decd9971c7d4dcfbc36f5b7176b947b52f9b633

# 4. Post-verify
echo "POST AD-e=$(git rev-parse --short phase-v1-AD-e) expect=$(git rev-parse --short origin/phase-v1-AD-e)"
echo "POST ship1=$(git rev-parse --short phase-v1-ship-1) expect=$(git rev-parse --short origin/phase-v1-ship-1)"
[ "$(git rev-parse phase-v1-AD-e)" = "$(git rev-parse origin/phase-v1-AD-e)" ] && echo "AD-e OK" || echo "AD-e FAIL"
[ "$(git rev-parse phase-v1-ship-1)" = "$(git rev-parse origin/phase-v1-ship-1)" ] && echo "ship-1 OK" || echo "ship-1 FAIL"
HOOK_POST=$(sha256sum .claude/hooks/allow-prp-deliverables.sh 2>/dev/null | cut -d' ' -f1)
[ "$HOOK_PRE" = "$HOOK_POST" ] && echo "HOOK PRESERVED" || echo "HOOK CHANGED"
echo "tree:"; git status --porcelain
```

---

DO NOT resume the daemon yet. Paste the full output back to the
v1-ship-1 advisor session. It will verify read-only, then we resume
the daemon together.

If either `update-ref` prints `cannot lock ref ... is at X but
expected Y`: the ref moved again — copy the X value and report it,
do NOT re-run with --force or drop the CAS arg.

Expected good output:
  AD-e OK
  ship-1 OK
  HOOK PRESERVED
  tree:  ?? .claude/hooks/allow-prp-deliverables.sh
