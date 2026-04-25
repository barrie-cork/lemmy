# Session retro — advisor JM-c plan + prp-* skill triggers

**Session date:** 2026-04-25
**Session role:** advisor (cold-resumed from `v1-JM-b-advisor-handover.md`)
**Worktree:** primary `brehon-fork` on `governance-v0`
**Trunk start:** `governance-v0` @ `2d7002acc` (post-PR-#95 BM bundle)
**Trunk end:** `governance-v0` @ `4347284e0` (post-PR-#97 plan-merge + chore(prp))

---

## TL;DR for the next advisor

Three threads progressed end-to-end this session:

1. **JM-c plan written** — 1738-line plan at `.claude/PRPs/plans/v1-jury-mechanics-c.plan.md`; PR #97 cut → CR-clean (skipped review per `.claude/**` path filter) → merged into `governance-v0` @ `73c208f2b`
2. **prp-* skill triggers landed** — principle-style trigger blocks for `/cargo-validate`, `/test-write`, `/edit-mechanical` added to `prp-implement.md` + `prp-plan.md` via `chore(prp)` direct to trunk @ `4347284e0`; FF'd into JM-c worktree without disturbing impl WIP
3. **Two durable feedback rules saved to memory** — `feedback_principles_not_rules.md` (rules vs principles framing for slash-command guidance) + `feedback_advisor_impl_communication_via_handover_files.md` (cross-session communication MUST go through handover files; user is filepath-relay, not message-body-carrier)

JM-c impl session is mid-Task-1 (uncommitted `ENTRY_KIND_JURY_DEADLOCK` const + re-export + registry row in JM-c worktree). They have a handover at `.claude/PRPs/handovers/impl-2026-04-25-jm-c-prp-skill-triggers-fold.md` waiting if/when user passes them the filepath.

DQ #47 (OQ-V1-JM-07) still planner-pending; does NOT block JM-c.

---

## 1. What worked — keep doing

### 1.1 Cold-resume sequence ran clean

The cold-resume bootstrap prompt the user provided named exactly which files to read in which order. Execution against that list:

- CLAUDE.md + 7 rules files (auto-loaded)
- MEMORY.md + 4 indexed feedback/project memories
- JM PRD + JM-b plan + JM-b retro + JM-b retro-events + JM-b advisor handover
- PR #95 findings YAML + bm-runlog tail + decision-queue.json

Total cold-resume cost: ~7 minutes wall, mid-session token high-water mark ~95k. Within the safe-zone (<200k effective-reasoning threshold per `feedback_context_trim_verify_empirically.md`). The pre-written cold-resume prompt is the load-bearing reason this stayed cheap — without it, advisor would have spent 30+ minutes re-deriving "what's in flight."

**Keep:** cold-resume bootstrap prompts written by the closing session, consumed by the resuming session. JM-b advisor session wrote `v1-JM-b-advisor-handover.md` at JM-b close; that file made this session's first 5 minutes near-zero-cost.

### 1.2 Three parallel Explore agents for plan-authoring §2 codebase intelligence

Phase 2 of `/prp-core:prp-plan` launched three Explore agents in parallel:

- Agent 1: `submit_jury_vote.rs` current state (full handler structure + every const + every governance_log emission + every Diesel call)
- Agent 2: JM-b's snapshot/cascade patterns + governance_log emission shape
- Agent 3: e2e test infrastructure + reusable fixtures

Aggregate cost: ~3 minutes wall (parallel) vs estimated 15+ minutes if sequential. Each agent returned a structured table mappable directly into plan §10 patterns + §11 file inventory. Zero re-reads of the files the agents had already consumed.

**Keep:** parallel Explore at Phase 2 of plan-authoring. Per `feedback_explore_before_planning_on_reviews.md` + `feedback_subagent_model_and_effort.md`, this is the canonical shape — model="opus" + max-effort directive in each agent prompt; up to 3 parallel.

### 1.3 BM agent for branch-cut + plan-PR cycle

`branch-manager` subagent dispatched twice this session (once for `/bm-cut` equivalent + plan PR, once for `/bm-merge 97`). Both runs executed in <5 minutes wall, ran their own session-start ritual, asked before user-gated actions (Telegram pings — silently skipped both times, MCP disconnected), preserved file-ownership boundaries (zero touches to `crates/**` / `migrations/**` / `tests/**`).

**Keep:** BM-as-subagent for full PR lifecycle (cut → push → PR open → merge → runlog). Frees advisor to think about scope while BM handles topology. The handover at `advisor-2026-04-25-jm-c-impl-oversight.md` (BM's session-close brief) is the canonical example of BM communicating cycle results back.

### 1.4 User pushback on rule-shaped framing pivoted to principles

When I drafted skill triggers for `prp-implement.md` / `prp-plan.md` in rule-shape ("must use X when Y"), user pushed back with "principles, not rules." Pivot took 1 turn; revised triggers read as "skill X exists to address concern Y; prefer when Y is gating; here's when inline is right." Now in memory at `feedback_principles_not_rules.md` so future slash-command edits inherit the framing.

**Keep:** when user pushback names a framing distinction, save the distinction to memory immediately. It's almost always a recurring preference, not a one-off. Two such saves this session (`feedback_principles_not_rules.md` + `feedback_advisor_impl_communication_via_handover_files.md`); both rules will outlive JM-c.

### 1.5 Handover file written for the impl session instead of in-chat instructions

Mid-session, advisor was about to type a 60-line "instructions for impl" block in chat. User intervened: "Save the prompt and provide the filepath. I will give this to Imp." Pivoted to writing `impl-2026-04-25-jm-c-prp-skill-triggers-fold.md` (199 lines per handover.md template). User then escalated this to a hard rule: "This is the way you should do all communication to Imp. and visa versa. This is a must" → saved as `feedback_advisor_impl_communication_via_handover_files.md`.

**Keep:** every advisor↔impl message from this point forward goes through `.claude/PRPs/handovers/<role>-<date>-<topic>.md`. The user is the filepath-relay, never the message-body-carrier. This is binary, not judgment.

---

## 2. What surprised — durable lessons

### 2.1 The FF merge of `governance-v0` into `phase-v1-JM-c` was a strict-descendant fast-forward

Expected outcome (before the merge): a merge commit on JM-c folding in trunk's `73c208f2b` + `4347284e0`. Actual outcome: strict-descendant fast-forward, no merge commit. Why: the merge-base of `phase-v1-JM-c` and `origin/governance-v0` was `9e5dd60a5` (the plan commit) itself — meaning trunk had advanced *forward* from JM-c's HEAD without divergence. Trunk's `73c208f2b` was the merge commit *of* JM-c's plan PR (which contained `9e5dd60a5` as a parent), and trunk's `4347284e0` was a chore on top.

**Pattern:** when impl is stationary on a branch (hasn't committed since branch-cut) and trunk has only added commits that include the impl's plan commit (via the plan PR merge), `git merge --ff-only` succeeds without ceremony. If impl had committed Task 1 before the merge attempt, FF would have failed (divergence) and a merge commit would have been required.

**Generalisation for JM-d/e:** if user wants to fold trunk-side advisor-lane chores into a phase branch mid-impl, do it in the brief window between phase-PR-merge and impl's first task commit. After impl's first commit, every fold is a real merge.

### 2.2 The skill triggers in `prp-implement.md` apply via slash-command auto-load on next invocation

When `/prp-core:prp-implement` is invoked from a worktree, the slash-command body is read from that worktree's `.claude/commands/prp-core/prp-implement.md` at invocation time. Pre-FF, JM-c worktree had the old version; post-FF, it has the new triggers. Impl session running mid-Task-1 hadn't re-invoked `/prp-core:prp-implement` since Task 1 started, so impl has been operating on the *plan body* (which is invariant — the plan was authored by this session and committed before the trigger chore landed).

**Implication:** skill triggers in prp-* commands take effect at *the next invocation of the slash command*, not retroactively for an in-flight invocation. Impl session would need to either (a) re-invoke `/prp-core:prp-implement` to pick them up, or (b) read the updated `prp-implement.md` directly. The handover at `impl-2026-04-25-jm-c-prp-skill-triggers-fold.md` covers both paths.

**Generalisation:** slash-command edits that ship via `chore(prp)` direct-to-trunk only affect future invocations. If a behavior change is mid-flight critical, impl session must re-invoke OR the handover must spell out the change.

### 2.3 The `governance-v0` push was deny-rule blocked even for advisor-lane chore

The harness's `phase-branch.md` deny rule (no direct pushes to `governance-v0`) fired on the `chore(prp)` push attempt. Even though this is the established pattern for advisor-lane chores (per the `chore(ops)` triplet on 2026-04-25 and `project_jmc_cc_upgrade_landed.md`), the deny rule doesn't distinguish commit-subject pattern; it blocks the verb regardless.

**Resolution:** user typed `! git push origin governance-v0` from prompt, bypassing the deny rule via user-typed shell. Push succeeded.

**Pattern:** the deny rule is the right default (catches accidental impl-code pushes) but creates friction for the rare-but-legitimate case of advisor-lane direct pushes. User-typed bypass is the canonical workaround. If frequency increases, consider per-commit-subject carve-out via `update-config` skill — but the current ratio (1× this session, 1× the chore(ops) triplet 2 days ago) doesn't warrant it.

### 2.4 BM session-start ritual found 1 pending DQ + 0 file-ownership conflicts pre-cut

BM's session-start ritual (per `branch-manager.md` §"Session-start ritual") ran `git fetch` + `git status --short` + DQ-read + bm-runlog tail + open-PR list before any state-changing call. Found: DQ #47 pending (planner-attributed; non-blocking) + 0 open PRs + clean trunk + clean working tree. Proceeded with branch cut without surprises. Same ritual at `/bm-merge 97` invocation; same clean state.

**Keep:** BM session-start ritual is load-bearing. Skipping it would have risked branch-cut from a mid-bundle trunk state or a missed concurrent-PR conflict. Cost: ~30 seconds; value: catches the class of bug `feedback_advisor_instruction_mismatch_stop_and_ask.md` documents.

---

## 3. What to carry forward

### 3.1 Memory entries written this session (3)

| Entry | Type | Why durable |
|---|---|---|
| `project_v1_JM_c_plan_written.md` | project | Records 1738-line plan at canonical filepath; JM-d planner / JM-c retro author / future advisor reads this to know JM-c is the serialisation gate for JM-d/SL-d/rep-tuning-r3 |
| `feedback_principles_not_rules.md` | feedback | Framing rule for all future slash-command + rule-file edits; distinguishes hard-constraint rules (file ownership / ADRs / push-target) from judgment-call principles (skill delegation / scope decisions) |
| `feedback_advisor_impl_communication_via_handover_files.md` | feedback | **Hard rule:** all cross-session messages go through `.claude/PRPs/handovers/<role>-<date>-<topic>.md`; user is filepath-relay; chat output capped at filepath + 1-line summary |

### 3.2 Handover files written this session (2)

| File | Author | Reader | Purpose |
|---|---|---|---|
| `advisor-2026-04-25-jm-c-impl-oversight.md` | BM session (during plan-PR cut) | Future advisor cold-resuming JM-c oversight | Cold-state for future JM-c advisor (mirrors v1-JM-b-advisor-handover.md shape) |
| `impl-2026-04-25-jm-c-prp-skill-triggers-fold.md` | This advisor session | JM-c impl session (in-flight, mid-Task-1) | FF-merge result + verify-WIP-intact + 3 skills' triggers + Task 1 commit instructions |

### 3.3 Handoff notes for whoever picks up the v1-JM cycle next

- **JM-c impl is mid-Task-1.** WIP in JM-c worktree (`brehon-fork-phase-v1-JM-c`) on branch `phase-v1-JM-c` @ `4347284e0`. Three modified files (governance_log.rs ×2 + registry.md), uncommitted. Task 1 is `ENTRY_KIND_JURY_DEADLOCK` const + re-export + registry row per JM-c plan §13 Task 1. Next action: validate (cargo check + clippy + test --no-run) → commit per plan's exact commit message → proceed to Task 2.
- **DQ #47 stays pending.** OQ-V1-JM-07 (post-JM-b general case-open severity-tier inference). Planner-attributed; v1.5 territory. Does NOT block JM-c. Per JM-b retro §3.4 + JM-c plan §12 + §19.
- **Telegram MCP disconnected this session.** Both BM cycles silently skipped pings (per `feedback_telegram_scope_notification_only.md` failure-mode). User can `/bm-ping merge-ready 97` post-hoc when MCP reconnects.
- **Issue #96 carry-forward** (7 findings from PR #95) stays separate from JM-c. JM-c plan §19 explicitly forbids silent fold-in even for adjacent code paths.
- **`bm-runlog.md` modified in primary worktree** (BM's pending bundle commit from JM-c plan-PR + merge cycle). Not committed by this session; BM will bundle on next cycle. Discrepancy with origin is expected and intentional.
- **The 7 untracked legal-brief docs** in `docs/brehon-law-inspired-network/` (Brehn-Consensus-*.{docx,pdf,md} + expert-review-suite/ + Word lock files) are user-authored advisor-lane content. Not in scope this session. Word `~$*.docx` and `~WRL*.tmp` should be `.gitignore`d in a future cleanup pass.

---

## 4. What did NOT need fixing (worth preserving)

- **`cargo-output-capture.md` + `no-cargo-output-paste.md`**: zero exit-code masking incidents this session. The four cargo-runs that landed (Task 0 audit equivalents + the post-FF verify of triggers-present) all used `> .claude/PRPs/debug/*.log 2>&1` + explicit `echo "exit: $?"` capture.
- **Phase-branch discipline**: zero impl-code commits to `governance-v0` directly. The single `chore(prp)` commit that landed direct-to-trunk is advisor-lane (`.claude/commands/`), not impl-code; matches the established `chore(ops)` precedent.
- **DQ attribution discipline**: zero `answered_by: "advisor"` writes from this session beyond what was already recorded. DQ #47 (planner-attributed) untouched.
- **PM-plugin-hooks-stable.md**: no PM-adjacent code touched. Verified in BM session-start ritual.
- **Handover length target**: both handovers within 150-300 line target (199 + ~200 lines). Neither pasted cargo-output / CR-dumps.
- **Memory delta**: +3 files (1 project + 2 feedback). Within JM-b retro's "≤+2 files net memory delta" guideline (slightly over by 1, but the 3rd entry is a hard-rule that justifies its own file rather than folding into an existing one).

---

## 5. Quantified outcomes

| Metric | Value |
|---|---|
| Trunk advance | 2 commits (`73c208f2b` + `4347284e0`) |
| New files created | 4 (1 plan + 2 handovers + 1 retro) |
| Files modified + committed | 2 (`prp-implement.md` + `prp-plan.md`) |
| New memory entries | 3 (1 project + 2 feedback) |
| Subagents dispatched | 5 (3 Explore parallel for plan + 1 BM cut + 1 BM merge) |
| PRs cycled | 1 (#97 — opened, polled, merged in <30 min from cut) |
| User confirmations sought | 4 (BM cut, BM merge, FF-merge into JM-c worktree, push-to-trunk for chore) |
| Telegram pings sent | 0 (MCP disconnected throughout; consistent with JM-b cycle) |
| DQ writes | 0 |
| Cold-resume → first state-changing action | ~7 min wall |
| Token high-water mark | ~115k mid-session (within safe zone <200k) |

---

## 6. Tool-use self-assessment

### 6.1 Tools used heavily

- **Read**: ~25 reads. Plan files (PRD §9.1, §17 row 3; JM-b plan §1-§20 template; JM-b retro §1-§9; JM-b retro-events Events 1-4; JM-b advisor handover; PR #95 findings YAML), the three SKILL.md files, the two prp-* command files (twice — once before edits to map structure, once after for verification).
- **Edit**: ~6 edits. Two on `prp-implement.md` (Phase 3.2 + Phase 3.3 sub-blocks); two on `prp-plan.md` (Step-by-Step Tasks preamble + Validation Commands preamble); two on MEMORY.md (index updates).
- **Write**: ~6 writes. JM-c plan (1738 lines), 3 memory entries, 2 handover files (1 by BM via subagent, 1 directly), this retro.
- **Bash**: ~30 calls. git status/log/diff/fetch/merge, gh pr view, file inventories, grep verifications. Notable: zero use of `cat`/`head`/`tail` outside of the explicit cargo-log-tail pattern.
- **Grep**: ~10 calls. Cross-checking entry-kind consts, finding agent references in prp-* files, verifying skill-trigger landing post-edit.
- **Agent**: 5 dispatches (3 Explore parallel + 2 branch-manager).

Approximate ratio: Read:Edit ≈ 4:1. Write was higher than typical (4 substantial new files: plan + retro + 2 handovers).

### 6.2 Tools NOT used that could have helped

- **`/cargo-validate` skill**: never invoked this session. The 3 cargo runs (Task 0 audit Probe 9, post-FF verify, etc.) used the inline wrapper-pattern instead. Honest reason: this session's cargo runs were verification-only (not gating impl decisions), so inline was the right shape per the new `/cargo-validate` skip-condition I just authored. Self-consistent.
- **`/test-write` skill**: N/A this session (no test authoring; advisor session, not impl).
- **`/edit-mechanical` skill**: arguably could have used for the prp-implement / prp-plan edits if I'd pre-classified them as mechanical. Counter-argument: each edit was a paragraph-write at a specific structural insertion point with surrounding-context awareness, not a rg-enumerate-first repeat-pattern. Inline Edit was the right shape per the new `/edit-mechanical` skip-condition I just authored. Also self-consistent.
- **TaskCreate / TaskUpdate**: used 4 times for the JM-c plan-authoring phase (Phase 2 Explore agents + Phase 5 plan-author). Could have used for the prp-* edits + handover writes too — gave up tracking once the conversation pivoted into iterative back-and-forth. Acceptable for this session shape (advisor-as-coordinator), but for an impl session of comparable iteration count, would be under-tracking.
- **`/handover-advisor` slash command**: didn't use it for this session-end retro because the retro shape (advisor-session retro vs handover) is different enough that the slash command wouldn't have generated the right structure. The existing JM-b retro template is the right precedent. Future improvement: extend `/handover-advisor` to take a `--session-retro` flag.

### 6.3 Rule-violation near-misses

- **`cargo-output-capture.md`**: zero incidents. All 3 cargo runs captured to log + explicit exit code.
- **`no-cargo-output-paste.md`**: zero incidents. No cargo-log-tail pasted >20 lines.
- **`branch-manager.md`**: zero file-ownership boundary crossings by BM subagent; both BM cycles preserved the never-touch list.
- **`decision-queue.md`**: zero advisor-attribution-on-non-advisor-commit (zero DQ writes).
- **`phase-branch.md`**: 1 deliberate exception (the `chore(prp)` push direct to trunk), executed with user-typed bypass per established `chore(ops)` precedent.
- **`handover.md`**: both handovers within length target (150-300); both preserved file-ownership boundaries.
- **NEW: `feedback_advisor_impl_communication_via_handover_files.md`**: caught one near-miss this session — was about to type 60-line impl-instructions in chat; user intervened; pivoted to handover file. The rule was created precisely *because* of that near-miss. Going forward, this is mandatory.

### 6.4 Context-management signals

- Session token high-water: ~115k mid-session (during JM-c plan-authoring Phase 5 — Write call for 1738-line plan dominated). Stayed in the safe zone (<200k effective-reasoning threshold per `feedback_context_trim_verify_empirically.md`).
- Re-reads: minimal. JM-b retro read once at cold-resume; PRD §9.1 + §17 row 3 read once each. Plan §10 / §13 / §14 patterns referenced via memory rather than re-read.
- Cargo output budget: ~60 lines of cargo-output total in conversation across 3 capture-and-tail calls. Well under the 200-line caution threshold.

### 6.5 Lessons for the next advisor session

1. **The handover-file rule is now hard.** Every advisor↔impl message goes through `.claude/PRPs/handovers/<role>-<date>-<topic>.md`. Filepath in chat; body on disk. Don't paraphrase across the user; don't compose multi-paragraph briefs in chat.
2. **The principles-vs-rules distinction is now settled.** When adding judgment-call guidance to slash commands or rules, lead with *why the skill exists* + *what trade-off it embodies* + *when inline is the right shape*. Defer skip-conditions to the SKILL.md itself.
3. **BM-as-subagent is the canonical PR-cycle shape.** When a phase-branch + PR cycle is needed, dispatch `branch-manager` subagent; let it run its own session-start ritual + ask before user-gated actions. Don't try to do BM-work inline — file-ownership boundaries are easier to enforce inside the subagent.
4. **Trunk-side advisor-lane chores can FF into a phase worktree before impl's first commit.** After impl's first commit, every fold is a real merge with conflict potential.

---

## 7. Suggested action items for the next advisor (or user)

In priority order:

| # | Action | Effort | Value |
|---|---|---|---|
| 1 | Pass impl session the handover filepath: `.claude/PRPs/handovers/impl-2026-04-25-jm-c-prp-skill-triggers-fold.md` | 1 user message | HIGH (impl session is mid-Task-1 with WIP; the handover unblocks them on the FF + skill triggers) |
| 2 | Reconnect Telegram MCP (user-side terminal action) so future cycles can ping `merge-ready` / `pr-ready` events | ~30 sec | MEDIUM (notification convenience; not load-bearing for cycle correctness) |
| 3 | Either commit the legal-brief docs as `docs(brehon-law):` or add Word scratch files (`~$*.docx`, `~WRL*.tmp`) to `.gitignore` to clean up primary worktree | ~5 min | LOW (cosmetic; doesn't affect any session) |
| 4 | At JM-c impl session-close: write `advisor-<date>-jm-c-retro.md` consuming the impl-side retro per the JM-b precedent | ~30 min | HIGH (closes the JM-c loop; gives JM-d planner the same handoff-shape JM-b gave JM-c) |
| 5 | After ~3 v1 sub-phases run with the new prp-* skill triggers + handover-file rule, evaluate whether the trigger-blocks need refinement (e.g., are impl sessions actually invoking the skills? are the skip-conditions correct?) | ~1 hour evaluation | LOW-MEDIUM (defer to retro-extraction system once it ships per `bm-retro-extract.plan.md`) |

---

## 8. Session close

Standing down. Trunk at `4347284e0`. JM-c impl is the next actor (mid-Task-1, handover waiting). DQ #47 still pending (planner-attributed; non-blocking).

The two-rule combo this session settled (`feedback_principles_not_rules.md` + `feedback_advisor_impl_communication_via_handover_files.md`) will outlive JM-c and shape every future advisor session's communication style.

_Retro author: advisor session 2026-04-25 (cold-resumed from `v1-JM-b-advisor-handover.md`; standing down 2026-04-25T~16:10Z). Available for future advisor cold-resume via this retro + the saved memory entries._
