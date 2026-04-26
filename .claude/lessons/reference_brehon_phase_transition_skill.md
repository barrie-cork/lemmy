---
name: Brehon phase transition skill
description: Reusable skill at .claude/skills/brehon-phase-transition/SKILL.md automates the five-deliverable handoff between Brehon phases. Registered in library.yaml. Invoke with /brehon-phase-transition <completing> <next>.
type: reference
originSessionId: 50cf5c6f-6b81-45b0-b3b4-45ac46b352d8
---
Skill for automating Brehon governance platform phase transitions. Created 2026-04-16, first used for Phase 2b→3 transition.

**Location:** `.claude/skills/brehon-phase-transition/SKILL.md` in homeserver repo

**Invocation:** `/brehon-phase-transition <completing-phase> <next-phase>` (e.g., `/brehon-phase-transition 3 4`)

**Five deliverables it produces:**
1. Archive current advisor-context file (rename to `-archive.md`)
2. Archive phase notes memory (rename `_notes` to `_complete`)
3. Create fresh notes skeleton for next phase
4. Write fresh advisor-context file incorporating retro lessons
5. Generate bootstrap prompt for next advisor session

Also updates MEMORY.md index and `project_brehon_governance_platform.md` phase status.
