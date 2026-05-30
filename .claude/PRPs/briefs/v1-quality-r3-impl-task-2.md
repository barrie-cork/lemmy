---
phase: v1-quality-r3
role: impl-task
n: 2
authored: 2026-05-30
authored_by: advisor (canonical brehon-fork / governance-v0 session)
base_branch: phase-v1-quality-r3
task_number: 2
---

# [role:impl-task] v1-quality-r3 Task 2 — EnvVarGuard-wrap ~10 test-body INIT/GOV setter sites (option-a)

## 1. Role + dispatch line

```
[role:impl-task] v1-quality-r3 task-2 EnvVarGuard wrap test-body setter sites — see .claude/PRPs/briefs/v1-quality-r3-impl-task-2.md
```

## 2. Scope

### What to produce

**~10 Edits** to `crates/server/tests/e2e.rs` — replace each `unsafe { set_var(INIT); [set_var(GOV);] }` block in test function bodies with `let _g_init = EnvVarGuard::set(...); [let _g_gov = EnvVarGuard::set(...);]` bindings. See §2.2 for full site list with verbatim old_string/new_string anchors.

### Explicit boundaries

- **ONLY edits to `crates/server/tests/e2e.rs`** — the ~10 test-body sites listed below.
- **Requires Task 1 to be committed first** (same file; T2 edits on top of T1's tree).
- Do **NOT** touch the 2 bootstrap sites (`governance_fixtures::bootstrap` @~831, `admin_config_fixtures::bootstrap` @~6144) — those were T1's option-b treatment; they have SAFETY comments and stay raw.
- Do **NOT** touch the `boot_context()` site at ~line 17474/17475 — already EnvVarGuard-wrapped in v1-quality-r2; do NOT re-wrap.
- Bind with named `_g_init` / `_g_gov` (NOT bare `_`) — load-bearing distinction.

### Files

```yaml
creates: []
modifies:
  - crates/server/tests/e2e.rs
requires:
  - task: 1
    reason: "T1 lands 2 SAFETY comments on the same file; T2 wraps the remaining sites. Serial (same-file e2e.rs)."
```

### The ~10 edits with verbatim anchors

Pre-located from phase-v1-quality-r3 tip per R11. Work through them in this order (top-to-bottom by line number) to avoid anchor drift.

---

#### Edit A — `report_to_modlog_golden_path` (~line 2568/2569)

The enclosing fn head is at ~2517: `async fn report_to_modlog_golden_path`.
This site has a multiline SIGNING_SEED_HEX const + SAFETY comment above the unsafe block:

`old_string`:
```
  const SIGNING_SEED_HEX: &str = "0000000000000000000000000000000000000000000000000000000000000001";
  // SAFETY: tests run with --test-threads=1 so no concurrent env mutation;
  // these vars are read by SETTINGS (LazyLock) and the governance log
  // signer at first call.
  unsafe {
    std::env::set_var("LEMMY_INITIALIZE_WITH_DEFAULT_SETTINGS", "1");
    std::env::set_var("GOVERNANCE_LOG_SIGNING_KEY", SIGNING_SEED_HEX);
  }
```

`new_string`:
```
  const SIGNING_SEED_HEX: &str = "0000000000000000000000000000000000000000000000000000000000000001";
  let _g_init = EnvVarGuard::set("LEMMY_INITIALIZE_WITH_DEFAULT_SETTINGS", "1");
  let _g_gov = EnvVarGuard::set("GOVERNANCE_LOG_SIGNING_KEY", SIGNING_SEED_HEX);
```

---

#### Edit B — `sponsor_liability_with_founder_multiplier` (~line 3347/3348)

The enclosing fn head is at ~3306: `async fn sponsor_liability_with_founder_multiplier`.
This site has a local `SIGNING_SEED_HEX` const + bare SAFETY comment:

`old_string`:
```
  const SIGNING_SEED_HEX: &str = "0000000000000000000000000000000000000000000000000000000000000001";
  // SAFETY: tests run with --test-threads=1; no concurrent env mutation.
  unsafe {
    std::env::set_var("LEMMY_INITIALIZE_WITH_DEFAULT_SETTINGS", "1");
    std::env::set_var("GOVERNANCE_LOG_SIGNING_KEY", SIGNING_SEED_HEX);
  }

  let (_container, host_port) = governance_fixtures::start_postgres().await?;
  let db_url = governance_fixtures::db_url(host_port);
  let _g_db_url = EnvVarGuard::set("LEMMY_DATABASE_URL", &db_url);
```

`new_string`:
```
  const SIGNING_SEED_HEX: &str = "0000000000000000000000000000000000000000000000000000000000000001";
  let _g_init = EnvVarGuard::set("LEMMY_INITIALIZE_WITH_DEFAULT_SETTINGS", "1");
  let _g_gov = EnvVarGuard::set("GOVERNANCE_LOG_SIGNING_KEY", SIGNING_SEED_HEX);

  let (_container, host_port) = governance_fixtures::start_postgres().await?;
  let db_url = governance_fixtures::db_url(host_port);
  let _g_db_url = EnvVarGuard::set("LEMMY_DATABASE_URL", &db_url);
```

---

#### Edit C — `all_mvp_endpoints_return_non_404` (~line 4114/4116) [multi-line GOV]

The enclosing fn head is at ~4082: `async fn all_mvp_endpoints_return_non_404`.
This site has NO local SIGNING_SEED_HEX — the GOV value is an inline literal spanning multiple lines:

`old_string`:
```
  unsafe {
    std::env::set_var("LEMMY_INITIALIZE_WITH_DEFAULT_SETTINGS", "1");
    std::env::set_var(
      "GOVERNANCE_LOG_SIGNING_KEY",
      "0000000000000000000000000000000000000000000000000000000000000001",
    );
  }

  let (_container, host_port) = governance_fixtures::start_postgres().await?;
  let db_url = governance_fixtures::db_url(host_port);
  let _g_db_url = EnvVarGuard::set("LEMMY_DATABASE_URL", &db_url);
```

(Uniqueness anchor: `lemmy_routes::middleware::session::SessionMiddleware` import immediately above this fn + no local const. Only occurrence of `SessionMiddleware` import near an unsafe block.)

`new_string`:
```
  let _g_init = EnvVarGuard::set("LEMMY_INITIALIZE_WITH_DEFAULT_SETTINGS", "1");
  let _g_gov = EnvVarGuard::set("GOVERNANCE_LOG_SIGNING_KEY", "0000000000000000000000000000000000000000000000000000000000000001");

  let (_container, host_port) = governance_fixtures::start_postgres().await?;
  let db_url = governance_fixtures::db_url(host_port);
  let _g_db_url = EnvVarGuard::set("LEMMY_DATABASE_URL", &db_url);
```

---

#### Edit D — `ineligible_user_cannot_be_picked_for_jury` (~line 4459/4461) [multi-line GOV]

The enclosing fn head is at ~4427: `async fn ineligible_user_cannot_be_picked_for_jury`.
Site has `two_sponsors_lose_endorsement_strength_on_sanction` pattern with local const:

`old_string`:
```
    const SIGNING_SEED_HEX: &str =
      "0000000000000000000000000000000000000000000000000000000000000001";
    // SAFETY: tests run with --test-threads=1; no concurrent env mutation.
    unsafe {
      std::env::set_var("LEMMY_INITIALIZE_WITH_DEFAULT_SETTINGS", "1");
      std::env::set_var("GOVERNANCE_LOG_SIGNING_KEY", SIGNING_SEED_HEX);
    }

    let (_container, host_port) = governance_fixtures::start_postgres().await?;
    let db_url = governance_fixtures::db_url(host_port);
    let _g_db_url = EnvVarGuard::set("LEMMY_DATABASE_URL", &db_url);
    {
      let mut sync_conn = PgConnection::establish(&db_url)?;
      governance_fixtures::apply_all_schema(&mut sync_conn)?;
    }

    let pool: ActualDbPool = build_db_pool_for_tests();
```

`new_string`:
```
    const SIGNING_SEED_HEX: &str =
      "0000000000000000000000000000000000000000000000000000000000000001";
    let _g_init = EnvVarGuard::set("LEMMY_INITIALIZE_WITH_DEFAULT_SETTINGS", "1");
    let _g_gov = EnvVarGuard::set("GOVERNANCE_LOG_SIGNING_KEY", SIGNING_SEED_HEX);

    let (_container, host_port) = governance_fixtures::start_postgres().await?;
    let db_url = governance_fixtures::db_url(host_port);
    let _g_db_url = EnvVarGuard::set("LEMMY_DATABASE_URL", &db_url);
    {
      let mut sync_conn = PgConnection::establish(&db_url)?;
      governance_fixtures::apply_all_schema(&mut sync_conn)?;
    }

    let pool: ActualDbPool = build_db_pool_for_tests();
```

---

#### Edit E — `governance_events_notify_fires` (~line 4788/4790) [multi-line GOV]

The enclosing fn head is at ~4763: `async fn governance_events_notify_fires`.
Site has distinctive imports: `tokio::sync::mpsc`, `tokio_postgres::{AsyncMessage, NoTls, Notification}`:

`old_string`:
```
  use std::{pin::Pin, time::Duration as StdDuration};
  use tokio::sync::mpsc;
  use tokio_postgres::{AsyncMessage, NoTls, Notification};

  unsafe {
    std::env::set_var("LEMMY_INITIALIZE_WITH_DEFAULT_SETTINGS", "1");
    std::env::set_var(
      "GOVERNANCE_LOG_SIGNING_KEY",
      "0000000000000000000000000000000000000000000000000000000000000001",
    );
  }

  let (_container, host_port) = governance_fixtures::start_postgres().await?;
```

`new_string`:
```
  use std::{pin::Pin, time::Duration as StdDuration};
  use tokio::sync::mpsc;
  use tokio_postgres::{AsyncMessage, NoTls, Notification};

  let _g_init = EnvVarGuard::set("LEMMY_INITIALIZE_WITH_DEFAULT_SETTINGS", "1");
  let _g_gov = EnvVarGuard::set("GOVERNANCE_LOG_SIGNING_KEY", "0000000000000000000000000000000000000000000000000000000000000001");

  let (_container, host_port) = governance_fixtures::start_postgres().await?;
```

---

#### Edit F — `underscore_prefix_usernames_still_register` (~line 4923) [INIT-only, no GOV]

The enclosing fn head is at ~4904: `async fn underscore_prefix_usernames_still_register`.
INIT-only site — no GOV pair. Distinctive import: `utils::validation::is_valid_actor_name`:

`old_string`:
```
  use lemmy_utils::{
    rate_limit::RateLimit, settings::SETTINGS, utils::validation::is_valid_actor_name,
  };
  use reqwest_middleware::ClientBuilder;

  unsafe {
    std::env::set_var("LEMMY_INITIALIZE_WITH_DEFAULT_SETTINGS", "1");
  }

  let (_container, host_port) = governance_fixtures::start_postgres().await?;
```

`new_string`:
```
  use lemmy_utils::{
    rate_limit::RateLimit, settings::SETTINGS, utils::validation::is_valid_actor_name,
  };
  use reqwest_middleware::ClientBuilder;

  let _g_init = EnvVarGuard::set("LEMMY_INITIALIZE_WITH_DEFAULT_SETTINGS", "1");

  let (_container, host_port) = governance_fixtures::start_postgres().await?;
```

---

#### Edit G — `sanction_notice_round_trip` (~line 5047/5048)

The enclosing fn head is at ~4981: `async fn sanction_notice_round_trip`.
Site has a multi-line comment block above (`// GOVERNANCE_LOG_SIGNING_KEY is read by...`) + local const + SAFETY comment:

`old_string`:
```
  // GOVERNANCE_LOG_SIGNING_KEY is read by the governance log signer at first
  // call; LEMMY_INITIALIZE_WITH_DEFAULT_SETTINGS makes SETTINGS bypass the
  // config-file load. Both DBs share the same signing key — fine for v0
  // since the test only reads each chain locally.
  const SIGNING_SEED_HEX: &str = "0000000000000000000000000000000000000000000000000000000000000001";
  // SAFETY: tests run with --test-threads=1 so no concurrent env mutation.
  unsafe {
    std::env::set_var("LEMMY_INITIALIZE_WITH_DEFAULT_SETTINGS", "1");
    std::env::set_var("GOVERNANCE_LOG_SIGNING_KEY", SIGNING_SEED_HEX);
  }

  // -- 1. Boot container A + apply schema. ------------------------------
```

`new_string`:
```
  // GOVERNANCE_LOG_SIGNING_KEY is read by the governance log signer at first
  // call; LEMMY_INITIALIZE_WITH_DEFAULT_SETTINGS makes SETTINGS bypass the
  // config-file load. Both DBs share the same signing key — fine for v0
  // since the test only reads each chain locally.
  const SIGNING_SEED_HEX: &str = "0000000000000000000000000000000000000000000000000000000000000001";
  let _g_init = EnvVarGuard::set("LEMMY_INITIALIZE_WITH_DEFAULT_SETTINGS", "1");
  let _g_gov = EnvVarGuard::set("GOVERNANCE_LOG_SIGNING_KEY", SIGNING_SEED_HEX);

  // -- 1. Boot container A + apply schema. ------------------------------
```

---

#### Edit H — `appeal_inside_window_succeeds_expired_rejects` (~line 5675/5677) [multi-line GOV]

The enclosing fn head is at ~5646: `async fn appeal_inside_window_succeeds_expired_rejects`.
Multi-line GOV value (no local const). Distinctive: this fn is adjacent to `appeal` naming.

`old_string`:
```
  unsafe {
    std::env::set_var("LEMMY_INITIALIZE_WITH_DEFAULT_SETTINGS", "1");
    std::env::set_var(
      "GOVERNANCE_LOG_SIGNING_KEY",
      "0000000000000000000000000000000000000000000000000000000000000001",
    );
  }

  let (_container, host_port) = governance_fixtures::start_postgres().await?;
  let db_url = governance_fixtures::db_url(host_port);
  let _g_db_url = EnvVarGuard::set("LEMMY_DATABASE_URL", &db_url);

  {
    let mut sync_conn = PgConnection::establish(&db_url)?;
    governance_fixtures::apply_all_schema(&mut sync_conn)?;
  }

  let pool: ActualDbPool = build_db_pool_for_tests();
  let client = client_builder(&SETTINGS).build()?;
```

**NOTE:** There are multiple occurrences of the bare `unsafe { set_var(INIT); set_var(\n GOV multi-line); }` block pattern. The Edit tool will fail if `old_string` is not unique. To make it unique, include enough surrounding context lines that appear only around this fn. If needed, extend the old_string to include the fn name in a preceding `#[tokio::test]` attribute or use the presence of `is_valid_actor_name`/specific comment nearby to distinguish. At impl time, read 10 lines above and below the target block and verify it appears exactly once in the file before editing. If not unique, add more context lines from around the fn head.

`new_string` (same pattern):
```
  let _g_init = EnvVarGuard::set("LEMMY_INITIALIZE_WITH_DEFAULT_SETTINGS", "1");
  let _g_gov = EnvVarGuard::set("GOVERNANCE_LOG_SIGNING_KEY", "0000000000000000000000000000000000000000000000000000000000000001");

  let (_container, host_port) = governance_fixtures::start_postgres().await?;
  let db_url = governance_fixtures::db_url(host_port);
  let _g_db_url = EnvVarGuard::set("LEMMY_DATABASE_URL", &db_url);

  {
    let mut sync_conn = PgConnection::establish(&db_url)?;
    governance_fixtures::apply_all_schema(&mut sync_conn)?;
  }

  let pool: ActualDbPool = build_db_pool_for_tests();
  let client = client_builder(&SETTINGS).build()?;
```

---

#### Edit I — `declining_juror_not_picked_as_own_replacement` (~line 5876/5878) [multi-line GOV]

The enclosing fn head is at ~5843: `async fn declining_juror_not_picked_as_own_replacement`.
Same multi-line GOV pattern. Uniqueness: fn-adjacent context lines. At impl time, verify old_string uniqueness before editing.

`old_string`:
```
  unsafe {
    std::env::set_var("LEMMY_INITIALIZE_WITH_DEFAULT_SETTINGS", "1");
    std::env::set_var(
      "GOVERNANCE_LOG_SIGNING_KEY",
      "0000000000000000000000000000000000000000000000000000000000000001",
    );
  }

  let (_container, host_port) = governance_fixtures::start_postgres().await?;
  let db_url = governance_fixtures::db_url(host_port);
  let _g_db_url = EnvVarGuard::set("LEMMY_DATABASE_URL", &db_url);

  {
    let mut sync_conn = PgConnection::establish(&db_url)?;
    governance_fixtures::apply_all_schema(&mut sync_conn)?;
  }

  let pool: ActualDbPool = build_db_pool_for_tests();
  let client = client_builder(&SETTINGS).build()?;
  let middleware_client = ClientBuilder::new(client).build();
```

**NOTE on Edits H and I:** These two functions have nearly identical surrounding context (both are `governance_fixtures`-based test fns with the same multi-line GOV block). If the Edit tool reports a non-unique old_string, extend the old_string to include a line that appears only near the specific fn — e.g., include the `async fn declining_juror_not_picked...` line itself from the function head, even though it's several lines above. Read the file around the target line to pick a unique extension.

`new_string`:
```
  let _g_init = EnvVarGuard::set("LEMMY_INITIALIZE_WITH_DEFAULT_SETTINGS", "1");
  let _g_gov = EnvVarGuard::set("GOVERNANCE_LOG_SIGNING_KEY", "0000000000000000000000000000000000000000000000000000000000000001");

  let (_container, host_port) = governance_fixtures::start_postgres().await?;
  let db_url = governance_fixtures::db_url(host_port);
  let _g_db_url = EnvVarGuard::set("LEMMY_DATABASE_URL", &db_url);

  {
    let mut sync_conn = PgConnection::establish(&db_url)?;
    governance_fixtures::apply_all_schema(&mut sync_conn)?;
  }

  let pool: ActualDbPool = build_db_pool_for_tests();
  let client = client_builder(&SETTINGS).build()?;
  let middleware_client = ClientBuilder::new(client).build();
```

---

#### Edit J — last test-body site `two_sponsors_lose_endorsement_strength_on_sanction` in the bottom module (~line 16823/16824)

The enclosing fn head is at ~16818: `async fn two_sponsors_lose_endorsement_strength_on_sanction` — this is a DIFFERENT occurrence from Edit D (which is the earlier `ineligible_user_cannot_be_picked_for_jury` fn that uses the same `const SIGNING_SEED_HEX` pattern). Confirm line number from phase branch before editing.

`old_string`:
```
    const SIGNING_SEED_HEX: &str =
      "0000000000000000000000000000000000000000000000000000000000000001";
    // SAFETY: tests run with --test-threads=1; no concurrent env mutation.
    unsafe {
      std::env::set_var("LEMMY_INITIALIZE_WITH_DEFAULT_SETTINGS", "1");
      std::env::set_var("GOVERNANCE_LOG_SIGNING_KEY", SIGNING_SEED_HEX);
    }

    let (_container, host_port) = governance_fixtures::start_postgres().await?;
    let db_url = governance_fixtures::db_url(host_port);
    let _g_db_url = EnvVarGuard::set("LEMMY_DATABASE_URL", &db_url);
    {
      let mut sync_conn = PgConnection::establish(&db_url)?;
      governance_fixtures::apply_all_schema(&mut sync_conn)?;
```

**NOTE on Edit J:** The `ineligible_user_cannot_be_picked_for_jury` fn (Edit D) and this fn have the same indent level for the `const SIGNING_SEED_HEX` + `unsafe { set_var; set_var; }` block, but different enclosing function names. Before applying Edit J, confirm the old_string is unique (only 1 occurrence). If the Edit tool reports multiple matches, extend the old_string by also including the `let pool: ActualDbPool = build_db_pool_for_tests();` line that follows, or by including a line from the function head. At impl time: `grep -n "two_sponsors_lose_endorsement_strength_on_sanction" crates/server/tests/e2e.rs` should show 2 occurrences (one fn head, one — if grep hits the name in comments/nearby); the set_var block inside this fn should be unique if anchored with enough surrounding lines.

`new_string`:
```
    const SIGNING_SEED_HEX: &str =
      "0000000000000000000000000000000000000000000000000000000000000001";
    let _g_init = EnvVarGuard::set("LEMMY_INITIALIZE_WITH_DEFAULT_SETTINGS", "1");
    let _g_gov = EnvVarGuard::set("GOVERNANCE_LOG_SIGNING_KEY", SIGNING_SEED_HEX);

    let (_container, host_port) = governance_fixtures::start_postgres().await?;
    let db_url = governance_fixtures::db_url(host_port);
    let _g_db_url = EnvVarGuard::set("LEMMY_DATABASE_URL", &db_url);
    {
      let mut sync_conn = PgConnection::establish(&db_url)?;
      governance_fixtures::apply_all_schema(&mut sync_conn)?;
```

---

### `use` visibility check (per plan §13 Task 2)

After each edit, verify the module can see `EnvVarGuard`. Test fns at the top level of `e2e.rs` reference it directly. Test fns inside nested `mod` blocks need `use super::EnvVarGuard;` or `use super::*;`. Task 0's Probe 5 confirms the struct is at the test-crate root. Check each enclosing module at impl time — if a `mod` block that contains a newly-wrapped fn lacks `use super::EnvVarGuard`, add it near the module's existing `use` block.

## 3. Required reading

- `.claude/PRPs/plans/v1-quality-r3.plan.md` §10.1 (EnvVarGuard struct shape), §10.2 (option-a wrap pattern), §13 Task 2 (IMPLEMENT section + VALIDATE block)
- `.claude/lessons/feedback_envvarguard_fixture_lifetime_footgun.md` — why `_g_init`/`_g_gov` not `_` (named binding is load-bearing)
- `.claude/lessons/feedback_fix_impl_pre_locate_e2e_anchors.md` — verbatim anchor discipline; if Edit tool reports non-unique old_string, extend the anchor by reading surrounding context (do NOT guess or paraphrase)
- `.claude/lessons/feedback_lemmy_error_no_std_error.md` — any new error paths use `LemmyResult<()>` with `?`
- `.claude/lessons/feedback_async_pool_test_pattern.md` — context for the test fn pattern being modified
- `.claude/lessons/feedback_gate4_full_e2e_env_refactor_class.md` — after T2, a full e2e gate is MANDATORY (R12); this is why the VALIDATE block below is compile+lint only (e2e runs separately as validate-pending-laptop-e2e)
- `.claude/lessons/feedback_validate_pending_laptop_must_use_wrapper.md` — all cargo calls use `.bat` wrappers (R9)

## 4. Constraints

- **BRANCH must be `phase-v1-quality-r3`** — confirm with `git branch --show-current` as first action.
- **R11**: apply the verbatim anchors above. If an old_string is non-unique, extend it (read the surrounding lines, include more context) — do NOT paraphrase or reconstruct from memory.
- **Named bindings**: use `_g_init` / `_g_gov` — NEVER `let _ = EnvVarGuard::set(...)`.
- **R9**: all cargo invocations use `scripts\brehon\cargo-check.bat` / `cargo-clippy.bat`. Never bare `cargo`.
- **R10**: all cargo output redirected to `.claude/PRPs/debug/v1-quality-r3-task2-*.log`.
- **Only the listed sites** — do not wrap any bootstrap sites or the boot_context site.
- **Attribution**: `from: "impl"` on any DQ entry. Never `answered_by: "advisor"` from impl session.
- DQ mid-task push: if you raise a DQ blocker, immediately push to `origin/phase-v1-quality-r3`.

### VALIDATE (after all edits, before commit)

```bash
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/v1-quality-r3-task2-check.log 2>&1"
echo "check exit: $?"
tail -20 .claude/PRPs/debug/v1-quality-r3-task2-check.log

cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/v1-quality-r3-task2-clippy.log 2>&1"
echo "clippy exit: $?"
tail -20 .claude/PRPs/debug/v1-quality-r3-task2-clippy.log

python3 -c "
import re
src = open('crates/server/tests/e2e.rs').read()
def line_of(pos): return src[:pos].count('\n') + 1
def in_envvarguard_impl(pos):
    above = src[max(0,pos-600):pos]
    return 'impl EnvVarGuard' in above[-600:] or 'impl Drop for EnvVarGuard' in above[-600:]
def has_safety(pos):
    above = src[max(0,pos-400):pos]
    return '// SAFETY:' in above
raw = []
for var in ('LEMMY_INITIALIZE_WITH_DEFAULT_SETTINGS','GOVERNANCE_LOG_SIGNING_KEY'):
    for m in re.finditer(r'std::env::set_var\(\s*\"'+re.escape(var)+r'\"', src):
        pos = m.start()
        if in_envvarguard_impl(pos):
            continue
        raw.append((line_of(pos), var, has_safety(pos)))
unsafe_without_safety = [(ln,v) for (ln,v,s) in raw if not s]
if unsafe_without_safety:
    print('FAIL — raw setter site(s) without SAFETY justification:', unsafe_without_safety)
    raise SystemExit(1)
n_init = len([1 for (_,v,_) in raw if v=='LEMMY_INITIALIZE_WITH_DEFAULT_SETTINGS'])
n_gov  = len([1 for (_,v,_) in raw if v=='GOVERNANCE_LOG_SIGNING_KEY'])
if (n_init, n_gov) != (2, 2):
    print(f'FAIL — expected exactly 2 INIT + 2 GOV sanctioned bootstrap raw sites, got {n_init} INIT + {n_gov} GOV:', raw)
    raise SystemExit(1)
print(f'OK — {n_init} INIT + {n_gov} GOV raw sites remain, all SAFETY-commented (the 2 documented bootstrap exceptions); all test-body sites wrapped.')
" > .claude/PRPs/debug/v1-quality-r3-task2-audit.log 2>&1
echo "audit exit: $?"
tail -5 .claude/PRPs/debug/v1-quality-r3-task2-audit.log
```

All three must exit 0 and audit must print `OK`. If not, DO NOT commit — raise a DQ blocker.

### Commit (only after VALIDATE passes)

Subject: `refactor(e2e): wrap test-body LEMMY_INITIALIZE + GOVERNANCE_LOG setters with EnvVarGuard (task 2)`

Commit ONLY `crates/server/tests/e2e.rs`. Nothing else.

Push: `git push origin phase-v1-quality-r3`

### After commit: emit validate-pending-laptop-e2e DQ entry

Per plan §14 + R12: after the Task 2 commit and push, the impl-task must raise a `kind: "validate-pending-laptop-e2e"` entry in `.claude/decision-queue.json`:

```json
{
  "id": "<generate via bash scripts/brehon/dq-v3-new-entry.sh>",
  "from": "impl",
  "kind": "validate-pending-laptop-e2e",
  "timestamp": "<ISO 8601 UTC>",
  "question": "Run full e2e gate (R12) for v1-quality-r3 at phase tip — env-var-management refactor class mandatory",
  "options": ["pass", "fail"],
  "context": "T2 wrapped all test-body INIT/GOV sites; T1 documented 2 bootstrap exceptions. Phase tip at <SHA>. Full e2e mandatory per gate4 rule.",
  "branch": "phase-v1-quality-r3",
  "phase_task": "2",
  "commands": ["cmd //c \"scripts\\\\brehon\\\\cargo-test.bat --workspace --test e2e --features full > .claude/PRPs/debug/v1-quality-r3-e2e.log 2>&1 && echo E2E_EXIT_0 >> .claude/PRPs/debug/v1-quality-r3-e2e.log || echo E2E_EXIT_NONZERO >> .claude/PRPs/debug/v1-quality-r3-e2e.log\""],
  "result": null,
  "log_slice": null,
  "failed_commands": null,
  "answer": null,
  "answered_by": null,
  "resolved_at": null,
  "approved_by": null,
  "approved_at": null
}
```

Generate the id via `bash scripts/brehon/dq-v3-new-entry.sh`. Append to `.claude/decision-queue.json` `pending[]` using `bash scripts/brehon/dq-v3-append-fragment.sh <fragment.json> --pending`. Commit + push the DQ update to `phase-v1-quality-r3`.

The advisor laptop session runs the e2e gate; do NOT run it yourself.
