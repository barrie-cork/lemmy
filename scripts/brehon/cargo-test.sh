#!/usr/bin/env bash
# Brehon dev utility: run `cargo test` on macOS / Linux with the same
# e2e --test-threads=1 guard as the Windows .bat sibling.
#
# Linux-side sibling of scripts/brehon/cargo-test.bat. Callers pass scope,
# crate, and runner flags as positional args; this wrapper just sets the
# repo root and invokes cargo with the user-supplied args, with one
# adjustment: when running `--test e2e` without `--no-run` and without an
# explicit `--test-threads=N`, append `-- --test-threads=1` so e2e
# testcontainers don't race.
#
# Per the .bat sibling and feedback_wrapper_script_flag_silence.md: callers
# control scope; the wrapper does not silently discard flags. The single
# behavioural override is the e2e thread-count guard, which is announced
# on stdout when it fires so callers can spot it in build logs.
#
# Usage (from any shell, any cwd inside the repo):
#     scripts/brehon/cargo-test.sh --workspace --features full
#     scripts/brehon/cargo-test.sh --test e2e -p lemmy_server                 # auto-injects --test-threads=1
#     scripts/brehon/cargo-test.sh --test e2e -p lemmy_server --no-run        # no injection (build only)
#     scripts/brehon/cargo-test.sh --test e2e -p lemmy_server -- --nocapture  # injection appended after `--`
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

echo "TOOLCHAIN_OK"
command -v cargo && cargo --version

# ---- libpq discovery on Windows (vcpkg-based, x64-windows dynamic triplet) ----
# Mirror of the cargo-test.bat sibling's libpq env wiring. Without this, a Git
# Bash invocation of this script links lemmy_server with no /LIBPATH for libpq
# and fails at MSVC link.exe with `LNK1181: cannot open input file 'libpq.lib'`.
# Per feedback_pq_sys_stale_cache.md (PMD #25): the env vars MUST be set in the
# same shell that invokes cargo, because cargo's pq-sys build fingerprint hashes
# PQ_LIB_DIR. If unset, cargo selects a stale "PQ_LIB_DIR=NotPresent" build
# output and emits no rustc-link-search directive.
case "${OSTYPE:-}" in
  msys*|cygwin*|win32*)
    : "${VCPKG_ROOT:=C:\\Users\\barri\\Developer\\vcpkg}"
    export VCPKG_ROOT
    export PQ_LIB_DIR="${VCPKG_ROOT}\\installed\\x64-windows\\lib"
    export PQ_INCLUDE_DIR="${VCPKG_ROOT}\\installed\\x64-windows\\include"
    # Prepend libpq.dll dir to PATH so the spawned lemmy_server.exe finds it at
    # e2e runtime. Use Git Bash mount form (`/c/...`) and `:` separator — Git
    # Bash converts to `;`-separated Windows PATH when invoking Windows processes.
    export PATH="/c/Users/barri/Developer/vcpkg/installed/x64-windows/bin:${PATH}"
    echo "PQ_LIB_DIR=${PQ_LIB_DIR}"
    ;;
esac
echo "---"
cd "$REPO_ROOT"

# Reconstruct args as a single string for grep-style detection (parity with
# the .bat sibling's findstr approach).
ARGS_STR="$*"

# The e2e thread-count guard: only fires when ALL of these hold:
#   - `--test e2e` is present
#   - `--no-run` is NOT present (we're actually running tests, not building)
#   - `--test-threads` is NOT already supplied
#
# When the guard fires, append `--test-threads=1`. If the caller already has
# a `--` runner separator, append after it; otherwise insert one.

if [[ "$ARGS_STR" == *"--test e2e"* ]] \
   && [[ "$ARGS_STR" != *"--no-run"* ]] \
   && [[ "$ARGS_STR" != *"--test-threads"* ]]; then
  if [[ "$ARGS_STR" == *" -- "* ]] || [[ "$ARGS_STR" == *" --"* ]]; then
    echo "BREHON_TEST_THREADS_GUARD: appending --test-threads=1 after existing --"
    exec cargo test "$@" --test-threads=1
  else
    echo "BREHON_TEST_THREADS_GUARD: appending '-- --test-threads=1' for e2e race safety"
    exec cargo test "$@" -- --test-threads=1
  fi
fi

exec cargo test "$@"
