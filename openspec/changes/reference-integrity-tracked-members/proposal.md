## Why

The reference-integrity contract says untracked filesystem state cannot change a verdict, but its
workspace-member census currently accepts any `crates/*/Cargo.toml` present in the worktree. An untracked
illustrative crate can therefore turn a deliberately skipped reference into an enforced violation.

## What Changes

- Derive the member set from the same Git-tracked path index that owns reference existence.
- Add a negative fixture proving an untracked crate manifest cannot change whether a `crates/<name>/...`
  reference is judged.
- Preserve the existing 0/1/2 contract and all recognized reference forms.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `reference-integrity`: Make the existing tracked-evidence requirement explicit for workspace-member
  classification and add the missing untracked-manifest scenario.

## Impact

Only `scripts/check_reference_integrity.sh`, its failure matrix, and the synced reference-integrity spec are
affected. Public Rust APIs, manifests, package versions, dependencies, and Tianheng law are unchanged.
