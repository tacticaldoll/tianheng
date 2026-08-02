## Context

`crates/guibiao/src/cargo_metadata.rs::is_self_dependency` is the single shared observation-source
filter consumed by `dependencies`, `dependencies_with_disallowed_source`, and `declared_features` —
the round-12 fix (see PROJECT.md Decisions) deliberately centralized it here so every crate rule is
closed by one change, not one at a time. Today it reads:

```rust
fn is_self_dependency(package: &Value, dependency: &Value) -> bool {
    let own_name = package["name"].as_str();
    own_name.is_some() && dependency["name"].as_str() == own_name
}
```

It matches on `name` alone. Verified independently (not just trusting the audit write-up) by
building a real two-crate probe and running `cargo metadata --no-deps` directly:

```jsonc
// probe crate "foo" declares: [dependencies] foo = { git = "https://example.invalid/foo.git" }
{"name":"foo","source":"git+https://example.invalid/foo.git","kind":null, ...}
```

`cargo metadata --no-deps` never resolves or fetches a dependency graph — it is a pure read of the
declared manifests, confirmed hermetic even for an unreachable git host, so this reproduction (and
the regression test built on it) needs no network access and is safe in CI. Feeding this exact
metadata through the real `guibiao::check(&Constitution, &Path)` entry point with each of the three
rule constructors the audit names —
`CrateBoundary::crate_("foo").restrict_dependency_sources_to([Registry, Path])`,
`.restrict_dependencies_to([])`, and `.forbid_dependency_on(["foo"])` — every one reads
`Outcome::Clean`, confirming the false negative end to end, not only at the `is_self_dependency`
unit level.

## Goals / Non-Goals

**Goals:**
- Close the false negative: a same-named dependency with a non-null (`git+`/`registry+`/`sparse+`)
  source must be governed like any other cross-crate dependency by every rule built on
  `dependencies` / `dependencies_with_disallowed_source` / `declared_features`.
- Preserve the existing, legitimate exemption for a genuine self-referential **path** dependency
  (`main = { path = "." }`) — the doctest/dogfooding idiom the function's doc comment already
  describes as its sole intended trigger.

**Non-Goals:**
- No change to `external_dependencies`, `classify_source`, or any other observation function —
  the fix is confined to `is_self_dependency`'s own predicate.
- No change to rule *shape*, the CLI surface, or the JSON/text/markdown projections.
- No attempt to detect a same-named external dependency as a "suspicious" pattern in its own
  right (e.g. a new lint/warning) — it is simply no longer exempted, and reacts exactly as an
  ordinary same-shaped external dependency would under the declared boundary.

## Decisions

**Add a `source.is_null()` conjunct to `is_self_dependency`.** The fix is a one-line narrowing:

```rust
fn is_self_dependency(package: &Value, dependency: &Value) -> bool {
    let own_name = package["name"].as_str();
    own_name.is_some() && dependency["name"].as_str() == own_name && dependency["source"].is_null()
}
```

Alternatives considered:
- *Classify via `classify_source` and require `SourceKind::Path`* — equivalent in outcome
  (`classify_source` maps a null source to `Path`) but adds an indirection and a needless
  dependency from `is_self_dependency` on `classify_source`'s residual-classification logic for
  no behavioral gain; rejected for the simpler, direct `is_null()` check already used by
  `external_dependencies`'s own "a null source is path/internal" convention two functions above.
- *Match on `kind == "dev"` too* (since the doctest/dogfooding idiom is conventionally a
  dev-dependency) — rejected: the doc comment and existing tests exempt the self-path pattern
  regardless of dependency kind (`Normal`, `Dev`, or `Build`), and the audit finding is about
  `source`, not `kind`; narrowing on `kind` as well would be an unrelated, unrequested behavior
  change with no observed trigger.

## Risks / Trade-offs

- [Risk] A crate could legitimately declare a *path* dependency on itself, resolving outside the
  workspace, that Cargo also permits — already covered identically before and after this fix,
  since `source` for a path dependency is always null regardless of workspace membership.
  → Mitigation: no change to that behavior; verified by re-running the existing
  `workspace_rule_never_flags_a_crates_own_self_referential_dev_dependency` and
  `no_dependency_rule_ever_flags_a_crates_own_self_referential_dependency` tests unmodified.
- [Risk] A downstream consumer could be relying on the old (over-broad) exemption to suppress a
  same-named external dependency they never intended to be governed. → Mitigation: none needed —
  PROJECT.md's Core Contract explicitly forbids exactly this shape of false negative; a consumer
  depending on it was depending on the bug. Pre-1.0, this is a patch-level correctness fix, not a
  breaking change (no public API changed).

## Migration Plan

None. Internal observation-source fix; no manifest, CLI, or public-API migration needed.

## Open Questions

None.
