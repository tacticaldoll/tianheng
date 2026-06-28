# AGENTS.md — 天衡 (Tianheng)

Working agreement for humans and AI agents. `PROJECT.md` is the contract (the *why* and
the invariants); this file is the *how* of contributing. Keep both short.

## Commits & PRs

- **No AI/agent attribution.** Commit messages and PR descriptions must NOT contain a
  `Co-Authored-By: Claude` trailer, a "Generated with Claude Code" footer, a "🤖" line, or
  any other tool-authorship mark. The history records *what changed and why*, not what
  typed it. This is a project rule, not a personal preference.
- **Self-describing style.** A message says what changed and why, in its own words — not
  an issue/PR number as a crutch. A reader should understand the change from the message
  alone.

## Branching and release

`main` is release-only: it carries nothing but linear, non-merge `release: X.Y.Z` snapshot
commits, each tagged `vX.Y.Z`. The fine-grained lifecycle commits (propose / apply / sync /
archive) never land on `main` individually — they collapse through two squash stages on the
way up: a change branch is squash-merged into `release/X.Y.Z`, and that release branch is
squash-merged into `main`.

Both squashes are performed by a GitHub pull request's "Squash and merge", not a local merge.
Strip GitHub's auto-appended `(#N)` from the squash subject (the self-describing-commit rule
above; the `release: X.Y.Z` snapshot subject is its one exception — a release commit's "change"
is the whole tree at that version, and the per-change "why" lives in the squashed change
commits and their PRs). A PR that touches a steward-owned path (`.github/CODEOWNERS`) is merged
by the steward.

Like the self-describing-commit rule above, this is a convention for humans and agents, not a
Tianheng reaction: a branching pattern is not an observable architectural fact, so the drift law
keeps it out of the constitution.

## Self-governance — don't weaken the law to make CI pass

Tianheng governs itself: `crates/tianheng/tests/self_governance.rs` runs Tianheng's own
reaction against the workspace as a `cargo test` gate. Its invariants:

- **Dependency-light core** — `guibiao` depends on `serde_json` only. Heavy
  dependencies (AST/runtime) belong in their own future crates, never the core.
- **Functional core ⊥ imperative shell** (crate-level) — `guibiao` must not depend on
  `tianheng`.

If a change makes this test fail, **fix the change**, not the test. A boundary is altered
only by a deliberate, human-reviewed amendment to `self_governance.rs` — never by quietly
weakening it so CI turns green.

## Versioning — SemVer honesty (the modou lesson)

- Pre-1.0 and at `0.0.x`: **no inter-release compatibility is promised**; any release may
  break. Do not vanity-bump the minor for a non-breaking change.
- Graduate to `0.1.0` only when the public API has settled enough to promise
  `0.1.x`-patch compatibility. After that: non-breaking → patch, breaking → minor.

## Drift law & minimalism (inherited, non-negotiable)

- **No drift type without an observation source; no target or name without a reaction** —
  at module, crate, and dimension granularity. Do not pre-create empty `semantic`/`runtime`
  crates or stub modules; a dimension's crate is born when it is built.
- **Fail loud only on observable misconfiguration.** No defensive over-foolproofing of
  impossible states.

## Outward / irreversible actions — confirm first

Merging to `main`, tagging, publishing to crates.io, force-pushing, and deleting a repo
are confirm-first: get explicit human sign-off even if a permission rule would auto-allow
it. (crates.io publishes are permanent — only yankable, never deletable.) The local
`.claude/settings.local.json` mirrors this with a `permissions.ask` rule on `gh pr merge`.
