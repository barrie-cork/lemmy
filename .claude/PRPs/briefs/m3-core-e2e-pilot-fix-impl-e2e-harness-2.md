# Brief: m3-core-e2e-pilot fix-impl #2 — rust-s3 builder API + LiveKit --bind + DB-perms bind-mount

## §1 Role + dispatch line

`[role:impl-task] m3-core-e2e-pilot-fix-impl-e2e-harness-2 — see .claude/PRPs/briefs/m3-core-e2e-pilot-fix-impl-e2e-harness-2.md`

You are the **impl-task** subagent (Sonnet 4.6). This is a **fix-impl task** — three small source/config fixes uncovered by Junior #761's e2e run AND a latent-blocker audit. After the edits, commit + push, write a `kind: "log"` DQ. Do NOT run the e2e suite (the advisor re-runs it).

## §2 Scope

**Branch:** `phase-m3-core-e2e-pilot` (tip `58d0b6214` or newer).

**Three fixes. Two files.** No Cargo.toml changes. No test-logic changes (only the S3 method call shape).

### Fix 1 — rust-s3 builder API (recording.rs)

File: `services/bridge/tests/recording.rs`. **Two occurrences** at lines 79 and 209:

```rust
let bucket = s3::Bucket::new_with_path_style(&s3_bucket, region, creds)
```

`Bucket::new_with_path_style` does NOT exist in rust-s3 0.34 — Junior #761 hit a COMPILE_FAIL on it (my prior brief guessed the method name). The correct API is the builder chain: `Bucket::new(...)` returns a `Result<Box<Bucket>>`, and `.with_path_style()` is a consuming builder method on `Bucket`. Change both occurrences to:

```rust
let bucket = s3::Bucket::new(&s3_bucket, region, creds)
    .map(|b| b.with_path_style())
```

**IMPORTANT — preserve the existing `?`/`.map_err(...)` tail.** Read each call site (lines 79 and 209) in full before editing — the original line is the START of a multi-line expression that ends with `.map_err(...)?` or similar. The `.with_path_style()` must be inserted so the bucket is path-styled BEFORE the error-handling tail. `Bucket::new` returns `Result<Box<Bucket>>` and `with_path_style(self) -> Box<Bucket>`, so `.map(|b| b.with_path_style())` keeps the `Result` shape for the existing `?`. Verify the surrounding lines compile-shape after your edit (the expression must still resolve to a `Box<Bucket>` bound to `bucket`).

**Verify count before editing:**
```bash
grep -c 'Bucket::new_with_path_style' services/bridge/tests/recording.rs
```
Must print `2`. After your edit, `grep -c 'Bucket::new_with_path_style'` must print `0` and `grep -c 'with_path_style()' ` must print `2`.

### Fix 2 — LiveKit v1.7 --bind 0.0.0.0 (docker-compose.yml)

File: `services/bridge/docker-compose.yml`, line 57:

```yaml
    command: ["--dev"]   # standalone dev mode; uses devkey/devsecret (matches lk-jwt env). No mounted config in v0.
```

Replace with:

```yaml
    command: ["--dev", "--bind", "0.0.0.0"]   # --dev = devkey/devsecret; --bind 0.0.0.0 so host-native tests reach Twirp via Docker port-map (v1.7 --dev alone binds container-loopback only)
```

**Why:** LiveKit v1.7 `--dev` binds to container-loopback (`127.0.0.1`) only. Docker port-maps `7880:7880` to the container's IP (`172.28.0.2:7880`), where nothing host-reachable listens → host-native cargo tests get `Connection refused` on every RoomClient/Twirp call. `--bind 0.0.0.0` (confirmed present in v1.7 via `docker run livekit/livekit-server:v1.7 --help`) forces all-interface binding so the Docker port-map works.

### Fix 3 — bridge DB host bind-mount for host-native test read access (docker-compose.e2e.yml)

File: `services/bridge/docker-compose.e2e.yml`.

**Problem:** bridge-a/bridge-b write `/data/bridge-a.db` (resp `bridge-b.db`) as container uid 1000. The named volumes `bridge_a_data`/`bridge_b_data` have root-owned host mountpoints, so the host-native test (run as user `barrie`) gets `Permission denied` reading the live DB and falls back to a STALE docker-cp snapshot — causing `clean_posture`, `participant_floor_fetch`, and `rtc_disabled_townhall_clean_posture` to fail on stale data.

**Fix:** switch both bridge DB volumes from named volumes to host bind-mounts under a test-user-readable dir.

1. Line 81: `      - bridge_a_data:/data` → `      - ./.e2e-data/bridge-a:/data`
2. Line 166: `      - bridge_b_data:/data` → `      - ./.e2e-data/bridge-b:/data`
3. In the top-level `volumes:` block (lines 178-182), REMOVE the now-unused `bridge_a_data:` (line 181) and `bridge_b_data:` (line 182) entries. Leave `minio_data:` and `tuwunel_b_data:` intact.

**Add the dirs to .gitignore** so the runtime SQLite files aren't tracked. Append to `services/bridge/.gitignore` (create if absent):
```
.e2e-data/
```
If a root `.gitignore` already covers `services/bridge/.e2e-data/`, skip — verify with `git check-ignore services/bridge/.e2e-data/bridge-a/bridge-a.db` (should print the path if ignored).

The run procedure (advisor-side, NOT your job) will `mkdir -p services/bridge/.e2e-data/{bridge-a,bridge-b} && chmod 0777` before `docker compose up`. You only make the compose edit + gitignore.

## §3 Required reading

- `services/bridge/tests/recording.rs` lines 70-90 and 200-215 (the two Bucket call sites + their error tails) — read BEFORE editing.
- `services/bridge/docker-compose.yml` lines 54-59 (livekit service).
- `services/bridge/docker-compose.e2e.yml` lines 78-82, 163-167, 178-182 (volume mounts + top-level volumes).
- `.claude/decision-queue.json` — read before the DQ-log write.

## §4 Procedure

1. Confirm branch: `git branch --show-current` = `phase-m3-core-e2e-pilot`.
2. Fix 1: edit recording.rs both call sites (read each in full first; preserve error tail).
3. Fix 2: edit docker-compose.yml line 57 command array.
4. Fix 3: edit docker-compose.e2e.yml (2 volume lines + remove 2 top-level volume entries) + add `.e2e-data/` to `services/bridge/.gitignore`.
5. Verify scope:
   ```bash
   git diff --stat
   ```
   Expect: `recording.rs`, `docker-compose.yml`, `docker-compose.e2e.yml`, `.gitignore` (4 files).
6. Verify Fix-1 counts:
   ```bash
   grep -c 'Bucket::new_with_path_style' services/bridge/tests/recording.rs   # must be 0
   grep -c 'with_path_style()' services/bridge/tests/recording.rs              # must be 2
   ```
7. Commit:
   ```bash
   git add services/bridge/tests/recording.rs services/bridge/docker-compose.yml services/bridge/docker-compose.e2e.yml services/bridge/.gitignore
   git commit -m "fix(bridge/e2e): rust-s3 builder API + LiveKit --bind 0.0.0.0 + DB host bind-mount"
   ```
   Commit body:
   ```
   Fix 1: recording.rs — Bucket::new_with_path_style (does not exist in rust-s3 0.34)
   → Bucket::new(...).map(|b| b.with_path_style()) builder chain. 2 call sites.

   Fix 2: docker-compose.yml — livekit command ["--dev"] → ["--dev","--bind","0.0.0.0"].
   v1.7 --dev binds container-loopback only; --bind 0.0.0.0 makes the Docker port-map
   reachable from host-native cargo tests.

   Fix 3: docker-compose.e2e.yml — bridge_a_data/bridge_b_data named volumes → host
   bind-mounts ./.e2e-data/bridge-{a,b}. Named-volume mountpoints are root-owned; the
   host-native test (uid barrie) could not read the live DB and fell back to a stale
   docker-cp snapshot. Bind-mount + chmod 0777 (run-time) gives the test read access.

   No Cargo.toml/Cargo.lock changes. No test-logic changes.

   LESSON: rust-s3 0.34 uses Bucket::new(...).with_path_style() builder chain, NOT a
   new_with_path_style constructor. LiveKit v1.7 --dev binds container-loopback; add
   --bind 0.0.0.0 for host-native reachability. Named Docker volumes are root-owned at
   the host mountpoint — host-native test readers need bind-mounts + perms, not named volumes.
   ```
8. Push: `git push origin phase-m3-core-e2e-pilot`.
9. Write `kind: "log"` DQ entry via `bash /srv/brehon-fork/scripts/brehon/dq-v3-new-entry.sh` + `dq-v3-append-fragment.sh`. Context: "Fix-impl #2: rust-s3 builder API (recording.rs 2×), LiveKit --bind 0.0.0.0 (docker-compose.yml), DB host bind-mount (docker-compose.e2e.yml). Advisor re-runs e2e #762 with mkdir+chmod 0777 of .e2e-data + Brehon:3000 up + stack without bridge-b." Commit + push the DQ.

## §5 Constraints

- **4 files only**: recording.rs, docker-compose.yml, docker-compose.e2e.yml, .gitignore.
- **NO Cargo.toml/Cargo.lock changes.**
- **NO e2e run** — advisor re-runs it (the run needs host-side mkdir/chmod + Brehon server, which is advisor-side setup).
- **Read each recording.rs call site in full before editing** — the `Bucket::new(...)` line is the head of a multi-line `Result` expression; the `.with_path_style()` must slot in before the `?`/`.map_err` tail without breaking the type.
- **LESSON trailer** (in commit body above).

## §6 DoD

- `git diff --stat HEAD~1` = 4 files.
- `grep -c 'Bucket::new_with_path_style' services/bridge/tests/recording.rs` → 0.
- `grep -c 'with_path_style()' services/bridge/tests/recording.rs` → 2.
- `grep -- '--bind' services/bridge/docker-compose.yml` → 1 hit (the livekit command line).
- `grep -c 'bridge_._data:/data' services/bridge/docker-compose.e2e.yml` → 0 (both converted to bind-mounts).
- `grep '.e2e-data' services/bridge/.gitignore` → 1 hit.
- DQ log entry committed + pushed.

## §7 HANDOVER

```yaml
HANDOVER:
  task: m3-core-e2e-pilot-fix-impl-e2e-harness-2
  fix1_rust_s3: <"2× with_path_style() builder" | "FAIL: <reason>">
  fix2_livekit_bind: <"--bind 0.0.0.0 added" | "FAIL: <reason>">
  fix3_db_bindmount: <"2 volumes → bind-mounts, 2 named volumes removed, gitignore added" | "FAIL: <reason>">
  files_changed: <list>
  dq_log_id: <id>
  notes: "<confirm recording.rs error-tail preserved; confirm only 4 files>"
```
