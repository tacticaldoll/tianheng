use super::super::*;
use super::dyn_trait::{dyn_findings, dyn_mod};
use super::helpers::*;
// --- async-exposure -------------------------------------------------------

pub(super) fn async_findings(
    name: &str,
    files: &[(&str, &str)],
    module: &str,
) -> Result<Vec<String>, String> {
    shape_findings("async", name, files, module, async_exposure_module_findings)
}

pub(super) fn async_mod(name: &str, body: &str) -> Result<Vec<String>, String> {
    async_findings(
        name,
        &[("lib.rs", "pub mod m;\n"), ("m.rs", body)],
        "crate::m",
    )
}

pub(super) fn async_observations(
    name: &str,
    body: &str,
) -> Result<Vec<(StructuredFactIdentity, String)>, String> {
    let tree = TempSrcTree::new(&format!("async-observation-{name}"));
    tree.write_all(&[("lib.rs", "pub mod registry;\n"), ("registry.rs", body)]);
    async_exposure_module_findings(tree.src(), &tree.root(), "crate::registry", "x").map(|facts| {
        facts
            .into_iter()
            .map(|(fact, _)| {
                let finding = fact.into_finding("app", "src/lib.rs");
                (finding.key().clone(), finding.text().to_string())
            })
            .collect()
    })
}

#[test]
pub(super) fn pacta_shaped_registry_signature_changes_preserve_async_seam_identity() {
    let first = async_observations(
        "pacta-v1",
        "pub struct Registry;\npub struct Contract;\nimpl Registry { pub async fn register(&self, contract: Contract) {} }\n",
    )
    .unwrap();
    let second = async_observations(
        "pacta-v2",
        "pub struct Registry;\npub struct Receipt;\nimpl Registry { pub async fn register(&mut self, name: &str, version: u64) -> Receipt { Receipt } }\n",
    )
    .unwrap();
    assert_eq!(first.len(), 1);
    assert_eq!(second.len(), 1);
    assert_eq!(first[0].0, second[0].0);
    assert_ne!(first[0].1, second[0].1);
}

#[test]
pub(super) fn async_production_violation_separates_target_rule_and_seam() {
    let (metadata, _fixture) = fixture_metadata(
        "async-identity",
        &[
            ("lib.rs", "pub mod registry;\n"),
            ("registry.rs", "pub async fn register(name: &str) {}\n"),
        ],
    );
    let boundary = AsyncExposureBoundary::in_crate("x")
        .module("crate::registry")
        .must_not_expose_async_fn()
        .because("registry operations keep a synchronous seam");
    let mut violations = Vec::new();
    check_async_exposure_boundary(&metadata, &boundary, &mut violations).unwrap();
    assert_eq!(violations.len(), 1);

    let id = violations[0].id();
    assert_eq!(id.target(), "crate::registry");
    let rule = id.rule_key();
    assert_eq!(rule.rule_type(), "tianheng.rule/hunyi/async-exposure");
    assert_eq!(
        rule.fields().collect::<Vec<_>>(),
        Vec::<(&str, &str)>::new()
    );
    let fact = id.fact();
    assert_eq!(fact.fact_type(), "tianheng.fact/hunyi/async-exposure");
    assert_eq!(fact.shape(), "async-free-function");
    assert_eq!(
        fact.fields().collect::<Vec<_>>(),
        vec![
            ("governing_package", "x"),
            ("module", "crate::registry"),
            ("name", "register"),
            ("owner", "crate::registry"),
            ("owner_kind", "module"),
        ]
    );
}

#[test]
pub(super) fn subtree_opt_in_preserves_anchored_finding_violation_id_identity() {
    let default_rule = AsyncExposureBoundary::in_crate("pkg")
        .module("crate::m")
        .must_not_expose_async_fn()
        .because("test");
    let subtree_rule = AsyncExposureBoundary::in_crate("pkg")
        .module("crate::m")
        .must_not_expose_async_fn()
        .including_submodules()
        .because("test");
    assert_eq!(
        default_rule.rule_key(),
        subtree_rule.rule_key(),
        "toggling including_submodules must not alter RuleKey identity"
    );
}

#[test]
pub(super) fn async_exposure_flags_a_public_async_free_fn() {
    assert_eq!(
        async_mod("free", "pub async fn connect() -> u8 { 0 }\n").unwrap(),
        ["async fn crate::m::connect() -> u8"],
    );
}

#[test]
pub(super) fn async_exposure_flags_a_public_inherent_async_method() {
    assert_eq!(
        async_mod(
            "inherent",
            "pub struct Service; impl Service { pub async fn run(&self) {} }\n"
        )
        .unwrap(),
        ["async fn <crate::m::Service>::run(&self)"],
    );
}

#[test]
pub(super) fn async_exposure_flags_a_public_trait_async_method_declaration() {
    assert_eq!(
        async_mod("trait", "pub trait Port { async fn fetch(&self) -> u8; }\n").unwrap(),
        ["async fn trait crate::m::Port::fetch(&self) -> u8"],
    );
}

#[test]
pub(super) fn async_exposure_does_not_flag_trait_impl_private_or_nonasync() {
    // Trait-impl async method: dictated by the trait declaration — not double-counted.
    assert!(
        async_mod(
            "traitimpl",
            "pub struct S; impl crate::T for S { async fn run(&self) {} }\n"
        )
        .unwrap()
        .is_empty(),
    );
    // Private async fn: not public API.
    assert!(
        async_mod("priv", "async fn helper() {}\n")
            .unwrap()
            .is_empty(),
    );
    // Non-async public fn: not async.
    assert!(
        async_mod("sync", "pub fn ready() -> u8 { 0 }\n")
            .unwrap()
            .is_empty(),
    );
}

#[test]
pub(super) fn async_exposure_finding_is_injective_across_same_named_owners() {
    // The crux: two same-named async methods across two inherent impls must NOT collide, or a
    // baselined one would mask the other (a false negative).
    let two_impls = async_mod(
        "two-impls",
        "pub struct A; pub struct B;\n\
         impl A { pub async fn run(&self) {} }\n\
         impl B { pub async fn run(&self) {} }\n",
    )
    .unwrap();
    assert_eq!(
        two_impls,
        [
            "async fn <crate::m::A>::run(&self)".to_string(),
            "async fn <crate::m::B>::run(&self)".to_string(),
        ],
        "same-named async methods across two impls yield two distinct owner-qualified findings",
    );
    // And two same-named async methods across two traits.
    let two_traits = async_mod(
        "two-traits",
        "pub trait T { async fn run(&self); }\npub trait U { async fn run(&self); }\n",
    )
    .unwrap();
    assert_eq!(
        two_traits,
        [
            "async fn trait crate::m::T::run(&self)".to_string(),
            "async fn trait crate::m::U::run(&self)".to_string(),
        ],
    );
}

#[test]
pub(super) fn async_exposure_boundary_carries_anchor_and_severity() {
    let b = AsyncExposureBoundary::in_crate("core")
        .module("crate::core")
        .must_not_expose_async_fn()
        .warn()
        .because("the core seam is synchronous");
    assert_eq!(b.crate_package(), "core");
    assert_eq!(b.module(), "crate::core");
    assert_eq!(b.severity(), Severity::Warn);
    // The subtree opt-in defaults off and threads through `.because`.
    assert!(!b.including_submodules());
    let sub = AsyncExposureBoundary::in_crate("core")
        .module("crate")
        .must_not_expose_async_fn()
        .including_submodules()
        .because("no async anywhere under the kernel");
    assert!(sub.including_submodules());
}

// --- async-exposure: subtree scope (`including_submodules`) ----------------

pub(super) fn async_subtree(
    name: &str,
    files: &[(&str, &str)],
    module: &str,
) -> Result<Vec<(String, String)>, String> {
    subtree_findings(
        "async",
        name,
        files,
        module,
        async_exposure_subtree_findings,
    )
}

/// Just the finding strings, sorted — for cases where the module attribution rides inside the
/// finding string anyway.
pub(super) fn async_subtree_labels(
    name: &str,
    files: &[(&str, &str)],
    module: &str,
) -> Vec<String> {
    async_subtree(name, files, module)
        .unwrap()
        .into_iter()
        .map(|(finding, _module)| finding)
        .collect()
}

#[test]
pub(super) fn async_subtree_reacts_to_a_submodule_async_fn_the_seam_scope_misses() {
    // The crux this whole opt-in exists for. A `pub async fn` in a *submodule* is invisible to the
    // default seam scope (anchored at `crate`, it sees only crate-root items) — the latent false
    // negative dogfooding `sans_io_pure` on 璇璣 surfaced. The subtree scope catches it.
    let files = &[
        ("lib.rs", "pub mod net;\n"),
        ("net.rs", "pub async fn connect() {}\n"),
    ];
    // Default seam scope at `crate` misses it entirely…
    assert_eq!(
        async_findings("seam-misses-sub", files, "crate").unwrap(),
        Vec::<String>::new(),
    );
    // …the subtree scope reacts, attributing it to the submodule.
    assert_eq!(
        async_subtree("sub-reacts", files, "crate").unwrap(),
        [(
            "async fn crate::net::connect()".to_string(),
            "crate::net".to_string()
        )],
    );
}

#[test]
pub(super) fn async_subtree_includes_the_anchor_modules_own_seam_byte_identically() {
    // The anchor module's own async fn is still caught, and its finding string is byte-identical to
    // the single-module path — so enabling the opt-in on a seam-only boundary adds deeper findings
    // without re-identifying the seam ones (baseline stability).
    let files = &[
        ("lib.rs", "pub mod m;\n"),
        ("m.rs", "pub async fn own() {}\npub mod deep;\n"),
        ("m/deep.rs", "pub async fn nested() {}\n"),
    ];
    let seam = async_findings("seam-parity", files, "crate::m").unwrap();
    assert_eq!(seam, ["async fn crate::m::own()"]);
    let subtree = async_subtree_labels("subtree-parity", files, "crate::m");
    assert_eq!(
        subtree,
        [
            "async fn crate::m::deep::nested()",
            "async fn crate::m::own()",
        ],
    );
    // The seam finding appears verbatim in the subtree result.
    assert!(subtree.contains(&seam[0]));
}

#[test]
pub(super) fn async_subtree_and_seam_both_fail_loud_on_an_unrenderable_owner() {
    let files = &[(
        "lib.rs",
        "pub struct Arr<const N: usize>;\npub struct Marker;\nimpl Marker { pub async fn before() {} }\nimpl<const N: usize> Arr<{ N + 1 }> { pub async fn unrenderable() {} }\n",
    )];
    let seam = async_findings("const-generic-owner-parity-seam", files, "crate").unwrap_err();
    let subtree = async_subtree("const-generic-owner-parity-subtree", files, "crate").unwrap_err();
    assert!(seam.contains("without a positional fallback"), "{seam}");
    assert!(
        subtree.contains("without a positional fallback"),
        "{subtree}"
    );
    assert!(!seam.contains("_#") && !subtree.contains("_#"));
}

#[test]
pub(super) fn async_cfg_branches_never_share_an_unrenderable_owner_fallback() {
    let files = &[
        (
            "lib.rs",
            "#[cfg(feature = \"u\")]\npub mod m;\n#[cfg(feature = \"w\")]\n#[path = \"m_w.rs\"]\npub mod m;\n",
        ),
        (
            "m.rs",
            "pub struct Arr<const N: usize>;\nimpl<const N: usize> Arr<{ N + 1 }> { pub async fn run() {} }\n",
        ),
        (
            "m_w.rs",
            "pub struct Arr<const N: usize>;\nimpl<const N: usize> Arr<{ N + 2 }> { pub async fn run() {} }\n",
        ),
    ];
    let error = async_subtree("cfg-split-owner-fallback-collision", files, "crate::m").unwrap_err();
    assert!(error.contains("without a positional fallback"), "{error}");
    assert!(!error.contains("_#"), "{error}");
}

#[test]
pub(super) fn async_subtree_reacts_through_inline_and_nested_modules() {
    // Inline `mod`, file `mod`, and a grandchild all react, each attributed to its own module.
    let files = &[
        (
            "lib.rs",
            "pub mod outer { pub async fn a() {} pub mod middle; }\n",
        ),
        ("outer/middle.rs", "pub async fn b() {}\npub mod leaf;\n"),
        ("outer/middle/leaf.rs", "pub async fn c() {}\n"),
    ];
    assert_eq!(
        async_subtree_labels("nested", files, "crate"),
        [
            "async fn crate::outer::a()",
            "async fn crate::outer::middle::b()",
            "async fn crate::outer::middle::leaf::c()",
        ],
    );
}

#[test]
pub(super) fn async_subtree_anchored_at_an_inline_module_follows_its_own_further_path_child() {
    // rustc ground truth (verified with a real rustc build): `#[path = "moved/leaf.rs"]` written
    // inside an INLINE `mod outer { … }` accumulates outer's own directory as the base — the file
    // actually compiles at `outer/moved/leaf.rs`, never `moved/leaf.rs` (which would sit beside
    // lib.rs itself). `walk_subtree_modules` used to re-derive the anchor's own `#[path]`-base as
    // `file.parent()` — correct for a file-form anchor, but wrong for an INLINE anchor (the inline
    // body stays in the *enclosing* file, whose own directory is not the inline module's
    // accumulated one) — silently substituting the wrong base for anything the subtree walk
    // itself needs to resolve a further `#[path]` from. `resolve_module_root`'s own returned
    // `path_base` (this fix) is used directly instead of being re-derived.
    let files = &[
        (
            "lib.rs",
            "pub mod outer {\n    #[path = \"moved/leaf.rs\"]\n    pub mod leaf;\n}\n",
        ),
        ("outer/moved/leaf.rs", "pub async fn seam() {}\n"),
    ];
    assert_eq!(
        async_subtree_labels("inline-anchor-path-child", files, "crate::outer"),
        ["async fn crate::outer::leaf::seam()"],
    );
}

#[test]
pub(super) fn async_subtree_walks_every_branch_of_a_cfg_split_anchor_not_just_the_first() {
    // rustc ground truth (verified with a real rustc build under either single-feature config):
    // `#[cfg(feature = "u")] pub mod foo;` (flat, own directory src/) paired with
    // `#[cfg(feature = "w")] #[path = "win/foo.rs"] pub mod foo;` (own directory src/win/) is the
    // standard per-platform shim — each arm plainly declares its OWN `pub mod bar;`, resolving to
    // a DIFFERENT real file (src/foo/bar.rs vs src/win/bar.rs). `resolve_module_root` correctly
    // unions both arms' items, but `walk_subtree_modules` used to thread only the FIRST arm's own
    // directory pair through to resolve those unioned items' own children — so the second arm's
    // `bar` silently resolved against the wrong directory and its own async fn was never observed.
    let files = &[
        (
            "lib.rs",
            "#[cfg(feature = \"u\")]\npub mod foo;\n#[cfg(feature = \"w\")]\n#[path = \"win/foo.rs\"]\npub mod foo;\n",
        ),
        ("foo.rs", "pub mod bar;\n"),
        ("foo/bar.rs", "pub async fn unix_leaf() {}\n"),
        ("win/foo.rs", "pub mod bar;\n"),
        ("win/bar.rs", "pub async fn win_leaf() {}\n"),
    ];
    assert_eq!(
        async_subtree_labels("cfg-split-anchor-both-branches", files, "crate::foo"),
        [
            "async fn crate::foo::bar::unix_leaf()",
            "async fn crate::foo::bar::win_leaf()",
        ],
    );
}

#[test]
pub(super) fn async_subtree_violations_name_each_branchs_own_file_not_a_shared_module_string_cache()
{
    // Round-5 finding: async_exposure_subtree_findings correctly emits one finding per branch
    // (fixed above), both tagged with the identical module string "crate::foo::bar" (a legitimate
    // cfg-split: unix_leaf lives in foo/bar.rs, win_leaf in win/bar.rs). Before this redesign,
    // push_multi_module_violations resolved each finding's file via per_finding_file, a cache
    // keyed ONLY by that module string — so the first finding processed populated the cache with
    // one branch's file, and the second finding (from the OTHER branch) silently reused it. Every
    // multi-module finding now pairs with the real file its own branch was resolved from (from
    // the subtree walker itself), so each violation's file must name its own real branch.
    let (metadata, _fixture) = fixture_metadata(
        "cfg-split-anchor-file-attribution",
        &[
            (
                "lib.rs",
                "#[cfg(feature = \"u\")]\npub mod foo;\n#[cfg(feature = \"w\")]\n#[path = \"win/foo.rs\"]\npub mod foo;\n",
            ),
            ("foo.rs", "pub mod bar;\n"),
            ("foo/bar.rs", "pub async fn unix_leaf() {}\n"),
            ("win/foo.rs", "pub mod bar;\n"),
            ("win/bar.rs", "pub async fn win_leaf() {}\n"),
        ],
    );
    let boundary = AsyncExposureBoundary::in_crate("x")
        .module("crate::foo")
        .must_not_expose_async_fn()
        .including_submodules()
        .because("each branch's finding must name its own real file");
    let mut violations = Vec::new();
    check_async_exposure_boundary(&metadata, &boundary, &mut violations).unwrap();
    assert_eq!(violations.len(), 2, "{violations:?}");
    let mut by_finding: std::collections::BTreeMap<String, &str> = Default::default();
    for v in &violations {
        by_finding.insert(
            v.finding.clone(),
            v.file
                .as_deref()
                .expect("a subtree finding carries its file"),
        );
    }
    assert!(
        by_finding["async fn crate::foo::bar::unix_leaf()"].ends_with("foo/bar.rs"),
        "unix_leaf must name foo/bar.rs: {by_finding:?}"
    );
    assert!(
        by_finding["async fn crate::foo::bar::win_leaf()"].ends_with("win/bar.rs"),
        "win_leaf must name win/bar.rs, never foo/bar.rs (a shared-cache misattribution): {by_finding:?}"
    );
}

#[test]
pub(super) fn async_subtree_does_not_duplicate_a_file_shared_by_two_plain_cfg_siblings() {
    // rustc ground truth: `#[cfg(feature = "u")] pub mod foo;` and `#[cfg(feature = "w")] pub mod
    // foo;` (both PLAIN, no #[path]) are two mutually-exclusive declarations of the SAME name that
    // resolve to the IDENTICAL real file (foo.rs) — neither build ever compiles it twice.
    // descend()'s file-form search used to push one branch per matching declaration regardless of
    // whether they resolved to the same file, so foo.rs's own async fn was observed (and reported)
    // twice.
    let files = &[
        (
            "lib.rs",
            "#[cfg(feature = \"u\")]\npub mod foo;\n#[cfg(feature = \"w\")]\npub mod foo;\n",
        ),
        ("foo.rs", "pub async fn seam() {}\n"),
    ];
    assert_eq!(
        async_subtree_labels("plain-cfg-siblings-one-file", files, "crate::foo"),
        ["async fn crate::foo::seam()"],
    );
}

#[test]
pub(super) fn async_subtree_scopes_to_the_anchored_subtree_not_the_whole_crate() {
    // Anchored at `crate::a`, an async fn under `crate::a` reacts; a sibling `crate::c` does not —
    // the subtree is bounded by the anchor, not the crate.
    let files = &[
        ("lib.rs", "pub mod a;\npub mod c;\n"),
        ("a.rs", "pub async fn af() {}\npub mod b;\n"),
        ("a/b.rs", "pub async fn bf() {}\n"),
        ("c.rs", "pub async fn cf() {}\n"),
    ];
    assert_eq!(
        async_subtree_labels("bounded", files, "crate::a"),
        ["async fn crate::a::af()", "async fn crate::a::b::bf()"],
    );
}

#[test]
pub(super) fn async_subtree_tolerates_a_cfg_gated_fileless_submodule() {
    // A `#[cfg]`-gated module with no file when the feature is off is tolerated (a stated bound),
    // not a scan error; the present modules still react.
    let files = &[
        (
            "lib.rs",
            "#[cfg(feature = \"absent\")]\npub mod gated;\npub mod present;\n",
        ),
        ("present.rs", "pub async fn here() {}\n"),
    ];
    assert_eq!(
        async_subtree_labels("cfg-tolerated", files, "crate"),
        ["async fn crate::present::here()"],
    );
}

#[test]
pub(super) fn async_subtree_errors_on_a_non_cfg_missing_submodule() {
    // A non-`#[cfg]` `mod x;` with no file is a scan error (exit 2) — "cannot judge", never a
    // silent pass that would under-react.
    let files = &[("lib.rs", "pub mod gone;\n")];
    assert!(async_subtree("non-cfg-missing", files, "crate").is_err());
}

/// A cfg_attr(path)-hidden submodule is observed, whichever candidate file exists — the identical
/// `resolve_child_modules`/`walk_subtree_modules` mechanism fixed by
/// `hunyi-cfg-attr-path-module-loss` for `scan_crate`'s own consumers. Not named in that change's
/// own commit message (a documentation gap a round-3 adversarial review found and closed) but the
/// same shared walker, independently reproduced here before being counted as fixed.
#[test]
pub(super) fn async_subtree_reacts_through_a_cfg_attr_wrapped_path_submodule() {
    let files = &[
        (
            "lib.rs",
            "#[cfg_attr(any(), path = \"never.rs\")]\npub mod net;\n",
        ),
        ("net.rs", "pub async fn connect() {}\n"),
    ];
    assert_eq!(
        async_subtree("cfg-attr-path", files, "crate").unwrap(),
        [(
            "async fn crate::net::connect()".to_string(),
            "crate::net".to_string()
        )],
    );
}

#[test]
pub(super) fn async_subtree_distinguishes_same_named_async_methods_across_modules() {
    // Cross-module dedup safety (the invariant `push_multi_module_violations` rests on): it flattens
    // findings to identity `(anchor, rule, finding)`, discarding the enclosing module — so two
    // same-named inherent async methods in *different* submodules stay distinct ONLY because the
    // finding string carries the module-qualified owner. If that owner ever lost its module prefix,
    // baselining one would mask the other (a false negative). This pins it.
    let files = &[
        ("lib.rs", "pub mod a;\npub mod b;\n"),
        (
            "a.rs",
            "pub struct S;\nimpl S { pub async fn run(&self) {} }\n",
        ),
        (
            "b.rs",
            "pub struct S;\nimpl S { pub async fn run(&self) {} }\n",
        ),
    ];
    assert_eq!(
        async_subtree_labels("cross-mod-owners", files, "crate"),
        [
            "async fn <crate::a::S>::run(&self)",
            "async fn <crate::b::S>::run(&self)",
        ],
    );
}

#[test]
pub(super) fn async_subtree_does_not_observe_a_body_nested_module() {
    // A `mod` declared inside a fn body is not part of the public module tree (its items are not
    // reachable as `crate::…`), so the subtree walk — which descends the public module tree, not fn
    // bodies — does not observe it. A stated bound: it is not public API, so async-exposure (which
    // governs the *public* seam) makes no claim about it, rather than silently asserting cleanliness.
    let files = &[(
        "lib.rs",
        "pub fn outer() { mod inner { pub async fn hidden() {} } }\n",
    )];
    assert_eq!(
        async_subtree_labels("body-nested", files, "crate"),
        Vec::<String>::new(),
    );
}

#[test]
pub(super) fn dyn_in_public_return_param_and_field_react() {
    assert_eq!(
        dyn_mod(
            "ret",
            "pub fn connect() -> Box<dyn crate::Port> { todo!() }\n"
        )
        .unwrap(),
        ["dyn crate::Port exposed by fn crate::m::connect"]
    );
    assert_eq!(
        dyn_mod(
            "param",
            "pub fn drive(x: &dyn crate::Port) { let _ = x; }\n"
        )
        .unwrap(),
        ["dyn crate::Port exposed by fn crate::m::drive"]
    );
    assert_eq!(
        dyn_mod("field", "pub struct S { pub p: Box<dyn crate::Port> }\n").unwrap(),
        ["dyn crate::Port exposed by field crate::m::S::p"]
    );
}

#[test]
pub(super) fn dyn_reacts_at_any_nesting_depth() {
    assert_eq!(
        dyn_mod(
            "vec",
            "pub fn all() -> Vec<Box<dyn crate::Port>> { todo!() }\n"
        )
        .unwrap(),
        ["dyn crate::Port exposed by fn crate::m::all"]
    );
    assert_eq!(
        dyn_mod(
            "opt",
            "pub fn maybe(x: Option<&dyn crate::Port>) { let _ = x; }\n"
        )
        .unwrap(),
        ["dyn crate::Port exposed by fn crate::m::maybe"]
    );
    // Nested inside an otherwise-static `impl Trait` return — still exposed to the caller.
    assert_eq!(
        dyn_mod(
            "impl-iter",
            "pub fn ports() -> impl Iterator<Item = Box<dyn crate::Port>> { std::iter::empty() }\n"
        )
        .unwrap(),
        ["dyn crate::Port exposed by fn crate::m::ports"]
    );
}

#[test]
pub(super) fn impl_trait_with_no_dyn_node_is_clean() {
    let out = dyn_mod(
        "impl-trait",
        "pub fn port() -> impl crate::Port { todo!() }\n",
    )
    .unwrap();
    assert!(out.is_empty(), "impl Trait carries no dyn node: {out:?}");
}

#[test]
pub(super) fn dyn_in_const_static_trait_method_assoc_default_and_where_react() {
    assert_eq!(
        dyn_mod("const", "pub const C: &dyn crate::Port = todo!();\n").unwrap(),
        ["dyn crate::Port exposed by const crate::m::C"]
    );
    assert_eq!(
        dyn_mod("static", "pub static S: &dyn crate::Port = todo!();\n").unwrap(),
        ["dyn crate::Port exposed by static crate::m::S"]
    );
    assert_eq!(
        dyn_mod(
            "trait-method",
            "pub trait Service { fn port(&self) -> Box<dyn crate::Port>; }\n"
        )
        .unwrap(),
        ["dyn crate::Port exposed by fn trait crate::m::Service::port"]
    );
    assert_eq!(
        dyn_mod(
            "assoc-default",
            "pub trait Service { type Out = Box<dyn crate::Port>; }\n"
        )
        .unwrap(),
        ["dyn crate::Port exposed by type trait crate::m::Service::Out"]
    );
    assert_eq!(
        dyn_mod(
            "where",
            "pub fn run<T>() where Box<dyn crate::Port>: Into<T> { todo!() }\n"
        )
        .unwrap(),
        ["dyn crate::Port exposed by fn crate::m::run"]
    );
}

#[test]
pub(super) fn dyn_in_an_inherent_impl_public_assoc_const_reacts() {
    // The dyn collector's inherent-impl arm now observes public associated `const`/`type`
    // positions (parity with the signature-coupling collector, which gained them this release), so a
    // `dyn` written in an inherent-impl `pub const` type reacts — it did not before.
    assert_eq!(
        dyn_mod(
            "inherent-assoc-const",
            "pub struct Config;\nimpl Config { pub const DEFAULT: &dyn crate::Port = todo!(); }\n",
        )
        .unwrap(),
        ["dyn crate::Port exposed by const <crate::m::Config>::DEFAULT"]
    );
}

#[test]
pub(super) fn public_alias_target_reacts_but_named_alias_is_not_expanded() {
    // The public alias item's own target exposes dyn → reacts at the alias.
    assert_eq!(
        dyn_mod("alias-item", "pub type Handler = Box<dyn crate::Port>;\n").unwrap(),
        ["dyn crate::Port exposed by type crate::m::Handler"]
    );
    // A public fn naming a *private* alias: the alias is not expanded (stated bound), and a
    // private alias is not itself exposed — so the dyn escapes (the documented bound), the
    // only finding being none.
    let out = dyn_mod(
        "alias-named",
        "type Handler = Box<dyn crate::Port>;\npub fn make() -> Handler { todo!() }\n",
    )
    .unwrap();
    assert!(
        out.is_empty(),
        "named private alias is not expanded: {out:?}"
    );
}

#[test]
pub(super) fn internal_dyn_is_structurally_clean() {
    let out = dyn_mod(
        "internal",
        "fn helper() -> Box<dyn crate::Port> { todo!() }\nstruct Private { p: Box<dyn crate::Port> }\n",
    )
    .unwrap();
    assert!(out.is_empty(), "internal dyn is never exposed: {out:?}");
}

#[test]
pub(super) fn dyn_with_multiple_bounds_renders_stably() {
    assert_eq!(
        dyn_mod(
            "bounds",
            "pub fn f() -> Box<dyn crate::Port + Send> { todo!() }\n"
        )
        .unwrap(),
        ["dyn crate::Port + Send exposed by fn crate::m::f"]
    );
}

#[test]
pub(super) fn distinct_closures_and_nested_dyns_do_not_collide_into_one_finding() {
    // The boxed-closure family must render its full shape, not a degenerate placeholder —
    // else two distinct exposed `dyn` collapse to one finding and a new one is masked by a
    // baselined one (the one forbidden bug). `Fn`/`FnMut` differ, so two findings.
    let out = dyn_mod(
        "closures",
        "pub fn a(cb: Box<dyn Fn(i32) -> i32>) { let _ = cb; }\n\
         pub fn b(cb: Box<dyn FnMut(String) -> bool>) { let _ = cb; }\n",
    )
    .unwrap();
    assert_eq!(
        out,
        [
            "dyn Fn(i32) -> i32 exposed by fn crate::m::a",
            "dyn FnMut(String) -> bool exposed by fn crate::m::b"
        ]
    );
    // A dyn nested inside another dyn's generic argument: BOTH are exposed dynamic
    // dispatch, so both react (any-depth node presence) — distinct, non-colliding findings.
    assert_eq!(
        dyn_mod(
            "nested",
            "pub fn f() -> Box<dyn crate::Foo<Box<dyn crate::Bar>>> { todo!() }\n"
        )
        .unwrap(),
        [
            "dyn crate::Bar exposed by fn crate::m::f",
            "dyn crate::Foo<Box<dyn crate::Bar>> exposed by fn crate::m::f"
        ]
    );
    // Associated-type bindings (`Iterator<Item = …>`, the most common assoc-bound dyn) keep
    // their payload — distinct item types stay distinct findings, not `dyn Iterator<_>`.
    let out = dyn_mod(
        "assoc",
        "pub fn a(x: Box<dyn Iterator<Item = u8>>) { let _ = x; }\n\
         pub fn b(x: Box<dyn Iterator<Item = u16>>) { let _ = x; }\n",
    )
    .unwrap();
    assert_eq!(
        out,
        [
            "dyn Iterator<Item = u16> exposed by fn crate::m::b",
            "dyn Iterator<Item = u8> exposed by fn crate::m::a"
        ]
    );
    // Macro-typed and fn-pointer generic args render by name/shape, not a shared `dyn _`.
    let out = dyn_mod(
        "macro-fnptr",
        "pub fn a(x: Box<dyn crate::Foo<fn(i32)>>) { let _ = x; }\n\
         pub fn b(x: Box<dyn crate::Foo<fn(u8)>>) { let _ = x; }\n",
    )
    .unwrap();
    assert_eq!(
        out,
        [
            "dyn crate::Foo<fn(i32)> exposed by fn crate::m::a",
            "dyn crate::Foo<fn(u8)> exposed by fn crate::m::b"
        ]
    );
}

#[test]
pub(super) fn same_shape_at_two_seams_stays_two_findings() {
    // The closed collision false-negative: two distinct public seams exposing the SAME dyn
    // shape must stay two findings, not collapse to one — else a new leak is masked by a
    // baselined one. Seam-qualification keeps them distinct.
    let out = dyn_mod(
        "two-seams",
        "pub fn a() -> Box<dyn crate::infra::Port> { todo!() }\n\
         pub fn b() -> Box<dyn crate::infra::Port> { todo!() }\n",
    )
    .unwrap();
    assert_eq!(
        out,
        [
            "dyn crate::infra::Port exposed by fn crate::m::a",
            "dyn crate::infra::Port exposed by fn crate::m::b"
        ],
        "the same dyn shape at two seams must not collapse to one finding",
    );
    // The same guarantee for signature-coupling: two fns exposing the SAME forbidden type
    // stay two findings, one per seam.
    let out = findings(
        "two-seams-sig",
        &[
            ("lib.rs", "pub mod domain;\n"),
            (
                "domain.rs",
                "pub fn a() -> crate::infra::DbPool { todo!() }\n\
                 pub fn b() -> crate::infra::DbPool { todo!() }\n",
            ),
        ],
        "crate::domain",
        &["crate::infra"],
    )
    .unwrap();
    assert_eq!(
        out,
        [
            "crate::infra::DbPool exposed by fn crate::domain::a",
            "crate::infra::DbPool exposed by fn crate::domain::b"
        ],
        "the same forbidden type at two seams must not collapse to one finding",
    );
}

#[test]
pub(super) fn the_dyn_trait_builder_carries_anchor_and_severity() {
    let b = DynTraitBoundary::in_crate("app")
        .module("crate::core")
        .must_not_expose_dyn()
        .warn()
        .because("the core seam is statically dispatched");
    assert_eq!(b.crate_package(), "app");
    assert_eq!(b.module(), "crate::core");
    assert_eq!(b.severity(), Severity::Warn);
    assert_eq!(b.reason(), "the core seam is statically dispatched");
}

#[test]
pub(super) fn dyn_unknown_module_is_a_constitution_error() {
    let err = dyn_findings(
        "unknown",
        &[("lib.rs", "pub mod m;\n"), ("m.rs", "// nothing\n")],
        "crate::ghost",
    )
    .unwrap_err();
    assert_eq!(err, unknown_module_error("crate::ghost", "x"));
}
