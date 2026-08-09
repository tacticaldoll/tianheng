## MODIFIED Requirements

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
