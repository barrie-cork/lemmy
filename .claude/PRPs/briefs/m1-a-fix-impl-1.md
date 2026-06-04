# m1-a fix-impl-1 — CR config fixes (cr-003, cr-006, cr-008)

## 1. Role + dispatch line

`[role:impl-task] m1-a fix-impl-1 — CR config fixes cr-003 cr-006 cr-008 — see .claude/PRPs/briefs/m1-a-fix-impl-1.md`

## 2. Scope

Fix three CR findings in config files (no Rust source changes):

### cr-003 — registration.yaml regex mismatch (major)

File: `services/bridge/registration.yaml`

The bridge generates puppet MXIDs with prefix `@_brehon_` (localpart starts with `_brehon_`), but the AS registration namespace regex is `@brehon_.*` (missing the leading underscore). This causes the homeserver to not recognise puppet MXIDs as owned by this AS, breaking appservice user-query routing.

Fix: change `namespaces.users[0].regex` from `"@brehon_.*"` to `"@_brehon_.*"`.

Current state:
```yaml
namespaces:
  users:
    - exclusive: false
      regex: "@brehon_.*"
```

After fix:
```yaml
namespaces:
  users:
    - exclusive: false
      regex: "@_brehon_.*"
```

### cr-008 — registration.yaml exclusive=false → exclusive=true (low)

Same file, same `namespaces.users[0]` block. The puppet user namespace should be `exclusive: true` to prevent other application services from claiming `@_brehon_.*` MXIDs, which would allow AS impersonation.

After both cr-003 and cr-008 fixes:
```yaml
namespaces:
  users:
    - exclusive: true
      regex: "@_brehon_.*"
```

Combine cr-003 and cr-008 into a single Edit on `services/bridge/registration.yaml`.

### cr-006 — docker-compose.yml ip_source comment (low)

File: `services/bridge/docker-compose.yml`

Find the comment referencing the Tuwunel ip_source setting (line ~34). Append the Tuwunel issue URL to the comment for operator reference.

Current state (approximate):
```yaml
      # ip_source: X-Forwarded-For   # Tuwunel: set if behind reverse proxy
```

After fix (append URL):
```yaml
      # ip_source: X-Forwarded-For   # Tuwunel: set if behind reverse proxy (https://github.com/girlbossceo/conduit/issues/465)
```

Read the file first to find the exact current comment text before Editing.

## 3. Required reading

- `services/bridge/registration.yaml` — read before editing (find exact current text)
- `services/bridge/docker-compose.yml` — read before editing (find exact ip_source comment)
- `.claude/PRPs/reviews/pr-179-findings.yaml` — for context on the findings (read-only)

## 4. Constraints

- **ONLY touch**: `services/bridge/registration.yaml` and `services/bridge/docker-compose.yml`
- **Do NOT touch**: any `src/*.rs` file, `Cargo.toml`, `Cargo.lock`, `tests/`, `crates/`, `migrations/`
- **No cargo validation required** — these are config-only changes (YAML); no `validate-pending-laptop` DQ needed
- **Commit**: one commit with subject `fix(bridge): CR cr-003/cr-006/cr-008 — namespace regex + exclusive + compose comment`
- **Push**: push `phase-m1-a` to origin after commit
- The `base_branch` for this task is `phase-m1-a` — worker forks from `phase-m1-a`, commits to it

## 5. Validation

After Editing both files:
1. Verify `registration.yaml` has `exclusive: true` and `regex: "@_brehon_.*"`
2. Verify `docker-compose.yml` ip_source comment contains the girlbossceo/conduit issue URL
3. Commit + push

No cargo check required (YAML-only changes).
