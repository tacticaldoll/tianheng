use super::super::*;
use super::helpers::*;
// --- unsafe confinement --------------------------------------------------

pub(super) fn unsafe_labels(
    name: &str,
    files: &[(&str, &str)],
    allowed: &[&str],
) -> Result<Vec<String>, String> {
    let tree = TempSrcTree::new(&format!("unsafe-{name}"));
    tree.write_all(files);
    let allowed: Vec<String> = allowed.iter().map(|a| a.to_string()).collect();
    unsafe_findings(tree.src(), &tree.root(), &allowed, "x").map(|fs| {
        fs.into_iter()
            .map(|(finding, _, _)| finding.to_string())
            .collect()
    })
}

pub(super) fn unsafe_keys(name: &str, source: &str) -> Result<Vec<StructuredFactIdentity>, String> {
    let tree = TempSrcTree::new(&format!("unsafe-keys-{name}"));
    tree.write_all(&[("lib.rs", "pub mod net;\n"), ("net.rs", source)]);
    unsafe_findings(tree.src(), &tree.root(), &["crate::ffi".to_string()], "x").map(|findings| {
        findings
            .into_iter()
            .map(|(fact, _, _)| fact.into_finding("app", "src/lib.rs").key().clone())
            .collect()
    })
}

/// A `cfg_attr`-wrapped `#[path]` on an INLINE module must never drop the body: `#[path]` has no
/// effect on an inline `mod` at all (rustc always compiles the body regardless of the wrapped
/// predicate), so treating the attribute as a skip bound here dropped the whole subtree's
/// observation — closes the false negative where 圭表 and 漏刻 both reacted on the identical file
/// and 渾儀 alone stayed silent.
#[test]
pub(super) fn cfg_attr_wrapped_path_on_an_inline_module_is_still_observed() {
    let out = unsafe_labels(
        "cfg-attr-inline",
        &[(
            "lib.rs",
            "#[cfg_attr(windows, path = \"x.rs\")]\npub mod inner {\n    pub fn f() { unsafe {} }\n}\n",
        )],
        &["crate::nowhere"],
    )
    .unwrap();
    assert_eq!(out, ["unsafe block in crate::inner"]);
}

/// A `cfg_attr`-wrapped `#[path]` FILE module's `cfg_attr` target is read when it exists and the
/// conventional file does not — the predicate may genuinely select the target on some build, and
/// `cfg_attr` never removes the `mod` item the way a bare `#[cfg]` does, so an absent conventional
/// file here is not itself an error.
#[test]
pub(super) fn cfg_attr_wrapped_path_target_is_read_when_the_conventional_file_is_absent() {
    let out = unsafe_labels(
        "cfg-attr-target-only",
        &[
            (
                "lib.rs",
                "#[cfg_attr(windows, path = \"win.rs\")]\npub mod imp;\n",
            ),
            ("win.rs", "pub fn f() { unsafe {} }\n"),
        ],
        &["crate::nowhere"],
    )
    .unwrap();
    assert_eq!(out, ["unsafe block in crate::imp"]);
}

/// Neither the `cfg_attr` target nor the conventional file existing at all, with no other
/// cfg-conditional gate on the declaration, is a genuine scan error (exit 2) — `cfg_attr` never
/// removes the `mod` item, so on every configuration SOME file must back it. Never a silent pass.
#[test]
pub(super) fn cfg_attr_wrapped_path_with_neither_candidate_present_fails_loud() {
    let err = unsafe_labels(
        "cfg-attr-both-absent",
        &[(
            "lib.rs",
            "#[cfg_attr(windows, path = \"win.rs\")]\npub mod imp;\n",
        )],
        &["crate::nowhere"],
    )
    .unwrap_err();
    assert!(
        err.contains("could not be located"),
        "neither candidate existing must fail loud, not silently pass: {err}"
    );
}

/// A `cfg_attr`-wrapped `#[path]` FILE module's conventional file is read even when the wrapped
/// predicate is always-false (`any()`) — the module declaration is never removed, so the
/// conventional file is what every build actually compiles here, and it must not vanish from the
/// crate-wide scan merely because a `cfg_attr(path)` attribute is present. Closes the false
/// negative where 圭表 and 漏刻 both reacted on the identical file and 渾儀 alone stayed silent.
#[test]
pub(super) fn cfg_attr_wrapped_path_conventional_file_is_read_when_the_predicate_is_always_false() {
    let out = unsafe_labels(
        "cfg-attr-file",
        &[
            (
                "lib.rs",
                "#[cfg_attr(any(), path = \"never.rs\")]\npub mod imp;\n",
            ),
            ("imp.rs", "pub fn f() { unsafe {} }\n"),
        ],
        &["crate::nowhere"],
    )
    .unwrap();
    assert_eq!(out, ["unsafe block in crate::imp"]);
}

#[test]
pub(super) fn unsafe_identity_survives_reorder_and_unrelated_insertion() {
    let before = unsafe_keys(
        "reorder-before",
        "pub struct Api;\nunsafe impl Send for Api {}\n",
    )
    .unwrap();
    let after = unsafe_keys(
        "reorder-after",
        "pub const UNRELATED: usize = 1;\npub struct Api;\nunsafe impl Send for Api {}\n",
    )
    .unwrap();
    assert_eq!(before, after);
}

#[test]
pub(super) fn unrenderable_unsafe_owner_fails_loud_without_an_ordinal_identity() {
    let error = unsafe_keys(
        "unrenderable-owner",
        "pub struct Arr<const N: usize>;\npub const N: usize = 1;\nunsafe impl Send for Arr<{ N + 1 }> {}\n",
    )
    .unwrap_err();
    // The refusal names WHAT was met, not only what is not invented: this self type carries a
    // const-generic expression with no supported rendering, which is a different fact from a path that
    // resolves nowhere and from a `#[cfg]`-collided alias, and all three used to reach one sentence.
    assert!(
        error.contains("its syntax has no supported rendering"),
        "{error}"
    );
    assert!(
        error.contains("no positional fallback is invented for it"),
        "{error}"
    );
    assert!(!error.contains("_#"), "{error}");
}

/// A `#[cfg]`-collided alias names its OWN cause, not the one an unrenderable type has.
///
/// **Three facts reached one sentence.** An owner could not be named because its path resolved to no
/// candidate, because two mutually-exclusive `#[cfg]` branches bound one alias to different types, or
/// because its syntax has no supported rendering — and every one of them emitted *cannot identify … without
/// a positional fallback*, which names the policy rather than what was met. The refusal is right in all
/// three; the sentence sends an adopter to the wrong place in two of them, and there is nothing in the
/// emitted text to grep for the one they have.
///
/// The verdict does not move: refusing was correct before and is correct now. What moves is whether the
/// refusal can be acted on.
///
/// Negative run, before the cause was a named value:
///
/// ```text
/// the refusal must name the collision it met, got: cannot identify unsafe impl self type in crate::net without a positional fallback
/// ```
///
/// The collision is not mentioned at all — the sentence names what is not invented and stops there.
#[test]
pub(super) fn a_cfg_collided_alias_names_its_own_cause() {
    let error = unsafe_keys(
        "cfg-collided-alias",
        "#[cfg(feature = \"x\")]\nuse crate::a::T as Alias;\n\
         #[cfg(not(feature = \"x\"))]\nuse crate::b::T as Alias;\n\
         unsafe impl Send for Alias {}\n",
    )
    .unwrap_err();
    assert!(
        error.contains("bind that alias to different types"),
        "the refusal must name the collision it met, got: {error}"
    );
    assert!(
        !error.contains("no supported rendering"),
        "an alias that renders perfectly must not be reported as unrenderable, got: {error}"
    );
}

#[test]
pub(super) fn unsafe_production_violation_separates_target_rule_and_fact_roles() {
    let (metadata, _fixture) = fixture_metadata(
        "unsafe-identity",
        &[
            ("lib.rs", "pub mod net;\npub mod ffi;\n"),
            ("net.rs", "pub unsafe fn decode() {}\n"),
            ("ffi.rs", ""),
        ],
    );
    let boundary = UnsafeBoundary::in_crate("x")
        .only_under(["crate::raw", "crate::ffi"])
        .because("unsafe stays behind the audited adapter");
    let mut violations = Vec::new();
    check_unsafe_boundary(&metadata, &boundary, &mut violations).unwrap();
    assert_eq!(violations.len(), 1);

    let id = violations[0].id();
    assert_eq!(id.target(), "x");
    let rule = id.rule_key();
    assert_eq!(rule.rule_type(), "tianheng.rule/hunyi/unsafe-confinement");
    assert_eq!(
        rule.fields().collect::<Vec<_>>(),
        vec![("allowed", "[\"crate::ffi\",\"crate::raw\"]")]
    );
    let fact = id.fact();
    assert_eq!(fact.fact_type(), "tianheng.fact/hunyi/unsafe-site");
    assert_eq!(fact.shape(), "unsafe-free-function");
    assert_eq!(
        fact.fields().collect::<Vec<_>>(),
        vec![
            ("module", "crate::net"),
            ("name", "decode"),
            ("unit", "lib.rs"),
        ]
    );
}

#[test]
pub(super) fn unsafe_block_outside_subtree_reacts() {
    let out = unsafe_labels(
        "block",
        &[
            ("lib.rs", "pub mod ffi;\npub mod net;\n"),
            (
                "ffi.rs",
                "pub fn ok() { unsafe { core::ptr::null::<u8>(); } }\n",
            ),
            (
                "net.rs",
                "pub fn f() { unsafe { core::ptr::null::<u8>(); } }\n",
            ),
        ],
        &["crate::ffi"],
    )
    .unwrap();
    assert_eq!(
        out,
        ["unsafe block in crate::net"],
        "a block outside the subtree reacts; one under it is clean: {out:?}"
    );
}

#[test]
pub(super) fn unsafe_fn_impl_trait_extern_outside_react() {
    let out = unsafe_labels(
        "kinds",
        &[
            ("lib.rs", "pub mod ffi;\npub mod net;\n"),
            ("ffi.rs", "\n"),
            (
                "net.rs",
                "pub unsafe trait Zeroable {}\npub unsafe fn decode() {}\nunsafe impl Zeroable for u8 {}\nunsafe extern \"C\" { fn c(); }\n",
            ),
        ],
        &["crate::ffi"],
    )
    .unwrap();
    assert_eq!(
        out,
        [
            "unsafe extern block in crate::net",
            "unsafe fn decode in crate::net",
            "unsafe impl Zeroable for u8 in crate::net",
            "unsafe trait Zeroable in crate::net",
        ],
        "every unsafe-keyword site outside the subtree reacts: {out:?}"
    );
}

#[test]
pub(super) fn unsafe_under_the_subtree_is_clean() {
    let out = unsafe_labels(
        "clean",
        &[
            ("lib.rs", "pub mod ffi;\n"),
            ("ffi.rs", "pub mod raw;\npub unsafe fn a() {}\n"),
            (
                "ffi/raw.rs",
                "pub fn b() { unsafe { core::ptr::null::<u8>(); } }\n",
            ),
        ],
        &["crate::ffi"],
    )
    .unwrap();
    assert!(
        out.is_empty(),
        "unsafe at the subtree and beneath it is clean: {out:?}"
    );
}

#[test]
pub(super) fn empty_allowed_set_is_a_constitution_error() {
    let err = unsafe_labels("empty", &[("lib.rs", "pub fn f() { unsafe {} }\n")], &[]).unwrap_err();
    assert!(
        err.contains("forbid(unsafe_code)"),
        "empty only_under points at #![forbid(unsafe_code)]: {err}"
    );
}

#[test]
pub(super) fn crate_root_allowed_set_is_a_constitution_error() {
    let err = unsafe_labels("root", &[("lib.rs", "pub fn f() {}\n")], &["crate"]).unwrap_err();
    assert!(err.contains("crate root"), "{err}");
}

/// An `allowed_locations` entry with an empty `::`-segment (leading, trailing, or doubled `::`)
/// must be a constitution error — never silently pass through into `matches_allowed`. Before the
/// fix, a malformed entry never matched any real module location, so a genuinely-confined
/// `unsafe` site was reported as a spurious violation instead of naming the actual typo in the
/// declaration (reproduced directly against this pure heart: `["crate::ffi::"]` on an `unsafe fn`
/// genuinely under `crate::ffi` produced a finding, not a clean error, before this test was
/// written to pin the fix).
#[test]
pub(super) fn unsafe_confinement_rejects_a_malformed_colon_allowed_location() {
    let files: &[(&str, &str)] = &[
        ("lib.rs", "pub mod ffi;\n"),
        ("ffi.rs", "pub unsafe fn a() {}\n"),
    ];
    for bad in ["::crate::ffi", "crate::ffi::", "crate::ffi::::sub"] {
        let err = unsafe_labels("malformed-allowed", files, &[bad]).unwrap_err();
        assert!(
            err.contains(bad),
            "constitution error must name the malformed allowed entry {bad:?}: {err}"
        );
    }
    // The empty string itself is also a malformed allowed entry — see must_not_expose's
    // identical note; this shares the same `validate_path_operands` guard.
    let empty_err = unsafe_labels("malformed-allowed-empty", files, &[""]).unwrap_err();
    assert!(
        empty_err.contains("is empty"),
        "constitution error must flag the empty allowed entry: {empty_err}"
    );
    // Control: the well-formed spelling for the identical, genuinely-confined site still passes
    // clean — the rejection above is a spelling gate, never a confinement regression.
    let clean = unsafe_labels("malformed-allowed-control", files, &["crate::ffi"]).unwrap();
    assert!(
        clean.is_empty(),
        "a well-formed allowed entry must still confine the genuinely-placed unsafe site: {clean:?}"
    );
}

#[test]
pub(super) fn unsafe_blocks_dedup_per_module() {
    let out = unsafe_labels(
        "dedup",
        &[
            ("lib.rs", "pub mod net;\n"),
            (
                "net.rs",
                "pub fn f() { unsafe {} unsafe {} }\npub fn g() { unsafe {} }\n",
            ),
        ],
        &["crate::ffi"],
    )
    .unwrap();
    assert_eq!(
        out,
        ["unsafe block in crate::net"],
        "N blocks in one module dedup to one stable finding: {out:?}"
    );
}

#[test]
pub(super) fn two_unsafe_impls_of_different_traits_stay_distinct() {
    let out = unsafe_labels(
        "impls",
        &[
            ("lib.rs", "pub mod net;\n"),
            (
                "net.rs",
                "pub struct Foo;\nunsafe impl Send for Foo {}\nunsafe impl Sync for Foo {}\n",
            ),
        ],
        &["crate::ffi"],
    )
    .unwrap();
    assert_eq!(
        out,
        [
            "unsafe impl Send for Foo in crate::net",
            "unsafe impl Sync for Foo in crate::net",
        ],
        "the trait is in the finding, so two unsafe impls do not collapse: {out:?}"
    );
}

#[test]
pub(super) fn two_unsafe_impls_of_one_trait_for_different_types_stay_distinct() {
    // Same trait, different self type: the finding is owner-qualified, so neither masks the other.
    // Were the self type omitted, a baseline of the first would silently accept the second — a
    // false negative (a new out-of-subtree `unsafe` site passing unobserved).
    let out = unsafe_labels(
        "impls-same-trait",
        &[
            ("lib.rs", "pub mod net;\n"),
            (
                "net.rs",
                "pub struct Foo;\npub struct Bar;\nunsafe impl Send for Foo {}\nunsafe impl Send for Bar {}\n",
            ),
        ],
        &["crate::ffi"],
    )
    .unwrap();
    assert_eq!(
        out,
        [
            "unsafe impl Send for Bar in crate::net",
            "unsafe impl Send for Foo in crate::net",
        ],
        "the self type is in the finding, so same-trait impls for different types do not collapse: {out:?}"
    );
}

#[test]
pub(super) fn two_same_named_unsafe_fns_on_different_owners_stay_distinct() {
    // Same method name, different inherent-impl self type: the finding must be owner-qualified,
    // else a baseline of the first silently accepts the second — a false negative (a new
    // out-of-subtree `unsafe` site passing unobserved). The unsafe-fn analogue of
    // `two_unsafe_impls_of_one_trait_for_different_types_stay_distinct`.
    let out = unsafe_labels(
        "unsafe-fns-same-name",
        &[
            ("lib.rs", "pub mod net;\n"),
            (
                "net.rs",
                "pub struct Foo;\npub struct Bar;\nimpl Foo { unsafe fn m(&self) {} }\nimpl Bar { unsafe fn m(&self) {} }\n",
            ),
        ],
        &["crate::ffi"],
    )
    .unwrap();
    assert_eq!(
        out,
        [
            "unsafe fn Bar::m in crate::net",
            "unsafe fn Foo::m in crate::net",
        ],
        "same-named unsafe fns on different owners must not collapse: {out:?}"
    );
}

#[test]
pub(super) fn two_same_named_unsafe_trait_fns_stay_distinct() {
    // Two traits in one module each declaring `unsafe fn m` must stay distinct findings, qualified
    // by the declaring trait — else a baseline of the first masks the second (a false negative).
    let out = unsafe_labels(
        "unsafe-trait-fns",
        &[
            ("lib.rs", "pub mod net;\n"),
            (
                "net.rs",
                "pub trait A { unsafe fn m(&self); }\npub trait B { unsafe fn m(&self); }\n",
            ),
        ],
        &["crate::ffi"],
    )
    .unwrap();
    assert_eq!(
        out,
        [
            "unsafe fn A::m in crate::net",
            "unsafe fn B::m in crate::net"
        ],
        "trait-declared unsafe fns must be qualified by their trait: {out:?}"
    );
}

#[test]
pub(super) fn trait_impl_unsafe_fn_stays_distinct_from_inherent_and_other_traits() {
    // A trait-impl `unsafe fn` is qualified by `<trait for self>`, not the self type alone: on ONE
    // self type, an inherent `unsafe fn m`, `impl A for Foo { unsafe fn m }`, and
    // `impl B for Foo { unsafe fn m }` are three distinct `unsafe` sites and MUST stay three
    // findings — else a baseline of the inherent (or one trait-impl) silently accepts a later-added
    // trait-impl `unsafe fn` on a *safe* trait (no independent `unsafe impl` finding): a new
    // out-of-subtree `unsafe` site passing unobserved, the forbidden false negative. Self-type-only
    // qualification (`unsafe fn Foo::m` for all three) collapsed them; this pins the fix.
    let out = unsafe_labels(
        "unsafe-fns-trait-impl",
        &[
            ("lib.rs", "pub mod net;\n"),
            (
                "net.rs",
                "pub struct Foo;\npub trait A { fn m(&self); }\npub trait B { fn m(&self); }\n\
                 impl Foo { unsafe fn m(&self) {} }\n\
                 impl A for Foo { unsafe fn m(&self) {} }\n\
                 impl B for Foo { unsafe fn m(&self) {} }\n",
            ),
        ],
        &["crate::ffi"],
    )
    .unwrap();
    assert_eq!(
        out,
        [
            "unsafe fn <A for Foo>::m in crate::net",
            "unsafe fn <B for Foo>::m in crate::net",
            "unsafe fn Foo::m in crate::net",
        ],
        "a trait-impl unsafe fn must be qualified by <trait for self>, distinct from the inherent \
         method and other trait impls on the same type: {out:?}"
    );
}

#[test]
pub(super) fn unsafe_in_an_unconditional_path_remapped_module_reacts() {
    // An unconditional `#[path = "relocated.rs"] mod net;` is followed to relocated.rs (there is no
    // conventional net.rs, so this only resolves by following the remap); its `unsafe fn`, outside
    // the allowed subtree, reacts attributed to the declared module path `crate::net`. Previously
    // the relocated module was skipped — a false negative (relocated `unsafe` passing unobserved).
    let out = unsafe_labels(
        "path-remap-unsafe",
        &[
            ("lib.rs", "#[path = \"relocated.rs\"]\npub mod net;\n"),
            ("relocated.rs", "pub unsafe fn poke() {}\n"),
        ],
        &["crate::ffi"],
    )
    .unwrap();
    assert_eq!(
        out,
        ["unsafe fn poke in crate::net"],
        "unsafe in an unconditional #[path] module is followed and reacts: {out:?}"
    );
}

#[test]
pub(super) fn path_in_a_non_mod_rs_file_resolves_from_the_containing_files_own_dir() {
    // rustc 1.x ground truth: a non-inline `#[path="bar.rs"]` written INSIDE src/foo.rs (reached via
    // `mod foo;`) resolves to src/bar.rs — the CONTAINING file's own directory — NOT src/foo/bar.rs.
    // The real unsafe fn lives at the rustc-correct src/bar.rs; a decoy sits at the wrong src/foo/bar.rs
    // the earlier (buggy) child_dir base would have read. Resolving from child_dir reads the decoy and
    // drops the real unsafe (Ok([]) — the forbidden false negative); this pins the corrected base.
    let out = unsafe_labels(
        "path-nonmodrs",
        &[
            ("lib.rs", "pub mod foo;\n"),
            ("foo.rs", "#[path = \"bar.rs\"]\npub mod bar;\n"),
            ("bar.rs", "pub unsafe fn poke() {}\n"),
            ("foo/bar.rs", "pub fn decoy() {}\n"),
        ],
        &["crate::ffi"],
    )
    .unwrap();
    assert_eq!(
        out,
        ["unsafe fn poke in crate::foo::bar"],
        "a #[path] inside a non-mod.rs file resolves from that file's own dir (src/bar.rs), not \
         src/foo/bar.rs: {out:?}"
    );
}

#[test]
pub(super) fn path_nested_in_an_inline_block_resolves_from_the_accumulated_dir() {
    // rustc ground truth (verified against rustc 1.96.0): a `#[path="other.rs"]` written INSIDE an
    // inline `mod inline { … }` at the crate root resolves to src/inline/other.rs — rustc accumulates
    // the inline-module name as a directory component onto the file's own dir. The real unsafe lives
    // at the rustc-correct src/inline/other.rs; a decoy sits at src/other.rs, which threading the
    // enclosing file_dir UNCHANGED through inline descent would have read (dropping the real unsafe,
    // Ok([]) — the forbidden false negative). Pins the accumulated inline base.
    let out = unsafe_labels(
        "path-inline-modrs",
        &[
            (
                "lib.rs",
                "pub mod inline { #[path = \"other.rs\"] pub mod inner; }\n",
            ),
            ("inline/other.rs", "pub unsafe fn poke() {}\n"),
            ("other.rs", "pub fn decoy() {}\n"),
        ],
        &["crate::ffi"],
    )
    .unwrap();
    assert_eq!(
        out,
        ["unsafe fn poke in crate::inline::inner"],
        "a #[path] nested in an inline block resolves from <file_dir>/inline (src/inline/other.rs), \
         not the src/other.rs orphan: {out:?}"
    );
}

#[test]
pub(super) fn path_nested_in_an_inline_block_in_a_non_mod_rs_file_accumulates_both_components() {
    // rustc ground truth (rustc 1.96.0): src/bar.rs (reached via `mod bar;`, a non-mod-rs file) with
    // `pub mod inline { #[path="p.rs"] pub mod inner; }` resolves inner to src/bar/inline/p.rs — the
    // base accumulates BOTH the non-mod-rs conventional-child dir (bar/) AND the inline name (inline/).
    // Real unsafe at the rustc-correct src/bar/inline/p.rs; a decoy at src/p.rs (the enclosing
    // file_dir base). Confirms the two components compose.
    let out = unsafe_labels(
        "path-inline-nonmodrs",
        &[
            ("lib.rs", "pub mod bar;\n"),
            (
                "bar.rs",
                "pub mod inline { #[path = \"p.rs\"] pub mod inner; }\n",
            ),
            ("bar/inline/p.rs", "pub unsafe fn poke() {}\n"),
            ("p.rs", "pub fn decoy() {}\n"),
        ],
        &["crate::ffi"],
    )
    .unwrap();
    assert_eq!(
        out,
        ["unsafe fn poke in crate::bar::inline::inner"],
        "the #[path] base accumulates bar/ and inline/ (src/bar/inline/p.rs), not src/p.rs: {out:?}"
    );
}

#[test]
pub(super) fn two_modules_sharing_one_path_target_are_not_a_false_cycle() {
    // rustc ground truth (rustc 1.96.0): `#[path="shared.rs"] pub mod a; #[path="shared.rs"] pub mod
    // b;` compiles cleanly — two sibling declarations legitimately resolving to one file is NOT a
    // cycle. A monotonic whole-tree visited set misreported the second reach as a "symlink loop"
    // (exit 2) on rustc-compilable input — a false positive and a 三儀 ⊥ 三儀 divergence (漏刻 accepts
    // it). The ancestor-path guard must accept it: the unsafe in shared.rs reacts under BOTH paths.
    let out = unsafe_labels(
        "path-shared-target",
        &[
            (
                "lib.rs",
                "#[path = \"shared.rs\"]\npub mod a;\n#[path = \"shared.rs\"]\npub mod b;\n",
            ),
            ("shared.rs", "pub unsafe fn poke() {}\n"),
        ],
        &["crate::ffi"],
    )
    .expect("two modules sharing one #[path] target is not a cycle (rustc compiles it)");
    assert_eq!(
        out,
        ["unsafe fn poke in crate::a", "unsafe fn poke in crate::b",],
        "a file shared by two #[path] declarations reacts under both module paths, no false cycle: \
         {out:?}"
    );
}

#[test]
pub(super) fn a_conventional_module_and_a_path_alias_to_it_are_not_a_false_cycle() {
    // rustc ground truth (rustc 1.96.0): `pub mod foo; #[path="foo.rs"] pub mod bar;` compiles — one
    // file (src/foo.rs) reached by a conventional decl and a #[path] alias is not a cycle. Pins the
    // second, conventional-branch face of the ancestor-guard fix.
    let out = unsafe_labels(
        "path-alias-conventional",
        &[
            (
                "lib.rs",
                "pub mod foo;\n#[path = \"foo.rs\"]\npub mod bar;\n",
            ),
            ("foo.rs", "pub unsafe fn poke() {}\n"),
        ],
        &["crate::ffi"],
    )
    .expect("a conventional module and a #[path] alias to the same file is not a cycle");
    assert_eq!(
        out,
        [
            "unsafe fn poke in crate::bar",
            "unsafe fn poke in crate::foo",
        ],
        "one file reached conventionally and via a #[path] alias reacts under both paths: {out:?}"
    );
}

#[test]
pub(super) fn unsafe_in_a_body_nested_mod_reacts() {
    // The propose-review false-negative guard: a `mod` inside a fn body is not descended by the
    // top-level walk; the collector's default recursion must still catch its unsafe.
    let out = unsafe_labels(
        "body-nested",
        &[
            ("lib.rs", "pub mod net;\n"),
            (
                "net.rs",
                "pub fn f() { mod raw { pub unsafe fn poke() {} } }\n",
            ),
        ],
        &["crate::ffi"],
    )
    .unwrap();
    assert_eq!(
        out,
        ["unsafe fn poke in crate::net"],
        "unsafe in a body-nested mod is attributed to the enclosing module, never dropped: {out:?}"
    );
}

#[test]
pub(super) fn unsafe_in_a_macro_body_is_a_stated_bound() {
    // Macro bodies are unexpanded (the dimension's inherited macro bound): the unsafe inside a
    // never-invoked macro definition is not observed — stated, not a silent claim. The one
    // carve-out is the transparent `cfg_if!`, whose arms are read as real code
    // (`an_unsafe_site_inside_a_cfg_if_arm_is_confined`); a `macro_rules!` definition transforms
    // identities and stays opaque.
    let out = unsafe_labels(
        "macro",
        &[
            ("lib.rs", "pub mod net;\n"),
            (
                "net.rs",
                "macro_rules! m { () => { unsafe {} }; }\npub fn f() {}\n",
            ),
        ],
        &["crate::ffi"],
    )
    .unwrap();
    assert!(
        out.is_empty(),
        "unsafe in a macro body is not observed: {out:?}"
    );
}
