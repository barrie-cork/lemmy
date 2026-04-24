# Impl → advisor relays

One file per relay. Filename = stable `id` used in frontmatter.

## Filename convention

`<id>.md` where `id` is `<scope>-<artifact>` kebab-case:
- `task<N>-status` — status report at task close
- `R<N.M>-ask` — question raised against a risk-register entry
- `DQ<N>-draft` — DQ framing drafted by impl for advisor to answer
- `blocker-<slug>` — impl hit something it cannot self-resolve
- `adhoc-<slug>` — everything else

## Schema (required)

```
---
id: <kebab-case, matches filename minus .md>
from: impl
to: advisor
ts: <ISO 8601 UTC, minute precision>
relates_to: <plan-task, DQ#, risk-register entry, or file path>
blocking: <true|false>
---

# Context
<one paragraph: what impl was doing when this came up>

# Ask
<numbered list of questions or options for advisor to pick; imperative>

# Evidence
<bullet list: file:line refs, commit SHAs, test names that substantiate the ask>

# Proposed
<impl's own lean if they have one; omit if genuinely undecided>
```

## Fixed-vocabulary section headings

Use these exact strings, in this order, omitting sections that don't apply:

1. `# Context`
2. `# Ask`
3. `# Evidence`
4. `# Proposed`

Scripts can split on `^# ` to extract sections; frontmatter yields with `yq`.

## What NOT to include

- Cargo output tails (use a file path in `# Evidence`; paste only if <10 lines)
- Speculation about what advisor might say
- Decisions impl could self-resolve with existing docs
- Meta-commentary about the relay itself

## Companion

- `../advisor-relays/` — advisor → impl direction
- `../bm-runlog.md` — BM/advisor state-changes
