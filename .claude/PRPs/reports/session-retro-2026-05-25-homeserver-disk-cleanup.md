# Session retro — 2026-05-25 — homeserver-disk-cleanup

**Harness:** claude-code
**Session window:** 2026-05-25 ~09:00 IST → ~09:20 IST (~20 min)
**Branch at start:** `629447b86` (`governance-v0`)
**Branch at end:** `629447b86` (`governance-v0`) — no commits
**Files touched:** 0 (operational SSH session only)
**Commits:** 0

## TL;DR

Short operational session: freed ~9.2 GB on the homeserver EliteDesk `/srv` filesystem via five
SSH cleanup steps (journal vacuum, pip/uv/trivy caches, Junior log trim). Disk moved from 53% →
49% used (106 GB → 115 GB free). No code changes; no Junior tasks; no auto-phase involvement.
The top carry-forward: pre-flight `du -sh` before each destructive `rm -rf` step worked cleanly
as a size-verification ritual and should be standardised for future cleanup sessions.

---

## What surprised us

**Advisor:**
- The journal archive held **2.5 GB** in old rotated segments — more than expected for a system
  that supposedly auto-rotates. The `3.0 GB` reading before vacuum showed the journal retention
  policy had accumulated ~months of archived segments despite the default 1 GB active limit.
  Cause: multiple boot-IDs worth of archived journals accumulating without a size cap (systemd
  default is space-based only with no time-cap unless configured).
- pip and uv caches together were **5.4 GB** — substantial for a server that isn't a primary
  development machine. Likely accumulated from one-off `pip install` / `uv add` runs during
  Junior task setup scripts over the past few months.
- The Junior log directory had **448 log files** (330 MB), accumulated silently. The current
  system has no automatic rotation policy for `.junior/logs/*.log`.

## What to change

| # | Change | Expected effect | Cost | Recurrence |
|---|---|---|---|---|
| 1 | Add a `scripts/brehon/cleanup-junior-logs.sh` helper that keeps the 50 most recent logs and is wired as a `post-task` hook or run weekly | Prevents the 448-log / 330 MB accumulation from recurring; current state is silent drift | minor | 1× this session; but log accumulation is continuous |
| 2 | Add `systemd` journal size cap to homeserver: `sudo journalctl --vacuum-size=500M` is reactive; add `SystemMaxUse=500M` + `SystemMaxFileSize=50M` to `/etc/systemd/journald.conf` so future accumulation is proactively bounded | Eliminates the need for a manual vacuum step; journal stays bounded without advisory intervention | minor | 1× this session; would recur next disk-pressure event |
| 3 | Add disk-headroom check to the weekly-review skill (or a cron alert): `df -h / | awk 'NR==2{gsub(/%/,"",$5); if($5>75) print "WARN: / at "$5"%"}'` — surface when use% crosses 75% rather than waiting for 100% failure | Gives early warning before the next OOM/crash; the last event (100% disk) crashed the daemon mid-run | minor | post-incident; disk pressure reoccurred once |

## What to carry forward

**Advisor:**
- **Pre-flight `du -sh` before each `rm -rf`** — running `du -sh <target>` before each deletion
  step gave a concrete bytes-freed figure per step and confirmed no accidental undershoot. Used
  cleanly across all 5 steps; worth making this the standard pattern for any cleanup session.
- **Sequential SSH cleanup with per-step size reporting** — the 5-step structure (vacuum → pip →
  uv → logs → trivy) worked well; each step was independent and could be stopped without losing
  prior work. Clean template for future disk-pressure triage.
- **Disk is still at 49% (115 GB free)** — this gives comfortable headroom for the next
  `cargo build` cycle (~30–40 GB peak for a full workspace build with target/). No immediate
  follow-up needed; monitor at next weekly-review.

---

## Three-signal scoring

| Skill / Agent / Command | Saved (min) | Wasted (min) | Surprise | Notes |
|---:|---:|---:|---|---|
| Direct SSH + sequential Bash | ~15 | 0 | low | Clean sequential execution; no retries needed |
| Pre-flight `du -sh` per step | ~5 | 0 | none | Confirmed sizes before deletion; used on every step |
| `df -h` bookend (before + after) | 2 | 0 | none | Made the net-freed figure concrete immediately |

## Complexity scores (heavy tasks only)

No impl-tasks, no Junior tasks, no multi-file edits. This was an operational SSH session only;
complexity scores are not applicable.

---

## Decisions to revisit

- The `SystemMaxUse` journald cap (change #2) requires a one-line edit to `/etc/systemd/journald.conf`
  on homeserver and a `sudo systemctl restart systemd-journald`. Low-risk, low-cost — could be
  applied in the next homeserver maintenance window.
- Evaluate adding a `cleanup-junior-logs.sh` script (change #1) to `scripts/brehon/` and wiring
  it to the weekly-review cron on homeserver. The 448-log accumulation is the highest-recurrence
  risk since Junior tasks run continuously.

---

## Promotion candidates (recurrence ≥ 2 in this session, or ≥ 1 here + ≥ 1 in prior memory)

- [ ] **Junior log cleanup script** (`scripts/brehon/cleanup-junior-logs.sh`, keep-50 policy):
  promote to a tracked script + wire into weekly cron. Command: `ls -t /srv/brehon-fork/.junior/logs/*.log | tail -n +51 | xargs rm -f`
- [ ] **journald size cap on homeserver**: promote to homeserver config runbook or a one-time
  ansible/script task. Edit `/etc/systemd/journald.conf`: `SystemMaxUse=500M`, `SystemMaxFileSize=50M`.

---

_Generated by `.claude/skills/session-retro/SKILL.md`. Lessons consulted:
`feedback_retro_not_report.md`, `feedback_four_role_retro_signals.md`,
`feedback_retro_task_complexity_score.md`._
