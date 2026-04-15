# Running v0 phases from your phone

How to drive the brehon-fork implementation phases from the Claude mobile app (or any browser) using Claude Code Remote Control. The Windows dev machine keeps running the work; mobile is just a window into the session.

Applies to all phases of [docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md](brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md).

---

## Prerequisites (one-time)

1. **Claude Code ≥ 2.1.51** on the Windows dev machine. Check with `claude --version`; upgrade if older.
2. **Claude mobile app** — iOS / Android — or any mobile browser pointed at https://claude.ai/code.
3. **claude.ai OAuth login.** Remote Control does not work with API keys. Run `claude` and `/login`, pick the claude.ai option. If `ANTHROPIC_API_KEY` is set in the shell environment, unset it first.
4. **Windows sleep disabled on AC power.** Settings → System → Power & battery → "Screen and sleep" → Sleep: **Never** while plugged in. The monitor can sleep; the OS cannot. An extended network outage of more than ~10 minutes also kills the Remote Control session, so keep the machine awake *and* online.
5. **Docker Desktop set to launch at login.** Every ralph iteration from Phase 0 onward needs a reachable Docker socket for testcontainers-rs.
6. **Run `claude` once in the brehon-fork directory** to accept the workspace trust dialog. Remote Control won't start otherwise.

---

## Session mode — interactive `--rc`

Three Remote Control modes exist (`claude remote-control` server mode, `claude --rc` interactive, and `/remote-control` inside an existing session). For v0 brehon work, use **interactive mode**:

```powershell
cd C:\Users\barri\Developer\brehon-fork
claude --rc "brehon-v0"
```

Why interactive and not server mode:

- The v0 phases are strictly sequential — Phase N depends on Phase N−1 — so there is no parallelism to exploit.
- One long-lived conversation across all phases keeps context (and ralph's learned-patterns scratchpad) coherent.
- You can still type from the Windows terminal *and* mobile simultaneously — Remote Control syncs both surfaces.

When the session starts it prints a session URL and you can press **space** to show a QR code.

---

## Connect from mobile

1. Open the Claude app (or browse to https://claude.ai/code).
2. Scan the QR code from the Windows terminal, or find the session in the list by its name (`brehon-v0`). A computer icon with a green status dot means it's connected.
3. Send a prompt. Slash commands work identically to the terminal.

If you don't have the Claude app installed yet, run `/mobile` inside Claude Code on the Windows machine — it prints a download QR.

---

## Driving the phases

All commands below are sent from the mobile app into the `brehon-v0` session. They run on the Windows host.

### Phase 0 — test harness

```
/prp-implement .claude/PRPs/plans/phase-0-test-harness.plan.md
```

One-shot. No ralph loop needed. When green, merge the feature branch to `governance-v0`:

```
Please fast-forward merge feature/phase-0-test-harness into governance-v0, then delete the feature branch.
```

Claude Code has Bash access on the host; it will do the git work inline.

### Phases 1, 2, 3, 6 — ralph-loopable

One plan file per phase under `.claude/PRPs/plans/`. For each:

```
/prp-ralph .claude/PRPs/plans/phase-1-schema.plan.md --max-iterations 20
```

Default `--max-iterations 20` is fine for Phases 1–3. Bump to 30 for Phase 6 (federation crosses more crates).

Between phases:
1. Wait for `<promise>COMPLETE</promise>`.
2. Merge the phase's feature branch to `governance-v0`.
3. Delete `.claude/prp-ralph.state.md` so the next phase starts clean.
4. Send the next `/prp-ralph` command.

### Phase 4 — first 5 endpoints (human in the loop)

Do **not** use ralph. Drive it with `/prp-implement` interactively from mobile:

```
/prp-implement .claude/PRPs/plans/phase-4-endpoints.plan.md
```

Respond to Claude's questions as they come up. The mobile surface is actually the easier place to do this — you can answer the OQ-006 threshold question and other judgment calls from anywhere without interrupting a loop.

### Phase 5 — reputation + sponsorship (ralph with mid-phase checkpoint)

Start the ralph run:

```
/prp-ralph .claude/PRPs/plans/phase-5-reputation.plan.md --max-iterations 30
```

After the two invariant tests (`sponsor_liability_propagates`, `ineligible_user_cannot_be_picked_for_jury`) are authored, cancel the loop, review the tests by hand, then resume:

```
/prp-ralph-cancel
```

Review, then restart `/prp-ralph` against the same plan — ralph reads its state file and picks up where it left off.

---

## Watch-outs

- **Ralph runs are long.** A 20-iteration Phase 1 is 30–90 minutes. Remote Control streams tool activity to mobile; you don't need to keep the app open. The session persists on the Windows host whether or not mobile is connected.
- **Network blip > 10 minutes kills the session.** Docs: "if your machine is awake but unable to reach the network for more than roughly 10 minutes, the session times out and the process exits." The ralph state file (`.claude/prp-ralph.state.md`) is on disk, so recovery is:
  1. Restart `claude --rc "brehon-v0"` on Windows
  2. Reconnect from mobile
  3. Re-run `/prp-ralph` against the same plan file
  Ralph resumes from the state file. The conversation history from before the disconnect is gone, but the phase's progress is preserved.
- **Windows sleep is the silent killer.** If the OS sleeps mid-ralph, the process dies, any running Docker container leaks, and the stop hook never fires. Verify the power plan before every long run.
- **Don't start an Ultraplan session while Remote Control is active.** Ultraplan disconnects Remote Control — only one can occupy the claude.ai/code interface at a time.
- **One remote session per interactive process.** If you need a second session (e.g. to run an unrelated task in another repo), start a separate `claude --rc` in a different terminal.

---

## Troubleshooting

| Error | Fix |
|---|---|
| `Remote Control requires a claude.ai subscription` | API key auth in use. `claude auth login` → claude.ai. Unset `ANTHROPIC_API_KEY`. |
| `Remote Control requires a full-scope login token` | You're using a `CLAUDE_CODE_OAUTH_TOKEN` or `claude setup-token` token. These are inference-only. `claude auth login` instead. |
| `Remote Control is not yet enabled for your account` | Unset `CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC`, `DISABLE_TELEMETRY`, `CLAUDE_CODE_USE_BEDROCK`, `CLAUDE_CODE_USE_VERTEX`, `CLAUDE_CODE_USE_FOUNDRY`. Then `/logout` → `/login`. |
| `Remote credentials fetch failed` | Re-run with `claude remote-control --verbose` to see the actual error. Common causes: not signed in, firewall blocking outbound :443 to the Anthropic API. |
| Session shows offline on mobile | Windows machine slept, crashed, or lost network > 10 min. Restart `claude --rc` on Windows. |

---

## Alternatives worth knowing about

- **Dispatch** (Claude Desktop + mobile) — wrong shape. Routes through Claude Desktop, not Claude Code CLI, so the Desktop session does not have access to brehon-fork's PRP commands, ralph skill, or cargo toolchain.
- **Claude Code on the web** — runs in Anthropic cloud, not your machine. No local filesystem, no brehon-fork, no Docker host for testcontainers. Not usable for this workflow.
- **Channels** (Telegram / Discord / iMessage) — overkill for v0. Would let you forward chat messages into a session, but you'd still need a Remote Control session running to receive them.

---

## Reference

- Claude Code Remote Control docs: https://code.claude.com/docs/en/remote-control
- [IMPLEMENTATION-PLAN-v0.md](brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md) — phase-by-phase breakdown
- `.claude/PRPs/plans/` — per-phase PRP plan files
- `.claude/skills/prp-ralph-loop/` — ralph loop skill source
- `.claude/commands/prp-core/` — PRP command definitions
