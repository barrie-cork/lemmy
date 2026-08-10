#!/usr/bin/env bash
# Surgical revert of the extism 1.30.0 / wasmtime 43.0.2 bump that landed on
# governance-v0 via merged Dependabot PR #218 (cargo-all group, 33 updates),
# keeping the other 32 bumps.
#
# NOTE (2026-08-10): #218 was MERGED to governance-v0 (fd6bbc6b5) before this
# ran, so the bad bump is on TRUNK, not in an open PR. This script now fixes
# forward: cut a branch from current trunk and re-pin. It does NOT touch the
# (now-merged, closed) PR branch.
#
# WHY: extism 1.30.0 (-> wasmtime 43.0.2) is known-UNBUILDABLE. wasmtime 43's
# no_std refactor dropped the std::error::Error impl extism 1.30.0 relies on ->
# 42x E0277 in the Docker server build. Reverted once at d12aed221 (2026-06-05).
# See MEMORY.md watch_extism_wasmtime_42_adopt. crates.io newest is still 1.30.0
# as of 2026-08-10 -> trigger NOT met; do NOT adopt.
#
# The root Cargo.toml still pins `extism = "1.21.0"` (unchanged by #218) -- the
# 1.30.0 came in via lockfile-only group resolution. So the revert is a lockfile
# re-pin: force extism back to 1.21.0 and cargo drags wasmtime + the whole tree
# back with it. No manifest edit needed for extism.
#
# RUN THIS ON LINUX (or via cargo-linux.sh) -- the lockfile is resolved the same
# on any host, but the compile proof at the end must be the Linux deploy target.
# The Windows host cannot build the extism/wasmtime chain natively.
set -euo pipefail

# rustup installs cargo to ~/.cargo/bin and puts it on PATH via ~/.cargo/env,
# which a non-interactive `ssh host 'cmd'` shell does NOT source. Source it so
# `cargo update` (line ~50) resolves. Harmless if cargo is already on PATH.
# shellcheck disable=SC1090,SC1091
[ -f "$HOME/.cargo/env" ] && . "$HOME/.cargo/env"
command -v cargo >/dev/null 2>&1 || { echo "cargo not found even after sourcing ~/.cargo/env"; exit 127; }

REPO="${1:-.}"
TRUNK="governance-v0"
WORK_BRANCH="chore/revert-extism-1.30"

cd "$REPO"

echo "==> 1. Cut a fresh branch from current trunk (has all 33 #218 bumps)"
git fetch origin "$TRUNK"
git checkout -B "$WORK_BRANCH" "origin/$TRUNK"

echo "==> 2. Confirm the lockfile currently has the bad versions"
grep -A1 '^name = "extism"'   Cargo.lock | grep 'version = "1.30.0"'  || { echo "extism 1.30.0 not in lock -- nothing to revert"; exit 1; }
grep -A1 '^name = "wasmtime"' Cargo.lock | grep 'version = "43.0.2"'  || { echo "wasmtime 43.0.2 not in lock -- unexpected"; exit 1; }

echo "==> 3. Re-pin ONLY extism back to 1.21.0 (wasmtime follows transitively)"
# --precise pins the exact version; the manifest constraint (=1.21.0 already, or
# ^1.21) permits it, so cargo re-resolves extism's subtree without touching the
# other 32 group members.
cargo update -p extism --precise 1.21.0

echo "==> 4. Verify the revert landed and nothing else moved unexpectedly"
grep -A1 '^name = "extism"'   Cargo.lock | grep 'version = "1.21.0"'  || { echo "FAIL: extism not back at 1.21.0"; exit 1; }
grep -A1 '^name = "wasmtime"' Cargo.lock | grep 'version = "41.0.4"'  || { echo "FAIL: wasmtime not back at 41.0.4 -- check cargo output"; exit 1; }
echo "    extism -> 1.21.0 OK ; wasmtime -> 41.0.4 OK"

echo "==> 5. Show the net Cargo.lock delta vs origin/governance-v0 for the two crates"
git --no-pager diff origin/governance-v0 -- Cargo.lock | grep -iE '^[+-].*(extism|wasmtime)' | head -40 || true

echo "==> 6. Commit"
git add Cargo.lock
git commit -m "chore(deps): drop extism 1.30.0/wasmtime 43 bump from cargo-all group (PR #218)

extism 1.30.0 (-> wasmtime 43.0.2) is known-unbuildable (wasmtime 43 no_std
refactor drops the std::error::Error impl extism relies on; 42x E0277 in the
Docker server build). Reverted once at d12aed221. #218 merged the whole
cargo-all group to governance-v0 (fd6bbc6b5); this keeps the other 32 bumps
and re-pins ONLY extism to 1.21.0 (wasmtime 41.0.4 follows).

Watch: watch_extism_wasmtime_42_adopt -- re-adopt only when an extism release
pairs wasmtime >=42 AND keeps/adapts the std::error::Error impl."

echo "==> 7. Linux compile proof on the deploy target (extism/wasmtime chain)"
# Invoke via `bash` not `./` -- cargo-linux.sh is tracked -rw-r--r-- (no +x bit),
# so `./script` gives Permission denied (exit 126). `bash script` ignores the bit.
bash scripts/brehon/cargo-linux.sh check --workspace --features full > /tmp/revert-extism-check.log 2>&1
status=$?
tail -30 /tmp/revert-extism-check.log
echo "cargo-linux check exit: $status"
[ $status -eq 0 ] || { echo "BUILD STILL RED after revert -- inspect /tmp/revert-extism-check.log (may be ed25519-dalek 3 or another #218 bump, NOT extism)"; exit $status; }

cat <<'EOF'

==> NEXT (do NOT skip -- the other 32 bumps rode in on the same merge):
    This branch still carries ed25519-dalek 2->3 (MAJOR; governance-log signing
    crate), jsonwebtoken 10->11 (auth), base64 0.22->0.23, diesel 2.3.10->2.3.11.
    The check above compiled them; before merging also run the test gate:

      ./scripts/brehon/cargo-linux.sh test --workspace --features full   # or e2e subset

    If ed25519-dalek 3 breaks the signing API at compile time, the check above
    already went red -- pin it back (cargo update -p ed25519-dalek --precise 2.x.y)
    or handle its migration in a dedicated deps sub-phase.

    Then: git push origin chore/revert-extism-1.30
          gh pr create --repo barrie-cork/lemmy --base governance-v0 \
            --head chore/revert-extism-1.30 --title "Revert extism 1.30.0/wasmtime 43 (unbuildable)"
    Let CR + the validate-pending gate run before merge.
EOF
