**Maintainer ack — red-flag advisory (ADR-013 EmergencyRemove arm)**

Acknowledged. The Task 2 fix-impl at commit `360c24cb9` refactored `accept_jury_assignment.rs` from a flat `match case.status` into a role-dispatch `match role { Original => match status, Appeal => match status }`. The `CaseStatus::EmergencyRemove` variant is preserved exhaustively in **both** role arms (lines 118 + 131 on the post-merge tip) and a third occurrence is added in `submit_jury_vote.rs::process_appeal_vote` as a terminal-state guard (line ~782). Net: ADR-013 invariant is strengthened, not weakened. The regex scanner can't see the refactor.

Diff verified against `governance-v0..phase-v1-JM-e`:

```
-    | CaseStatus::EmergencyRemove        # one removed (the original flat-match arm)
+      | CaseStatus::EmergencyRemove      # added in role=Original × status arm
+      | CaseStatus::EmergencyRemove      # added in role=Appeal × status arm
+    CaseStatus::Closed | CaseStatus::EmergencyRemove | CaseStatus::AdminReview  # added in process_appeal_vote terminal guard
```

Net change: +2 references to the variant. Safe to merge past this advisory.

— Brehon maintainer ack, 2026-05-02
