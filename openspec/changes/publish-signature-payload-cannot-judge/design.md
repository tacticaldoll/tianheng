## Context

`${tag_object%"$tag_signature"}` has no failure signal: a non-matching suffix yields `tag_object` unchanged. Feeding
that value to `ssh-keygen` collapses reconstruction failure into cryptographic invalidity.

## Decision

Before writing or verifying signature material, require `tag_object` to end with the exact non-empty signature
returned by `for-each-ref`. Route mismatch through `cannot_judge`; only an exactly reconstructed payload may reach
the invalid-signature exit-1 path.

The matrix stubs only the signature query, appending text to a real extracted SSH block. This preserves the valid
tag, verifier round-trip, and armor-kind checks while making only the suffix relation false.

## Verification

The new direction must exit 1 against silent suffix removal and exit 2 after the explicit relation check, followed
by the full publish-source matrix and repository Definition of Done.
