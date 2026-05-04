---
name: ADR-013 red-flag scanner false-positive on EmergencyRemove arm-split refactors
description: The github-actions ADR-013 scanner is regex-only and can't read Rust `match` exhaustiveness. Every PR where an EmergencyRemove arm is split into sibling arms (even when both halves enumerate the variant exhaustively) trips the scanner. 4× recurrence (PR #107 1× + PR #111 10×); needs an AST-aware/exemption fix, not per-PR maintainer ack.
type: feedback
---
**Rule:** the github-actions ADR-013 red-flag scanner produces false-positives on every PR that contains an `EmergencyRemove` arm-split refactor. The pattern is "an ADR-013 variant was removed" detected via regex, but the variant is actually preserved in sibling arms — the scanner can't see Rust match exhaustiveness. Per-PR maintainer-ack is high-friction; the scanner needs an AST-aware/exemption fix.

**Why this matters:**
- 4× recurrence so far:
  - **PR #107 (v1-JM-e):** 1× false-positive on `accept_jury_assignment.rs` flat-match → role-dispatch refactor. EmergencyRemove preserved exhaustively in both `Original` and `Appeal` role arms; net +2 references. Acked via `pr-107-redflag-ack.md`.
  - **PR #111 (v1-SL-a):** 10× false-positive on `is_public_status` arm split in `crates/api/api/src/governance/get_case.rs:48-55` (3 statuses → 6 statuses split). EmergencyRemove preserved in the false-arm enumeration. Each push re-fires the scanner.
- Pattern: any arm-split refactor that splits an EmergencyRemove enumeration trips the scanner. This is the **modal outcome** whenever a sub-phase touches EmergencyRemove arms, not an edge case.
- Cost: ~5 min per PR to author the maintainer-ack citation chain + post in PR comment + admin-bypass merge. Cumulative annoyance >> per-PR cost.
- The scanner is regex-based: `grep` for `CaseStatus::EmergencyRemove` removed in diff. Doesn't understand AST structure or Rust's exhaustive-match guarantee.

**How to apply:**
- **Per-PR maintainer-ack pattern (workaround until fix):** in the PR body, post a "Maintainer ack — red-flag advisory (ADR-013 EmergencyRemove arm)" comment with:
  1. Citation of every line where EmergencyRemove appears post-refactor
  2. Diff verification showing net change is +N references (not removal)
  3. Explanation that exhaustive arm enumeration preserves the variant
- See `.claude/PRPs/reviews/pr-107-redflag-ack.md` for the canonical citation form.
- **Forward gate (proposed v1-SL-a §7 follow-up #7):** queue infra fix at `barrie-cork/lemmy` against the ADR-scanner workflow:
  - **(a)** AST-aware scanner — read Rust syntax via `syn` or `rust-analyzer`, not regex
  - **(b)** Exemption-pattern allowlist — add a workflow-level exemption for arm-split refactors that preserve the variant in both halves
  - **(c)** Downgrade to informational — change the scanner from "advisory blocking" to "informational" on PRs into governance-v0 (CR + maintainer review already covers the case)
- **Highest-leverage:** this is among the highest-priority infra fixes from SL-a — the false-positive class will recur on every sub-phase that touches EmergencyRemove arms (most v1 sub-phases will).

**Generalises to:** any ADR-bound enum invariant where exhaustive matching is mandatory and the scanner is regex-only. Not specific to ADR-013 / EmergencyRemove — same class would fire on any `_ => unreachable!()` removal that's been replaced with explicit arms.

**Symptom to recognise:** a PR that does an arm-split or flat-match → role-dispatch refactor on a CaseStatus or similar ADR-bound enum gets a github-actions "Removal of `<variant>` reference" red-flag on every push. The scanner posts the same finding repeatedly with each new commit; the maintainer-ack must be re-stated or, more typically, the admin-bypass relies on the prior ack carrying forward.

**Retire when:** the ADR-scanner workflow ships the AST-aware fix (option a) or the exemption allowlist (option b). Until then, per-PR maintainer-ack is the workaround. Source: v1-SL-a retro §3.5 + §7 follow-up #7.
