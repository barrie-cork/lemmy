# Conformance-Audit Dogfood — `v1-federation-inbound-b` (Two Snapshots)

**Date authored:** 2026-05-21 (Task 7 dogfood execution)  
**Plan reference:** `.claude/PRPs/plans/brehon-conformance-audit.plan.md` §13 Task 7  
**Skill version:** 1.0.0 (axes 1-6, SKILL.md + six axis sub-files shipped at Task 1/2)  
**Scope:** PRECON-8 two-snapshot calibration proving axis-4 detection catches Finding 6.1

## Purpose

This dogfood report calibrates the `brehon-conformance-audit` skill against known ground truth
from `v1-federation-inbound-b`. The skill is applied to two snapshots of
`crates/apub/activities/src/governance/inbox.rs`:

- **Snapshot 1** (pre-fix-impl-3): the `.unwrap_or_default()` footgun is present.
- **Snapshot 2** (current merged tip): fix-impl-3 (`8b04e69a6`) has closed the finding.

Expected result: skill MUST flag axis-4 Tier-1 on Snapshot 1, MUST NOT flag it on Snapshot 2.

---

## Snapshot 1: pre-fix-impl-3 (SHA 649871f7d, parent of 8b04e69a6)

**SHA:** `649871f7d6a60d19fab57877003bd8da1c67ce86`  
**File inspected:** `crates/apub/activities/src/governance/inbox.rs`

### Axis-4 detection (per `axes/4-error-idiom.md`)

**Step 1 — grep for `.unwrap_or_default()` at trust-boundary sites:**

```
$ git show 649871f7d:crates/apub/activities/src/governance/inbox.rs | grep -n "\.unwrap_or_default()"
743:    .unwrap_or_default();
```

One hit at **line 743**, inside `receive_remote_moderation_label` (function starts at line 733).

**Step 2 — context check (5 lines before L743):**

```rust
// L738-743 (Snapshot 1):
  let peer_domain = activity
    .actor
    .inner()
    .domain()
    .map(str::to_string)
    .unwrap_or_default();
```

Confirms: `.unwrap_or_default()` is applied directly to `.domain()` on a remote actor — a
trust-boundary `Option<&str>` → `String` conversion that silently produces `""` when the
remote actor has no domain. This is the defining pattern for axis-4 Tier-1 flags.

**Step 3 — locate same-file sibling with hard-error pattern:**

```
$ git show 649871f7d:crates/apub/activities/src/governance/inbox.rs | grep -n "\.ok_or_else"
153:    .ok_or_else(|| {
265:    .ok_or_else(|| {
432:  val.ok_or_else(|| {
```

Primary sibling at **line 153** — `receive_remote_sanction_notice` (function starts L136):

```rust
// L150-162 (Snapshot 1, receive_remote_sanction_notice):
  let source_instance = activity
    .actor
    .inner()
    .domain()
    .ok_or_else(|| {
      LemmyErrorType::Unknown(format!(
        "remote sanction notice actor {} has no domain",
        activity.actor.inner(),
      ))
    })?
    .to_string();
```

Secondary sibling at **line 265** — `receive_remote_trust_attestation` (function starts L249):

```rust
// L262-273 (Snapshot 1, receive_remote_trust_attestation):
  let peer_domain = activity
    .actor
    .inner()
    .domain()
    .ok_or_else(|| {
      LemmyErrorType::Unknown(format!(
        "remote trust attestation actor {} has no domain",
        activity.actor.inner(),
      ))
    })?
    .to_string();
```

**Verdict:** `receive_remote_moderation_label` uses `.domain().map(str::to_string).unwrap_or_default()`
while two same-file siblings four lines away use `.domain().ok_or_else(|| LemmyErrorType::Unknown(...))?`.
This is a **Tier-1 axis-4 divergence** — enforced contract weakening with sibling proof.

### Axis-4 prediction (Snapshot 1)

| Field | Value |
|---|---|
| axis | 4 |
| risk_tier | 1 |
| target | `crates/apub/activities/src/governance/inbox.rs:743` |
| sibling | `crates/apub/activities/src/governance/inbox.rs:153` |
| evidence | `axis-4: inbox.rs:743 new=.unwrap_or_default() sibling=inbox.rs:153 sibling=.ok_or_else(\|\| LemmyErrorType::Unknown(...))?` |

**Result: FLAGGED (Tier-1)** ✓

### Other axes (Snapshot 1)

Axis-1 through 6 detection commands run against `inbox.rs` Snapshot 1 confirm no additional
Tier-1 findings in `receive_remote_moderation_label`. Sibling-divergence check passes for
axes 1 (conn-type), 2 (append reborrow), 3 (trait-bound), 5 (conn acquisition), 6 (ADR-015
pseudonym) — the new function follows the established pattern on each of these axes.

---

## Snapshot 2: current merged tip (SHA 4a60667c9)

**SHA:** `4a60667c9a4938b62d3150ce8677ffcd429ca9c4` (includes fix-impl-3 `8b04e69a6`)  
**File inspected:** `crates/apub/activities/src/governance/inbox.rs`

### Axis-4 detection (post-fix)

**Step 1 — grep for `.unwrap_or_default()` at trust-boundary sites:**

```
$ git show 4a60667c9:crates/apub/activities/src/governance/inbox.rs | grep -n "\.unwrap_or_default()"
(no output)
```

Zero hits. The `.unwrap_or_default()` call on `.domain()` is gone.

**Step 2 — verify the fix in `receive_remote_moderation_label`:**

```rust
// L738-748 (Snapshot 2, receive_remote_moderation_label — post-fix):
  let peer_domain = activity
    .actor
    .inner()
    .domain()
    .ok_or_else(|| {
      LemmyErrorType::Unknown(format!(
        "remote moderation label actor {} has no domain",
        activity.actor.inner(),
      ))
    })?
    .to_string();
```

The function now mirrors `receive_remote_trust_attestation` (L265) exactly — same
`.ok_or_else(|| LemmyErrorType::Unknown(format!(...)))?.to_string()` idiom,
message noun = "moderation label".

### Axis-4 prediction (Snapshot 2)

**Result: NOT FLAGGED** ✓ — no `.unwrap_or_default()` on a trust-boundary `.domain()` call
in `receive_remote_moderation_label`. Prediction array for Snapshot 2 axis-4 target is empty.

---

## Cross-snapshot diff

| Item | Snapshot 1 (`649871f7d`) | Snapshot 2 (`4a60667c9`) |
|---|---|---|
| `inbox.rs:743` pattern | `.map(str::to_string).unwrap_or_default()` | `.ok_or_else(\|\| LemmyErrorType::Unknown(...))?` |
| axis-4 Tier-1 flag on `receive_remote_moderation_label` | **YES** | **NO** |
| `grep -c "unwrap_or_default()" inbox.rs` | 1 | 0 |
| `grep -c "has no domain" inbox.rs` | 2 | 3 |

**Conclusion:** axis-4 Tier-1 flag appears in Snapshot 1 and disappears in Snapshot 2.
The cross-snapshot diff confirms the skill's detection is **calibrated to reality** — it
flags the defect before the fix, clears after the fix.

Fix commit: `8b04e69a6655698305a38d4c22c300480ccbe4e6`  
Author-date: `2026-05-19T20:27:54+00:00`

---

## Ground-truth seeding

### Fix-commit body (`git log -1 --format=%B 8b04e69a6`)

```
fix(v1-federation-inbound-b): receive_remote_moderation_label hard-errors on domainless actor (Phase-6 sibling mirror, Finding 6.1, fix-impl 3)

## 1-hunk change

Before (L742-743):
    .map(str::to_string)
    .unwrap_or_default();

After (L742-748):
    .ok_or_else(|| {
      LemmyErrorType::Unknown(format!(
        "remote moderation label actor {} has no domain",
        activity.actor.inner(),
      ))
    })?
    .to_string();

Mirrors receive_remote_trust_attestation L261-271 verbatim, message noun = moderation label.

## §2.4 grep-acceptance proof

- grep -c "unwrap_or_default()" inbox.rs → 0 (PASS: Finding-6.1 site gone)
- grep -c "has no domain" inbox.rs → 3 (PASS: 2 siblings + 1 new)
- grep -n "remote moderation label actor {} has no domain" inbox.rs → L744 (PASS: exactly 1)
- grep -c siblings → 2 (PASS: unchanged)
- git diff --stat → 1 file changed, 7 insertions(+), 2 deletions(-) (PASS: 1 hunk only)

## §4.2 pre-push cargo-check + cargo-clippy -D warnings

- cargo-check.sh --workspace --features full → FI3_CHECK_EXIT_0 (GREEN)
- cargo-clippy.sh --workspace --features full --no-deps -- -D warnings → FI3_CLIPPY_EXIT_0 (GREEN)
```

### Compiler verdict (ground_truth_compile_caught)

The `.unwrap_or_default()` on `.domain()` at L743 **compiled clean** under
`cargo check --workspace --features full`. This is the defining characteristic of a
**latent footgun**: the compiler sees only the type `String` produced by `unwrap_or_default()`
and accepts it — it has no knowledge that the empty-string default violates the federation
trust-boundary invariant. The `disallowed_methods` Clippy lint (axis-4 Track B) would catch
a naked `.unwrap_or_default()` in the module root, but this call site was in a function body,
not a module-level deny scope, at the time of the original commit.

`ground_truth_compile_caught: []` — this is intentional. The finding was LATENT.

### Runtime confirmation (ground_truth_runtime)

Fix-impl-3 commit `8b04e69a6` is the ground-truth runtime confirmation per Rule 3 (post-merge
bug fix within 30 days, same file, axis-relevant pattern modification). The commit modifies
exactly the `inbox.rs:743` site that axis-4 flagged, replacing `.unwrap_or_default()` with the
canonical `.ok_or_else(|| LemmyErrorType::Unknown(...))` idiom.

```json
{
  "axis": "4",
  "target": "crates/apub/activities/src/governance/inbox.rs:743",
  "source": "fix-impl-3",
  "evidence_commit_sha": "8b04e69a6655698305a38d4c22c300480ccbe4e6"
}
```

---

## Metrics computed

Seed file: `.claude/PRPs/audit-metrics/v1-federation-inbound-b.json`

```
$ bash .claude/skills/brehon-conformance-audit/scripts/compute-metrics.sh \
    .claude/PRPs/audit-metrics/v1-federation-inbound-b.json
============================================================
Brehon Conformance-Audit Metrics
============================================================
Files: .claude/PRPs/audit-metrics/v1-federation-inbound-b.json

Axis        TP    FP    FN  Precision   Recall
--------------------------------------------------------
axis-4        1     0     0      1.000    1.000

axis-4 precision: 1.000
axis-4 recall: 1.000

Lead time (median): 27.5h after ground-truth event (1 data point(s))

Latent-footgun catch rate (axis-4): 1
  (axis-4 findings the skill caught but the compiler missed)

============================================================
```

### Interpretation

| Metric | Value | Meaning |
|---|---|---|
| axis-4 precision | 1.000 | 1 true positive, 0 false positives — the one prediction was correct |
| axis-4 recall | 1.000 | 1 true positive, 0 false negatives — the skill caught the only known instance |
| Lead time | +27.5h | Retrospective dogfood: skill "run" on 2026-05-21, fix landed 2026-05-19T20:27Z |
| Latent-footgun catch rate | 1 | The prediction caught a runtime-class footgun the compiler did NOT catch |

The +27.5h lead time is expected for a retrospective calibration dogfood. In production use (skill
invoked at brief-author time, before fix-impl-3 would have been written), the lead time would be
negative (skill flags before the ground-truth event), which is the target behaviour.

---

## Calibration conclusion

The `brehon-conformance-audit` skill correctly:

1. **Flags** `receive_remote_moderation_label` at `inbox.rs:743` as axis-4 Tier-1 in Snapshot 1
   (pre-fix-impl-3), citing `receive_remote_sanction_notice` (L153) as the sibling proof.
2. **Clears** the flag in Snapshot 2 (post-fix-impl-3), confirming that the detection is
   specific to the defect pattern and does not produce false positives after the fix lands.
3. **Records** the latent-footgun category correctly: compiler clean → runtime confirmation →
   latent-footgun catch rate = 1.

Calibration data point committed to `.claude/PRPs/audit-metrics/v1-federation-inbound-b.json`.
METRICS.md `## Worked example` section updated with real numbers.
