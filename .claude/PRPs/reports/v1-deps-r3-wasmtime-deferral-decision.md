# v1 close-out Phase 5 — wasmtime (deps-r3) deferral decision

**Date:** 2026-06-04
**Decision owner:** user (ADR-012 risk gate; surfaced, not auto-picked)
**Decision:** **Keep deferred** + document rationale + dismiss the open Dependabot
alerts + set a `/schedule` watch on **extism releases** (not the stale PR the plan
named). No code change. No ADR-012 risk taken.
**Lane:** `phase-v1-closeout` (laptop-local, M1-isolation-safe — touches only
`.claude/` + `docs/`, zero `crates/`).

---

## TL;DR

The 12 open wasmtime-family Dependabot alerts are kept deferred for v1. The two
**critical** alerts are sandbox-escapes scoped to **aarch64**, which Brehon does
**not** deploy (the deploy target is x86_64; aarch64 is explicitly unsupported in
`docker/docker-compose.yml`). The remaining 10 are **medium/low** and all require
**attacker-controlled WASM bytecode** to trigger — but in Brehon the WASM guests
are **operator-authored governance plugins** (ADR-012 Extism host), not
attacker-supplied modules. Practical exposure is therefore low. Closing the alerts
would require **forking extism** (it pins wasmtime 41.0.4 via internal
wiggle-macro APIs, so a bare `[patch.crates-io]` override will not compile) — a
1–3 day, high-ADR-012-risk change not justified by the exposure. Revisit when an
upstream extism release adopts wasmtime ≥ 42.

---

## Ground-truth verification (corrects two stale plan premises)

The close-out plan's Phase 5 section carried two assertions that were **wrong** on
verification (2026-06-04). Recording the corrections so future sessions don't
re-inherit them:

### Correction 1 — NOT "13 aarch64-only criticals"

Plan §250-254 said *"the 13 wasmtime alerts are aarch64-only criticals."* Live
ground-truth (`gh api repos/barrie-cork/lemmy/dependabot/alerts`, 2026-06-04):

- **12** open wasmtime-family alerts (not 13 — Dependabot re-counted; the Phase-4
  watchpoint anticipated this).
- **2 critical** — both genuinely **aarch64-only** sandbox escapes:
  - `GHSA-xx5w-cvp6-jv83` (alert #46) — Winch backend on aarch64, sandbox-escaping memory access.
  - `GHSA-jhxm-h53p-jm7w` (alert #43) — Cranelift on aarch64, miscompiled guest heap access → sandbox escape.
- **8 medium + 2 low** — these are **NOT aarch64-scoped**. They are arch-neutral
  or x86_64. Notably `GHSA-qqfj-4vcm-26hv` (alert #40) is explicitly **x86-64**
  (`f64x2.splat` segfault / out-of-sandbox load). Others are component-model
  string transcoding OOB read/write, table-allocation panic, Winch-backend
  table-op panics, pooling-allocator data leakage, `flags`-lifting panic.

So the honest picture is: **the two scary sandbox-escape criticals are off our
deploy arch; there is some x86_64/arch-neutral exposure, but all of it is
medium/low.**

### Correction 2 — the watch target "PR #847" is dead and wrong

Plan §254/§259 said *"tracking `extism/extism` PR #847, open since 2026-04-07"* and
recommended a `/schedule` watch on it. Verified:

- **PR #847 is CLOSED** (closed 2025-05-19 by @zshipko, branch deleted).
- It bumped **wasi-common/wiggle to 31.0.0** — *older* than current, wrong direction.
- extism **already shipped its wasmtime-41 upgrade** in **v1.21.0** (release notes:
  "Upgrade wasmtime to v41", via PR #897 by @milesj). v1.21.0 is **exactly what
  `Cargo.lock` pins** — we are already on the newest extism, which already did its
  latest wasmtime bump (to 41.0.4, the version carrying these alerts).
- **v1.21.0 (published 2026-03-26) is the newest extism crate version.** There is
  **no live upstream PR** bumping extism to wasmtime ≥ 42 right now.

**Implication:** "watch PR #847" is not actionable — there is nothing live to watch.
The correct watch target is **the extism repository's next release** (a wasmtime
≥ 42 bump will arrive as a new extism release, or as a new — not-yet-existing —
upstream PR). The `/schedule` watch is set on releases accordingly.

---

## Why keep-deferred is defensible (the exploitability argument)

Severity counts overstate the risk here. Every one of the 12 alerts is a
*guest-WASM-escaping-the-sandbox* or *malformed-WASM-crashing/leaking-the-host*
class bug. Triggering any of them requires **control over the WASM module being
executed**.

In Brehon's architecture (ADR-012), the WASM runtime (Extism) executes **governance
plugins authored and loaded by the instance operator** — it is **not** a
public-facing runtime executing untrusted, attacker-supplied modules. An attacker
would already need to control plugin bytecode (i.e. already have operator-level
plugin-deployment access) to reach any of these code paths. That is a fundamentally
lower-exposure threat model than a multi-tenant or public WASM execution service.

Combined with the two true criticals being **off our deploy architecture**, the
residual real-world risk for a v1 (pre-pilot, private, x86_64, operator-plugin-only)
deployment is **low**.

## Why not fork-and-patch now

- extism 1.21.0 pins wasmtime **41.0.4** through internal **wiggle-macro** APIs that
  are version-coupled. A bare `[patch.crates-io]` override to wasmtime ≥ 42 **will
  not compile** — the macro-generated glue is tied to the 41.x ABI.
- Closing the alerts therefore means **forking extism itself** and carrying a
  wasmtime bump + any wiggle/wasi API adaptation. Estimated 1–3 days, and extism is
  **the** governance plugin host (ADR-012) — a fork is high-risk surface to own
  through pilot.
- The exposure (above) does not justify that cost for v1. Re-evaluate at the M-series
  milestones or the moment upstream extism ships wasmtime ≥ 42.

---

## Actions taken (this decision)

1. **This decision record** — durable rationale + the two ground-truth corrections.
2. **`kind: "log"` DQ entry** (Phase 4-T4 handoff) — audit-trail anchor pointing here.
3. **`/schedule` watch on extism releases** — fires periodically to check
   `crates.io` / `github.com/extism/extism/releases` for a release carrying
   wasmtime ≥ 42; when one lands, adopt it (bump extism, which transitively bumps
   wasmtime, which closes the alerts) instead of forking.
4. **Dependabot dismissals — DONE 2026-06-04** (via `gh api PATCH .../dependabot/alerts/<N>`,
   `dismissed_reason: tolerable_risk`). All 12 wasmtime alerts verified dismissed
   (0 wasmtime open post-dismissal). Live open-alert count: 18 → 7 (the 7 remaining
   are NOT wasmtime — see below). Note: GitHub's `dismissed_comment` caps at **280
   chars**; the 257-char comment actually used is below.

## Dependabot dismissal — DONE (audit record)

The 12 wasmtime alerts (#37–#47 + #60) were dismissed 2026-06-04 as
`tolerable_risk` with this comment (257 chars — GitHub caps `dismissed_comment` at 280):
> Deferred (deps-r3): extism 1.21.0 pins wasmtime 41.0.4 via wiggle-macros; bare
> patch won't compile. WASM guests are operator-authored governance plugins
> (ADR-012). Criticals aarch64-only; we deploy x86_64. See report
> v1-deps-r3-wasmtime-deferral-decision.md

**NOT dismissed (the 7 still open are deliberate):** astral-tokio-tar ×3 (dev-only,
via testcontainers — user chose to skip dismissal + bump post-M1 instead) + rustls-webpki
×3 + idna ×1 (Phase 4-T1/T2 *fixes*, M1-gated — dismissing the rustls-webpki HIGH #55
would hide a real fixable alert). Per the Phase 4-T3 correction: astral-tokio-tar 0.6.0
is NOT the fix release (the plan's premise was inverted) — 0.6.1/0.6.2 are the fixes;
we pin the vulnerable 0.6.0, so "dispute" was wrong. It's dev-only so exposure is nil,
and the right remedy is a testcontainers bump when M1 frees the lockfile.

| Alert # | GHSA | Severity | Arch scope |
|---|---|---|---|
| 46 | GHSA-xx5w-cvp6-jv83 | critical | aarch64 only (off-target) |
| 43 | GHSA-jhxm-h53p-jm7w | critical | aarch64 only (off-target) |
| 60 | GHSA-p8xm-42r7-89xg | medium | arch-neutral (table alloc panic) |
| 47 | GHSA-f984-pcp8-v2p7 | medium | Winch backend |
| 45 | GHSA-394w-hwhg-8vgm | medium | arch-neutral (component string transcode OOB write) |
| 42 | GHSA-q49f-xg75-m9xw | medium | Winch backend (table.fill panic) |
| 40 | GHSA-qqfj-4vcm-26hv | medium | **x86-64** (f64x2.splat) |
| 39 | GHSA-m758-wjhj-p3jq | medium | arch-neutral (flags lifting panic) |
| 38 | GHSA-jxhv-7h78-9775 | medium | arch-neutral (utf-16 transcode panic) |
| 37 | GHSA-hx6p-xpx3-jvvv | medium | arch-neutral (utf-16→latin1 OOB read) |
| 44 | GHSA-6wgr-89rj-399p | low | pooling-allocator data leakage |
| 41 | GHSA-m9w2-8782-2946 | low | Winch (64-bit table data leakage) |

(12 alerts; counts verified live 2026-06-04. The earlier "13" in the plan was a
stale Dependabot count.)

---

## Revisit trigger

Adopt wasmtime ≥ 42 when **upstream extism publishes a release that carries it**
(monitored by the `/schedule` watch). At that point: bump extism in `Cargo.toml`/
`Cargo.lock`, run the laptop cargo+e2e gate + Docker `cargo-linux.sh` compile proof,
confirm the 12 alerts clear, and remove the Dependabot dismissals. Forking extism
remains the fallback only if upstream stalls past pilot AND the threat model changes
(e.g. Brehon begins executing non-operator WASM).
