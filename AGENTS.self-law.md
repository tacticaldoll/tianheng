# Tianheng Self-Law Projection

Generated from `shengmo::law::constitution()` by `crates/shengmo/tests/self_governance.rs`.
**Do not edit by hand.** If this file is stale, regenerate it:
`BLESS=1 cargo test -p shengmo self_law_projection_is_fresh`.
If the law itself is wrong, amend `shengmo::law` through review — never edit this projection.
The law is named by module rather than by file here: this header registers the unit holding the
projection fresh, and a second tracked path in it would be an ambiguous claim about which one does.

Read the projection below as the imitable shape of Tianheng itself, and work *with* the reaction:

- Declare intent in Rust; the source is the single source of truth.
- Observe only what has a real observation source; name nothing that does not react.
- React with the outcomes: `0` clean, `1` violation, `2` constitution/usage error.
- On a violation, repair toward the boundary's declared reason — never weaken the law to pass.
- 三儀 (圭表 static · 渾儀 semantic · 漏刻 runtime) measure; 垂象 surfaces a reaction, 實錄 records one, 校讎 amends one.

# Constitution: tianheng

## Static boundaries

### `xuanji` (crate)

> 璇璣 is the dimension-agnostic reaction model: serde_json only, below every dimension, and must not depend on any workspace member

- **rule**: restrict dependencies to (only: serde_json)
- **kind**: crate · **severity**: enforce

### `xingbiao` (crate)

> 星表 is the shared metadata substrate: serde_json only, reading cargo metadata beneath the dimensions without depending on workspace members

- **rule**: restrict dependencies to (only: serde_json)
- **kind**: crate · **severity**: enforce

### `guibiao` (crate)

> the 圭表 static core stays dependency-light: serde_json, xuanji (reaction model), and xingbiao (metadata substrate) only. functional core ⊥ imperative shell: 圭表 must not depend on the 天衡 shell. 三儀 ⊥ 三儀: naming no sibling dimension, the observation dimensions are composed only by the 天衡 shell, never by each other

- **rule**: restrict dependencies to (only: serde_json, xuanji, xingbiao)
- **kind**: crate · **severity**: enforce

### `hunyi` (crate)

> 渾儀 is the semantic AST dimension: quarantined syn dependency only. 三儀 ⊥ 三儀: it depends on no sibling dimension and never on the 天衡 shell (functional dimension ⊥ imperative shell)

- **rule**: restrict dependencies to (only: xuanji, xingbiao, serde_json, syn)
- **kind**: crate · **severity**: enforce

### `louke` (crate)

> 漏刻 is the runtime dimension: it depends on 璇璣 and 星表 only. 三儀 ⊥ 三儀: naming no sibling dimension, it reacts in prod independently of the 天衡 shell

- **rule**: restrict dependencies to (only: xuanji, xingbiao)
- **kind**: crate · **severity**: enforce

### `tianheng` (crate)

> the 天衡 shell remains the outward composition layer: direct normal edges end at observation dimensions and projection serialization, never at the lower reaction model or metadata substrate

- **rule**: restrict dependencies to (only: guibiao, hunyi, louke, serde_json)
- **kind**: crate · **severity**: enforce

### `shengmo` (crate)

> 繩墨 declares this law through the shell alone: the allowlist is 天衡 and serde_json, so no edge to 圭表, 渾儀, 漏刻 or 璇璣 can exist and every boundary here is stated through the surface the shell publishes. serde_json reads cargo's own message stream, where a reaction's corpus comes from the build rather than from a list

- **rule**: restrict dependencies to (only: tianheng, serde_json)
- **kind**: crate · **severity**: enforce

### `kanhe` (crate)

> 勘合 names the shell, the law's own locator and serde_json, never a dimension: no edge to 圭表, 渾儀, 漏刻 or 璇璣 can exist, so a repository check reads the record through the same published surface an adopter has rather than through an observation engine

- **rule**: restrict dependencies to (only: shengmo, tianheng, serde_json)
- **kind**: crate · **severity**: enforce

### `xuanji::crate` (module)

> 璇璣 is the measure-only reaction model: it reads no ambient clock inline and exposes no async surface — time and effects enter only through the dimensions above it, never the model itself

- **rule**: inline symbol path confined to module (confined_prefix: std::time; ending_with: now)
- **kind**: module · **severity**: enforce · **crate**: xuanji

### `guibiao::crate` (module)

> path canonicalization and cycle/dedup guards in guibiao must resolve through `xingbiao::canonicalize_or_fail` or `try_visit` for unified failure handling

- **rule**: inline symbol path confined to module (confined_prefix: std::fs; ending_with: canonicalize)
- **kind**: module · **severity**: enforce · **crate**: guibiao

### `hunyi::crate` (module)

> path canonicalization and cycle/dedup guards in hunyi must resolve through `xingbiao::canonicalize_or_fail` or `try_visit` for unified failure handling

- **rule**: inline symbol path confined to module (confined_prefix: std::fs; ending_with: canonicalize)
- **kind**: module · **severity**: enforce · **crate**: hunyi

### `louke::crate` (module)

> path canonicalization and cycle/dedup guards in louke must resolve through `xingbiao::try_visit` for unified failure handling

- **rule**: inline symbol path confined to module (confined_prefix: std::fs; ending_with: canonicalize)
- **kind**: module · **severity**: enforce · **crate**: louke

## Async-exposure boundaries

### `xuanji::crate` (semantic)

> 璇璣 is the measure-only reaction model: it reads no ambient clock inline and exposes no async surface — time and effects enter only through the dimensions above it, never the model itself

- **rule**: must not expose async fn (including_submodules: true; scan_depth: subtree)
- **kind**: semantic · **severity**: enforce · **crate**: xuanji
