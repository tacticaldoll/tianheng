## Context

圭表's static engine (`guibiao`) runs `cargo metadata --no-deps` (`cargo_metadata.rs:10`) and
evaluates each `CrateBoundary` whose `Rule` is one of four `#[non_exhaustive]` variants. Each
`Rule` owns its `label`/`text`/`json_params`/`findings`; the four matches are exhaustive (no `_`
arm), so a new variant is a compile error until handled everywhere (the drift discipline). The
metadata carries each member's **declared** `dependencies[]`, each entry with `name` (the real
package name), `rename` (the local alias, if any), `source`, `req`, `optional`, and `kind`.
`external_dependencies` already reads `dependency["source"]` (null → internal/path, non-null →
external) — by *presence*, not prefix, so alt-registries count as external.

This change reads that same declared `source` one notch finer — git vs. registry vs. path —
behind a new `Rule` variant. Verified against `cargo metadata --no-deps` on a probe manifest:
a `git = "…"` dependency has `source = "git+…"` (even with a `version` key, and even when
`optional = true`); a `path = "…"` dependency has `source = null`; a registry dependency has
`source = "registry+…"`; `dependency["name"]` is the **real** package name and `rename` holds
any local alias.

## Goals / Non-Goals

**Goals:**
- A `SourceKind { Registry, Git, Path }` allowlist rule, `restrict_dependency_sources_to([...])`,
  flagging any dependency whose **declared** source kind is outside the allowlist.
- Reuse the static engine's dependency walk, projection, baseline, and exit-code contract; the
  only new logic is the per-rule source classifier. No new observation source, no new crate, and
  hermetic (`--no-deps`, manifests only).

**Non-Goals (one is a named future capability):**
- **Resolved build provenance (capability B, deferred).** "What my build *actually* pulls from,
  after `[patch]`/`replace-with`" lives in the resolve graph (`cargo metadata` *with* deps),
  which this rule does not read. B is a distinct future capability — born when built — that adds
  a resolved-layer read and catches patch-redirected sources; it is recorded in BACKLOG, not
  built here. (See Decision 2 for why A and B are genuinely different intents, not A being an
  incomplete B.)
- **A `cargo publish` oracle.** This is source-kind hygiene, not a publish simulation (Decision 3).
- Switching to a resolved/with-deps observation, or touching the `serde_json`-only allowlist.

## Decisions

### Decision 1 — Classify the DECLARED source, robustly, into three kinds

Each governed dependency's declared `source` classifies:

```
  source == null            → Path     (a path/internal dependency)
  source starts with "git+" → Git      (cargo spells a declared git source "git+<url>")
  otherwise (non-null)      → Registry (registry+, sparse+, alternative registries — residual)
```

Only `Git` is matched by a prefix and only `Path` by null; `Registry` is the residual, so a new
registry scheme classifies correctly without code change — the same robustness
`external_dependencies` already relies on. A finding is each governed dependency whose classified
`SourceKind` is not in the allowlist; the finding name is `dependency["name"]` (the **real**
package name, so a renamed git dependency reports `serde`, not the alias — **native parity** with
the sibling rules, which also report the real name).

### Decision 2 — Declared layer is the right SSOT for the manifest-hygiene / publishability intent

An earlier draft of this change read the *resolved* source and was killed for "missing `[patch]`";
exploration then showed that miss is correct for **this** intent and that A and B are two genuine
capabilities, not one:

```
   A — manifest-source hygiene / publishability         B — resolved build provenance (deferred)
   ──────────────────────────────────────────          ─────────────────────────────────────────
   declared layer (--no-deps, the crate's manifest)     resolved layer (resolve graph, lock+patch)
   "my manifest declares no git/path source"            "my build pulls from no git source"
   ✓ governs OPTIONAL git (crates.io rejects it too)    ✓ catches [patch]/replace → git
   [patch] correctly NOT a violation: it is workspace-  ✗ misses optional-off git (not in build —
     local, not in the published manifest, never          correct for B's intent)
     blocks cargo publish                                lock-dependent, heavier
   hermetic; this change                                a future born-when-built capability
```

Neither dominates: A governs optional git (publish-relevant) and is blind to patch (publish-
irrelevant); B catches patch (build-relevant) and is blind to optional-off (build-irrelevant). So
A reading the declared layer is the *correct* design for "prepare to publish", not an incomplete
B. The prior review's patch "BLOCKER" applied B's intent to A's mechanism; under A's intent a
patch-redirected dependency reading as `Registry` is the right reaction (no violation).

### Decision 3 — Source-kind hygiene, not a publish oracle (a stated bound)

A `{ git = "…", version = "…" }` or `{ path = "…", version = "…" }` dependency declares a
non-registry source (so it classifies `Git`/`Path` and is flagged) yet `cargo publish` succeeds
(the version is used at publish time). The probe confirms the `source` stays `git+…` regardless of
a `version` key. The rule does **not** parse `version`/`req` to distinguish these (the `req = "*"`
signal is ambiguous with an explicit `version = "*"`), so it is deliberately conservative: it
flags any declared non-registry source. This is stated in the spec and the builder doc — the rule
is a *hygiene guard over declared source kinds*, not a publish-eligibility decision.

### Decision 4 — Dependency-kind scoping mirrors the sibling rules

Like every `Rule`, the source rule carries a `DependencyKind` (default `Normal`, settable via the
existing `.dependency_kind(...)` draft method) and observes the declared dependencies of that
kind, via the existing `kind_matches`/`dependencies` path. No new kind machinery.

## Risks / Trade-offs

- **[Mistaken for a publishability guarantee]** → Mitigation: Decision 3 pinned in spec + builder
  doc; the precise publish-guard is `[Registry, Path]` (forbid git), and the git+version / path+
  version over-flag and the patch blind spot are stated.
- **[Confusion with capability B]** an adopter may expect patch-redirected sources to be caught. →
  Mitigation: Decision 2 and the proposal state plainly that resolved provenance is a separate
  future capability; A is declared-layer by design.
- **[source-string brittleness]** → only `git+`/null matched positively, `Registry` residual; a
  future cargo `git+` spelling change is a test-guarded assumption.
- **[non-exhaustive Rule churn]** adding the variant touches four matches. → the intended drift
  discipline; the compiler enforces no silent omission.

## Migration Plan

No migration. Purely additive: adopters opt in with `restrict_dependency_sources_to([...])`. The
`--no-deps` path and every existing rule's findings are unchanged. Self-governance and the existing
static tests must stay green; new unit tests cover registry/git/path classification (incl. an
optional git dep and a git+version dep), the real-name finding under rename, and dependency-kind
scoping. Rollback removes the boundary (and, if needed, the variant + builder).

## Open Questions

- **Convenience sugar.** Whether to ship `only_registry_dependencies()` for
  `restrict_dependency_sources_to([Registry])`. Leaning **no** for v0.1.2 — the explicit allowlist
  is clearer about the publish-guard intent `[Registry, Path]` and avoids implying a publishability
  guarantee.
