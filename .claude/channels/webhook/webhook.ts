#!/usr/bin/env bun
/**
 * brehon-webhook channel — pushes external events (CodeRabbit, GitHub Actions,
 * cargo build status) into a running Claude Code session.
 *
 * Listens on localhost:8788. Anything POSTed with the right X-Sender header
 * arrives in Claude's context as a <channel source="brehon-webhook" ...> tag.
 *
 * One-way only: Claude reads events and acts; no reply tool wired (use
 * Telegram for two-way DM).
 *
 * See .claude/channels/setup-telegram.md for the wider setup.
 */

import { Server } from '@modelcontextprotocol/sdk/server/index.js';
import { StdioServerTransport } from '@modelcontextprotocol/sdk/server/stdio.js';

const PORT = 8788;

// Allow-listed sender tokens. Anything not in this set is silently dropped.
// Override via env: BREHON_WEBHOOK_SENDERS=tok1,tok2,...
const allowed = new Set(
  (process.env.BREHON_WEBHOOK_SENDERS ?? 'gh-actions,coderabbit,local-cargo')
    .split(',')
    .map(s => s.trim())
    .filter(Boolean),
);

const mcp = new Server(
  { name: 'brehon-webhook', version: '0.1.0' },
  {
    capabilities: { experimental: { 'claude/channel': {} } },
    instructions:
      'Events from this channel arrive as <channel source="brehon-webhook" kind="..." ...>. ' +
      'They are one-way: read the body, take whatever action the brief asks for, no reply expected. ' +
      'kind="ci-failure" should trigger a triage; kind="coderabbit-critical" should be addressed in the current PR; ' +
      'kind="cargo-status" is informational only.',
  },
);

await mcp.connect(new StdioServerTransport());

Bun.serve({
  port: PORT,
  hostname: '127.0.0.1', // localhost only
  async fetch(req) {
    if (req.method !== 'POST') {
      return new Response('only POST is accepted', { status: 405 });
    }

    const sender = req.headers.get('X-Sender') ?? '';
    if (!allowed.has(sender)) {
      return new Response('forbidden', { status: 403 });
    }

    const body = await req.text();
    const url = new URL(req.url);

    // Build meta from headers + URL. Each entry becomes an attribute on the
    // <channel> tag for routing context. Keys must be identifiers (letters,
    // digits, underscores). Hyphens are silently dropped by Claude Code.
    const meta: Record<string, string> = {
      sender,
      kind: req.headers.get('X-Kind') ?? 'generic',
      path: url.pathname,
    };
    const ref = req.headers.get('X-Ref');
    if (ref) meta.ref = ref;

    // CR #85: wrap MCP dispatch in try/catch — transport hiccups otherwise
    // turn into an unstructured failure path for webhook calls. Return 502
    // on dispatch failure so callers (gh-actions etc.) can retry sensibly.
    try {
      await mcp.notification({
        method: 'notifications/claude/channel',
        params: {
          content: body,
          meta,
        },
      });
      return new Response('ok');
    } catch (err) {
      console.error('[brehon-webhook] MCP dispatch failed:', { err, meta });
      return new Response('upstream dispatch failed', { status: 502 });
    }
  },
});

// Log to stderr so it shows in `claude --debug-file`.
console.error(
  `[brehon-webhook] listening on http://127.0.0.1:${PORT} (senders: ${[...allowed].join(', ')})`,
);
