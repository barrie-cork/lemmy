---
name: Newtype crate locations — db_schema vs db_schema_file
description: LocalUserId / CommunityId / ModerationCaseId live in lemmy_db_schema::newtypes. Only PersonId and InstanceId are re-exported from lemmy_db_schema_file.
type: feedback
originSessionId: bae44467-63e7-4c21-ad41-057cefebb80e
---
When writing tests or handlers that reference ID newtypes, import from the right crate:

**lemmy_db_schema::newtypes** (authoritative definitions — `crates/db_schema/src/newtypes.rs`):
- `LocalUserId`
- `CommunityId`
- `ModerationCaseId`
- `PersonId` (also here, canonical definition)
- plus many more (PostId, CommentId, JuryAssignmentId, ReputationSnapshotId, etc.)

**lemmy_db_schema_file** (re-exports a small subset for use in the generated schema — `crates/db_schema_file/src/lib.rs`):
- `PersonId` ✓
- `InstanceId` ✓
- everything else → errors with `cannot find type X in crate lemmy_db_schema_file`

**Why:** `lemmy_db_schema_file` is a thin crate holding the generated Diesel `schema::...` module; it re-exports only the two newtypes that appear in enough schema-adjacent contexts to warrant it. The rest live in the parent `lemmy_db_schema` crate where the Diesel impls live.

**How to apply:**
- If you see `cannot find type <Id> in crate lemmy_db_schema_file`, change `lemmy_db_schema_file::<Id>` to `lemmy_db_schema::newtypes::<Id>`
- E2E tests that mix `lemmy_db_schema_file::schema::moderation_case` (the schema table reference) with an id newtype — keep the schema path with `db_schema_file`, swap the newtype path to `db_schema::newtypes`
- Happens in stash/plan code copied from design-doc snippets — the snippets pick the wrong crate path

Discovered 2026-04-19 during Phase 5c task 68/69 recovery. Three distinct instances in stash-shipped test code needed the fix.
