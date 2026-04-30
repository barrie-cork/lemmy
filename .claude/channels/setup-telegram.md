# Telegram channel setup runbook

This runbook wires the Anthropic-supported Telegram channel plugin into the
Brehon project so you can DM your Claude Code session from your phone, get
notified when long ralph loops finish, and approve permission prompts
remotely.

Pairs with the [webhook channel](webhook/README.md) for CI/CodeRabbit events.

## Prerequisites

- Claude Code ≥ 2.1.80 (you have 2.1.116)
- Bun ≥ 1.0 (you have 1.3.12)
- claude.ai login (Console / API key auth is **not** supported for channels)
- The bot token from BotFather (already saved to `~/.claude/brehon-telegram-token.txt`)

## One-time setup

Run these commands in your **interactive Claude Code session** (the slash
commands cannot be executed by an agent — they require user invocation).

```text
/plugin marketplace add anthropics/claude-plugins-official
/plugin install telegram@claude-plugins-official
/reload-plugins
```

If `/plugin install` says "not found in any marketplace," run
`/plugin marketplace update claude-plugins-official` first, then retry.

Configure the bot token (token is read from the stashed file):

```bash
/telegram:configure $(cat ~/.claude/brehon-telegram-token.txt)
```

This writes the token to `~/.claude/channels/telegram/.env` (path owned by
the plugin, outside the repo, gitignored at the user level).

After configuring, restart Claude Code with the channel flag:

```bash
claude --channels plugin:telegram@claude-plugins-official
```

The Telegram plugin starts polling for messages from your bot.

## First-time pairing

1. Open Telegram on your phone
2. Find your bot (the username you set in BotFather, ending `bot`)
3. Send any message — the bot replies with a 5-character pairing code
4. Back in Claude Code: `/telegram:access pair <code>`
5. Lock the allowlist: `/telegram:access policy allowlist`

After this, only your Telegram account can push messages into the session.

## Combine with the webhook channel

To get *both* phone DMs and CI/CodeRabbit push events into the same session:

```bash
claude --dangerously-load-development-channels --channels \
  plugin:telegram@claude-plugins-official \
  server:brehon-webhook
```

<!-- cr-issue-#85: development-channel flag is required for the
     `server:brehon-webhook` channel during the research preview;
     the alias on line 80 already shows the correct form. -->


(Multiple channels can be passed space-separated. The webhook channel is
defined in `.mcp.json` at the repo root — see `webhook/README.md`.)

Note: the custom webhook channel requires
`--dangerously-load-development-channels server:brehon-webhook` during the
research preview, since it isn't on the Anthropic-curated allowlist.

## Per-session startup pattern

Add this alias to your shell profile (`~/.bashrc` or `~/.zshrc`):

```bash
alias brehon-claude='claude --dangerously-load-development-channels \
  --channels plugin:telegram@claude-plugins-official server:brehon-webhook'
```

Then `brehon-claude` starts a session with both channels active.

## Use cases

- **Long ralph loop ends** → bot DMs you the iteration count + DQ state
- **CodeRabbit Critical finding lands** → webhook pushes the count + PR URL
  into the session; if you're at the terminal, Claude addresses it
  immediately; if not, you read the Telegram echo on your phone
- **Permission prompt during unattended work** → the Telegram plugin
  declares the [permission relay capability](https://code.claude.com/docs/en/channels-reference#relay-permission-prompts),
  so the prompt is forwarded to Telegram with a 5-letter ID; reply
  `yes <id>` or `no <id>` from your phone to approve/deny

## Security

The bot token in `~/.claude/brehon-telegram-token.txt` grants full control
of the bot. **Never commit it.** It is outside the repo, but if you set up
a new machine, copy the file rather than re-pasting through chat.

The Telegram plugin gates inbound messages on a sender allowlist (configured
via `/telegram:access`). Without `policy allowlist`, anyone who finds the
bot username can push messages.

## Rotating the token

If the token leaks:

1. Open BotFather in Telegram → `/mybots` → select your bot → API Token →
   Revoke current token → Generate new one
2. Overwrite the stash:

   ```bash
   echo "NEW_TOKEN_HERE" > ~/.claude/brehon-telegram-token.txt
   ```

3. Re-run `/telegram:configure $(cat ~/.claude/brehon-telegram-token.txt)`
4. Restart Claude Code with the channel flag

## Disabling the channel temporarily

Just start `claude` without the `--channels` flag. The bot stays configured
but doesn't poll, so messages queue up in Telegram until the next session
that includes the flag.

## References

- [Channels overview](https://code.claude.com/docs/en/channels)
- [Channels reference](https://code.claude.com/docs/en/channels-reference)
- [Permission relay](https://code.claude.com/docs/en/channels-reference#relay-permission-prompts)
- Token stash: `~/.claude/brehon-telegram-token.txt`
- Plugin source: <https://github.com/anthropics/claude-plugins-official/tree/main/external_plugins/telegram>
