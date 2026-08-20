## 1. Differential proof

- [x] 1.1 Confirm `crates/kanhe/Cargo.toml` carries `syn` (with `extra-traits`) and `proc-macro2` (with
      `span-locations`) as dev-dependencies.
- [x] 1.2 Add a syn-based reader in `crates/kanhe/tests/refusal_register.rs`, alongside the hand-rolled one,
      covering `countable`/`calls`, `unparsed_constructions`, `aliases_a_constructor`, `code_only`, and
      `read()`.
- [x] 1.3 Add a test holding the two readers to agreement over every
      `crates/kanhe/tests/fixtures/refusal_scan/*.rs.txt` fixture, with the one documented exception (a raw
      string site) asserted explicitly rather than silently excluded.
- [x] 1.4 Add a test holding the two readers to agreement over this repository's own corpus
      (`crates/kanhe/src`, `crates/kanhe/tests`, `crates/kanhe/src/tests`), including the full `Register`
      each produces.
- [x] 1.5 Add "seen to fail" evidence: a deliberately pre-fix reproduction of the byte-char-literal
      swallowing bug, shown to reproduce on the fixture that first found it, with the syn-based reader shown
      correct on the same input.
- [x] 1.6 `cargo test -p kanhe --test refusal_register -- --nocapture` green.

## 2. Cutover

- [x] 2.1 Replace `code_only`/`countable`/`calls`/`unparsed_constructions`/`aliases_a_constructor`/`read`'s
      internals with the syn-based implementation; delete the hand-rolled character scanner, the old
      `first_literal_args`/`imports_and_rest`/`imports_and_rest_of`/`opens_a_use`/`drop_imports`, and the
      now-superseded differential-only tests.
- [x] 2.2 Reclassify `a_raw_literal_site` in `the_reader_answers_the_corpus_written_for_it`'s tables: it
      moves out of `UNREADABLE_SITE_CASES` (the syn-based reader reads a raw string site correctly) and into
      the successfully-parsed controls. No other existing `#[test]`'s name or assertions change.
- [x] 2.3 `BLESS=1 TIANHENG_WORKSPACE_TESTS=1 cargo test -p kanhe --test refusal_register` and confirm
      `git diff docs/refusal-register.md` is empty (byte-identical to before the cutover).
- [x] 2.4 `cargo clippy --all-targets --all-features -p kanhe -- -D warnings` and `cargo fmt -p kanhe --check`
      green.

## 3. Close the declared bound

- [x] 3.1 Narrow `openspec/specs/repository-checks/spec.md`'s "A construction shape the register's reader
      does not model — a stated bound" scenario to the name-resolution residue; update the raw-string
      mention in "A registered construction this reader cannot parse is not counted as absent".
- [x] 3.2 Update `crates/kanhe/src/bounds.rs`'s matching `BoundDecl::unpinned` (description, `because`, and
      tracker string) to match the reworded scenario exactly.
- [x] 3.3 Update the matching `BACKLOG.md` watch entry.
- [x] 3.4 `BLESS=1 TIANHENG_WORKSPACE_TESTS=1 cargo test -p kanhe --test bound_register` and
      `BLESS=1 TIANHENG_WORKSPACE_TESTS=1 cargo test -p kanhe --test observation_bound_model`, regenerating
      `docs/observation-bounds.md` and `docs/observation-bound-extents.md`.

## 4. Definition of Done

- [ ] 4.1 `cargo build --workspace`
- [ ] 4.2 `cargo clippy --all-targets --all-features -- -D warnings`
- [ ] 4.3 `cargo fmt --all --check`
- [ ] 4.4 `TIANHENG_WORKSPACE_TESTS=1 cargo test --workspace --all-features`
- [ ] 4.5 `npx --no-install openspec validate --specs --strict`
- [ ] 4.6 `TIANHENG_WORKSPACE_TESTS=1 cargo test -p kanhe --test reference_integrity`
