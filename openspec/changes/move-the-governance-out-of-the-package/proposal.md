## Why

**The governance apparatus ships to every adopter.** Measured:

```
$ cargo package --list -p tianheng | grep -c '^tests/'
50
```

All 25 top-level reaction targets are in that tarball — including `merge_message.rs`, which judges this
repository's squash messages; `release_coherence.rs`, which judges its `CHANGELOG.md`; and
`dod_coherence.rs`, which judges its `AGENTS.md` against its `ci.yml`. An adopter downloads them and they can
only skip.

**The capability governing them states the criterion they fail.** `rust-self-governance-gates/spec.md:143`:

> `scripts/` and `docs/` alike ship in **zero** packages, which is what makes them self-governance rather than
> product

By its own test, machinery that ships in a package is product. These ship in a package. The filing says
governance, the location says product, and the two have never met.

**This is the physical cause of a naming conflation.** `AGENTS.md` claims that `self_governance.rs` "and
sibling Rust integration tests (`crates/tianheng/tests/*.rs`) run Tianheng's own reactions against the
workspace" — false for 20 of the 25, which reach no shipped API at all. The claim is easy to write because
the twenty sit in the same directory as the five that do. A repository's own law living under a package's
`tests/` lends its name to everything beside it.

**The packaged self-test's subject is mostly skips.** That job exists to catch a fixture-path bug that only
appears from the tarball. Today most of what it runs is governance reactions that detect no workspace and
return. Moving them leaves it running tests that actually exercise the packaged crate.

## What Changes

- **A new workspace member, 繩墨 (`shengmo`) — the inked line**, `publish = false`. A carpenter snaps it to
  mark true; everything is judged against it, and the line is not part of the furniture. Deliberately not an
  astronomical instrument: it is not one of the 三儀 and not product, and the name says so.
- **The law becomes the member's library.** `tianheng_constitution()` is code — the product capability applied
  to its own author — and it currently sits at line 69 of a 668-line test file. It moves to
  `crates/shengmo/src/`, exported, with the tests reduced to the reaction that runs it. `AGENTS.self-law.md`
  is then projected from a library rather than from a private function inside a test binary.
- **The repository reactions move into its `tests/`** — 16 further targets, with the support modules and
  fixtures they own.
- **The prose that restates the law is retired with it.** The rule already exists and already reacts, but only
  for line-comment blocks under `crates/tianheng/src` and only against the shell's allowlist. Measured,
  `PROJECT.md` names every member of `guibiao`'s live allowlist — `serde_json, xuanji, xingbiao` — which is
  the same second source of truth in a file class nothing scans. The reaction stops being scoped to one
  crate's comments and one dimension, and the restated census becomes a pointer to the projection.
- **`crates/tianheng/tests/` keeps what tests the crate**: the cross-dimension conformance matrices, the CLI
  and baseline behaviour, and the adopter-surface compile contract — everything whose subject is the code in
  the tarball.
- Every invocation follows: the Definition of Done, six CI jobs, `scripts/publish.sh`, `scripts/merge-pr.sh`,
  `.github/CODEOWNERS`, and the projection register.
- No license texts in the new member's directory — `cargo publish` never packages it, and CI's `license-files`
  job already encodes that rule for `publish = false` members. Its manifest still inherits
  `license.workspace = true`, so its metadata matches its siblings.

**Not a rename.** The capability and vocabulary rectification is the change that follows this one; it is
written against the new locations rather than the old, so the two do not churn the same files twice.

## Capabilities

### New Capabilities

None. The apparatus moves; what it judges does not change.

### Modified Capabilities

- `rust-self-governance-gates`: a repository reaction SHALL live outside every published package, which is
  what its own text already gives as the criterion for governance rather than product. Its subject moves from
  `crates/tianheng/tests/` to the new member.
- `projection-register`: its requirement counts the correspondence "per blessing call site in Rust tests under
  `crates/tianheng/tests/`", and those call sites move.
- `self-law-projection`: the law is a library rather than a function in a test file, and the reaction
  forbidding a restated allowlist stops being scoped to one crate's comments and one dimension — it reads
  every declared allowlist against every tracked governance document.

`reference-integrity` is **not** listed: it derives workspace members from tracked `crates/<name>/Cargo.toml`
paths, and a new member is a new datum for a rule that does not change.

## Impact

- `Cargo.toml` — one new member in an explicit, glob-free list.
- `crates/shengmo/` — new, unpublished, holding the law and the repository reactions.
- `crates/tianheng/tests/` — 16 targets and their support modules leave; the conformance plumbing stays.
- `.github/workflows/ci.yml`, `AGENTS.md`, `scripts/publish.sh`, `scripts/merge-pr.sh`, `.github/CODEOWNERS`,
  `docs/projection-register.md`, `README.md`.
- **The published `tianheng` crate loses 16 test targets from its tarball.** No API, binary, or behaviour
  changes for an adopter; what leaves is what they could never run.
- No version change.
