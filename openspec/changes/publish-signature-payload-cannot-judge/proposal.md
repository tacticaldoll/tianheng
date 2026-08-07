## Why

Shell suffix removal silently returns the original tag object when Git's extracted signature is not an exact
suffix. The following cryptographic check then exits as a wrong-source violation, although the gate actually failed
to reconstruct the signed payload and cannot judge the tag.

## What Changes

- Assert that the extracted signature is the tag object's exact suffix before removing it.
- Report a mismatch as exit 2 cannot-judge.
- Add a matrix direction that corrupts only the extracted block after a real signed tag and working verifier exist.

## Capabilities

### Modified Capabilities

- `publish-source-integrity`: payload reconstruction failure is distinct from an invalid signature verdict.

## Impact

An extraction mismatch changes from exit 1 to exit 2. Normal publish-source verdicts, manifests, versions, and
published APIs are unchanged.
