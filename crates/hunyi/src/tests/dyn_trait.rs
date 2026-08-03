use super::super::*;
use super::helpers::*;
// --- dyn-trait-boundary ---------------------------------------------------

/// Like [`findings`] but for the dyn-trait capability: write `files`, return the rendered
/// `dyn` shapes exposed by `module`. Shape-only, so it takes no forbidden set.
pub(super) fn dyn_findings(
    name: &str,
    files: &[(&str, &str)],
    module: &str,
) -> Result<Vec<String>, String> {
    shape_findings("dyn", name, files, module, dyn_module_findings)
}

pub(super) fn dyn_mod(name: &str, body: &str) -> Result<Vec<String>, String> {
    dyn_findings(
        name,
        &[("lib.rs", "pub mod m;\n"), ("m.rs", body)],
        "crate::m",
    )
}

/// Like [`dyn_findings`] but for the operand-scoped rule: write `files`, return the rendered
/// `dyn` shapes whose principal trait resolves into `forbidden`.
pub(super) fn dyn_operand_findings(
    name: &str,
    files: &[(&str, &str)],
    module: &str,
    forbidden: &[&str],
    deps: &[&str],
) -> Result<Vec<String>, String> {
    operand_findings(
        "dyn",
        name,
        files,
        module,
        forbidden,
        deps,
        dyn_operand_module_findings,
    )
}

pub(super) fn dyn_operand_mod(
    name: &str,
    body: &str,
    forbidden: &[&str],
) -> Result<Vec<String>, String> {
    dyn_operand_findings(
        name,
        &[("lib.rs", "pub mod m;\n"), ("m.rs", body)],
        "crate::m",
        forbidden,
        &[],
    )
}

#[test]
pub(super) fn a_dyn_in_a_supertrait_or_assoc_type_bound_is_observed() {
    // A `dyn` inside a supertrait's generic argument, or inside a
    // public associated type's `: Bound`, is a real exposed trait-object in the trait's public
    // contract. The sibling signature-coupling collector already walks these bound positions
    // (paths_in_bounds); the dyn collector now matches it — previously it skipped supertraits and
    // associated-type bounds entirely, silently dropping the dyn (a false negative → exit 0).
    assert!(
        dyn_mod(
            "supertrait-dyn",
            "pub trait Facade: AsRef<Box<dyn crate::ports::Port>> {}\n",
        )
        .unwrap()
        .contains(&"dyn crate::ports::Port exposed by trait crate::m::Facade".to_string()),
        "a dyn in a supertrait generic argument must be observed",
    );
    assert!(
        dyn_mod(
            "assoc-bound-dyn",
            "pub trait F { type Bar: AsRef<Box<dyn crate::ports::Port>>; }\n",
        )
        .unwrap()
        .contains(&"dyn crate::ports::Port exposed by type trait crate::m::F::Bar".to_string()),
        "a dyn in an associated-type bound must be observed",
    );
}

#[test]
pub(super) fn a_dyn_in_an_inherent_impl_generic_bound_is_observed() {
    // Round-2 fix: a `dyn` in an inherent impl's own generic-param bound is exposed on the inherent
    // API; the dyn collector's inherent-impl arm now walks the impl generics (parity with the path
    // collector's fix #9 and with the struct/enum/trait arms).
    let out = dyn_mod(
        "dyn-impl-generics",
        "pub struct Foo<T>(T);\nimpl<T: AsRef<Box<dyn crate::ports::Port>>> Foo<T> { pub fn m(&self) {} }\n",
    )
    .unwrap();
    // The seam now names the bounded parameter (`generics: T`), not a bare `(generics)`: two impl
    // blocks bounding different parameters to the same forbidden type are two distinct violations,
    // and rendering them identically made a report unreadable even where identity was correct.
    assert!(
        out.iter()
            .any(|f| f.contains("dyn crate::ports::Port") && f.contains("(generics: T)")),
        "a dyn in an inherent-impl generic bound must be observed and name its bound: {out:?}"
    );
}

#[test]
pub(super) fn dyn_operand_flags_a_named_trait_and_passes_others() {
    // A dyn of the listed trait is flagged; a dyn of an unlisted trait passes.
    assert_eq!(
        dyn_operand_mod(
            "named",
            "pub fn c() -> Box<dyn crate::ports::Port> { todo!() }\n",
            &["crate::ports::Port"],
        )
        .unwrap(),
        ["dyn crate::ports::Port exposed by fn crate::m::c"],
    );
    assert!(
        dyn_operand_mod(
            "other",
            "pub fn e() -> Box<dyn std::error::Error> { todo!() }\n",
            &["crate::ports::Port"],
        )
        .unwrap()
        .is_empty(),
        "a dyn of an unlisted trait passes",
    );
}

#[test]
pub(super) fn dyn_operand_honors_a_module_prefix() {
    // A module-prefix operand forbids any dyn of a trait under it (exact-or-`::` prefix).
    assert_eq!(
        dyn_operand_mod(
            "prefix",
            "pub fn c() -> Box<dyn crate::ports::Port> { todo!() }\n",
            &["crate::ports"],
        )
        .unwrap(),
        ["dyn crate::ports::Port exposed by fn crate::m::c"],
    );
}

#[test]
pub(super) fn dyn_operand_matches_a_reexported_trait_by_its_defining_path() {
    // The trait is defined at crate::ports::Port and re-exported as crate::Port; the module
    // exposes `dyn crate::Port`. Forbidding either path matches — both canonicalize through
    // the re-export closure to the defining path.
    let files = &[
        (
            "lib.rs",
            "pub mod ports;\npub use crate::ports::Port;\npub mod m;\n",
        ),
        ("ports.rs", "pub trait Port {}\n"),
        ("m.rs", "pub fn c() -> Box<dyn crate::Port> { todo!() }\n"),
    ];
    // Forbid by the DEFINING path — the exposed facade `crate::Port` canonicalizes to it.
    assert_eq!(
        dyn_operand_findings(
            "reexport-defining",
            files,
            "crate::m",
            &["crate::ports::Port"],
            &[],
        )
        .unwrap(),
        ["dyn crate::Port exposed by fn crate::m::c"],
        "a dyn written through a re-export facade matches the forbidden defining path",
    );
}

#[test]
pub(super) fn a_cfg_sibling_child_module_does_not_shadow_a_different_branchs_own_extern_principal()
{
    // Round-7 finding: extern_resolution computed externs_type/renames_bare ONCE over the
    // flattened union of every #[cfg] branch's items (feeding operand_module_findings, backing
    // dyn-trait/impl-trait operand-scoped boundaries) -- the identical conflation round 6 fixed
    // for signature-coupling's use-map, left unfixed here too. The "u" branch (platform.rs)
    // declares a LOCAL `mod traits { .. }`; the mutually-exclusive "w" branch (win_platform.rs)
    // has no local `mod traits` at all and its own `dyn traits::Marker` genuinely names the real
    // extern crate `traits`. Before the fix, the "u" branch's local `mod traits` silently
    // suppressed the "w" branch's own genuine extern dyn-principal match.
    let files = &[
        (
            "lib.rs",
            "#[cfg(feature = \"u\")] pub mod platform;\n\
             #[cfg(feature = \"w\")] #[path = \"win_platform.rs\"] pub mod platform;\n",
        ),
        (
            "platform.rs",
            "pub mod traits { pub trait Marker {} }\npub fn open() -> u8 { 0 }\n",
        ),
        (
            "win_platform.rs",
            "pub fn f() -> Box<dyn traits::Marker> { todo!() }\n",
        ),
    ];
    assert_eq!(
        dyn_operand_findings(
            "cfg-sibling-childmod-shadow",
            files,
            "crate::platform",
            &["traits::Marker"],
            &["traits"],
        )
        .unwrap(),
        ["dyn traits::Marker exposed by fn crate::platform::f"],
        "the w branch's own genuine extern dyn-principal must react, regardless of the u \
         branch's own local mod traits",
    );
}

#[test]
pub(super) fn a_cfg_split_module_with_two_inline_siblings_child_module_does_not_shadow_the_others_own_extern_principal()
 {
    // Round-8 finding, the operand-scoped (`shape_scan.rs`/`crate_scope.rs`) analogue of
    // `a_cfg_split_module_with_two_inline_siblings_child_module_does_not_shadow_the_others_extern_reexport`
    // above: `operand_module_findings` groups its per-branch `FileExternScope` (and `uses_by_branch`)
    // by branch index too, not just by file — two INLINE `#[cfg]` siblings share the identical
    // enclosing lib.rs, so a file-keyed group would let the "u" arm's local `mod traits` suppress
    // the "w" arm's genuine extern `dyn traits::Marker`, the identical conflation the file-form
    // version above exercises, but with both arms declared inline in one shared file.
    let files = &[(
        "lib.rs",
        "#[cfg(feature = \"u\")] pub mod platform {\n\
         pub mod traits { pub trait Marker {} }\n\
         pub fn open() -> u8 { 0 }\n}\n\
         #[cfg(feature = \"w\")] pub mod platform {\n\
         pub fn f() -> Box<dyn traits::Marker> { todo!() }\n}\n",
    )];
    assert_eq!(
        dyn_operand_findings(
            "cfg-split-inline-inline-childmod-shadow",
            files,
            "crate::platform",
            &["traits::Marker"],
            &["traits"],
        )
        .unwrap(),
        ["dyn traits::Marker exposed by fn crate::platform::f"],
        "the w arm's own genuine extern dyn-principal must react, regardless of the u arm's own \
         local mod traits, even though both arms are inline and share lib.rs",
    );
}

#[test]
pub(super) fn dyn_operand_ignores_auto_trait_markers() {
    // `dyn Port + Send`: the sole non-auto trait is Port. Forbidding Port flags it; forbidding
    // only the Send marker flags nothing (Send is an auto trait, never an operand, and a bare Send
    // does not resolve).
    assert_eq!(
        dyn_operand_mod(
            "marker-port",
            "pub fn c() -> Box<dyn crate::ports::Port + Send> { todo!() }\n",
            &["crate::ports::Port"],
        )
        .unwrap(),
        ["dyn crate::ports::Port + Send exposed by fn crate::m::c"],
    );
    assert!(
        dyn_operand_mod(
            "marker-send",
            "pub fn c() -> Box<dyn crate::ports::Port + Send> { todo!() }\n",
            &["Send"],
        )
        .unwrap()
        .is_empty(),
        "the trailing Send marker is not the operand",
    );
}

#[test]
pub(super) fn dyn_operand_matches_when_an_auto_trait_is_written_before_the_principal() {
    // `dyn Send + crate::ports::Port` — the auto trait is written FIRST. Rust allows this (only
    // lifetimes are order-constrained), so the principal is not "the first trait bound"; skipping
    // auto traits, Port is the operand and forbidding it must flag the exposure. Taking the first
    // trait bound (Send) would silently pass a forbidden operand — a false negative.
    assert_eq!(
        dyn_operand_mod(
            "auto-first",
            "pub fn c() -> Box<dyn Send + crate::ports::Port> { todo!() }\n",
            &["crate::ports::Port"],
        )
        .unwrap(),
        ["dyn Send + crate::ports::Port exposed by fn crate::m::c"],
    );
    // Two auto traits before the principal is still resolved.
    assert_eq!(
        dyn_operand_mod(
            "auto-first-2",
            "pub fn c() -> Box<dyn Send + Sync + crate::ports::Port> { todo!() }\n",
            &["crate::ports::Port"],
        )
        .unwrap(),
        ["dyn Send + Sync + crate::ports::Port exposed by fn crate::m::c"],
    );
}

#[test]
pub(super) fn dyn_operand_matches_a_dyn_nested_deep() {
    // Nested inside Vec<Box<dyn …>> — still matched by its principal trait.
    assert_eq!(
        dyn_operand_mod(
            "nested",
            "pub fn c() -> Vec<Box<dyn crate::ports::Port>> { todo!() }\n",
            &["crate::ports::Port"],
        )
        .unwrap(),
        ["dyn crate::ports::Port exposed by fn crate::m::c"],
    );
}

#[test]
pub(super) fn dyn_operand_empty_set_degenerates_to_any() {
    // An empty forbidden set reacts to any dyn — identical to shape-only, never a no-op.
    let body = "pub fn c() -> Box<dyn crate::ports::Port> { todo!() }\n";
    assert_eq!(
        dyn_operand_mod("empty", body, &[]).unwrap(),
        dyn_mod("empty-shape", body).unwrap(),
        "must_not_expose_dyn_of([]) matches exactly what shape-only must_not_expose_dyn does",
    );
    assert_eq!(
        dyn_operand_mod("empty2", body, &[]).unwrap(),
        ["dyn crate::ports::Port exposed by fn crate::m::c"],
    );
}

#[test]
pub(super) fn dyn_operand_boundary_carries_its_operands_and_severity() {
    let b = DynTraitBoundary::in_crate("core")
        .module("crate::core")
        .must_not_expose_dyn_of(["crate::ports::Port"])
        .warn()
        .because("the core seam must not leak a dyn Port");
    assert_eq!(b.forbidden_operands(), ["crate::ports::Port"]);
    assert_eq!(b.severity(), Severity::Warn);
    // Shape-only still constructs an empty operand set (regression guard).
    let shape = DynTraitBoundary::in_crate("core")
        .module("crate::core")
        .must_not_expose_dyn()
        .because("no dyn at all");
    assert!(shape.forbidden_operands().is_empty());
}
