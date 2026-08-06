## Why

Six defects across two independent reviews, all one shape: **the corpus was taken to be the whole blob when the
property was about a distinguished part of it.**

| site | what satisfied the property | measured |
| --- | --- | --- |
| `projection_register.rs` holder check | a bare **comment** mentioning the call | a `// … assert_projection_matches( …` line added to an unrelated file made it register as a holder |
| `projection_register.rs` reachability | an **HTML comment**, invisible to a reader | `<!-- docs/x.md -->` as the only mention satisfied "a reader can find this" |
| `gate_shape_contract.rs` Definition-of-Done membership | a path **mentioned**, not invoked | `test -f scripts/check_whitespace_hygiene.sh` in the block, and the reaction passed |
| `gate_shape_contract.rs` exit-code / both-directions properties | a twin's **header comment** | whole-file `contains` over text whose own doc comment says "a property of executed text" |
| `check_publish_source.sh` signature | a block quoted in the tag **message** | closed already in `ed25624`, same shape |
| my own two probes | a `usage:` banner listing every flag; a dragged-along `--manifest-path` | each measured the wrong thing and produced a wrong finding |

**A helper existed.** `uncommented()` was used by nine of eleven properties in one file, and by **none** of the
seven in the other — and that file is where two of the defects live. So the failure was not a missing helper: it was
that **forgetting it was possible**.

## What Changes

**A corpus is no longer handed to a recognizer as `&str`.** It arrives as a region, decided once and carried in the
type: `Source` yields `executed()`, `header()`, `prose()`, and an explicit `whole()`. A recognizer that wants
executed text cannot be given the whole file, because the types differ.

- `Prose` excludes fenced blocks **and HTML comments** — the second is a measured defect, not a precaution.
- `Executed` excludes comments; `Header` is everything above the first `##`.
- `whole()` is the deliberate escape, spelled so it is greppable. The family already treats `dyn` this way: not
  forbidden, but visible where it matters.

**Two requirements tighten, because two properties were checking the wrong thing rather than checking it loosely:**

- `gate-shape-contract`: Definition-of-Done membership means the file is **invoked**, in command position — not
  that its path appears somewhere in the block.
- `projection-register`: the document-to-holder correspondence is per **(call site, document)**, not per file. A
  second `assert_projection_matches` in an existing holder currently blesses an unregistered document silently —
  measured.

## Capabilities

### Modified Capabilities

- `gate-shape-contract`: properties read the region they are about; membership requires an invocation.
- `projection-register`: the correspondence is per call site; reachability excludes HTML comments.

## Impact

- **New**: `crates/tianheng/tests/support/region.rs`, shared by both reactions through the existing `support` module.
- **Modified**: both reactions' recognizers, and their projections where a cell changes.
- **Not affected**: no crate's public API, no `Constitution`, no baseline format, no gate. Version class **PATCH**.
