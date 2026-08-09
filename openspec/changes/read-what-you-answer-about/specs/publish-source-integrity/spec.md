## ADDED Requirements

### Requirement: A path the gate classifies SHALL be the path it was given

Every path the cleanliness judgement reads, compares, or asks git about SHALL be carried as raw bytes, using
git's `-z` form for `ls-files`, `status`, and `check-ignore`. Git prints a path containing special or
non-ASCII bytes in a **quoted** form, and a quoted spelling is a different string: asking `check-ignore` about
it asks about a file that does not exist.

Both directions follow from that one substitution, and both were measured on a fixture rather than reasoned
about. A file named `ignored-普通`, ignored by a **tracked** `.gitignore`, is listed as
`"ignored-\346\231\256\351\200\232"`; `check-ignore` returns exit 1 for that literal, the source goes unshown,
and the gate refuses a file the repository itself ignores. Strip the quoting instead and `check-ignore`
answers about a *different* path — so a file hidden by this clone's own exclude could be cleared by a tracked
pattern that happens to match the quoted spelling.

A classification that could not be produced SHALL be a cannot-judge naming what went unclassified, never an
empty classification. `check-ignore` exiting non-zero because it could not run is not the same fact as
`check-ignore` matching nothing, and treating them alike lets a failed classifier read as an answer.

#### Scenario: A file with special bytes is ignored by tracked repository content

- **WHEN** a tracked `.gitignore` ignores a file whose name git prints quoted
- **THEN** the gate accepts it, because clean is defined by the repository and the same exclusion applies to
  what `cargo publish` would package

#### Scenario: The exclusion classifier cannot run

- **WHEN** `check-ignore` fails rather than reporting no match
- **THEN** the gate refuses as a cannot-judge naming the paths it could not classify, rather than treating an
  unusable classifier as one that found nothing
