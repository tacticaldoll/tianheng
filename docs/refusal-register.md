# Refusal register

Every refusal site in this repository, and what holds it. A site is registered by being constructed through `refusal::violation_at` or `refusal::cannot_judge_at`, and **held** by a direction calling `refusal::expect` with the same identity, compared by running rather than by reading a message.

**What this document does not claim.** `refusal::violation` and `refusal::cannot_judge` construct a refusal carrying no site identity — `Site::OutsideRegister` — so this register does not see them. Its corpus, `crates/kanhe/src`, holds **none** of them, which is the figure beside *carry no identity at all* above. The test targets do hold them, and this corpus excludes those: none is registered, held, or declared here, and whether any should have taken an identity is a judgement this document does not make. **No count of them is given**, and two different things stand in the way. One is a floor: a name taken by reference reads the same whether it is the constructor or a local that shares its spelling, and no reader of text can decide which. The other is a debt: whether an occurrence is inside a closure's parameter list is a **position**, and this reader answers it by counting the pipes standing before the name on its line — which is right for every shape the corpus holds and is an approximation of the position rather than the position itself. The first cannot be closed; the second can.

A site that no direction holds is **declared unheld**, with why, an owner and a tracker, in the table this register reads. There is no third state among *registered* sites: one is held or declared, and the register refuses anything else.

Generated from `crates/kanhe/src/**.rs` by `crates/kanhe/tests/refusal_register.rs`. **Do not edit by hand** — regenerate with `BLESS=1 TIANHENG_WORKSPACE_TESTS=1 cargo test -p kanhe --test refusal_register`. A stale projection fails that gate.

**14 of 126 refusal sites are declared unheld.** 0 carry no identity at all, which is a state this repository does not keep — the register refuses a non-zero figure here.

## Declared unheld

### `publish-source-integrity#release-tag-unreadable`

- because measured, not assumed: every ref-store perturbation a fixture can build answers `1`, which is              this read's ANSWER — an unreadable `refs/tags` and a `refs/tags` replaced by a file both exit              `1`, and in that state `rev-parse HEAD` fails first at `128`, so the judgement never reaches              this arm. The classifier itself IS observed, against a directory that is no repository; what no              fixture can build is a repository whose earlier reads succeed and whose tag read declines
- owner: Engine
- tracked by `BACKLOG.md` — *a refusal reachable only by a broken tool is not observed*

### `release-coherence#directory-entry-unreadable`

- because a directory entry that errors while the directory itself enumerates is produced by the filesystem between two syscalls, and a fixture would have to hold that window open
- owner: Engine
- tracked by `BACKLOG.md` — *a refusal reachable only by a broken tool is not observed*

### `release-coherence#metadata-has-no-workspace-root`

- because cargo emits `workspace_root` for every workspace it can load, so reaching this means replacing cargo with something that answers differently — the direction would then observe the replacement
- owner: Engine
- tracked by `BACKLOG.md` — *a refusal reachable only by a broken tool is not observed*

### `release-coherence#metadata-package-has-no-manifest-path`

- because same corpus as its sibling above: a package cargo reports without a manifest path is not a shape cargo produces, so the fixture is a fake cargo
- owner: Engine
- tracked by `BACKLOG.md` — *a refusal reachable only by a broken tool is not observed*

### `release-coherence#member-manifest-outside-workspace-root`

- because cargo resolves member paths against the root it reports, so a member outside it is a disagreement inside cargo rather than a shape a manifest can carry
- owner: Engine
- tracked by `BACKLOG.md` — *a refusal reachable only by a broken tool is not observed*

### `release-coherence#scripts-not-enumerable`

- because `git ls-files` failing while the same process already read the repository is a git failure mid-run, and simulating it means putting a fake git on the path
- owner: Engine
- tracked by `BACKLOG.md` — *a refusal reachable only by a broken tool is not observed*

### `release-coherence#cargo-metadata-unrunnable`

- because cargo absent from the path of a process cargo is running
- owner: Engine
- tracked by `BACKLOG.md` — *a refusal reachable only by a broken tool is not observed*

### `release-coherence#cargo-metadata-not-json`

- because cargo emitting something that is not JSON, which is a fake cargo by construction
- owner: Engine
- tracked by `BACKLOG.md` — *a refusal reachable only by a broken tool is not observed*

### `publish-source-integrity#repository-root-is-not-a-worktree`

- because git failing to *start*, as against git running and refusing — the refusing half is observed. Reaching this means a machine without git, and a fixture that removes git from the path tests the path manipulation
- owner: Engine
- tracked by `BACKLOG.md` — *a refusal reachable only by a broken tool is not observed*

### `publish-source-integrity#ssh-keygen-unavailable`

- because the same shape one tool over, and with the same fixture: `ssh-keygen` removed from the path
- owner: Engine
- tracked by `BACKLOG.md` — *a refusal reachable only by a broken tool is not observed*

### `publish-source-integrity#signature-mechanism-round-trip-failed`

- because the gate's own probe signs and verifies a payload before trusting its verdict; reaching this means an `ssh-keygen` that signs and then fails to verify its own signature
- owner: Engine
- tracked by `BACKLOG.md` — *a refusal reachable only by a broken tool is not observed*

### `publish-source-integrity#signature-block-unreadable`

- because git failing to read a tag object it has already resolved, mid-run
- owner: Engine
- tracked by `BACKLOG.md` — *a refusal reachable only by a broken tool is not observed*

### `publish-source-integrity#signature-is-not-the-tag-object-suffix`

- because a tag object whose signature block is not its own suffix is one git does not write, so the fixture would be a hand-assembled object testing this reader against that assembly
- owner: Engine
- tracked by `BACKLOG.md` — *a refusal reachable only by a broken tool is not observed*

### `publish-source-integrity#signature-unwritable`

- because a scratch directory this process created and cannot write to. Running as root defeats the fixture outright, which makes the direction's own result depend on who runs it
- owner: Engine
- tracked by `BACKLOG.md` — *a refusal reachable only by a broken tool is not observed*

## Held

### `publish-source-integrity#exclusion-classifier-cannot-run`

- produced in `crates/kanhe/src/publish_source_gate.rs`
- observed by `crates/kanhe/tests/publish_source.rs`

### `publish-source-integrity#head-is-not-the-release-snapshot`

- produced in `crates/kanhe/src/publish_source_gate.rs`
- observed by `crates/kanhe/tests/publish_source.rs`

### `publish-source-integrity#head-is-not-the-tip-of-main`

- produced in `crates/kanhe/src/publish_source_gate.rs`
- observed by `crates/kanhe/tests/publish_source.rs`

### `publish-source-integrity#head-subject-unreadable`

- produced in `crates/kanhe/src/publish_source_gate.rs`
- observed by `crates/kanhe/tests/publish_source.rs`

### `publish-source-integrity#release-tag-absent`

- produced in `crates/kanhe/src/publish_source_gate.rs`
- observed by `crates/kanhe/tests/publish_source.rs`

### `publish-source-integrity#release-tag-carries-no-signature`

- produced in `crates/kanhe/src/publish_source_gate.rs`
- observed by `crates/kanhe/tests/publish_source.rs`

### `publish-source-integrity#release-tag-does-not-name-head`

- produced in `crates/kanhe/src/publish_source_gate.rs`
- observed by `crates/kanhe/tests/publish_source.rs`

### `publish-source-integrity#release-tag-is-lightweight`

- produced in `crates/kanhe/src/publish_source_gate.rs`
- observed by `crates/kanhe/tests/publish_source.rs`

### `publish-source-integrity#remote-has-no-main`

- produced in `crates/kanhe/src/publish_source_gate.rs`
- observed by `crates/kanhe/tests/publish_source.rs`

### `publish-source-integrity#remote-main-unreadable`

- produced in `crates/kanhe/src/publish_source_gate.rs`
- observed by `crates/kanhe/tests/publish_source.rs`

### `publish-source-integrity#repository-root-has-no-manifest`

- produced in `crates/kanhe/src/publish_source_gate.rs`
- observed by `crates/kanhe/tests/publish_source.rs`

### `publish-source-integrity#signature-armour-unverifiable`

- produced in `crates/kanhe/src/publish_source_gate.rs`
- observed by `crates/kanhe/tests/publish_source.rs`

### `publish-source-integrity#signature-does-not-verify`

- produced in `crates/kanhe/src/publish_source_gate.rs`
- observed by `crates/kanhe/tests/publish_source.rs`

### `publish-source-integrity#signature-scratch-unclaimable`

- produced in `crates/kanhe/src/publish_source_gate.rs`
- observed by `crates/kanhe/tests/publish_source.rs`

### `publish-source-integrity#signature-verifier-reached-no-verdict`

- produced in `crates/kanhe/src/publish_source_gate.rs`
- observed by `crates/kanhe/src/tests/publish_source_gate.rs`

### `publish-source-integrity#tag-commit-unresolvable`

- produced in `crates/kanhe/src/publish_source_gate.rs`
- observed by `crates/kanhe/tests/publish_source.rs`

### `publish-source-integrity#tag-object-unreadable`

- produced in `crates/kanhe/src/publish_source_gate.rs`
- observed by `crates/kanhe/tests/publish_source.rs`

### `publish-source-integrity#tracking-question-unaskable`

- produced in `crates/kanhe/src/publish_source_gate.rs`
- observed by `crates/kanhe/tests/publish_source.rs`

### `publish-source-integrity#workspace-manifest-unreadable`

- produced in `crates/kanhe/src/publish_source_gate.rs`
- observed by `crates/kanhe/tests/publish_source.rs`

### `publish-source-integrity#workspace-version-absent`

- produced in `crates/kanhe/src/publish_source_gate.rs`
- observed by `crates/kanhe/tests/publish_source.rs`

### `publish-source-integrity#workspace-version-malformed`

- produced in `crates/kanhe/src/publish_source_gate.rs`
- observed by `crates/kanhe/tests/publish_source.rs`

### `publish-source-integrity#workspace-version-unreadable`

- produced in `crates/kanhe/src/publish_source_gate.rs`
- observed by `crates/kanhe/tests/publish_source.rs`

### `publish-source-integrity#worktree-hides-untracked-files`

- produced in `crates/kanhe/src/publish_source_gate.rs`
- observed by `crates/kanhe/tests/publish_source.rs`

### `publish-source-integrity#worktree-is-not-clean`

- produced in `crates/kanhe/src/publish_source_gate.rs`
- observed by `crates/kanhe/tests/publish_source.rs`

### `publish-source-integrity#worktree-state-unreadable`

- produced in `crates/kanhe/src/publish_source_gate.rs`
- observed by `crates/kanhe/tests/publish_source.rs`

### `release-coherence#adopter-entry-names-own-machinery`

- produced in `crates/kanhe/src/release_coherence_gate.rs`
- observed by `crates/kanhe/tests/release_coherence.rs`

### `release-coherence#breaking-without-migration-section`

- produced in `crates/kanhe/src/release_coherence_gate.rs`
- observed by `crates/kanhe/tests/release_coherence.rs`

### `release-coherence#cargo-metadata-failed`

- produced in `crates/kanhe/src/release_coherence_gate.rs`
- observed by `crates/kanhe/tests/release_coherence.rs`

### `release-coherence#changelog-or-manifest-unreadable`

- produced in `crates/kanhe/src/release_coherence_gate.rs`
- observed by `crates/kanhe/tests/release_coherence.rs`

### `release-coherence#changelog-section-repeats-a-heading`

- produced in `crates/kanhe/src/release_coherence_gate.rs`
- observed by `crates/kanhe/tests/release_coherence.rs`

### `release-coherence#crate-manifest-unreadable`

- produced in `crates/kanhe/src/release_coherence_gate.rs`
- observed by `crates/kanhe/tests/release_coherence.rs`

### `release-coherence#crate-package-name-absent`

- produced in `crates/kanhe/src/release_coherence_gate.rs`
- observed by `crates/kanhe/tests/release_coherence.rs`

### `release-coherence#crate-package-name-unreadable`

- produced in `crates/kanhe/src/release_coherence_gate.rs`
- observed by `crates/kanhe/tests/release_coherence.rs`

### `release-coherence#dated-release-notes-missing`

- produced in `crates/kanhe/src/release_coherence_gate.rs`
- observed by `crates/kanhe/tests/release_coherence.rs`

### `release-coherence#dependency-declares-several-paths`

- produced in `crates/kanhe/src/release_coherence_gate.rs`
- observed by `crates/kanhe/src/tests/release_coherence_gate.rs`

### `release-coherence#dependency-path-unreadable`

- produced in `crates/kanhe/src/release_coherence_gate.rs`
- observed by `crates/kanhe/src/tests/release_coherence_gate.rs`

### `release-coherence#directory-listing-unreadable`

- produced in `crates/kanhe/src/release_coherence_gate.rs`
- observed by `crates/kanhe/tests/release_coherence.rs`

### `release-coherence#directory-not-enumerable`

- produced in `crates/kanhe/src/release_coherence_gate.rs`
- observed by `crates/kanhe/tests/release_coherence.rs`

### `release-coherence#example-declares-several-packages`

- produced in `crates/kanhe/src/release_coherence_gate.rs`
- observed by `crates/kanhe/tests/release_coherence.rs`

### `release-coherence#example-declares-several-pins`

- produced in `crates/kanhe/src/release_coherence_gate.rs`
- observed by `crates/kanhe/tests/release_coherence.rs`

### `release-coherence#example-dependency-key-unreadable`

- produced in `crates/kanhe/src/release_coherence_gate.rs`
- observed by `crates/kanhe/src/tests/release_coherence_gate.rs`

### `release-coherence#example-manifest-unreadable`

- produced in `crates/kanhe/src/release_coherence_gate.rs`
- observed by `crates/kanhe/tests/release_coherence.rs`

### `release-coherence#example-package-value-unreadable`

- produced in `crates/kanhe/src/release_coherence_gate.rs`
- observed by `crates/kanhe/src/tests/release_coherence_gate.rs, crates/kanhe/tests/release_coherence.rs`

### `release-coherence#example-pin-absent`

- produced in `crates/kanhe/src/release_coherence_gate.rs`
- observed by `crates/kanhe/tests/release_coherence.rs`

### `release-coherence#example-pin-disagrees`

- produced in `crates/kanhe/src/release_coherence_gate.rs`
- observed by `crates/kanhe/tests/release_coherence.rs`

### `release-coherence#example-pin-unreadable`

- produced in `crates/kanhe/src/release_coherence_gate.rs`
- observed by `crates/kanhe/tests/release_coherence.rs`

### `release-coherence#example-requires-no-family-crate`

- produced in `crates/kanhe/src/release_coherence_gate.rs`
- observed by `crates/kanhe/tests/release_coherence.rs`

### `release-coherence#git-unrunnable`

- produced in `crates/kanhe/src/release_coherence_gate.rs`
- observed by `crates/kanhe/tests/release_coherence.rs`

### `release-coherence#internal-pin-absent`

- produced in `crates/kanhe/src/release_coherence_gate.rs`
- observed by `crates/kanhe/tests/release_coherence.rs`

### `release-coherence#internal-pin-disagrees`

- produced in `crates/kanhe/src/release_coherence_gate.rs`
- observed by `crates/kanhe/src/tests/release_coherence_gate.rs, crates/kanhe/tests/release_coherence.rs`

### `release-coherence#internal-pin-several`

- produced in `crates/kanhe/src/release_coherence_gate.rs`
- observed by `crates/kanhe/src/tests/release_coherence_gate.rs`

### `release-coherence#internal-pin-unreadable`

- produced in `crates/kanhe/src/release_coherence_gate.rs`
- observed by `crates/kanhe/src/tests/release_coherence_gate.rs`

### `release-coherence#lock-missing-workspace-package`

- produced in `crates/kanhe/src/release_coherence_gate.rs`
- observed by `crates/kanhe/tests/release_coherence.rs`

### `release-coherence#lock-package-name-unreadable`

- produced in `crates/kanhe/src/release_coherence_gate.rs`
- observed by `crates/kanhe/tests/release_coherence.rs`

### `release-coherence#lock-package-version-disagrees`

- produced in `crates/kanhe/src/release_coherence_gate.rs`
- observed by `crates/kanhe/tests/release_coherence.rs`

### `release-coherence#lock-several-sourceless-entries`

- produced in `crates/kanhe/src/release_coherence_gate.rs`
- observed by `crates/kanhe/tests/release_coherence.rs`

### `release-coherence#lock-version-unreadable`

- produced in `crates/kanhe/src/release_coherence_gate.rs`
- observed by `crates/kanhe/tests/release_coherence.rs`

### `release-coherence#member-does-not-inherit-workspace-version`

- produced in `crates/kanhe/src/release_coherence_gate.rs`
- observed by `crates/kanhe/tests/release_coherence.rs`

### `release-coherence#no-crate-manifests-found`

- produced in `crates/kanhe/src/release_coherence_gate.rs`
- observed by `crates/kanhe/tests/release_coherence.rs`

### `release-coherence#no-example-manifests-found`

- produced in `crates/kanhe/src/release_coherence_gate.rs`
- observed by `crates/kanhe/tests/release_coherence.rs`

### `release-coherence#no-internal-path-dependency-found`

- produced in `crates/kanhe/src/release_coherence_gate.rs`
- observed by `crates/kanhe/tests/release_coherence.rs`

### `release-coherence#no-tracked-file-for-any-member`

- produced in `crates/kanhe/src/release_coherence_gate.rs`
- observed by `crates/kanhe/tests/release_coherence.rs`

### `release-coherence#release-comparison-link-wrong`

- produced in `crates/kanhe/src/release_coherence_gate.rs`
- observed by `crates/kanhe/tests/release_coherence.rs`

### `release-coherence#release-date-disagrees-with-its-commit`

- produced in `crates/kanhe/src/release_coherence_gate.rs`
- observed by `crates/kanhe/tests/release_coherence.rs`

### `release-coherence#release-history-shallow`

- produced in `crates/kanhe/src/release_coherence_gate.rs`
- observed by `crates/kanhe/tests/release_coherence.rs`

### `release-coherence#release-history-subject-malformed`

- produced in `crates/kanhe/src/release_coherence_gate.rs`
- observed by `crates/kanhe/tests/release_coherence.rs`

### `release-coherence#release-history-unreadable`

- produced in `crates/kanhe/src/release_coherence_gate.rs`
- observed by `crates/kanhe/tests/release_coherence.rs`

### `release-coherence#release-history-version-malformed`

- produced in `crates/kanhe/src/release_coherence_gate.rs`
- observed by `crates/kanhe/tests/release_coherence.rs`

### `release-coherence#release-snapshot-version-disagrees`

- produced in `crates/kanhe/src/release_coherence_gate.rs`
- observed by `crates/kanhe/tests/release_coherence.rs`

### `release-coherence#repository-root-has-no-changelog`

- produced in `crates/kanhe/src/release_coherence_gate.rs`
- observed by `crates/kanhe/tests/release_coherence.rs`

### `release-coherence#repository-root-has-no-manifest`

- produced in `crates/kanhe/src/release_coherence_gate.rs`
- observed by `crates/kanhe/tests/release_coherence.rs`

### `release-coherence#unreleased-comparison-link-wrong`

- produced in `crates/kanhe/src/release_coherence_gate.rs`
- observed by `crates/kanhe/tests/release_coherence.rs`

### `release-coherence#unreleased-has-no-adopter-narrative`

- produced in `crates/kanhe/src/release_coherence_gate.rs`
- observed by `crates/kanhe/tests/release_coherence.rs`

### `release-coherence#unreleased-not-empty-in-state`

- produced in `crates/kanhe/src/release_coherence_gate.rs`
- observed by `crates/kanhe/tests/release_coherence.rs`

### `release-coherence#unreleased-section-not-exactly-one`

- produced in `crates/kanhe/src/release_coherence_gate.rs`
- observed by `crates/kanhe/tests/release_coherence.rs`

### `release-coherence#workspace-version-absent`

- produced in `crates/kanhe/src/release_coherence_gate.rs`
- observed by `crates/kanhe/tests/release_coherence.rs`

### `release-coherence#workspace-version-behind-latest-release`

- produced in `crates/kanhe/src/release_coherence_gate.rs`
- observed by `crates/kanhe/tests/release_coherence.rs`

### `release-coherence#workspace-version-malformed`

- produced in `crates/kanhe/src/release_coherence_gate.rs`
- observed by `crates/kanhe/tests/release_coherence.rs`

### `release-coherence#workspace-version-unreadable`

- produced in `crates/kanhe/src/release_coherence_gate.rs`
- observed by `crates/kanhe/tests/release_coherence.rs`

### `repository-checks#admitted-types-clause-names-no-type`

- produced in `crates/kanhe/src/merge_message_gate.rs`
- observed by `crates/kanhe/tests/merge_message.rs`

### `repository-checks#backticks-unpaired`

- produced in `crates/kanhe/src/reading.rs`
- observed by `crates/kanhe/src/tests/reading.rs`

### `repository-checks#capability-declares-no-subject`

- produced in `crates/kanhe/src/capability_subjects.rs`
- observed by `crates/kanhe/src/tests/capability_subjects.rs`

### `repository-checks#capability-declares-several-subjects`

- produced in `crates/kanhe/src/capability_subjects.rs`
- observed by `crates/kanhe/src/tests/capability_subjects.rs`

### `repository-checks#capability-subject-bullet-unreadable`

- produced in `crates/kanhe/src/capability_subjects.rs`
- observed by `crates/kanhe/src/tests/capability_subjects.rs`

### `repository-checks#capability-subject-glob-matches-nothing`

- produced in `crates/kanhe/src/capability_subjects.rs`
- observed by `crates/kanhe/src/tests/capability_subjects.rs`

### `repository-checks#capability-subject-glob-unresolvable`

- produced in `crates/kanhe/src/capability_subjects.rs`
- observed by `crates/kanhe/src/tests/capability_subjects.rs`

### `repository-checks#capability-subject-lists-no-glob`

- produced in `crates/kanhe/src/capability_subjects.rs`
- observed by `crates/kanhe/src/tests/capability_subjects.rs`

### `repository-checks#census-document-unreadable`

- produced in `crates/kanhe/src/census.rs`
- observed by `crates/kanhe/tests/census.rs`

### `repository-checks#census-figure-disagrees`

- produced in `crates/kanhe/src/census.rs`
- observed by `crates/kanhe/tests/census.rs`

### `repository-checks#census-figure-unreadable`

- produced in `crates/kanhe/src/census.rs`
- observed by `crates/kanhe/tests/census.rs`

### `repository-checks#census-record-boundary-undecided`

- produced in `crates/kanhe/src/census.rs`
- observed by `crates/kanhe/tests/census.rs`

### `repository-checks#change-touches-a-governed-path-unaccounted`

- produced in `crates/kanhe/src/capability_subjects.rs`
- observed by `crates/kanhe/src/tests/capability_subjects.rs`

### `repository-checks#citation-names-a-gate-registered-several-times`

- produced in `crates/kanhe/src/gate_identity.rs`
- observed by `crates/kanhe/src/tests/gate_identity.rs`

### `repository-checks#citation-names-an-unregistered-gate`

- produced in `crates/kanhe/src/gate_identity.rs`
- observed by `crates/kanhe/src/tests/gate_identity.rs`

### `repository-checks#citation-names-no-test-target`

- produced in `crates/kanhe/src/gate_identity.rs`
- observed by `crates/kanhe/src/tests/gate_identity.rs`

### `repository-checks#citation-target-listing-unreadable`

- produced in `crates/kanhe/src/gate_identity.rs`
- observed by `crates/kanhe/src/tests/gate_identity.rs`

### `repository-checks#date-names-no-day`

- produced in `crates/kanhe/src/reading.rs`
- observed by `crates/kanhe/src/tests/reading.rs`

### `repository-checks#date-names-no-month`

- produced in `crates/kanhe/src/reading.rs`
- observed by `crates/kanhe/src/tests/reading.rs`

### `repository-checks#date-not-the-declared-shape`

- produced in `crates/kanhe/src/reading.rs`
- observed by `crates/kanhe/src/tests/reading.rs`

### `repository-checks#fields-miscounted`

- produced in `crates/kanhe/src/reading.rs`
- observed by `crates/kanhe/src/tests/reading.rs`

### `repository-checks#squash-body-is-a-bare-commit-list`

- produced in `crates/kanhe/src/merge_message_gate.rs`
- observed by `crates/kanhe/tests/merge_message.rs`

### `repository-checks#squash-body-is-empty`

- produced in `crates/kanhe/src/merge_message_gate.rs`
- observed by `crates/kanhe/tests/merge_message.rs`

### `repository-checks#squash-breaking-without-a-migration-footer`

- produced in `crates/kanhe/src/merge_message_gate.rs`
- observed by `crates/kanhe/tests/merge_message.rs`

### `repository-checks#squash-commits-unavailable`

- produced in `crates/kanhe/src/merge_message_gate.rs`
- observed by `crates/kanhe/tests/merge_message.rs`

### `repository-checks#squash-message-carries-an-attribution`

- produced in `crates/kanhe/src/merge_message_gate.rs`
- observed by `crates/kanhe/tests/merge_message.rs`

### `repository-checks#squash-subject-carries-a-serial`

- produced in `crates/kanhe/src/merge_message_gate.rs`
- observed by `crates/kanhe/tests/merge_message.rs`

### `repository-checks#squash-subject-is-not-conventional`

- produced in `crates/kanhe/src/merge_message_gate.rs`
- observed by `crates/kanhe/tests/merge_message.rs`

### `repository-checks#squash-subject-is-not-the-title`

- produced in `crates/kanhe/src/merge_message_gate.rs`
- observed by `crates/kanhe/tests/merge_message.rs`

### `repository-checks#squash-title-unavailable`

- produced in `crates/kanhe/src/merge_message_gate.rs`
- observed by `crates/kanhe/tests/merge_message.rs`

### `repository-checks#the-only-found-none`

- produced in `crates/kanhe/src/selection.rs`
- observed by `crates/kanhe/src/tests/selection.rs, crates/kanhe/src/tests/wrapper_parser.rs`

### `repository-checks#the-only-found-several`

- produced in `crates/kanhe/src/selection.rs`
- observed by `crates/kanhe/src/tests/selection.rs, crates/kanhe/src/tests/wrapper_parser.rs`

### `repository-checks#wrapper-cites-no-gate`

- produced in `crates/kanhe/src/gate_identity.rs`
- observed by `crates/kanhe/src/tests/gate_identity.rs`
