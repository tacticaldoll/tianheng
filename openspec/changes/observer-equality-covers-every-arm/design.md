# Design

## One array, or three hand-written arms

Both reactions are three-way. Written as three hand-written arms — which is what they are today — a fourth
dimension arrives and one arm is forgotten, and the forgotten one is the arm that silently proves nothing. So
the dimensions become one array:

```rust
struct Dimension {
    label: &'static str,
    /// Declares a boundary of this dimension that this workspace VIOLATES.
    declare: fn(Constitution) -> Constitution,
    /// Folds this dimension's observer into a run, reading its own boundaries from the constitution.
    fold: for<'a> fn(Run<'a>, &Constitution) -> Run<'a>,
    /// A violation of this kind proves this dimension's arm fired.
    reacted: fn(BoundaryKind) -> bool,
    /// The observer's own declaration, and the dimension's export, for the bijection.
    declared_bounds: fn() -> Vec<BoundDecl>,
    exported_bounds: fn() -> Vec<BoundDecl>,
}
```

`reacted` is a predicate rather than a `BoundaryKind` because the static dimension owns two kinds (`Crate` and
`Module`), and because `BoundaryKind` is `#[non_exhaustive]` — a downstream crate cannot match it exhaustively,
so a predicate is the only honest spelling. That is also why this design does **not** try to derive the
dimension set from `BoundaryKind`'s variants: the compiler will not help here, and a hand-kept count of the
array's length asserted beside the array is the `declared-set-beside-its-enumerator` defect wearing a different
hat.

What guards a **deleted** entry instead is the constitution itself. After the array has declared everything, all
three of `static_boundaries()`, `semantic_boundaries()` and `runtime_boundaries()` must be non-empty. A deleted
entry empties its dimension's accessor, and that assertion fires — checked against the object, not counted by
hand.

## The array's order is part of the comparison

`Run::observe` folds eagerly, and the requirement *Assembly order is semantically observable* already says so:
order decides which cannot-judge is reported, and `merge_outcomes` concatenates violations in fold order. The
compared strings are `Debug` renderings of the merged reports, so the array must be in the built-in path's
order — 圭表, then 渾儀, then 漏刻. Reordering the array is therefore not a cosmetic edit, and the array carries
that as a comment where someone would be tempted to sort it.

Now that the fixture is non-empty in every dimension, the built-in path's `semantic_boundaries().is_empty()`
short-circuit no longer fires for it. That was the one behavioural difference between the two paths reachable
from an empty declaration, and it was never exercised before, because before this change the fixture always took
the skipping branch.

## Which violating boundary per dimension

Each has to be a boundary this workspace genuinely breaks, found by running it rather than by reading:

| Dimension  | Declaration                                                        | Measured reaction |
| ---------- | ------------------------------------------------------------------ | ----------------- |
| 圭表 static | `CrateBoundary::crate_("xuanji").restrict_dependencies_to(["syn"])` | its real `serde_json` edge falls outside the allowlist |
| 渾儀 semantic | `ImplTraitBoundary::in_crate("hunyi").module("crate")`             | `SemanticBoundaries::crate_packages` returns `impl Iterator<Item = &str>` |
| 漏刻 runtime | `RuntimeBoundary::at(<a seam name nothing probes>)`                 | declared-but-unprobed |

The semantic one is the *narrowest* reacting declaration found: one violation, from one public method. A
visibility ceiling on `xuanji`'s root was tried first and produces eight, which makes a failure message hard to
read without proving anything more.

The runtime one is chosen so it cannot become accidentally satisfied: a seam name no probe in the tree names
reacts as declared-but-unprobed, and the only way to make it stop reacting is to add a probe for a seam invented
for this fixture. An empty runtime declaration was measured first and is `Clean` — which is the whole defect
this change is about, so it is recorded here rather than left as a thing to rediscover.

## Delegation, not a shared helper

`evaluate_constitution`'s runtime arm becomes
`RuntimeObserver::new(constitution.runtime_boundaries().to_vec()).observe(manifest_path)`.

The alternative was a third function both call. Rejected: it adds public surface to `louke` for the benefit of
one internal caller, and it states something weaker. "The built-in path is the observer for this dimension"
cannot drift; "two callers of one helper" can still drift in what each caller does around the call. The `to_vec`
is a handful of small structs once per run.

This is the one place the change trades an observed property for a constructed one, so the spec names it. The
static and semantic arms stay independently implemented — `check_and_cover` carries the coverage advisory the
protocol has no notion of, and collapsing it would double this workspace's single `cargo metadata` read.
