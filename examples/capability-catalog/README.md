# Capability catalog

This is contract coverage, not an architecture recommendation or onboarding path. It keeps the
focused examples small by collecting the published boundary families that otherwise have no
adopter-shaped reaction owner.

The crate is deliberately red: its dependency-source declaration, external import placement, trait
impl, marker impl, `dyn Trait` API, and `impl Trait` API each violate the Constitution in
`src/governance.rs`. Tests identify those reactions by structured identity rather than human
wording. Start with the standalone or composed examples when learning Tianheng.

Two families additionally carry a closed-false-negative shape from the `v0.2.3..release/0.3.1` adversarial
sweep, reusing the SAME declared boundary rather than a second one (the point is that one correctly
scoped boundary catches every shape, not just the plain one it was first written against):
`src/marked.rs`'s `CfgGatedMarked` acquires the forbidden marker only inside a `cfg_if!` arm, and
`src/misplaced.rs`'s `Rogue` implements the forbidden trait only behind the "const-eval trick"
(`const _: () = { impl … };`) — both previously escaped observation and now react by name
(`tests/reaction.rs`). A third test, `a_malformed_forbidden_operand_fails_loud_as_a_constitution_error`,
declares its own tiny, isolated Constitution to demonstrate the sweep's malformed-operand fix as a
failure mode on purpose: a leading-`::` operand now fails loud as a constitution error (exit 2)
instead of silently matching nothing.

The repository's executable family ledger lives in `scripts/test_examples.sh`: it counts this
catalog's families only after the real evaluator and structured assertions above succeed, then
compares all example owners with the deliberately reviewed inventory. The ledger does not
infer families from builder methods; OpenSpec/API review still decides whether a new insertion path
is a family, depth, modifier, or shorthand.
