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
- 三儀 (圭表 static · 渾儀 semantic · 漏刻 runtime) measure; 三司 (垂象 · 實錄 · 校讎) administer.

# Constitution: tianheng

## Static boundaries

### `xuanji`

> 璇璣 is the dimension-agnostic reaction model: serde_json only, and below every dimension — it must not depend on any workspace member (no engine, no shell), so nothing in the family sits beneath it

- **rule**: restrict dependencies to (only: serde_json)
- **kind**: crate · **severity**: enforce

### `xingbiao`

> 星表 is the shared declared-workspace-data substrate: serde_json only, and below every dimension like 璇璣 — it reads `cargo metadata` and must not depend on any workspace member, so the static and semantic dimensions read the workspace through one source of truth, not two hand-copied twins that drift apart. Not 璇璣: it does IO (spawns cargo) and observes, so it is not the measure-only reaction model — a substrate beneath the dimensions, not the measure they react in

- **rule**: restrict dependencies to (only: serde_json)
- **kind**: crate · **severity**: enforce

### `guibiao`

> the 圭表 core stays dependency-light: serde_json is the only external dependency (no syn / proc-macro, no heavy graph or runtime crates); the internal dependencies on 璇璣 (the shared reaction model) and 星表 (the shared metadata substrate) are the price of the family split — both are serde_json-only bases below the dimensions: the model renders no verdict and the substrate only reads the workspace, neither drags in an engine. 三儀 ⊥ 三儀: this allowlist names no sibling dimension, so 圭表 cannot depend on 渾儀 (nor, when born, 漏刻) — the dimensions are composed only by the 天衡 shell, never by each other

- **rule**: restrict dependencies to (only: serde_json, xuanji, xingbiao)
- **kind**: crate · **severity**: enforce

### `guibiao`

> functional core ⊥ imperative shell: the 圭表 core crate must not depend on the 天衡 gate/shell

- **rule**: forbid dependency on (crates: tianheng)
- **kind**: crate · **severity**: enforce

### `hunyi`

> 渾儀 is the semantic dimension and the sole holder of the heavy syn AST dependency — quarantined here, never the core or the model; it depends on 璇璣 (the reaction model), 星表 (the shared metadata substrate), serde_json, and syn only. 三儀 ⊥ 三儀: it never depends on the sibling 圭表 dimension (nor, when born, 漏刻), and never on the 天衡 shell — the dimensions are composed only by the shell, never by each other (functional dimension ⊥ imperative shell)

- **rule**: restrict dependencies to (only: xuanji, xingbiao, serde_json, syn)
- **kind**: crate · **severity**: enforce

### `louke`

> 漏刻 is the runtime dimension and ships into the user's production binary, so its hot path stays production-light: it depends on 璇璣 (the reaction model) only — no syn, no static engine, no sibling dimension. 星表 is an additive, `audit`-feature-gated exception (never reaches the production hot path): the CI-only probe scanner's own cycle guard routes through 星表's shared canonicalize/cycle-guard primitives, the same ones 圭表/渾儀 already use, rather than carrying a third independently hand-rolled copy. 三儀 ⊥ 三儀: naming no sibling, it cannot depend on the 圭表/渾儀 dimensions, and it reacts in prod independently of the 天衡 shell (serde_json reaches it only transitively via 璇璣, cold-path only)

- **rule**: restrict dependencies to (only: xuanji, xingbiao)
- **kind**: crate · **severity**: enforce

### `tianheng`

> the 天衡 shell composes the 三儀 into one reaction, so it depends on the 圭表 static core, the 渾儀 semantic dimension, and the 漏刻 runtime dimension (whose CI probe-coverage face it composes into `check`), reads exact Cargo target roots through the shared 星表 substrate, and projects with serde_json; all edges point to dimensions or shared bases beneath the shell

- **rule**: restrict dependencies to (only: guibiao, hunyi, louke, serde_json, xingbiao)
- **kind**: crate · **severity**: enforce

### `crate`

> 璇璣 is the measure-only reaction model: it reads no ambient clock inline and exposes no async surface — time and effects enter only through the dimensions above it, never the model itself. The clock axis reacts via 圭表 (must-not-call-inline `std::time::…::now`), the async axis via 渾儀 (must-not-expose an async public fn)

- **rule**: inline symbol path confined to module (confined_prefix: std::time; ending_with: now)
- **kind**: module · **severity**: enforce · **crate**: xuanji

### `crate::module_resolve`

> path canonicalization for this resolver's own cycle/dedup guard must go through the shared, fail-loud `xingbiao::try_visit`, never be re-hand-rolled inline here — the 0.2.2 lesson (a canonicalize-failure policy hand-rolled per call site drifted to disagreeing behavior across this crate)

- **rule**: inline symbol path confined to module (confined_prefix: std::fs; ending_with: canonicalize)
- **kind**: module · **severity**: enforce · **crate**: hunyi

### `crate::module_scan::reachability`

> path canonicalization for this walker's own cycle/dedup guard must go through the shared, fail-loud `xingbiao::canonicalize_or_fail`/`try_visit`, never be re-hand-rolled inline here — the 0.2.2 lesson (this exact file once carried three disagreeing canonicalize-failure policies at once)

- **rule**: inline symbol path confined to module (confined_prefix: std::fs; ending_with: canonicalize)
- **kind**: module · **severity**: enforce · **crate**: guibiao

### `crate::scan`

> path canonicalization for this crate-wide walker's own cycle/dedup guard must go through the shared, fail-loud `xingbiao::canonicalize_or_fail`, never be re-hand-rolled inline here — a sibling instance of the 0.2.2 lesson found in this same crate's `module_resolve` (a second, independently hand-rolled wrapper here once carried its own disagreeing error-message policy)

- **rule**: inline symbol path confined to module (confined_prefix: std::fs; ending_with: canonicalize)
- **kind**: module · **severity**: enforce · **crate**: hunyi

### `crate::audit::scan`

> this CI-only probe scanner's module-cycle guard must go through the shared, fail-loud `xingbiao::try_visit`, never be re-hand-rolled inline here — closes the same class of drift 圭表/渾儀's own guards were confined against, now that 漏刻's self-law permits the additive, `audit`-feature-gated `xingbiao` dependency this routes through

- **rule**: inline symbol path confined to module (confined_prefix: std::fs; ending_with: canonicalize)
- **kind**: module · **severity**: enforce · **crate**: louke

## Async-exposure boundaries

### `crate`

> 璇璣 is the measure-only reaction model: it reads no ambient clock inline and exposes no async surface — time and effects enter only through the dimensions above it, never the model itself. The clock axis reacts via 圭表 (must-not-call-inline `std::time::…::now`), the async axis via 渾儀 (must-not-expose an async public fn)

- **rule**: must not expose async fn (including_submodules: true)
- **kind**: semantic · **severity**: enforce · **crate**: xuanji

