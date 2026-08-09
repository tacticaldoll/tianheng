# publish-source-integrity Specification

## Purpose

Govern what must be true of the source `cargo publish` runs from: the committed state asserted before an
irreversible act, the tag signature actually verified rather than shape-matched, and the one thing about that
signature this gate deliberately does not judge.
## Requirements
### Requirement: A publish SHALL run only from the tagged release commit on the remote's main

`cargo publish` SHALL be reachable only from a source where all of the following hold. Each is committed state;
none is about packaged content.

- The worktree SHALL be clean, so `HEAD` describes what would be packaged.
- `HEAD` SHALL be a `release: X.Y.Z` snapshot commit whose version is the workspace version.
- `vX.Y.Z` SHALL exist, SHALL be an annotated tag, and SHALL point at `HEAD`.
- `HEAD` SHALL be the tip of the remote's `main`, read live rather than from a possibly-stale `refs/remotes/`.
- The gate SHALL be read-only: it never fetches, commits, tags, or publishes.

**Clean is defined by the repository, not by the checkout.** A file ignored by **tracked** repository content
is clean, because `cargo publish` applies the same exclusion and would not package it either. A file hidden by
this clone (`.git/info/exclude`) or this machine (`core.excludesFile`, including the `$XDG_CONFIG_HOME/git/ignore`
default that no configuration names) is **not** clean: the same commit would otherwise get different verdicts
in different places, which is the one thing a governance gate must never do.

The judgement's own git invocations SHALL therefore run hermetically **and** neutralise `core.excludesFile`
explicitly. Measured: hermetic invocation alone leaves the XDG default applying, so a repair that stopped there
would read as closed while still hiding files.

What no configuration can neutralise SHALL be classified by **source** rather than refused wholesale. The paths
git excludes are the difference between an unexcluded listing and an excluded one, and each one's source file
is readable; a source SHALL count as repository content only if it is **tracked**, because an untracked
`.gitignore` reports a repository-looking source while being no more part of the repository than the clone's
own exclude file. Refusing whenever a clone carries an exclude file was the simpler alternative and is
rejected: it trades a false clean for a false alarm on the gate standing before an irreversible act.

`cargo publish` stamps the commit it ran on into every tarball's `.cargo_vcs_info.json`, and a version can never
be re-uploaded, so that pointer is permanent from the moment it lands. The `0.4.0` family records a release
branch's tip rather than the commit its tag names; nothing about the shipped content is wrong, which is what
makes the class easy to miss and impossible to correct.

#### Scenario: The worktree is not clean

- **WHEN** the gate runs with any modified file, or any untracked file not ignored by tracked repository content
- **THEN** it exits `1`, because `HEAD` no longer describes what would be packaged

#### Scenario: A file is hidden by configuration outside the repository

- **WHEN** an untracked file is excluded by `core.excludesFile`, by its XDG default, or by `.git/info/exclude`
- **THEN** the gate still refuses; the verdict may not depend on where the checkout happens to sit, and the
  gate neutralises what it can and classifies by source what it cannot

#### Scenario: A file is ignored by tracked repository content

- **WHEN** an untracked file is excluded by a `.gitignore` the repository tracks
- **THEN** the gate does **not** refuse; the exclusion is part of the source being published, and `cargo publish`
  applies it too

#### Scenario: The exclusion source is a `.gitignore` the repository does not track

- **WHEN** an untracked `.gitignore` hides a file
- **THEN** the gate refuses; the file is hidden by the checkout rather than by the repository, whatever the
  source file is called

#### Scenario: HEAD is not the release snapshot the tag names

- **WHEN** `HEAD` is a commit on top of the `release: X.Y.Z` snapshot — a release branch's tip, whose tree may
  be identical
- **THEN** it exits `1`, because cargo records the commit and an identical tree does not save you

#### Scenario: HEAD is not the tip of the remote's main

- **WHEN** the remote's `main` names a different commit
- **THEN** it exits `1`, read live from the remote rather than from a local remote-tracking ref

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
