---
name: A time-windowed hook's window is either a look-back or an age-gate — classify before touching logic
description: When a user reports a time-gated hook "fires too early/often", the config minute-value is ambiguous — it can be a LOOK-BACK window (how far back to search for evidence) or an AGE-GATE (how long before enforcement starts). They are config-identical but semantically opposite. Classify which before changing logic, or you mis-diagnose the trigger.
type: feedback
---

# Time-windowed hook: look-back vs age-gate are config-identical, opposite in meaning

## TL;DR

A hook with a `WINDOW_MINUTES=60` (or any `--since`, `-mmin`, `created_at >= now - N`
style window) is ambiguous in a way that bites diagnosis. The same `60` can mean
two opposite things:

- **Look-back window:** "search the last 60 minutes for evidence that condition X
  was satisfied." Absence of evidence in the window → enforce.
- **Age-gate:** "wait until this thing is 60 minutes old before enforcing at all."
  Too young → never enforce.

These are **config-identical** (both are just a 60-minute interval) but produce
**opposite behaviour** on a fresh/young subject. When a user says a time-gated
hook "fires too early" or "fires within seconds of start", the bug is almost
always that a look-back window is being *mistaken for* an age-gate — the code
has no age concept at all; it just looks like it does because the look-back
happens to match the number the user had in mind.

**Rule: before touching logic, classify the window. Read the actual query/test
the window feeds.** Does the window bound a search for *prior evidence* (look-back)
or a wait *before first enforcement* (age-gate)? Get that wrong and you'll "fix"
the wrong dimension.

## The incident (2026-06-11, `retro-check.sh`)

The retro-check Stop hook nagged a brand-new CC session for a post-task retro
within seconds of session start. The user's mental model: "enforce only after
60 min of session." The code's `WINDOW_MINUTES=60`: "look back 60 min in the PMD
for an existing `Task retro:` / `Session retro:` row; if none, block the Stop."

The hook had **zero notion of session age**. A fresh session has no retro yet, so
the 60-min look-back found nothing and the hook blocked on the first turn-end.
The `60` the user expected (age) and the `60` in the code (look-back) were
unrelated — which is exactly why the bug was invisible to anyone scanning the
config value.

The fix added a *separate, real* age-gate (stamp first-seen epoch per session,
fail open until ≥60 min old) and left the look-back window intact — because the
look-back was correct *for sessions old enough to be enforced*. Two distinct
60-minute concepts, both legitimate, doing different jobs. Conflating them would
have either disabled enforcement entirely or left the false-positive in place.

## How to apply

When a user reports "this time-gated check fires too early / too often / on a
fresh thing":

1. **Find the window's consumer.** Grep for the minute variable; read the
   query/condition it feeds (`SELECT … WHERE created_at >= datetime('now', '-N')`,
   `git log --since`, `find -mmin`, `gh run list --created`, etc).
2. **Classify:**
   - Window bounds a search for *prior evidence* → **look-back**. A young subject
     legitimately has no evidence yet; if that triggers enforcement, you need a
     *separate* age-gate, not a bigger window.
   - Window bounds a *wait before first enforcement* → **age-gate**. Firing too
     early means the age computation is wrong (unstable key, missing marker, reset
     each invocation).
3. **Don't widen the look-back to fix an age problem.** Widening the look-back to
   "fix" a fresh-subject false-positive only delays the nag; it doesn't add the
   age concept. The correct fix for a fresh-subject false-positive on a look-back
   hook is to *add* an age-gate that fails open while young.

## Generalises to

Any periodic/threshold check where a single interval value is doing
double-duty in the reader's mind: rate limiters (window vs cooldown), staleness
checks (TTL vs grace period), retry backoffs (look-back for prior attempts vs
minimum-age-before-retry), cron freshness gates. The failure mode is always the
same — the config value is ambiguous, and the two readings diverge precisely on
young/fresh subjects.

## Symptom to recognise

A complaint of the form "X fires immediately / within seconds / on a brand-new
Y" against a check that has a time window in its config. That phrasing is the
tell: the subject is too *new* to have produced the evidence the look-back is
searching for — and the check has no age concept to suppress the fire. Reach for
"add an age-gate," not "widen the window." Related discipline:
`feedback_runbook_audit_drift_post_event_check.md` ("every claim is a
hypothesis" — the config's apparent meaning is a hypothesis until you read its
consumer).
