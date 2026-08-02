## MODIFIED Requirements

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

#### Scenario: A crate's own self-referential PATH dependency is never a violation under any crate rule

- **WHEN** the target crate declares a dependency on ITSELF (its own package name) with a **null declared `source`** in its selected table — a real, Cargo-legal pattern (e.g. a `[dev-dependencies]` path dependency on `.`, used for doctest/dogfooding) — and a boundary using ANY crate rule (forbid-dependency-on, restrict-dependencies-to, restrict-workspace-dependencies-to, restrict-dependency-sources-to, or a feature-granularity rule naming the target's own crate) governs the target
- **THEN** the system reports no violation arising from that self-referential edge under any of these rules — a genuine path self-dependency names no OTHER crate, so it can never be the cross-crate concern any of them exist to govern, regardless of workspace-membership set inclusion or dependency kind (`Normal`, `Dev`, or `Build`)

#### Scenario: A same-named but externally-sourced dependency is NOT exempted

- **WHEN** the target crate declares a dependency whose name equals its own package name but whose declared `source` is **non-null** (a `git`/registry source — e.g. `foo = { git = "…" }` declared by package `foo`, the real-world wrapper/fork/self-comparison pattern) and a boundary using ANY crate rule governs the target
- **THEN** the system treats it as an ordinary dependency and reacts exactly as it would for any other externally-sourced dependency of that name — the self-referential exemption applies only to the genuine null-source path idiom above, never to a same-named edge that resolves to a different, externally-sourced package

#### Scenario: A newly added workspace member is governed without a constitution edit

- **WHEN** a new crate `new-backend` is added to the workspace, the target depends on it, and the unchanged boundary's allowlist does not include `new-backend`
- **THEN** the system emits a violation naming `new-backend`, because workspace membership is derived from `cargo metadata` rather than a hand-maintained list

#### Scenario: A path dependency outside the workspace is not a workspace dependency

- **WHEN** the target crate declares a `path` dependency on a crate that is not a member of the workspace, under a forbid-all-workspace boundary
- **THEN** the system reports no violation, because the dependency resolves to a package outside `workspace_members`
