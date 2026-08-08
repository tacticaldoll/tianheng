# Design

## Where the reaction lives, and why shell is right here

The census that motivated this change splits `scripts/` into three populations by **subject**:

| population | subject | lines | is shell right? |
|---|---|---|---|
| text / tree hygiene | tracked bytes | 1,338 (35%) | **yes** — git is the interface, regex density 5–25 sites is proportionate |
| release ritual | git state and manifests at a moment | 288 (8%) | **yes** |
| Rust semantics via `cargo` | the workspace's own typed values, reached by parsing `cargo` stdout | 1,670 (44%) | **no** — this is the recorded debt |

This reaction's subject is **`CHANGELOG.md` text**. It is population 1, so it belongs in
`check_release_coherence.sh` and the census argues *for* shell here rather than against it. No `cargo`
invocation is added and no `cargo` output is parsed.

## The shape read from the document

`changelog_sections()` already emits a tab-separated shape (`SECTION` / `HEADING` / `BREAKING`). It gains
one record kind, `CITATION`, carrying the section, the heading in force, and the machinery path named:

```
SECTION   ## [Unreleased]
HEADING   ## [Unreleased]   Fixed
CITATION  ## [Unreleased]   Fixed   scripts/check_publish_source.sh
```

A citation is attributed to the list item it appears in and the item to the `### ` heading above it,
both of which are the document's grammar rather than its claims. A citation before any `###` heading is
attributed to the section with an empty heading, which is adopter-facing by the rule below.

## What counts as this repository's own machinery

Two forms are cited in the current document and both must be recognised:

- a path token under `scripts/` — `scripts/check_publish_source.sh`, `scripts/lib/capture.sh`
- a bare basename that names a tracked file under `scripts/` — `check_pin_bites.sh`,
  `test_example_suite.sh`

The second form is decided against `git ls-files scripts/`, **not** against a hand-kept list of gate
names. A list beside its enumerator lets a new script be added and never measured; the enumerator is the
only authority. This is the same discipline `gate-shape-contract` already applies to the gate surface.

Recognition is by **token**, so a basename appearing as ordinary prose is not a citation. The current
document names every one of them in backticks; a bare unquoted `check_pin_bites.sh` in prose is a
declared limit rather than a silently widened matcher.

## Which headings are adopter-facing

Every `### ` heading in a release section **except `Self-governance`**. Defining the exception rather
than enumerating the adopter set is what keeps the rule sound when a future section grows a heading
nobody anticipated: an unanticipated heading is adopter-facing, which is the direction that reacts.

## Scope: `[Unreleased]` only

A dated `## [X.Y.Z] - DATE` section is a record of what was true at that release. Rewriting `[0.4.0]`'s
five entries to satisfy a rule written after 0.4.0 shipped would falsify the record — the same reason
`docs/history/` is left alone. A section becomes record by being dated, so the rule needs no list of
exempt versions.

This is a deliberate non-assertion over dated sections and is declared as an observation bound, not left
to be inferred from the implementation.

## The residual, and why it is a bound rather than a wider matcher

An entry whose subject is this repository's own governance but which names no `scripts/` path is
invisible. Reaching it needs a judgement over the entry's *subject* — the prose instrument `AGENTS.md`
records as designed, measured three times and rejected. Widening the matcher toward it (heading
keywords, phrase lists) would trade a declared, bounded blindness for an undeclared false-positive
surface, which is the wrong direction under the Core Contract.

Declared, not approximated.

## Why the falsified draft is kept in the proposal

The first rule drafted was *cite no path that ships in no package*. It was killed by enumerating the
real citation population: all 15 cited paths ship in no package, including `COOKBOOK.md` and `docs/*.md`
which are adopter surface. Recording it costs three lines and stops the next author re-deriving a proxy
that measurement has already refused.

## Twin matrix

`gate-shape-contract` requires the failure matrix, so `test_release_coherence.sh` gains directions
asserting **exit codes**, not messages:

- an adopter heading naming a `scripts/` path → exit 1
- the same entry moved under `### Self-governance` → exit 0
- an adopter heading naming a bare `check_*.sh` basename that `git ls-files scripts/` resolves → exit 1
- a bare basename that resolves to **no** tracked file under `scripts/` → exit 0, so the rule is about
  the enumerator rather than about the `check_` prefix
- a **dated** section naming a `scripts/` path → exit 0, pinning the scope decision above
- the citation shape unreadable → exit 2 rather than a clean report over nothing
