# Capability catalog

This is contract coverage, not an architecture recommendation or onboarding path. It keeps the
focused examples small by collecting the published boundary families that otherwise have no
adopter-shaped reaction owner.

The crate is deliberately red: its dependency-source declaration, external import placement, trait
impl, marker impl, `dyn Trait` API, and `impl Trait` API each violate the Constitution in
`src/governance.rs`. Tests identify those reactions by structured identity rather than human
wording. Start with the standalone or composed examples when learning Tianheng.

The repository's executable family ledger lives in `scripts/test_examples.sh`: it counts this
catalog's families only after the real evaluator and structured assertions above succeed, then
compares all example owners with the deliberately reviewed inventory. The ledger does not
infer families from builder methods; OpenSpec/API review still decides whether a new insertion path
is a family, depth, modifier, or shorthand.
