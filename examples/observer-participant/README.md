# `observer-participant` — joining a run from outside the family

The other examples adopt 天衡's dimensions. This one **extends** it: a crate with a house rule that no
dimension of 三儀 governs, folded into the same run, reporting through the same verdict and the same exit code.

The rule is small on purpose — *every module file in a governed subtree opens with a `//!` header* — because the
subject is the seam, not the rule. It is neither a dependency edge (圭表), nor an exposed type (渾儀), nor a
runtime origin (漏刻), so there is no boundary DSL to express it and nothing to wait for the family to add.

```rust
Run::over(&manifest)
    .observe(StaticObserver::new(constitution().static_boundaries().clone()))
    .observe(ModuleHeaderObserver::reading(["src"]))
    .verdict()
```

Two faults are deliberate and neither is a bug: `src/api.rs` imports `crate::infra`, which 圭表 reacts to, and
`src/undocumented.rs` opens with no `//!` header, which the participant reacts to. `cargo run --bin demo` exits
`1` and prints both, then prints what the participant declines to see.

## What this example is here to prove

**The public surface is enough.** Everything the participant needs — `Observer`, `Violation::new`,
`ViolationId`, `RuleKey`, `StructuredFactIdentity`, `BoundDecl`, `BoundId`, `Extent` — comes from
`tianheng::prelude`. Nothing was added to any crate to make this compile. If it had needed an export, that would
have been the finding: a protocol a third party cannot use is not a protocol.

**A participant's bounds can be computed.** `ModuleHeaderObserver` declares two bounds per configured subtree,
with the id, the shape, the reason and the pin all built by `format!` at the moment it is asked, because *which*
bounds it has depends on what it was told to read. `BoundId`'s owned-or-borrowed form exists for exactly this,
and nothing inside the family exercises it — every family declaration is a literal.

**A computed id needs two forms, and the type enforces neither.** A multi-segment subtree — `src/bin` — put a
second slash into the id, so `house-rules/a-file-nested-below-src/bin-is-out-of-reach` read to a human as a
second capability separator. The pin beside it was already sanitised into a Rust identifier, and the id was
not, in the same expression. Both are derived now: the pin with underscores, the id with the shape this
family's own headings take. Nothing catches this for you — `BoundId` validates no form, and the reaction that
names a malformed one audits *this family's* specs, not yours. It is a convention because it reads; a
participant that audits its own bounds defines what its own accounting needs.

**And it declares more than one *extent*.** One bound is a shape it never reads: it lists a subtree one level deep
and never descends, so a nested module file is out of reach. The other is a shape it reads and judges **too
harshly** — the rule tests a file's first line, so a real module header sitting below a licence comment reads as
absent, even though a reader of that file learns exactly what the rule says it should. That distance between a
rule's wording and the reason it gives for itself is what `Reached::OverReacts` names.

It is **declared rather than closed**, deliberately. Skipping a leading comment block would trade this edge for a
`/* … */` header and for an inner attribute above the doc comment, and would leave the rule's wording saying
something other than what the code does. Declaring it is what a participant owes a reader; fixing every edge is
not.

**An outsider inherits the contract.** A subtree the participant was told to read and could not is exit `2`, not
a quiet pass. Reporting clean because the look failed is the one bug 天衡 forbids outright, and joining a run
does not exempt anyone from it. `tests/reaction.rs` asserts that direction alongside the reacting ones.

**Composition is asserted, not assumed.** The tests bind each contribution's own structured identity rather than
the exit code, because each fault reacts on its own — `exit_code() == 1` would keep holding if the participant
stopped contributing entirely.

## What it found

`BoundaryKind` has **no value a participant owns**. An outsider's violation must claim one of the family's four
— `Crate`, `Module`, `Semantic`, `Runtime` — so this participant reports `Module` as the nearest honest fit while
governing nothing 圭表 would call a module boundary. The kind is the label a report and a SARIF render carry —
**not** a baseline, which records `(target, rule_key, fact)` and no kind at all — so a borrowed one misleads a
consumer filtering by dimension without making anyone's recorded entries stale. That is a vocabulary decision
rather than a variant to add in passing, and it is recorded in `BACKLOG.md` with its trigger instead of being
worked around here.

## Running it

Run through the repository's gate, which patches the family crates to local source:

```bash
bash the examples suite
```
