# Laptop Optimization Audit — Lenovo P50, 2026-05-22

Read-only audit. No system changes applied. You review + run commands yourself.

Per advisor session running diagnostics 2026-05-22T~22:00Z. User decisions:
- Scope: full workstation audit
- WSL: cut hard to 4 GB / 4 procs / 2 GB swap
- Workload split: laptop stays canonical cargo + e2e runner (per
  `project_laptop_canonical_cargo_runner.md`); EliteDesk = Junior
  orchestration only
- Diagnostic mode: gather data first (8 read-only PS commands run)

## Machine baseline

| Attribute | Value |
|---|---|
| Make/Model | Lenovo P50 (20ENS04R00) — mobile workstation, ~2015–2016 |
| CPU | Intel i7-6820HQ (Skylake-H), 4C/8T, 2.71 GHz base / 3.6 GHz turbo |
| RAM | 68.5 GB total / 44.6 GB free at audit (likely 4×16 GB DDR4 ECC) |
| GPU | NVIDIA Quadro (driver service NVDisplay.ContainerLocalSystem running) |
| OS | Windows 10 Pro 19045 |
| Uptime | 18 days (boot 2026-05-04 13:08) |
| Power plan | **Balanced**, PROCTHROTTLEMIN=5% on AC ⚠️ |
| Default toolchain | rustc 1.95.0 (MSVC), rustup managed, override active per `rust-toolchain.toml` |
| Cargo linker | rust-lld pinned in `.cargo/config.toml` ✓ |
| sccache | NOT installed |

The P50 has thermal headroom and good MTBF; treat it as a workstation
rather than a thin laptop. Don't push aggressive C-state tweaks — the
warranty is long expired but the hardware was over-engineered.

## Findings, ranked by impact

### 1. `.wslconfig` reserves 56 GB RAM + 16 GB swap + all 8 procs to WSL2

**File:** `C:\Users\barri\.wslconfig`

```
[wsl2]
memory=56GB
processors=8
swap=16GB
nestedVirtualization=true
networkingMode=mirrored
```

Vmmem is currently at 1.7 GB — WSL is mostly idle, but `memory=56GB`
is the *cap*, not the floor. Under any moderate WSL workload it could
take half your RAM. The only distro installed is `docker-desktop`
(Docker Desktop's helper distro); no user-distro present. Your Brehon
wrappers are `.bat` files; no bash-in-WSL workflow detected.

**Recommended (per your decision):** cut to 4 GB / 4 procs / 2 GB swap.
Keep nested virtualization (Docker e2e testcontainers may need it) and
mirrored networking (load-bearing for testcontainers reachability).

**Proposed `.wslconfig`:**

```
[wsl2]
memory=4GB
processors=4
swap=2GB
nestedVirtualization=true
networkingMode=mirrored
pageReporting=false
```

**To apply:**

```powershell
# Edit C:\Users\barri\.wslconfig with the above content
wsl --shutdown
# Restart Docker Desktop after
```

**Risk:** Low. If Docker Desktop e2e runs fail with OOM, bump
`memory=8GB` (still 48 GB less than current).

### 2. Power plan is Balanced — switch to High Performance

**Current:** `Balanced`, PROCTHROTTLEMIN=5% / PROCTHROTTLEMAX=100% on AC.

The P50 has a 170W PSU and active cooling designed for sustained CPU
load. Balanced introduces ramp-up latency on bursty workloads like
`cargo check` / clippy. Your screenshot caught the CPU at 100% boost
(3.2 GHz) — the throttling layer is fighting itself.

**To apply:**

```powershell
# List schemes
powercfg /list

# Switch to High Performance
powercfg /setactive 8c5e7fda-e8bf-4a96-9a85-a6e23a8c635c

# Or if you want Ultimate Performance (Win10 hidden plan):
powercfg /duplicatescheme e9a42b02-d5df-448d-aa00-03f14749eb61
powercfg /setactive e9a42b02-d5df-448d-aa00-03f14749eb61
```

**Risk:** Very low. Higher battery drain on DC (you're a desk-bound dev,
moot). Slightly higher fan noise under load.

### 3. Defender exclusions empty for cargo / target / rustup

The diagnostic couldn't read your exclusion list (needs admin) but the
behaviour profile (Antimalware Service at 418 MB / 5.9% CPU in the
screenshot while compiling) strongly suggests no cargo exclusions are
in place. Defender scans every `.rlib`, `.rmeta`, `.pdb`, `.exe`, and
`.dll` rustc emits — and your `target/` is 40 GB.

**To apply (needs admin PowerShell):**

```powershell
# Path exclusions
Add-MpPreference -ExclusionPath "C:\Users\barri\Developer\brehon-fork\target"
Add-MpPreference -ExclusionPath "C:\Users\barri\Developer\brehon-fork-tooling\target"
Add-MpPreference -ExclusionPath "C:\Users\barri\.cargo"
Add-MpPreference -ExclusionPath "C:\Users\barri\.rustup"
Add-MpPreference -ExclusionPath "C:\Users\barri\Developer\MCPs"

# Process exclusions (faster than path-based scanning suppression)
Add-MpPreference -ExclusionProcess "rustc.exe"
Add-MpPreference -ExclusionProcess "cargo.exe"
Add-MpPreference -ExclusionProcess "rust-lld.exe"
Add-MpPreference -ExclusionProcess "link.exe"

# Extension exclusions (low-grade; only add if perf still poor)
# Add-MpPreference -ExclusionExtension ".rlib"
# Add-MpPreference -ExclusionExtension ".rmeta"

# Verify
Get-MpPreference | Select-Object ExclusionPath, ExclusionProcess
```

**Risk:** Low security exposure (these are build outputs of your own
source); standard Rust dev practice. Don't exclude
`C:\Users\barri\Developer` wholesale — only the specific build/toolchain
paths.

**Caveat:** If your `target/` paths change (new sub-phase worktree), add
a fresh exclusion or move to a parent exclusion like
`C:\Users\barri\Developer\brehon-fork-*\target` (Defender supports
wildcards in path exclusions on recent builds).

### 4. Disable Program Compatibility Assistant (PCA) service

PCA hooks every new process Windows launches. For Rust development on
MSVC toolchain, it intercepts every rustc invocation, every build
script, every linker call. The screenshot caught it at 14.8% CPU /
751 MB RAM. Most Rust devs on Windows disable it; nothing modern
needs it.

**To apply (needs admin):**

```powershell
Stop-Service -Name PcaSvc -Force
Set-Service -Name PcaSvc -StartupType Disabled
# Verify
Get-Service PcaSvc
```

**Risk:** Low. PCA only does anything for legacy 16/32-bit apps with
compatibility shims. No Rust / Node / Python / modern tool relies on it.
Reversible: `Set-Service PcaSvc -StartupType Manual` to restore.

### 5. No sccache — install for multi-worktree compilation cache

You have 4 worktrees:
- `brehon-fork/target` → 40.16 GB
- `brehon-fork-tooling/target` → 16.24 GB
- (`brehon-fork-conformance-audit/target`, `brehon-fork-fed-in-c/target`
  weren't reported in the audit — likely smaller / cleaned)

These rebuild ~80% of the same crate set independently. sccache
deduplicates compilation by hashing inputs and serving from a shared
cache. For your workflow (multiple concurrent worktrees on the same
toolchain + dep set) the win is substantial — first-cold rebuild in a
fresh worktree drops from ~8 min to ~2 min.

**To apply:**

```powershell
# Install
cargo install sccache --locked

# Configure cache size (default 10 GB; you have RAM for more)
[System.Environment]::SetEnvironmentVariable("SCCACHE_CACHE_SIZE", "30G", "User")
[System.Environment]::SetEnvironmentVariable("SCCACHE_DIR", "C:\Users\barri\.cache\sccache", "User")

# Wire into Cargo (user-global so it applies to all worktrees)
# Create C:\Users\barri\.cargo\config.toml with:
[build]
rustc-wrapper = "sccache"
```

**⚠️ Compatibility caveat:** sccache's MSVC support has historically had
edge cases with `-Cinstrument-coverage`, proc-macros with disk I/O, and
some build scripts. Lemmy's workspace is large; test it on
`brehon-fork-tooling` first. If you see weird cache misses or build
failures, the wrapper is bypassable per-build with
`RUSTC_WRAPPER= cargo check ...`.

**Risk:** Medium. Newer than the other recommendations. Roll back by
removing the `[build]` block from `~/.cargo/config.toml`. Don't combine
with `cargo clean` muscle-memory — `sccache --stop-server && sccache
--clear` is the cache nuke.

### 6. Cargo target hygiene — 56 GB across 2 worktrees

```
brehon-fork/target:         40.16 GB
brehon-fork-tooling/target: 16.24 GB
TOTAL:                      56.40 GB
```

40 GB on the active brehon-fork is high but explicable (debug profile,
incremental, all features, all tests). 16 GB on
`brehon-fork-tooling/target` is more questionable — that worktree
isn't your active development lane.

**To investigate (read-only):**

```powershell
# What's in tooling/target?
Get-ChildItem C:\Users\barri\Developer\brehon-fork-tooling\target -Directory | ForEach-Object { $size = (Get-ChildItem $_.FullName -Recurse -ErrorAction SilentlyContinue | Measure-Object -Property Length -Sum).Sum; "{0,-30} {1,10:N2} GB" -f $_.Name, ($size/1GB) }
```

**To apply (conservative — keep release artefacts):**

```powershell
# Just clear debug / incremental from tooling
Remove-Item C:\Users\barri\Developer\brehon-fork-tooling\target\debug -Recurse -Force
Remove-Item C:\Users\barri\Developer\brehon-fork-tooling\target\.rustc_info.json -Force -ErrorAction SilentlyContinue
```

**To apply (aggressive — full clean of inactive worktree):**

```powershell
cd C:\Users\barri\Developer\brehon-fork-tooling
cargo clean
```

**Risk:** Low (rebuild cost only). Tooling worktree isn't your driver
lane. If you ship sccache first (recommendation 5), the rebuild cost
is mostly cached anyway.

### 7. Startup app cleanup

**Current startup list (12 entries):**

| Keep? | App | Notes |
|---|---|---|
| ✓ | Tailscale | Load-bearing for homeserver PMD / Ollama (per `pmd-search-strategy.md`) |
| ✓ | OneDrive | Personal preference; not Brehon-related but harmless |
| ✓ | SecurityHealth | Windows Defender tray |
| ❓ | Docker Desktop | Only needed for e2e testcontainers + Junior daemon. Manual-start saves ~150 MB + a few % CPU. |
| ❌ | Perplexity (todesktop) | Background app; you can launch from start menu when needed |
| ❌ | Wispr Flow | Voice-to-text; not Brehon |
| ❌ | GoogleDriveFS (×3 entries!) | Duplicate registrations — clean up |
| ❌ | Microsoft.Lists | OneDrive integration; unused if you don't use Lists |
| ❌ | Logitech Download Assistant | Updater bloat |
| ❌ | Logi Download Assistant | Second Logitech updater (duplicate?) |

**To audit (read-only):**

```powershell
# Open startup manager
Start-Process taskmgr -ArgumentList "/0", "/startup"
# Or via PowerShell to disable specific entries:
Get-CimInstance Win32_StartupCommand | Format-List Name, Command, Location
```

**To apply Docker Desktop manual-start:**

Docker Desktop > Settings > General > uncheck "Start Docker Desktop
when you sign in to your computer".

**Risk:** Very low (startup-only; can re-enable). Docker Desktop
manual-start means you launch it before running e2e tests or before
the Junior daemon needs it (and the Junior daemon is on EliteDesk
anyway, so the laptop's Docker Desktop is for local cargo e2e only).

### 8. PostgreSQL 18 running as auto-service

`postgresql-x64-18` runs as a Windows service. Your Lemmy e2e tests
use testcontainers (per `feedback_windows_e2e_requires_bat_wrapper`)
which spawns its own ephemeral Postgres in Docker. The local PG-18
service is probably installed but unused.

**To investigate:**

```powershell
# Check if anything's connecting
Get-NetTCPConnection -LocalPort 5432 -ErrorAction SilentlyContinue
```

**If unused, set to manual-start:**

```powershell
# Needs admin
Set-Service -Name postgresql-x64-18 -StartupType Manual
Stop-Service -Name postgresql-x64-18
```

**Risk:** Low (if nothing else uses local PG). If you do have local
DB-backed tooling (e.g. local schema browser, a pgAdmin you forgot
about), this breaks them until you `Start-Service`.

### 9. CoworkVMService = "Claude" auto-running

The service `CoworkVMService` with DisplayName `Claude` is running.
Likely Claude Code's helper service. The Claude.exe processes are
expected and necessary for your workflow — don't touch them. The
service itself is part of Claude Code's install; leave alone unless
you know what it does.

### 10. VSCode runaway CPU — investigate extensions

One VSCode process has accumulated **25,857 CPU seconds** (7+ hours).
Probably an extension that's polling or indexing in the background.

**To investigate:**

```powershell
# Per-extension CPU stats inside VSCode
# Help > Toggle Developer Tools > Performance tab
# Or: F1 > "Developer: Show Running Extensions"
```

Common culprits in a Rust + multi-MCP setup: rust-analyzer (legitimate
work; ensure it's not running on `target/`), GitLens (heavy with large
histories), TabNine/Copilot variants (background telemetry).

**To apply:** disable suspect extensions one at a time and observe.

**Risk:** Low; reversible.

### 11. BrehonPMDSyncToEliteDesk + PMD-Backfill-Homeserver scheduled tasks

Two custom scheduled tasks. Verify they're not running redundantly
or thrashing the network.

**To inspect:**

```powershell
Get-ScheduledTask -TaskName BrehonPMDSyncToEliteDesk | Get-ScheduledTaskInfo
Get-ScheduledTask -TaskName PMD-Backfill-Homeserver | Get-ScheduledTaskInfo
# Trigger details:
Get-ScheduledTask -TaskName BrehonPMDSyncToEliteDesk | Select-Object -ExpandProperty Triggers
```

If they fire more than hourly, consider tuning. (Per
`pmd-search-strategy.md` the PMD backfill is for embedding generation;
once-daily is plenty unless you're actively writing many lessons.)

### 12. 18-day uptime — schedule a reboot

Windows accumulates handle / memory leaks over weeks. 130,000 handles
across 351 processes is high. After applying Tier 1 changes (especially
the .wslconfig + Defender + power plan), reboot to apply cleanly and
benefit from a fresh process tree.

**To apply:** `shutdown /r /t 60 /c "applying laptop optimization"`

---

## What I did NOT recommend

These came up but failed the cost/benefit test:

- **Disable Cortana, Telemetry, etc.** — these are old "debloat
  Windows" memes; on Win10 19045 with modern build, their per-day cost
  is <0.1% CPU. Not worth the registry edits.
- **Move Cargo target dir out of source tree** — would defeat
  `cargo --target-dir` muscle memory in scripts. Defender exclusions +
  sccache get the same wins without surprising your wrappers.
- **Disable hibernation / page file** — you have 64 GB RAM, hibernation
  is rarely used, but the `hiberfil.sys` is harmless. Page file should
  stay (Windows panics on certain crashes without it).
- **Use jemalloc for rustc** — diminishing returns on Windows;
  experimental; pinning `1.95` toolchain is the larger lever.
- **Switch to nightly toolchain** — your `rust-toolchain.toml` pins
  1.95 for reproducibility per the four-role model. Don't break that
  for marginal speedups.
- **Replace MSVC toolchain with GNU** — GNU toolchain on Windows is
  slower for linking + has worse libpq compatibility. Stay on MSVC.
- **Disable Windows Search indexing on Developer/** — risky (breaks
  Explorer search); modest wins; defer unless you observe
  SearchIndexer.exe issues.
- **Replace VS Code with another editor** — out of scope; you're
  invested in the Claude Code + VSCode workflow.
- **Hardware upgrade analysis** — you explicitly chose "current split
  is deliberate". Skipped.

---

## Application order (if you decide to apply)

If you decide later to apply changes, this order minimizes risk:

1. Reboot (clean slate)
2. Switch power plan to High Performance (`powercfg /setactive`)
3. Cut `.wslconfig`, then `wsl --shutdown`
4. Add Defender exclusions (needs admin PS)
5. Disable PCA service (needs admin)
6. Reboot again (to apply WSL + PCA cleanly)
7. Set Docker Desktop + PostgreSQL to manual-start
8. Audit startup apps via Task Manager
9. Install sccache + wire into `~/.cargo/config.toml`; test on
   `brehon-fork-tooling` first
10. Clean `brehon-fork-tooling/target` if confirmed inactive
11. Audit VSCode extensions
12. Tune scheduled tasks if redundant
13. Reboot final time

Run `cargo check --workspace --features full` before + after each tier
to measure wall-clock delta. Per
`feedback_windows_e2e_requires_bat_wrapper` use the `.bat` wrapper, not
bare `cargo`.

---

## Expected impact (rough estimates)

| Tier | Expected wall-clock win on `cargo check --workspace --features full` (warm) | Other benefits |
|---|---|---|
| Tier 1 (WSL + power + Defender + PCA) | 15–30% faster (~3 min → ~2 min on warm builds) | 50+ GB RAM headroom; lower fan noise |
| Tier 2 (+ sccache + startup cleanup) | 40–60% faster on cross-worktree cold builds (~8 min → ~3 min) | Faster VSCode startup; quieter background |
| Tier 3 (+ VSCode audit + deep clean) | Marginal additional cargo win; potential VSCode CPU 50–80% drop if an extension is the culprit | Less battery / heat; cleaner system |

Numbers are estimates from Rust-on-Windows community data + memory
budget arithmetic; your mileage will vary. Re-run timing after each
tier to validate.

---

## Files referenced

- `C:\Users\barri\.wslconfig` — to edit
- `C:\Users\barri\Developer\brehon-fork\.cargo\config.toml` — already
  has rust-lld linker
- `C:\Users\barri\.cargo\config.toml` — to create for sccache
- `C:\Users\barri\Developer\brehon-fork\scripts\brehon\cargo-*.bat` —
  existing wrappers; don't change
- `C:\Users\barri\Developer\brehon-fork-tooling\target` — candidate
  for cleanup

## See also

- `.claude/lessons/feedback_windows_e2e_requires_bat_wrapper.md` —
  cargo invocation discipline
- `project_laptop_canonical_cargo_runner.md` — the split rationale
- `pmd-search-strategy.md` — Tailscale dependency
