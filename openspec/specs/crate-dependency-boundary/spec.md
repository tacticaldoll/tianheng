# crate-dependency-boundary Specification

## Purpose

Detect crate-dependency drift: compare each declared `Boundary` against the
observed dependencies of its target crate (via `cargo metadata`) and react with a
distinct CI exit code — clean, boundary violation, or constitution error — never a
silent pass. This is Tianheng's first reaction and the proof of its core contract: a
declared boundary in Rust produces a real, non-bypassable reaction when violated.
## Requirements
### Requirement: Constitution declared in Rust

The constitution SHALL be expressed as Rust code and is the single source of
truth. A `Constitution` SHALL carry a name and a list of `Boundary` values. The
system MUST NOT require TOML, YAML, Markdown, or any generated policy file to
declare or run a boundary.

#### Scenario: Boundary declared in Rust

- **WHEN** a developer writes `Constitution::new("example").boundary(CrateBoundary::crate_("example-core").deny_external_dependencies().because("..."))`
- **THEN** the constitution holds one boundary targeting the crate `example-core` with rule `DenyExternalDependencies` and a non-empty reason

### Requirement: Target resolution

For each boundary, the system SHALL resolve the named `CrateTarget` to a real
package in the target Cargo workspace before evaluating its rule. If the target
crate cannot be found in the workspace, the system SHALL treat this as a
**constitution error** (a misconfiguration) — failing loud with a distinct exit
code rather than a silent pass, and distinct from a boundary violation so a
mistyped crate name is not reported as architectural drift.

#### Scenario: Target resolves to a workspace package

- **WHEN** a boundary targets `example-core` and that package exists in the workspace metadata
- **THEN** the system observes that package's dependencies for comparison

#### Scenario: Unresolvable target is a constitution error

- **WHEN** a boundary targets a crate name that is not present in the workspace metadata
- **THEN** the system emits a constitution error reporting the unresolved target and exits with code 2 (distinct from a boundary violation, which is exit 1)

### Requirement: External dependency classification

The `DenyExternalDependencies` rule SHALL classify the target crate's dependencies in
its selected table (normal `[dependencies]` by default) by source: a dependency
resolving to a registry or git source is external; a dependency resolving to a
workspace path is internal and allowed. The rule SHALL consider only the boundary's
selected dependency kind (see Dependency kind selection); tables other than the
selected one are out of scope.

#### Scenario: External dependency violates the boundary

- **WHEN** the target crate declares a normal dependency whose source is a registry or git source
- **THEN** the system emits a violation naming that dependency

#### Scenario: Internal path dependency is allowed

- **WHEN** the target crate declares only dependencies that resolve to workspace paths
- **THEN** the system reports no external-dependency violation for that boundary

#### Scenario: Dev and build dependencies are ignored by default

- **WHEN** the target crate declares a registry dependency only under `[dev-dependencies]` or `[build-dependencies]` and the boundary selects the default normal kind
- **THEN** the system does not emit an external-dependency violation

### Requirement: CI reaction

The system SHALL distinguish three outcomes by exit code so a CI gate can tell
architectural drift from misconfiguration: **exit 0** when no boundary is
violated; **exit 1** when one or more boundaries are violated; **exit 2** for a
constitution or scan error (e.g. an unresolvable target or an unreadable target
workspace). For exit 1 and exit 2 the system SHALL print a human-readable report.

#### Scenario: Clean target passes

- **WHEN** the target crate has no external dependencies
- **THEN** the system reports that the boundary is satisfied and exits 0

#### Scenario: Violation fails CI

- **WHEN** one or more boundaries are violated
- **THEN** the system prints a report and exits 1

#### Scenario: Misconfiguration is distinguishable from violation

- **WHEN** evaluation cannot proceed because a target is unresolvable or the workspace cannot be read
- **THEN** the system prints a constitution/scan error and exits 2, never exit 0 (no silent pass) and never exit 1 (not reported as a boundary violation)

### Requirement: Human-readable violation report

A violation report SHALL identify the target crate, the rule, the offending
finding (e.g. the dependency name), and a human-readable reason supplied with
the boundary, and SHALL state that the reaction failed.

#### Scenario: Report explains the violation

- **WHEN** the target crate `example-core` declares the external dependency `serde`
- **THEN** the report names the target `example-core`, the rule "deny external dependencies", the finding `serde`, the boundary's reason, and indicates CI failed

### Requirement: Multiple boundaries

A constitution MAY declare more than one boundary. When it does, the system SHALL
evaluate every boundary and aggregate all resulting violations into a single
report. A constitution error on any boundary (an unresolvable target) SHALL
supersede any boundary violation found in the same run: the run reports a
constitution error (exit 2), not a violation (exit 1), because a boundary that
could not be evaluated makes the run's verdict untrustworthy. The order in which
boundaries are declared SHALL NOT change the outcome class.

#### Scenario: Violations across boundaries are aggregated

- **WHEN** a constitution declares two boundaries targeting two different crates that each declare an external dependency
- **THEN** the report contains a violation for each crate, and the system exits 1

#### Scenario: A constitution error supersedes a violation

- **WHEN** a constitution declares one boundary that is violated and one boundary whose target crate does not exist in the workspace
- **THEN** the system reports a constitution error and exits 2, not a violation (exit 1)

#### Scenario: Order of boundaries does not change the outcome class

- **WHEN** the same set of boundaries is evaluated in any order and at least one target is unresolvable
- **THEN** the system reports a constitution error (exit 2) regardless of declaration order

### Requirement: External dependency allowlist

The `DenyExternalDependencies` rule SHALL support an optional allowlist of crate names. An external dependency whose name is in the allowlist SHALL NOT be reported as a violation; an external dependency not in the allowlist SHALL still be a violation. A rule declared with no allowlist SHALL behave exactly as v0.1 (deny every external dependency). The allowlist SHALL apply only to the same normal `[dependencies]` the rule already classifies.

#### Scenario: An allowed external dependency is not a violation

- **WHEN** the target crate declares the external dependency `serde` and the boundary allows `serde`
- **THEN** the system reports no violation for that dependency

#### Scenario: A non-allowed external dependency still violates

- **WHEN** the target crate declares the external dependency `serde` and the boundary's allowlist does not contain `serde`
- **THEN** the system emits a violation naming `serde`

### Requirement: Forbid dependency on named crates

A boundary SHALL support a rule that forbids a dependency on specific named crates. A dependency in the boundary's selected table (normal `[dependencies]` by default) whose name matches a forbidden name SHALL be a violation, whether that dependency resolves to an external source or to an internal workspace path. This enables both deny-specific-crate and crate → crate layering ("core must not depend on adapters"). Tables other than the selected dependency kind SHALL be out of scope (see Dependency kind selection).

#### Scenario: A forbidden external crate is a violation

- **WHEN** the target crate declares the external dependency `serde` and the boundary forbids `serde`
- **THEN** the system emits a violation naming `serde` and exits 1

#### Scenario: A forbidden internal crate is a violation (layering)

- **WHEN** the target crate `core` declares a workspace path dependency on `adapters` and the boundary forbids `adapters`
- **THEN** the system emits a violation naming `adapters`, even though that dependency is internal and the external rule would ignore it

#### Scenario: A crate that is not depended on is clean

- **WHEN** the boundary forbids a crate the target does not depend on
- **THEN** the system reports no violation for that boundary

### Requirement: Boundary severity

A boundary SHALL carry a severity that controls how its violations react: `enforce` or `warn`. A boundary declared without an explicit severity SHALL default to `enforce`, preserving prior behavior. The violations of a `warn`-severity boundary SHALL still be observed and reported, but SHALL NOT by themselves cause the reaction to fail; a `warn` boundary is the advisory rung of adoption, observed before it is enforced. Severity SHALL be declared in Rust alongside the boundary, never in a separate file.

#### Scenario: A warn boundary's violation is reported but does not fail

- **WHEN** a `warn`-severity boundary is violated and no `enforce`-severity boundary is violated
- **THEN** the system reports the violation but the reaction does not fail (the exit code is 0)

#### Scenario: An enforce boundary still fails

- **WHEN** an `enforce`-severity boundary is violated
- **THEN** the system reports the violation and the reaction fails (exit 1), regardless of any warn boundaries

#### Scenario: A boundary defaults to enforce

- **WHEN** a boundary is declared without an explicit severity and is violated
- **THEN** the reaction fails (exit 1), exactly as before this capability existed

### Requirement: Restrict dependencies to an allowlist

A boundary SHALL support a rule that restricts the target crate's dependencies in its selected table (normal `[dependencies]` by default) to a closed allowlist of crate names. Every dependency in that table whose name is not in the allowlist SHALL be a violation, whether it resolves to an external source or to an internal workspace path. An empty allowlist SHALL forbid every dependency in that table (stricter than the deny-external rule, which still permits internal path dependencies). Tables other than the selected dependency kind SHALL be out of scope (see Dependency kind selection). The rule SHALL carry severity and react through the report, baseline, and projection exactly as the existing crate rules do.

#### Scenario: A dependency outside the allowlist is a violation

- **WHEN** the target crate declares a normal dependency `serde` and the boundary restricts dependencies to `["other"]`
- **THEN** the system emits a violation naming `serde` and exits 1

#### Scenario: A dependency inside the allowlist is clean

- **WHEN** the target crate's only normal dependency is `serde` and the boundary restricts dependencies to `["serde"]`
- **THEN** the system reports no violation for that boundary

#### Scenario: An internal path dependency outside the allowlist is a violation

- **WHEN** the target crate `core` declares a workspace path dependency on `adapters` and the boundary restricts dependencies to an allowlist that omits `adapters`
- **THEN** the system emits a violation naming `adapters`, even though it is internal and the deny-external rule would ignore it

#### Scenario: An empty allowlist forbids every dependency

- **WHEN** the target crate declares any normal dependency and the boundary restricts dependencies to `[]`
- **THEN** the system emits a violation for that dependency

### Requirement: Restrict workspace dependencies to an allowlist

A boundary SHALL support a rule that restricts the target crate's dependencies on **other workspace members** to a closed allowlist of crate names, where workspace membership is observed from `cargo metadata`. A dependency in the boundary's selected table (normal `[dependencies]` by default) whose resolved package is another workspace member and whose name is not in the allowlist SHALL be a violation; an empty allowlist SHALL forbid every workspace dependency (the `forbid_all_workspace_dependencies()` shorthand). External (registry/git) dependencies SHALL NOT be considered by this rule, distinguishing it from `restrict_dependencies_to`, which governs all dependencies in the selected table. A workspace member added after the boundary is declared SHALL be governed without any change to the constitution. Names SHALL match the package name, not a local alias. The rule SHALL carry severity and react through the report, baseline, and projection exactly as the other crate rules do. Tables other than the selected dependency kind SHALL be out of scope (see Dependency kind selection).

#### Scenario: A workspace dependency outside the allowlist is a violation

- **WHEN** the target crate `backend` declares a normal dependency on the workspace member `other-backend`, and the boundary restricts workspace dependencies to `["core"]`
- **THEN** the system emits a violation naming `other-backend` and exits 1

#### Scenario: A workspace dependency inside the allowlist is clean

- **WHEN** the target crate's only workspace dependency is on `core`, and the boundary restricts workspace dependencies to `["core"]`
- **THEN** the system reports no violation for that boundary

#### Scenario: An external dependency is ignored by the workspace rule

- **WHEN** the target crate declares the external dependency `serde` and the boundary restricts workspace dependencies to `["core"]`
- **THEN** the system reports no violation for `serde`, because the rule considers only workspace members

#### Scenario: An empty allowlist forbids every workspace dependency

- **WHEN** the target crate declares a normal dependency on any other workspace member and the boundary forbids all workspace dependencies (an empty allowlist)
- **THEN** the system emits a violation for that workspace dependency

#### Scenario: A newly added workspace member is governed without a constitution edit

- **WHEN** a new crate `new-backend` is added to the workspace, the target depends on it, and the unchanged boundary's allowlist does not include `new-backend`
- **THEN** the system emits a violation naming `new-backend`, because workspace membership is derived from `cargo metadata` rather than a hand-maintained list

#### Scenario: A path dependency outside the workspace is not a workspace dependency

- **WHEN** the target crate declares a `path` dependency on a crate that is not a member of the workspace, under a forbid-all-workspace boundary
- **THEN** the system reports no violation, because the dependency resolves to a package outside `workspace_members`

### Requirement: Dependency kind selection

A crate boundary SHALL select which dependency table its rule observes — `Normal` (the default), `Dev`, or `Build` — declared in Rust via `.dependency_kind(kind)` on the boundary builder. A boundary that does not select a kind SHALL observe the normal `[dependencies]` table, preserving prior behavior exactly. When `Dev` or `Build` is selected the rule SHALL observe `[dev-dependencies]` or `[build-dependencies]` respectively and SHALL NOT observe the normal table; a boundary observes exactly one table, so governing two tables SHALL be expressed as two boundaries. The selection SHALL apply uniformly to every crate rule (deny-external, forbid, restrict, restrict-workspace), and SHALL appear in the projection when it is not `Normal`.

The `finding` SHALL be **kind-qualified** so the same dependency name governed under two tables stays distinct: a `Normal` finding is the bare dependency name (preserving prior baselines), while `Dev`/`Build` findings carry a ` (dev)`/` (build)` suffix. Without this, two boundaries governing the same crate under the same rule but different kinds would emit the identical `(target, rule, finding)`, and baselining one table's violation would mask a new violation of the same dependency in the other table (the one forbidden bug).

#### Scenario: The same dependency in two tables stays distinct findings

- **WHEN** a crate declares the same dependency (e.g. `serde`) from a forbidden source in both `[dependencies]` and `[dev-dependencies]`, governed by two same-rule boundaries differing only by kind, and the normal violation is recorded in the baseline
- **THEN** the dev violation still reacts: its finding `serde (dev)` differs from the baselined `serde`, so it is not masked

#### Scenario: A boundary defaults to normal dependencies

- **WHEN** a crate boundary is declared without selecting a dependency kind
- **THEN** its rule observes the normal `[dependencies]` table, exactly as before this capability existed

#### Scenario: A dev-kind boundary observes dev-dependencies

- **WHEN** a boundary selects `Dev` and the target declares a matching dependency only under `[dev-dependencies]`
- **THEN** the rule observes that dev-dependency and does not observe the normal table

#### Scenario: The selected kind appears in the projection

- **WHEN** a boundary selects a dependency kind other than `Normal`
- **THEN** the `list` projection, in text and JSON, shows the selected kind


### Requirement: Declared feature-request observation model

For a feature-granularity rule targeting a dependency `C`, the system SHALL compute the set of
features of `C` that the target crate **declares** — its **authored** feature request — from the
target's declared dependencies on `C` in the boundary's selected dependency kind (normal
`[dependencies]` by default). `C` SHALL be matched by the resolved **package name**, not a local
alias/`rename`, exactly as the other crate rules match dependency names. A target MAY declare `C`
under more than one edge of the selected kind — for example a plain `[dependencies]` entry and a
`[target.'cfg(...)'.dependencies]` entry, which Cargo emits as separate edges both of `Normal` kind.
The declared set SHALL be the **union** across every such edge: it contains a feature if any of those
edges lists it in `features = [...]`, and contains the pseudo-feature `default` if any of those edges
leaves default features enabled (`cargo metadata`'s `uses_default_features` boolean true, i.e. the
edge does not set `default-features = false`). The computation SHALL read only the target package's
declared dependency edges and SHALL depend on no other crate, so the result is attributable to the
target alone.

The system SHALL NOT expand the declared set through `C`'s own `[features]` table (it SHALL NOT fold
in the members of `C`'s default set, nor the features a declared feature transitively enables), and
SHALL NOT derive the set from `cargo metadata`'s resolved `resolve.nodes[].features`. The former is
unobservable for an external `C` under the shared `cargo metadata --no-deps` substrate (external
packages are absent from `packages[]`); the latter is unstable, because Cargo feature unification
merges every workspace crate's enables of `C` into one resolved set and would attribute another
crate's enabled feature to this target. The rule therefore governs the **authored** feature request,
a declared-not-resolved property at the same altitude as the existing declared-dependency rules.

#### Scenario: A declared feature is observed

- **WHEN** the target declares `C = { version = "1", features = ["extra"] }` and a feature rule targets `C`
- **THEN** the declared set for `C` contains `extra`

#### Scenario: Default features are represented by the `default` pseudo-feature

- **WHEN** the target declares `C = { version = "1" }` without `default-features = false` (so `uses_default_features` is true)
- **THEN** the declared set for `C` contains `default`

#### Scenario: Disabling default features drops the `default` pseudo-feature

- **WHEN** the target declares `C = { version = "1", default-features = false, features = ["extra"] }`
- **THEN** the declared set for `C` contains `extra` and does not contain `default`

#### Scenario: Transitive enables are not chased

- **WHEN** `C`'s manifest declares `full = ["unstable"]`, and the target declares `C = { version = "1", features = ["full"] }`
- **THEN** the declared set for `C` contains `full` and does **not** contain `unstable`, because the rule reads the target's authored request and does not expand `C`'s feature graph

#### Scenario: Another workspace crate's enable is not attributed to the target

- **WHEN** the target declares `C` with `default-features = false` and no features, while a different workspace member declares `C = { features = ["unstable"] }`
- **THEN** the declared set for the target is empty, because it is computed from the target's own declared edge, not the unified resolved set

#### Scenario: The declared set unions across multiple edges to the same dependency

- **WHEN** the target declares `C = { features = ["a"] }` in `[dependencies]` and `C = { default-features = false, features = ["unstable"] }` under `[target.'cfg(windows)'.dependencies]`
- **THEN** the declared set for `C` is the union `{ a, unstable, default }` — `default` because the plain edge leaves default features enabled — so a rule forbidding `C`'s `unstable` emits a violation

#### Scenario: A dependency matched by package name, not local alias

- **WHEN** the target declares `myc = { package = "real-c", features = ["unstable"] }` and a rule targets the package name `real-c`
- **THEN** the declared set for `real-c` contains `unstable`; a rule written against the local alias `myc` matches nothing, because matching is by resolved package name as the other crate rules do

### Requirement: Restrict a dependency's declared features to an allowlist

A boundary SHALL support a rule that restricts the features a target declares on a named dependency
`C` to a closed allowlist of feature names. Any feature in the target's declared set for `C` (per the
Declared feature-request observation model) whose name is not in the allowlist SHALL be a violation.
An empty allowlist SHALL forbid the target from declaring **any** feature of `C`, including the
`default` pseudo-feature (i.e. it requires `default-features = false` and no explicit features). The
rule SHALL apply to the boundary's selected dependency kind, and tables other than that kind SHALL be
out of scope (see Dependency kind selection). The rule SHALL carry severity and react through the
report, baseline, and projection exactly as the other crate rules do. When the target does not depend
on `C` in the selected table, the rule SHALL report no violation (there is no declared set to
constrain).

#### Scenario: A declared feature outside the allowlist is a violation

- **WHEN** the target declares `C`'s feature `unstable` and the boundary restricts `C`'s features to `["stable"]`
- **THEN** the system emits a violation for `C`'s `unstable` feature and exits 1

#### Scenario: A declared feature inside the allowlist is clean

- **WHEN** the target's only declared feature of `C` is `stable`, it sets `default-features = false`, and the boundary restricts `C`'s features to `["stable"]`
- **THEN** the system reports no violation for that boundary

#### Scenario: An empty allowlist forbids declaring any feature

- **WHEN** the target declares any feature of `C` (or leaves default features enabled) and the boundary restricts `C`'s features to `[]`
- **THEN** the system emits a violation

#### Scenario: A rule on a crate the target does not depend on is clean

- **WHEN** the boundary restricts the features of a crate `C` that the target does not depend on in the selected table
- **THEN** the system reports no violation for that boundary

### Requirement: Forbid a dependency's declared features

A boundary SHALL support a rule that forbids the target from declaring specific named features of a
dependency `C`. Any feature in the target's declared set for `C` (per the Declared feature-request
observation model) whose name matches a forbidden name SHALL be a violation; a forbidden feature the
target does not declare SHALL NOT be a violation. Forbidding the `default` pseudo-feature SHALL be a
way to require `default-features = false`. An empty forbidden set SHALL be a no-op that always reports
clean (a vacuous rule, symmetric with forbidding a crate the target does not depend on). The rule
SHALL apply to the boundary's selected dependency kind, carry severity, and react through the report,
baseline, and projection exactly as the other crate rules do.

#### Scenario: A forbidden feature that is declared is a violation

- **WHEN** the target declares `C`'s feature `unstable` and the boundary forbids `C`'s `unstable`
- **THEN** the system emits a violation for `C`'s `unstable` feature and exits 1

#### Scenario: Forbidding `default` requires default-features to be off

- **WHEN** the boundary forbids `C`'s `default` and the target declares `C` without `default-features = false`
- **THEN** the system emits a violation for `C`'s `default` feature

#### Scenario: A forbidden feature the target does not declare is clean

- **WHEN** the boundary forbids `C`'s `unstable`, and the target's declared set for `C` does not contain `unstable` (even if `C`'s `full`, which the target declares, would transitively enable it)
- **THEN** the system reports no violation for that boundary, because transitive enables are not chased

#### Scenario: An empty forbidden set is a no-op

- **WHEN** a boundary forbids an empty set of features of `C`
- **THEN** the system reports no violation for that boundary regardless of what the target declares

### Requirement: Feature-qualified finding

The `finding` of a feature-granularity violation SHALL be qualified by both the dependency name and
the offending feature name, in the form `C/feature`. Because a feature declared on a dependency edge
is a plain feature name (Cargo forbids the `dep:`, `pkg/feat`, and `pkg?/feat` syntaxes in a
dependency's `features` list; those exist only in a manifest's `[features]` table, which this rule
does not read), the feature name contains no `/`, so `C/feature` is unambiguous. The feature-rule
polarities (restrict / forbid) SHALL each carry a `rule` label distinct from each other and from the
crate rules, so that `(target, rule, finding)` stays injective: a `restrict` and a `forbid` rule that
both flag `C/unstable` on the same target remain distinct triples. Baselining one feature's violation
SHALL NOT mask a new violation of a different feature of the same dependency. When the selected
dependency kind is not `Normal`, the finding SHALL additionally carry the kind qualifier consistent
with Dependency kind selection.

#### Scenario: Two forbidden features of the same crate stay distinct

- **WHEN** the target declares both `C/unstable` and `C/nightly`, both are forbidden, and `C/unstable` is recorded in the baseline
- **THEN** the `C/nightly` violation still reacts, because its finding differs from the baselined `C/unstable`

#### Scenario: The report names the dependency, feature, and reason

- **WHEN** a feature rule on `C` is violated by the declared feature `unstable`
- **THEN** the report names the target crate, the feature rule, the finding `C/unstable`, the boundary's reason, and indicates the reaction failed
