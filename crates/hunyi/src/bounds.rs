//! 渾儀's declared observation bounds, typed.
//!
//! Each entry classifies a bound one of the semantic dimension's specs already declares, keyed on the id that
//! spec derives from the declaring scenario's heading. See `xuanji::bound` for what each extent means and why
//! the type is nested.
//!
//! Most of this dimension's bounds are `OutOfReach`, and the reason is one shape repeated: an AST the resolver
//! cannot follow — a macro it does not expand, a glob whose leaves it cannot enumerate, a foreign crate it does
//! not parse. That is visible here as a group precisely because the classification is typed rather than worded.

use xuanji::{BoundDecl, BoundId, Extent, FactGranularity, Owner, Reached};

/// Every observation bound 渾儀 declares, grouped by the capability that declares it.
pub fn observation_bounds() -> Vec<BoundDecl> {
    vec![
        // --- semantic-async-exposure-boundary ---
        BoundDecl::pinned(
            BoundId::new("semantic-async-exposure-boundary/a-body-nested-module-is-a-stated-bound"),
            "`pub async fn` inside a `mod` declared in a function body",
            Extent::Reached(Reached::NotAViolation {
                because: "a `mod` inside a fn body is not public API — it is unreachable as `crate::…`, so \
                          there is nothing exposed to react to".into(),
            }),
            "async_subtree_does_not_observe_a_body_nested_module",
        ),
        // --- semantic-dyn-trait-boundary ---
        BoundDecl::pinned(
            BoundId::new(
                "semantic-dyn-trait-boundary/a-public-item-naming-such-an-alias-is-not-expanded-a-stated-bound",
            ),
            "a public item whose signature names a public alias that itself holds a `dyn`",
            Extent::Reached(Reached::AsIntended {
                bounded: FactGranularity::Identity,
                because: "the `dyn` is already caught at the alias declaration, so naming it again through \
                          the item would be a second finding for one shape".into(),
            }),
            "a_private_alias_hiding_a_dyn_is_a_stated_bound",
        ),
        BoundDecl::pinned(
            BoundId::new(
                "semantic-dyn-trait-boundary/a-private-alias-hiding-a-dyn-in-a-public-position-is-a-stated-bound",
            ),
            "a non-public `type` alias holding a `dyn`, named by a public signature",
            Extent::OutOfReach {
                because: "the resolver does not expand `type` aliases, so the `dyn` is never seen from the \
                          public position that exposes it".into(),
            },
            "a_private_alias_hiding_a_dyn_is_a_stated_bound",
        ),
        BoundDecl::pinned(
            BoundId::new("semantic-dyn-trait-boundary/a-macro-generated-dyn-is-a-documented-bound"),
            "a `dyn` appearing only in a macro's expansion, with no `dyn` token at the call site",
            Extent::OutOfReach {
                because: "macros are not expanded — the universal 渾儀 macro-expansion bound — so the token \
                          never enters the observed AST".into(),
            },
            "a_macro_generated_dyn_is_a_documented_coverage_bound",
        ),
        BoundDecl::pinned(
            BoundId::new("semantic-dyn-trait-boundary/an-unrenderable-sub-node-is-a-stated-bound"),
            "two trait objects differing only inside a sub-node that cannot be rendered",
            Extent::Reached(Reached::AsIntended {
                bounded: FactGranularity::Identity,
                because: "a complex const-generic expression, a same-named macro, a `verbatim` type or a \
                          lifetime cannot be rendered stably, so the two share one subject and key — each \
                          still reacts on first occurrence, and only baseline-dedup granularity is bounded".into(),
            }),
            "an_unrenderable_sub_node_is_a_stated_rendering_bound",
        ),
        // --- semantic-dyn-trait-operand-boundary ---
        BoundDecl::pinned(
            BoundId::new(
                "semantic-dyn-trait-operand-boundary/a-genuinely-unresolvable-bare-principal-is-a-documented-bound",
            ),
            "`dyn Frobnicate` where the bare principal has no `use`, no dependency, and no local declaration",
            Extent::OutOfReach {
                because: "the oracle does not over-reach a single bare segment, so a prelude or glob-imported \
                          trait resolves to nothing rather than to a guess".into(),
            },
            "dyn_operand_genuinely_unresolvable_bare_principal_is_a_bound",
        ),
        // --- semantic-forbidden-marker ---
        BoundDecl::pinned(
            BoundId::new(
                "semantic-forbidden-marker/an-unresolvable-hand-impl-self-type-is-a-documented-bound",
            ),
            "a hand-written impl whose self-type arrives through a glob import",
            Extent::OutOfReach {
                because: "a glob's leaves are not enumerable, so the self-type cannot be resolved to its \
                          definition — the co-located, `use`-imported, re-export-spelled and alias cases do \
                          resolve and react".into(),
            },
            "an_unresolvable_glob_self_type_is_a_documented_bound",
        ),
        // --- semantic-impl-trait-operand-boundary ---
        BoundDecl::pinned(
            BoundId::new(
                "semantic-impl-trait-operand-boundary/a-genuinely-unresolvable-bare-principal-is-a-documented-bound",
            ),
            "`impl Frobnicate` where the bare principal has no `use`, no dependency, and no local declaration",
            Extent::OutOfReach {
                because: "the same resolver limit as the `dyn` operand dimension — a single bare segment is \
                          not over-reached".into(),
            },
            "impl_trait_operand_genuinely_unresolvable_bare_principal_is_a_bound",
        ),
        // --- semantic-reexport-exposure ---
        BoundDecl::pinned(
            BoundId::new("semantic-reexport-exposure/an-underscore-rename-is-a-documented-bound"),
            "`pub use crate::infra::DbPool as _;` under a boundary forbidding that module",
            Extent::Reached(Reached::NotAViolation {
                because: "`as _` binds no nameable path a consumer can reach, so nothing is exposed".into(),
            }),
            "restricted_and_private_and_underscore_reexports_do_not_react",
        ),
        BoundDecl::pinned(
            BoundId::new("semantic-reexport-exposure/a-sibling-root-glob-is-a-documented-bound"),
            "`pub use crate::elsewhere::*;` where that module transitively re-exports a forbidden type",
            Extent::OutOfReach {
                because: "the glob's leaves are not enumerable here, so the transitively re-exported leaf is \
                          never seen".into(),
            },
            "sibling_root_glob_does_not_react",
        ),
        BoundDecl::pinned(
            BoundId::new(
                "semantic-reexport-exposure/an-ancestor-root-glob-spanning-a-deeper-forbidden-prefix-is-a-documented-bound",
            ),
            "`pub use crate::infra::*;` where the forbidden prefix is deeper than the glob root",
            Extent::OutOfReach {
                because: "whether the glob root publicly re-exports the deeper forbidden subtree cannot be \
                          enumerated — the sharper sub-case of the sibling glob, declared separately rather \
                          than lumped with it".into(),
            },
            "ancestor_root_glob_over_a_deeper_forbidden_prefix_does_not_react",
        ),
        BoundDecl::pinned(
            BoundId::new(
                "semantic-reexport-exposure/a-facade-hop-re-exporting-a-privately-used-bare-name-is-a-stated-bound",
            ),
            "a private import followed by a bare `pub use Foo;`, re-exported onward",
            Extent::Reached(Reached::UnderReacts {
                because: "the closure captures inline `pub use` paths only, so the hop through a privately \
                          imported bare name is not followed".into(),
                owner: Owner::Engine,
            }),
            "facade_hop_reexporting_a_privately_used_bare_name_is_a_stated_bound",
        ),
        BoundDecl::pinned(
            BoundId::new(
                "semantic-reexport-exposure/a-non-forbidden-root-external-glob-is-a-documented-bound",
            ),
            "`pub use worklane_core::spi::*;` under a boundary forbidding a different module of that crate",
            Extent::OutOfReach {
                because: "an external glob's individual leaves are not enumerable, so none is observed".into(),
            },
            "extern_glob_nonforbidden_root_is_a_stated_bound",
        ),
        BoundDecl::pinned(
            BoundId::new(
                "semantic-reexport-exposure/a-re-export-renamed-through-a-foreign-module-is-a-documented-bound",
            ),
            "a re-export of a foreign prelude path that itself re-exports a forbidden module's type",
            Extent::OutOfReach {
                because: "the foreign chain is not parsed, so only the written path is matched — the reaction \
                          never claims to have followed it".into(),
            },
            "foreign_prelude_rename_is_a_stated_bound",
        ),
        BoundDecl::pinned(
            BoundId::new(
                "semantic-reexport-exposure/a-module-scoped-extern-crate-rename-is-a-documented-bound",
            ),
            "`extern crate worklane_core as wc;` declared inside a module rather than at the crate root",
            Extent::Reached(Reached::UnderReacts {
                because: "only crate-root renames are collected, since a module-scoped alias binds locally, \
                          so the alias head is not resolved to the crate it names".into(),
                owner: Owner::Engine,
            }),
            "module_scoped_extern_crate_rename_is_a_stated_bound",
        ),
        // --- semantic-signature-coupling ---
        BoundDecl::pinned(
            BoundId::new(
                "semantic-signature-coupling/an-invocation-inside-an-impl-body-is-a-stated-bound",
            ),
            "a `cfg_if!` invocation inside an `impl` body exposing a forbidden type",
            Extent::Reached(Reached::UnderReacts {
                because: "transparency covers item position, and an impl-body invocation's arms are impl \
                          items observed through different walkers — a declared gap rather than a claimed \
                          reaction".into(),
                owner: Owner::Engine,
            }),
            "a_cfg_if_inside_an_impl_body_is_a_stated_bound",
        ),
        BoundDecl::pinned(
            BoundId::new(
                "semantic-signature-coupling/a-macro-under-another-name-is-not-treated-as-transparent-a-stated-bound",
            ),
            "an arbitrary macro invocation whose body holds item-shaped content",
            Extent::OutOfReach {
                because: "the invocation is not transparent and its body is not read — extracting from it \
                          would read an `impl` body's braces as an arm and report an item the macro may never \
                          emit".into(),
            },
            "an_arbitrary_macro_body_is_not_read_as_transparent_arms",
        ),
        BoundDecl::pinned(
            BoundId::new(
                "semantic-signature-coupling/a-plain-item-nested-the-same-way-stays-a-stated-bound",
            ),
            "a plain item nested inside a body, where an `impl` in the same position is recovered",
            Extent::Reached(Reached::NotAViolation {
                because: "a plain item there is genuinely scoped to that body and unreachable as `crate::…`, \
                          exactly like the body-nested-module bound".into(),
            }),
            "a_plain_fn_directly_in_a_const_body_stays_a_stated_bound",
        ),
        BoundDecl::pinned_by_many(
            BoundId::new(
                "semantic-signature-coupling/an-impl-nested-one-level-further-or-static-wrapped-is-a-stated-bound",
            ),
            "an `impl` nested one level deeper than the recovered position, or wrapped in a `static`",
            Extent::Reached(Reached::UnderReacts {
                because: "only an `impl` directly in such a body is recovered, so a deeper or \
                          `static`-wrapped one exposes without being observed".into(),
                owner: Owner::Engine,
            }),
            "an_impl_nested_one_level_further_stays_a_stated_bound",
            ["a_static_wrapped_impl_stays_a_stated_bound"],
        ),
        // --- semantic-trait-impl-exposure ---
        BoundDecl::pinned(
            BoundId::new(
                "semantic-trait-impl-exposure/a-glob-imported-type-in-an-impl-position-is-a-documented-bound",
            ),
            "an impl position naming a type that arrives through a glob import",
            Extent::OutOfReach {
                because: "a glob's leaves are not enumerable, so the type in that position resolves to \
                          nothing".into(),
            },
            "a_glob_imported_type_in_an_impl_position_is_a_documented_coverage_bound",
        ),
        // --- semantic-trait-impl-locality ---
        BoundDecl::pinned(
            BoundId::new(
                "semantic-trait-impl-locality/a-macro-generated-impl-is-a-documented-bound",
            ),
            "an `impl` appearing only in a macro's expansion",
            Extent::OutOfReach {
                because: "macros are not expanded, so the impl never enters the observed AST".into(),
            },
            "a_macro_generated_impl_is_a_documented_bound",
        ),
        BoundDecl::pinned(
            BoundId::new(
                "semantic-trait-impl-locality/a-cfg-gated-module-with-an-absent-file-is-skipped-not-a-scan-error-a-stated-bound",
            ),
            "a cfg-gated module declaration whose source file is absent from the checkout",
            Extent::Reached(Reached::DeclinesToRefuse {
                because: "the whole-crate walk skips the module rather than failing the gate with a scan \
                          error, because an absent cfg-gated file is an ordinary checkout state and refusing \
                          to judge on it would make the gate unusable".into(),
            }),
            "hunyi::a_cfg_gated_module_with_no_file_is_skipped_not_errored",
        ),
        // --- semantic-unsafe-confinement ---
        BoundDecl::pinned(
            BoundId::new(
                "semantic-unsafe-confinement/macro-generated-unsafe-is-a-documented-bound",
            ),
            "an `unsafe` block or item appearing only in a macro's expansion",
            Extent::OutOfReach {
                because: "macros are not expanded, so the `unsafe` token never enters the observed AST".into(),
            },
            "unsafe_in_a_macro_body_is_a_stated_bound",
        ),
        // --- semantic-visibility-boundary ---
        BoundDecl::pinned(
            BoundId::new(
                "semantic-visibility-boundary/a-macro-generated-item-is-a-documented-bound",
            ),
            "a `pub` item appearing only in a macro's expansion",
            Extent::OutOfReach {
                because: "macros are not expanded, so the item never enters the observed AST".into(),
            },
            "a_macro_invocation_pub_item_is_a_documented_bound",
        ),
        BoundDecl::pinned(
            BoundId::new(
                "semantic-visibility-boundary/a-pub-in-narrow-path-item-may-over-react-under-a-tight-ceiling-a-stated-bound",
            ),
            "`pub(in crate::a) fn` on an item already directly in `crate::a`, under a `Module` ceiling",
            Extent::Reached(Reached::OverReacts {
                because: "the conservative `Crate` rank exceeds the `Module` ceiling, so an effectively \
                          private item may react — never a silent pass".into(),
            }),
            "a_pub_in_narrow_path_over_reacts_under_a_module_ceiling",
        ),
    ]
}
