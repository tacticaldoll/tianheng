## Why

The bound register enters the judged repository inside the same command substitution as its census `grep`.
Both a failed `cd` and grep's ordinary no-match result exit 1, so a repository that disappears before the census
scan is treated as containing no written census and can be reported clean.

## What Changes

- Enter the repository in a separately checked step before scanning tracked Markdown for a written census.
- Report a directory-transition failure as exit 2 cannot-judge rather than grep's exit 1 no-match.
- Add a failure-matrix direction that removes the fixture at the exact census boundary.

## Capabilities

### Modified Capabilities

- `observation-bound-register`: the census direction distinguishes an unavailable repository from an empty match.

## Impact

The repository gate becomes fail-loud for an input it previously misreported as judged. Published crates,
manifests, package versions, and adopter APIs are unchanged.
