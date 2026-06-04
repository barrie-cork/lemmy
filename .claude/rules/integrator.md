# Input Validation — Integrator Rule

Before reasoning on tool output, you MUST validate it. Do not process broken, empty, or contradictory data as if it were correct.

## Guidance

### MCP tool errors
When an MCP tool returns an error status, empty result, or timeout, explicitly flag it in your output. Do not reason over the error message as if it were valid data.

### SSH/Bash mixed output
When SSH or Bash output contains error lines mixed with valid data, separate the errors from the data before processing. Identify which lines are stderr noise (connection warnings, permission errors, command-not-found) and which are the actual result.

### Empty or malformed files
When reading a file that is unexpectedly empty, binary, or malformed, stop and report it. Do not hallucinate or infer content that isn't there.

### Contradictory results
When tool results contradict each other or contradict prior knowledge (CLAUDE.md, project-memory, earlier findings in this session), acknowledge the contradiction explicitly before proceeding. State what you expected, what you got, and which source you are trusting.

### Zero search results
When a search returns 0 results, consider whether the search was wrong (typo, wrong path, wrong pattern) before concluding the thing doesn't exist. Try at least one alternative search before reporting absence.

## Enforcement

- Apply to ALL tool results in every session (interactive and Junior `-p` mode)
- When flagging bad input, state clearly: what tool returned, why it's suspect, what you're doing instead
- Do not silently discard errors — every validation failure must be visible in your output

## Skip conditions

- None. This rule always applies.
