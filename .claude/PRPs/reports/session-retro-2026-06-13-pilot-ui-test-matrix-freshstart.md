# Session retro — 2026-06-13 — pilot-ui-test-matrix-freshstart

**Harness:** claude-code
**Session window:** ~2026-06-13T20:00Z → ~2026-06-13T22:10Z (~130 min)
**Branch at start:** `200055446` (`governance-v0`)
**Branch at end:** `f02808f52` (`governance-v0`)
**Files touched:** 2 committed (`docker/docker-compose.yml`, new viewer handover) + Docker volume/container state (uncommitted, off-repo)
**Commits:** 2 (auto: 0, explicit: 2) — `a05f6fac4` Eruda disable, `f02808f52` viewer handover. (`200055446` LEMMY_UI_BACKEND landed pre-window.)

## TL;DR

Phase-8 pilot session with three threads: (1) Chrome-driven UI verification of the
pilot Lemmy instance — browser-side API resolution, admin login, and the full
register→pending→admin-queue→deny pipeline all confirmed green; (2) found and fixed
a go-live blocker — `LEMMY_UI_ERUDA=true` injected a full-screen dev-console overlay
reserving ~678px at the top, pushing the real UI down for every tester; (3) a Matrix
`server_name` migration (`localhost` → `matrix.agentgrey.app`, Cloudflare-Tunnel
exposed) forced a fresh-start wipe of both Matrix volumes, which I executed behind two
confirmation gates while preserving Lemmy Postgres. Most load-bearing finding: **Matrix
`server_name` is bound to its data volume — changing it REQUIRES a wipe, no in-place
rename.** Top change proposal: promote that + the puppet-room viewing model to a lesson,
because the "view a juror room" request will recur and the non-obvious facts (puppet
rooms, server_name binding, registration-off-after-exposure) cost real diagnosis time.

---

## What surprised us

- **The Eruda overlay was invisible to the accessibility tree but dominated the screenshot.**
  `read_page`/`find` reported a clean Lemmy UI; the screenshot showed a full-screen dev
  console. Eruda renders in a fixed/shadow overlay the a11y tree doesn't surface.
  Diagnosing "blank-looking screenshot but DOM has 40 posts" needed computed-style +
  `getBoundingClientRect` inspection, not the a11y tree. ~10 min of confusion before the
  cause (678px top spacer) was isolated.
- **Juror "rooms" are bridge AS-puppet Matrix rooms, not anything a Lemmy login can reach.**
  The user asked "can I view the juror room?" — the honest answer required discovering that
  members are `@_brehon_*` puppets (no passwords), and the `juror1-5` accounts are Lemmy
  Postgres rows, not Matrix accounts. The viewing path is "real Matrix account + Element +
  join public room," which is a meaningfully different setup than expected.
- **Jury rooms are `join_rule:public` + `history_visibility:shared`** — pleasant surprise:
  a viewer can join by alias and read full back-history with no invite/puppet wrangling.
  Made the viewer plan much simpler than feared.
- **`server_name=localhost` only breaks client auto-discovery + federation, not direct-URL
  connection.** A register call over the LAN IP succeeded and returned a usable token even
  while server_name was `localhost`. (Mooted by the migration to a real domain, but it
  reframed the whole "is this even viewable remotely" question mid-session.)
- **Registration silently flipped OFF after the migration.** The config I'd read earlier had
  `allow_registration = true`; post-wipe the live server returned `M_FORBIDDEN: Registration
  disabled`. The other lane had tightened it for the public-exposed server — correct, but an
  unannounced state change that the viewer-account plan has to route around.
- **The registration-applications API returns `items[]`, not `registration_applications[]`.**
  My first parser keyed the wrong field and falsely reported "0 pending" — twice — before
  raw-response inspection caught it. Nearly mis-reported a working pipeline as broken.

## What to change

| # | Change | Expected effect | Cost | Recurrence |
|---|---|---|---|---|
| 1 | Promote a lesson `feedback_matrix_server_name_volume_binding.md` (Tuwunel refuses to reuse a DB under a changed server_name; changing it = mandatory volume wipe; juror rooms are AS-puppet rooms not Lemmy logins; public+shared so joinable by a viewer account) | Next "view a juror room" / "change Matrix domain" request skips ~30 min of re-derivation | minor (write 1 file) | 1× here + recurs whenever Matrix topology changes |
| 2 | Add a guard/lesson note: **inspect raw API response shape before trusting a parsed count** when verifying a pipeline (the `items[]` vs `registration_applications[]` miss). Fold into existing `feedback_verify_before_trusting_shell_output.md` rather than a new file | Stops false "0 results → feature broken" conclusions on JSON-shaped APIs | minor (edit existing lesson) | 2× this session (two false-0 reports) |
| 3 | Capture an **ASCII-only-in-shell-JSON** reminder in the deny path: the em-dash in `deny_reason` broke Tuwunel's deserializer (`invalid unicode code point at line 1 column 101`). Cross-link from the existing `feedback_powershell_ascii_only_code_strings.md` to "also applies to curl `-d` JSON bodies on any platform" | Avoids the deserialize-error retry loop on API calls with prose fields | trivial (one cross-link line) | 1× here + 1× prior (PowerShell ASCII lesson exists) → threshold met |
| 4 | When a multi-lane handover changes shared infra state (here: `server_name`, `allow_registration`), the changing lane should leave a one-line note in `pilot-internal-SHARED-STATE.md`. The registration-off flip surprised this session because it was undocumented cross-lane | Cross-lane infra state changes become discoverable without probing live config | minor (process note, already partly covered by SHARED-STATE discipline) | 1× here |

## What to carry forward

- **Confirm-before-destroy with scoped AskUserQuestion gates.** Two gates before the Matrix
  wipe (scope: Matrix-only vs full-reset; then bridge-DB inclusion) prevented any chance of
  wiping Lemmy Postgres. The "wipe" ask was genuinely ambiguous; surfacing the two-volume
  distinction was the right call. Used cleanly twice.
- **`--no-deps` for every scoped container recreate.** `lemmy-ui` recreate for the Eruda fix
  and the proposed `element-web` deploy both honor the "don't restart lemmy/bridge/tuwunel
  without coordination" constraint. The Matrix fresh-start restart was the one user-directed
  exception, explicitly flagged.
- **Verify the fix at the layer the bug lives.** The Eruda fix wasn't trusted on the env-var
  alone — verified container env (`docker exec … env`), then DOM (`window.eruda` undefined),
  then first-paint screenshot. Same for LEMMY_UI_BACKEND (network request 200 + console-clean
  + HTML `lemmyBackend` string). Caught that container-env-correct ≠ browser-correct.
- **Authoritative test over inference for reachability.** Tunnel "confirmed" = an actual
  `curl https://matrix.agentgrey.app/_matrix/client/versions → 200 TLS-valid`, not "cloudflared
  process is running." The live request is the proof; the process check is a hint.
- **Secrets stay out of artifacts.** The cloudflared tunnel token surfaced in a process probe;
  it was deliberately excluded from the handover doc. Keep that reflex.

---

## Three-signal scoring

| Skill / Agent / Command | Saved (min) | Wasted (min) | Surprise | Notes |
|---|---:|---:|---|---|
| Chrome browser automation (computer/navigate/find/form_input/network) | 25 | 10 | medium | Drove full UI verification a human would do by hand; ~10 wasted on the Eruda a11y-vs-screenshot confusion |
| `javascript_tool` (in-page diagnosis) | 15 | 0 | low | Computed-style + bounding-rect inspection cracked the Eruda layout mystery the a11y tree couldn't |
| Bash (curl API verification, SSH Docker ops) | 20 | 5 | low | API-level confirmation of register/login/deny pipeline; 5 wasted on the `items[]` parser miss + the em-dash deserialize error |
| AskUserQuestion (×4 gates) | 8 | 0 | none | Clean forks: approve-method, wipe-scope, bridge-DB, viewer-approach. Prevented a destructive over-wipe |
| Volume-wipe + fresh-start sequence | 12 | 0 | medium | Tuwunel `Cannot reuse` resolved cleanly; surprise = registration silently OFF afterward |
| Handover doc authoring | 10 | 0 | none | Self-contained resume brief; sibling-pattern matched |
| `memory_write_eval` (retro at Stop hook) | — | 3 | low | Hook fired 3× before retro written — minor friction of the destructive-ops pivot displacing the eval |

## Complexity scores (heavy tasks only)

No Junior impl-tasks ran (all advisor-direct). Heaviest unit = the Matrix fresh-start
sequence (stop → rm containers → rm 2 volumes → recreate → verify), advisor-driven over
SSH. Approx `0 files / 0 commits / ~12 min / ~8 min` (the 8-min gap = waiting on container
init + boot verification). Well within envelope; no watchdog relevance (no Junior worker).

## Decisions to revisit

- **`.well-known/matrix/client` returns M_NOT_FOUND** — Element auto-discovery from a bare
  MXID fails; manual HS-URL entry works. Worth serving the well-known file for a clean tester
  experience. Noted in the viewer handover; not yet a lesson.
- **Viewer-account creation needs `allow_registration` temp-flip** since the public server has
  reg off. Acceptable for one account but worth a cleaner admin-account path if more Matrix
  accounts get created during the pilot.

---

## Promotion candidates (recurrence ≥ 2 in this session, or ≥ 1 here + ≥ 1 in prior memory)

- [ ] Change #1: promote to `.claude/lessons/feedback_matrix_server_name_volume_binding.md` (cross-harness lesson)
- [x] Change #2: **DONE** — written as new lesson `.claude/lessons/feedback_inspect_raw_api_response_shape_before_trusting_count.md` (the `feedback_verify_before_trusting_shell_output` target turned out to be PMD-only, no disk file; authored a standalone lesson cross-linked to that PMD pattern). PMD copy = entry 999 (hook-synced); manual dup 1001 demoted.
- [x] Change #3: **DONE** — added "non-ASCII in API request bodies" section + See-also to `.claude/lessons/feedback_powershell_ascii_only_code_strings.md` (em-dash in curl `-d` JSON broke Tuwunel's deserializer).
- [ ] Change #4: process note in `pilot-internal-SHARED-STATE.md` discipline (cross-lane infra-state changes leave a one-liner)
- [ ] PMD eval write: already done this session (eval ID 996 + this retro)

---

_Generated by `.claude/skills/session-retro/SKILL.md`. Lessons consulted:
`feedback_retro_not_report.md`, `feedback_four_role_retro_signals.md`,
`feedback_retro_task_complexity_score.md`._
