## MODIFIED Requirements

### Requirement: Signature exposure facts use structural seam roles

Every signature-coupling fact SHALL separately encode the forbidden subject and the public seam
roles that make the exposure distinct. Semantic member positions such as tuple-field indices MAY be
identity-bearing observations; scan order, item ordinal, and renderer fallback position SHALL NOT.
An inherent method or associated `const`/`type` seam SHALL additionally encode the **module the impl
block itself is written in**, distinct from the self type's own canonical owner path: Rust's
coherence rules let an inherent `impl` for one type be written in any module of the same crate, so
two impl blocks in different modules for the identical self type — each declaring a same-named
public method or associated item — resolve to the identical owner and therefore MUST NOT collapse to
one seam. This module role is on behalf of every capability that builds an inherent-method/associated
seam through the shared `PublicSeam` vocabulary (dyn-trait, impl-trait), not signature-coupling
alone, matching how this spec already states shared anchor-resolution properties on their behalf.

#### Scenario: Two exposed seams stay distinct
- **WHEN** the same forbidden subject appears at two public seams
- **THEN** their structured seam roles differ and accepting one does not mask the other

#### Scenario: Reordering does not alter a seam
- **WHEN** unrelated items are inserted or declarations reordered
- **THEN** pre-existing exposure identities remain unchanged

#### Scenario: Two impl blocks in different modules for the same owner stay distinct inherent-method seams

- **WHEN** a type is declared in one module and inherent-`impl`'d with a same-named public method in
  two OTHER, sibling modules (a platform-conditional split, e.g. `plat_unix`/`plat_win` both writing
  `impl Conn { pub fn open(&self) -> impl crate::Port { … } }` for a `Conn` declared in `common`),
  observed by a capability that walks both modules in one evaluation
- **THEN** the two impl sites produce two distinct seams, qualified by each impl block's own
  declaring module in addition to the self type's owner, so accepting one does not mask the other —
  the same guarantee an inherent method already held across two DIFFERENT self types, extended to
  hold across two impl blocks of the SAME self type written in different modules

#### Scenario: The same guarantee holds for an inherent associated const/type seam

- **WHEN** two impl blocks in different modules for the same owner type each declare a same-named
  public associated `const` or `type`
- **THEN** the two associated-item seams stay distinct by the same module-qualified rule
