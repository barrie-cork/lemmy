---
name: Windows Bash↔Python `/tmp` + git-show + UTF-8 traps (consolidated)
description: Three traps that compound on Windows when Bash invokes Python on git-show output via /tmp. Use the canonical recipe.
type: feedback
originSessionId: 15a698a1-d5bc-4990-b8cf-24f77e301839
---
When the advisor session needs to read a file out of a git ref (e.g.
inspecting `decision-queue.json` on a worker branch) and parse it as JSON
via Python, three Windows-specific footguns compound. They have all been
hit individually before; this is the consolidated lesson + the canonical
recipe that side-steps all three at once.

## The three traps

1. **`git show <ref>:<path>` argument mangling.** When the `<ref>` is a
   long branch name containing slashes (typical Junior worktree branch:
   `junior/role-impl-task-...-md-154`), Bash inside Claude Code on
   Windows runs through the PowerShell tool layer underneath. PowerShell
   normalises the colon in the `git show <ref>:<path>` argument and
   replaces it with a semicolon — so git sees
   `origin\junior\role-...-md-154;.claude\decision-queue.json` and
   errors with `fatal: ambiguous argument`. Already documented at
   `feedback_cmd_c_redirect_exit_code_capture.md` and
   `feedback_python_utf8_encoding_windows.md` for adjacent cases, but
   the `git show` form specifically is its own reproducer.

2. **`/tmp` Bash↔Python path divergence.** Git Bash on Windows mounts
   `/tmp` to `$TEMP` (typically `C:\Users\<user>\AppData\Local\Temp`).
   When Bash writes to `/tmp/foo.json`, the file is real on disk at
   the Windows path. But when Python is invoked from Bash and is given
   the literal string `/tmp/foo.json`, Python's `os.path` resolution
   does NOT consult MSYS mounts — it looks for `\tmp\foo.json` from
   the current working drive's root, which doesn't exist. So the
   sequence "Bash creates `/tmp/foo`, Python reads `/tmp/foo`" fails
   silently with `FileNotFoundError`. Already documented at
   `feedback_windows_tmp_path_unreliable.md` (the cross-cutting form);
   this lesson re-emphasises it specifically for the Bash→Python
   handoff.

3. **Python 3.14 default codec on Windows is cp1252.** When Bash pipes
   bytes to `python -c "..."` via stdin, Python opens stdin with
   `cp1252` by default. `decision-queue.json` contains UTF-8
   characters (em-dashes in DQ questions, slashes in branch names with
   non-ASCII byte sequences from cargo error messages); cp1252 chokes
   immediately on first non-ASCII byte. Already documented at
   `feedback_python_utf8_encoding_windows.md`. Symptom:
   `UnicodeDecodeError: 'charmap' codec can't decode byte 0xe2`.

## Canonical recipe (sidesteps all three)

**The helper script is the FIRST move, not the recovery.** When you need to read
a file out of a slash-containing ref (any `phase-*`, `junior/*`, `origin/<branch>`
with a `/`), reach for `scripts/brehon/git-show-json.sh <ref> <path>` (it works on
any file, not just JSON — it resolves the ref to a SHA first, writes to a
Windows-explicit temp path, leaves it for Python with `encoding='utf-8'`) — or
`scripts/brehon/resolve-dq-canonical.sh <phase>` for DQ specifically — **before**
attempting a raw `git show <ref>:<path>`. The raw form silently mangles
colon→semicolon and `/`→`\` and returns a short `fatal: ambiguous argument` error
that reads like file content (a 3-line "file" is the tell). Hitting the mangle
then recovering with the helper wastes ~2 min every time; the helper-first reflex
makes the recovery unnecessary. (Re-hit 2026-06-12 m2-late-2: attempted raw
`git show origin/phase-m2-late-2:.claude/...` first, got the mangled 3-line error,
then reached for the helper. The helper should have been the first call.)

For any "read JSON out of a git ref and process it in Python" pattern:

```bash
# Step 1 — git show via SHORT SHA, never branch-name-with-colon-suffix.
# Capture to a Windows-explicit temp file path (NOT /tmp).
SHA=$(git -C C:/Users/barri/Developer/brehon-fork rev-parse origin/<branch>)
git -C C:/Users/barri/Developer/brehon-fork show ${SHA}:.claude/decision-queue.json \
  > "$LOCALAPPDATA/Temp/dq.json"

# Step 2 — Python reads with explicit encoding='utf-8'. Use raw string for
# the path; never rely on /tmp.
python -c "
import io, json
d = json.load(io.open(r'C:\\Users\\barri\\AppData\\Local\\Temp\\dq.json', encoding='utf-8'))
# ... your processing ...
"
```

Key rules:

- **Never put a `<ref>:<path>` argument on a `git show` command** when
  the ref has slashes. Capture the SHA first, then `git show <SHA>:<path>`.
  The SHA itself contains no characters PowerShell rewrites.
- **Never write to `/tmp` from Bash and expect Python to read it.**
  Use `$LOCALAPPDATA/Temp/<name>` from Bash side — that resolves to
  the same Windows directory both Bash and Python agree on. Or use a
  path under the repo (gitignored) like `.claude/scratch/<name>.json`.
- **Always pass `encoding='utf-8'` to Python file open** when reading
  any file the project authored (DQ JSON, runlog Markdown, plan
  Markdown, briefs). Default cp1252 will fail on the first em-dash.
- **Prefer reading from a file over stdin pipe.** Even if you write
  the file yourself moments earlier, the file path is interpretable
  by both Bash and Python; a stdin pipe forces Python's default
  codec.

## Why this consolidated lesson exists

The three traps were each documented separately. But they HIT TOGETHER
in the same single tick of the advisor's polling loop on 2026-05-09
(during v1-SL-c-2 fix-impl-1 verification). Each one cost one Bash
roundtrip + one debugging probe; total cost ~3-4 wasted tool
invocations, ~100 lines of error output in context. The PMD-search
pre-queue pattern won't surface three independent lessons as relevant
to a single inline script — only a consolidated recipe does. Cite this
file when reaching for `git show <ref>:<file>` + Python in one Bash
block.

## Trap 4 (adjacent): `git rev-parse --short <refA> <refB>` multi-ref mangling

`git rev-parse --short governance-v0 origin/governance-v0` (two refs
in one invocation) is **also** mangled by the PowerShell tool layer
on Windows — the two ref arguments get concatenated/normalised into a
single bad token and git aborts with `fatal: Needed a single
revision`. This is the same PowerShell-arg-rewrite family as trap 1
(`git show <ref>:<path>`), but the reproducer is multi-arg
`rev-parse`, not the colon.

Confirmed 2026-05-16 (v1-AD-e bm-cut recovery session): the
single-ref forms `git rev-parse governance-v0` and `git rev-parse
origin/governance-v0` each worked fine individually; only the
two-ref-in-one-call form failed. It aborted mid-`&&`-chain, killing
a commit that would otherwise have run.

**Fix:** never pass multiple refs to one `git rev-parse` on Windows.
Issue separate single-ref invocations:

```bash
# WRONG (mangled on Windows):
git rev-parse --short governance-v0 origin/governance-v0
# RIGHT:
git rev-parse --short governance-v0
git rev-parse --short origin/governance-v0
```

Especially important: do NOT put a multi-ref `rev-parse` as a
pre-flight check at the head of a `&&` chain whose tail is a commit
or push — the mangled abort silently skips the state-changing tail.
Either split the rev-parse out as its own command, or drop it from
the chain entirely.

## Symptom to recognise

- `fatal: ambiguous argument 'origin\<branch>;.claude\<file>'` →
  trap 1, switch to short SHA.
- `FileNotFoundError: '/tmp/<file>'` after Bash `ls -la /tmp/<file>`
  shows it exists → trap 2, switch path to `$LOCALAPPDATA/Temp/<file>`.
- `UnicodeDecodeError: 'charmap' codec can't decode byte 0x__` →
  trap 3, add `encoding='utf-8'` and read from file not stdin.
- `fatal: Needed a single revision` from a `git rev-parse <refA>
  <refB>` (two refs) → trap 4, split into separate single-ref
  invocations; check it wasn't heading a `&&` chain with a
  commit/push tail.

## Generalises to

Any project script that mixes Bash + Python on Windows, particularly
on Claude Code's Bash tool which uses PowerShell underneath. The same
patterns apply to ralph loops, /bm-* verbs that inspect cross-branch
state, retro-authoring scripts that diff plan files vs commit history,
and any sub-agent dispatch that pre-validates work via JSON inspection.

The recipe scales: substitute `git show <SHA>:` with `gh api` or any
other byte-emitting CLI; the file-path + encoding rules carry over
unchanged.

## When to skip

- Bash-only (no Python). Exit-code propagation rules from
  `cargo-output-capture.md` apply but not these three.
- Python-only (no Bash). Just remember `encoding='utf-8'`.
- Linux runtime (Junior daemon on EliteDesk). `/tmp` works, refs with
  colons work, Python 3 default UTF-8 — none of these traps reproduce.
  This lesson is **Windows advisor session-specific**.
