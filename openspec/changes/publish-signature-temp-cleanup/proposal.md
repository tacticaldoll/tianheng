## Why

The publish-source gate creates its signature workspace before installing the cleanup trap. An interruption or
acquisition failure after the directory exists but before the next command runs can leave that workspace in the
system temporary directory.

## What Changes

- Install cleanup before attempting to acquire the signature workspace.
- Keep cleanup inert until a path has actually been assigned.
- Add a matrix direction whose allocator creates and reports a directory before failing, proving the preinstalled
  trap removes the partial acquisition.

## Capabilities

### Modified Capabilities

- `publish-source-integrity`: temporary signature material is removed even when workspace acquisition fails after
  creating it.

## Impact

Successful publish-source verdicts, manifests, versions, and public APIs are unchanged. A failed temporary
workspace acquisition remains cannot-judge but no longer leaves the acquired directory behind.
