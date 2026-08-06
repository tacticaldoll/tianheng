//! 漏刻's declared observation bounds, typed.
//!
//! Each entry classifies a bound the runtime dimension's spec already declares, keyed on the id that spec
//! derives from the declaring scenario's heading. See `xuanji::bound` for what each extent means and why the
//! type is nested.
//!
//! **The set depends on the `audit` feature**, because a bound is a property of a *reaction* and an audit-OFF
//! build contains none of the probe audit. Five of the six declarations describe `audit_probe_coverage` — the
//! scanner that lives behind `audit`, as `observer` does and for the same reason — so declaring them in a build
//! that compiles no scanner would tell a reader a bound exists for a reaction the crate does not have. The one
//! that survives audit-OFF describes the always-present origin derivation on the hot path.

use xuanji::{BoundDecl, BoundId, Extent, Reached};
// Only the audit-scoped declarations name an owner or a fact granularity, so both imports are gated with
// them. `cargo clippy -p louke` — the isolated audit-OFF pass — is what would report either as unused.
#[cfg(feature = "audit")]
use xuanji::{FactGranularity, Owner};

/// The observation bounds 漏刻 declares **in this build**.
///
/// Not "every bound its spec declares": five of the six describe the probe audit, which an audit-OFF build does
/// not contain. See this module's header.
pub fn observation_bounds() -> Vec<BoundDecl> {
    // Mutable only where something extends it. The allow is scoped to the configuration where the statement
    // below is compiled out, rather than blanket — `cargo clippy -p louke` reported this the moment the five
    // audit declarations moved behind the gate, which is the pass that exists for exactly this class.
    #[cfg_attr(not(feature = "audit"), allow(unused_mut))]
    let mut bounds = vec![        BoundDecl::new(
            BoundId::new(
                "runtime-origin-assertion/a-composite-shape-yields-a-truncated-origin-a-stated-bound",
            ),
            "a registered type that is a reference, tuple, array, pointer, or function pointer",
            Extent::Reached(Reached::OverReacts {
                because: "the derived origin is a truncated rendering matching no module name, so the \
                          crossing reacts fail-closed rather than being admitted through the wrapper".into(),
            }),
            "the_derived_origin_honors_its_stated_shape_bounds",
        ),
    ];
    #[cfg(feature = "audit")]
    bounds.extend(audit_bounds());
    bounds
}

/// The bounds of the **probe audit**, present only where the audit is compiled.
///
/// Gated for the reason written above `mod observer`: the reaction these describe lives behind `audit`, and a
/// declaration without its reaction is the unbacked claim the bound model exists to refuse. The order within
/// this list is free — every projection of the register sorts by id.
#[cfg(feature = "audit")]
fn audit_bounds() -> Vec<BoundDecl> {
    vec![        BoundDecl::new(
            BoundId::new(
                "runtime-origin-assertion/source-outside-a-member-s-library-or-binary-target-subtree-is-out-of-scope-a-stated-bound",
            ),
            "a probe or seam mention in `tests/`, `examples/`, or `build.rs`",
            Extent::OutOfReach {
                because: "the audit's corpus is the member's library and binary targets, so it never reads \
                          those files at all".into(),
            },
            "source_outside_lib_or_bin_target_subtree_is_out_of_scope_corpus_bound",
        ),
        BoundDecl::new(
            BoundId::new(
                "runtime-origin-assertion/a-production-probe-behind-a-non-production-cfg-is-still-counted-a-stated-bound",
            ),
            "a seam whose only probe sits behind `#[cfg(test)]` or another non-production predicate",
            Extent::Reached(Reached::UnderReacts {
                because: "the audit is cfg-blind and counts the probe as coverage, so a seam with no \
                          production probe reads as probed".into(),
                owner: Owner::Engine,
            }),
            "production_probe_behind_non_production_cfg_is_counted_as_coverage",
        ),
        BoundDecl::new(
            BoundId::new(
                "runtime-origin-assertion/identical-expression-repeated-in-the-same-function-collapses-to-one-finding-a-stated-bound",
            ),
            "`assert_boundary!(SEAM, obj)` written twice verbatim in one function",
            Extent::Reached(Reached::AsIntended {
                bounded: FactGranularity::Identity,
                because: "no further source content distinguishes the two occurrences, so they share one \
                          finding — the site still reacts".into(),
            }),
            "identical_expression_repeated_in_the_same_function_collapses_to_one_violation",
        ),
        BoundDecl::new(
            BoundId::new(
                "runtime-origin-assertion/an-absolute-path-literal-s-target-outside-the-anchor-keeps-its-absolute-label-a-stated-bound",
            ),
            "a module reached only through an absolute `#[path = \"/…\"]` outside the scanning anchor",
            Extent::Reached(Reached::AsIntended {
                bounded: FactGranularity::Presentation,
                because: "the literal has no textual relationship to the anchor, so the site is named with \
                          the raw absolute path — the violation is still emitted".into(),
            }),
            "an_absolute_path_literal_outside_the_anchor_keeps_the_path_the_literal_wrote",
        ),
        BoundDecl::new(
            BoundId::new(
                "runtime-origin-assertion/a-probe-behind-a-symlinked-subdirectory-is-seen-from-the-root-and-not-from-the-directory-a-stated-bound",
            ),
            "a seam whose only probe sits in a module reached by `#[path]` into a symlinked directory",
            Extent::Reached(Reached::UnderReacts {
                because: "the root-file run reports the seam covered while the directory run reports it \
                          unprobed, so which entry point observed it decides the answer".into(),
                // Not a value of its own: one entry-dependent instance does not earn one, and the direction
                // that matters — a seam reported covered when it is not — is recorded either way. The entry
                // point is the layer, so the ownership is inherited rather than this engine's.
                owner: Owner::Inherited {
                    from: "the corpus entry point".into(),
                },
            }),
            "a_symlinked_subdirectory_is_descended_from_a_root_file_and_not_from_a_directory",
        ),
    ]
}
