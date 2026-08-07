## Context

`examples/observer-participant` is an isolated adopter-shaped crate, but the examples gate rewrites its Tianheng
dependency to this local checkout. Its reach test reads the manifest and Rust imports, proving that a crate outside
the family needs only the shell's public exports. It neither downloads nor inspects a published package. Calling
the surface “published” conflates Rust visibility with a release event that has not occurred for 0.5.0.

## Goals / Non-Goals

**Goals:**

- Name the proven property as public/exported reachability on every relevant surface.
- Rename the reach test so CI output states the property it actually verifies.
- Preserve legitimate uses of “published” for crates.io artifacts and publish-source governance.

**Non-Goals:**

- Publish version 0.5.0 or assert package contents.
- Change the observer protocol, prelude exports, example dependency, or reach-test algorithm.
- Add a terminology detector over prose.

## Decisions

### Use `public surface` for the live contract

The canonical requirement, README, test, changelog, and backlog SHALL call the protocol surface public. Where a
sentence describes the action that would make an API reachable, use “export” or “add an export,” not “publish.”

Retaining “published” as aspirational release prose was rejected because the files present the proof as current,
and the gate observes the current checkout. Replacing every repository occurrence was also rejected because
artifact provenance, crates.io state, and publish-source checks genuinely concern publication.

### Keep the existing reaction

Rename `the_participant_reaches_only_the_published_shell` to name the public shell. Keep its dependency census,
forbidden direct-import set, and assertions unchanged. No new guard is needed because the finding is false prose
around an already-reacting property.

## Risks / Trade-offs

- **A later release makes “published” factually true** → The durable contract remains public reachability; the
  example still does not verify registry contents, so `public` stays the more precise word after release.
- **A broad replacement corrupts real publication history** → Edits are limited to observer-protocol claims and
  verified with contextual full-file sweeps.
