## Why

Four judgements answer about something other than what they read. Each was reported by review, and each is
reproduced here rather than reasoned about.

**The publish gate asks `check-ignore` about a path that does not exist.** `ls-files --others` prints a path
with special bytes in git's **quoted** form, and that string is passed to `check-ignore` verbatim. Measured on
a fixture: a file named `ignored-普通`, ignored by a **tracked** `.gitignore`, is listed as
`"ignored-\346\231\256\351\200\232"`, and `check-ignore` returns exit 1 for that literal. The gate then reads
the unshown source as this checkout's and refuses a file the repository itself ignores — the direct opposite
of the scenario `publish-source-integrity` states. And the other direction is worse: with the quoting removed,
`check-ignore` answers **about a different file**, so a path hidden by this clone's own exclude could be
cleared by a tracked pattern matching the quoted spelling.

**A failed classification is read as an answer.** The same function drops `check-ignore`'s non-zero status
with `unwrap_or_default()`, so a classifier that could not run is indistinguishable from one that found
nothing.

**The bound register's package enumeration is not tracked content.** It reads `crates/` from the working
directory and drops failed entries, which `observation-bound-register` already forbids in writing: a listing
that emits some entries and then fails "leaves a short list that reads as authoritative", and every citation
in a package never enumerated is reported as one the harness does not register — a filesystem failure charged
to the register.

**An anchor is invented rather than refused.** `audit_corpus_and_anchor` returns `Result<_, String>` and does
not use it for the one unrepresentable case: with no workspace root, an unresolvable manifest directory, and
an unreadable working directory, it invents `/`. The anchor is baseline identity — every observed file is
labelled relative to it — so an invented one mislabels every finding silently.

## What Changes

- The publish gate reads and classifies paths with `-z`, so the bytes it asks about are the bytes it was
  given: `ls-files -z`, `status -z`, and `check-ignore -z -v --stdin`.
- `check-ignore`'s failure becomes a **cannot-judge** naming what could not be classified, rather than an
  empty classification.
- The package enumeration comes from tracked content, and a failed enumeration refuses rather than shortening.
- The anchor uses the error channel already in its signature.
- The last `filter_map(entry.ok())` in the tree — a direction asserting **no** leftover sibling file — stops
  dropping the entries it is counting.

## Capabilities

### Modified Capabilities

- `publish-source-integrity`: its stated scenario — a file ignored by tracked repository content is clean —
  gains the case that broke it, a path git prints quoted. Its subject covers `scripts/publish.sh` and the
  gate's judgement module.
- `observation-bound-register`: the package enumeration requirement gains the scenario that holds it.

`rust-repository-reactions` claims `crates/kanhe/**` and is accounted for here: the reactions change, their
requirements do not. `violation-baseline` and `cli-check-runner` both claim
`crates/tianheng/tests/baseline_cli.rs`, whose direction is repaired without either requirement changing. `release-coherence` claims `CHANGELOG.md`, which records this.

## Impact

- `crates/kanhe/src/publish_source_gate.rs`, `crates/kanhe/tests/bound_register.rs`,
  `crates/xingbiao/src/lib.rs`, `crates/tianheng/tests/baseline_cli.rs`.
- No public API changes — `audit_corpus_and_anchor` already returns `Result`. No version change.
