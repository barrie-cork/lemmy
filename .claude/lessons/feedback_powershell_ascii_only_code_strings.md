---
name: PowerShell scripts must be ASCII-only in code strings (Win PS 5.1 cp1252 parse break)
description: A non-ASCII char (em-dash, curly quote, §, arrow) inside a Write-Host / string literal in a .ps1 file is read by Windows PowerShell 5.1 as cp1252 mojibake, which can break the string terminator and fail the PARSE — and depending on invocation path the failure can masquerade as a silent no-op rather than a loud error. Keep all code-string literals ASCII; the 2026-06-12 sync-user-skills.ps1 build burned ~8 debug probes on this before the em-dash was spotted.
type: feedback
---

When a `.ps1` file is run under **Windows PowerShell 5.1** (`powershell.exe`, the default on the laptop), the parser reads the file as the system ANSI codepage (cp1252), NOT UTF-8. A non-ASCII character inside a string literal — em-dash `—`, curly quotes `“ ” ‘ ’`, `§`, arrows `→`, `✅`, en-dash `–` — is decoded as mojibake (`—` → `â€"`). When that mojibake lands inside a double-quoted `Write-Host "..."` literal, the stray bytes can break the string terminator and **fail the parse of the whole script**:

```
The string is missing the terminator: ".
Missing closing '}' in statement block or type definition.
    + FullyQualifiedErrorId : TerminatorExpectedAtEndOfString
```

## Why this is worse than the Python equivalent

`feedback_python_utf8_encoding_windows.md` covers the *data* version of this trap — Python's `open()` defaults to cp1252 and silently corrupts non-ASCII *content* on write-back. The PowerShell version is a different, sometimes nastier failure:

- **Python:** mojibake corrupts the *data* the script processes; the script still runs. Caught by `git diff`.
- **PowerShell:** mojibake corrupts the *code* itself; the script may fail to PARSE. And — the part that costs debugging time — **depending on the invocation path, the parse failure can surface as a silent no-op** (the script appears to "run" and even prints a success message from a partially-parsed earlier block) rather than a loud `ParserError`. The 2026-06-12 `sync-user-skills.ps1` build hit exactly this: the script printed "Snapshot updated" while writing zero files, because the em-dashes in three `Write-Host` literals corrupted the parse. ~8 debug probes chased phantom bugs in path math and copy logic before the em-dash was the actual cause.

## How to apply

1. **Keep every code-string literal in a `.ps1` ASCII-only.** Use `-` (hyphen) not `—` (em-dash); `'` `"` (straight) not `‘ ’ “ ”` (curly); `->` not `→`; spell out `OK`/`WARN` not `✅`/`⚠`. This is the rule — the comments matter less than the string literals, but make the whole file ASCII for safety.

2. **Detect before committing** — grep the script for non-ASCII:
   ```bash
   grep -nP '[^\x00-\x7F]' scripts/brehon/<name>.ps1   # any hit = fix it
   ```
   A clean grep is the gate. Add it to any PowerShell-script authoring checklist.

3. **If you must emit a non-ASCII char to the console** (rare), don't put it in the source literal — build it from a codepoint at runtime (`[char]0x2014` for em-dash) and ensure `$OutputEncoding` / the console codepage is UTF-8. Almost never worth it; prefer ASCII output.

4. **The Write tool emits UTF-8.** When you author a `.ps1` via the Write tool and your prose instinct inserts an em-dash (Claude does this habitually in explanatory text), it lands as a real UTF-8 em-dash byte — which Win PS 5.1 then mis-reads. So the trap fires precisely because the authoring tool is UTF-8-correct and the *runtime* is not. Fix at author time: re-read the file's strings, or run the grep above, before first execution.

## Verify-before-trusting tie-in

This incident is also a `pattern_verify_before_trusting_shell_output` case: the broken script printed a success message while doing nothing. The em-dash was found only by verifying the actual file count (`Get-ChildItem -Recurse -File | Measure`) against the claimed success — not by trusting the script's own "updated" output. When a PowerShell script reports success, verify the side effect landed; a cp1252 parse break is one of the ways "success" can be a lie.

## Generalises beyond PowerShell: non-ASCII in API request bodies

The same ASCII-discipline applies to **any prose field you put in a JSON body sent over the wire** — not just `.ps1` literals. On 2026-06-13 a `curl -d '{"deny_reason":"... — denied ..."}'` against the Tuwunel Matrix API failed with `{"error":"unknown","message":"Json deserialize error: invalid unicode code point at line 1 column 101"}` — the em-dash (`—`) in the prose `deny_reason` broke the server's JSON deserializer. Retrying with an ASCII hyphen (`-`) fixed it immediately. This is platform-independent (it was a Linux server rejecting the body), but it bites hardest when authoring via the Write/Bash tools, which emit UTF-8 prose by reflex (Claude inserts em-dashes habitually in explanatory text — the same author-time reflex that causes the `.ps1` parse break).

**How to apply:** when an API request body carries a free-text/prose field (`reason`, `answer`, `description`, commit-message-shaped strings), keep it ASCII — `-` not `—`, straight quotes not curly, no `→`/`§`/emoji — OR build the JSON with a tool that escapes Unicode correctly (`python -c "json.dumps(..., ensure_ascii=True)"`, `jq -n`) rather than hand-writing the literal in a shell `-d` argument. The `grep -nP '[^\x00-\x7F]'` gate works on the request body too. See the `json_dump_ensure_ascii_false` PMD pattern (the inverse: when you DO want to preserve non-ASCII, use a JSON serializer, never a raw shell literal).

## See also

- `feedback_python_utf8_encoding_windows.md` — the data-side sibling (cp1252 corrupts content on write-back; this lesson is the code-side analogue where cp1252 breaks the parse).
- `json_dump_ensure_ascii_false` (PMD pattern) — JSON serialisation handling of non-ASCII; the right tool when prose fields must carry Unicode (use a serializer, never a hand-written shell literal).
- `pattern_cross_platform_divergences` (PMD) — Windows ≠ Mac ≠ Linux; cp1252 default codepage is one of the canonical divergences.
- `pattern_verify_before_trusting_shell_output` (PMD) — the script's success message was false; only file-count verification exposed the no-op.
- `feedback_batch_goto_eof_clobbers_errorlevel.md` — sibling Windows-shell trap (the `.bat` equivalent of a script lying about success).
