# 26 - Storage Strategy

Prevents uncontrolled disk growth on a long-running install. Enforced by
`zenpaws-storage`, with limits configurable in Settings but shipping with
sane defaults.

## Quotas (defaults - user-adjustable in Settings, Phase 8)

| Category | Default limit | Eviction policy |
|---|---|---|
| Image cache (originals) | 2 GB | LRU eviction once over limit |
| Thumbnail cache | 500 MB | LRU eviction; thumbnails are cheap to regenerate |
| Temporary in-progress transfers | Bounded by free disk space + the 2GB per-file limit (`08_FILE_TRANSFER_PROTOCOL.md`) | Cleared immediately on cancel/failure, never left as orphaned partial files |
| Logs | 50 MB total, rotated | Oldest rotated out first |

## Directory layout (under the OS-appropriate app data dir)

```
<app-data>/
  zenpaws.sqlite            # WAL mode - also produces zenpaws.sqlite-wal/-shm
  downloads/                # completed, user-facing file downloads
  cache/
    images/
    thumbnails/
  transfers/tmp/            # in-progress chunked transfers, never user-visible
  logs/
```

## Why this matters for the low-end-hardware target

Unbounded caches on a 4GB-RAM/HDD target machine degrade the whole system,
not just ZenPaws - quotas here are a correctness requirement for the
performance budgets in `13_PERFORMANCE_GUIDELINES.md`, not just tidiness.

## Cleanup triggers

Quota checks run on app startup and after each completed transfer/image
receive - not on a background timer that would itself cost idle CPU,
consistent with the "avoid unnecessary polling" rule in
`06_NETWORK_ARCHITECTURE.md`.
