# Tianheng Self-Law Projection

Generated from `tianheng_constitution()` in `crates/tianheng/tests/self_governance.rs`.
**Do not edit by hand.** If this file is stale, regenerate it:
`BLESS=1 cargo test -p tianheng self_law_projection_is_fresh`.
If the law itself is wrong, amend `self_governance.rs` through review — never edit this projection.

Read the projection below as the imitable shape of Tianheng itself, and work *with* the reaction:

- Declare intent in Rust; the source is the single source of truth.
- Observe only what has a real observation source; name nothing that does not react.
- React with the outcomes: `0` clean, `1` violation, `2` constitution/usage error.
- On a violation, repair toward the boundary's declared reason — never weaken the law to pass.
- 三儀 (圭表 static · 渾儀 semantic · 漏刻 runtime) measure; 垂象 surfaces a reaction, 實錄 records one, 校讎 amends one.

# Constitution: tianheng

## Static boundaries

### `xuanji`

> 璇璣 is the dimension-agnostic reaction model: serde_json only, below every dimension, and must not depend on any workspace member

- **rule**: restrict dependencies to (only: serde_json)
- **kind**: crate · **severity**: enforce

### `xingbiao`

> 星表 is the shared metadata substrate: serde_json only, reading cargo metadata beneath the dimensions without depending on workspace members

- **rule**: restrict dependencies to (only: serde_json)
- **kind**: crate · **severity**: enforce

### `guibiao`

> the 圭表 static core stays dependency-light: serde_json, xuanji (reaction model), and xingbiao (metadata substrate) only. functional core ⊥ imperative shell: 圭表 must not depend on the 天衡 shell. 三儀 ⊥ 三儀: naming no sibling dimension, the observation dimensions are composed only by the 天衡 shell, never by each other

- **rule**: restrict dependencies to (only: serde_json, xuanji, xingbiao)
- **kind**: crate · **severity**: enforce

### `hunyi`

> 渾儀 is the semantic AST dimension: quarantined syn dependency only. 三儀 ⊥ 三儀: it depends on no sibling dimension and never on the 天衡 shell (functional dimension ⊥ imperative shell)

- **rule**: restrict dependencies to (only: xuanji, xingbiao, serde_json, syn)
- **kind**: crate · **severity**: enforce

### `louke`

> 漏刻 is the runtime dimension: hot path depends on 璇璣 only, with xingbiao audit-gated for CI probe coverage. 三儀 ⊥ 三儀: naming no sibling dimension, it reacts in prod independently of the 天衡 shell

- **rule**: restrict dependencies to (only: xuanji, xingbiao)
- **kind**: crate · **severity**: enforce

### `tianheng`

> the 天衡 shell remains the outward composition layer: direct normal edges end at observation dimensions and projection serialization, never at the lower reaction model or metadata substrate

- **rule**: restrict dependencies to (only: guibiao, hunyi, louke, serde_json)
- **kind**: crate · **severity**: enforce

### `shengmo`

> 繩墨 is an adopter of 天衡, not a member of the family it governs: it declares this law through the shell's published surface and reaches no dimension directly, so the repository's own governance exercises exactly the surface an adopter has

- **rule**: restrict dependencies to (only: tianheng)
- **kind**: crate · **severity**: enforce

### `crate`

> 璇璣 is the measure-only reaction model: it reads no ambient clock inline and exposes no async surface — time and effects enter only through the dimensions above it, never the model itself

- **rule**: inline symbol path confined to module (confined_prefix: std::time; ending_with: now)
- **kind**: module · **severity**: enforce · **crate**: xuanji

### `crate`

> path canonicalization and cycle/dedup guards in guibiao must resolve through `xingbiao::canonicalize_or_fail` or `try_visit` for unified failure handling

- **rule**: inline symbol path confined to module (confined_prefix: std::fs; ending_with: canonicalize)
- **kind**: module · **severity**: enforce · **crate**: guibiao

### `crate`

> path canonicalization and cycle/dedup guards in hunyi must resolve through `xingbiao::canonicalize_or_fail` or `try_visit` for unified failure handling

- **rule**: inline symbol path confined to module (confined_prefix: std::fs; ending_with: canonicalize)
- **kind**: module · **severity**: enforce · **crate**: hunyi

### `crate`

> path canonicalization and cycle/dedup guards in louke must resolve through `xingbiao::try_visit` for unified failure handling

- **rule**: inline symbol path confined to module (confined_prefix: std::fs; ending_with: canonicalize)
- **kind**: module · **severity**: enforce · **crate**: louke

## Async-exposure boundaries

### `crate`

> 璇璣 is the measure-only reaction model: it reads no ambient clock inline and exposes no async surface — time and effects enter only through the dimensions above it, never the model itself

- **rule**: must not expose async fn (including_submodules: true; scan_depth: subtree)
- **kind**: semantic · **severity**: enforce · **crate**: xuanji
