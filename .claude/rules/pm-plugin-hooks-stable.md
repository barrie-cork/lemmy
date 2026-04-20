# PM plugin hooks — stability guarantee

Seven Extism plugin-hook call sites in the private-message path are
load-bearing for V2 messaging (ADR-012). Do not delete, rename, or
remove either half of any pair without a new ADR.

## Hook inventory

- `local_private_message_before_create` — `api_crud/src/private_message/create.rs:75`
- `local_private_message_after_create` — `api_crud/src/private_message/create.rs:77`
- `local_private_message_before_update` — `api_crud/src/private_message/update.rs:57`
- `local_private_message_after_update` — `api_crud/src/private_message/update.rs:60`
- `federated_private_message_before_receive` — `apub/objects/src/objects/private_message.rs:170`
- `federated_private_message_after_receive` — `apub/objects/src/objects/private_message.rs:173`
- `plugin_hook_notification` (PM path) — `api_utils/src/notify.rs:305`

## Do not

- Delete any call site
- Rename any hook string
- Remove one half of a before/after pair
- Wrap in a feature flag that defaults off

## Acceptable

- Add new hooks alongside existing seven
- Change form type `T` (document in upstream-rebase notes)
- Reorder within handler if hook still fires on every success path
- Replace dispatch mechanism (V2 cares about names + call sites, not host)

If a refactor requires renaming/removing/gating a hook, write a
decision-queue entry citing this rule and V2/messaging.md §8.1.

## Detection

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
