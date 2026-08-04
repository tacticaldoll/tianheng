## 1. Prove the gap is real before closing it

- [ ] 1.1 Add a test that registers a rogue type under an allowlisted origin through the current
      `__from_register_origin(TypeId, &str, &str)` and asserts the crossing produces **no** reaction —
      the false negative, reproduced in-tree rather than described. It must fail (i.e. stop compiling)
      once task 2.2 lands; that transition is the evidence the gap closed.
- [ ] 1.2 Record the derivation shapes as a test over `std::any::type_name`: nested inline module,
      crate-root type, generic instantiation, type alias, foreign type, `Box<dyn _>`, function-local
      type. These pin what the implementation may assume, so a future toolchain change surfaces here
      rather than in an adopter's allowlist.

## 2. Derive the entry from its type

- [ ] 2.1 Add the origin derivation: from a type's reported path, strip the argument list from the
      **first top-level** `<` (delimiter-aware, so `Repo<std::string::String>` is handled), then take
      everything before the final path separator. Unit-test it against every shape from task 1.2,
      including the stated-bound cases.
- [ ] 2.2 Replace `OriginEntry::__from_register_origin(TypeId, &'static str, &'static str)` with a
      generic, argument-free expansion target over the registered type; derive the type identity,
      origin, and type name inside it. Rewrite its doc comment: the honest bound it currently states
      is gone, and what remains to state is why the constructor is still `pub` (macro expansion at the
      call site) and why that no longer matters.
- [ ] 2.3 Update `register_origin!` to expand to the new target — same invocation spelling
      (`register_origin!(MyType)`), no `module_path!()`, no `TypeId::of`, no `type_name` at the call
      site. Rewrite its doc comment away from "captures `module_path!()`".
- [ ] 2.4 Confirm the registry and hot path are untouched: origins are still `&'static str`, still
      resolved once at startup, still a lock-free read, still no allocation per crossing. If the
      derivation cannot yield `&'static str`, stop and reconsider before changing the registry's
      storage — that would be a different change from this one.

## 3. Retire the residual's footprint in the same commit

- [ ] 3.1 Delete `a_hand_built_origin_entry_is_accepted_a_known_trust_bound` — there is no residual
      left for it to pin.
- [ ] 3.2 Invert `the_origin_guarantee_is_never_summarized_as_absolute`: the process-trust-boundary
      prose must now be **absent** from `crates/louke/README.md`, `crates/louke/src/dsl.rs`,
      `openspec/specs/runtime-origin-assertion/spec.md`, and `PROJECT.md`, and the derived-origin
      statement must be present. Watch the inverted guard fail against the pre-sweep prose before
      trusting it.
- [ ] 3.3 Add a test asserting the closure in the direction that matters: the only entry constructible
      for a type is the one naming its own defining module — so a rogue type cannot be registered under
      a blessed origin at all. This replaces 3.1's test as the pinned invariant.

## 4. Move every surface that states the old bound

- [ ] 4.1 `PROJECT.md`: drop 漏刻's exception from the Core Contract's "Non-bypassable, precisely"
      paragraph. The scope sentence about the governed code's shape stays; the cooperative-registration
      carve-out goes.
- [ ] 4.2 `crates/louke/README.md`: replace the observed-at-the-registration-site paragraph with the
      derived-from-the-type statement, including the newtype answer for a foreign type.
- [ ] 4.3 `README.md` and `COOKBOOK.md`: update the `register_origin!` narration — the invocation is
      unchanged, but "registered inside its own module" stops being a requirement of the idiom and
      becomes a consequence of what an origin is.
- [ ] 4.3a Give those samples a reaction, or state that they have none. `crates/tianheng/src/lib.rs`'s
      `ReadmeDoctests` compiles `crates/tianheng/README.md` — which mentions `register_origin!`
      **zero** times — while the root `README.md` and `COOKBOOK.md`, which do carry the samples, are
      included by nothing and therefore compiled by nothing. The idiom exists precisely so "snippets
      cannot rot (a wrong signature or removed export fails `cargo test`)", so a sample of the API this
      change reshapes sits outside the net that exists for it. Either bring the sample under a
      `#[cfg(doctest)]` include (watch it fail against a deliberately wrong invocation first) or record
      the bound where the sample lives. Do not leave it as prose the next macro change can silently rot.
- [ ] 4.4 `CHANGELOG.md`: one `**BREAKING**` entry naming the closure, the origin's new meaning, the
      byte-identical result for the documented idiom, the migration step for a registration written
      away from its type's module, and the absence of any baseline impact. Also correct this window's
      own earlier entries, which now describe a cooperative trust boundary that no longer exists. Record
      that the 0.3.x residual was CI-preventable for a Tianheng-governed workspace (圭表's
      `must_not_call_inline("louke::OriginEntry").strict_external()` reacts to the hand-written bypass),
      which the prose never said — as history, not as a recipe for a hole that is now closed.
- [ ] 4.5 `BACKLOG.md`: close the DESIGN-BREAKING entry (keep its reproduction record), and retire the
      two closure paths it lists as open options.

## 5. Verify and land

- [ ] 5.1 Run the full Definition of Done, including `bash scripts/check_dod_coherence.sh` and both
      `cargo doc --document-private-items` passes.
- [ ] 5.2 Run `bash scripts/test_examples.sh` — `examples/composed` registers inside each adapter's own
      module, so its `only_origins(["composed_app::adapters::blessed"])` must still react exactly as
      declared with **no** edit. If it needs one, decision 2 of `design.md` is wrong and this change
      stops here.
- [ ] 5.3 `openspec validate --all`, then sync the delta into `openspec/specs/runtime-origin-assertion/`
      and prune the dated archive copy.
- [ ] 5.4 Confirm the version is untouched (`0.3.0`) and open the PR into `release/0.4.0`.
