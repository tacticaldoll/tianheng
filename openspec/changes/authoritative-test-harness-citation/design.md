## Context

`check_bound_register.sh` decides whether a `PINNED-BY` citation names a real test by reading source text: a
`grep` for the definition form, then an upward read of the attribute run. Three reviews have now produced
falsifiers against that approach, and the third produced three at once — a `#[cfg(any())]`-removed `#[test]`,
a `#[test] fn` in an uninvoked `macro_rules!` body, and a definition inside a raw string, all accepted with
exit 0.

The previous change met the same class by **declaring a residual** (a definition inside a block comment) and
stating it in the projection. That was the wrong call. Its own comments rejected the harness enumeration twice
on an unmeasured premise — "it needs a compiled workspace, and the whole failure matrix is throwaway
repositories holding one `lib.rs` and no manifest". Measured:

| measurement | result |
|---|---|
| `cargo test --workspace --all-features -- --list`, warm | 1s, 1251 tests |
| the 36 cited names present in it | 36 / 36 |
| per package (six of them), warm | 746ms total |
| a throwaway fixture crate with a 6-line manifest, **cold** | 107ms |
| `#[cfg(any())]` test in that fixture | not listed |
| uninvoked macro-body test in that fixture | not listed |

A throwaway repository can carry a manifest. The cost was estimated from inside the code rather than measured,
which is the failure `BACKLOG.md`'s own governance preamble warns about, and it cost a wrong residual
declaration that stood for one change.

## Goals / Non-Goals

**Goals:**

- Test-ness is decided by the only exact observation source: the harness's own enumeration.
- Crate precision survives, because a citation may be crate-qualified.
- The failure matrix still proves every direction, including the fallback and the degradation.
- The retired residual is recorded as retired, not silently deleted.

**Non-Goals:**

- Lexing Rust in bash. The point of this change is that it is unnecessary.
- Keeping the register gate free of a build dependency. That is given up deliberately; see the risk below.
- Enumerating doc tests, or tests behind non-default features other than `--all-features`.

## Decisions

**1. Per package, not per workspace.**

`cargo test -- --list` prints `module::path::name: test` with no crate label, so a workspace-wide index keyed
on the leaf name loses the crate. That is not theoretical here:
`a_cfg_gated_module_with_no_file_is_skipped_not_errored` is registered in **both** `hunyi` and `louke`, and the
register cites the `hunyi::` one. A workspace-wide match would let a citation qualified to a crate whose test
had been cfg-disabled be satisfied by the other crate's live test — the exact hole this change closes,
reintroduced by the shortcut. Per-package enumeration costs 746ms warm for all six.

**2. The harness is authoritative; the text scan keeps two jobs.**

The scan still answers *where* the definition is (the enumeration carries no file or line) and *how many*
there are, which is what makes a crate-qualified citation exact and a duplicated name a refusal. It no longer
answers test-ness when the harness is available — consulting both would produce disagreement noise on shapes
the harness already excludes.

**3. The fallback exists, and says so.**

The failure matrix builds repositories with one `lib.rs` and no manifest, deliberately, because most of the
register's directions have nothing to do with Rust. There the attribute-run walk decides test-ness. A silent
fallback would be the worst of both — the gate would report the clean it claims while running a direction it
declared secondary — so the reaction prints which direction decided. A fixture asserts the notice, so the
degradation itself is pinned.

**4. An unproducible enumeration is `cannot judge`, not a fallback.**

A root manifest present but `cargo test --list` failing means the workspace does not build or `cargo` is
missing. Test-ness is then *undecided*, and the family's contract has an exit class for exactly that. Falling
back would convert a broken workspace into a weaker clean. The Definition of Done runs the register lines after
`cargo test`, so in practice this state is already red upstream.

**5. Retire the residual out loud.**

The projection's third floor and the `BACKLOG.md` entry it produced both go. The entry is moved to the closed
records with its reproduction and the measurement that dissolved it, because it was filed one change ago on a
premise this change disproves — deleting it quietly would leave the next reader to rediscover why a residual
appeared and vanished.

**6. Raw identifiers accepted, ASCII stated.**

`r#name` is a Rust identifier and the register promises to impose no naming convention, so the validation
accepts the `r#` prefix (`#` is not an ERE metacharacter, so nothing else changes). Non-ASCII identifiers stay
refused, and the requirement now says so instead of implying full Rust grammar: the pattern is byte-oriented,
nothing needs it, and the refusal is loud.

**7. The line-shape disagreement gets its own diagnostic.**

The scan needs `fn` and the name on one line. With the harness authoritative, a definition split across lines
produces a *disagreement* — registered but not located — and the reaction names the line shape it requires
instead of reporting the test absent. That turns an undeclared limitation into a diagnostic, which is better
than declaring it in prose and leaving the message misleading.

## Risks / Trade-offs

- **[The gate acquires a build dependency]** → accepted deliberately, and the cost is bounded by ordering: the
  Definition of Done already runs the shell gates after `cargo test`, and CI's register step moves after its
  build step, so the enumeration is warm (1s) rather than a duplicate compile. Run standalone on a cold
  checkout it compiles the workspace, which is the honest price of the only exact observation source. The
  `AGENTS.md` comment on those lines says so, since a contributor otherwise meets it as a surprise.
- **[`--all-features` is not every configuration]** → a test registered only under a non-default feature
  combination outside `--all-features` would be reported unregistered. That is the same combination the
  Definition of Done's own `cargo test` runs, so a citation the gate refuses is one that CI does not run
  either.
- **[The fallback is weaker than the authority, and fixtures mostly exercise the fallback]** → three
  manifest-bearing fixtures exercise the authority (a real test, a cfg-disabled test, a macro-body test), and
  one fixture asserts the degradation notice, so neither path is unproven.
- **[Enumeration adds ~750ms to a warm gate run]** → measured, and the gate previously ran in well under a
  second. The trade is one second for the difference between reading text and reading what runs.

## Migration Plan

None for adopters. Contributors gain an ordering requirement: run the register lines after a build, which the
Definition of Done list already does.

## Open Questions

None.
