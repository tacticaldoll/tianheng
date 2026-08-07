## Why

The observer-participant proof compiles against this checkout's exported API, but its README, test, canonical
specification, changelog, and release-horizon backlog repeatedly call that surface “published.” Version 0.5.0 is
unreleased and the example gate patches dependencies to local source, so the proof establishes public
reachability, not registry publication.

## What Changes

- Use `public` or `exported` for the observer protocol surface exercised from outside the family.
- Rename the participant reach test and diagnostics to the property they actually check.
- Correct every observer-protocol statement in the unreleased changelog and release-horizon backlog while
  preserving legitimate historical/package-publication terminology elsewhere.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `observer-protocol`: state the outside-participant requirement and failure scenario against the public API
  surface, without claiming that the unreleased API has been published.

## Impact

The change touches OpenSpec prose, the observer-participant README and test naming/comments/diagnostics, the
unreleased changelog, and a release-horizon backlog paragraph. It changes no API, import, dependency, package
source, test assertion, law, manifest, package version, reaction, or adopter behavior.
