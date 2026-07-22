# 天衡 / tianheng

**懸衡以待,失衡即應。** — *Hold the balance ready; react the moment it tips.*

The **shell** of the [Tianheng](https://github.com/tacticaldoll/tianheng) crate family:
reactive architectural governance that weighs the shape you *declared* against the shape the
code *is*, and reacts the moment they no longer balance. **Govern by reaction, not
instruction.**

天衡 (the celestial balance) is the imperative shell + facade — CLI (arg parsing, filesystem,
stdout/stderr) and the `run` reaction that composes every dimension into one. You declare a
single `Constitution` carrying all 三儀 (the three instruments) and call `run`:

```rust,no_run
use tianheng::prelude::*;

// One declared constitution carries every dimension — the single source of truth.
fn constitution() -> Constitution {
    Constitution::new("my-project")
        // 圭表 (static): imports & dependencies
        .boundary(
            CrateBoundary::crate_("my-core")
                .deny_external_dependencies()
                .because("my-core must stay dependency-light"),
        )
        // 渾儀 (semantic): folded in via typed adders, e.g.
        // .signature_boundary(SemanticBoundary::in_crate("my-app")…)
        // 漏刻 (runtime): a declared seam, audited for probe coverage at CI
        // .runtime(RuntimeBoundary::at("domain-entry").only_origins(["my_app::domain"]).because(…))
}

fn main() -> std::process::ExitCode {
    // One declaration in; the 三儀 compose into the one reaction.
    tianheng::run(&constitution(), std::env::args())
}
```

### The adopter surface

`tianheng::prelude::*` is the supported composed entrypoint, with two purpose-only tiers:

- **Declare and run:** `Constitution`, the terminal static/semantic/runtime boundary types,
  `SansIoPure`, their selector enums, `Severity`, and `run`.
- **Inspect the reaction:** `Outcome`, reports, violations, structured finding/violation identity,
  baselines, boundary/rule model types, the pure static `check`, and `check_constitution` for the
  unified law.

Rules remain builder-owned even though they are inspectable: obtain `Rule` or `ModuleRule` from a
built boundary's `rule()` accessor and match known fields with `..`. For a focused semantic
signature-coupling test, import `tianheng::check_semantic` explicitly; it is not the full semantic
bundle. Normal composed governance stays `Constitution` + `run`.

In a library test, inspect all three CI-time reactions without capturing CLI output:

```rust,no_run
use std::path::Path;
use tianheng::prelude::*;

let law = Constitution::new("my-project");
let outcome = check_constitution(&law, Path::new("Cargo.toml"));
match outcome {
    Outcome::Violations(report) => {
        let violation = &report.violations[0];
        assert!(!violation.id().target().is_empty());
        assert!(!violation.rule_key().rule_type().is_empty());
        assert!(!violation.fact().fact_type().is_empty());
    }
    Outcome::Clean => {}
    Outcome::ConstitutionError(message) => panic!("cannot evaluate the law: {message}"),
    _ => unreachable!("Outcome is non-exhaustive"),
}
```

`check_constitution` requires an explicit manifest and returns the raw, unbaselined `Outcome`. It
does run Cargo metadata and source observation. Use `run` for nearest-manifest discovery, baseline
gate/write modes, coverage advisories, text/JSON/SARIF presentation, and process exit handling.

The same reaction model is available directly from `guibiao`, `hunyi`, and `louke`; each
instrument can be adopted and inspected without importing this facade. Tianheng is the one
built-in cross-instrument composer. This release adds neither a dimension/plugin trait nor a
`tianheng::testing` assertion DSL: architecture tests call the existing pure check functions and
assert on structured `Outcome` values.

`your-binary check --manifest-path path/to/Cargo.toml` reacts against *your* constitution:
exit `0` (clean / warn-only / fully baselined), `1` (enforced violation), `2`
(constitution/scan error). `list` projects the declared constitution and never reacts.
`check --format json` projects the reaction as JSON (the machine contract); `--format sarif`
emits a vendor-neutral SARIF 2.1.0 document for CI code-scanning. The violation's `reason` is the
repair direction; the human text report and the Markdown projection lead with it.

## The instruments (三儀)

`tianheng` composes the three observation dimensions, each its own crate, each real drift
(declared vs. observed) — never a style lint:

| 儀 | Crate | Observes |
|---|---|---|
| 圭表 (static) | [`guibiao`](https://crates.io/crates/guibiao) | imports, dependencies & their declared source kind (`cargo metadata` + `use` scan) |
| 渾儀 (semantic) | [`hunyi`](https://crates.io/crates/hunyi) | type exposure (incl. public re-exports and the opt-in trait-impl surface), impl locality, visibility, forbidden markers, `dyn` & `impl Trait` (existential) exposure (shape-only & named-operand), `async fn` (implicit existential) exposure (AST/`syn`) |
| 漏刻 (runtime) | [`louke`](https://crates.io/crates/louke) | the concrete type behind a `dyn Trait` crossing a seam (runtime `TypeId`) |

Beneath them sits [`xuanji`](https://crates.io/crates/xuanji) — the dimension-agnostic
reaction model. The 漏刻 prod face is wired in your own binary
(`louke::install(constitution().runtime_boundaries()…)`); its CI probe-coverage face is run
by `tianheng check`.

> The published `tianheng` binary is a **demo** bound to a sample constitution (it governs a
> crate named `example-core`). Tianheng is consumed as a **library**: declare your own
> constitution and expose your own binary, as above.

Tianheng governs **itself** with its own reaction (`crates/tianheng/tests/self_governance.rs`):
the core must not depend on the shell, `syn` is quarantined to `hunyi`, `xuanji` stays beneath
every dimension.

## License

Licensed under either of Apache-2.0 or MIT, at your option.
