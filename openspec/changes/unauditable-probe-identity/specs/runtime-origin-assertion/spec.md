## ADDED Requirements

### Requirement: An un-auditable probe's identity distinguishes distinct offending expressions

An un-auditable-probe fact SHALL identify the offending non-literal seam expression's own source
text (its first macro-argument span, trimmed) and its **owner-qualified enclosing item**, alongside
its file, so two non-literal expressions differing by file, enclosing item, or expression text
remain distinct findings, and baselining one SHALL NOT mask another. The owner-qualified enclosing
item SHALL NOT be a bare innermost name: for a free `fn` it is the module path plus the fn name; for
a method inside `impl Type { … }` it is the `Self` type plus the method name; for a method inside
`impl Trait for Type { … }` it is the trait path, the `Self` type, and the method name; for a
trait's own default-body method it is the trait name plus the method name — mirroring the owner/
trait_ref qualification `semantic-unsafe-confinement` already uses for the identical same-named-item
collision. A bare method name alone SHALL NOT be used, since two distinct owners may share one.

Byte-identical expression text within the same file and the same owner-qualified enclosing item
collapses to one finding — a stated bound, not a silent gap: at that granularity no further source
content distinguishes the two occurrences, so they represent the same restated fact (mirroring
`module-boundary`'s "the same import on multiple lines is one violation" precedent), not two masked
problems. Neither the enclosing-item qualification nor the expression text SHALL be derived from
byte offset, line number, or occurrence count.

#### Scenario: Same expression in two different free functions stays distinct

- **WHEN** `fn a() { assert_boundary!(SEAM_A, obj); }` and `fn b() { assert_boundary!(SEAM_A, obj); }` appear in the same file
- **THEN** `audit_probe_coverage` emits two distinct un-auditable-probe violations, distinguished by their enclosing function, and baselining one does not suppress the other

#### Scenario: Same-named method in two different impls stays distinct

- **WHEN** a file contains `impl A { fn probe(&self) { assert_boundary!(SEAM_A, obj); } }` and `impl B { fn probe(&self) { assert_boundary!(SEAM_A, obj); } }`
- **THEN** `audit_probe_coverage` emits two distinct un-auditable-probe violations, distinguished by their owner (`A` vs `B`), even though the method name and expression text are identical, and baselining one does not suppress the other

#### Scenario: Same-named method in two different trait impls of the same type stays distinct

- **WHEN** a file contains `impl Foo for T { fn probe(&self) { assert_boundary!(SEAM_A, obj); } }` and `impl Bar for T { fn probe(&self) { assert_boundary!(SEAM_A, obj); } }`
- **THEN** `audit_probe_coverage` emits two distinct un-auditable-probe violations, distinguished by their trait (`Foo` vs `Bar`) even though the `Self` type, method name, and expression text are identical

#### Scenario: Two distinct expressions in the same function stay distinct

- **WHEN** a single `fn` contains both `assert_boundary!(SEAM_A, obj)` and `assert_boundary!(compute_seam(), obj)`
- **THEN** `audit_probe_coverage` emits two distinct un-auditable-probe violations, distinguished by their expression text

#### Scenario: Identical expression repeated in the same function collapses to one finding

- **WHEN** a single `fn` contains `assert_boundary!(SEAM_A, obj)` written twice, verbatim
- **THEN** `audit_probe_coverage` emits one un-auditable-probe violation for that site — a stated bound, since no further source content distinguishes the two occurrences

#### Scenario: Two files with the identical expression stay distinct by file

- **WHEN** `assert_boundary!(SEAM_A, obj)` appears in `src/a.rs` and, separately, in `src/b.rs`
- **THEN** `audit_probe_coverage` emits two distinct un-auditable-probe violations, distinguished by file
