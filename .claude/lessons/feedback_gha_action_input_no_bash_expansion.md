---
name: GHA action inputs are literal — no bash $(...) expansion
description: GitHub Actions passes `with:` inputs to actions as literal strings; only ${{ }} GHA expressions are expanded. Don't put $(cat /tmp/file) in an action's prompt/input — it ships verbatim
type: feedback
originSessionId: 90e5f20a-76ea-4cb8-81bd-3ba4b68943eb
---
When wiring up `anthropics/claude-code-action@v1` (or any GHA action), the `with:` inputs are passed as **literal strings**. Bash command substitution `$(...)` and shell variables `$FOO` inside an input value are **NOT expanded** by the action runner. Only GitHub Actions expression syntax `${{ ... }}` is interpolated before the action receives the input.

**Why:** caught two latent bugs in `.github/workflows/governance-ai-review.yml` and `.github/workflows/claude-code-action.yml` on 2026-04-25. Both files had:

```yaml
prompt: |
  ...
  $(cat /tmp/rubric.md)
```

Claude was receiving the literal string `$(cat /tmp/rubric.md)` as part of the prompt body. The previous run step had assembled the rubric file successfully — but it never made it into the prompt. The action ran without error (Claude posted a comment) and nobody noticed because Claude's output looked plausible despite missing the actual rubric. Silent-failure pattern.

**How to apply:** when an action's input needs file contents, use the GHA multiline step-output pattern:

```yaml
- name: Load file into step output
  id: payload
  run: |
    {
      echo "body<<EOF"
      cat /tmp/rubric.md
      echo "EOF"
    } >> "$GITHUB_OUTPUT"

- uses: anthropics/claude-code-action@v1
  with:
    prompt: |
      ## Rubric
      ${{ steps.payload.outputs.body }}
```

Single-line outputs work too: `echo "name=value" >> "$GITHUB_OUTPUT"`. For multiline, the heredoc form (`name<<EOF\n...\nEOF`) is mandatory.

**Step-output limits:** values up to ~1MB per output; ~1MB total per `$GITHUB_OUTPUT` file per step. A 28KB rubric or diff fits comfortably.

**Detection:** when reviewing any GHA workflow that uses `prompt:`, `script:`, `text:`, or any other free-form input on a third-party action, grep the value for `\$\(`. If found, it's almost certainly a bug — the only legitimate use is when the *literal* `$(...)` text is what you want to send to the action (rare).

**Related rule:** `.claude/rules/cargo-output-capture.md` — same family of "the runtime is doing something different than you think" bugs, applied to bash pipes vs cargo exit codes.
