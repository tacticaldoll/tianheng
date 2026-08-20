## Context

`refusal_register.rs` answers two questions over `crates/kanhe/src/**.rs`: does this text construct a
registered refusal (`refusal::violation_at`/`refusal::cannot_judge_at`) and what is its site, and does this
text construct an unregistered one (`refusal::violation`/`refusal::cannot_judge`, no site). Both answers were
produced by a character-by-character state machine tracking comment/string/raw-string/char-literal state and
a set of position heuristics (counting `|` characters before a name on its line to detect a closure
parameter, checking whether a `.` immediately follows a name to detect a field access). Both kinds of logic
are approximations of facts a real parser states exactly, and the fixture corpus at
`crates/kanhe/tests/fixtures/refusal_scan/` documents five rounds of the approximation being wrong in a new
way.

## Goals / Non-Goals

**Goals:**
- Replace the lexical half of the reader (comment/string/literal recognition, `use`-statement splitting)
  with `syn` parsing, closing the class of bug the fixture corpus documents.
- Replace the position heuristics (binder detection, field-access exclusion, definition exclusion) with
  exact checks on the parsed AST's node types, which state the same facts precisely instead of
  approximately.
- Preserve every existing test's name and assertions, and the external shape of `Register`/`read()`.
- Prove the replacement via differential testing against the hand-rolled reader before deleting it, per this
  repository's own "a guard is not a guard until it has been seen to fail".

**Non-Goals:**
- Resolving the bare-reference-vs-local-variable ambiguity (`a_constructor_taken_by_name`,
  `a_siteful_constructor_taken_by_name`) via name resolution or any other means. `syn`'s AST does not carry
  binding information, and building a resolver for this narrow purpose is out of proportion to what it would
  buy: this register's actual corpus does not shadow a refusal constructor's name with an unrelated local,
  and the existing tests already assert the ambiguous case counts as a construction.
- Changing anything about `kanhe::region` or the shared comment-stripping module it exposes; this migration
  is scoped to `refusal_register.rs`'s own reader.

## Decisions

### Decision 1: Parse via `syn::File`, falling back to `syn::Block::parse_within`

**Rationale**: A real, compiling corpus file is always a valid `syn::File`. A fixture written to exercise one
shape in isolation (e.g. `let x = a_violation(1);` alone) is often not — there is no item for a file to hold
— but it is always a valid sequence of block statements, since an item is itself a valid statement inside a
block (Rust 2018+). Trying the file grammar first and falling back to the block grammar reads every real
corpus file (including one with a leading `#![...]` inner attribute, which only the file grammar accepts)
and every fixture, with one function rather than a per-shape dispatch.

**Alternatives considered**: Parsing everything as `Block::parse_within` alone. Rejected because inner
attributes at a file's top (`crates/kanhe/src/lib.rs` has one) are not valid block-statement syntax, so this
would need its own special case — exactly the kind of accreted exception this migration exists to stop
writing.

### Decision 2: A `syn::visit::Visit` walk keyed on AST node type, not text position

**Rationale**: The four things the old reader needed to exclude — a function's own name, a `use` import, a
pattern-bound name (closure parameter, `let` binding, match arm), and a field-access base — are each a
distinct node type in `syn`'s AST (`Signature::ident`, `UseTree`, `Pat`, `Expr::Field::base`) that is never
represented as an `Expr::Path`. Overriding `visit_expr_path`/`visit_expr_call`/`visit_expr_field`/
`visit_expr_method_call` and recording a finding only for a real `Expr::Path` occurrence therefore excludes
all four *by construction*, with no separate rule to state and keep correct. This directly replaces the old
`binds`/`projects`/`defines` heuristics (pipe-counting, dot-checking, `fn`-suffix-checking).

**Alternatives considered**: A generic `syn::visit::Visit` walk overriding only `visit_ident` and comparing
text. Rejected because it cannot distinguish a `Pat::Ident` from an `Expr::Path` — the exact distinction the
old reader's heuristics existed to approximate — so it would not close the intended bug class.

### Decision 3: `code_only`'s replacement copies real token spans and blanks the gaps between them

**Rationale**: One existing test (`the_reader_swallows_no_declaration_from_the_corpus_it_reads`) exercises
`code_only` directly as a text transformer (comments and literal interiors blanked, line count preserved),
independent of the AST-based counting logic. Plain `//`/`/* */` comments have no span accessible through
`syn`/`proc_macro2`'s public API outside a proc-macro context, so `code_only` cannot itself become an AST
walk. It can still stop being a character-by-character scanner: `proc_macro2::TokenStream::from_str` (via the
`span-locations` feature already enabled) gives the exact byte range of every real token, including a raw
string or byte char literal's full span; anything *between* two token spans is, by Rust's own grammar, only
ever whitespace or a comment. Copying token spans verbatim and blanking the gaps (keeping whitespace, so a
`pub`/`fn` boundary does not fuse into `pubfn`) reproduces `code_only`'s contract using the real tokenizer's
boundaries instead of a hand-tracked comment/string state, and is immune to the same bug class by
construction — a `//` inside a raw string is part of that string's single token span and is never treated as
a comment start.

### Decision 4: Differential test first, cutover second, as two commits

**Rationale**: `AGENTS.md`'s "a guard is not a guard until it has been seen to fail" applies to this migration
itself, not only to the checks it produces. Adding the syn-based reader alongside the hand-rolled one and
proving agreement over every fixture and this repository's own corpus — including one deliberate,
documented disagreement (a raw-string site, which the old reader is documented not to read and the new one
reads correctly with no extra code) — is evidence a same-commit rewrite could not produce, because there
would be nothing left to compare against once the old reader is gone.

**Alternatives considered**: Rewriting and cutting over in one commit, relying on the existing fixture suite
and the `docs/refusal-register.md` byte-identical regeneration as proof. Rejected because neither of those
alone is "seen to fail" evidence in the sense this repository already holds itself to: the fixture suite is
run against whichever reader is current, so a shared bug in both would not surface, and the byte-identical
projection only proves agreement on the sites this repository's own corpus happens to construct today, not
on the specific historical bug shapes the fixture corpus was written to catch.

## Risks / Trade-offs

- **Risk**: A construction reached only through a macro's own definition body (`macro_rules! { ... }`),
  rather than at a call site, is opaque to `syn` — it does not expand macros, and a `macro_rules!` body is an
  unparsed token stream to it. → **Mitigation**: not mitigated; not exercised by this repository's corpus
  today (verified: no `violation_at`/`cannot_judge_at`/`violation`/`cannot_judge` construction lives inside a
  `macro_rules!` body in `crates/kanhe/src`), and not a shape this change claims to close. Left as residual
  risk rather than a stated bound, since it is narrower than what was previously undeclared and unmeasured.
- **Risk**: `code_only`'s token-gap reconstruction and the AST-based counting logic could in principle
  disagree with each other on some shape neither the fixture corpus nor this repository's own source
  exercises. → **Mitigation**: the differential test compares both against the hand-rolled reader over the
  real corpus (not only the fixtures), and `BLESS=1` regenerating `docs/refusal-register.md` after cutover
  reproduces it byte-for-byte, which is the strongest evidence available that the two readers agree
  everywhere this repository's own text currently reaches.
