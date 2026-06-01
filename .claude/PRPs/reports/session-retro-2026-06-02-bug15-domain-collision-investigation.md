# Session retro — 2026-06-02 — bug15-domain-collision-investigation

**Harness:** claude-code
**Session window:** 2026-06-01 ~22:30 → 2026-06-02 ~00:15 (~105 min)
**Branch at start:** `5a5deb30c` (`governance-v0`)
**Branch at end:** `4185bbc61` (`governance-v0`)
**Files touched:** 4 (docker-smoke-test-plan.md, lemmy.hjson [reverted], docker-compose-vanilla.yml [reverted], new lesson + new handover)
**Commits:** 3 explicit (Phase 9 results, BUG-15 root-cause correction, federation lesson) — all direct on `governance-v0` (meta-work per phase-branch.md)

## TL;DR

Two threads: Phase 9 (TOTP MFA) closed clean — all 6 checks passed via API, no friction worth a retro on its own. The load-bearing thread was BUG-15: the prior session's handover prescribed a concrete fix (`hostname: host.docker.internal:8536` + clear the Brehon DB volume) for bidirectional federation. Treating that prescription as a **hypothesis rather than a contract** — and testing it against the actual Lemmy source before committing — revealed the fix is architecturally wrong: Lemmy keys federated instances by **port-stripped domain** (`get_hostname_without_port`, `settings/mod.rs:71-81`), so two instances on one host via `host.docker.internal:<port>` collapse to the same domain and each treats the other as itself. The change was attempted, empirically falsified (vanilla `resolve_object` failed silently because it resolved the remote domain to its own local instance), and reverted cleanly. The single most-load-bearing carry-forward: **a handover's named fix is a hypothesis; verify its premise against the actual code/system before executing, especially when the premise names a specific mechanism as the defect.**

---

## What surprised us

- **The handover's BUG-15 fix was confidently wrong, and wrong in a non-obvious way.** It correctly identified that `localhost` is unreachable cross-container, but the proposed remedy (`host.docker.internal:8536`) introduced a worse, silent failure — a domain collision — because Lemmy strips the port when keying instances. The original-author's reasoning stopped at "make it reachable" and never checked how Lemmy *identifies* a remote.
- **Two false leads the handover baked in as fact, both falsifiable in <5 min each.** (1) "Clear the DB volume" — unnecessary: `Claims::validate` (`claims.rs:26-35`) never checks JWT `iss`, so a hostname change doesn't invalidate tokens. (2) The handover never noticed `tls_enabled` defaults to `true` (`structs.rs:39`), which is a *real* second config bug (dev ap_ids bake as `https://` behind a plain-HTTP proxy) — so the handover was simultaneously over-cautious (DB wipe) and under-complete (missed tls).
- **The failure was completely silent.** `resolve_object_failed` with **zero fetch logged** even at `lemmy_apub=debug` / `activitypub_federation=debug`. The instance short-circuits before any HTTP because it resolves the remote domain to itself. A naive read would blame connectivity; the `instance` table (one row = own domain) was the tell.
- **The connectivity actually *worked* after the change** — in-container `curl` fetched the Brehon actor fine. That's a trap: the reachability half of the fix succeeded, which would have falsely "confirmed" the fix to anyone not testing the actual `resolve_object` path end-to-end.

## What to change

| # | Change | Expected effect | Cost | Recurrence |
|---|---|---|---|---|
| 1 | When a handover/bootstrap brief prescribes a fix that **names a specific mechanism as the defect** (here: "hostname is the blocker"), run the existing falsifiable-hypothesis pass (`feedback_falsifiable_hypothesis_before_structural_fix.md`) against *handover-authored* fixes too — not just DQ-authored structural fixes. The lesson currently scopes to DQ entries and structural-fix DQs; extend its trigger list to include `handover RESUME "next action"` lines. | Catches wrong-premise fixes before the destructive step (DB wipe was on the table) | minor (one-line scope extension to an existing lesson) | 1× this session + the lesson already has 2 prior incidents (DQ #338, PR#132) → meets ≥2 |
| 2 | The new `feedback_lemmy_federation_domain_collision_one_host.md` lesson must surface when a future session resumes Phase 8 federation work. It is PMD-searchable (#765, top hit on "two lemmy instances federation testing") AND cited in `docker-smoke-test-plan.md` §Phase 8 BUG-15. Verify the smoke-test plan is the *first* doc a federation-resume session reads (it is — it's the canonical phase ledger). | The fact fires at resume time without relying on the resumer searching the right terms | done this session | net-new (no prior federation-topology lesson existed) |
| 3 | Capture the `host.docker.internal:port` domain-collision as a hard constraint in any *future* "spin up a second instance for federation" brief — point at the upstream `docker/federation/` topology (distinct container hostnames) as the only working pattern. Add a one-line note to the smoke-test plan's "Deferred" section so the next attempt doesn't re-derive it. | Prevents a third session from re-attempting same-host-different-port | minor (one-line note) | 1× (this is the first federation-resume attempt) |

## What to carry forward

- **Handover-premise-as-hypothesis.** The session's core win: read the handover's prescribed fix, but verify its *premise* against the actual code/system before executing. Here that meant reading `get_hostname_without_port`, `Claims::validate`, and the `tls_enabled` default — ~10 min of reading that saved committing a wrong change (and a needless DB wipe). This is the second strong data point for `feedback_handover_assumptions_need_empirical_verification` + `feedback_falsifiable_hypothesis_before_structural_fix`.
- **Dry-run-then-commit for destructive DB mutations.** The ap_id rewrite was run inside a `BEGIN; … ROLLBACK;` transaction first to show exactly which rows would change (and prove the `local=true` + `LIKE` filters excluded the federated vanilla rows), THEN re-run with `COMMIT`. Used once, cleanly — the dry-run caught that person id=3's `changeme.invalid` ap_id should be left alone while only its inbox_url got fixed.
- **Clean revert discipline.** When the fix proved wrong, both halves (config files via `git checkout`, DB ap_ids via a reverse SQL transaction) were restored to the exact pre-change state and both stacks restarted to baseline — verified by re-reading the Brehon actor id (`https://localhost/u/lemmy`) and confirming distinct domains. No residue left for the next session to trip over.
- **Inspect-before-delete on the stray file.** The 502KB `:LOCALAPPDATATempdq-worker.json` was `head`-inspected (confirmed a redundant DQ snapshot) before `rm`. Trivial here, but the habit is what prevents deleting something load-bearing.

---

## Three-signal scoring

| Skill / Agent / Command | Saved (min) | Wasted (min) | Surprise | Notes |
|---|---:|---:|---|---|
| Manual source read (claims.rs, settings/mod.rs, structs.rs) before acting | 40 | 0 | high | The ~10-min read revealed the domain collision + ruled out the DB wipe — prevented a wrong commit and a needless destructive op. Highest-leverage action of the session. |
| AskUserQuestion (BUG-15 approach fork) | 5 | 0 | none | Clean 3-option fork; user picked "full surgical fix now" with full context that the premise was already overturned. |
| Phase 9 TOTP API testing (Python HMAC-SHA1 token gen) | 25 | 0 | medium | Surprise: plan said "WebAuthn passkey, needs 2nd device" but impl is TOTP — fully testable via API, no device needed. Saved the entire "find a WebAuthn client" detour. |
| Dry-run SQL transaction (BEGIN/ROLLBACK) | 10 | 5 | low | 5 min wasted on MSYS path-mangling (`/tmp/...` → Windows path, `-f` arg mangled) before switching to stdin redirect. The dry-run itself was net-positive. |
| Vanilla debug-log recreation | 0 | 15 | medium | Bumped `RUST_LOG` to broad debug + recreated vanilla to capture the resolve error — but the failure is *pre-fetch* so nothing logged regardless. Wasted; the `instance` table query (1 cmd) was what actually diagnosed it. |
| session-retro skill | 10 | 0 | low | This retro; confirmed auto-phase section correctly dormant (leftover RT-r3 JSON ≠ trigger). |

## Complexity scores (heavy tasks only)

Per `feedback_retro_task_complexity_score.md`. Format: `<files>/<commits>/<runtime-min>/<max-log-silence-min>`.

| Task | Files | Commits | Runtime (min) | Max log silence (min) |
|---|---:|---:|---:|---:|
| BUG-15 investigate + attempt + revert + document | 4 | 2 | ~70 | <2 (interactive, no background jobs) |
| Phase 9 TOTP MFA test + doc | 2 | 1 | ~30 | <2 |

Neither task flagged (all under 55min/40min/8-file thresholds).

## Decisions to revisit

- **Should the smoke-test plan's Phase 8 be re-run under the `docker/federation/` topology to actually close bidirectional federation?** It's deferred and not needed for v0 (ADR-014 outbound already verified). Worth a deliberate decision at pilot-prep time, not now — bidirectional federation only matters once Brehon governance signals need to round-trip with real remotes.

---

## Promotion candidates (recurrence ≥ 2 in this session, or ≥ 1 here + ≥ 1 in prior memory)

- [ ] **Change #1** (extend falsifiable-hypothesis trigger to handover-authored fixes): edit `.claude/lessons/feedback_falsifiable_hypothesis_before_structural_fix.md` to add `handover RESUME "next action"` lines to its trigger list. Meets threshold (1× here + 2 prior incidents in the lesson).
- [x] **Change #2** (federation domain-collision lesson): already promoted to `.claude/lessons/feedback_lemmy_federation_domain_collision_one_host.md` + PMD #765 this session.

---

_Generated by `.claude/skills/session-retro/SKILL.md`. Lessons consulted:
`feedback_retro_not_report.md`, `feedback_four_role_retro_signals.md`,
`feedback_retro_task_complexity_score.md`._
