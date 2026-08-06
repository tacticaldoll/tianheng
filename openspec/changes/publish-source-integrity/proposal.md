## Why

Two findings, and the second is why this is a capability rather than a one-line fix.

**1. The gate before the irreversible act accepts an unsigned tag.** Measured on a fixture:

```
$ git tag -a v9.9.9 -F -   # annotated, UNSIGNED, message quoting a verification log
release: 9.9.9
-----BEGIN SSH SIGNATURE-----
U1NIU0lHAAAAAQ== (quoted text, not a signature)
-----END SSH SIGNATURE-----

$ git cat-file tag refs/tags/v9.9.9 | grep -q '^-----BEGIN .* SIGNATURE-----$'
→ matches; the gate accepts it
$ git verify-tag v9.9.9
→ Couldn't decode signature: invalid format   (the tag is genuinely unsigned)
```

The assertion greps the **whole tag object, message included**, so a quoted signature block — a pasted
verification log, a maintainer's note — satisfies it. `cargo publish` stamps a permanent, non-re-uploadable
commit pointer, and this is the last automated check before it.

**2. No specification owns this gate.** All 34 specs were searched: none states that a publish must come from a
signed annotated tag at the tip of `main`. The gate's contract lives in its own header comment and in
`CHANGELOG.md` prose. So there is nowhere to declare the bound this fix creates, and nothing in the claim
surface says what the gate must hold — while `gate-shape-contract` exempts it from Definition-of-Done
membership *by name*, which is the one place a reader is told it is special.

**And the bound the header already states has a cause that is refuted.** It reads:

> the signature check asserts that the tag object CARRIES a signature, not that the signature verifies.
> **Verification needs an allowed-signers configuration that exists on a maintainer's machine and not in CI**

Measured, with no allowed-signers file anywhere (`GIT_CONFIG_GLOBAL=/dev/null`):

| mechanism | genuinely signed tag | unsigned tag quoting a signature |
| --- | --- | --- |
| the current `grep` | passes | **passes** ← the defect |
| `git verify-tag` | exit 1, `allowedSignersFile needs to be configured` | exit 1, **the identical message** |
| `%(contents:signature)` non-empty | passes | **passes** (returns the quoted text) |
| `ssh-keygen -Y check-novalidate -n git` | **passes** | **refuses**, `Couldn't decode signature: invalid format` |

So **verification does not need allowed-signers — attribution does.** The stated cause conflated the two, and
that conflation is exactly what kept the gate at shape-matching. Separated: *is this a valid signature over this
payload* is environment-independent and checkable; *is the signer authorized* needs the configuration CI has
not, and only that remains a bound.

This is also a live instance of `observation-bound-model`'s own declared bound — *whether a declaration's stated
cause is the real cause is not observed* — the second such instance found today. Recorded rather than quietly
corrected.

## What Changes

**A new capability, `publish-source-integrity`,** whose subject is what must be true of the source `cargo
publish` runs from.

- **It states the five assertions the gate already makes**, so the claim surface finally carries them: a clean
  worktree; `HEAD` a `release: X.Y.Z` snapshot whose version is the workspace version; `vX.Y.Z` annotated,
  signed, and pointing at `HEAD`; `HEAD` the tip of the remote's `main`, read live; and read-only, 0/1/2. These
  are described, not migrated — every one already has a direction in `scripts/test_publish_source.sh`.
- **The signature requirement strengthens** from *carries a signature block* to *carries a cryptographically
  valid signature over the tag payload*, verified with `ssh-keygen -Y check-novalidate -n git` over
  `%(contents:signature)` and the object with that block removed.
- **A signature this gate cannot verify is cannot-judge (2), never a violation.** A non-SSH signature is the
  live case: the family signs with SSH, but a GPG-signed tag is expressible and this mechanism cannot read it.
  Reporting it as a wrong source would be a false refusal on the irreversible path.
- **A tag-object read failure becomes 2, not 1.** It is currently folded into `|| fail`, which reports a
  cannot-judge as a wrong source.
- **The bound narrows and its cause is corrected**: *whether the signer is authorized is not observed*, owner
  inherited from the verification environment — not the engine, because no change to this gate could close it
  without a configuration CI does not have.

## Capabilities

### New Capabilities

- `publish-source-integrity`: what must hold of the source a publish runs from — the committed state asserted,
  the signature actually verified, and the one thing about a signature this gate deliberately does not judge.

### Modified Capabilities

None. `release-coherence` governs repository *state* — which phase the tree is in and whether its narrative
agrees — not the publish act, and folding this into it would make one capability answer two questions.

## Impact

- **New**: this specification, and one declared observation bound with its pinning test.
- **Modified**: `scripts/check_publish_source.sh` (the signature assertion, the read-failure status), and
  `scripts/test_publish_source.sh` (three directions: a quoted-signature refusal, a valid signature accepted
  with no allowed-signers, a non-SSH signature as cannot-judge).
- **Modified**: `CHANGELOG.md` — a **Fixed** entry; an adopter verifying a tarball's provenance is the reader
  who cares.
- **Not affected**: no crate's public API, no `Constitution`, no baseline format, no published artifact. Version
  class **PATCH**. Nothing already published changes; the gate's judgment of a correct release is unchanged.
