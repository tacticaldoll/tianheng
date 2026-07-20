# Capability catalog

This is contract coverage, not an architecture recommendation or onboarding path. It keeps the
focused examples small by collecting the public 0.2.x boundary families that otherwise have no
adopter-shaped reaction owner.

The crate is deliberately red: its dependency-source declaration, external import placement, trait
impl, marker impl, `dyn Trait` API, and `impl Trait` API each violate the Constitution in
`src/governance.rs`. Tests identify those reactions by structured identity rather than human
wording. Start with the standalone or composed examples when learning Tianheng.
