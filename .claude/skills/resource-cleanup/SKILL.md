---
name: resource-cleanup
description: >
  Identify and clean up wasted disk, memory, and Docker resources on the home server.
  DO use when: disk usage is high, user asks about disk space, cleanup, pruning, freeing space,
  "what's using disk", "clean up the server", or after large Docker operations (rebuilds, migrations).
  Also use when server-boss alerts show disk > 80% or memory pressure.
  Do NOT use for: application-level code cleanup or refactoring, deleting user data or media files,
  uninstalling services (use docker-compose changes instead).
---

# Resource Cleanup

Systematically identify and safely clean wasted resources on the EliteDesk home server. This skill works over SSH — all commands run on `barrie@192.168.1.157`.

## Philosophy

- **Scan first, act second** — always show the user what will be freed before deleting
- **Never touch user data** — media, databases, uploads, and config are off-limits
- **Docker is the biggest offender** — unused images, build cache, and containerd layers accumulate fast
- **Caches regenerate** — package manager and tool caches are always safe to clear

## Step 1: Assess Current State

Run these commands over SSH to get the full picture:

```bash
# Disk overview
df -h /

# Top-level space consumers (excluding media mounts)
sudo du -sh /srv/media-local/docker /var/cache /home /srv/webdata /srv/backups /tmp 2>/dev/null | sort -rh

# Docker-specific breakdown
docker system df
docker system df -v | head -40
```

Report the current usage percentage and the top consumers in a clear table.

## Step 2: Identify Safe-to-Clean Categories

Work through each category and report what can be freed:

### Docker (usually the biggest wins)

```bash
# Unused images (not referenced by any container)
docker image ls --filter "dangling=true" -q | wc -l
docker images --format "table {{.Repository}}\t{{.Tag}}\t{{.Size}}" | grep "<none>"

# All unused images (no running container)
docker image prune -a --dry-run

# Build cache
docker builder prune --dry-run

# Dangling volumes (no container reference)
docker volume ls -f dangling=true

# Containerd layers (can accumulate separately from Docker)
sudo du -sh /var/lib/containerd/
```

### System Caches

```bash
# APT package cache
sudo du -sh /var/cache/apt/

# Journal logs (check total size)
journalctl --disk-usage

# Temp files
sudo du -sh /tmp/ /var/tmp/
```

### User/Tool Caches

```bash
# Check each cache directory
for d in ~/.cache/trivy ~/.cache/pip ~/.cache/uv \
         ~/.npm ~/.cache/yarn ~/.cache/node-gyp \
         ~/.cache/playwright ~/.local/share/pipx; do
  [ -d "$d" ] && du -sh "$d"
done
```

### Old Backups

```bash
# Postgres backup retention (should be 14 days)
ls -lh /srv/backups/postgres/ | head -20
find /srv/backups/postgres/ -name "*.sql.gz" -mtime +14 | wc -l
```

## Step 3: Present Findings

Summarise findings in a table like:

| Category | Size | Risk | Command |
|----------|------|------|---------|
| Docker unused images | 12 GB | Safe | `docker image prune -a` |
| Docker build cache | 7.7 GB | Safe | `docker builder prune -a` |
| APT cache | 1.2 GB | Safe | `sudo apt-get clean` |
| Journal logs | 800 MB | Safe | `sudo journalctl --vacuum-size=200M` |
| Trivy cache | 600 MB | Safe | `rm -rf ~/.cache/trivy` |

**Always ask the user before executing cleanup commands.** Present the table and wait for confirmation.

## Step 4: Execute Cleanup

Run confirmed cleanups. Use `-f` (force) flags to avoid interactive prompts over SSH:

```bash
# Docker cleanup (most impactful)
docker image prune -a -f
docker builder prune -a -f
docker volume prune -f

# System cleanup
sudo apt-get clean
sudo journalctl --vacuum-size=200M

# User caches (adjust based on what's found)
rm -rf ~/.cache/trivy
rm -rf ~/.cache/pip
rm -rf ~/.npm/_cacache
```

## Step 5: Verify Results

```bash
df -h /
docker system df
```

Report before/after disk usage and the total space freed.

## Safety Rules

1. **Never run `rm -rf` on data directories** — `/srv/media`, `/srv/webdata`, `/srv/backups` (except expired backups)
2. **Never prune running containers** — `docker container prune` is safe but verify first
3. **Don't delete Docker volumes with data** — only prune truly dangling ones
4. **Keep journal logs** — vacuum to 200M, don't delete entirely
5. **Don't clean containerd layers if Docker is running** — can cause issues; only clean during maintenance windows
6. **Postgres backups** — respect the 14-day retention policy, don't delete newer backups

## Memory Integration

After a significant cleanup (> 5 GB freed), write a project memory:

```
type: qa-result
tags: disk-cleanup, maintenance
title: "Freed X GB — [main category]"
```

This feeds into weekly review metrics and helps track disk growth trends.
