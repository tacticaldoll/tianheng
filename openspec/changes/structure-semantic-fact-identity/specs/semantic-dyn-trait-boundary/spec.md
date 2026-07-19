## MODIFIED Requirements

### Requirement: Stated coverage bounds with no false negative

Within the resolvable public surface there SHALL be no false negative: a `dyn` node that
*is* syntactically present in an exposed position MUST react, and the system MUST NOT
silently pass an exposed `dyn` it was able to observe. The capability inherits 渾儀's
**incidental, already-stated** coverage bounds unchanged and SHALL NOT silently assert a
boundary clean when one applies: a `dyn` introduced by **macro expansion** (the call site
writes no `dyn` token), a `dyn` reached only through a **glob import** or a
**`#[path]`-remapped module**, and a `dyn` reached only by expanding a **named `type`
alias** are out of scope. No *new* essential gap is introduced by this capability.

#### Scenario: A macro-generated dyn is a documented coverage bound

- **WHEN** a macro invoked in the governed module expands to a public signature containing `dyn`, while the call site writes no `dyn` token
- **THEN** the system does not claim to observe it (the universal 渾儀 macro-expansion bound), rather than silently asserting the boundary is clean

#### Scenario: A resolvable exposed dyn is never silently passed

- **WHEN** a `dyn` node is syntactically present in a public signature of the governed anchor
- **THEN** the system emits a violation, never exit 0 for that boundary

#### Scenario: Distinct exposed dyn shapes produce distinct findings

- **WHEN** the governed anchor exposes two structurally different trait objects whose differing payload is observable — the boxed-closure family (`Box<dyn Fn(i32) -> i32>` vs `Box<dyn FnMut(String) -> bool>`), associated-type bindings (`dyn Iterator<Item = u8>` vs `<Item = u16>`), nested trait objects, lifetimes, simple const generics, macro-named or fn-pointer generic arguments — and one is recorded in the baseline as accepted
- **THEN** the second still reacts: its canonical `subject` key field differs from the first. The dimension MUST preserve every observable distinguishing payload in that field (never collapse the realistic shapes above to one key — which would silently pass a new exposure under a baselined one, the one forbidden bug); its human rendering remains diagnostic rather than identity

#### Scenario: The same dyn shape at two seams stays distinct findings

- **WHEN** the governed anchor exposes the *same* `dyn` shape (e.g. `Box<dyn crate::infra::Port>`) at two distinct public seams — two functions, or a function and a field — and one is recorded in the baseline as accepted
- **THEN** the second still reacts: its structured seam kind and item/module/owner/member fields differ, so two seams sharing a `subject` do not collapse to one `(target, rule, finding_key)` and baselining one MUST NOT mask the other (the one forbidden bug); the human finding remains seam-qualified as `{rendered shape} exposed by {seam}`

#### Scenario: An unrenderable sub-node is a stated rendering bound

- **WHEN** two trait objects differ only inside a sub-node that cannot be rendered without macro expansion, token printing, or edit-unstable spans — a complex const-generic *expression* (`dyn Foo<{ N + 1 }>`), a same-named macro with different arguments (`dyn Foo<m!(1)>` vs `dyn Foo<m!(2)>`), a `verbatim` type, or a distinction carried only by a **lifetime** (a reference lifetime or an HRTB `for<'a>` binder, which carry no architectural intent and are not rendered)
- **THEN** the system does not claim to distinguish them: they share a canonical `subject` field and key at the same seam (each still *reacts* on first occurrence; only baseline-dedup granularity is bounded). This is a **stated subject-rendering bound** — the same `(target, rule, finding_key)` granularity bound `semantic-trait-impl-locality`'s `(impl for <self_ty>)` fact carries — declared here, never a silent claim of cleanliness
