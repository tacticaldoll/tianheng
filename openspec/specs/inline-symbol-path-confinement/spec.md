# inline-symbol-path-confinement Specification

## Purpose

The 圭表 (static) inline-symbol-path confinement — the layer-(b) sibling of
`external-crate-confinement`, observing **calls** rather than `use` imports. Within a governed
module subtree it forbids inline symbol-path calls resolving under a declared module-path prefix
(the "core reads no ambient clock; time is injected" pattern). A call-vs-mention default keeps 圭表
free of a read-verb heuristic (type annotations and constants pass); `.ending_with([verbs])`
narrows to adopter-declared read verbs, `.strict_prefix_only()` escalates to any mention. Path
heads resolve through the alias-carrying use-map, local `type` aliases, and the local `pub use`
re-export closure to a fixpoint; a glob that can bring a prefix-resolving name into scope reacts
fail-closed (stated by hazard, not shape). Source-observed on the hand-rolled 圭表 token scanner
(serde_json-only, no `syn`); not `cargo-deny`'s resolved/whole-graph lane.

## Requirements

### Requirement: Inline-symbol-path confinement declared in Rust

A module boundary SHALL support forbidding, within a governed module subtree, inline symbol
paths resolving under a declared **module-path prefix**, declared as
`ModuleBoundary::in_crate(p).module(s).must_not_call_inline(prefix).because("…")`, where `s`
is the governed subtree and `prefix` is a module-path prefix (e.g. `std::time`). The target is
a **prefix**, never a hand-copied leaf/function list (a leaf list drifts — the `freeze_methods`
anti-pattern). The optional modifiers (`.ending_with`, `.strict_prefix_only`) SHALL hang off a
**dedicated inline-confinement draft stage**, distinct from the shared module-rule draft, so
they cannot be applied to `must_not_import` / `confine_external_crate` (no modifier pollution of
other module rules). It SHALL carry a severity (default `enforce`, `warn` available) like every
other module rule and be accepted by the umbrella `Boundary`. The system MUST NOT require any
generated policy file.

#### Scenario: A confinement holds its subtree, prefix, and reason
- **WHEN** a developer declares `ModuleBoundary::in_crate("app").module("crate::core").must_not_call_inline("std::time").because("time is injected")`
- **THEN** the constitution holds a module boundary on crate `app`, governing subtree `crate::core`, forbidding inline calls under prefix `std::time`, with a non-empty reason and default `enforce` severity

### Requirement: Call-vs-mention default

By default (no narrowing modifier) the system SHALL react on an inline path resolving under the
prefix **only when it is applied as a call** (`path(...)` or `path::<...>(...)`). A path used as
a type annotation, a bare constant reference, or any non-call position SHALL NOT react. This
distinction is structural — the engine keys on the presence of a call application, never on a
built-in notion of which verb is a "read". A forbidden path taken as a **value** rather than
called (`let f = std::time::SystemTime::now; f()`) is a mention under the default and is covered
only by `.strict_prefix_only()` — a stated bound (see "Observation bounds"), not a claim.

#### Scenario: An associated-function call under the prefix reacts
- **WHEN** `crate::core` contains `std::time::SystemTime::now()` and a boundary forbids inline calls under `std::time` on `crate::core`
- **THEN** the system emits a violation naming the offending call and module

#### Scenario: A type annotation under the prefix passes
- **WHEN** `crate::core` contains `fn handle(ev: Event, now: std::time::Instant)` (a type annotation, no call)
- **THEN** the system reports no violation (a mention is not a call — the core may receive injected time)

#### Scenario: A deterministic constant under the prefix passes
- **WHEN** `crate::core` reads `std::time::SystemTime::UNIX_EPOCH` (a constant, no call)
- **THEN** the system reports no violation (a constant is not a call and is not an ambient read)

### Requirement: Prefix resolution follows use-map, type-alias, and local re-export to a fixpoint

The system SHALL resolve a written path's head to a canonical path before prefix-matching, using
the enclosing module's `use`-map (**aliases included**), the crate's local `type` aliases, and
the crate's local `pub use` re-export closure, chased to a fixpoint. Each of the following SHALL
resolve under the prefix and react: a fully-qualified path (`std::time::SystemTime::now()`); a
rename (`use std::time::SystemTime as SysT; SysT::now()`); a bare path (`use std::time;
time::SystemTime::now()`); a local `type` alias (`type Clock = std::time::SystemTime; Clock::now()`);
and a local re-export used from the governed subtree, **same- or cross-module**
(`pub use std::time::SystemTime;` in one local module, then `SystemTime::now()`). An unresolved
head SHALL NOT be matched by leaf alone (that would be a false positive on a same-named local
type). Within this resolvable scope there SHALL be no false negative.

#### Scenario: A renamed alias resolves and reacts
- **WHEN** `crate::core` declares `use std::time::SystemTime as SysT;` then calls `SysT::now()`
- **THEN** the system resolves `SysT` through the use-map to `std::time::SystemTime` and reacts

#### Scenario: A bare path resolves and reacts
- **WHEN** `crate::core` declares `use std::time;` then calls `time::Instant::now()`
- **THEN** the system resolves `time` to `std::time` and reacts

#### Scenario: A local type alias resolves and reacts
- **WHEN** `crate::core` declares `type Clock = std::time::SystemTime;` then calls `Clock::now()`
- **THEN** the system resolves `Clock` to `std::time::SystemTime` and reacts (a `type` alias is followed, not treated as an unresolved local)

#### Scenario: A cross-module local re-export resolves and reacts
- **WHEN** `crate::support` declares `pub use std::time::SystemTime;`, `crate::core` declares `use crate::support::SystemTime;` then calls `SystemTime::now()`
- **THEN** the system chases the local re-export closure to `std::time::SystemTime` and reacts

#### Scenario: A multi-hop type alias resolves to a fixpoint
- **WHEN** `crate::core` declares `type A = std::time::SystemTime; type B = A;` then calls `B::now()`
- **THEN** the system chases `B → A → std::time::SystemTime` to a fixpoint and reacts (resolution is not single-hop)

#### Scenario: A multi-hop local re-export resolves to a fixpoint
- **WHEN** `crate::a` declares `pub use std::time::SystemTime;`, `crate::b` declares `pub use crate::a::SystemTime;`, `crate::core` uses `crate::b::SystemTime` then calls `SystemTime::now()`
- **THEN** the system chases the two-hop re-export closure to `std::time::SystemTime` and reacts

#### Scenario: An unresolved same-named local is not matched by leaf
- **WHEN** `crate::core` defines a local type `Instant` (not `std::time::Instant`) and calls `Instant::now()`, with no `use` / `type` / re-export bringing `std::time::Instant` into scope
- **THEN** the system does NOT react (leaf-only matching is rejected — it would be a false positive)

### Requirement: A glob that can bring a prefix-resolving name into scope reacts (fail-closed)

The rule SHALL be stated by the **hazard**, not a single glob shape (an enumerated shape list
would itself drift): the system SHALL react (fail-closed) on a glob import within the governed
subtree whenever the glob can bring into scope a name that resolves under the confined prefix but
that the scanner cannot enumerate, naming the glob import as the finding. After resolving the
glob's own path through the same use-map / type-alias / re-export closure, the system SHALL react
when the resolved glob path is: (a) the confined prefix or **beneath** it (`use std::time::*`,
`use std::time::ext::*`); (b) an **ancestor** of the prefix — the glob brings the prefix's next
segment below the ancestor into scope (`use std::*` brings module `time`, the segment below `std`,
into scope); or (c) a **local module whose own re-export closure reaches under the prefix** —
where "reaches" applies this same hazard test recursively (chased to a fixpoint / visited set,
cycle-safe), over that module's re-exports resolved through the combined use-map + type-alias
closure: a concrete `pub use std::time::…` (or `pub use std::time;` / `pub type … = std::time::…;`),
OR a glob/ancestor re-export in that module that itself reaches the prefix (`pub use std::time::*;`
/ `pub use std::*;` inside `crate::support`, then `use crate::support::*;` in the subtree). Grouped or mixed glob forms (`use std::time::{*}`, `use std::time::{self, *}`)
SHALL be treated as globs. A glob finding SHALL NOT be suppressed by `.ending_with` narrowing (a
glob has no call terminal segment; narrowing applies to calls only). The scanner cannot prove such
a glob introduces no forbidden read, so the glob itself is the violation — one finding, never an FP
flood, never a silent pass.

#### Scenario: A glob of the confined prefix reacts
- **WHEN** `crate::core` declares `use std::time::*;` under a boundary confining `std::time`
- **THEN** the system reacts, naming the glob import as the finding

#### Scenario: A glob above the prefix reacts
- **WHEN** `crate::core` declares `use std::*;` (bringing module `time` into scope) then `time::Instant::now()`, under a boundary confining `std::time`
- **THEN** the system reacts on the glob `use std::*;` (an ancestor glob that brings the prefix's next segment below the ancestor, `time`, into scope), rather than silently passing the unresolvable bare `time::…`

#### Scenario: A glob of a local re-exporting module reacts
- **WHEN** `crate::support` declares `pub use std::time::SystemTime;`, `crate::core` declares `use crate::support::*;` then `SystemTime::now()`, under a boundary confining `std::time`
- **THEN** the system reacts on the glob `use crate::support::*;` (a local module whose observable re-exports reach under the prefix)

#### Scenario: A glob of a local module that itself globs the prefix reacts (recursive hazard)
- **WHEN** `crate::support` declares `pub use std::time::*;`, `crate::core` declares `use crate::support::*;` then `Instant::now()`, under a boundary confining `std::time`
- **THEN** the system reacts on the glob `use crate::support::*;` — the hazard test applied recursively to `support`'s re-export closure finds a glob reaching under the prefix (the "family not shape" rule does not stop at one level)

#### Scenario: An aliased-prefix glob reacts (glob path resolved first)
- **WHEN** `crate::core` declares `use std::time as t; use t::*;` under a boundary confining `std::time`
- **THEN** the system resolves the glob's own path `t → std::time` through the use-map, then reacts (case (a)), rather than missing it because the glob was written through an alias

#### Scenario: A glob finding is not suppressed by narrowing
- **WHEN** a boundary declares `.must_not_call_inline("std::time").ending_with(["now"])` and `crate::core` declares `use std::time::*;`
- **THEN** the system still reacts on the glob (narrowing filters call terminal segments, not globs)

### Requirement: Explicit read-verb narrowing owns its false negative

A confinement MAY be narrowed with `.ending_with([…])`; when narrowed, the system SHALL react
only on calls whose terminal segment (leaf-exact) is one of the declared verbs (e.g. `["now"]`).
Narrowing is a deliberate, adopter-owned act: a read reachable only through a verb the adopter
did not declare (a future `::current()`) SHALL be a false negative the **adopter** accepts by
narrowing. The engine MUST NOT bake a default verb set of its own.

#### Scenario: Narrowing drops a benign constructor call
- **WHEN** a boundary declares `.must_not_call_inline("std::time").ending_with(["now"])` and `crate::core` calls both `std::time::Instant::now()` and `std::time::Duration::from_secs(5)`
- **THEN** the system reacts on `Instant::now()` and does NOT react on `Duration::from_secs(5)` (terminal `from_secs` is not a declared verb)

#### Scenario: A future read verb outside the declared set is a documented adopter-owned bound
- **WHEN** a boundary is narrowed to `.ending_with(["now"])` and `crate::core` calls `std::time::SystemTime::current()` (hypothetical non-`now` read)
- **THEN** the system does NOT react (a false negative the adopter owns by narrowing), rather than the engine silently guessing which verbs are reads

### Requirement: Strict escalation forbids non-call mentions

A confinement MAY be escalated with `.strict_prefix_only()`; when escalated, the system SHALL
react on **any** path resolving under the prefix, call or not — including type annotations,
constants, and value-position mentions. This is the whole-surface isolation posture for a subtree
that may not even name the module. Narrowing and escalation are mutually exclusive: combining
`.ending_with(…)` with `.strict_prefix_only()` on one boundary SHALL be a constitution error
(exit 2), never a silent precedence choice.

#### Scenario: Strict flags a type annotation
- **WHEN** a boundary declares `.must_not_call_inline("std::time").strict_prefix_only()` and `crate::core` contains `now: std::time::Instant` (a type annotation)
- **THEN** the system reacts (strict forbids mentions, not only calls)

#### Scenario: Combining narrowing and strict is a constitution error
- **WHEN** a boundary declares `.must_not_call_inline("std::time").ending_with(["now"]).strict_prefix_only()`
- **THEN** the system reacts with exit 2 (a contradictory declaration), not a silent resolution

### Requirement: Macro bodies are conservatively scanned, never silently skipped

Within a governed subtree, a macro-invocation body SHALL be token-scanned for paths resolving
under the prefix through the enclosing module use-map, and SHALL react on a match. The system
MUST NOT silently skip macro-invocation bodies — that would be a false negative, the one
forbidden bug (real reads hide in `cfg_if!` / logging / async DSL bodies).

#### Scenario: A forbidden call inside a macro body reacts
- **WHEN** `crate::core` contains `cfg_if! { if #[cfg(feature="x")] { std::time::Instant::now() } }` under a boundary forbidding inline calls under `std::time`
- **THEN** the system reacts (the macro body is scanned, not skipped)

### Requirement: Observation bounds are stated, not silent

The following SHALL be OUT OF SCOPE as stated coverage bounds, never a claimed reaction and never
a silent pass beyond them: (1) a read whose type is not in a plain written path — a
receiver-method call (`instant.elapsed()`) or a UFCS-qualified call (`<Type as Trait>::now()`,
type inside `<…>`) — no type inference; (2) an alias introduced *within* an unexpanded
macro-invocation body; (3) a symbol name assembled by fragment/proc-macro construction (`paste!`,
`concat_idents!`) or generated by a proc-macro; (4) a path reached through an **external**-crate
re-export (foreign AST is not observed); (5) a forbidden path taken as a **value** (fn-item /
closure) rather than called — covered only under `.strict_prefix_only()`; and (6) the module
scanner's **inherited file-scope bounds** — a `#[path]`-remapped module (including a
`cfg_attr`-wrapped `#[path]`), `#[cfg]`-gated code
(observed as written, cfg-blind), and the lib+bin conventional-path conflation — **except**
macro-invocation bodies, which this rule overrides by scanning them (per "Macro bodies are
conservatively scanned"). Each bound is a declared non-observation, not a silent pass on a case
within scope.

#### Scenario: A `#[path]`-remapped file in the subtree is a documented bound
- **WHEN** a `#[path = "…"]`-remapped module inside `crate::core` contains `std::time::Instant::now()`
- **THEN** the system does not claim to observe it (the `#[path]` remap is an inherited scanner bound, as in `external-crate-confinement`) — a stated bound, not a silent assertion of cleanliness

#### Scenario: A receiver-method read is a documented bound
- **WHEN** `crate::core` calls `some_instant.elapsed()` where `some_instant` is an `Instant` value received by injection
- **THEN** the system does not claim to observe it (no type inference on the receiver) — a stated bound, not a silent assertion of cleanliness

#### Scenario: A path taken as a value is a documented bound under the default
- **WHEN** `crate::core` writes `let f = std::time::SystemTime::now; f();` under a default (non-strict) confinement
- **THEN** the system does not react (value-position mention is a stated bound under the default; `.strict_prefix_only()` catches it) — declared, not silent

#### Scenario: An external-crate re-export is a documented bound
- **WHEN** a foreign crate re-exports `std::time::SystemTime` and `crate::core` reaches it through that foreign path
- **THEN** the system does not claim to observe it (foreign AST is not scanned) — a stated bound

### Requirement: Constitution errors are loud, never silent

A misdeclared boundary SHALL react with exit 2 (constitution error), never a silent no-op: an
empty prefix; an empty verb set passed to `.ending_with([])`; the contradictory
`.ending_with(…).strict_prefix_only()` combination; and a governed subtree anchor that resolves
to no reachable module. A governed source file that exists but cannot be read SHALL likewise be a
scan error (exit 2), never silently skipped. In contrast, a **valid** prefix that matches no
inline call in a resolvable subtree is **clean** (exit 0), not an error — a confinement with zero
findings is a passing reaction, exactly as a never-imported confined crate is clean under
`external-crate-confinement`.

#### Scenario: An empty prefix is a constitution error
- **WHEN** a boundary declares `.must_not_call_inline("")`
- **THEN** the system reacts with exit 2 (a misdeclaration), never a silent match-everything or match-nothing

#### Scenario: A valid confinement with no matching call is clean
- **WHEN** a boundary confines `std::time` on `crate::core` and `crate::core` makes no inline call resolving under `std::time`
- **THEN** the system reports no violation and the reaction passes (exit 0)

### Requirement: Identity distinguishes the confined prefix and the call

A violation's baseline identity SHALL distinguish both the **confined prefix** and the **specific
offending element**, so that no two distinct confinements or offending elements collapse into one
baseline entry (which would let a baseline mask a new violation — the one forbidden bug). The
`finding` SHALL identify the offending element and its module: for a **call**, the resolved
canonical call path plus the call-site module; for a **glob** import (fail-closed), the glob's
import path plus its module. Distinct canonical call paths, or distinct glob imports, therefore
stay distinct findings; two textually-different calls resolving to the *same* canonical path in
the *same* module are the *same* violation (finding-level dedup, as the other module rules do —
not per-source-occurrence). The confined prefix SHALL be carried in the identity (in `target` or
in the `finding`), so two confinements with nested prefixes on the same subtree (e.g. `std` and
`std::time`) breached by the same call do not share an identity. The rule string alone SHALL NOT
be relied on to distinguish prefixes.

#### Scenario: Nested-prefix confinements do not mask each other
- **WHEN** `crate::core` is confined against both `std` and `std::time`, both breached by `std::time::Instant::now()`, and the `std` violation is in the baseline
- **THEN** the `std::time` violation still fails the reaction (exit 1) — the confined prefix is part of the identity, so baselining one prefix does not mask the other

#### Scenario: Two distinct calls in one module stay distinct
- **WHEN** `crate::core` calls both `std::time::Instant::now()` and `std::time::SystemTime::now()`, and only the first is in the baseline
- **THEN** the second still fails the reaction (finding is per-call: the resolved call path plus module, so one baselined call does not mask another)

### Requirement: CI reaction, severity, and baseline parity

The system SHALL fold inline-symbol-path findings into the same exit-code contract as the other
dimensions (`0` clean / `1` enforce violation / `2` constitution or scan error) and aggregate
them with the other boundaries. A boundary SHALL carry a severity (`enforce` default, `warn`
reports without failing), and its violations SHALL be gated against the same `Baseline` (identity
per "Identity distinguishes the confined prefix and the call"), so a project may adopt on a dirty
subtree and gate only on new calls.

#### Scenario: A warn boundary reports without failing
- **WHEN** a `warn`-severity inline-symbol-path boundary is violated and no enforce-severity boundary is violated
- **THEN** the system reports the violation but the reaction does not fail (exit 0)

#### Scenario: A new call beyond the baseline fails
- **WHEN** an enforce-severity boundary has an inline call not present in the baseline
- **THEN** the system fails the reaction (exit 1) for that new call
