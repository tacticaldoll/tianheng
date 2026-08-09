# publish-source-integrity Specification

## Purpose

Govern what must be true of the source `cargo publish` runs from: the committed state asserted before an
irreversible act, the tag signature actually verified rather than shape-matched, and the one thing about that
signature this gate deliberately does not judge.
## Subject

- `scripts/publish.sh`
- `crates/kanhe/tests/publish_source.rs`
- `crates/kanhe/tests/publish_source_integrity.rs`
- `crates/kanhe/src/publish_source_gate.rs`

## Requirements
### Requirement: A publish SHALL run only from the tagged release commit on the remote's main

`cargo publish` SHALL be reachable only from a source where all of the following hold, **and only through a
gate observed to have run**. Each is committed state; none is about packaged content.

A wrapper that asks for this gate and reads only its exit status cannot tell *judged and clean* from *judged
nothing*: `libtest` exits `0` for a filter matching no test, so a renamed or silenced gate would let the
publish proceed with none of the conditions below checked. Reachability is therefore a property of the run
having happened, not of the command having been issued.

#### Scenario: The publish gate did not run

- **WHEN** the wrapper's gate invocation selects no test, or selects one that is ignored
- **THEN** the publish is refused before `cargo publish` is reached, and the refusal says the gate did not run
  rather than reporting the source clean

### Requirement: The release tag's signature SHALL be verified, not shape-matched

The gate SHALL assert that `vX.Y.Z` carries a **cryptographically valid** signature over the tag payload, not
that the tag object contains a line resembling a signature header.

Matching the shape accepts an unsigned tag whose *message* quotes a signature block — a pasted verification log,
a maintainer's note — because the assertion reads the whole object, message included. Measured on a fixture: such
a tag passes the shape match while `git verify-tag` reports `Couldn't decode signature: invalid format`.

Verification SHALL be environment-independent: the same tag SHALL receive the same verdict on a maintainer's
machine and in CI. `git verify-tag` SHALL NOT be used for it — measured with no allowed-signers file, it exits
non-zero with an identical `allowedSignersFile needs to be configured` message for a genuinely signed tag and an
unsigned one alike, so a gate built on it would always report cannot-judge in CI: the check disabled while
appearing strengthened.

Temporary signature material SHALL be owned by cleanup before its directory is acquired. If acquisition creates
and reports a directory before failing, the gate SHALL exit cannot-judge and SHALL remove that directory.

The payload SHALL be reconstructed by removing the signature block as a **suffix** of the tag object, never by
stripping from the first line resembling a signature header. Measured on a genuinely signed tag whose message also
quotes a verification log, stripping from the first such line truncates the payload and refuses a real signature —
a false refusal introduced by the hardening itself. Suffix removal keeps a quoted block inside the payload, where
it belongs.

The extracted signature SHALL be proven to be the exact suffix of the tag object before payload reconstruction.
A mismatch SHALL exit `2` cannot-judge; it SHALL NOT reach cryptographic verification as an exit-`1` invalid
signature.

A signature this gate cannot read SHALL be cannot-judge (`2`), never a violation. A non-SSH signature is the live
case. Reporting it as a wrong source would be a false refusal before an irreversible act.

A failure to read the tag object SHALL likewise be `2`, not `1`.

#### Scenario: An unsigned tag quotes a signature block in its message

- **WHEN** `vX.Y.Z` is annotated, unsigned, and its message contains a `-----BEGIN SSH SIGNATURE-----` line
- **THEN** the gate exits `1`, because the tag carries no signature and a quoted one is text

#### Scenario: A genuinely signed tag is accepted with no allowed-signers configuration

- **WHEN** `vX.Y.Z` is signed and no `gpg.ssh.allowedSignersFile` is configured or exists
- **THEN** the gate accepts the signature, because validity is verifiable without attribution and the verdict
  must not depend on where the gate ran

#### Scenario: Signature workspace acquisition fails after creating a directory

- **WHEN** temporary-workspace acquisition creates and reports its directory but returns failure
- **THEN** the gate exits `2` cannot-judge and removes the partially acquired directory

#### Scenario: A signed tag whose message also quotes a signature block

- **WHEN** `vX.Y.Z` is genuinely signed and its message contains a quoted `-----BEGIN SSH SIGNATURE-----` block
  before the real trailer
- **THEN** the gate accepts it, because the payload is reconstructed by suffix removal and the quote stays inside
  the payload

#### Scenario: Extracted signature and tag object disagree

- **WHEN** Git's extracted non-empty SSH signature is not the exact suffix of the tag object read by the gate
- **THEN** the gate exits `2` because it cannot reconstruct the signed payload reliably

#### Scenario: A signature the gate cannot read

- **WHEN** `vX.Y.Z` carries a signature this mechanism cannot verify — a non-SSH one
- **THEN** the gate exits `2` naming what it could not read, never `1`

### Requirement: Observation bounds

Each bound declared here SHALL carry a typed declaration classifying where its measure stops, keyed on its
derived id, per `observation-bound-model`.

#### Scenario: Whether the tag's signer is authorized is not observed — a stated bound

- **WHEN** `vX.Y.Z` carries a cryptographically valid signature made by a key no maintainer authorized
- **THEN** the gate accepts it, a stated bound: validity is verifiable without configuration and **attribution is
  not**, needing an allowed-signers file that exists on a maintainer's machine and not in CI. The ownership is
  inherited from the verification environment rather than held by this engine, because no change to this gate
  closes it — giving CI an allowed-signers file is what would
- **PINNED-BY** `a_valid_signature_from_an_unauthorized_key_is_accepted`
