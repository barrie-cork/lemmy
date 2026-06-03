# Merge-forward from governance-v0 can surface clippy debt

After every merge-forward that pulls quality-r* or lint-fix commits from
`governance-v0`, run `brehon-verify` (or at minimum
`cargo clippy --workspace --features full --no-deps -- -D warnings`)
before queueing the next Junior task.

**Why:** the phase branch's own code may be clean, but the incoming
commits may introduce fixes to other crates that trip `-D warnings` in
the now-updated workspace. v1-redaction-r1 example: quality-r3c added
`reputation_snapshot.rs` clippy fixes; the pre-bm-pr merge-forward
pulled them in; `brehon-verify` failed on reputation_snapshot.rs even
though redaction.rs was untouched. Fix-impl-1 added ~1 hour.

**How to apply:**
1. After any merge-forward whose incoming commits include subjects
   containing `clippy`, `lint`, `quality-r*`, or `fix(lint)`:
   run `cargo clippy --workspace --features full --no-deps -- -D warnings`
   before authoring the next impl-task or bm-pr brief.
2. If the clippy run fails on a file the phase branch never touched,
   file a fix-impl brief scoped to that file only (not a phase-branch
   regression — pre-existing debt surfaced by workspace update).
3. If the clippy run passes, no DQ needed — advance.

**Source:** `8b272783b` LESSON trailer, v1-redaction-r1 fix-impl-1 brief
(2026-06-01).
