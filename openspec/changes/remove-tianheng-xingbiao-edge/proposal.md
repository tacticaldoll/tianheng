## Why

The Tianheng shell declares and carries a direct normal dependency on `xingbiao` even though its
source consumes metadata only through the dimension crates it composes. That unused edge makes the
self-law allowlist broader than the architecture it claims to preserve and leaves future direct
metadata coupling silently permitted.

## What Changes

- Tighten the Tianheng shell's enforced normal-dependency allowlist to the three composed dimensions
  and `serde_json`.
- Remove the unused direct `xingbiao` dependency from the `tianheng` crate.
- Regenerate the checked-in self-law projection and align adjacent explanatory comments.
- Prove the amendment by observing the tightened law reject the existing edge before migration and
  reject an independent disallowed edge after migration.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `self-law-projection`: The projected and enforced Tianheng shell dependency boundary becomes a
  minimal account of its direct normal dependencies.

## Impact

The change affects `crates/tianheng/Cargo.toml`, Tianheng's steward-owned self-constitution,
`AGENTS.self-law.md`, and prose that names the shell allowlist. It changes no public Rust API,
manifest version, package version, baseline, wire format, or adopter migration requirement. The
law edit remains a candidate until steward review accepts it.
