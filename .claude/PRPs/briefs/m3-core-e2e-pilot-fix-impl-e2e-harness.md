# Brief: m3-core-e2e-pilot fix-impl — e2e harness: LiveKit v1.7 pin + rust-s3 path-style

## §1 Role + dispatch line

`[role:impl-task] m3-core-e2e-pilot-fix-impl-e2e-harness — see .claude/PRPs/briefs/m3-core-e2e-pilot-fix-impl-e2e-harness.md`

## §2 Scope

**Branch:** `phase-m3-core-e2e-pilot` (tip `ce2e96541` or newer).

**Two source fixes only.** No new files. No Cargo.toml changes. No test-logic changes.

### Fix A — LiveKit compose image: v1.8 → v1.7

File: `services/bridge/docker-compose.yml`

Find:
```yaml
    image: livekit/livekit-server:v1.8   # Apache-2.0; pin exact tag at impl
```

Replace with:
```yaml
    image: livekit/livekit-server:v1.7   # Apache-2.0; pinned to v1.7 (HTTP/1.1 Twirp compat with livekit-api 0.4.x)
```

**Why:** `livekit-api 0.4.24` uses `reqwest` HTTP/1.1 for Twirp API calls. LiveKit Server v1.8.x switched the Twirp transport to require HTTP/2 (h2c), causing `connection reset by peer` on every RoomClient call. Downgrading to v1.7.x restores HTTP/1.1 Twirp compatibility without changing any Rust source.

### Fix B — rust-s3 path-style: `Bucket::new` → `Bucket::new_with_path_style`

File: `services/bridge/tests/recording.rs`

There are **exactly 2** occurrences of `s3::Bucket::new(` in this file. Replace both with `s3::Bucket::new_with_path_style(`. The signature is identical — same arguments, same return type.

**Why:** `rust-s3 0.34` defaults to virtual-hosted style S3 URLs (`http://<bucket>.<endpoint>/key`). With `endpoint = "http://localhost:9000"` this constructs `http://recordings.localhost:9000/...` which fails DNS resolution. Path-style (`http://localhost:9000/<bucket>/key`) is required for MinIO and any single-host S3 deployment. MinIO itself defaults to path-style.

**Verify count before editing:**
```bash
grep -c 'Bucket::new(' services/bridge/tests/recording.rs
```
Must print `2`. If not 2, raise a `kind: "blocker"` DQ and STOP.

## §3 Required reading

- `services/bridge/docker-compose.yml` — read before editing (confirm current livekit image line)
- `services/bridge/tests/recording.rs` — read before editing (confirm 2× `Bucket::new(` anchors)
- `.claude/decision-queue.json` — read; no DQ write needed if fixes are clean

## §4 Procedure

1. Checkout and confirm branch:
   ```bash
   git branch --show-current  # must be phase-m3-core-e2e-pilot
   ```

2. Apply Fix A (1-line edit to docker-compose.yml).

3. Apply Fix B (2-line edits to recording.rs — one per occurrence of `Bucket::new(`).

4. Verify no other files touched:
   ```bash
   git diff --stat
   ```
   Must show exactly 2 files: `services/bridge/docker-compose.yml` + `services/bridge/tests/recording.rs`.

5. Commit:
   ```bash
   git add services/bridge/docker-compose.yml services/bridge/tests/recording.rs
   git commit -m "fix(bridge/e2e): LiveKit v1.7 pin + rust-s3 path-style for host-native e2e"
   ```
   Commit body:
   ```
   Fix A: livekit-server v1.8 → v1.7 (HTTP/1.1 Twirp compat with livekit-api 0.4.x;
   v1.8 requires h2c which reqwest/HTTP1 rejects with connection reset).
   
   Fix B: Bucket::new → Bucket::new_with_path_style (2×) in recording.rs;
   virtual-hosted style on localhost:9000 constructs recordings.localhost:9000 (DNS NXDOMAIN);
   path-style constructs localhost:9000/recordings/key (correct for MinIO).
   
   No Cargo.toml / Cargo.lock changes; no test-logic changes.
   
   LESSON: livekit-api 0.4.x is HTTP/1.1-only; pin compose image to v1.7 until livekit-api 0.5.x
   (h2 transport) is adopted. rust-s3 Bucket::new defaults to virtual-hosted; always use
   Bucket::new_with_path_style for local/MinIO endpoints.
   ```

6. Push to phase branch:
   ```bash
   git push origin phase-m3-core-e2e-pilot
   ```

7. Write a `kind: "log"` DQ entry documenting the fixes and confirming no validate-pending DQ is needed (this is a compose + test-helper fix — no Brehon workspace cargo involved):
   ```bash
   bash /srv/brehon-fork/scripts/brehon/dq-v3-new-entry.sh
   ```
   Fragment context: `"Fix A: livekit v1.8→v1.7 (HTTP/1.1 Twirp compat). Fix B: Bucket::new_with_path_style×2 in recording.rs (path-style for localhost MinIO). No validate-pending needed (bridge-only, not crates/). Advisor re-runs e2e."` — append to resolved[].

8. Commit DQ log entry and push.

## §5 Constraints

- **Only 2 files**: `docker-compose.yml` + `tests/recording.rs`. No other edits.
- **No Cargo.toml / Cargo.lock changes**: `livekit-api` version stays at `0.4`; the fix is the compose image, not the Rust dep.
- **No validate-pending DQ**: bridge `services/bridge/` is a separate workspace from `crates/`. The advisor will re-run the e2e tests (with `BRIDGE_DB_PATH` set) after this commit.
- **Anchor uniqueness**: the `Bucket::new(` anchor must appear exactly 2× in recording.rs — verify before editing.
- **LESSON trailer** in commit body (shown above).

## §6 DoD

- `git diff --stat HEAD~1` shows exactly 2 files changed.
- `grep 'v1.7' services/bridge/docker-compose.yml` → 1 hit on the livekit service line.
- `grep 'Bucket::new_with_path_style' services/bridge/tests/recording.rs` → 2 hits.
- `grep -c 'Bucket::new(' services/bridge/tests/recording.rs` → 0 hits (all replaced).
- DQ log entry written, committed, pushed.

## §7 HANDOVER

```yaml
HANDOVER:
  task: m3-core-e2e-pilot-fix-impl-e2e-harness
  docker_compose_fix: <"v1.7 line present" | "FAIL: <reason>">
  rust_s3_fix: <"2× Bucket::new_with_path_style" | "FAIL: <reason>">
  files_changed: <list>
  dq_log_id: <id>
  notes: "<confirm no other files touched>"
```
