# Brief: m3-core-infra fix-impl-cr4b (E0308 in cr-4 route test)

## §1 Role + dispatch line

`[role:impl-task] m3-core-infra-fix-cr4b-e0308-sessionmiddleware-deref — see .claude/PRPs/briefs/m3-core-infra-fix-impl-cr4b.md`

## §2 Scope

Fix a single E0308 type-mismatch in the cr-4 route test the laptop e2e validation caught.
The new `m3_actor_pseudonym_endpoint_route...` test passes `(*context).clone()` to
`SessionMiddleware::new(...)`, but `*context` is `Arc<LemmyContext>` (Actix `Data<LemmyContext>`
derefs to `Arc<T>`) while `SessionMiddleware::new` wants a bare `LemmyContext`. One `*` short.

**The fix (exact, single line, `crates/server/tests/e2e/governance.rs:5476`):**
```rust
      .wrap(SessionMiddleware::new((*context).clone()))
```
→
```rust
      .wrap(SessionMiddleware::new((**context).clone()))
```
`context` is `Data<LemmyContext>` (from `governance_fixtures::bootstrap()`, return type
`Data<LemmyContext>` per `crates/server/tests/e2e/common/mod.rs:741`). `Data<T>` derefs to
`Arc<T>`; `**context` = `LemmyContext`, which is what `SessionMiddleware::new(context: LemmyContext)`
takes (`crates/routes/src/middleware/session.rs:21`). The adjacent `.app_data(context.clone())`
is ALREADY correct (`context` is `Data`, app_data wants `Data`) — do NOT change it.

**Produces:** one 1-character edit (`*context` → `**context`) on that one line. Nothing else.

**Do NOT:** change any other line, file, or the `.app_data` call. Do NOT touch cr-5/cr-6/cr-7/cr-8
(already validated). cr-3 is rebutted.

**Branch:** forks from `phase-m3-core-infra` (current tip `c2bdde353`).

## §3 Required reading

- `crates/server/tests/e2e/governance.rs:5473-5479` — the broken `init_service` block.
- `crates/server/tests/e2e/governance.rs:3241-3247` — the canonical sibling harness (note: ITS
  `context` is a bare `LemmyContext` so it uses `Data::new(context.clone())` + `new(context.clone())`;
  the NEW test's `context` is already `Data<LemmyContext>`, hence the `**` double-deref — do NOT
  blindly copy the sibling's single-deref form).
- `.claude/lessons/feedback_fix_impl_pre_locate_e2e_anchors.md` — pre-locate the verbatim anchor.

## §4 Constraints

### Anchor uniqueness gate (pre-edit)

`grep -c 'SessionMiddleware::new((\*context).clone())' crates/server/tests/e2e/governance.rs` → must be `1`.
If not 1, STOP + raise a DQ blocker.

### DoD

`grep -c 'SessionMiddleware::new((\*\*context).clone())' crates/server/tests/e2e/governance.rs` → `1`
AND `grep -c 'SessionMiddleware::new((\*context).clone())' ...` → `0` (old form gone).

### Validate

Write a `validate-pending-laptop-e2e` DQ entry:
```json
{
  "kind": "validate-pending-laptop-e2e",
  "commands": ["cmd //c \"scripts\\\\brehon\\\\cargo-test.bat --workspace --test e2e --features full > .claude/PRPs/debug/m3-cr4b-e2e.log 2>&1\""],
  "branch": "phase-m3-core-infra",
  "phase_task": "cr-4b",
  "e2e_filter": "test(m3_actor_pseudonym)"
}
```
**Commit + push + RAISE THE DQ (commit it + push it) BEFORE finalize** — the prior cr-fix
workers' validate DQs went missing at finalize-merge; ensure this one is committed to the worker
branch so the advisor sees it. Then **stop** — do NOT run cargo. Commit subject:
`fix(e2e): cr-4b correct SessionMiddleware Data deref (E0308 fix-in-pr)`. Add a `LESSON:` trailer.
