---
role: bm-task
verb: bm-cut
phase: m3-core-recording
pr_number: null
related_dq: null
created: 2026-06-19
---

# BM-cut brief — m3-core-recording (M3 town halls — Phase 5)

**Role:** `[role:bm-task]`
**Phase:** `m3-core-recording` (branch `phase-m3-core-recording`)
**Authored:** 2026-06-19
**Authored by:** advisor (canonical brehon-fork session, on `governance-v0`)
**Base:** `governance-v0` HEAD at task-execution time (`99ab500ef` or later)
**Lane mode:** Mode B (mobile remote-control) per `.claude/rules/multi-lane-worktree.md` §"Lane modes". No laptop-side phase worktree; advisor drives via Junior dispatch from canonical `brehon-fork`. All cargo (crates Windows + bridge Linux) runs laptop-side via validate-pending-laptop[-linux] DQs regardless of mode.

> **Context:** m3-core-recording is M3 Phase 5 — optional evidentiary-store
> recording for town-hall rooms, gated on `record_town_halls=true` (default false).
> LiveKit Egress → MP4 → MinIO (generic-S3 via `rust-s3`) → `content_sha256` →
> callback into the binary's `append_room_event` for the FIRST
> `room_recording_uploaded` chain entry (const + ROOM_KINDS already REGISTERED in
> M2/m3-core-entry-kinds — emit-only). Participant-floor fetch authz (ADR-015).
> `record_town_halls=false` clean-posture zero-side-effect invariant. LOW
> complexity (score 3, proceed-as-one), bridge-side logic on top of stage-mode's
> emit seam + emergency-mute's `federated` trivial-DTO precedent + m3-core-infra's
> `recording_config` column + `livekit_jwt`. **The governing plan
> `.claude/PRPs/plans/m3-core-recording.plan.md` ALREADY EXISTS** (on
> `governance-v0` at `99ab500ef`, plan-approved at gate 1) — it is the bm-cut
> justification. The normal plan-presence check APPLIES. The plan must be
> reachable on `governance-v0` at task-execution time.

---

## 1. Role + dispatch line

```
[role:bm-task] m3-core-recording bm-cut — see .claude/PRPs/briefs/m3-core-recording-bm-cut-1.md
```

You are the **bm-task** subagent (Haiku 4.5 — git/yq/gh ops, no heavy reasoning). Execute the branch-manager verb `bm-cut` per `.claude/commands/bm/bm-cut.md`, step by step.

---

## 2. Scope

**Produce:** phase branch `phase-m3-core-recording` cut from `governance-v0` HEAD, pushed to `origin`, with upstream tracking set.

- **Phase:** `m3-core-recording` (branch `phase-m3-core-recording`)
- **PR number:** none (bm-cut — PR doesn't exist yet)
- **Trunk SHA:** `99ab500ef` on `governance-v0` (or later — base is HEAD at execution time)

**Explicit boundaries:**
- Cut branch `phase-m3-core-recording` from `governance-v0` at its current HEAD
- Push with `git push -u origin phase-m3-core-recording`
- **Plan-presence check APPLIES (normal):** the governing plan `.claude/PRPs/plans/m3-core-recording.plan.md` IS present on `governance-v0` (commit `99ab500ef` or later). Verify it is reachable: `git cat-file -e governance-v0:.claude/PRPs/plans/m3-core-recording.plan.md`. If absent, STOP and file a DQ.
- Write a one-line runlog entry to `.claude/runlog/m3-core-recording-runlog.md` (create if absent): `bm: bm-cut complete — phase-m3-core-recording cut from governance-v0 @ <SHA>`
- Commit + push the runlog entry on the new branch (per bm-cut.md Phase 5)

**Do NOT:**
- Open a PR (that comes at phase close after all impl tasks)
- Modify any file under `crates/`, `migrations/`, `docs/`, `services/bridge/`, `.claude/PRPs/plans/`, `.claude/PRPs/prds/`
- Merge or rebase anything
- Author or edit the plan (it already exists)

---

## 3. Required reading

1. `.claude/commands/bm/bm-cut.md` — the canonical bm-cut procedure; follow it step by step (note **Phase 8** finalize hazard)
2. `.claude/rules/branch-manager.md` — file-ownership boundaries and pre-cut checklist
3. `.claude/rules/phase-branch.md` — branch naming convention and pre-cut verifications
4. `.claude/rules/gh-pr-fork-target.md` — `--repo barrie-cork/lemmy` on every `gh` call

---

## 4. Constraints

- **Branch name must be exactly** `phase-m3-core-recording` (no variant)
- **Base must be** `governance-v0` (never `main`)
- **Pre-cut checks** (per `branch-manager.md`):
  1. `git status --short` on `governance-v0` — must be clean
  2. `git fetch origin && git log governance-v0..origin/governance-v0 --oneline` — must be empty (governance-v0 in sync)
  3. **Plan-presence check APPLIES:** `git cat-file -e governance-v0:.claude/PRPs/plans/m3-core-recording.plan.md` must succeed
- **Push with upstream tracking:** `git push -u origin phase-m3-core-recording`
- **Runlog:** create `.claude/runlog/m3-core-recording-runlog.md` if absent; append the bm-cut entry; commit + push on the **new branch** `phase-m3-core-recording` (per bm-cut.md Phase 5, the runlog commit is the first commit on the new branch)
- **DQ mid-task discipline:** if a blocker arises, write a `kind: "blocker"` DQ entry (use `bash scripts/brehon/dq-v3-new-entry.sh` for the id), commit + push, and stop

---

## 5. Success signals

- `git ls-remote origin refs/heads/phase-m3-core-recording` returns non-empty (branch exists on origin)
- `gh api repos/barrie-cork/lemmy/branches/phase-m3-core-recording --jq '.name + " @ " + .commit.sha[0:9]'` confirms the branch
- Runlog entry appended + committed + pushed on `phase-m3-core-recording`
- Push to origin succeeded with upstream tracking set

---

## 6. Out of scope

- Editing impl code, plan files, design docs, `services/bridge/**`, or any file in the never-touch list
- Opening a PR (that comes at phase close)
- Merging or rebasing anything
- Authoring or editing the plan (it already exists at `99ab500ef`)

---

## 7. KNOWN harness limitation — bm-cut creates divergence (do NOT merge back)

**bm-cut creates a divergence point, NOT a feature branch.** `phase-m3-core-recording` must **NEVER** be merged back into `governance-v0` by the finalize agent. The daemon's generic post-job finalize may wrongly run `git merge --no-ff phase-m3-core-recording` INTO daemon-local `governance-v0`. Confirmed repeatedly (v1-AD-e #282, v1-ship-1, the bm-pr variant on m3-core-entry-kinds PR #200 auto-merge, and the long-name refspec class in stage-mode/emergency-mute) — so the advisor's post-task daemon-trunk verification is MANDATORY this phase.

- **You (the bm-task worker):** just cut + push the branch + push the runlog. Do NOT merge anything. If the runlog write is blocked by the CC `.claude/**` sensitive-file gate, stop and report.
- **The advisor (post-task):** will verify daemon-local trunk as a ROUTINE step (`ssh homeserver 'cd /srv/brehon-fork && git log governance-v0 --oneline -1'`) and recover via `git update-ref` + push if a spurious merge landed. For the finalize-merge of this long-named worker branch, apply `feedback_daemon_long_name_refspec_finalize.md` (`git fetch origin <worker>:refs/heads/_fin<id>`) proactively.

---

## HANDOVER

```yaml
HANDOVER:
  task: m3-core-recording-bm-cut
  filesCreated: [.claude/runlog/m3-core-recording-runlog.md]
  filesModified: []
  keyDecisions:
    - "phase-m3-core-recording cut from governance-v0 @ 99ab500ef or later"
    - "Plan ALREADY EXISTS (.claude/PRPs/plans/m3-core-recording.plan.md @ 99ab500ef) — normal plan-presence check applies; gate-1 approved 2026-06-19"
    - "LOW complexity (score 3, proceed-as-one); bridge-side logic composing stage-mode emit seam + emergency-mute federated DTO precedent + m3-core-infra recording_config column + livekit_jwt"
    - "7 tasks: Task0 barrier; Task1[P](crates Windows 5-field recording DTO) + Task2[P](bridge S3 config + record_town_halls flag helpers); Task3 serial(requires:2, the rust-s3 dep-add); Task4 serial(requires:1,3); Task5 serial(requires:3); Task6 serial-docker(requires:3,4,5); Task7 retro"
    - "FIRST emission of room_recording_uploaded (const REGISTERED M2/m3-core-entry-kinds — emit-only; do NOT re-register or bump count from 72)"
    - "ONE new dep = rust-s3 (generic-S3, Task3 only) + sha2 transitive->direct; Egress via existing reqwest+livekit_jwt (no LiveKit SDK); NO new const/migration/sidecar/emitter"
    - "Bridge compiles Linux-only (cargo-linux.sh --manifest-path); Tasks 2-6 write validate-pending-laptop-linux DQ; Task3 = the headline lock-resolution gate"
    - "One in-scope crates/** touch: Task1 RoomEventPayload 5 optional recording fields (trivial DTO mirroring federated); entry-kind registry UNTOUCHED; no migration (recording_config column already shipped)"
  notes: "bm-cut only — no PR yet. Mode B lane (no laptop worktree). Next: /auto-phase m3-core-recording resumes; bm-cut-done routes to impl-cohort-1 (Cohort A = Task1[P]+Task2[P], file-disjoint across crates Windows / bridge Linux runners)."
```

Brief complete. Dispatch as:

```
[role:bm-task] m3-core-recording bm-cut — see .claude/PRPs/briefs/m3-core-recording-bm-cut-1.md
```

Base branch: `governance-v0`
