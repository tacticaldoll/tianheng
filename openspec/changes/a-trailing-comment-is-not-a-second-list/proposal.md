# Change: a trailing comment is not a second list, and a file count is not prose

## Why

Four findings from this window's closing review, each verified before being accepted.

1. **A trailing comment on the delegation line reads as a divergent list.** `every_observer_declares_exactly_its_
   dimension_s_bounds` requires each `bounds()` body to hold exactly `observation_bounds()`. `Executed::lines()`
   filters comment *lines* and not trailing comments, so `observation_bounds() // the dimension's own export`
   compares unequal. Measured: adding that comment to 渾儀's observer makes the reaction report an offence.

   A false positive, so the safe direction — but it contradicts the region discipline this family built two
   changes ago, where a comment is prose and never executed text. And it is a **regression against the
   established convention**, not a gap in it: both whole-line recognizers in `gate_shape_contract.rs` already
   strip a trailing comment before comparing (`argument.find(" #")`, `line.find('#')`). The new recognizer did
   not.

2. **The method is located by raw text search.** `bounds_body` takes `Source::whole()` and finds the first
   `fn bounds(` anywhere in the file, comments included. No observer file mentions it in a doc comment today, so
   this is latent rather than live — but the recognizer would happily brace-match from a sentence *about* the
   method, and that is the fourth variant of a trap this family has paid for: recognize by position, never by the
   bare marker.

3. **A file count was written into prose, in three places.** "the eight files under `src/runner/`" appears in the
   spec, in the reaction's doc comment, and in the changelog. It is correct today — measured, eight — and it is
   exactly the shape this window already dismantled once, where a comment said fifty-three declarations while the
   register counted fifty-four. The claim does not need the number.

4. **"the register's 55th bound"** in the changelog is an ordinal a reader will read as a total. Another bound
   added before release leaves it true as history and misleading as a figure. `check_bound_register.sh` compares
   written censuses of the form *N bounds across M capabilities* and cannot see this phrasing.

## What Changes

- The recognizer strips a trailing comment before comparing, following the convention already in
  `gate_shape_contract.rs`, and the spec says a trailing comment is prose rather than a second list.
- `fn bounds(` is located **by line position** — a line whose trimmed start is the signature — so a mention
  inside a comment or a string cannot be brace-matched from.
- The three file counts become the claim without the number; the ordinal becomes the fact.

## Impact

- Affected specs: `observer-protocol`
- Affected code: `crates/tianheng/tests/observer_protocol.rs`, `CHANGELOG.md`
- No public API change, no version bump.
