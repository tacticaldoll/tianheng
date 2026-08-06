//! 圭表's declared observation bounds, typed.
//!
//! Each entry classifies a bound the static dimension's specs already declare, keyed on the id those specs
//! derive from the declaring scenario's heading. The scenario states the bound for a *reader*; this states what
//! kind of stop it is for a *reaction*, and `observation-bound-model`'s reaction holds the two sets equal.
//!
//! A library item rather than a test item, deliberately: a `#[cfg(test)]` declaration is compiled only when this
//! crate is under test, so nothing outside it could enumerate these, and the bijection needs one reaction that
//! sees every dimension at once.

use xuanji::{BoundDecl, BoundId, Extent, FactGranularity, Owner, Reached};

/// Every observation bound 圭表 declares, in the order its specs declare them.
pub fn observation_bounds() -> Vec<BoundDecl> {
    vec![
        // --- crate-source-boundary ---
        BoundDecl::new(
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
        // --- external-crate-confinement ---
        BoundDecl::new(
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
        BoundDecl::new(
            BoundId::new(
                "external-crate-confinement/the-lib-and-bin-conventional-path-conflation-is-a-stated-bound",
            ),
            "a package whose library and binary conventional source paths coincide",
            Extent::Reached(Reached::AsIntended {
                bounded: FactGranularity::Identity,
                because: "the two targets' module graphs are not told apart, so a finding names one \
                          compilation unit where two share a path — the reaction still fires".into(),
            }),
            "confine_external_crate_conflates_coincident_lib_and_bin_conventional_paths",
        ),
        BoundDecl::new(
            BoundId::new(
                "external-crate-confinement/a-confined-crate-use-inside-a-string-or-macro-body-is-not-observed-a-stated-bound",
            ),
            "a confined-crate `use` written inside a string literal or a macro body",
            Extent::OutOfReach {
                because: "comments, string literals and macro bodies are stripped before scanning".into(),
            },
            "confine_ignores_a_use_inside_a_string_literal",
        ),
        BoundDecl::new(
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
        // --- inline-symbol-path-confinement ---
        BoundDecl::new(
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
        BoundDecl::new(
            BoundId::new(
                "inline-symbol-path-confinement/a-receiver-method-read-is-a-documented-bound",
            ),
            "a read reached through a method call on a receiver",
            Extent::OutOfReach {
                because: "no type inference is performed on the receiver, so the confined path is never \
                          resolved from the call site".into(),
            },
            "inline_receiver_method_read_is_a_bound",
        ),
        BoundDecl::new(
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
        BoundDecl::new(
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
        BoundDecl::new(
            BoundId::new(
                "inline-symbol-path-confinement/an-extern-crate-rename-is-a-stated-bound-under-strict-external",
            ),
            "a call reached through an `extern crate … as` alias head under strict-external",
            Extent::Reached(Reached::UnderReacts {
                because: "the use-map is built from `use` declarations only, so an `extern crate` rename \
                          binds an alias the resolver does not know".into(),
                owner: Owner::Engine,
            }),
            "inline_strict_external_extern_crate_rename_is_a_stated_bound",
        ),
        BoundDecl::new(
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
    ]
}
