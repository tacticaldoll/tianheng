## Why

圭表 governs *which* crates a target may depend on — by name (`forbid_dependency_on`,
`restrict_dependencies_to`) and by external/internal split (`deny_external_dependencies`). But
it cannot govern the **source kind a dependency declares**: a `git = "…"` dependency, or a
`path = "…"` dependency, is invisible to every existing rule even though `cargo metadata`
records the declared `source`. A crate prepared for `crates.io` wants a rigid line — its
manifest must declare no `git` (and perhaps no `path`) dependencies, since crates.io rejects a
published manifest that names a non-registry source (an *optional* git dependency blocks
publishing too). None of this is expressible today.

This change deepens 圭表's dependency-governance on the **same observation** it already uses
(`cargo metadata --no-deps` — the declared manifest) from *which crate* to *which declared
source kind*. It is the static-dimension counterpart to the v0.1.2 渾儀 depth addition
(dyn-trait), and the same "grow by depth, not width" axis: a new `Rule` variant on the proven
static engine, no new observation source, no new crate, and fully hermetic (a pure function of
the manifests).

## What Changes

- **New 圭表 capability — declared dependency-source boundary.** A `CrateBoundary` may restrict
  the **declared source kinds** of its dependencies, via a new `Rule` variant and a
  `restrict_dependency_sources_to([...])` builder over a `SourceKind { Registry, Git, Path }`
  allowlist. A dependency whose declared source kind is not in the allowlist is a violation.
  Optional dependencies are governed (they are declared regardless of feature state) — exactly
  the publishability-relevant case.
- **Declarative, not a lint.** The allowed source set is intent (a publishable infra crate
  declares `[Registry, Path]` to forbid `git`; a workspace tool may declare the opposite), with
  no universal right answer.
- **Additive only.** A new `#[non_exhaustive]` `Rule` variant (handled in the four exhaustive
  `Rule` matches) plus a new builder; no existing rule, signature, or projection behavior
  changes; the `--no-deps` observation is unchanged.

## Capabilities

### New Capabilities
- `crate-source-boundary`: a crate boundary may restrict the **declared source kinds**
  (`Registry` / `Git` / `Path`) of its dependencies, observed from each dependency's
  `cargo metadata` (declared) `source` field. The source-kind counterpart of
  `crate-dependency-boundary` (which governs dependency *names*); reuses the static engine's
  dependency walk, projection, baseline, and exit-code contract — only the per-rule `findings`
  classifier is new.

### Modified Capabilities
<!-- None. crate-dependency-boundary's requirements do not change; this is a new sibling Rule
     variant on the same --no-deps engine. -->

## Impact

- **Crate:** `guibiao` (圭表) only. New `SourceKind` enum + `Rule` variant + builder; the variant
  is handled in `Rule::{label, text, json_params, findings}` (the compiler enforces
  exhaustiveness — drift discipline). A new classifier reads each governed dependency's declared
  `source` (null → `Path`, `git+…` → `Git`, else → `Registry`). `serde_json`-only allowlist
  untouched.
- **Observation:** a **finer read of the same source** — the declared `dependency["source"]`,
  already inspected by `external_dependencies` for the external/internal split, now read for the
  git-vs-registry distinction. The `--no-deps` invocation is unchanged; the rule is **hermetic**
  (a pure function of the manifests, no lockfile or network).
- **Stated bounds:**
  - It governs the **declared** source kind, not the *resolved* one: a registry dependency
    redirected to git/path by `[patch]` / `[source] replace-with` reads as `Registry`. This is
    correct for the manifest-hygiene / publishability intent — `[patch]` is workspace-local, is
    not part of a published manifest, and does not block `cargo publish`. **Resolved build
    provenance** ("what my build actually pulls from, after patch") is a distinct future
    capability (B), born when built, reusing a resolve-graph read.
  - It is source-kind **hygiene**, not a `cargo publish` oracle: a `{ git = "…", version = "…" }`
    or `{ path = "…", version = "…" }` dependency declares a non-registry source yet publishes
    fine (the version is used at publish time); such a dependency is classified by its declared
    source (`Git`/`Path`) and flagged. The rule does not parse `version` keys. Stated, not
    silently implied.
- **SemVer:** additive, non-breaking → **0.1.2 patch**, bundled into the 0.1.2 release alongside
  dyn-trait-boundary.
