---
name: Windows — author DQ/JSON values containing backslash paths via a Write-tool fragment, never inline python -c
description: A DQ/JSON value that must contain a Windows backslash path (e.g. cmd //c "scripts\brehon\cargo-check.bat ...") will be silently mangled if built via inline `python -c` / heredoc. `\b`→backspace gives `scriptsrehon`; `\c` is a bad-escape warning; `\'` is a SyntaxError. Author the entry as a JSON fragment via the Write tool, then merge by reading the fragment file — zero shell/Python string escaping.
type: feedback
---

When a `decision-queue.json` entry (or any JSON) must carry a value containing a **Windows backslash path** — typically a `validate-pending-laptop` entry whose `commands[]` are `cmd //c "scripts\brehon\cargo-*.bat ..."` — building that entry via an inline `python -c "..."` or a `python << 'PYEOF'` heredoc **silently corrupts the path**:

- `\b` in a Python string literal → **backspace control char**. `scripts\brehon` becomes `scriptsrehon` in the written file. The JSON is structurally valid; the path is wrong; the laptop validation then runs a non-existent command.
- `\c` → `SyntaxWarning: invalid escape sequence` (harmless but noisy; the literal backslash survives by luck, not design).
- `\'` (escaping a quote next to a backslash) → `SyntaxError: unexpected character after line continuation character` — the **entire script fails to parse**, so the `json.dump` never runs (and any `&&`-chained cleanup also doesn't).

Even verification scripts are affected: `'scripts\brehon\' in c` is itself a Python `SyntaxError` (unterminated literal). Checking for the substring requires `chr(92)` (backslash) constructed at runtime, not a backslash literal in the source.

**Confirmed:** v1-AD-e DQ #246 reconstruction (2026-05-17). Two consecutive bugs (`\b`→`scriptsrehon`, then `\'` SyntaxError) cost ~3 extra tool calls + a fallback Write-fragment workaround before the entry was correct.

This is **distinct from** `feedback_windows_bash_python_git_show_tmp_traps` (that's `git show <ref>:<path>` colon/slash path-mangling by the Windows shell). This one is **Python-source backslash-in-string-literal mangling** — a different layer.

## How to apply

**The reliable pattern (zero escaping):**

1. Author the DQ/JSON entry as a standalone fragment file with the **Write tool** (the Write tool does no shell/Python escaping — backslashes in the content are literal):
   `.claude/dq<N>-fragment.json` containing the single entry object, with `cmd //c "scripts\\brehon\\cargo-check.bat ..."` written exactly (JSON-escaped backslashes — `\\` — which is correct JSON and what `json.load` round-trips to a single backslash).
2. Merge by a Python script that **reads the fragment file** (`json.load(open(fragment))`) and appends it — the script contains NO path literals, so nothing to mangle:
   ```python
   d = json.load(io.open('.claude/decision-queue.json', encoding='utf-8'))
   frag = json.load(io.open('.claude/dq<N>-fragment.json', encoding='utf-8'))
   d['pending'].append(frag)
   json.dump(d, io.open('.claude/decision-queue.json','w',encoding='utf-8'), ensure_ascii=False, indent=2)
   ```
3. Verify with a backslash check built via `chr(92)` (NOT a literal): `needle = 'scripts' + chr(92) + 'brehon' + chr(92); assert needle in cmd`.
4. `rm` the fragment file after the merge succeeds.

**Do NOT** build a `commands[]` value containing `scripts\brehon\...` via inline `python -c` or a heredoc with the path in the Python source. The mangle is silent (the `\b` case) — the file looks written, the JSON validates, and the corruption only surfaces when the laptop runs the broken command.

**Generalises to:** any Windows tool-call that constructs JSON/structured content containing backslash paths through a Python/shell string layer. Route the content through a Write-tool file and read it back; never let a backslash path transit a Python string literal.
