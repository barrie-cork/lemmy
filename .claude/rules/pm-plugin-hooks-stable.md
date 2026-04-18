# PM plugin hooks — stability guarantee

Seven Extism plugin-hook call sites in the private-message path are
**load-bearing for the V2 messaging bridge** (see
`docs/brehon-law-inspired-network/V2/messaging.md` §4.4 and §8.1). Do
not delete them, rename them, or remove either the `_before_*` or
`_after_*` half of any pair without writing a new ADR that supersedes
ADR-012.

This rule auto-loads in `-p` mode. Every ralph loop touching PM code
reads it at first iteration.

## The hook names and their call sites

As of Lemmy 1.0-beta fork tip (Phase 5 era), the seven PM-path hooks
and their locations are:

### Local PM create

- `local_private_message_before_create` — `crates/api/api_crud/src/private_message/create.rs:75`
- `local_private_message_after_create`  — `crates/api/api_crud/src/private_message/create.rs:77`

### Local PM update

- `local_private_message_before_update` — `crates/api/api_crud/src/private_message/update.rs:57`
- `local_private_message_after_update`  — `crates/api/api_crud/src/private_message/update.rs:60`

### Federated PM receive

- `federated_private_message_before_receive` — `crates/apub/objects/src/objects/private_message.rs:170`
- `federated_private_message_after_receive`  — `crates/apub/objects/src/objects/private_message.rs:173`

### Notification fan-out (PM-adjacent)

- `plugin_hook_notification` (PM path) — invoked from
  `crates/api/api_utils/src/notify.rs:305` inside
  `notify_private_message_internal`. The hook **name** passed in is
  internal to `plugins.rs`; what matters is the call site being
  reachable from PM insert + update paths.

## The rule

### Do not

- **Delete** any of the seven call sites.
- **Rename** any hook string (e.g. `local_private_message_before_create`
  → `pm_before_create`). Plugins are registered by hook name; a rename
  is a breaking change even if the function signature is identical.
- **Remove one half of a pair** (e.g. delete `_after_create` while
  keeping `_before_create`). The V2 bridge relies on both halves so it
  can mutate the insert form (`_before_*`) and observe the committed
  row (`_after_*`).
- **Wrap** a hook call in a feature flag that defaults off. The hook
  must fire on every request path that reaches it.

### Acceptable changes

- **Add** new hooks alongside the existing seven. Phases adding
  governance-related PM behaviour should introduce new names, not
  repurpose existing ones.
- **Change the form type** `T` passed through
  `plugin_hook_before::<T>` — this breaks plugin compatibility but not
  the hook contract. Acceptable if the form type changes for unrelated
  Lemmy reasons; document in the upstream-rebase notes.
- **Reorder** a `_before_*` or `_after_*` call within its handler, so
  long as the hook still fires on every success path through the
  handler.
- **Replace** the hook dispatch mechanism (e.g. if upstream Lemmy
  moves from Extism to a different plugin host). The V2 bridge cares
  about hook *names* and *call sites*, not the host implementation.

### When in doubt — surface

If a refactor plan appears to require renaming, removing, or gating
any of these hooks, **stop and write a decision-queue entry** citing
this rule and V2/messaging.md §8.1. Do not silently edit the hook name
and hope the V2 bridge adapts later — V2 hasn't been written yet, so
there is nobody to adapt.

## Why this matters

V2 messaging (post-v0) will ship a Rust appservice bridge that mirrors
Brehon PMs into Matrix rooms. The bridge uses
`federated_private_message_after_receive` to observe inbound PMs and
`local_private_message_before_create` to validate outbound PMs before
they hit Postgres. If either hook disappears, V2 must either (a) patch
core PM code directly (contradicts ADR-012's "plugins for governance
hooks" rationale), or (b) poll the `private_message` table (adds
latency and load for every user). Both are worse than the status quo.

Keeping the hooks stable costs nothing today; losing them costs a V2
rewrite later.

## Detection

A simple pre-merge check: each of the six hook-name string literals must appear at least once under `crates/`. Non-recursive `grep -c crates/` (without `-r`) either errors with `Is a directory` or silently returns 0 on modern GNU grep, so use a per-hook loop that fails closed:

```bash
for h in \
  local_private_message_before_create \
  local_private_message_after_create \
  local_private_message_before_update \
  local_private_message_after_update \
  federated_private_message_before_receive \
  federated_private_message_after_receive; do
  rg -q "\"$h\"" crates/ || { echo "missing hook literal: $h"; exit 1; }
done
```

Each hook appears exactly once as a string literal at its call site (see the inventory above), so the check is "every hook present", not a count threshold. A count-based threshold is misleading because the six literals live in only three files (`private_message/create.rs`, `private_message/update.rs`, `apub/objects/src/objects/private_message.rs`) — aggregate counts don't distinguish "all six present" from "three present twice". Consider wiring this loop into the existing lint workflow.

## References

- V2/messaging.md §4.4 — full hook inventory
- V2/messaging.md §8.1 — "v0 hooks we must preserve"
- ADR-012 — Extism plugin host
- `crates/api/api_utils/src/plugins.rs:40-127` — hook dispatch implementations (`plugin_hook_before`, `plugin_hook_after`, `plugin_hook_notification`)
