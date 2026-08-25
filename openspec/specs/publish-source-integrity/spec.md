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

The gate runs as `cargo test -p kanhe --test publish_source`, invoked by `scripts/publish.sh`, so *violation*
and *cannot-judge* below name values of its result type rather than process statuses.

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

### Requirement: A live remote read SHALL preserve whether it failed

The publish-source gate SHALL distinguish failure to execute the live `refs/heads/main` read from a successful
read that returns no such ref. A failed read SHALL be a cannot-judge naming the remote and preserving the read's
cause. A successful response without `refs/heads/main` SHALL be a cannot-judge naming the absent ref. Neither
condition SHALL be reported as a wrong-source violation or collapsed into the other's diagnostic.

#### Scenario: The live remote cannot be read

- **WHEN** `git ls-remote <remote> refs/heads/main` fails
- **THEN** the gate refuses as a cannot-judge naming the remote and the Git failure, rather than treating the
  response as empty

#### Scenario: The live remote has no main ref

- **WHEN** the live remote read succeeds but returns no `refs/heads/main`
- **THEN** the gate refuses as a cannot-judge naming the absent ref, distinct from a command that could not run

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
and reports a directory before failing, the gate SHALL refuse as a **cannot-judge** and SHALL remove that
directory.

The payload SHALL be reconstructed by removing the signature block as a **suffix** of the tag object, never by
stripping from the first line resembling a signature header. Measured on a genuinely signed tag whose message also
quotes a verification log, stripping from the first such line truncates the payload and refuses a real signature —
a false refusal introduced by the hardening itself. Suffix removal keeps a quoted block inside the payload, where
it belongs.

The extracted signature SHALL be proven to be the exact suffix of the tag object before payload reconstruction.
A mismatch SHALL be a **cannot-judge**; it SHALL NOT reach cryptographic verification and be reported as a
**violation** — an invalid signature.

A signature this gate cannot read SHALL be a **cannot-judge**, never a violation. A non-SSH signature is the live
case. Reporting it as a wrong source would be a false refusal before an irreversible act.

A failure to read the tag object SHALL likewise be a **cannot-judge**, never a violation.

**A verifier that reached no verdict is likewise a cannot-judge**, and the rule reaches the *mechanism* and
not only the signature's text. A verifier that could not be started, could not be handed its payload, or could
not be reaped has not rejected anything, so reporting it as an invalid signature names a fact nobody observed.
The round-trip probe narrows this and does not close it: it proves the mechanism before any verdict, so what
remains is a second invocation failing where the first succeeded. The verification result SHALL therefore
distinguish *did not run* from *ran and rejected* in its own type, rather than by a convention at the call
site — a boolean collapses the two and the caller cannot recover them.

#### Scenario: The signature verifier reaches no verdict

- **WHEN** the verifier cannot be started, cannot be handed the payload, or cannot be reaped
- **THEN** the gate refuses as a **cannot-judge** naming the mechanism, never as an invalid signature — the
  exit class `scripts/publish.sh` reads is `2` rather than `1`, because a gate that did not judge is not a
  gate that disagreed
- **PINNED-BY** `a_verifier_that_could_not_run_is_not_a_bad_signature`

#### Scenario: The signature verifier runs and rejects the payload

- **WHEN** the verifier executes and exits non-zero over the reconstructed payload
- **THEN** the gate refuses as a **violation**, because a completed verification rejecting the payload is a
  disagreement about the tag rather than a fact about the machine. Held in the same direction as the arm
  above, so closing the cannot-judge class cannot close this one with it
- **PINNED-BY** `a_verifier_that_could_not_run_is_not_a_bad_signature`

#### Scenario: An unsigned tag quotes a signature block in its message

- **WHEN** `vX.Y.Z` is annotated, unsigned, and its message contains a `-----BEGIN SSH SIGNATURE-----` line
- **THEN** the gate refuses it as a **violation**, because the tag carries no signature and a quoted one is
  text

#### Scenario: A genuinely signed tag is accepted with no allowed-signers configuration

- **WHEN** `vX.Y.Z` is signed and no `gpg.ssh.allowedSignersFile` is configured or exists
- **THEN** the gate accepts the signature, because validity is verifiable without attribution and the verdict
  must not depend on where the gate ran

#### Scenario: Signature workspace acquisition fails after creating a directory

- **WHEN** temporary-workspace acquisition creates and reports its directory but returns failure
- **THEN** the gate refuses as a **cannot-judge** and removes the partially acquired directory

#### Scenario: A signed tag whose message also quotes a signature block

- **WHEN** `vX.Y.Z` is genuinely signed and its message contains a quoted `-----BEGIN SSH SIGNATURE-----` block
  before the real trailer
- **THEN** the gate accepts it, because the payload is reconstructed by suffix removal and the quote stays inside
  the payload

#### Scenario: Extracted signature and tag object disagree

- **WHEN** Git's extracted non-empty SSH signature is not the exact suffix of the tag object read by the gate
- **THEN** the gate refuses as a **cannot-judge**, because it cannot reconstruct the signed payload reliably

#### Scenario: A signature the gate cannot read

- **WHEN** `vX.Y.Z` carries a signature this mechanism cannot verify — a non-SSH one
- **THEN** the gate refuses as a **cannot-judge** naming what it could not read, never as a violation

### Requirement: Observation bounds

Each bound declared here SHALL carry a typed declaration classifying where its measure stops, keyed on its
derived id, per `observation-bound-model`.

#### Scenario: The tree changing after the gate passed is not observed — a stated bound

- **WHEN** the repository is altered between the source gate reporting its single pass and `cargo publish`
  reading the tree — a commit, an amend, a tag moved, or the remote's `main` advancing
- **THEN** nothing reacts. The gate is one process and the act is another, and the wrapper holds no handle
  that ties them: `cargo publish` takes no argument naming the commit it must package, so there is no
  `--match-head-commit` to pin what was judged, which is what closes the equivalent window for the commit
  set on the merge path. What narrows it is `cargo publish`'s own refusal of a dirty worktree, which is a
  weaker property than the gate's — a tree amended and committed is clean again and packages a different
  commit under the same tag. The wrapper `cd`s once and `exec`s, so the window is two statements wide rather
  than a whole `cargo test`, and narrowing is all it can do
- **UNPINNED** `BACKLOG.md` — *the window the publish wrapper can only narrow*

#### Scenario: Whether the tag's signer is authorized is not observed — a stated bound

- **WHEN** `vX.Y.Z` carries a cryptographically valid signature made by a key no maintainer authorized
- **THEN** the gate accepts it, a stated bound: validity is verifiable without configuration and **attribution is
  not**, needing an allowed-signers file that exists on a maintainer's machine and not in CI. The ownership is
  inherited from the verification environment rather than held by this engine, because no change to this gate
  closes it — giving CI an allowed-signers file is what would
- **PINNED-BY** `a_valid_signature_from_an_unauthorized_key_is_accepted`

### Requirement: A path the gate classifies SHALL be the path it was given

Every path the cleanliness judgement reads, compares, or asks git about SHALL be carried as raw bytes, using
git's `-z` form for `ls-files`, `status`, and `check-ignore`. Git prints a path containing special or
non-ASCII bytes in a **quoted** form, and a quoted spelling is a different string: asking `check-ignore` about
it asks about a file that does not exist.

Both directions follow from that one substitution, and both were measured on a fixture rather than reasoned
about. A file named `ignored-普通`, ignored by a **tracked** `.gitignore`, is listed as
`"ignored-\346\231\256\351\200\232"`; `check-ignore` returns exit 1 for that literal, the source goes unshown,
and the gate refuses a file the repository itself ignores. Strip the quoting instead and `check-ignore`
answers about a *different* path — so a file hidden by this clone's own exclude could be cleared by a tracked
pattern that happens to match the quoted spelling.

A classification that could not be produced SHALL be a cannot-judge naming what went unclassified, never an
empty classification. `check-ignore` exiting non-zero because it could not run is not the same fact as
`check-ignore` matching nothing, and treating them alike lets a failed classifier read as an answer.

That rule is about **every git read this gate makes whose answer is an exit status**, not about
`check-ignore` alone. Where a subcommand answers with a status, the gate SHALL read the status that is the
answer and treat every other non-zero as a refusal to answer. Stated over the class because it arrived
through a second door: `ls-files --error-unmatch` exits `1` for *this path is not tracked* and `128` for a
directory that is no repository or an index it cannot parse, and a reader that asked only *did git fail*
reported the second as the first — one repair after the sibling rule above was written, and one exit status
outside the split that repair made.

#### Scenario: A tag read that declines is not a snapshot that was never tagged

- **WHEN** the read that decides whether the release tag exists cannot be answered by git — the path is no
  repository, or the ref store cannot be read
- **THEN** the gate refuses as a cannot-judge, rather than reporting the tag absent as a **violation**. This
  read took every failure as *there is no tag*, which is the third door onto the rule stated over the class
  above. `--quiet` is what makes the split exist: measured, a bare `rev-parse --verify` exits `128` both for
  an absent ref and for a directory that is no repository, so the answer and the refusal are one status until
  it is passed
- **PINNED-BY** `a_directory_git_will_not_read_is_not_a_repository_with_no_tag`
- **PINNED-BY** `a_repository_git_can_read_answers_both_ways_about_a_tag`

#### Scenario: A file with special bytes is ignored by tracked repository content

- **WHEN** a tracked `.gitignore` ignores a file whose name git prints quoted
- **THEN** the gate accepts it, because clean is defined by the repository and the same exclusion applies to
  what `cargo publish` would package

#### Scenario: The exclusion classifier cannot run

- **WHEN** `check-ignore` fails rather than reporting no match
- **THEN** the gate refuses as a cannot-judge naming the paths it could not classify, rather than treating an
  unusable classifier as one that found nothing

#### Scenario: The tracking read cannot be made

- **WHEN** `ls-files --error-unmatch` exits for a reason other than the path being untracked — the directory
  is no repository, or its index cannot be parsed
- **THEN** the gate refuses as a cannot-judge saying which status it met, rather than reading it as the
  answer that the path is untracked

### Requirement: The dirty-worktree diagnostic names each path as itself

Where the worktree is not clean, the refusal SHALL name each offending path **as the repository holds it**,
one record per line. `git status` is read with `-z`, so a path carrying non-ASCII bytes arrives unquoted and
the records arrive NUL-separated; the diagnostic SHALL split on that separator rather than interpolating the
stream, and no separator or octal escape SHALL reach the operator.

This render changed twice with nothing observing it — it broke when `-z` was added, since only emptiness was
tested, and was repaired again without either change failing a direction. A direction SHALL use at least two
dirty paths, because with one record a run-together render and a one-per-line render are the same string.

#### Scenario: Two dirty paths, one of them non-ASCII

- **WHEN** the worktree holds two untracked files, one named with non-ASCII bytes
- **THEN** the refusal names both, each on its own line, the non-ASCII one spelled as itself rather than in
  git's quoted form, and carries no record separator
- **PINNED-BY** `the_dirty_worktree_diagnostic_names_each_path_unescaped_and_one_per_line`
