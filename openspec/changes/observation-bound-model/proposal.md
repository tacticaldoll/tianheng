## Why

An observation bound — the claim that a reaction deliberately stops at a named shape — is the most
load-bearing sentence this family writes. It reads as **permission**: it tells a future auditor that a real
escape is governed policy. `observation-bound-register` made every bound carry a citation and made the set
enumerable. What it never governed is **what kind of stop each bound describes**, and that has been
accumulating in an unconstrained slot ever since.

The slot is literal. `check_bound_register.sh` recognizes a bound by

```
BOUND_HEADING='^#### Scenario: .*(stated|documented)( [A-Za-z-]+)? bounds?'
```

`( [A-Za-z-]+)?` is a one-word capture with **no vocabulary**. Measured across the declared bounds: sixteen
distinct phrasings, and three findings that make the drift concrete rather than untidy.

**`stated` and `documented` carry no information.** Two specs use both bare forms *inside the same spec*, and
the same qualifier appears under both words. It is a synonym pair, and half the bounds use neither qualifier
at all.

**One qualifier spans opposite sides of the line this family exists to hold.** `cfg-blind` appears twice.
`external-crate-confinement` uses it where a `#[cfg]`-dead import is observed as live — reacting **more** than
the truth. `runtime-origin-assertion` uses it where a probe behind `#[cfg(test)]` counts as coverage, so a
seam whose only production probe lives there is reported as **probed** — reacting **less** than the truth,
which is a false negative. One word, two directions, and the direction is the only thing that decides whether
a bound is a safe conservatism or a declared hole.

**A misclassification has already cost a wrong urgency call.** The `#[cfg_attr(pred, path=…)]` backlog entry
predicted a false negative; the reproduction found a constitution error, exit 2 — fail-loud, never a silent
pass. That entry's own recorded lesson is "the risk class is what decides urgency". With a declared kind the
prediction would have had to name one, and the mismatch would have been visible before the work was scheduled.

Why now: the 0.5.0 window is open, and the model is purely additive to a published crate, so it costs adopters
no migration. Every later change that declares a bound — the observer protocol that follows this one, and
`gate-shape-contract` whose proposal currently mints two more phrasings — either lands on a governed
vocabulary or adds to the drift.

## What Changes

**璇璣 (`xuanji`) gains a published bound model, and the classification moves from an adjective into types.**

- **Illegal states are unrepresentable in the data, not asserted about it.** `Extent` is nested: a shape the
  observation source never reached has nowhere to carry a claim about how the reaction treated it. The
  `#[cfg_attr]` misclassification above cannot be written in this model.
- **The value set is derived from the declared bounds, not invented.** Reading all of them yields seven ways a
  measure stops — out of reach; reached and refusing to judge; reached and *deliberately not* refusing;
  reached and over-reacting; reached and under-reacting; reached and correctly not a violation; reached,
  reacting exactly, and bounded only in what the fact carries. Six have live instances, cited in the delta
  spec. **Refusing to judge has none**, and is kept because the misclassification this model exists to prevent
  was exactly a confusion between it and *out of reach* — a direction that cannot be named cannot be predicted
  with.
- **A declared false negative names its owner.** `UnderReacts` carries `Owner` — this engine, a layer beneath
  it, or the adopter — because a false negative with nobody responsible for closing it is the shape that
  outlives its reason. One bound says so outright: "a false negative the adopter owns by narrowing".
- **What the pinning test must demonstrate is derived from the extent, never declared beside it.** A second
  copy carries no information and can contradict the first.
- **圭表, 渾儀, and 漏刻 each export their own declarations** as library items, not test items, so one reaction
  can see all of them and so the next change's `Observer::bounds()` has something to delegate to.
- **The heading's qualifier slot closes; the marker words do not change.** Exactly half the declared bounds
  carry a qualifier and half carry the bare marker. The qualifier is what did the damage — it read as a
  taxonomy and spanned both sides of the false-negative line — so it is what goes. `stated` versus
  `documented` is left alone: it is noise, and it misleads nobody. Narrowing the sweep to the harmful half also
  halves what it churns, which matters because of the next point.
- **Removing a qualifier changes that bound's id**, since the id is derived from the heading's slug. Every
  in-tree `(bound: …)` reference to a swept bound moves with it, and the register's existing
  reference-resolution reaction is what catches a missed one. This is the sweep's real cost and it is stated
  rather than discovered.
- **A new reaction holds the two sides in bijection** — every bound scenario in a spec has a declaration in
  code and every declaration has a scenario — and projects the extents into a generated, staleness-checked
  document.

## Capabilities

### New Capabilities

- `observation-bound-model`: the published type model for a declared observation bound — where a measure
  stops, who owns closing it, what its pinning test must demonstrate — and the reaction binding the specs'
  declarations to the code's in both directions.

### Modified Capabilities

- `observation-bound-register`: the heading marker loses its free qualifier slot, and a declared bound gains a
  second obligation beside its citation — a typed declaration in code, keyed on the bound id the register
  already derives.

## Impact

- **New**: `crates/xuanji/src/bound.rs`, exported from the crate root. Purely additive public API.
- **New**: `observation_bounds()` in `guibiao`, `hunyi`, and `louke` — the three crates that own declared
  bounds today. `xuanji`, `xingbiao`, and `tianheng` declare none and gain no export.
- **New**: a bijection-and-projection reaction under `crates/tianheng/tests/` — the only crate that sees all
  three dimensions — and its generated projection under `docs/`.
- **Modified**: every spec heading carrying a qualifier loses it; the statement's meaning moves into the
  declaration's rationale. Their derived ids change, and every in-tree reference to them changes with them.
- **Modified**: `scripts/check_bound_register.sh`'s `BOUND_HEADING`, plus an explicit refusal naming a
  qualified heading. **`BOUND_PROSE` is deliberately left permissive** — it is the register's detection floor,
  not a requirement on authored form, and narrowing it would stop it catching a bound stated in prose with a
  qualifier. That would put a false negative into the one direction that stops the register being completed by
  declaring only the convenient bounds.
- **Not affected**: no existing signature, no `Constitution`, no boundary DSL, no baseline format, no report
  shape. **Adopters migrate nothing**; the new surface is opt-in to read.
- Version class: **MINOR** — new published API on a pre-1.0 line, with no breaking change. Stated explicitly
  because "new public API" is easily read as requiring adopter work, and here it requires none.
