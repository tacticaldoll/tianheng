//! 圭表's declared observation bounds, typed.
//!
//! Each entry classifies a bound the static dimension's specs already declare, keyed on the id those specs
//! derive from the declaring scenario's heading. The scenario states the bound for a *reader*; this states what
//! kind of stop it is for a *reaction*, and `observation-bound-model`'s reaction holds the two sets equal.
//!
//! A library item rather than a test item, deliberately: a `#[cfg(test)]` declaration is compiled only when this
//! crate is under test, so nothing outside it could enumerate these, and the bijection needs one reaction that
//! sees every dimension at once.

use xuanji::{BoundDecl, BoundId, Extent, Owner, Reached};

/// Every observation bound 圭表 declares, in the order its specs declare them.
pub fn observation_bounds() -> Vec<BoundDecl> {
    vec![
        BoundDecl::pinned(
            BoundId::new(
                "crate-source-boundary/a-git-plus-version-dependency-is-flagged-though-it-would-publish-a-stated-bound",
            ),
            "a dependency declaring both `git` and `version` under a registry-only allowlist",
            Extent::Reached(Reached::OverReacts {
                because: "the rule governs the declared source kind, not publish-eligibility, so a dependency \
                          that would `cargo publish` successfully is still classified `Git`".into(),
            }),
            "source_rule_flags_every_git_source_outside_a_registry_or_path_allowlist",
        ),
        BoundDecl::pinned(
            BoundId::new(
                "crate-dependency-boundary/an-optional-dependency-edge-is-observed-as-a-declared-one-a-stated-bound",
            ),
            "a dependency edge declared `optional = true`, reachable only when a feature enables it",
            Extent::Reached(Reached::OverReacts {
                because: "the rules read the declared dependency set, and cargo reports an optional edge in \
                          it like any other, so an edge confined to a non-default feature is governed as \
                          though it were unconditional — no rule can express *only when that feature is on*"
                    .into(),
            }),
            "an_optional_dependency_edge_is_observed_as_a_declared_one",
        ),
        BoundDecl::pinned(
            BoundId::new(
                "external-crate-confinement/cfg-gated-code-is-observed-as-written-a-stated-bound",
            ),
            "a confined-crate import under a `#[cfg(...)]` the build would not enable",
            Extent::Reached(Reached::OverReacts {
                because: "the predicate is never evaluated, so a dead arm is observed as live — cfg-blindness \
                          inherited from the module scanner, which reacts wider than the build".into(),
            }),
            "confine_external_crate_is_cfg_blind_to_unenabled_cfg_arms",
        ),
        BoundDecl::pinned(
            BoundId::new(
                "external-crate-confinement/a-confined-crate-use-inside-a-string-or-macro-body-is-not-observed-a-stated-bound",
            ),
            "a confined-crate `use` written inside a string literal or a macro body",
            Extent::OutOfReach {
                because: "a comment is no token, a string literal is one literal token, and no import rule reads a `use` \
                          written inside a macro's group other than a `cfg_if!` arm"
                    .into(),
            },
            "confine_ignores_a_use_inside_a_string_literal_or_macro_body",
        ),
        BoundDecl::pinned(
            BoundId::new(
                "external-crate-confinement/an-extern-crate-declaration-is-not-observed-a-stated-bound",
            ),
            "`extern crate libc;` reaching a confined crate without a `use`",
            Extent::Reached(Reached::UnderReacts {
                because: "the rule observes `use` imports only, so a crate reached through an \
                          `extern crate` declaration and fully-qualified paths is not seen".into(),
                owner: Owner::Engine,
            }),
            "confine_ignores_an_extern_crate_declaration",
        ),
        BoundDecl::pinned(
            BoundId::new(
                "inline-symbol-path-confinement/a-future-read-verb-outside-the-declared-set-is-a-documented-bound",
            ),
            "a read expressed through a verb outside the adopter's declared set",
            Extent::Reached(Reached::UnderReacts {
                because: "the engine declines to guess which verbs are reads, so a verb the declaration \
                          omits is not observed".into(),
                owner: Owner::Adopter,
            }),
            "inline_a_verb_outside_the_declared_set_is_a_bound",
        ),
        BoundDecl::pinned_by_many(
            BoundId::new(
                "inline-symbol-path-confinement/a-receiver-method-read-is-a-documented-bound",
            ),
            "a read reached through a method call on a receiver, or through a path beginning with `<`",
            Extent::OutOfReach {
                because: "no type inference is performed on the receiver or the qualified type, so the confined \
                          path is never resolved from the call site".into(),
            },
            "inline_receiver_method_read_is_a_bound",
            ["inline_qualified_path_is_the_type_directed_bound"],
        ),
        BoundDecl::pinned(
            BoundId::new(
                "inline-symbol-path-confinement/a-path-taken-as-a-value-is-a-documented-bound-under-the-default",
            ),
            "a confined path mentioned in value position rather than called",
            Extent::Reached(Reached::UnderReacts {
                because: "value-position mentions are not observed under the default; the adopter's \
                          `strict_prefix_only()` reacts to them".into(),
                owner: Owner::Adopter,
            }),
            "inline_value_capture_is_a_bound_under_the_default",
        ),
        BoundDecl::pinned(
            BoundId::new(
                "inline-symbol-path-confinement/an-external-crate-re-export-is-a-documented-bound",
            ),
            "a confined path reached through an external crate's own re-export",
            Extent::OutOfReach {
                because: "foreign ASTs are not scanned, so a re-export chain leaving this workspace is \
                          never followed".into(),
            },
            "inline_foreign_reexport_of_the_confined_path_is_a_bound",
        ),
        BoundDecl::pinned(
            BoundId::new(
                "inline-symbol-path-confinement/a-prefix-segment-past-what-guibiao-reads-is-not-verified-a-stated-bound",
            ),
            "an inline-call prefix misspelled after a crate's name or after an item of the crate, or starting at a \
             segment nothing confirms and whose `crate::` reading names nothing",
            Extent::Reached(Reached::DeclinesToRefuse {
                because: "another crate's contents are its own source and associated items are not collected, and \
                          a dependency's crate name can differ from what `--no-deps` metadata reports, so refusing \
                          what those segments name would refuse prefixes that are right"
                    .into(),
            }),
            "a_prefix_past_what_guibiao_reads_is_not_verified",
        ),
        BoundDecl::pinned(
            BoundId::new(
                "module-boundary/a-cfg-before-a-separator-its-construct-holds-is-not-read-a-stated-bound",
            ),
            "a `mod` with no file in a block whose item, statement, match arm, parameter or field carries a `cfg` \
             and holds a `,`, a brace group or an attribute of its own before that block — a generic list's or a \
             `where` clause's comma, a closure's parameters, an `if`'s block before `else`, a struct literal or \
             pattern, a tuple struct's earlier fields, or a type's generic arguments before an array length",
            Extent::Reached(Reached::RefusesToJudge {
                because: "the owner of an enclosing group is read back to the previous `;`, `,`, brace group or \
                          attribute, so a construct holding one of those before the group is not reached and its \
                          `cfg` is not read; the missing file is refused rather than tolerated"
                    .into(),
            }),
            "a_cfg_before_a_separator_its_construct_holds_is_not_read",
        ),
        BoundDecl::pinned(
            BoundId::new(
                "inline-symbol-path-confinement/a-prefix-naming-a-macro-generated-item-is-refused-a-stated-bound",
            ),
            "a `crate::` inline-call prefix naming an item a macro invocation defines",
            Extent::Reached(Reached::RefusesToJudge {
                because: "no declaration inside a macro invocation's group other than a `cfg_if!` arm is recorded, so \
                          the item is absent from the set the prefix is held to and the prefix is refused as naming nothing"
                    .into(),
            }),
            "a_prefix_naming_a_macro_generated_item_is_refused",
        ),
        BoundDecl::pinned(
            BoundId::new(
                "inline-symbol-path-confinement/the-fully-qualified-external-call-is-a-stated-bound-under-the-default",
            ),
            "a fully-qualified call into an external crate with no `use`",
            Extent::Reached(Reached::UnderReacts {
                because: "the default observes `use`-rooted paths, leaving the un-`use`d fully-qualified \
                          spelling to the adopter's stricter opt-in".into(),
                owner: Owner::Adopter,
            }),
            "inline_strict_external_absent_fully_qualified_call_is_a_bound",
        ),
        BoundDecl::pinned(
            BoundId::new(
                "module-boundary/an-example-test-bench-or-build-script-root-is-not-governed-a-stated-bound",
            ),
            "an example, test, bench or build-script target's source",
            Extent::Reached(Reached::UnderReacts {
                because: "the governed corpus is a package's library-kind and `bin` roots, the code it ships; \
                          a target compiled beside those rather than into them is outside it"
                    .into(),
                owner: Owner::Engine,
            }),
            "an_example_root_is_not_governed",
        ),
        BoundDecl::pinned_by_many(
            BoundId::new(
                "inline-symbol-path-confinement/a-macro-generated-item-called-bare-in-its-own-module-is-not-observed-a-stated-bound",
            ),
            "a bare call, in its own module, of an item a macro invocation generates",
            Extent::Reached(Reached::UnderReacts {
                because: "no declaration inside a macro invocation's group other than a `cfg_if!` arm is recorded, so \
                          the generated item is not in the scope table and a head no scope binds names nothing; a crate-rooted path naming \
                          it from another module still reacts"
                    .into(),
                owner: Owner::Engine,
            }),
            "a_macro_generated_item_called_bare_in_its_module_is_a_bound",
            ["a_crate_rooted_call_of_a_macro_generated_item_reports"],
        ),
        BoundDecl::pinned(
            BoundId::new(
                "inline-symbol-path-confinement/a-use-written-in-a-macro-group-outside-any-block-binds-nothing-a-stated-bound",
            ),
            "a path beside a `use` written directly in a macro's group, outside any block the group holds",
            Extent::Reached(Reached::UnderReacts {
                because: "where a macro expands what its group holds is not read, so a `use` written directly in the \
                          group binds in no scope, and a path it would bind names nothing; a `use` a block in the group \
                          holds binds that block's paths"
                    .into(),
                owner: Owner::Engine,
            }),
            "a_use_written_in_a_macro_group_outside_any_block_binds_nothing",
        ),
        BoundDecl::pinned(
            BoundId::new(
                "inline-symbol-path-confinement/a-prelude-name-called-bare-is-not-read-as-its-std-path-a-stated-bound",
            ),
            "a bare call of a prelude name — `drop(x)`, `Some(..)`, `Box::new(..)` — under a standard-library prefix",
            Extent::Reached(Reached::UnderReacts {
                because: "the prelude's contents are not read, so a head no scope binds names nothing rather \
                          than the standard-library path the prelude would give it"
                    .into(),
                owner: Owner::Engine,
            }),
            "a_prelude_name_called_bare_is_not_read_as_its_std_path",
        ),
        BoundDecl::pinned(
            BoundId::new(
                "inline-symbol-path-confinement/a-parenthesized-fn-bound-is-read-as-a-call-a-stated-bound",
            ),
            "a trait bound of the `Fn` family written with parenthesized arguments — `F: Fn(u8) -> u8`, \
             `impl FnOnce()`, `dyn FnMut(u8)`",
            Extent::Reached(Reached::OverReacts {
                because: "a path's role is read from the tokens beside it, and a parenthesized bound is written as a \
                          call is, so no reading of the tokens tells the bound from the call"
                    .into(),
            }),
            "a_parenthesized_fn_bound_is_read_as_a_call",
        ),
        BoundDecl::pinned(
            BoundId::new(
                "inline-symbol-path-confinement/a-path-in-a-pattern-position-is-read-as-a-call-a-stated-bound",
            ),
            "a tuple-struct or tuple-variant path in a pattern position — a `let`, `if let`, `while let` or \
             let-else pattern, a `for` loop's, a match arm's, a `fn` or closure parameter's, a macro's arguments, \
             a destructuring assignment's left side",
            Extent::Reached(Reached::OverReacts {
                because: "a path's role is read from the tokens beside it, and a tuple-struct or tuple-variant \
                          pattern followed by its parenthesized fields is written as a call is, so no reading of \
                          the tokens tells the pattern from the call"
                    .into(),
            }),
            "a_path_in_a_pattern_position_is_read_as_a_call",
        ),
        BoundDecl::pinned(
            BoundId::new(
                "inline-symbol-path-confinement/a-qualified-path-after-a-closing-brace-is-read-as-a-rooted-path-a-stated-bound",
            ),
            "under `.strict_external()`, the tail of a qualified path opening a statement right after a `}` — \
             `if c {} <W>::md5x();`",
            Extent::Reached(Reached::OverReacts {
                because: "a `}` ends a block-like operand as well as a statement, and is read as an operand's \
                          end so a comparison after a block never opens a qualified path; the `<` after it is \
                          then a comparison, and the tail after its `>` is read as a rooted path, which names a \
                          dependency where its first segment is one"
                    .into(),
            }),
            "a_qualified_path_after_a_closing_brace_is_read_as_a_rooted_path",
        ),
        BoundDecl::pinned(
            BoundId::new(
                "inline-symbol-path-confinement/a-qualified-path-a-shift-opens-in-a-generic-list-is-read-as-a-rooted-path-a-stated-bound",
            ),
            "under `.strict_prefix_only()` and `.strict_external()`, the tail of a qualified path the second `<` of \
             a `<<` opens in a generic list — `Vec<<u8 as Tr>::md5x>`",
            Extent::Reached(Reached::OverReacts {
                because: "the token before the `<<` ends an operand, as in `1 << n > ::std::process::id() && n > 0`, \
                          so the `<<` is read as a shift and the tail after the inner `>` as a rooted path, which \
                          names a dependency where its first segment is one; telling the list from the shift needs a \
                          type from a value"
                    .into(),
            }),
            "a_qualified_path_a_shift_opens_in_a_generic_list_is_read_as_a_rooted_path",
        ),
        BoundDecl::pinned(
            BoundId::new(
                "inline-symbol-path-confinement/a-generic-parameter-named-like-an-import-is-read-as-the-import-a-stated-bound",
            ),
            "a head naming a generic parameter that shares its name with a `use` of the enclosing module",
            Extent::Reached(Reached::OverReacts {
                because: "generic parameter lists are not read, so the head is resolved through whatever the \
                          module binds under that name"
                    .into(),
            }),
            "inline_generic_parameter_named_like_an_import_is_read_as_the_import",
        ),
        BoundDecl::pinned(
            BoundId::new(
                "inline-symbol-path-confinement/a-local-binding-named-like-an-import-is-read-as-the-import-a-stated-bound",
            ),
            "a bare head naming a `fn` or closure parameter, or a `let` binding, that shares its name with an import or \
             an item in scope",
            Extent::Reached(Reached::OverReacts {
                because: "parameters and `let` bindings are not recorded in the scope table, so the head is resolved \
                          through whatever the enclosing scopes bind under that name"
                    .into(),
            }),
            "a_local_binding_named_like_an_import_is_read_as_the_import",
        ),
        BoundDecl::pinned(
            BoundId::new(
                "inline-symbol-path-confinement/an-import-in-a-block-of-what-is-not-read-is-read-with-the-scope-around-it-a-stated-bound",
            ),
            "a bare head a block binds only through an import of what the scanner does not read, which the scope \
             around the block also binds",
            Extent::Reached(Reached::OverReacts {
                because: "whether such an import holds the name in the namespace the head is read in is not read, so \
                          the head is resolved through the block's import and through the scope around the block both"
                    .into(),
            }),
            "a_block_import_of_what_is_not_read_is_read_with_the_scope_around_it",
        ),
        BoundDecl::pinned(
            BoundId::new(
                "inline-symbol-path-confinement/an-import-of-what-is-not-read-beside-a-glob-is-read-with-the-glob-a-stated-bound",
            ),
            "a bare head a scope binds only through an import of what the scanner does not read, which a glob of \
             that scope also brings",
            Extent::Reached(Reached::OverReacts {
                because: "whether such an import holds the name in the namespace the head is read in is not read, so \
                          the head is resolved through the import and through the scope's globs both"
                    .into(),
            }),
            "an_import_of_what_is_not_read_beside_a_glob_is_read_with_the_glob",
        ),
        BoundDecl::pinned(
            BoundId::new(
                "inline-symbol-path-confinement/a-cfg-closed-re-export-ring-is-read-in-time-exponential-in-its-length-a-stated-bound",
            ),
            "a ring of modules each re-exporting a name from the next under one `cfg` and from elsewhere under its \
             negation, the last closing it",
            Extent::Reached(Reached::DeclinesToRefuse {
                because: "an answer read past a cut cycle depends on the walk it was entered from and is not \
                          remembered, so each link is re-read once per path to it and the reading doubles per link; \
                          no budget refuses it, and the chain cap bounds the depth rather than the time"
                    .into(),
            }),
            "a_cfg_closed_re_export_ring_is_read_in_time_exponential_in_its_length",
        ),
        BoundDecl::pinned(
            BoundId::new(
                "inline-symbol-path-confinement/a-cfg-gated-name-beside-a-glob-is-read-with-the-glob-a-stated-bound",
            ),
            "a name a scope binds or declares only by items a `cfg` gates, which a glob of that scope, or the scope \
             around a block, also names",
            Extent::Reached(Reached::OverReacts {
                because: "the predicate is never evaluated, so on a build that compiles the gated item in, the name is \
                          resolved through what the lookup reads past the scope as well as through it"
                    .into(),
            }),
            "a_cfg_gated_name_beside_a_glob_is_read_with_the_glob",
        ),
        BoundDecl::pinned(
            BoundId::new(
                "inline-symbol-path-confinement/a-glob-reacts-to-any-alias-or-re-export-beneath-its-resolved-module-a-stated-bound",
            ),
            "a glob import whose resolved module has, anywhere beneath it, a `type` alias or `pub use` of the confined \
             prefix, whether or not the glob actually imports that name",
            Extent::Reached(Reached::OverReacts {
                because: "the glob hazard asks whether any definition beneath the glob's resolved module resolves under \
                          the prefix, not whether the glob brings that name into scope"
                    .into(),
            }),
            "a_sibling_test_glob_reacts_to_an_alias_in_its_resolved_module",
        ),
    ]
}
