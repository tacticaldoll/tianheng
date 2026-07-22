# crate-source-boundary Specification

## Purpose
The 圭表 (static) capability that governs the **declared source kind** of a crate's
dependencies — git vs. registry vs. path — read from each dependency's `cargo metadata`
(`--no-deps`) `source` field. It is the source-kind complement of `crate-dependency-boundary`
(which governs dependency *names*): where that answers "which crates may I depend on", this
answers "from which kind of source may they be declared". A crate prepared for crates.io
declares `[Registry, Path]` to forbid any git source (an *optional* git dependency blocks
publishing too). It deepens the proven static engine on the *same* observation — a finer read
of the manifest, no new source, no new crate, hermetic (a pure function of the manifests).

## Requirements

### Requirement: Dependency-source boundary declared in Rust

A dependency-source boundary SHALL be expressed as Rust code on a `CrateBoundary`, part of the
single source of truth, composed with the other dimensions at the gate. It SHALL name a target
crate and a closed allowlist of **source kinds** — `Registry`, `Git`, `Path` — its dependencies
may **declare**, via `restrict_dependency_sources_to([...])`, a human-readable reason, and a
severity. A dependency whose declared source kind is not in the allowlist is a violation. An
empty allowlist forbids every dependency (of the governed kind). The system MUST NOT require
TOML, YAML, Markdown, or any generated policy file to declare or run the boundary.

#### Scenario: Source boundary declared in Rust

- **WHEN** a developer writes `CrateBoundary::crate_("infra").restrict_dependency_sources_to([SourceKind::Registry, SourceKind::Path]).because("infra must publish to crates.io, so its manifest declares no git dependencies")`
- **THEN** a crate boundary is held, targeting `infra`, permitting only `Registry` and `Path` declared source kinds, with a non-empty reason and a default `enforce` severity, ready to be composed with the static dimension at the gate

### Requirement: Declared source kind classified from cargo metadata

The system SHALL classify each governed dependency's source kind from its `cargo metadata`
(`--no-deps`) declared `source` field: a **null** source is `Path`; a source beginning `git+` is
`Git`; any other non-null source is `Registry` (covering `registry+`, `sparse+`, and alternative
registries as the residual). Only `Git` is matched by a positive prefix and only `Path` by null;
`Registry` is the residual. The reaction is **hermetic** — a pure function of the manifests, with
no lockfile read and no network.

#### Scenario: A path dependency classifies as Path

- **WHEN** a governed dependency has a null declared `source` (a `path = "…"` dependency)
- **THEN** the system classifies it as `Path`

#### Scenario: A git dependency classifies as Git

- **WHEN** a governed dependency's declared `source` begins with `git+`
- **THEN** the system classifies it as `Git`

#### Scenario: A registry or alternative-registry dependency classifies as Registry

- **WHEN** a governed dependency's declared `source` is non-null and does not begin with `git+` (e.g. `registry+https://…` or `sparse+https://…`)
- **THEN** the system classifies it as `Registry`, as the residual kind

### Requirement: A dependency outside the allowed source set is a violation

The system SHALL emit a violation for each governed dependency whose declared source kind is not
in the boundary's allowlist, and SHALL report no violation when every governed dependency's
declared source kind is allowed. The governed surface is the target's dependencies of the
boundary's `DependencyKind` (default `Normal`, settable), mirroring the sibling dependency rules;
dependencies of other kinds are not observed. **Optional** dependencies ARE governed — they are
declared regardless of feature state, which is the publishability-relevant case (a published
manifest naming a git source is rejected even when the dependency is optional). The finding is the
dependency's **real package name** (not its local rename), matching the sibling rules.

#### Scenario: A git dependency violates a registry-or-path allowlist

- **WHEN** the target declares a `git = "…"` dependency and the boundary permits only `[Registry, Path]`
- **THEN** the system emits a violation naming that dependency, because its declared source kind `Git` is outside the allowlist

#### Scenario: A path dependency violates a registry-only allowlist

- **WHEN** the target declares a `path = "…"` dependency and the boundary permits only `[Registry]`
- **THEN** the system emits a violation naming that dependency, because its declared source kind `Path` is outside the allowlist

#### Scenario: An optional git dependency is governed

- **WHEN** the target declares an `optional = true` git dependency and the boundary permits only `[Registry, Path]`
- **THEN** the system emits a violation, because an optional dependency is declared regardless of feature state and a published manifest naming a git source is rejected even when optional

#### Scenario: A renamed git dependency reports its real package name

- **WHEN** the target declares `mydep = { git = "…", package = "serde" }` and the boundary permits only `[Registry]`
- **THEN** the system emits a violation whose finding is the real package name `serde`, matching how the sibling name-based rules report a dependency

#### Scenario: An inherited workspace git dependency is governed

- **WHEN** the target declares `dep = { workspace = true }`, the workspace root declares that dependency with a `git = "…"` source, and the boundary permits only `[Registry, Path]`
- **THEN** the system emits a violation, because the member's metadata reads the inherited source as `git+…` (cargo flattens an inherited dependency into the member's published manifest, carrying the git source) — so the publishability-relevant source is governed even when declared by inheritance

#### Scenario: All-registry dependencies satisfy a registry allowlist

- **WHEN** every governed (normal) dependency declares a registry source and the boundary permits `[Registry]`
- **THEN** the system reports no violation for that boundary

#### Scenario: A dev-dependency is not governed by a normal-scoped boundary

- **WHEN** the target declares a `git` dev-dependency and the boundary is `Normal`-scoped permitting `[Registry]`
- **THEN** the system reports no violation, because the dev-dependency table is not the governed surface

### Requirement: Stated bounds — declared not resolved, hygiene not publish oracle

The boundary SHALL govern the **declared** source kind and SHALL state two bounds rather than
silently overreach. (1) It does not observe the **resolved** source: a registry dependency
redirected to git/path by `[patch]` or `[source] replace-with` declares a `registry+` source and
SHALL classify as `Registry` (no violation) — correct for the manifest-hygiene intent, since
`[patch]` is workspace-local, is not part of a published manifest, and does not block
`cargo publish`. Observing the **resolved** source is out of scope for Tianheng — that
whole-graph build-provenance concern is supply-chain tooling's lane (cargo-deny's `[sources]`,
which reads the resolved graph), not a future Tianheng capability. (2) It is not a
`cargo publish` oracle: a `{ git/path = "…", version = "…" }` dependency declares a non-registry
source yet publishes successfully; the system SHALL classify it by its declared source (`Git`/
`Path`) and flag it, NOT parse the `version` key — a deliberately conservative hygiene guard.

#### Scenario: A patch-redirected dependency is governed by its declared registry source

- **WHEN** the target declares a registry dependency that `[patch]` redirects to a `git` source, and the boundary permits only `[Registry]`
- **THEN** the system classifies it as `Registry` (its declared source is `registry+…`) and reports no violation — the declared layer does not observe the patch, which is correct here because `[patch]` does not block publishing; observing the resolved git source is supply-chain tooling's lane (cargo-deny's `[sources]`), outside Tianheng's declared per-target layer

#### Scenario: A git-plus-version dependency is flagged though it would publish

- **WHEN** the target declares `dep = { git = "…", version = "…" }` and the boundary permits only `[Registry]`
- **THEN** the system classifies it as `Git` (its declared source is `git+…`) and emits a violation — even though such a dependency would `cargo publish` successfully — because the rule governs the declared source kind, a stated conservative bound, not publish-eligibility

### Requirement: CI reaction, severity, baseline, and projection parity

The dependency-source boundary SHALL share the 圭表 reaction contract with the existing crate
dependency rules: findings fold into the same aggregated report and exit-code outcome (**0** clean,
**1** enforce violation, **2** constitution/scan error such as an unreadable workspace or an absent
target crate); the boundary carries a severity (`enforce` default, or `warn`) and is gated against
the same `Baseline` under the shared violation identity `(target, rule, finding_key)`, the finding being
the offending dependency's real package name (kind-qualified with a ` (dev)`/` (build)` suffix for a
non-`Normal` boundary, per `crate-dependency-boundary`'s Dependency kind selection); and the rule projects through the existing generic
`CrateBoundary` text/JSON/markdown projection, its parameters being the allowed source-kind list.
The implementation SHALL keep `guibiao` `serde_json`-only (no new dependency) and SHALL NOT change
the `--no-deps` invocation.

#### Scenario: A source violation fails CI

- **WHEN** an enforce-severity source boundary is violated
- **THEN** the system prints a report naming the target, the rule, the offending dependency, and the reason, and exits 1 (the allowed source kinds appear in the `list` projection, not the violation report — as for the other crate rules)

#### Scenario: An absent target crate is a constitution error

- **WHEN** a source boundary targets a crate not present in the workspace
- **THEN** the system emits a constitution/scan error and exits 2, never exit 0 and never exit 1

#### Scenario: Severity and baseline behave as for the other crate rules

- **WHEN** a `warn`-severity source boundary is violated and no enforce boundary is, or an enforce boundary's only violations are all in the baseline
- **THEN** the reaction does not fail (exit 0); and a source violation not present in the baseline fails the reaction (exit 1)

#### Scenario: The rule projects in list output

- **WHEN** the constitution is projected via `list` (text/json/markdown)
- **THEN** the source boundary appears with its target, rule label, allowed source kinds, severity, and reason — through the existing generic crate-boundary projection, no separate projector

### Requirement: Source-policy reactions use semantic identity

Every source-policy violation SHALL identify its governed target, structured rule key (including
identity-bearing policy roles), and dimension-owned source fact. Presentation and policy rendering
SHALL remain outside identity.

#### Scenario: Source presentation changes without re-keying
- **WHEN** only the displayed source-policy wording changes
- **THEN** an existing target/rule/fact baseline match remains valid
