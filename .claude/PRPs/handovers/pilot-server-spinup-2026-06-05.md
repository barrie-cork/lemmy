# Handover — Pilot server spin-up (Brehon Lemmy fork on homeserver)

**Date:** 2026-06-05
**Session goal:** Deploy the Brehon governance Lemmy fork as a running pilot on
the homeserver (EliteDesk), via Docker, then open the admin page in Chrome
(Browser 1, non-headless, deviceId `9ffbc352-c883-4d3a-990c-0adf9d22f069`) so
the user can see it.
**Status:** PAUSED mid-flow by user; resume in a fresh session.

---

## RESUME — next concrete action

The dependency blocker is FIXED and pushed. The **only remaining work** is:

1. **(User must do first)** Restart the Junior daemon — needs interactive sudo:
   ```
   ssh homeserver
   sudo systemctl start junior@brehon-fork
   ```
   (It was stopped during this session for the build; restart left undone
   because SSH can't do interactive sudo. Non-blocking for the deploy itself.)

2. **Rebuild the server image** (the fix is on the daemon checkout already):
   ```bash
   ssh homeserver "cd /srv/brehon-fork/docker && nohup docker compose build lemmy > /tmp/brehon-build2.log 2>&1 & echo PID=\$!"
   ```
   - Cold-ish build ~20-40 min; the wasmtime cluster will now compile (was the
     blocker). Monitor `/tmp/brehon-build2.log` for `error[` and `free -h`.
   - **RAM mitigation (optional, user-approved pattern):** stopping the
     non-essential containers frees ~3.3 GB. The set is listed below under
     "Homeserver service mitigation". They are currently RUNNING again
     (restored at pause). Re-stop only if the build needs headroom; the box
     has ~8.5 GB free with everything up, which is borderline but workable.

3. **Bring the stack up + verify** (Task 4):
   ```bash
   ssh homeserver "cd /srv/brehon-fork/docker && docker compose up -d"
   ```
   - Verify: `docker compose ps` all healthy (proxy, lemmy, lemmy-ui, pictrs,
     postgres). Migrations run automatically on first `lemmy_server` boot.
   - Smoke: `curl -s http://localhost:1236/api/v4/site` (UI/proxy) and the AGPL
     endpoint `curl -s http://localhost:8536/api/v4/source` (shipped v1-ship-1).
   - **Host ports:** proxy `1236` + `8536`, postgres `5433`. All were free.

4. **Restore + open Chrome** (Tasks 5, 6):
   - Confirm all originally-running services are up (they were restored at pause).
   - Open Chrome Browser 1 (non-headless) to the running UI so the user can see
     it and log in. URL depends on how the user reaches the homeserver
     (localhost only binds on the box; via Tailscale use the homeserver's
     Tailscale name/IP, e.g. `http://homeserver:1236` or the Tailscale IP).
     ASK the user which URL works from their browser before navigating.

---

## THE BLOCKER WE HIT + FIXED (do not re-investigate — it's solved)

**Symptom:** First `docker compose build lemmy` failed after ~92s with **42
E0277 compile errors** in `extism-1.30.0/src/current_plugin.rs`:
`?` couldn't convert error: `wasmtime::Error: std::error::Error` is not satisfied.

**Root cause (confirmed via crates.io sparse index + empirical cargo):**
- `extism 1.30.0` declares `wasmtime ^43` and transitively hard-locks the whole
  wasmtime cluster to **exactly 43.0.2** (via `wasi-common 43.0.2` requiring
  `^43.0.2`) — pinning wasmtime DOWN to 43.0.1 is rejected by the resolver.
- wasmtime 43's `no_std` refactor removed the blanket `wasmtime::Error:
  std::error::Error` impl (now gated behind the `std` feature). extism 1.30.0's
  code relies on the old impl → doesn't compile.
- `extism 1.30.0` is the LATEST extism; no upstream fix release exists.
- The bad pairing was introduced by `v1-deps-r3` (#183) + finalized by
  **Dependabot group bump `966ced682`** (wasmtime 41.0.4 → 43.0.2), **never
  Linux-compile-verified** (textbook `feedback_linux_compile_proof_is_a_gate`
  violation — the governance-v0 tip did not build).

**Fix applied (user chose "revert extism to 1.21.0"):**
- `Cargo.toml`: `extism = { version = "1.21.0", ... }` (was 1.30.0).
- `cargo update -p extism --precise 1.21.0` → wasmtime cluster down to 41.0.4.
- **Validated on laptop (canonical runner):** `cargo check --workspace
  --features full` → `Finished dev profile in 6m02s`, **exit 0, 0 errors**
  (synchronous run, real exit code — a prior background run gave an unreliable
  exit-0-with-empty-log, re-run synchronously to confirm).
- **Committed `d12aed221`** on `governance-v0`, pushed
  (`563aa0355..d12aed221`). Daemon checkout fast-forwarded to `d12aed221`.

**Tradeoff (user-accepted):** re-opens the 13 wasmtime Dependabot alerts that
deps-r3 closed. Per roadmap, those criticals are **aarch64-only** and Brehon
deploys **x86_64**, so practical exposure is low. Proper fix (extism release
compatible with wasmtime 43, or a vetted fork) → **future `deps-r4`**. GitHub
showed "4 vulnerabilities (3 moderate, 1 low)" on push — expected.

---

## STATE FACTS (verified this session)

- **Laptop** `C:/Users/barri/Developer/brehon-fork`: branch `governance-v0`,
  HEAD `d12aed221` (the fix). Working tree clean except pre-existing untracked
  `.claude/PRPs/debug/v1-deps-r2-dq-linux-fragment.json` (NOT mine).
- **Homeserver** `/srv/brehon-fork`: branch `governance-v0`, HEAD `d12aed221`
  (fast-forwarded). `docker/plugins/` dir created (empty; fixes the
  `./plugins` compose mount — no `.wasm` plugins exist yet, which is fine for
  first boot; governance hooks just won't fire).
- **Docker:** v29.3.0, Compose v5.1.0. Stack = `docker/docker-compose.yml`
  (proxy/lemmy/lemmy-ui/pictrs/postgres). Full source build via cargo-chef
  (`docker/Dockerfile`, RUST_RELEASE_MODE=debug). Config defaults kept:
  admin `lemmy` / pw `lemmylemmy` / site `lemmy-dev` / hostname `localhost`
  (in `docker/lemmy.hjson`; `setup:` block seeds admin ONCE on fresh PG volume).
- **Host ports for Brehon:** 1236, 8536 (proxy), 5433 (postgres) — all free.
- **No dangling Docker images** from the failed build (verified).

## Homeserver service mitigation (RAM)

Stopped during build, **restored at pause** (all running again). If re-stopping
for the rebuild:
```bash
docker stop web-archive-opensearch web-archive-api web-archive-frontend \
  agent-grey-agent-grey-1 agent-grey-agent-grey-worker-1 agent-grey-agent-grey-beat-1 \
  docker-n8n-1 docker-jellyfin-1 docker-glin-1 wg2-qa-wg2-qa-1 docker-uptime-kuma-1
```
Frees ~3.3 GB (→ ~13 GB free). **Keep running:** pihole (DNS — do NOT stop),
postgres/redis (infra), ollama (PMD/Junior), portainer, brehon-jmd-pg (idle).
Restore with `docker start <same list>`.

## Gotchas hit this session (avoid re-tripping)

- `-p lemmy_server --features full` → ERROR "does not contain this feature"
  (returns spurious exit 0). Use `--workspace --features full`. (MEMORY-known.)
- Background `cargo ... &` via the Bash tool gave an empty log + unreliable
  exit-0 notification. **Run the validation gate synchronously** to trust it.
- The `.claude/hooks/check-cargo-pipe.sh` hook blocks piping cargo through
  grep/tail/head in the SAME command. Capture to a file in one call, read in a
  separate call.
- Junior daemon stop/start needs interactive sudo — SSH can't do it. User runs
  `sudo systemctl start junior@brehon-fork`. (A `systemctl stop` attempt this
  session SIGTERM'd it into `failed` state anyway; harmless, but restart needed.)

## Task list at pause

1. ✅ Pause daemon + stop non-essential services
2. ✅ Create plugins/ dir + pre-flight compose config
3. ✅ Build (first attempt failed → fixed; **rebuild NOT yet run**)
4. ⏳ Bring stack up + verify health
5. 🔄 Restore services (containers DONE; **daemon restart pending — user sudo**)
6. ⏳ Open Brehon admin page in Chrome
7. ✅ Commit + push extism revert (`d12aed221`)
