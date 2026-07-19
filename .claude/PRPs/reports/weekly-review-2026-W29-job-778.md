# Weekly Review — 2026-W29 (daemon-side, job-778)

Run: 2026-07-19 02:31 UTC (Sunday), Junior task job-778, on homeserver/EliteDesk.
Skill: `.claude/skills/weekly-review/SKILL.md`. Branch:
`junior/review-read-and-follow-claude-skills-weekly-review-skill-md-778`.
Base repo HEAD: `e0f191065` — **frozen at 2026-07-05** (last W27/job-776 review merge;
no trunk commits merged in the intervening two weeks).

## Headline — PMD store-split ROOT CAUSE ISOLATED, and W27's diagnosis corrected

`project-memory-http.service` (MainPID **1546**, started 2026-07-10 11:22) is **still
orphaned** — 4th consecutive flag (W26 ×2, W27, W29):

```
/proc/1546/fd/22 -> /srv/brehon-fork/.project-memory/memory.db      (deleted)
/proc/1546/fd/23 -> /srv/brehon-fork/.project-memory/memory.db-wal  (deleted)
/proc/1546/fd/24 -> /srv/brehon-fork/.project-memory/memory.db-shm  (deleted)
```

There are genuinely **two live stores**:

| Store | Reached by | State at run start |
|---|---|---|
| on-disk (path-based) | **stdio** MCP — Junior daemon tasks; `memory-backfill` | 1012 rows, newest `2026-06-26` |
| orphan inode | **HTTP** daemon — all **laptop advisor** sessions | 1013 rows, newest `2026-07-17` |

### Correction 1 — W27's prune did NOT hit the live store

W27 reported pruning the live on-disk store 1012 → 566 and cited an "arithmetic integrity
check" (`pre 1012 → pruned 446 → post 566; qa 550 → 104; exact`) as proof it was talking to
the durable store. **This is falsified.** Snapshot forensics over
`/srv/backups/pmd/brehon-fork/{weekly,daily}/` show the on-disk store was continuously:

```
20260705.db  total=1012  qa=550  newest=2026-06-26 10:46:25
20260712.db  total=1012  qa=550  newest=2026-06-26 10:46:25
20260719.db  total=1012  qa=550  newest=2026-06-26 10:46:25   (+ every daily in between)
```

It **never dropped to 566**. W27's prune landed on the orphan. Its arithmetic only proved
the orphan was *internally self-consistent* — not that it was the durable inode.

> **Lesson:** internal arithmetic consistency does not identify *which inode* you are
> talking to. Verify store identity against an **out-of-band artifact** (the hourly
> snapshots), not against the store's own before/after numbers.

### Correction 2 — the data-loss exposure was 1 row, not weeks

W27 inferred "laptop retro writes lost since ~2026-06-11" — implying weeks of loss. Reading
the orphan **directly** via `/proc/1546/fd/22` shows the true diff:

- orphan total **1013** vs on-disk **1012** → **exactly 1 row** exists only in the orphan:
  id `1134`, *"Task retro: README roadmap/ecosystem rewrite + v1-roadmap.json ground-truth
  sync"*, `2026-07-17 11:40:16`.

Exposure was small because the repo has been idle ~2 weeks. The apparent "3-week gap" W27
cited was a *store-identity artifact*, not lost writes.

### Falsified hypotheses (do not re-investigate)

- `restore-drill.sh` — restores only into `/srv/restore-test/` scratch (`cp "$ARTIFACT"
  "$SCRATCH/pmd.db"`), never the live path. Runs Sun 04:00; last Jul 12.
- `backup-pmd-snapshots.sh` — read-only (`sqlite3 "$src" ".backup '$dest'"`), writes only
  under `/srv/backups/pmd/`.

Neither replaces `memory.db`. **The replacement vector remains unidentified**; it occurred
between 2026-07-10 (daemon start) and 2026-07-17.

### Actions taken this run (data now safe)

1. **Salvaged** orphan row 1134 into the durable on-disk store (tagged
   `salvaged-from-orphan`, verified present).
2. **Preserved** the full orphan DB at
   `.project-memory/rescue/orphan-inode-20260719T023624Z.db` (+ `at-risk-row-1134.json`).
3. Pruned 550 expired rows from on-disk.

**The restart is now safe to perform** — nothing unique remains in the orphan. It is still
an outward-facing, coordinate-first action (a laptop advisor session may be an active HTTP
client), so it was **not** executed by this Junior task:

```bash
sudo systemctl restart project-memory-http.service
ls -la /proc/$(systemctl show -p MainPID --value project-memory-http.service)/fd | grep memory.db  # must NOT say (deleted)
```

## Steps executed (daemon-runnable)

### 1. Memory health + prune ✓
- Pre-prune 1012 → **pruned 550** (all expired `qa-result`, 30-day expiry; 0 superseded) → 462.
- Dry-run inspected first (all 550 confirmed `qa-result`, all genuinely past `expires_at`).
- Post-salvage on-disk total **463**. Remaining: pattern 246, summary 193, issue-note 12,
  decision 8, deploy-note 2, bug 1, qa-result 1 (the salvage).

### 1b. PMD embedding backfill — LAPTOP-ONLY per skill; read-only diagnostic here ✓
On-disk store: **0 unembedded rows**. `memory-backfill.timer` is live (5-min cadence, last
fired 02:33). Embeddings current for the durable store.

### 1c. Lesson frontmatter sweep ✓ — 2 files flagged, NOT auto-fixed
`scripts/brehon/lesson-frontmatter-lint.sh` over **232** lesson files (exit 2):
- **BROKEN:** `feedback_daemon_sync_before_dispatch.md` — `name:` empty/absent → silently
  skipped by PMD sync, invisible to `memory_search_hybrid`.
- **NESTED:** `feedback_bm_pr_daemon_finalize_merges_phase_into_trunk.md` — `type:` nested
  under `metadata:`; imports but diverges from corpus convention.
- **Unchanged for 3 consecutive weeks** (W26 → W27 → W29). Frontmatter content is a human
  authoring decision — surfaced, not auto-fixed, per skill.

### 2. Promote candidates — none actionable
`memory_review` returned only generic infra tags (brehon-fork 523, success 384, lesson 311,
advisor 248, role-signal 194…). No file/pattern clusters warranting `docs/memory/`
promotion. Same as W26/W27.

### 2b. Lesson clustering — SKIPPED (skill: TH-only; skip on server repos).

### 2c. Retro-harvest sweep ✓
**`mtime` filtering is broken on the daemon** — the Junior worktree is created fresh, so all
304 report files report today's mtime and the skill's `-mtime -7` matches *everything* (306
false positives). Used `git log --since='7 days ago'` instead → **0 reports touched**. Last
report-touching commit is `d35d0323e`, the W27 review itself.
Written to gitignored `.claude/harvest/2026-W29.md`; carry-over items re-verified as still
un-promoted. Surfacing only, no auto-promotion.

### 2d. MEMORY.md ACTIVE-line drift — LAPTOP-ONLY (skill path `C:/Users/barri/…` absent on daemon).

### 2e. Role-signal drain + role-health — LAPTOP-ONLY (needs laptop DB + `ssh homeserver` scp *from* homeserver).

### 3. Eval metrics aggregation ✓ (with frozen-window caveat)
**0 evals in the trailing 7 days** and 0 in the trailing 30. Newest `qa-result` in the
durable store predates 2026-06-26; newest anywhere (orphan) is 2026-07-17. This is a
genuine activity gap (repo idle), *not* a retro-bypass — `.claude/governance-log/retro-bypass.jsonl`
was not implicated.

Stats over the 100-row sample taken **before** the prune (durable record, since the rows are
now deleted):
- n=100 scored; **avg score 0.73** (median 0.72) — upper edge of the calibrated 0.60–0.75 band.
- Outcomes: 79 success / 10 partial / 1 failure / 10 untagged → success rate 79–89%.
- Top root causes: `environment-issue` (12), `requirements-misread` (3),
  `type_alias_caller_mismatch` (3), `missing-verification` (3). Only 41/100 rows carried a
  `ROOT_CAUSE` line at all.
- **Calibration flag: 10/100 rows scored >0.85** (two at 1.00), against the rubric's "above
  0.85 should be rare". 6 of the 10 are low-complexity `bm-*` mechanical verbs — inflation
  concentrates there.
- **Duplicate-retro storm:** across the full 550-row prune set, 9 title clusters covered 32
  rows (5.8%) — worst `Task retro: v1-SL-e idle polling` ×6, `idle continues` ×5,
  `m2-late-1-task-5 spawn-site` ×5. Consistent with the known retro-check bug where the Stop
  hook's `memory_search` could not see just-written entries, so retros were re-written 2–6×.
  This inflates every historical task-count denominator.

### 4. Git hygiene ✓ — clean
`git worktree prune`: nothing to prune. 2 worktrees (canonical `governance-v0` + this job).
Only `junior/*` branch present is this task's own. No stale review branches >7d other than
this task's own branch. Nothing to delete.

### 4b. Disk headroom ✓ — improved
`/` at **61%** (134G/232G, 88G free) — **down from 75% at W27**, back below the investigate
threshold. Journal 3.1G. No separate `/var/log` or `/tmp` mounts. No action needed.
(Note: the skill's `ssh homeserver "df …"` fails here — this task *runs on* homeserver, and
the daemon has no self-SSH key. Ran `df` locally.)

### 5. Summary ✓
Summary memory written to the durable on-disk store (id **1135**). Global `METRICS.md` does
not exist (only `.claude/skills/brehon-conformance-audit/METRICS.md`, a different artifact);
W26/W27 set the report-only precedent — followed, not created unilaterally.

## Findings for the human (priority order)

1. **Restart `project-memory-http.service` — now SAFE.** The one at-risk row has been
   salvaged into the durable store, so a restart no longer loses data. Command above.
   Coordinate first if a laptop advisor session is live.
2. **The `memory.db` replacement vector is still unidentified.** `restore-drill.sh` and
   `backup-pmd-snapshots.sh` are both cleared. Without finding the actual writer, the daemon
   will re-orphan after any restart (as it did between W26 and W27, and again after the
   2026-07-10 start). This is the durable fix, and it is still open.
3. **Weekly-review's own store-identity blindness.** W27 mis-diagnosed because it trusted
   internal arithmetic. Recommend the skill gain a Step-0 store-identity assertion: compare
   the MCP's reported row count against the newest `/srv/backups/pmd/<repo>/hourly/*.db`
   snapshot, and against `/proc/<MainPID>/fd/*` for `(deleted)` markers, before any prune.
4. **Step 2c's `-mtime -7` filter is unusable in a Junior worktree** (all files look new;
   306/304 false-positive rate). The skill should specify `git log --since` instead. W26's
   job-773 already learned this; it has not been folded back into `SKILL.md`.
5. **Step 4b hardcodes `ssh homeserver`** but the daemon *is* homeserver — fails with
   `Permission denied (publickey)`. Skill should branch on `hostname`.
6. **Fix the 2 lesson frontmatter files** — unchanged for 3 weeks (see 1c).
7. **Retro score calibration drifting high** — 10% of rows >0.85, concentrated in `bm-*`
   mechanical verbs; plus a 5.8% duplicate-retro rate inflating task counts.
8. **Laptop-only steps 1b/2d/2e + TH-only 2b not run** (daemon context) — run laptop-side to
   complete the week, especially 2e's role-health strip-candidate report.
9. **Weekly-review keeps dispatching against a frozen daemon worktree** (HEAD 2026-07-05,
   now 2 weeks stale) — metrics and harvest have no fresh material. Consider running
   weekly-review laptop-side, or syncing the daemon worktree to trunk before dispatch.
   *(Repeated from W27 finding 5 — unactioned.)*
