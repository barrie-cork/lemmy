# Advisor → impl relays

One file per relay. Filename = stable `id` used in frontmatter.

## Filename convention

`<id>.md` where `id` is `<scope>-<artifact>` kebab-case:
- `R<N.M>-answer` — answer to a risk-register entry (e.g. `R10.1-answer`)
- `DQ<N>-answer` — answer to a DQ entry (e.g. `DQ47-answer`)
- `task<N>-<slug>` — guidance for a specific task (e.g. `task11-retro-prep`)
- `adhoc-<slug>` — everything else

## Schema (required)

```
---
id: <kebab-case, matches filename minus .md>
from: advisor
to: impl
ts: <ISO 8601 UTC, minute precision>
relates_to: <plan-task, DQ#, risk-register entry, or file path>
decision: <single token; omit for info-only relays>
---

# Decision
<one sentence verdict; omit section for info-only relays>

# Instructions
<numbered list, one action per line; imperative verbs>

# Retro carry (Task N)
<bullet list of items to capture at phase retro; omit if none>

# Next
<one sentence: what impl does immediately after pasting>
```

## Fixed-vocabulary section headings

Use these exact strings, in this order, omitting sections that don't apply:

1. `# Decision`
2. `# Instructions`
3. `# Retro carry`
4. `# Next`

Scripts can split on `^# ` to extract sections; frontmatter yields with `yq`.

## What NOT to include

- Greeting prose ("Hi impl, here's the answer")
- Advisor reasoning narrative (put reasoning in commit messages or risk register, not relays)
- Meta-commentary about the relay itself
- Citations to external URLs (use relative repo paths)

## Companion

- `../impl-relays/` — impl → advisor direction
- `../bm-runlog.md` — BM/advisor state-changes
