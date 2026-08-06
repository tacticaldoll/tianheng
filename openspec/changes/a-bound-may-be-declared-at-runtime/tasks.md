## 1. The type change, with rustc as the enumeration

- [ ] 1.1 `BoundId` holds `Cow<'static, str>`; `new` takes `impl Into<Cow<'static, str>>`; `as_str` lends `&str`.
- [ ] 1.2 `BoundDecl`'s `shape` and `pinned_by` likewise, and `new` takes `impl Into<…>` for both.
- [ ] 1.3 Every `because` field and `Owner::Inherited { from }` becomes `Cow<'static, str>`.
- [ ] 1.4 Drop `const fn` only where `Cow` forbids it, and confirm no `const` item constructs a declaration.
- [ ] 1.5 Build, and fix every site rustc names — never a grep, because a pattern over multi-line string
      continuations cannot promise no false negative (design D3).

## 2. The guard for what this buys

- [ ] 2.1 A test constructing a bound whose id, shape and rationale are **computed** (`format!`), asserting it
      behaves as a literal one does: same extent, same derived evidence.
- [ ] 2.2 Observe it failing without the change — it will not compile, so record the compiler error as the
      observation. A type-level guard's negative run is a build failure, and saying so is more honest than
      claiming a runtime assertion.
- [ ] 2.3 Assert a literal declaration still borrows: `matches!(…, Cow::Borrowed(_))` on one family declaration,
      so the zero-allocation half is a property and not an intention.

## 3. Nothing else moves

- [ ] 3.1 The extents projection must come out **byte-identical** — the strings did not change, only their type.
      That is the guard against a mangled rationale during the mechanical edit.
- [ ] 3.2 The bijection, the register's figures (53 bounds across 19 capabilities, 0 unpinned) and the
      declared-false-negative count (14) must all be unchanged.

## 4. The three CHANGELOG corrections from the same review

- [ ] 4.1 Name `StaticObserver`, `SemanticObserver` and `RuntimeObserver` in the `Observer` entry.
- [ ] 4.2 Say that `louke::RuntimeObserver` is behind the `audit` feature, and that `tianheng` enables it — which is
      why the omission was easy to miss.
- [ ] 4.3 Mention `tianheng::testing::assert_projection_matches`.
- [ ] 4.4 Record this change itself in the bound-model entry, as a shape refinement rather than a migration, and say
      why it owes no `**BREAKING**` mark: measured, none of these types exist in `v0.4.0`.

## 5. Verification

- [ ] 5.1 Full Definition of Done, then again from a clean clone.
- [ ] 5.2 `openspec validate` strict before and after sync.
- [ ] 5.3 Confirm the whole public surface still compiles for an adopter shape: the doc example / prelude path.
