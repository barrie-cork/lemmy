# Session retro — 2026-06-12 — harness-audit-trims-dispatch

**Harness:** claude-code
**Session window:** 2026-06-12T13:18Z → 2026-06-12T13:55Z (~40 min)
**Branch at start:** `610581d2a` (`governance-v0`)
**Branch at end:** `a9ca028f7` (`governance-v0`; also cut `chore/harness-trims` at same SHA)
**Files touched:** 2 created (`harness-audit-2026-06-12.md` report, `harness-trims-impl-1.md` brief)
**Commits:** 1 (explicit; atomic burst on shared canonical checkout)

## TL;DR

Ran `/harness-audit` (7 high-confidence wins, ~6,260 tok; headline: 5-day organic rule growth
+8,442 chars fully consumed the prior audit's −8,683 dedup win), then authored a P2–P7 bundle
brief and dispatched Junior #664 on an isolated `chore/harness-trims` branch to avoid the live
m2-late-2 Mode-B session sharing this checkout. Most load-bearing finding: one pre-queue
`memory_search_hybrid` call surfaced TWO would-be dispatch-killers (daemon refspec doesn't fetch
`chore/*`; Junior sensitive-file gate blocks Write on `.claude/**`) — both neutralised in the
brief/setup before the worker ever started. Top change proposal: stop asking the audit's Phase-1
subagent to carry the class column at all (3rd consecutive audit it drifted labels while claiming
"no mismatches"); the parent already owns the deterministic table.

---

## What surprised us

- **The Phase-1 Explore subagent mislabeled 3 files AND reported "no class-mismatches detected"**
  (`pre-phase-harness-audit` → claimed ALWAYS; `pmd-search-strategy` + `phase-branch` → claimed
  SCOPED). 3rd consecutive audit with class-drift — but the first with zero damage, because Phase-0
  Step-6's parent-side grep is now the authority and counts were re-verified via `wc` (the
  subagent's *numbers* were exact; only its *labels* lied). The cross-check instruction inside the
  subagent prompt demonstrably does not work; detection moved parent-side is what works.
- **One pre-queue PMD search paid for the entire §2.3 discipline in a single call** — surfaced
  (a) the m2-late planning retro's sensitive-file gate finding (Junior Write blocked on ALL
  `.claude/**`; 5 rejections, /tmp+cp workaround) and (b) the daemon refspec gap. Without (a) the
  worker would have floundered on its very first edit of a 100%-`.claude/` task; without (b) the
  task would have died in <1s with `fatal: invalid reference`.
- **The audit's net delta was −241 chars despite a −8,683 applied dedup** — the always-load rules
  corpus regrew +8,442 chars in 5 days (~1,700/day), mostly mid-orchestration inline narrative
  added by live sessions. The advisory `rule-narrative-bloat-reminder.sh` hook is being out-paced.
- **`lemmy-hooks.ts:139–141` hard-loads three rules into Pi sessions** (`decision-queue.md`,
  `phase-branch.md`, `gh-pr-fork-target.md`) — previously "Pi-shared" was folklore from the skill's
  out-of-scope list; now it's evidence, and it correctly blocked a tempting trim (gh-pr-fork-target
  is 100% duplicated elsewhere but must stay).
- **Minor self-inflicted scare:** `head`-only read of `harness-deleted-rules.txt` showed just the
  comment header → briefly concluded the 2026-06-07 deletion skipped its ledger step; full read
  showed 4 entries present. ~2 min wasted; `pattern_verify_before_trusting_shell_output` strikes
  again in miniature.

## What to change

| # | Change | Expected effect | Cost | Recurrence |
|---|---|---|---|---|
| 1 | `.claude/skills/harness-audit/SKILL.md` Phase 1: remove the class column from the subagent's job entirely — subagent reports ONLY `path/lines/chars`; parent joins class from the Phase-0 Step-6 table mechanically. Delete the in-prompt "cross-check" instruction (proven ineffective 3×). | Eliminates the drift surface instead of detecting it; no more "no mismatch" false assurances to audit | minor | 3× (2026-05-09, 2026-05-31, 2026-06-12) |
| 2 | Author `.claude/lessons/feedback_junior_sensitive_file_gate_claude_paths.md` (knowledge currently lives only in PMD eval #872 + a MEMORY.md PROMOTE note) AND add a row to `advisor-orchestrator.md` §2.4 mandatory-injection table: brief file list includes `.claude/**` → inject the /tmp+cp workaround block + this lesson. | Every future `.claude/**`-targeting Junior brief gets the workaround mechanically instead of relying on the author remembering PMD search results | minor | 3× (m2-late planning #639, bm-task findings-yaml refusals, this session's pre-arm) |
| 3 | Extend `.claude/lessons/feedback_daemon_refspec_excludes_meta_phase_branches.md` with the `chore/*` variant (explicit `git fetch origin chore/X:chore/X` requirement), OR add `+refs/heads/chore/*` to the daemon refspec via the same `git config --add` recipe (and finally track it in the shim-patch manifest — the lesson's own TODO is still open). | First Junior task on any future `chore/*` branch doesn't die with `invalid reference`; refspec fix survives shim restores | minor | 1× this session + 1× prior (phase-brehon-* class, 2026-05-20) |

## What to carry forward

- **Deterministic-grep-as-authority over subagent judgment** (harness-audit Phase-0 Step-6
  pattern): when a subagent must classify, compute the classification parent-side mechanically
  and have the subagent only *count* against it. Generalises to any inventory-style delegation.
- **Pre-queue `memory_search_hybrid` before EVERY dispatch** (§2.3) — this session it converted
  two latent task-killers into two brief paragraphs. The 2-second search is the cheapest
  insurance in the orchestration stack.
- **Chore-branch isolation for meta-work during a live phase:** author brief on trunk → push →
  `git branch chore/<slug> governance-v0` → push → daemon-local ref via explicit refspec fetch →
  `create_task base_branch=chore/<slug>`. Zero checkout switches anywhere (laptop canonical stayed
  on `governance-v0`, daemon stayed on `phase-m2-late-2`), zero contention with the in-flight
  session, finalize-merge lands on the chore branch for later advisor-side merge.
- **Atomic-burst commit discipline on the shared canonical checkout** (status → stage-own-files →
  commit → isolation-verify → push, one chain) — used once, clean, no race with the concurrent
  Mode-B session that had committed 13 times in the prior hour.

---

## Three-signal scoring

| Skill / Agent / Command | Saved (min) | Wasted (min) | Surprise | Notes |
|---|---:|---:|---|---|
| `/harness-audit` (full pipeline) | 45 | 5 | medium | vs manual inventory+scoring; 5 min on drift re-verification |
| Phase-1 Explore subagent | 10 | 5 | medium | counts exact, labels drifted; parent `wc` re-verify required |
| Phase-0 Step-6 deterministic grep | 10 | 0 | low (good) | nullified the drift; correctly caught `multi-lane-worktree.md` the 2 prior audits missed |
| `memory_search_hybrid` (pre-queue §2.3) | 30+ | 0 | high (good) | surfaced sensitive-file gate + refspec gap — two averted task failures |
| `/precheck` | 10 | 0 | none | 5/5 PASS; STALE gov-v0 correctly identified as irrelevant to chore-base dispatch |
| Junior dispatch leg (branch + ssh fetch + create_task) | 15 | 0 | none | clean; checkout isolation verified at each step |
| Ledger `head`-only read | 0 | 2 | low | self-inflicted; full-read before concluding |

## Complexity scores (heavy tasks only)

| Task | Files | Commits | Runtime (min) | Max log silence (min) |
|---|---:|---:|---:|---:|
| /harness-audit end-to-end (advisor-side) | 1 | 0* | 20 | n/a |
| brief author + dispatch leg (advisor-side) | 2 | 1 | 20 | n/a |

*report committed later together with the brief in `a9ca028f7`. No Junior task ran to completion
during this session (#664 queued at close — its complexity score belongs to the next session's
review).

## Decisions to revisit

- **Growth-rate guard:** if the next harness audit shows rules-only ALWAYS > 125 KB, the audit
  report's watch item proposes hardening `rule-narrative-bloat-reminder.sh` from advisory to
  blocking-warn. Revisit at next audit, not before.
- **Telegram completion hook (daemon ID 4)** shows `BuildMessage: ModuleNotFound` — pings for
  #664 and future tasks may silently not fire. Daemon-side fix; cite by purpose not ID per
  `feedback_daemon_telegram_completion_hook.md`.
- **P1 (MEMORY.md `/memory-prune`) is still open** — laptop user-scope, couldn't be delegated to
  Junior; 228 lines > 195 truncation threshold means content is being silently dropped TODAY.

---

## Promotion candidates (recurrence ≥ 2 in this session, or ≥ 1 here + ≥ 1 in prior memory)

- [ ] Change #1: update `.claude/skills/harness-audit/SKILL.md` Phase 1 (subagent loses the class column)
- [ ] Change #2: promote to `.claude/lessons/feedback_junior_sensitive_file_gate_claude_paths.md` + `advisor-orchestrator.md` §2.4 table row
- [ ] Change #3: extend `feedback_daemon_refspec_excludes_meta_phase_branches.md` with `chore/*` variant (and/or daemon `git config --add` + shim-manifest tracking)

---

_Generated by `.claude/skills/session-retro/SKILL.md`. Lessons consulted:
`feedback_retro_not_report.md`, `feedback_four_role_retro_signals.md`,
`feedback_retro_task_complexity_score.md`. Auto-phase section omitted: auto-state JSONs on disk
belong to the concurrent m2-late-2 session; this session never invoked or mutated `/auto-phase`
state (Step 0.5 trigger conditions 1–3 all negative)._
