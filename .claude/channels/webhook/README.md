# brehon-webhook channel

A localhost MCP channel that pushes external events into a running Claude
Code session. Pairs with the [Telegram channel](../setup-telegram.md): use
Telegram for two-way DMs from your phone, use this webhook for unattended
event pushes from CI / CodeRabbit / cargo wrappers.

## Install

```bash
cd .claude/channels/webhook
bun install
```

## Wire into Claude Code

Add to the **repo root** `.mcp.json` (create if absent):

```json
{
  "mcpServers": {
    "brehon-webhook": {
      "command": "bun",
      "args": ["./.claude/channels/webhook/webhook.ts"]
    }
  }
}
```

Start a session with both channels:

```bash
claude --dangerously-load-development-channels \
  --channels plugin:telegram@claude-plugins-official server:brehon-webhook
```

The `--dangerously-load-development-channels` flag is required because this
custom channel isn't on the Anthropic-curated allowlist (research preview
constraint).

## Push an event

The server listens on `http://127.0.0.1:8788` (localhost only). POST events
with three required headers:

| Header     | Purpose                                                                 |
| :--------- | :---------------------------------------------------------------------- |
| `X-Sender` | Must match `BREHON_WEBHOOK_SENDERS` (default: `gh-actions,coderabbit,local-cargo`) |
| `X-Kind`   | Categorises the event for the agent's routing logic                     |
| `X-Ref`    | Optional. Anything that helps the agent find the source (PR #, commit SHA) |

Examples:

```bash
# CodeRabbit Critical finding (from a GH Actions step)
curl -X POST http://127.0.0.1:8788 \
  -H "X-Sender: coderabbit" \
  -H "X-Kind: coderabbit-critical" \
  -H "X-Ref: pr/47" \
  -d "1 critical finding on PR #47: actor-binding check missing in admin_emergency_remove"

# Cargo build done (from local script wrapping a long ralph build)
curl -X POST http://127.0.0.1:8788 \
  -H "X-Sender: local-cargo" \
  -H "X-Kind: cargo-status" \
  -H "X-Ref: phase-v1-AD-c task 4" \
  -d "cargo test --test e2e -p lemmy_server: PASS in 8m32s"

# CI failure summary (GH Actions)
curl -X POST http://127.0.0.1:8788 \
  -H "X-Sender: gh-actions" \
  -H "X-Kind: ci-failure" \
  -H "X-Ref: pr/47" \
  -d "$(gh run view --log-failed | head -200)"
```

## Customising sender tokens

Override the default allowlist via env:

```bash
BREHON_WEBHOOK_SENDERS=gh-actions,coderabbit,local-cargo,argo-cd \
  claude --dangerously-load-development-channels \
    --channels plugin:telegram@claude-plugins-official server:brehon-webhook
```

The MCP server reads the env at startup, so changes require restarting
Claude Code.

## Wiring into GH Actions

Sketch (place in `.github/workflows/ai-review-impact.yml` or similar):

```yaml
- name: Push CodeRabbit summary to brehon-webhook
  if: ${{ steps.coderabbit.outputs.critical_count > 0 }}
  run: |
    curl -X POST https://your-tunnel.example.com:8788 \
      -H "X-Sender: coderabbit" \
      -H "X-Kind: coderabbit-critical" \
      -H "X-Ref: pr/${{ github.event.pull_request.number }}" \
      -d "${{ steps.coderabbit.outputs.summary }}"
```

For GH Actions to reach a localhost endpoint on your dev machine, you need
either (a) a reverse tunnel (cloudflared, ngrok), (b) a self-hosted runner
on the same machine, or (c) a polling shim that pulls GH events into the
local webhook. Pick whichever fits your security posture.

## Test

In one terminal:

```bash
bun .claude/channels/webhook/webhook.ts
# Stays foreground; this is just for testing the listener; in production
# Claude Code spawns it as a subprocess.
```

In another:

```bash
curl -X POST http://127.0.0.1:8788 \
  -H "X-Sender: local-cargo" \
  -H "X-Kind: cargo-status" \
  -d "test event"
# Expect: 200 OK with body "ok"
```

A bad sender:

```bash
curl -i -X POST http://127.0.0.1:8788 \
  -H "X-Sender: random-attacker" \
  -d "should be dropped"
# Expect: 403 forbidden
```

## Troubleshooting

- **"port already in use"**: another process holds 8788. `lsof -i :8788`
  (or `netstat -ano | findstr 8788` on Windows) → kill the stale process.
- **No events arrive in Claude**: run `/mcp` in the session → look for
  `brehon-webhook` and its connection state. "Failed to connect" usually
  means a syntax error in `webhook.ts`; check `~/.claude/debug/<session>.txt`.
- **Bun not found**: `npm i -g bun` or follow <https://bun.sh/docs/installation>.

## See also

- [Channels reference](https://code.claude.com/docs/en/channels-reference)
- [Telegram setup](../setup-telegram.md)
- Companion plan: `~/.claude/plans/are-we-fully-utilising-majestic-planet.md` (gap 2)
