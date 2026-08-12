## Context

`crates/kanhe/tests/gate_identity.rs` already enumerates the exact surface this needs:

```rust
git ls-files scripts/  →  filter(.sh)  →  citations(script, text)  →  offences(…)
```

with a vacuity guard on the enumeration (`!scripts.is_empty()`) and one on the extraction
(`!cited.is_empty()`). The second is the aggregate: every script's citations are pushed into one `cited`
vector and the assertion is that the **vector** is non-empty. A script contributing zero is unobserved.

Nothing else in the repository sees it either. `projection_register.rs` keeps a `check_*` recogniser asserting
its own emptiness, which is the right instinct but a thin slice — it catches a `check_*` gate that writes a
projection under `BLESS`, not one that merely judges. `repository-checks`' Purpose says none of these checks is
product, and its opening says `git ls-files scripts/` names only wrappers; both are prose.

## Goals / Non-Goals

**Goals:**

- Every enumerated script is examined on its own, so *this script cites nothing* is a finding rather than a
  gap between two vacuity guards.
- The judgement lives where the crate's other judgements live — a pure function returning the shared kinded
  refusal, with a synthetic failure matrix beside it.

**Non-Goals:**

- Deciding whether a script "carries a verdict" by reading what it does. That is a judgement over source
  prose, the instrument this repository has designed, measured three times and rejected. Citing a gate is a
  *reference*, and reference resolution here is mechanical — the same reason `--exact` identifiers are already
  pinned like paths.
- Reaching a script outside `scripts/`. The enumeration is the capability's declared subject glob and stays
  that.
- `scripts/publish.sh`'s environment blindness, `--auto`, or anything else in the wrapper allowlist. Untouched.

## Decisions

**Hold it per script, not by counting.** The weaker shape — assert the citation count is at least the script
count — would pass for two scripts where one cites twice and the other not at all. Counting is what the
aggregate guard already does one level up, and it is what failed. Each enumerated script is asked its own
question and named in its own refusal.

**A `Violation`, not a `CannotJudge`.** A script that was read and carries no citation is a source that
**disagrees** with the requirement, not one the check could not read. The existing arm already types an
unreadable script and an unbindable identifier as their own classes, so this slots beside them rather than
blurring one.

**The strong reading, chosen deliberately: every tracked script must be a wrapper.** The weaker alternative —
refuse only a script that *looks like* it judges — needs an instrument this repository rejected. The strong
reading is mechanical, and it is what the capability already claims about itself. Its cost is real and is
written into the requirement rather than left to be discovered: `scripts/` is now closed, and a future
convenience script either lives elsewhere or arrives with an amendment.

**The judgement is pure and the repository check is thin**, matching the nine matrix tests already in
`crates/kanhe/src/tests/gate_identity.rs`, which drive `citations` and `offences` with synthetic input and a
stub lister. A matrix can then state the shapes directly — a script that cites, one that does not, one whose
only citation is commented out — and the descriptions become rows that run rather than sentences beside the
code.

**The negative run is against the real repository, not only the matrix.** A new pure function's matrix fails
before the function exists, which proves nothing about whether the repository direction bites. So the recorded
negative run tracks a citation-free script, runs the direction, and observes it named — then reverts. The
matrix is the permanent guard; this is the evidence that the guard is wired to the tree.

## Risks / Trade-offs

**`scripts/` is closed by this, and someone will eventually want to open it** → Stated in the requirement, not
implied by the code, so the next person meets a sentence explaining the trade rather than a check they read as
a bug. Amending it is one delta.

**A script could cite a gate and still judge on its own afterwards** → True, and not closed here. Citing is
necessary, not sufficient. The sufficient version needs the prose instrument this repository rejected, so what
is bought is the *shape* — a script that defers to nothing cannot exist — rather than a proof that a script
that defers does nothing else. Worth saying plainly instead of letting the requirement read as more than it
holds.

**Both vacuity guards must survive** → The new per-member direction makes `!cited.is_empty()` look redundant,
since every script citing at least one implies the aggregate is non-empty. It is not redundant when the script
enumeration is itself empty in some future tree, and removing a guard because a sibling currently implies it is
how a floor is lost. Both stay.

## Migration Plan

None. `scripts/` and `crates/kanhe/` ship in zero packages, no published surface moves, and the workspace
version stays where it is. Both tracked scripts already cite their gates, so nothing existing is refused —
verified before the requirement was written, not after.

## Open Questions

- **Should the same per-member discipline be swept across the other repository checks?** `prelude_promise.rs`
  is the one whose corpus is typed rather than produced, and it is a separate finding with its own filing. Named
  here so the pattern is visible, not absorbed.
