use super::super::*;
use super::helpers::*;
// --- trait-impl-locality ------------------------------------------------

pub(super) fn locality_findings(
    name: &str,
    files: &[(&str, &str)],
    trait_path: &str,
    allowed: &[&str],
) -> Result<Vec<String>, String> {
    let tree = TempSrcTree::new(&format!("loc-{name}"));
    tree.write_all(files);
    let allowed: Vec<String> = allowed.iter().map(|s| s.to_string()).collect();
    let result = trait_impl_findings(tree.src(), &tree.root(), trait_path, &allowed, "x");
    // The pure-heart tests assert on findings only; drop the resolved anchor and the per-finding
    // module/file here. `locality_anchor` below is for the tests that are about the anchor itself.
    result.map(|reaction| {
        reaction
            .findings
            .into_iter()
            .map(|(finding, _module, _file)| finding.to_string())
            .collect()
    })
}

/// The resolved trait anchor a declaration denotes — the value that becomes the violation's `target`
/// and rule key, so a test can assert identity stability across two equivalent spellings without
/// reconstructing a whole `Violation`.
pub(super) fn locality_anchor(
    name: &str,
    files: &[(&str, &str)],
    trait_path: &str,
) -> Result<String, String> {
    let tree = TempSrcTree::new(&format!("loc-anchor-{name}"));
    tree.write_all(files);
    trait_impl_findings(tree.src(), &tree.root(), trait_path, &[], "x").map(|r| r.anchor)
}

/// Two mutually-exclusive `#[cfg]`-gated `use ... as T;` aliases for an `impl T for Foo`'s trait
/// name: the anchor match must react regardless of which alias is declared first. Before the fix,
/// `resolve_path`'s single-candidate lookup (plus the single-candidate `canonicalize_through_reexports`)
/// took only one `use`-map entry, so whether the anchored trait was ever seen depended on
/// declaration order (found on adversarial review of `hunyi-cfg-branch-use-reexport-merging`).
#[test]
pub(super) fn trait_impl_anchor_reacts_when_the_forbidden_alias_is_declared_first() {
    let out = locality_findings(
        "anchor-cfg-forbidden-first",
        &[
            ("lib.rs", "pub mod command;\npub mod other;\npub mod domain;\n"),
            ("command.rs", "pub trait Command {}\n"),
            ("other.rs", "pub trait Other {}\n"),
            (
                "domain.rs",
                "#[cfg(unix)]\nuse crate::command::Command as T;\n#[cfg(not(unix))]\nuse crate::other::Other as T;\npub struct Foo;\nimpl T for Foo {}\n",
            ),
        ],
        "crate::command::Command",
        &["crate::commands"],
    )
    .unwrap();
    assert_eq!(
        out,
        ["crate::domain (impl crate::command::Command for crate::domain::Foo)"]
    );
}

/// The identical shape with the anchored trait's alias declared SECOND. Before the fix this
/// silently passed (`Ok([])`).
#[test]
pub(super) fn trait_impl_anchor_reacts_when_the_forbidden_alias_is_declared_second() {
    let out = locality_findings(
        "anchor-cfg-forbidden-second",
        &[
            ("lib.rs", "pub mod command;\npub mod other;\npub mod domain;\n"),
            ("command.rs", "pub trait Command {}\n"),
            ("other.rs", "pub trait Other {}\n"),
            (
                "domain.rs",
                "#[cfg(not(unix))]\nuse crate::other::Other as T;\n#[cfg(unix)]\nuse crate::command::Command as T;\npub struct Foo;\nimpl T for Foo {}\n",
            ),
        ],
        "crate::command::Command",
        &["crate::commands"],
    )
    .unwrap();
    assert_eq!(
        out,
        ["crate::domain (impl crate::command::Command for crate::domain::Foo)"]
    );
}

#[test]
pub(super) fn an_impl_outside_the_allowed_location_is_a_finding() {
    let out = locality_findings(
        "outside",
        &[
            ("lib.rs", "pub mod command;\npub mod domain;\n"),
            ("command.rs", "pub trait Command {}\n"),
            (
                "domain.rs",
                "use crate::command::Command;\npub struct Foo;\nimpl Command for Foo {}\n",
            ),
        ],
        "crate::command::Command",
        &["crate::commands"],
    )
    .unwrap();
    assert_eq!(
        out,
        ["crate::domain (impl crate::command::Command for crate::domain::Foo)"]
    );
}

#[test]
pub(super) fn two_misplaced_impls_do_not_dedup_collapse_when_a_blanket_impls_param_shadows_an_alias()
 {
    // Round-10 finding: `canonical_self_owner` never received round 9's impl_type_params shadow at
    // all -- unlike resolve_self_type (containment.rs), it unconditionally resolved any bare self
    // type via resolve_path. This is not merely a cosmetic label: the `owner` it renders is part of
    // `SemanticFact::MisplacedImpl`'s finding IDENTITY, deduplicated by exact equality. A module
    // declaring `use Foo as T;` alongside BOTH a blanket `impl<T> Command for T {}` (T is the
    // impl's own generic parameter) AND a genuine direct `impl Command for Foo {}` had the blanket
    // impl's bare `T` incorrectly resolve through the alias to the SAME canonical owner string as
    // the direct impl's own (correctly resolved) owner -- two textually and semantically distinct
    // misplaced-impl violations collapsed into one reported finding, a real false negative (one
    // genuine violation silently vanishing), not just a wrong display string. Fixed by giving
    // `canonical_self_owner` the same `impl_type_params` shadow `resolve_self_type` already has.
    let out = locality_findings(
        "owner-collapse-blanket-and-direct",
        &[
            ("lib.rs", "pub mod command;\npub mod domain;\n"),
            ("command.rs", "pub trait Command {}\n"),
            (
                "domain.rs",
                "use crate::command::Command;\nuse crate::domain::sub::Foo as T;\npub mod sub { pub struct Foo; }\nimpl<T> Command for T {}\nimpl Command for crate::domain::sub::Foo {}\n",
            ),
        ],
        "crate::command::Command",
        &["crate::commands"],
    )
    .unwrap();
    assert_eq!(
        out.len(),
        2,
        "both the blanket impl (its own param T) and the direct impl on Foo are genuinely distinct \
         misplaced-impl violations and must not dedup-collapse into one: {out:?}"
    );
}

#[test]
pub(super) fn a_cfg_dual_declared_module_backed_by_one_file_does_not_duplicate_its_impl_finding() {
    // Round-6 finding: resolve_child_modules (scan.rs, backing the whole-crate scan) had no
    // canonical-file dedup for two mutually-exclusive #[cfg] arms plainly declaring the IDENTICAL
    // name resolving to the ONE real file -- unlike module_resolve.rs's descend(), which gained
    // exactly this dedup in 0.2.2. A renderable const-generic owner keeps this test focused on the
    // cfg/file de-duplication contract; unrenderable identity is covered separately by a fail-loud
    // reaction. Verified against real rustc: both `cargo check --features u`
    // and `--features w` compile cleanly with exactly one `impl Command for Arr<2>`.
    let out = locality_findings(
        "cfg-dual-same-file",
        &[
            (
                "lib.rs",
                "pub trait Command {}\npub struct Arr<const N: usize>;\n\
                 #[cfg(feature = \"u\")]\npub mod foo;\n#[cfg(feature = \"w\")]\npub mod foo;\n",
            ),
            ("foo.rs", "impl crate::Command for crate::Arr<2> {}\n"),
        ],
        "crate::Command",
        &["crate::allowed_elsewhere"],
    )
    .unwrap();
    assert_eq!(
        out.len(),
        1,
        "one real impl, backed by one real file under either #[cfg] arm, must be one finding: {out:?}"
    );
}

#[test]
pub(super) fn an_impl_inside_the_allowed_location_is_clean() {
    let out = locality_findings(
        "inside",
        &[
            ("lib.rs", "pub mod command;\npub mod commands;\n"),
            ("command.rs", "pub trait Command {}\n"),
            (
                "commands.rs",
                "use crate::command::Command;\npub struct Foo;\nimpl Command for Foo {}\n",
            ),
        ],
        "crate::command::Command",
        &["crate::commands"],
    )
    .unwrap();
    assert!(
        out.is_empty(),
        "an impl in the allowed location is clean: {out:?}"
    );
}

#[test]
pub(super) fn a_nested_module_beneath_the_allowed_prefix_is_clean() {
    let out = locality_findings(
        "nested-allowed",
        &[
            ("lib.rs", "pub mod command;\npub mod commands;\n"),
            ("command.rs", "pub trait Command {}\n"),
            ("commands.rs", "pub mod greet;\n"),
            (
                "commands/greet.rs",
                "use crate::command::Command;\npub struct Foo;\nimpl Command for Foo {}\n",
            ),
        ],
        "crate::command::Command",
        &["crate::commands"],
    )
    .unwrap();
    assert!(
        out.is_empty(),
        "beneath an allowed prefix is clean: {out:?}"
    );
}

#[test]
pub(super) fn a_prefix_colliding_sibling_location_is_not_allowed() {
    let out = locality_findings(
        "sibling",
        &[
            ("lib.rs", "pub mod command;\npub mod commandeer;\n"),
            ("command.rs", "pub trait Command {}\n"),
            (
                "commandeer.rs",
                "use crate::command::Command;\npub struct Foo;\nimpl Command for Foo {}\n",
            ),
        ],
        "crate::command::Command",
        &["crate::commands"],
    )
    .unwrap();
    assert_eq!(
        out,
        ["crate::commandeer (impl crate::command::Command for crate::commandeer::Foo)"],
        "a sibling of the allowed prefix is not allowed"
    );
}

#[test]
pub(super) fn an_impl_in_any_of_several_allowed_locations_is_clean() {
    let out = locality_findings(
        "multi-allowed",
        &[
            ("lib.rs", "pub mod command;\npub mod builtins;\n"),
            ("command.rs", "pub trait Command {}\n"),
            (
                "builtins.rs",
                "use crate::command::Command;\npub struct Foo;\nimpl Command for Foo {}\n",
            ),
        ],
        "crate::command::Command",
        &["crate::commands", "crate::builtins"],
    )
    .unwrap();
    assert!(out.is_empty(), "any one allowed location suffices: {out:?}");
}

#[test]
pub(super) fn a_bare_same_module_trait_name_reacts() {
    // B1: the impl is in the trait's own (disallowed) module, with a bare `Command`
    // and no `use`. Resolving the bare name against the current module is required —
    // leaving it unresolved would silently pass a real misplaced impl.
    let out = locality_findings(
        "bare-same-module",
        &[
            ("lib.rs", "pub mod command;\n"),
            (
                "command.rs",
                "pub trait Command {}\npub struct Foo;\nimpl Command for Foo {}\n",
            ),
        ],
        "crate::command::Command",
        &["crate::commands"],
    )
    .unwrap();
    assert_eq!(
        out,
        ["crate::command (impl crate::command::Command for crate::command::Foo)"]
    );
}

#[test]
pub(super) fn a_renamed_trait_import_reacts() {
    let out = locality_findings(
        "renamed-trait",
        &[
            ("lib.rs", "pub mod command;\npub mod domain;\n"),
            ("command.rs", "pub trait Command {}\n"),
            (
                "domain.rs",
                "use crate::command::Command as Cmd;\npub struct Foo;\nimpl Cmd for Foo {}\n",
            ),
        ],
        "crate::command::Command",
        &["crate::commands"],
    )
    .unwrap();
    assert_eq!(
        out,
        ["crate::domain (impl crate::command::Command for crate::domain::Foo)"]
    );
}

#[test]
pub(super) fn a_super_relative_trait_import_reacts() {
    // The relative-use false negative: `use super::command::Command` populates the
    // use-map with the relative string; resolution must canonicalize it against the
    // module before matching the anchor, or a real misplaced impl silently passes.
    let out = locality_findings(
        "super-trait",
        &[
            ("lib.rs", "pub mod command;\npub mod domain;\n"),
            ("command.rs", "pub trait Command {}\n"),
            (
                "domain.rs",
                "use super::command::Command;\npub struct Foo;\nimpl Command for Foo {}\n",
            ),
        ],
        "crate::command::Command",
        &["crate::commands"],
    )
    .unwrap();
    assert_eq!(
        out,
        ["crate::domain (impl crate::command::Command for crate::domain::Foo)"]
    );
}

#[test]
pub(super) fn a_cfg_gated_module_with_no_file_is_skipped_not_errored() {
    // A `#[cfg(feature = "x")] mod optional;` with no `optional.rs` (the feature is off)
    // is legal Rust; the whole-crate walk must skip it, never fail the gate (exit 2).
    let out = locality_findings(
        "cfg-absent-mod",
        &[
            (
                "lib.rs",
                "pub mod command;\n#[cfg(feature = \"never\")]\npub mod optional;\n",
            ),
            ("command.rs", "pub trait Command {}\n"),
        ],
        "crate::command::Command",
        &["crate::commands"],
    )
    .unwrap();
    assert!(
        out.is_empty(),
        "a cfg-gated absent module is skipped: {out:?}"
    );
}

#[test]
pub(super) fn a_reexported_trait_path_reacts() {
    // S1: the impl reaches the trait through a facade re-export; resolution must
    // follow the pub use chain to match the anchor.
    let out = locality_findings(
        "reexport-impl",
        &[
            (
                "lib.rs",
                "pub mod command;\npub mod facade;\npub mod domain;\n",
            ),
            ("command.rs", "pub trait Command {}\n"),
            ("facade.rs", "pub use crate::command::Command;\n"),
            (
                "domain.rs",
                "use crate::facade::Command;\npub struct Foo;\nimpl Command for Foo {}\n",
            ),
        ],
        "crate::command::Command",
        &["crate::commands"],
    )
    .unwrap();
    assert_eq!(
        out,
        ["crate::domain (impl crate::command::Command for crate::domain::Foo)"]
    );
}

#[test]
pub(super) fn an_anchor_named_at_a_reexport_path_resolves_not_a_constitution_error() {
    // B2: the boundary names the trait at its facade path; this must resolve to the
    // real local trait (not a false exit-2) and still react to misplaced impls.
    let out = locality_findings(
        "reexport-anchor",
        &[
            (
                "lib.rs",
                "pub mod command;\npub mod facade;\npub mod domain;\n",
            ),
            ("command.rs", "pub trait Command {}\n"),
            ("facade.rs", "pub use crate::command::Command;\n"),
            (
                "domain.rs",
                "use crate::command::Command;\npub struct Foo;\nimpl Command for Foo {}\n",
            ),
        ],
        "crate::facade::Command",
        &["crate::commands"],
    )
    .unwrap();
    assert_eq!(
        out,
        ["crate::domain (impl crate::command::Command for crate::domain::Foo)"]
    );
}

#[test]
pub(super) fn an_unresolvable_trait_anchor_is_a_constitution_error() {
    let err = locality_findings(
        "ghost-trait",
        &[
            ("lib.rs", "pub mod command;\n"),
            ("command.rs", "pub trait Command {}\n"),
        ],
        "crate::command::Ghost",
        &["crate::commands"],
    )
    .unwrap_err();
    assert_eq!(err, unknown_trait_error("crate::command::Ghost", "x"));
}

/// An `allowed_locations`/`only_implemented_in` entry with an empty `::`-segment (leading,
/// trailing, or doubled `::`) must be a constitution error — never silently pass through into
/// `matches_allowed`. Before the fix, `matches_allowed`'s plain `path_within` never matched a
/// malformed entry against any real module location, so a legitimately-placed impl was reported
/// as a spurious violation instead of naming the actual typo in the declaration (reproduced
/// directly against this pure heart: `["::crate::api"]` on an impl genuinely under `crate::api`
/// produced a finding, not a clean error, before this test was written to pin the fix).
#[test]
pub(super) fn trait_impl_rejects_a_malformed_colon_allowed_location() {
    let files: &[(&str, &str)] = &[
        ("lib.rs", "pub mod api;\n"),
        (
            "api.rs",
            "pub trait Command {}\npub struct Foo;\nimpl Command for Foo {}\n",
        ),
    ];
    for bad in ["::crate::api", "crate::api::", "crate::api::::sub"] {
        let err = locality_findings("malformed-allowed", files, "crate::api::Command", &[bad])
            .unwrap_err();
        assert!(
            err.contains(bad),
            "constitution error must name the malformed allowed entry {bad:?}: {err}"
        );
    }
    // The empty string itself is also a malformed allowed entry — see must_not_expose's
    // identical note; this shares the same `validate_path_operands` guard.
    let empty_err = locality_findings(
        "malformed-allowed-empty",
        files,
        "crate::api::Command",
        &[""],
    )
    .unwrap_err();
    assert!(
        empty_err.contains("is empty"),
        "constitution error must flag the empty allowed entry: {empty_err}"
    );
    // Control: the well-formed spelling for the identical, genuinely-in-place impl still passes
    // clean — the rejection above is a spelling gate, never a locality regression.
    let clean = locality_findings(
        "malformed-allowed-control",
        files,
        "crate::api::Command",
        &["crate::api"],
    )
    .unwrap();
    assert!(
        clean.is_empty(),
        "a well-formed allowed entry must still admit the in-place impl: {clean:?}"
    );
}

#[test]
pub(super) fn a_non_anchored_traits_impl_is_ignored() {
    let out = locality_findings(
        "other-trait",
        &[
            ("lib.rs", "pub mod command;\npub mod domain;\n"),
            ("command.rs", "pub trait Command {}\npub trait Other {}\n"),
            (
                "domain.rs",
                "use crate::command::Other;\npub struct Foo;\nimpl Other for Foo {}\n",
            ),
        ],
        "crate::command::Command",
        &["crate::commands"],
    )
    .unwrap();
    assert!(out.is_empty(), "only the anchored trait reacts: {out:?}");
}

#[test]
pub(super) fn an_inline_module_impl_is_located() {
    let out = locality_findings(
        "inline-impl",
        &[
            (
                "lib.rs",
                "pub mod command;\npub mod domain { use crate::command::Command; pub struct Foo; impl Command for Foo {} }\n",
            ),
            ("command.rs", "pub trait Command {}\n"),
        ],
        "crate::command::Command",
        &["crate::commands"],
    )
    .unwrap();
    assert_eq!(
        out,
        ["crate::domain (impl crate::command::Command for crate::domain::Foo)"]
    );
}

#[test]
pub(super) fn a_glob_imported_trait_is_a_documented_bound() {
    let out = locality_findings(
        "glob-trait",
        &[
            ("lib.rs", "pub mod command;\npub mod domain;\n"),
            ("command.rs", "pub trait Command {}\n"),
            (
                "domain.rs",
                "use crate::command::*;\npub struct Foo;\nimpl Command for Foo {}\n",
            ),
        ],
        "crate::command::Command",
        &["crate::commands"],
    )
    .unwrap();
    assert!(
        out.is_empty(),
        "a glob-imported trait is out of scope, not silently matched: {out:?}"
    );
}

#[test]
pub(super) fn an_unconditional_path_remapped_module_is_followed_and_its_impl_reacts() {
    // An unconditional `#[path = "weird.rs"] mod domain;` is now *followed* to weird.rs: a
    // disallowed impl there reacts, attributed to the module `crate::domain` (its declared path,
    // regardless of the file it lives in). Previously the module was skipped — a false negative
    // (a disallowed impl in a relocated module passing unobserved).
    let out = locality_findings(
        "path-remapped",
        &[
            (
                "lib.rs",
                "pub mod command;\n#[path = \"weird.rs\"]\npub mod domain;\n",
            ),
            ("command.rs", "pub trait Command {}\n"),
            (
                "weird.rs",
                "use crate::command::Command;\npub struct Foo;\nimpl Command for Foo {}\n",
            ),
        ],
        "crate::command::Command",
        &["crate::commands"],
    )
    .unwrap();
    assert_eq!(
        out,
        ["crate::domain (impl crate::command::Command for crate::domain::Foo)"],
        "the impl in the #[path]-relocated module is followed and reacts: {out:?}"
    );
}

#[test]
pub(super) fn a_cfg_attr_remapped_module_target_is_followed_when_the_conventional_file_is_absent() {
    // `#[cfg_attr(<pred>, path = "…")]` never removes the `mod` item the way a bare `#[cfg]`
    // does, so SOME file must back it on every build; with no conventional `domain.rs` present,
    // the `cfg_attr` target is the only candidate and is followed — closing the false negative
    // where this module (and any impl inside it) silently vanished from the crate-wide scan.
    let out = locality_findings(
        "cfg-attr-remapped",
        &[
            (
                "lib.rs",
                "pub mod command;\n#[cfg_attr(windows, path = \"weird.rs\")]\npub mod domain;\n",
            ),
            ("command.rs", "pub trait Command {}\n"),
            (
                "weird.rs",
                "use crate::command::Command;\npub struct Foo;\nimpl Command for Foo {}\n",
            ),
        ],
        "crate::command::Command",
        &["crate::commands"],
    )
    .unwrap();
    assert_eq!(
        out,
        ["crate::domain (impl crate::command::Command for crate::domain::Foo)"],
        "the impl in the cfg_attr-remapped module is followed and reacts: {out:?}"
    );

    // A NESTED cfg_attr remap's target is followed the identical way when the conventional file
    // is absent.
    let nested = locality_findings(
        "cfg-attr-nested-remapped",
        &[
            (
                "lib.rs",
                "pub mod command;\n#[cfg_attr(a, cfg_attr(b, path = \"weird.rs\"))]\npub mod domain;\n",
            ),
            ("command.rs", "pub trait Command {}\n"),
            (
                "weird.rs",
                "use crate::command::Command;\npub struct Foo;\nimpl Command for Foo {}\n",
            ),
        ],
        "crate::command::Command",
        &["crate::commands"],
    )
    .unwrap();
    assert_eq!(
        nested,
        ["crate::domain (impl crate::command::Command for crate::domain::Foo)"],
        "the impl in the nested-cfg_attr-remapped module is followed and reacts: {nested:?}"
    );
}

#[test]
pub(super) fn a_cfg_attr_without_a_path_meta_is_scanned_normally() {
    // The inverse false negative: a cfg_attr carrying NO `path` meta is a normal file module and
    // must be scanned, or its violations would silently vanish.
    let out = locality_findings(
        "cfg-attr-no-path",
        &[
            (
                "lib.rs",
                "pub mod command;\n#[cfg_attr(test, allow(dead_code))]\npub mod domain;\n",
            ),
            ("command.rs", "pub trait Command {}\n"),
            (
                "domain.rs",
                "use crate::command::Command;\npub struct Foo;\nimpl Command for Foo {}\n",
            ),
        ],
        "crate::command::Command",
        &["crate::commands"],
    )
    .unwrap();
    assert!(
        !out.is_empty(),
        "a cfg_attr without a path meta is a normal module and must be scanned: {out:?}"
    );

    // Twin alignment: only a `path = "…"` NAME-VALUE is a remap. A bare `path` meta (not a valid
    // `#[path]`) is NOT a remap — so the module is scanned, matching guibiao's byte scanner (which
    // requires `path =`). Previously hunyi over-matched any `path`-named meta.
    let bare = locality_findings(
        "cfg-attr-bare-path",
        &[
            (
                "lib.rs",
                "pub mod command;\n#[cfg_attr(test, path)]\npub mod domain;\n",
            ),
            ("command.rs", "pub trait Command {}\n"),
            (
                "domain.rs",
                "use crate::command::Command;\npub struct Foo;\nimpl Command for Foo {}\n",
            ),
        ],
        "crate::command::Command",
        &["crate::commands"],
    )
    .unwrap();
    assert!(
        !bare.is_empty(),
        "a bare `path` meta (not `path = \"…\"`) is not a remap; the module is scanned: {bare:?}"
    );
}

#[test]
pub(super) fn two_impls_in_one_module_are_distinct_findings_by_self_type() {
    let out = locality_findings(
        "distinct-self",
        &[
            ("lib.rs", "pub mod command;\npub mod domain;\n"),
            ("command.rs", "pub trait Command {}\n"),
            (
                "domain.rs",
                "use crate::command::Command;\npub struct A;\npub struct B;\nimpl Command for A {}\nimpl Command for B {}\n",
            ),
        ],
        "crate::command::Command",
        &["crate::commands"],
    )
    .unwrap();
    assert_eq!(
        out,
        [
            "crate::domain (impl crate::command::Command for crate::domain::A)",
            "crate::domain (impl crate::command::Command for crate::domain::B)"
        ]
    );
}

/// Declaring one trait through a facade `pub use` and through its defining path must produce ONE
/// identity, because it is one governed thing.
///
/// Matching already resolved both sides — the anchor is expanded through the crate's own re-export
/// closure before any impl is compared — but identity kept the raw declared string, so renaming a
/// constitution declaration between two equivalent spellings (a pure refactor, no code change)
/// silently invalidated every affected baseline entry: each accepted violation re-fired as new while
/// its recorded entry reported stale. Both the violation `target` and the rule key now come from the
/// resolved anchor, so the two spellings converge.
#[test]
pub(super) fn two_equivalent_trait_spellings_resolve_to_one_anchor_identity() {
    let files: &[(&str, &str)] = &[
        (
            "lib.rs",
            "pub mod command;\npub mod api;\npub mod domain;\n",
        ),
        ("command.rs", "pub trait Command {}\n"),
        // The facade: `crate::api::Command` re-exports `crate::command::Command`.
        ("api.rs", "pub use crate::command::Command;\n"),
        (
            "domain.rs",
            "pub struct Foo;\nimpl crate::command::Command for Foo {}\n",
        ),
    ];
    // Asserted on the real `ViolationId` the reaction publishes, through `check_trait_impl_boundary`
    // — NOT on the resolved anchor alone. An earlier version of this test compared anchors and passed
    // even with the identity wiring reverted to the declared spelling: the anchor was always computed
    // correctly; what was broken was whether identity used it. A guard bound to the wrong surface
    // reports the surrounding contract, not the change (AGENTS.md).
    let (metadata, _tree) = fixture_metadata("spelling-identity", files);
    let ids_for = |declared: &str| {
        let boundary = TraitImplBoundary::in_crate("x")
            .trait_("crate::command::Command")
            .only_implemented_in("crate::command")
            .because("locality");
        // Re-declare with the spelling under test, keeping every other field identical.
        let boundary = TraitImplBoundary {
            trait_path: declared.to_string(),
            ..boundary
        };
        let mut violations = Vec::new();
        check_trait_impl_boundary(&metadata, &boundary, &mut violations).unwrap();
        violations.iter().map(|v| v.id()).collect::<Vec<_>>()
    };
    let via_facade = ids_for("crate::api::Command");
    let via_defining = ids_for("crate::command::Command");
    assert_eq!(via_facade.len(), 1, "the fixture must really violate");
    assert_eq!(
        via_facade, via_defining,
        "two equivalent spellings of one trait must publish one violation identity, or a pure \
         declaration refactor invalidates every baseline entry for the same real fact"
    );
    assert_eq!(
        locality_anchor("facade", files, "crate::api::Command").unwrap(),
        "crate::command::Command",
        "and the anchor itself is the defining path, not the facade as declared"
    );
}

/// The declaration is what is ambiguous when its facade reaches two different traits, so the reaction
/// is a constitution error naming both — not an arbitrary pick that makes identity a coin flip.
#[test]
pub(super) fn a_cfg_collided_trait_facade_is_a_constitution_error() {
    let error = locality_anchor(
        "collided-facade",
        &[
            (
                "lib.rs",
                "pub mod a;\npub mod b;\npub mod api;\npub mod domain;\n",
            ),
            ("a.rs", "pub trait Command {}\n"),
            ("b.rs", "pub trait Command {}\n"),
            (
                "api.rs",
                "#[cfg(unix)]\npub use crate::a::Command;\n#[cfg(not(unix))]\npub use crate::b::Command;\n",
            ),
            ("domain.rs", "pub struct Foo;\n"),
        ],
        "crate::api::Command",
    )
    .unwrap_err();
    for expected in [
        "must anchor to exactly one trait",
        "crate::a::Command",
        "crate::b::Command",
        "Declare the defining path instead of the facade",
    ] {
        assert!(
            error.contains(expected),
            "the error must name the ambiguity and both candidates, missing `{expected}`: {error}"
        );
    }
}

/// A cfg-collided self-type alias cannot be named injectively, so observation refuses instead of
/// collapsing two independent sites onto one owner label.
///
/// `#[cfg(unix)] use crate::a::Foo as X; #[cfg(not(unix))] use crate::b::Bar as X;` with an `impl
/// Marker for X` in each branch is two genuinely different types acquiring the governed trait.
/// Resolution is cfg-blind by design, so both bindings are live candidates and neither may be
/// preferred; the owner renderer used to take whichever came first, giving both sites the SAME owner.
/// Owner is a dedup key, so they collapsed to one violation and a baseline accepting the first
/// suppressed the second — the false negative the Core Contract forbids outright.
///
/// The candidate SET is identical for both sites, so it cannot separate them either: what is left is
/// to refuse to name it. Exit 2 "cannot judge" over a silent collapse is the Core Contract's own
/// ordering, and the same reaction an unrenderable self type already gets.
#[test]
pub(super) fn a_cfg_collided_self_type_alias_fails_loud_instead_of_collapsing_two_owners() {
    let error = locality_findings(
        "cfg-collided-owner-alias",
        &[
            (
                "lib.rs",
                "pub mod marker;\npub mod a;\npub mod b;\npub mod domain;\n",
            ),
            ("marker.rs", "pub trait Marker {}\n"),
            ("a.rs", "pub struct Foo;\n"),
            ("b.rs", "pub struct Bar;\n"),
            (
                "domain.rs",
                "#[cfg(unix)]\nuse crate::a::Foo as X;\n#[cfg(not(unix))]\nuse crate::b::Bar as X;\n\
                 impl crate::marker::Marker for X {}\n",
            ),
        ],
        "crate::marker::Marker",
        &["crate::marker"],
    )
    .unwrap_err();
    assert!(
        error.contains("without a stable structural label"),
        "the ambiguity must reach the shared fail-loud identity gate: {error}"
    );
    assert!(
        error.contains("mutually-exclusive"),
        "the diagnostic must name the cfg collision as the cause, not just the failure: {error}"
    );
    assert!(
        !error.contains("_#"),
        "the internal sentinel must never surface to an adopter: {error}"
    );
}

/// The control for the guard above: ONE alias binding, the ordinary case, still resolves and reacts
/// normally. Without this, "refuse when ambiguous" could quietly become "refuse whenever an alias is
/// involved" and the suite would not notice.
#[test]
pub(super) fn a_single_aliased_self_type_still_resolves_to_its_owner() {
    let out = locality_findings(
        "single-alias-owner",
        &[
            ("lib.rs", "pub mod marker;\npub mod a;\npub mod domain;\n"),
            ("marker.rs", "pub trait Marker {}\n"),
            ("a.rs", "pub struct Foo;\n"),
            (
                "domain.rs",
                "use crate::a::Foo as X;\nimpl crate::marker::Marker for X {}\n",
            ),
        ],
        "crate::marker::Marker",
        &["crate::marker"],
    )
    .unwrap();
    assert_eq!(
        out,
        ["crate::domain (impl crate::marker::Marker for crate::a::Foo)"],
        "a single alias candidate must still render its resolved owner"
    );
}

#[test]
pub(super) fn const_generic_expr_self_types_fail_loud_without_positional_identity() {
    // The ordinary owner renderer cannot distinguish these complex const expressions. Publishing
    // scan position would make identity drift under reorder/insertion, so observation must fail.
    let error = findings(
        "const-generic-expr",
        &[
            ("lib.rs", "pub mod domain;\n"),
            (
                "domain.rs",
                "pub struct Arr<const N: usize>(u8);\n\
                 impl Arr<{ 1 + 1 }> { pub fn a(&self) -> crate::infra::T { todo!() } }\n\
                 impl Arr<{ 2 + 2 }> { pub fn a(&self) -> crate::infra::T { todo!() } }\n",
            ),
        ],
        "crate::domain",
        &["crate::infra"],
    )
    .unwrap_err();
    assert!(error.contains("stable structural label"), "{error}");
    assert!(!error.contains("_#"), "{error}");
}

#[test]
pub(super) fn owner_is_canonical_across_written_forms() {
    // The same self type written two ways — a bare `impl Foo` and a fully-qualified
    // `impl crate::m::Foo` — must render to the IDENTICAL canonical owner
    // `crate::m::Foo`, so the token form never over-splits a single type into two owners.
    let out = findings(
        "canonical-forms",
        &[
            ("lib.rs", "pub mod m;\n"),
            (
                "m.rs",
                "pub struct Foo;\n\
                 impl Foo { pub fn a(&self) -> crate::infra::T { todo!() } }\n\
                 impl crate::m::Foo { pub fn b(&self) -> crate::infra::T { todo!() } }\n",
            ),
        ],
        "crate::m",
        &["crate::infra"],
    )
    .unwrap();
    assert_eq!(
        out,
        [
            "crate::infra::T exposed by fn <crate::m::Foo>::a",
            "crate::infra::T exposed by fn <crate::m::Foo>::b",
        ],
        "both written forms of the same self type render the identical canonical owner",
    );
}

#[test]
pub(super) fn a_cfg_gated_impl_is_observed_as_written() {
    // `#[cfg]` is not evaluated: syn parses every branch, so a misplaced impl behind a
    // disabled feature is still observed (a deliberate, documented over-approximation).
    let out = locality_findings(
        "cfg-gated",
        &[
            ("lib.rs", "pub mod command;\npub mod domain;\n"),
            ("command.rs", "pub trait Command {}\n"),
            (
                "domain.rs",
                "use crate::command::Command;\npub struct Foo;\n#[cfg(feature = \"never\")]\nimpl Command for Foo {}\n",
            ),
        ],
        "crate::command::Command",
        &["crate::commands"],
    )
    .unwrap();
    assert_eq!(
        out,
        ["crate::domain (impl crate::command::Command for crate::domain::Foo)"]
    );
}

#[test]
pub(super) fn a_macro_generated_impl_is_a_documented_bound() {
    // A `make_impl!(…)` invocation is an `Item::Macro`, not an `Item::Impl` — syn does
    // not expand it, so the impl it would generate is out of scope, not silently matched.
    let out = locality_findings(
        "macro-impl",
        &[
            ("lib.rs", "pub mod command;\npub mod domain;\n"),
            ("command.rs", "pub trait Command {}\n"),
            ("domain.rs", "make_impl!(Foo);\n"),
        ],
        "crate::command::Command",
        &["crate::commands"],
    )
    .unwrap();
    assert!(
        out.is_empty(),
        "a macro-generated impl is out of scope, not silently matched: {out:?}"
    );
}

#[test]
pub(super) fn the_builder_carries_severity() {
    // Severity (and thus baseline/exit-code parity via the shared 璇璣 model) is plumbed
    // from the builder into each Violation by `check_trait_impl_boundary`.
    let warn = TraitImplBoundary::in_crate("app")
        .trait_("crate::command::Command")
        .only_implemented_in("crate::commands")
        .warn()
        .because("advisory first");
    assert_eq!(warn.severity(), Severity::Warn);

    let enforce = TraitImplBoundary::in_crate("app")
        .trait_("crate::command::Command")
        .only_implemented_in("crate::commands")
        .because("enforced");
    assert_eq!(enforce.severity(), Severity::Enforce);
}

#[test]
pub(super) fn every_hunyi_rule_family_has_exact_semantic_identity() {
    fn assert_rule(rule: xuanji::RuleKey, expected_type: &str, expected_fields: &[(&str, &str)]) {
        assert_eq!(rule.rule_type(), expected_type);
        assert_eq!(rule.fields().collect::<Vec<_>>(), expected_fields);
    }

    let signature = SignatureBoundary::in_crate("x")
        .module("crate::api")
        .must_not_expose("r#crate::infra")
        .and_not_expose("crate::storage")
        .including_trait_impls()
        .because("presentation only");
    assert_rule(
        signature.rule_key(),
        "tianheng.rule/hunyi/signature-exposure",
        &[
            ("forbidden", "[\"crate::infra\",\"crate::storage\"]"),
            ("including_trait_impls", "true"),
        ],
    );

    let dyn_trait = DynTraitBoundary::in_crate("x")
        .module("crate::api")
        .must_not_expose_dyn_of(["crate::Port", "crate::Other"])
        .because("r");
    assert_rule(
        dyn_trait.rule_key(),
        "tianheng.rule/hunyi/dyn-trait-exposure",
        &[("forbidden_operands", "[\"crate::Other\",\"crate::Port\"]")],
    );

    let impl_trait = ImplTraitBoundary::in_crate("x")
        .module("crate::api")
        .must_not_expose_impl_trait_of(["crate::Port"])
        .including_submodules()
        .because("r");
    assert_rule(
        impl_trait.rule_key(),
        "tianheng.rule/hunyi/impl-trait-exposure",
        &[("forbidden_operands", "[\"crate::Port\"]")],
    );

    let locality = TraitImplBoundary::in_crate("x")
        .trait_("r#crate::Port")
        .only_implemented_in("crate::adapter")
        .and_in("crate::infra")
        .because("r");
    assert_rule(
        locality.rule_key(),
        "tianheng.rule/hunyi/trait-impl-locality",
        &[
            ("allowed_locations", "[\"crate::adapter\",\"crate::infra\"]"),
            ("trait", "crate::Port"),
        ],
    );

    let marker = ForbiddenMarkerBoundary::in_crate("x")
        .module("crate::domain")
        .must_not_acquire("serde::Serialize")
        .and_not_acquire("serde::Deserialize")
        .because("r");
    assert_rule(
        marker.rule_key(),
        "tianheng.rule/hunyi/forbidden-marker",
        &[("forbidden", "[\"serde::Deserialize\",\"serde::Serialize\"]")],
    );

    let visibility = VisibilityBoundary::in_crate("x")
        .module("crate::internal")
        .max_visibility(VisibilityCeiling::Super)
        .because("r");
    assert_rule(
        visibility.rule_key(),
        "tianheng.rule/hunyi/visibility-ceiling",
        &[("ceiling", "super")],
    );

    let async_exposure = AsyncExposureBoundary::in_crate("x")
        .module("crate::core")
        .must_not_expose_async_fn()
        .including_submodules()
        .because("r");
    assert_rule(
        async_exposure.rule_key(),
        "tianheng.rule/hunyi/async-exposure",
        &[],
    );

    let unsafe_confinement = UnsafeBoundary::in_crate("x")
        .only_under(["crate::ffi", "crate::platform"])
        .because("r");
    assert_rule(
        unsafe_confinement.rule_key(),
        "tianheng.rule/hunyi/unsafe-confinement",
        &[("allowed", "[\"crate::ffi\",\"crate::platform\"]")],
    );
}

#[test]
pub(super) fn hunyi_rule_identity_is_set_order_stable_and_parameter_sensitive() {
    let left = SignatureBoundary::in_crate("x")
        .module("crate::api")
        .must_not_expose("crate::infra")
        .and_not_expose("crate::storage")
        .because("first wording");
    let reordered = SignatureBoundary::in_crate("other")
        .module("crate::elsewhere")
        .must_not_expose("crate::storage")
        .and_not_expose("crate::infra")
        .and_not_expose("crate::infra")
        .warn()
        .because("different wording")
        .with_anchor("GOV-1");
    let expanded = SignatureBoundary::in_crate("x")
        .module("crate::api")
        .must_not_expose("crate::infra")
        .and_not_expose("crate::storage")
        .and_not_expose("crate::transport")
        .because("first wording");
    let deeper = SignatureBoundary::in_crate("x")
        .module("crate::api")
        .must_not_expose("crate::infra")
        .and_not_expose("crate::storage")
        .including_trait_impls()
        .because("first wording");

    assert_eq!(left.rule_key(), reordered.rule_key());
    assert_ne!(left.rule_key(), expanded.rule_key());
    assert_ne!(left.rule_key(), deeper.rule_key());
}
