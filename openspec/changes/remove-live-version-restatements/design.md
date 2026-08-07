## Context

The repository now governs version prose: immutable history, migration targets, and active release planning may
name versions; long-lived live documentation does not restate a current or prospective version. Example manifests
are deliberately adopter-facing mutable surfaces and are checked by release coherence, but nearby comments and
README snippets repeat their values without owning their updates.

## Goals / Non-Goals

**Goals:**

- Make example manifests the only mutable source for their published-version requirements.
- Preserve useful dependency-role explanations without numeric restatements.
- Keep `[Unreleased]` free to communicate the intended release before the mechanical version bump.
- Correct the observer-participant surface claim.

**Non-Goals:**

- Bump workspace or example versions during development.
- Remove immutable historical, migration, fixture, or provenance versions.
- Add a prose-number detector; the governance explicitly rejects that instrument.

## Decisions

### Link or name the manifest requirement

README dependency snippets that only duplicate the adjacent `Cargo.toml` are replaced by links to that manifest
and prose describing dependency roles. Comments say "manifest requirement" or "published-version requirement"
instead of copying the value. The actual requirement remains actionable and release-coherence continues to react
when it no longer accepts the workspace version.

### Keep intended release names in `[Unreleased]`

The changelog is the active adopter-facing release-planning surface. It may say `0.5.0` while the development
workspace remains on the last released version; version preparation later moves manifests, lock entries, and the
dated changelog section together.

### Preserve anchored history

A release number that identifies a shipped false negative, publish provenance, a migration boundary, or a test
fixture remains. Removing those would erase the subject rather than eliminate drift.

## Risks / Trade-offs

- A reader must follow the local manifest link for the exact current requirement. That indirection is deliberate:
  it reaches the mutable source that release coherence checks.
- The prose rule remains a contribution discipline rather than a heuristic detector, because version-looking
  literals have too many legitimate historical and fixture meanings.
