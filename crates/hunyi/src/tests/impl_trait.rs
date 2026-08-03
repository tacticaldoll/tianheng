use super::super::*;
use super::helpers::*;
// --- impl-trait-boundary (existential exposure) ---------------------------

/// Like [`dyn_findings`] but for the impl-trait capability: write `files`, return the rendered
/// `impl …` shapes returned by `module`'s public API.
pub(super) fn impl_trait_findings(
    name: &str,
    files: &[(&str, &str)],
    module: &str,
) -> Result<Vec<String>, String> {
    shape_findings("impl", name, files, module, impl_trait_module_findings)
}

pub(super) fn impl_trait_mod(name: &str, body: &str) -> Result<Vec<String>, String> {
    impl_trait_findings(
        name,
        &[("lib.rs", "pub mod m;\n"), ("m.rs", body)],
        "crate::m",
    )
}

#[test]
pub(super) fn impl_trait_flags_a_returned_impl_trait() {
    assert_eq!(
        impl_trait_mod("ret", "pub fn make() -> impl crate::Port { todo!() }\n").unwrap(),
        ["impl crate::Port exposed by fn crate::m::make"],
    );
}

#[test]
pub(super) fn impl_trait_flags_a_nested_returned_impl_trait() {
    assert_eq!(
        impl_trait_mod(
            "nested",
            "pub fn maybe() -> Option<impl crate::Port> { todo!() }\n"
        )
        .unwrap(),
        ["impl crate::Port exposed by fn crate::m::maybe"],
        "an impl Trait at depth in the return type is existential and reacts",
    );
}

#[test]
pub(super) fn impl_trait_flags_a_trait_method_rpit() {
    assert_eq!(
        impl_trait_mod(
            "rpitit",
            "pub trait T { fn make(&self) -> impl crate::Port; }\n"
        )
        .unwrap(),
        ["impl crate::Port exposed by fn trait crate::m::T::make"],
        "a trait method's declared RPIT is the existential, governed at the declaration",
    );
}

#[test]
pub(super) fn impl_trait_does_not_flag_an_argument_position() {
    // APIT is universal (a caller-chosen generic), not an existential leak.
    assert!(
        impl_trait_mod("apit", "pub fn drive(p: impl crate::Port) { let _ = p; }\n")
            .unwrap()
            .is_empty(),
        "argument-position impl Trait is not governed",
    );
}

#[test]
pub(super) fn impl_trait_does_not_flag_an_async_fn() {
    // async fn leaks a compiler-inserted `impl Future`, not a written `impl Trait` — a
    // distinct, out-of-scope existential form (stated bound).
    assert!(
        impl_trait_mod("async", "pub async fn connect() -> u8 { 0 }\n")
            .unwrap()
            .is_empty(),
        "async fn's implicit impl Future is out of scope",
    );
}

#[test]
pub(super) fn impl_trait_does_not_flag_a_private_fn_or_a_trait_impl_method() {
    // Private fn: not public API.
    assert!(
        impl_trait_mod("priv", "fn make() -> impl crate::Port { todo!() }\n")
            .unwrap()
            .is_empty(),
        "a private fn's RPIT is not public API",
    );
    // Trait-impl method: return shape dictated by the trait declaration (governed there).
    assert!(
        impl_trait_mod(
            "traitimpl",
            "pub struct S; impl crate::T for S { fn make(&self) -> impl crate::Port { todo!() } }\n"
        )
        .unwrap()
        .is_empty(),
        "a trait-impl method's return is not double-counted",
    );
}

#[test]
pub(super) fn impl_trait_renders_iterator_and_fn_shapes_distinctly() {
    assert_eq!(
        impl_trait_mod(
            "iter",
            "pub fn it() -> impl Iterator<Item = u8> { todo!() }\n"
        )
        .unwrap(),
        ["impl Iterator<Item = u8> exposed by fn crate::m::it"],
    );
    assert_eq!(
        impl_trait_mod("clo", "pub fn f() -> impl Fn(i32) -> i32 { todo!() }\n").unwrap(),
        ["impl Fn(i32) -> i32 exposed by fn crate::m::f"],
    );
}

#[test]
pub(super) fn impl_trait_boundary_carries_anchor_and_severity() {
    let b = ImplTraitBoundary::in_crate("core")
        .module("crate::core")
        .must_not_expose_impl_trait()
        .warn()
        .because("the core seam must return named types");
    assert_eq!(b.crate_package(), "core");
    assert_eq!(b.module(), "crate::core");
    assert_eq!(b.severity(), Severity::Warn);
}

// --- operand-scoped impl-trait --------------------------------------------

pub(super) fn impl_trait_operand_findings(
    name: &str,
    files: &[(&str, &str)],
    module: &str,
    forbidden: &[&str],
    deps: &[&str],
) -> Result<Vec<String>, String> {
    operand_findings(
        "impl",
        name,
        files,
        module,
        forbidden,
        deps,
        impl_trait_operand_module_findings,
    )
}

pub(super) fn impl_trait_operand_mod(
    name: &str,
    body: &str,
    forbidden: &[&str],
) -> Result<Vec<String>, String> {
    impl_trait_operand_findings(
        name,
        &[("lib.rs", "pub mod m;\n"), ("m.rs", body)],
        "crate::m",
        forbidden,
        &[],
    )
}

#[test]
pub(super) fn impl_trait_operand_flags_a_named_trait_and_passes_others() {
    assert_eq!(
        impl_trait_operand_mod(
            "named",
            "pub fn make() -> impl crate::ports::Port { todo!() }\n",
            &["crate::ports::Port"],
        )
        .unwrap(),
        ["impl crate::ports::Port exposed by fn crate::m::make"],
    );
    // A returned impl Iterator (ergonomic existential) passes when only a domain port is forbidden.
    assert!(
        impl_trait_operand_mod(
            "iter",
            "pub fn it() -> impl Iterator<Item = u8> { todo!() }\n",
            &["crate::ports::Port"],
        )
        .unwrap()
        .is_empty(),
        "a returned impl of an unlisted (and bare-std) trait passes",
    );
}

#[test]
pub(super) fn impl_trait_operand_honors_a_module_prefix() {
    assert_eq!(
        impl_trait_operand_mod(
            "prefix",
            "pub fn make() -> impl crate::ports::Port { todo!() }\n",
            &["crate::ports"],
        )
        .unwrap(),
        ["impl crate::ports::Port exposed by fn crate::m::make"],
    );
}

#[test]
pub(super) fn impl_trait_operand_matches_a_reexported_trait_by_its_defining_path() {
    let files = &[
        (
            "lib.rs",
            "pub mod ports;\npub use crate::ports::Port;\npub mod m;\n",
        ),
        ("ports.rs", "pub trait Port {}\n"),
        ("m.rs", "pub fn make() -> impl crate::Port { todo!() }\n"),
    ];
    assert_eq!(
        impl_trait_operand_findings("reexport", files, "crate::m", &["crate::ports::Port"], &[])
            .unwrap(),
        ["impl crate::Port exposed by fn crate::m::make"],
    );
}

#[test]
pub(super) fn impl_trait_operand_ignores_auto_trait_markers() {
    assert_eq!(
        impl_trait_operand_mod(
            "marker-port",
            "pub fn make() -> impl crate::ports::Port + Send { todo!() }\n",
            &["crate::ports::Port"],
        )
        .unwrap(),
        ["impl crate::ports::Port + Send exposed by fn crate::m::make"],
    );
    assert!(
        impl_trait_operand_mod(
            "marker-send",
            "pub fn make() -> impl crate::ports::Port + Send { todo!() }\n",
            &["Send"],
        )
        .unwrap()
        .is_empty(),
        "the trailing Send marker is not the operand",
    );
}

#[test]
pub(super) fn impl_trait_operand_matches_an_auto_trait_written_before_the_principal() {
    // `impl Send + crate::ports::Port` — auto trait first (valid Rust; impl-Trait bounds are an
    // unordered set). Skipping auto traits, Port is the operand and forbidding it must flag it.
    assert_eq!(
        impl_trait_operand_mod(
            "auto-first",
            "pub fn make() -> impl Send + crate::ports::Port { todo!() }\n",
            &["crate::ports::Port"],
        )
        .unwrap(),
        ["impl Send + crate::ports::Port exposed by fn crate::m::make"],
    );
}

#[test]
pub(super) fn impl_trait_operand_matches_a_second_non_auto_trait() {
    // `impl crate::ports::Port + crate::ports::Sink` — a returned `impl Trait` may name several
    // non-auto traits. Forbidding the SECOND one must flag it: the returned type genuinely is a
    // Sink. Matching only the first non-auto trait would silently pass it (a false negative).
    assert_eq!(
        impl_trait_operand_mod(
            "second-trait",
            "pub fn make() -> impl crate::ports::Port + crate::ports::Sink { todo!() }\n",
            &["crate::ports::Sink"],
        )
        .unwrap(),
        ["impl crate::ports::Port + crate::ports::Sink exposed by fn crate::m::make"],
    );
}

#[test]
pub(super) fn impl_trait_operand_matches_a_nested_returned_impl() {
    assert_eq!(
        impl_trait_operand_mod(
            "nested",
            "pub fn maybe() -> Option<impl crate::ports::Port> { todo!() }\n",
            &["crate::ports::Port"],
        )
        .unwrap(),
        ["impl crate::ports::Port exposed by fn crate::m::maybe"],
    );
}

#[test]
pub(super) fn impl_trait_operand_empty_set_degenerates_to_any() {
    let body = "pub fn make() -> impl crate::ports::Port { todo!() }\n";
    assert_eq!(
        impl_trait_operand_mod("empty", body, &[]).unwrap(),
        impl_trait_mod("empty-shape", body).unwrap(),
        "must_not_expose_impl_trait_of([]) matches exactly what shape-only does",
    );
}

#[test]
pub(super) fn impl_trait_operand_inherits_return_position_scoping() {
    // APIT and async fn stay out of scope under the operand variant too.
    assert!(
        impl_trait_operand_mod(
            "apit",
            "pub fn drive(p: impl crate::ports::Port) { let _ = p; }\n",
            &["crate::ports::Port"],
        )
        .unwrap()
        .is_empty(),
        "argument-position impl Trait is not governed even with a matching operand",
    );
    assert!(
        impl_trait_operand_mod(
            "async",
            "pub async fn c() -> u8 { 0 }\n",
            &["crate::ports::Port"]
        )
        .unwrap()
        .is_empty(),
    );
}

#[test]
pub(super) fn impl_trait_operand_boundary_carries_operands_and_severity() {
    let b = ImplTraitBoundary::in_crate("core")
        .module("crate::core")
        .must_not_expose_impl_trait_of(["crate::ports::Port"])
        .warn()
        .because("the core seam must not return an existential Port");
    assert_eq!(b.forbidden_operands(), ["crate::ports::Port"]);
    assert_eq!(b.severity(), Severity::Warn);
    let shape = ImplTraitBoundary::in_crate("core")
        .module("crate::core")
        .must_not_expose_impl_trait()
        .because("no existential at all");
    assert!(shape.forbidden_operands().is_empty());
}

#[test]
pub(super) fn impl_trait_boundary_carries_anchor_and_including_submodules() {
    let b = ImplTraitBoundary::in_crate("core")
        .module("crate::core")
        .must_not_expose_impl_trait()
        .warn()
        .because("the core seam must return named types, not an existential");
    assert_eq!(b.severity(), Severity::Warn);
    // The subtree opt-in defaults off and threads through `.because`.
    assert!(!b.including_submodules());
    let sub = ImplTraitBoundary::in_crate("core")
        .module("crate")
        .must_not_expose_impl_trait()
        .including_submodules()
        .because("no existential anywhere under the kernel");
    assert!(sub.including_submodules());
}

// --- impl-trait: subtree scope (`including_submodules`) -------------------

pub(super) fn impl_trait_subtree(
    name: &str,
    files: &[(&str, &str)],
    module: &str,
) -> Result<Vec<(String, String)>, String> {
    subtree_findings("impl", name, files, module, impl_trait_subtree_findings)
}

/// Just the finding strings, sorted — for cases where the module attribution rides inside the
/// finding string anyway.
pub(super) fn impl_trait_subtree_labels(
    name: &str,
    files: &[(&str, &str)],
    module: &str,
) -> Vec<String> {
    impl_trait_subtree(name, files, module)
        .unwrap()
        .into_iter()
        .map(|(finding, _module)| finding)
        .collect()
}

#[test]
pub(super) fn impl_trait_subtree_reacts_to_a_submodule_return_the_seam_scope_misses() {
    // The crux this opt-in exists for, mirroring async-exposure's own: a returned `impl Trait` in a
    // *submodule* is invisible to the default seam scope (anchored at `crate`, it sees only
    // crate-root items) — the gap the `no_existential_leak` composed profile's own honesty
    // requires closed for its impl-trait half. The subtree scope catches it.
    let files = &[
        ("lib.rs", "pub mod net;\n"),
        ("net.rs", "pub fn make() -> impl crate::Port { todo!() }\n"),
    ];
    // Default seam scope at `crate` misses it entirely…
    assert_eq!(
        impl_trait_findings("seam-misses-sub", files, "crate").unwrap(),
        Vec::<String>::new(),
    );
    // …the subtree scope reacts, attributing it to the submodule.
    let subtree = impl_trait_subtree("sub-reacts", files, "crate").unwrap();
    assert_eq!(subtree.len(), 1);
    assert_eq!(subtree[0].1, "crate::net");
    assert!(subtree[0].0.contains("impl crate::Port"), "{:?}", subtree);
}

/// A cfg_attr(path)-hidden submodule is observed, whichever candidate file exists — the identical
/// `resolve_child_modules`/`walk_subtree_modules` mechanism fixed by
/// `hunyi-cfg-attr-path-module-loss` for `scan_crate`'s own consumers. Not named in that change's
/// own commit message (a documentation gap a round-3 adversarial review found and closed) but the
/// same shared walker, independently reproduced here before being counted as fixed.
#[test]
pub(super) fn impl_trait_subtree_reacts_through_a_cfg_attr_wrapped_path_submodule() {
    let files = &[
        (
            "lib.rs",
            "#[cfg_attr(any(), path = \"never.rs\")]\npub mod net;\n",
        ),
        ("net.rs", "pub fn make() -> impl crate::Port { todo!() }\n"),
    ];
    let subtree = impl_trait_subtree("cfg-attr-path", files, "crate").unwrap();
    assert_eq!(subtree.len(), 1);
    assert_eq!(subtree[0].1, "crate::net");
    assert!(subtree[0].0.contains("impl crate::Port"), "{:?}", subtree);
}

#[test]
pub(super) fn impl_trait_subtree_includes_the_anchor_modules_own_seam_byte_identically() {
    // The anchor module's own returned `impl Trait` is still caught, and its finding string is
    // byte-identical to the single-module path — so enabling the opt-in on a seam-only boundary
    // adds deeper findings without re-identifying the seam ones (baseline stability).
    let files = &[
        ("lib.rs", "pub mod m;\n"),
        (
            "m.rs",
            "pub fn own() -> impl crate::Port { todo!() }\npub mod deep;\n",
        ),
        (
            "m/deep.rs",
            "pub fn nested() -> impl crate::Port { todo!() }\n",
        ),
    ];
    let seam = impl_trait_findings("seam-parity", files, "crate::m").unwrap();
    assert_eq!(seam.len(), 1);
    let subtree = impl_trait_subtree_labels("subtree-parity", files, "crate::m");
    assert_eq!(subtree.len(), 2);
    // The seam finding appears verbatim in the subtree result.
    assert!(subtree.contains(&seam[0]));
}

#[test]
pub(super) fn impl_trait_subtree_scopes_to_the_anchored_subtree_not_the_whole_crate() {
    let files = &[
        ("lib.rs", "pub mod a;\npub mod c;\n"),
        (
            "a.rs",
            "pub mod b;\npub fn make() -> impl crate::Port { todo!() }\n",
        ),
        ("a/b.rs", "pub fn make() -> impl crate::Port { todo!() }\n"),
        ("c.rs", "pub fn make() -> impl crate::Port { todo!() }\n"),
    ];
    let subtree = impl_trait_subtree("bounded", files, "crate::a").unwrap();
    let modules: Vec<&str> = subtree.iter().map(|(_, m)| m.as_str()).collect();
    assert!(modules.contains(&"crate::a"));
    assert!(modules.contains(&"crate::a::b"));
    assert!(!modules.contains(&"crate::c"), "{:?}", modules);
}

/// The BACKLOG false negative this change closes: `owner` is the self type's canonical path, not
/// where the impl block is written, so two inherent impls of the SAME type in DIFFERENT modules
/// (a platform-conditional split — here `plat_unix`/`plat_win` both writing `impl Conn` for a
/// `Conn` declared in `common`) previously collapsed to one violation when both declared a
/// same-named public RPIT method. `PublicSeam::InherentMethod` now carries the impl block's own
/// declaring module, so both react — the two-module case impl-trait's subtree scan is the one
/// capability that can currently observe in a single evaluation.
#[test]
pub(super) fn impl_trait_subtree_two_platform_modules_impling_the_same_owner_stay_distinct() {
    let files = &[
        (
            "lib.rs",
            "pub mod common;\npub mod plat_unix;\npub mod plat_win;\n",
        ),
        ("common.rs", "pub struct Conn;\n"),
        (
            "plat_unix.rs",
            "use crate::common::Conn;\nimpl Conn { pub fn open(&self) -> impl crate::Port { todo!() } }\n",
        ),
        (
            "plat_win.rs",
            "use crate::common::Conn;\nimpl Conn { pub fn open(&self) -> impl crate::Port { todo!() } }\n",
        ),
    ];
    let subtree = impl_trait_subtree("plat-split", files, "crate").unwrap();
    let modules: Vec<&str> = subtree.iter().map(|(_, m)| m.as_str()).collect();
    assert_eq!(subtree.len(), 2, "{:?}", subtree);
    assert!(modules.contains(&"crate::plat_unix"), "{:?}", subtree);
    assert!(modules.contains(&"crate::plat_win"), "{:?}", subtree);
    for (finding, _module) in &subtree {
        assert!(finding.contains("impl crate::Port"), "{finding}");
    }
}

/// The same BACKLOG false negative the impl-trait test above pins, for signature-coupling:
/// neither signature-coupling nor dyn-trait has a subtree scanner that can observe two modules
/// in one evaluation (impl-trait is the one capability that does), so this proves the identical
/// property the other way — two independent module-scoped evaluations, one per platform module,
/// must produce distinct STRUCTURED IDENTITY (the `SemanticFact`, which feeds
/// `StructuredFactIdentity` via `PublicSeam::key_fields`'s `seam_module`) even though `Display`
/// deliberately renders the same text for both (see `PublicSeam::InherentMethod`'s own doc
/// comment: "Identity-only: `Display` ignores it"). An earlier draft of this test compared the
/// rendered strings instead of the fact identity and failed — not because production is broken,
/// but because the rendered message is intentionally module-blind while the baseline identity is
/// not; comparing `SemanticFact` values (as production's own dedup does) is the correct check.
#[test]
pub(super) fn signature_coupling_two_platform_modules_impling_the_same_owner_stay_distinct() {
    let tree = TempSrcTree::new("sig-two-platform-modules");
    tree.write_all(&[
        (
            "lib.rs",
            "pub mod common;\npub mod infra;\npub mod plat_unix;\npub mod plat_win;\n",
        ),
        ("common.rs", "pub struct Conn;\n"),
        ("infra.rs", "pub struct Secret;\n"),
        (
            "plat_unix.rs",
            "use crate::common::Conn;\nimpl Conn { pub fn describe(&self) -> crate::infra::Secret { todo!() } }\n",
        ),
        (
            "plat_win.rs",
            "use crate::common::Conn;\nimpl Conn { pub fn describe(&self) -> crate::infra::Secret { todo!() } }\n",
        ),
    ]);
    let forbidden = vec!["crate::infra::Secret".to_string()];
    let unix = crate::exposure::module_findings(
        tree.src(),
        &tree.root(),
        "crate::plat_unix",
        &forbidden,
        "x",
        false,
        &[],
    )
    .unwrap();
    let win = crate::exposure::module_findings(
        tree.src(),
        &tree.root(),
        "crate::plat_win",
        &forbidden,
        "x",
        false,
        &[],
    )
    .unwrap();
    assert_eq!(unix.len(), 1, "{unix:?}");
    assert_eq!(win.len(), 1, "{win:?}");
    // Same rendered text (Display is module-blind by design)...
    assert_eq!(unix[0].0.to_string(), win[0].0.to_string());
    // ...but distinct structured identity, which is what baseline dedup actually keys on.
    assert_ne!(
        unix[0].0, win[0].0,
        "two impl blocks for the same owner in different modules must not collapse to one \
         structured fact: {unix:?} vs {win:?}"
    );
}

/// Same property as above, for dyn-trait's operand-scoped resolver.
#[test]
pub(super) fn dyn_trait_two_platform_modules_impling_the_same_owner_stay_distinct() {
    let tree = TempSrcTree::new("dyn-two-platform-modules");
    tree.write_all(&[
        (
            "lib.rs",
            "pub mod common;\npub mod infra;\npub mod plat_unix;\npub mod plat_win;\n",
        ),
        ("common.rs", "pub struct Conn;\n"),
        ("infra.rs", "pub trait Port {}\n"),
        (
            "plat_unix.rs",
            "use crate::common::Conn;\nimpl Conn { pub fn make(&self) -> Box<dyn crate::infra::Port> { todo!() } }\n",
        ),
        (
            "plat_win.rs",
            "use crate::common::Conn;\nimpl Conn { pub fn make(&self) -> Box<dyn crate::infra::Port> { todo!() } }\n",
        ),
    ]);
    let forbidden = vec!["crate::infra::Port".to_string()];
    let unix = crate::dyn_trait::dyn_operand_module_findings(
        tree.src(),
        &tree.root(),
        "crate::plat_unix",
        &forbidden,
        "x",
        &[],
    )
    .unwrap();
    let win = crate::dyn_trait::dyn_operand_module_findings(
        tree.src(),
        &tree.root(),
        "crate::plat_win",
        &forbidden,
        "x",
        &[],
    )
    .unwrap();
    assert_eq!(unix.len(), 1, "{unix:?}");
    assert_eq!(win.len(), 1, "{win:?}");
    assert_eq!(unix[0].0.to_string(), win[0].0.to_string());
    assert_ne!(
        unix[0].0, win[0].0,
        "two impl blocks for the same owner in different modules must not collapse to one \
         structured fact: {unix:?} vs {win:?}"
    );
}

/// Same property as the two tests above, for the `InherentAssoc` seam kind itself (assoc
/// `const`/`type`), which — unlike `InherentMethod` — had no two-module regression test at all
/// before this, in any capability. Mirrors `dyn_in_an_inherent_impl_public_assoc_const_reacts`'s
/// fixture shape, split across two platform modules for the same owner.
#[test]
pub(super) fn dyn_inherent_assoc_const_two_platform_modules_impling_the_same_owner_stay_distinct() {
    let tree = TempSrcTree::new("assoc-two-platform-modules");
    tree.write_all(&[
        ("lib.rs", "pub mod common;\npub mod plat_unix;\npub mod plat_win;\n"),
        ("common.rs", "pub struct Config;\n"),
        (
            "plat_unix.rs",
            "use crate::common::Config;\nimpl Config { pub const DEFAULT: &dyn crate::Port = todo!(); }\n",
        ),
        (
            "plat_win.rs",
            "use crate::common::Config;\nimpl Config { pub const DEFAULT: &dyn crate::Port = todo!(); }\n",
        ),
    ]);
    let unix =
        crate::dyn_trait::dyn_module_findings(tree.src(), &tree.root(), "crate::plat_unix", "x")
            .unwrap();
    let win =
        crate::dyn_trait::dyn_module_findings(tree.src(), &tree.root(), "crate::plat_win", "x")
            .unwrap();
    assert_eq!(unix.len(), 1, "{unix:?}");
    assert_eq!(win.len(), 1, "{win:?}");
    assert_eq!(unix[0].0.to_string(), win[0].0.to_string());
    assert_ne!(
        unix[0].0, win[0].0,
        "two impl blocks for the same owner in different modules must not collapse to one \
         structured fact: {unix:?} vs {win:?}"
    );
}

/// The `InherentGenerics` seam's own per-position role, closing what module-plus-owner cannot.
#[test]
pub(super) fn two_impl_blocks_in_one_module_stay_distinct_by_their_bound() {
    // The recorded reproduction this seam's `bound` role closes: two separate inherent impl blocks on
    // the same type, in the SAME module, each exposing the same forbidden subject through a DIFFERENT
    // bound. Module-plus-owner cannot separate them — module says where a block is written, owner says
    // what it is for, neither says which block — so both facts were identical and collapsed to one
    // violation, letting a baseline accepting the first suppress the second's never-accepted one.
    //
    // The discriminator is the bounded thing's own name, keyed exactly like the sibling trait-impl
    // `where` position. It is deliberately NOT the block's index:
    // `semantic-signature-coupling` forbids identity resting on scan order or item ordinal, so a
    // positional key would trade one defect for a rule violation.
    let tree = TempSrcTree::new("sig-two-blocks-one-module");
    tree.write_all(&[
        ("lib.rs", "pub mod common;\npub mod infra;\npub mod plat;\n"),
        ("common.rs", "pub struct Conn<T, U>(pub T, pub U);\n"),
        ("infra.rs", "pub trait Secret {}\n"),
        (
            "plat.rs",
            "use crate::common::Conn;\n\
             impl<T: crate::infra::Secret, U> Conn<T, U> { pub fn open(&self) {} }\n\
             impl<T, U: crate::infra::Secret> Conn<T, U> { pub fn close(&self) {} }\n",
        ),
    ]);
    let forbidden = vec!["crate::infra::Secret".to_string()];
    let findings = crate::exposure::module_findings(
        tree.src(),
        &tree.root(),
        "crate::plat",
        &forbidden,
        "x",
        false,
        &[],
    )
    .unwrap();
    assert_eq!(
        findings.len(),
        2,
        "two blocks bounding different parameters to the same forbidden type are two distinct \
         violations, not one: {findings:?}"
    );
    let facts: std::collections::BTreeSet<_> = findings.iter().map(|(fact, _)| fact).collect();
    assert_eq!(
        facts.len(),
        2,
        "and their structured facts must differ, which is what a baseline keys on: {findings:?}"
    );
}

/// The same seam's other axis: two blocks for one owner in DIFFERENT modules. This seam had no module
/// in its identity at all — unlike a method or an associated item it carries no per-item name to fall
/// back on, so `owner` was its whole distinguishing content and two blocks for one owner collapsed
/// outright. Rust permits the two blocks (coherence constrains trait impls, never inherent ones).
#[test]
pub(super) fn signature_coupling_two_platform_modules_impl_generics_stay_distinct() {
    let tree = TempSrcTree::new("sig-two-platform-generics");
    tree.write_all(&[
        (
            "lib.rs",
            "pub mod common;\npub mod infra;\npub mod plat_unix;\npub mod plat_win;\n",
        ),
        ("common.rs", "pub struct Conn<T>(pub T);\n"),
        ("infra.rs", "pub trait Secret {}\n"),
        (
            "plat_unix.rs",
            "use crate::common::Conn;\nimpl<T: crate::infra::Secret> Conn<T> { pub fn open(&self) {} }\n",
        ),
        (
            "plat_win.rs",
            "use crate::common::Conn;\nimpl<T: crate::infra::Secret> Conn<T> { pub fn open(&self) {} }\n",
        ),
    ]);
    let forbidden = vec!["crate::infra::Secret".to_string()];
    let findings_at = |module: &str| {
        crate::exposure::module_findings(
            tree.src(),
            &tree.root(),
            module,
            &forbidden,
            "x",
            false,
            &[],
        )
        .unwrap()
    };
    let unix = findings_at("crate::plat_unix");
    let win = findings_at("crate::plat_win");
    assert_eq!(unix.len(), 1, "{unix:?}");
    assert_eq!(win.len(), 1, "{win:?}");
    // Same rendered text (Display is module-blind here too, matching `InherentMethod`)...
    assert_eq!(unix[0].0.to_string(), win[0].0.to_string());
    // ...but distinct structured identity, which is what a baseline keys on.
    assert_ne!(
        unix[0].0, win[0].0,
        "two impl blocks for the same owner in different modules must not collapse their own \
         generics seams into one structured fact: {unix:?} vs {win:?}"
    );
}

/// The `InherentGenerics` property again for dyn-trait, which builds that seam through its own
/// second construction site (`collect_item_dyn_exposures`) — a guard on signature-coupling alone
/// would leave that site free to pass a module-blind constant.
#[test]
pub(super) fn dyn_two_platform_modules_impl_generics_stay_distinct() {
    let tree = TempSrcTree::new("dyn-two-platform-generics");
    tree.write_all(&[
        (
            "lib.rs",
            "pub mod common;\npub mod plat_unix;\npub mod plat_win;\n",
        ),
        ("common.rs", "pub struct Conn<T>(pub T);\n"),
        (
            "plat_unix.rs",
            "use crate::common::Conn;\nimpl<T: AsRef<Box<dyn crate::Port>>> Conn<T> { pub fn open(&self) {} }\n",
        ),
        (
            "plat_win.rs",
            "use crate::common::Conn;\nimpl<T: AsRef<Box<dyn crate::Port>>> Conn<T> { pub fn open(&self) {} }\n",
        ),
    ]);
    let unix =
        crate::dyn_trait::dyn_module_findings(tree.src(), &tree.root(), "crate::plat_unix", "x")
            .unwrap();
    let win =
        crate::dyn_trait::dyn_module_findings(tree.src(), &tree.root(), "crate::plat_win", "x")
            .unwrap();
    assert_eq!(unix.len(), 1, "{unix:?}");
    assert_eq!(win.len(), 1, "{win:?}");
    assert_eq!(unix[0].0.to_string(), win[0].0.to_string());
    assert_ne!(
        unix[0].0, win[0].0,
        "two impl blocks for the same owner in different modules must not collapse their own \
         generics seams into one structured fact: {unix:?} vs {win:?}"
    );
}

/// The same property for the `ExternCrate` seam: `pub extern crate <dep>;` republishes one external
/// crate root, and a crate may write it in more than one module. The crate name alone was the whole
/// identity, so two such re-exports collapsed — the sibling shape of the generics case above, in the
/// one other seam that carried no module.
#[test]
pub(super) fn two_modules_republishing_one_extern_crate_stay_distinct() {
    let tree = TempSrcTree::new("sig-two-module-extern-crate");
    tree.write_all(&[
        ("lib.rs", "pub mod alpha;\npub mod beta;\n"),
        ("alpha.rs", "pub extern crate worklane_core;\n"),
        ("beta.rs", "pub extern crate worklane_core;\n"),
    ]);
    let forbidden = vec!["worklane_core".to_string()];
    let deps = vec!["worklane_core".to_string()];
    let findings_at = |module: &str| {
        crate::exposure::module_findings(
            tree.src(),
            &tree.root(),
            module,
            &forbidden,
            "x",
            false,
            &deps,
        )
        .unwrap()
    };
    let alpha = findings_at("crate::alpha");
    let beta = findings_at("crate::beta");
    assert_eq!(alpha.len(), 1, "{alpha:?}");
    assert_eq!(beta.len(), 1, "{beta:?}");
    assert_eq!(alpha[0].0.to_string(), beta[0].0.to_string());
    assert_ne!(
        alpha[0].0, beta[0].0,
        "two modules republishing the same extern crate must not collapse to one structured \
         fact: {alpha:?} vs {beta:?}"
    );
}

#[test]
pub(super) fn impl_trait_subtree_tolerates_a_cfg_gated_fileless_submodule() {
    let files = &[
        (
            "lib.rs",
            "#[cfg(feature = \"never\")]\npub mod optional;\npub mod present;\n",
        ),
        (
            "present.rs",
            "pub fn make() -> impl crate::Port { todo!() }\n",
        ),
    ];
    let subtree = impl_trait_subtree("cfg-fileless", files, "crate").unwrap();
    assert_eq!(subtree.len(), 1);
    assert_eq!(subtree[0].1, "crate::present");
}

#[test]
pub(super) fn impl_trait_subtree_errors_on_a_non_cfg_missing_submodule() {
    let files = &[("lib.rs", "pub mod missing;\n")];
    let err = impl_trait_subtree("non-cfg-missing", files, "crate").unwrap_err();
    assert!(err.contains("missing"), "{err}");
}

#[test]
pub(super) fn impl_trait_subtree_does_not_observe_a_body_nested_module() {
    let files = &[(
        "lib.rs",
        "pub fn outer() { mod inner { pub fn hidden() -> impl crate::Port { todo!() } } }\n",
    )];
    let subtree = impl_trait_subtree("body-nested", files, "crate").unwrap();
    assert!(subtree.is_empty(), "{:?}", subtree);
}

#[test]
pub(super) fn impl_trait_subtree_and_seam_both_fail_loud_on_an_unrenderable_owner() {
    // Mirrors `async_subtree_and_seam_both_fail_loud_on_an_unrenderable_owner` exactly: impl-trait's
    // owner resolution (`canonical_self_owner`) produces an internal positional sentinel for a
    // genuinely unrenderable self type, caught by the shared `reject_positional_identity` gate
    // (invoked via `sort_attributed_facts`/`sort_faceted_facts`) — never published as identity,
    // under either scope.
    let files = &[(
        "lib.rs",
        "pub struct Arr<const N: usize>;\npub struct Marker;\nimpl Marker { pub fn before() -> impl crate::Port { todo!() } }\nimpl<const N: usize> Arr<{ N + 1 }> { pub fn unrenderable() -> impl crate::Port { todo!() } }\n",
    )];
    let seam = impl_trait_findings("const-generic-owner-parity-seam", files, "crate").unwrap_err();
    let subtree =
        impl_trait_subtree("const-generic-owner-parity-subtree", files, "crate").unwrap_err();
    assert!(seam.contains("without a stable structural label"), "{seam}");
    assert!(
        subtree.contains("without a stable structural label"),
        "{subtree}"
    );
    assert!(!seam.contains("_#") && !subtree.contains("_#"));
}

#[test]
pub(super) fn impl_trait_subtree_cfg_branches_never_share_an_unrenderable_owner_fallback() {
    // Mirrors `async_cfg_branches_never_share_an_unrenderable_owner_fallback` exactly: two
    // mutually-exclusive `#[cfg]` branches of the same module each declare a same-named type with
    // an unrenderable const-generic self-type argument (`Arr<{ N + 1 }>` vs `Arr<{ N + 2 }>`). This
    // is the actual case task 1.3a's continuous-ordinal threading protects: a hardcoded or
    // reset-per-module ordinal would let the two branches' sentinels collide into one internal
    // value before `reject_positional_identity` ever runs. The gate still fails loud either way, so
    // this proves the ordinal is threaded correctly, not merely that the gate exists (the single-site
    // test above already proves the gate; this one proves the counter feeding it).
    let files = &[
        (
            "lib.rs",
            "#[cfg(feature = \"u\")]\npub mod m;\n#[cfg(feature = \"w\")]\n#[path = \"m_w.rs\"]\npub mod m;\n",
        ),
        (
            "m.rs",
            "pub struct Arr<const N: usize>;\nimpl<const N: usize> Arr<{ N + 1 }> { pub fn run() -> impl crate::Port { todo!() } }\n",
        ),
        (
            "m_w.rs",
            "pub struct Arr<const N: usize>;\nimpl<const N: usize> Arr<{ N + 2 }> { pub fn run() -> impl crate::Port { todo!() } }\n",
        ),
    ];
    let error =
        impl_trait_subtree("cfg-split-owner-fallback-collision", files, "crate::m").unwrap_err();
    assert!(
        error.contains("without a stable structural label"),
        "{error}"
    );
    assert!(!error.contains("_#"), "{error}");
}

#[test]
pub(super) fn impl_trait_operand_scoped_boundary_reacts_across_file_and_inline_submodules() {
    let (metadata, _fixture) = fixture_metadata(
        "impltrait-operand-subtree",
        &[
            (
                "lib.rs",
                "pub trait Port {}\npub trait Other {}\npub mod file;\n\
                 pub mod inline { pub fn other() -> impl crate::Other { todo!() } }\n",
            ),
            (
                "file.rs",
                "use crate::Port as ApiPort;\npub fn port() -> impl ApiPort { todo!() }\n",
            ),
        ],
    );
    let boundary = ImplTraitBoundary::in_crate("x")
        .module("crate")
        .must_not_expose_impl_trait_of(["crate::Port"])
        .including_submodules()
        .because("r");
    let mut violations = Vec::new();
    check_impl_trait_boundary(&metadata, &boundary, &mut violations).unwrap();
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].target(), "crate");
    assert!(violations[0].finding.contains("port"));
    assert!(
        violations[0]
            .file
            .as_deref()
            .is_some_and(|file| file.ends_with("file.rs"))
    );
}
