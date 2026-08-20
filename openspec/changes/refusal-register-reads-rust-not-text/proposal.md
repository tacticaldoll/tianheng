## Why

`crates/kanhe/tests/refusal_register.rs` decides which refusal sites this repository has registered, and
which modules still construct one without a site identity, by scanning source text with a hand-rolled
character-by-character lexer rather than parsing it. That lexer has needed a dedicated arm for every new
Rust shape it was found wrong on — a byte prefix before a char literal reading `b'"'` as opening a runaway
string, a raw string's hash count, an escaped newline splitting a line count, a `use` list `cargo fmt`
wrapped across lines, a closure's closing pipe read as though it still opened the parameter list. Each fix
was correct and each was local; none of them made the next shape less likely, because the reader was never
parsing Rust — it was pattern-matching against Rust well enough for the corpus measured so far. The
capability's own declared bound (`repository-checks`, scenario "A construction shape the register's reader
does not model — a stated bound") named this directly: closing it "means the compiler enumerating the sites
rather than a reader — which is a change to what a site *is*, not another shape added to a scanner".

## What Changes

- Replace the hand-rolled scanner in `crates/kanhe/tests/refusal_register.rs` with a reader built on `syn`
  (already a workspace dependency) parsing this repository's own Rust into a real syntax tree, walked with
  `syn::visit::Visit` to find calls to `refusal::violation_at`/`refusal::cannot_judge_at`/`refusal::violation`/
  `refusal::cannot_judge` however they are reached — a direct call, a path-qualified one, or a bare reference
  taken by value — and to extract a call's first string-literal argument (plain or raw) as its site.
- Keep `Register`, `read()`, `countable`/`calls`/`unparsed_constructions`/`aliases_a_constructor`/`code_only`'s
  exact signatures and every existing `#[test]`'s name and assertions; only the internals implementing them
  change. `Cargo.toml` already carries `syn`/`proc-macro2` as dev-dependencies for this purpose.
- Add a differential test holding the syn-based reader to agreement with the hand-rolled one over every
  `crates/kanhe/tests/fixtures/refusal_scan/*.rs.txt` fixture and over this repository's own corpus (source
  and test directories), plus "seen to fail" evidence: a deliberately pre-fix reproduction of the
  `b'"'`-swallowing bug, run to show it still swallows the construction on the exact fixture that first found
  it, while the syn-based reader reads the same bytes correctly without needing that fix at all.
- Narrow the `repository-checks` capability's stated bound (`A construction shape the register's reader does
  not model`) to what the syn-based reader still cannot answer: whether a bare reference to a registered
  constructor's name is the constructor taken by value or a local variable sharing its spelling, which is not
  decidable from syntax and needs name resolution the reader does not have. The lexical half of the old bound
  (byte char literals, raw strings, wrapped imports, multi-line closures) is closed, not merely narrower.

## Capabilities

### Modified Capabilities

- `repository-checks`: the refusal register's reader is now a real Rust parser rather than a text scanner;
  its stated bound narrows to the name-resolution residue described above, and a second scenario's list of
  shapes the reader cannot parse to a site drops the raw-string-literal case, which the new reader reads.

## Impact

- **Code**: `crates/kanhe/tests/refusal_register.rs` (the reader and its tests), `crates/kanhe/Cargo.toml`
  (dev-dependencies already present on this branch), `crates/kanhe/src/bounds.rs` (the typed bound
  declaration matching the narrowed spec scenario).
- **Docs**: `docs/refusal-register.md` (regenerated, byte-identical to before over this repository's real
  corpus — proving no semantic regression), `docs/observation-bounds.md` and
  `docs/observation-bound-extents.md` (regenerated to reflect the narrowed bound), `BACKLOG.md` (the matching
  watch entry reworded to the narrowed residue).
- **Specs**: `openspec/specs/repository-checks/spec.md`.
