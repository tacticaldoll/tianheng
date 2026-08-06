# Change: the family's declarations stay literal, measurably

## Why

`observation-bound-model` says it outright:

> The family's own declarations SHALL remain literals. A bound is a property of the *reaction*, and this family's
> reactions know their limits when they are written; the owned form exists for implementors whose reactions do not.

**Nothing measures it.** Every constructor takes `impl Into<Cow<'static, str>>`, so a family declaration rewritten
as `format!("…{capability}…")` compiles, allocates on every run of the register and the extent projection, and is
noticed by no reaction and no gate. The requirement is a normative SHALL with no reaction — the class this window
has been closing everywhere else.

The distinction matters more now than when it was written, because it has a live counter-example. The change
immediately before this one added `examples/observer-participant`, whose declarations are *deliberately* computed —
id, shape, reason and pin all built with `format!`. So the tree now holds both kinds, and which kind belongs where
is exactly the thing prose alone cannot keep straight.

## What Changes

- **`BoundDecl::borrows_every_string()`** — a public method answering whether every string a declaration carries
  borrows rather than owning. It reaches the id, the shape, the pin, the extent's rationale, and an inherited
  owner's layer name, by **exhaustive in-crate matches**: a variant added later with a string of its own fails to
  compile here rather than going unmeasured.
- **A reaction asserting it over every one of the family's declarations**, naming any that allocates.
- **The converse is demonstrated**, so the discriminant cannot be a constant: a `xuanji` unit test declares a bound
  with computed strings and asserts the answer is `false`, and asserts it for each string position independently —
  a single `&&` chain that stopped early would otherwise pass while measuring one field.

## Impact

- Affected specs: `observation-bound-model`
- Affected code: `crates/xuanji/src/bound.rs`, `crates/xuanji/src/tests.rs`,
  `crates/tianheng/tests/observation_bound_model.rs`
- **Public API addition** (additive, nothing removed or moved): one method on an existing type. Exposed rather
  than private because the declarations being judged live in the dimension crates, which a `xuanji` unit test
  cannot read — and because "does this declaration allocate" is an honest question an adopter auditing a
  governance run can ask too.
- The extent projection must stay **byte-identical**: this change measures declarations, it does not alter one.
