use super::super::*;
use super::async_exposure::async_mod;
use super::async_exposure::async_subtree_labels;
use super::dyn_trait::dyn_mod;
use super::forbidden_marker::marker_findings;
use super::helpers::*;
use super::impl_trait::impl_trait_mod;
use super::trait_impl::locality_findings;
use super::unsafe_confinement::unsafe_labels;
use super::visibility::vis_findings;
// --- transparent macro (`cfg_if!`) arm observation --------------------------
//
// `cfg_if!` wraps human-authored items in arms without transforming their identities, so 圭表 reads
// them as real code. 渾儀 did not: `syn` parses the invocation as an opaque `Item::Macro`, and every
// capability matches concrete item variants — a measured exposure false negative on ordinary,
// compilable source (the identical function reacted at module top level and passed inside an arm).
// The shapes below are the ten a feasibility spike measured, plus the controls without which an
// emptiness assertion could pass vacuously.

/// Shape 1: `if` / `else`. BOTH arms are observed — the union is cfg-blind, so a violation in the
/// arm this build does not select still reacts (a stated bound: knowing which arm compiles requires
/// evaluating the whole feature/target resolution, cargo's job, not a scanner's).
#[test]
pub(super) fn cfg_if_if_else_arms_both_expose_forbidden_types() {
    let out = findings(
        "cfg-if-both-arms",
        &[
            ("lib.rs", "pub mod api;\npub mod infra;\n"),
            (
                "api.rs",
                "cfg_if::cfg_if! {\n\
                      if #[cfg(unix)] {\n\
                          pub fn unix_leak() -> crate::infra::Secret { loop {} }\n\
                      } else {\n\
                          pub fn fallback_leak() -> crate::infra::Secret { loop {} }\n\
                      }\n\
                 }\n",
            ),
            ("infra.rs", "pub struct Secret;\n"),
        ],
        "crate::api",
        &["crate::infra"],
    )
    .unwrap();
    assert_eq!(
        out,
        vec![
            "crate::infra::Secret exposed by fn crate::api::fallback_leak",
            "crate::infra::Secret exposed by fn crate::api::unix_leak",
        ],
        "both cfg_if arms' exposures must react (cfg-blind union): {out:?}"
    );
}

/// Shape 2: a single `if` arm with no `else`.
#[test]
pub(super) fn cfg_if_if_only_arm_exposes_a_forbidden_type() {
    let out = findings(
        "cfg-if-if-only",
        &[
            ("lib.rs", "pub mod api;\npub mod infra;\n"),
            (
                "api.rs",
                "cfg_if::cfg_if! {\n\
                      if #[cfg(unix)] {\n\
                          pub fn leak() -> crate::infra::Secret { loop {} }\n\
                      }\n\
                 }\n",
            ),
            ("infra.rs", "pub struct Secret;\n"),
        ],
        "crate::api",
        &["crate::infra"],
    )
    .unwrap();
    assert_eq!(
        out,
        vec!["crate::infra::Secret exposed by fn crate::api::leak"],
        "an if-only cfg_if arm must be observed: {out:?}"
    );
}

/// Shape 3: an `else if` chain — every arm, not just the first and last.
#[test]
pub(super) fn cfg_if_else_if_chain_exposes_every_arm() {
    let out = findings(
        "cfg-if-chain",
        &[
            ("lib.rs", "pub mod api;\npub mod infra;\n"),
            (
                "api.rs",
                "cfg_if::cfg_if! {\n\
                      if #[cfg(unix)] {\n\
                          pub fn a_leak() -> crate::infra::Secret { loop {} }\n\
                      } else if #[cfg(windows)] {\n\
                          pub fn b_leak() -> crate::infra::Secret { loop {} }\n\
                      } else {\n\
                          pub fn c_leak() -> crate::infra::Secret { loop {} }\n\
                      }\n\
                 }\n",
            ),
            ("infra.rs", "pub struct Secret;\n"),
        ],
        "crate::api",
        &["crate::infra"],
    )
    .unwrap();
    assert_eq!(
        out,
        vec![
            "crate::infra::Secret exposed by fn crate::api::a_leak",
            "crate::infra::Secret exposed by fn crate::api::b_leak",
            "crate::infra::Secret exposed by fn crate::api::c_leak",
        ],
        "every arm of an else-if chain must be observed: {out:?}"
    );
}

/// Shape 4: a `cfg_if!` nested inside an arm — the flattening recurses.
#[test]
pub(super) fn nested_cfg_if_inside_an_arm_exposes_a_forbidden_type() {
    let out = findings(
        "cfg-if-nested",
        &[
            ("lib.rs", "pub mod api;\npub mod infra;\n"),
            (
                "api.rs",
                "cfg_if::cfg_if! {\n\
                      if #[cfg(unix)] {\n\
                          cfg_if::cfg_if! {\n\
                              if #[cfg(target_pointer_width = \"64\")] {\n\
                                  pub fn inner_leak() -> crate::infra::Secret { loop {} }\n\
                              }\n\
                          }\n\
                      }\n\
                 }\n",
            ),
            ("infra.rs", "pub struct Secret;\n"),
        ],
        "crate::api",
        &["crate::infra"],
    )
    .unwrap();
    assert_eq!(
        out,
        vec!["crate::infra::Secret exposed by fn crate::api::inner_leak"],
        "a nested cfg_if's arm must be observed: {out:?}"
    );
}

/// Shape 5: an arm-declared **file** module enters the module walk. Were the declaration invisible,
/// the anchor would not resolve at all (exit 2, "unknown module") — and in the whole-crate walk the
/// entire subtree beneath it would go unobserved.
#[test]
pub(super) fn a_cfg_if_arm_declared_file_module_is_walked() {
    let out = findings(
        "cfg-if-arm-mod",
        &[
            (
                "lib.rs",
                "pub mod infra;\n\
                 cfg_if::cfg_if! {\n\
                      if #[cfg(unix)] {\n\
                          pub mod api;\n\
                      }\n\
                 }\n",
            ),
            (
                "api.rs",
                "pub fn leak() -> crate::infra::Secret { loop {} }\n",
            ),
            ("infra.rs", "pub struct Secret;\n"),
        ],
        "crate::api",
        &["crate::infra"],
    )
    .unwrap();
    assert_eq!(
        out,
        vec!["crate::infra::Secret exposed by fn crate::api::leak"],
        "a mod declared only inside a cfg_if arm must be descended: {out:?}"
    );
}

/// Shape 6: an arm-declared **inline** module.
#[test]
pub(super) fn a_cfg_if_arm_declared_inline_module_is_walked() {
    let out = findings(
        "cfg-if-arm-inline-mod",
        &[
            (
                "lib.rs",
                "pub mod infra;\n\
                 cfg_if::cfg_if! {\n\
                      if #[cfg(unix)] {\n\
                          pub mod api {\n\
                              pub fn leak() -> crate::infra::Secret { loop {} }\n\
                          }\n\
                      }\n\
                 }\n",
            ),
            ("infra.rs", "pub struct Secret;\n"),
        ],
        "crate::api",
        &["crate::infra"],
    )
    .unwrap();
    assert_eq!(
        out,
        vec!["crate::infra::Secret exposed by fn crate::api::leak"],
        "an inline mod declared inside a cfg_if arm must be descended: {out:?}"
    );
}

/// Shape 7: the outer delimiter of the invocation is irrelevant — `cfg_if!( … );` is the same macro.
#[test]
pub(super) fn a_paren_delimited_cfg_if_invocation_is_transparent() {
    let out = findings(
        "cfg-if-paren",
        &[
            ("lib.rs", "pub mod api;\npub mod infra;\n"),
            (
                "api.rs",
                "cfg_if::cfg_if!(\n\
                      if #[cfg(unix)] {\n\
                          pub fn leak() -> crate::infra::Secret { loop {} }\n\
                      }\n\
                 );\n",
            ),
            ("infra.rs", "pub struct Secret;\n"),
        ],
        "crate::api",
        &["crate::infra"],
    )
    .unwrap();
    assert_eq!(
        out,
        vec!["crate::infra::Secret exposed by fn crate::api::leak"],
        "a paren-delimited cfg_if invocation must be observed: {out:?}"
    );
}

/// Shape 8: a `cfg_if!` written **inside an inline module** — the flattening applies at every level
/// the walk descends to, not only a file's top level.
#[test]
pub(super) fn a_cfg_if_inside_an_inline_module_is_transparent() {
    let out = findings(
        "cfg-if-in-inline-mod",
        &[
            ("lib.rs", "pub mod api;\npub mod infra;\n"),
            (
                "api.rs",
                "pub mod inner {\n\
                      cfg_if::cfg_if! {\n\
                          if #[cfg(unix)] {\n\
                              pub fn leak() -> crate::infra::Secret { loop {} }\n\
                          }\n\
                      }\n\
                 }\n",
            ),
            ("infra.rs", "pub struct Secret;\n"),
        ],
        "crate::api::inner",
        &["crate::infra"],
    )
    .unwrap();
    assert_eq!(
        out,
        vec!["crate::infra::Secret exposed by fn crate::api::inner::leak"],
        "a cfg_if inside an inline module must be observed: {out:?}"
    );
}

/// Shape 9: an arm body that does not parse as items yields nothing — never a scan error. The
/// invocation is `cfg_if!`-named, so the arm walk runs; its body is statements, which `syn::File`
/// rejects. The forbidden exposure at top level is the control: it proves the fixture is otherwise
/// live, so the arm's emptiness is not an artifact of nothing being observed at all.
#[test]
pub(super) fn a_cfg_if_arm_body_that_does_not_parse_as_items_is_not_a_scan_error() {
    let out = findings(
        "cfg-if-unparseable-arm",
        &[
            ("lib.rs", "pub mod api;\npub mod infra;\n"),
            (
                "api.rs",
                "pub fn control_leak() -> crate::infra::Secret { loop {} }\n\
                 cfg_if::cfg_if! {\n\
                      if #[cfg(unix)] {\n\
                          let _not_an_item = 1;\n\
                      }\n\
                 }\n",
            ),
            ("infra.rs", "pub struct Secret;\n"),
        ],
        "crate::api",
        &["crate::infra"],
    )
    .unwrap();
    assert_eq!(
        out,
        vec!["crate::infra::Secret exposed by fn crate::api::control_leak"],
        "an unparseable arm body must yield no items and no error, leaving the rest observed: {out:?}"
    );
}

/// Shape 10: the **name gate**, and why it is load-bearing rather than conservative. Arm extraction
/// reads every top-level brace group of the body as an arm; in an arbitrary macro's body an
/// `impl Foo { … }`'s braces ARE such a group, so an unnamed-gated walk would recover a `fn hidden`
/// this macro may never emit verbatim — a false positive. Paired with the control below, which
/// proves the identical function DOES react when written as an item.
#[test]
pub(super) fn an_arbitrary_macro_body_is_not_read_as_transparent_arms() {
    let out = findings(
        "arbitrary-macro-body",
        &[
            ("lib.rs", "pub mod api;\npub mod infra;\n"),
            (
                "api.rs",
                "generate_wrapper! {\n\
                      impl Foo {\n\
                          pub fn hidden() -> crate::infra::Secret { loop {} }\n\
                      }\n\
                 }\n",
            ),
            ("infra.rs", "pub struct Secret;\n"),
        ],
        "crate::api",
        &["crate::infra"],
    )
    .unwrap();
    assert!(
        out.is_empty(),
        "only cfg_if is transparent: an arbitrary macro's body must not be read as arms: {out:?}"
    );
}

/// The control for the name gate: the same `fn hidden` written as a real item reacts. Without this,
/// the emptiness above could hold for any reason at all — an unresolvable anchor, a forbidden set
/// that matches nothing — rather than because the macro body was left unread.
#[test]
pub(super) fn the_same_exposure_written_as_an_item_reacts() {
    let out = findings(
        "arbitrary-macro-control",
        &[
            ("lib.rs", "pub mod api;\npub mod infra;\n"),
            (
                "api.rs",
                "pub struct Foo;\n\
                 impl Foo {\n\
                      pub fn hidden() -> crate::infra::Secret { loop {} }\n\
                 }\n\
                 pub fn hidden() -> crate::infra::Secret { loop {} }\n",
            ),
            ("infra.rs", "pub struct Secret;\n"),
        ],
        "crate::api",
        &["crate::infra"],
    )
    .unwrap();
    assert_eq!(
        out,
        vec![
            "crate::infra::Secret exposed by fn <crate::api::Foo>::hidden",
            "crate::infra::Secret exposed by fn crate::api::hidden",
        ],
        "the control exposure must react as an item — both the inherent-impl method the macro \
         fixture wrapped and the free function: {out:?}"
    );
}

/// An arm-declared module is **cfg-conditional**: every arm is gated by a predicate in the macro
/// header, so an absent conventional file is a legitimate configuration, not a broken declaration —
/// 圭表's settled rule for the same shape, adopted rather than re-derived.
#[test]
pub(super) fn a_cfg_if_arm_declared_module_with_no_source_file_is_tolerated() {
    let out = findings(
        "cfg-if-arm-absent-file",
        &[
            (
                "lib.rs",
                "pub mod api;\npub mod infra;\n\
                 cfg_if::cfg_if! {\n\
                      if #[cfg(unix)] {\n\
                          pub mod unix_only;\n\
                      }\n\
                 }\n",
            ),
            (
                "api.rs",
                "pub fn leak() -> crate::infra::Secret { loop {} }\n",
            ),
            ("infra.rs", "pub struct Secret;\n"),
        ],
        "crate::api",
        &["crate::infra"],
    )
    .unwrap();
    assert_eq!(
        out,
        vec!["crate::infra::Secret exposed by fn crate::api::leak"],
        "an arm-declared module with no file must be tolerated, not a scan error: {out:?}"
    );
}

/// The control for that tolerance: the identical declaration **without** the arm still fails loud.
/// Otherwise the tolerance above could be indistinguishable from never having read the declaration.
#[test]
pub(super) fn the_same_module_declaration_outside_an_arm_still_fails_loud() {
    let error = findings(
        "cfg-if-arm-absent-control",
        &[
            (
                "lib.rs",
                "pub mod api;\npub mod infra;\npub mod unix_only;\n",
            ),
            (
                "api.rs",
                "pub fn leak() -> crate::infra::Secret { loop {} }\n",
            ),
            ("infra.rs", "pub struct Secret;\n"),
        ],
        "crate::api",
        &["crate::infra"],
    )
    .unwrap_err();
    assert!(
        error.contains("unix_only") || error.contains("no source file"),
        "an unconditional declaration with no file must stay a scan error: {error}"
    );
}

/// Arm membership tolerates an ABSENCE; it never tolerates an **ambiguity**. No predicate value
/// makes two conventional files compile as one module, so a dual-backed arm-declared module stays a
/// constitution error — the same ordering all three dimensions apply to this shape.
#[test]
pub(super) fn a_cfg_if_arm_declared_dual_backed_module_is_still_a_scan_error() {
    let error = findings(
        "cfg-if-arm-dual-backed",
        &[
            (
                "lib.rs",
                "pub mod api;\npub mod infra;\n\
                 cfg_if::cfg_if! {\n\
                      if #[cfg(unix)] {\n\
                          pub mod dual;\n\
                      }\n\
                 }\n",
            ),
            (
                "api.rs",
                "pub fn leak() -> crate::infra::Secret { loop {} }\n",
            ),
            ("infra.rs", "pub struct Secret;\n"),
            ("dual.rs", "pub struct A;\n"),
            ("dual/mod.rs", "pub struct A;\n"),
        ],
        "crate::api",
        &["crate::infra"],
    )
    .unwrap_err();
    assert!(
        error.contains("resolves to both"),
        "a dual-backed arm-declared module must stay a scan error: {error}"
    );
}

/// The whole-crate walks read arms too, not only the anchored descent: an `unsafe` site written
/// inside a `cfg_if!` arm is confined like any other. This exercises `walk_unsafe`, a separate
/// descent from the one every test above goes through.
#[test]
pub(super) fn an_unsafe_site_inside_a_cfg_if_arm_is_confined() {
    let out = unsafe_labels(
        "cfg-if-arm",
        &[
            ("lib.rs", "pub mod ffi;\npub mod net;\n"),
            ("ffi.rs", ""),
            (
                "net.rs",
                "cfg_if::cfg_if! {\n\
                      if #[cfg(unix)] {\n\
                          pub unsafe fn decode() {}\n\
                      }\n\
                 }\n",
            ),
        ],
        &["crate::ffi"],
    )
    .unwrap();
    assert_eq!(
        out,
        vec!["unsafe fn decode in crate::net"],
        "an unsafe site inside a cfg_if arm must react: {out:?}"
    );
}

/// A trait impl inside an arm enters the crate-wide impl scan (`walk_module`), which feeds
/// trait-impl locality and the re-export/alias closures every other capability resolves through.
#[test]
pub(super) fn a_trait_impl_inside_a_cfg_if_arm_is_located() {
    let out = locality_findings(
        "cfg-if-arm",
        &[
            ("lib.rs", "pub mod command;\npub mod domain;\n"),
            ("command.rs", "pub trait Command {}\n"),
            (
                "domain.rs",
                "use crate::command::Command;\n\
                 pub struct Foo;\n\
                 cfg_if::cfg_if! {\n\
                      if #[cfg(unix)] {\n\
                          impl Command for Foo {}\n\
                      }\n\
                 }\n",
            ),
        ],
        "crate::command::Command",
        &["crate::commands"],
    )
    .unwrap();
    assert_eq!(
        out,
        vec!["crate::domain (impl crate::command::Command for crate::domain::Foo)"],
        "a trait impl inside a cfg_if arm must be located: {out:?}"
    );
}

// Task-1 verification rather than assumption: the four single-module-anchored capabilities share one
// resolution entry, but "they share it" is a claim about the code, so each is measured on an
// arm-wrapped construct of its own shape. The two crate-wide walks are covered above
// (`an_unsafe_site_inside_a_cfg_if_arm_is_confined`, `a_trait_impl_inside_a_cfg_if_arm_is_located`).

/// Visibility: a bare `pub` inside an arm is declared surface like any other.
#[test]
pub(super) fn a_bare_pub_inside_a_cfg_if_arm_reacts() {
    let out = vis_findings(
        "cfg-if-arm",
        &[
            ("lib.rs", "pub mod m;\n"),
            (
                "m.rs",
                "cfg_if::cfg_if! {\n\
                 if #[cfg(unix)] {\n\
                 pub fn wide() {}\n\
                 }\n\
                 }\n",
            ),
        ],
        "crate::m",
    )
    .unwrap();
    assert_eq!(
        out,
        vec!["pub fn wide"],
        "a bare pub inside a cfg_if arm must react: {out:?}"
    );
}

/// dyn-trait: a `dyn` seam inside an arm crosses the boundary like any other.
#[test]
pub(super) fn a_dyn_seam_inside_a_cfg_if_arm_reacts() {
    let out = dyn_mod(
        "cfg-if-arm",
        "pub trait Port {}\n\
         cfg_if::cfg_if! {\n\
         if #[cfg(unix)] {\n\
         pub fn get() -> Box<dyn Port> { loop {} }\n\
         }\n\
         }\n",
    )
    .unwrap();
    assert_eq!(
        out.len(),
        1,
        "a dyn seam inside a cfg_if arm must react: {out:?}"
    );
}

/// impl-trait: an existential return inside an arm is exposed like any other.
#[test]
pub(super) fn an_impl_trait_seam_inside_a_cfg_if_arm_reacts() {
    let out = impl_trait_mod(
        "cfg-if-arm",
        "pub trait Port {}\n\
         cfg_if::cfg_if! {\n\
         if #[cfg(unix)] {\n\
         pub fn get() -> impl Port { loop {} }\n\
         }\n\
         }\n",
    )
    .unwrap();
    assert_eq!(
        out.len(),
        1,
        "an impl-trait seam inside a cfg_if arm must react: {out:?}"
    );
}

/// async-exposure: a public `async fn` inside an arm is an async seam like any other.
#[test]
pub(super) fn an_async_fn_inside_a_cfg_if_arm_reacts() {
    let out = async_mod(
        "cfg-if-arm",
        "cfg_if::cfg_if! {\n\
         if #[cfg(unix)] {\n\
         pub async fn serve() {}\n\
         }\n\
         }\n",
    )
    .unwrap();
    assert_eq!(
        out.len(),
        1,
        "an async fn inside a cfg_if arm must react: {out:?}"
    );
}

/// forbidden-marker: a derive inside an arm is acquired like any other. This capability walks the
/// whole crate (`scan_crate`), so it also pins the type-definition half of that walk.
#[test]
pub(super) fn a_forbidden_marker_inside_a_cfg_if_arm_reacts() {
    let out = marker_findings(
        "cfg-if-arm",
        &[
            ("lib.rs", "pub mod domain;\n"),
            (
                "domain.rs",
                "cfg_if::cfg_if! {\n\
                 if #[cfg(unix)] {\n\
                 #[derive(serde::Serialize)]\n\
                 pub struct Order;\n\
                 }\n\
                 }\n",
            ),
        ],
        "crate::domain",
        &["serde::Serialize"],
    )
    .unwrap();
    assert_eq!(
        out.len(),
        1,
        "a forbidden derive inside a cfg_if arm must react: {out:?}"
    );
}

/// The subtree walk (`including_submodules`) records each module's items itself, on a descent
/// separate from the anchored one every test above uses — so it needs its own arm coverage: an
/// `async fn` inside a submodule's `cfg_if!` arm must react at the subtree scope.
#[test]
pub(super) fn a_subtree_scope_observes_an_async_fn_inside_a_cfg_if_arm() {
    let out = async_subtree_labels(
        "cfg-if-arm",
        &[
            ("lib.rs", "pub mod net;\n"),
            (
                "net.rs",
                "cfg_if::cfg_if! {\n\
                 if #[cfg(unix)] {\n\
                 pub async fn connect() {}\n\
                 }\n\
                 }\n",
            ),
        ],
        "crate",
    );
    assert_eq!(
        out.len(),
        1,
        "an async fn inside a submodule's cfg_if arm must react at the subtree scope: {out:?}"
    );
}

/// The unqualified spelling. Every test above writes `cfg_if::cfg_if!`; an adopter who wrote
/// `use cfg_if::cfg_if;` invokes it as a bare `cfg_if!`, and the name test matches the path's LAST
/// segment precisely so both spellings are one shape (圭表 matches the same way).
#[test]
pub(super) fn an_unqualified_cfg_if_invocation_is_transparent() {
    let out = findings(
        "cfg-if-unqualified",
        &[
            ("lib.rs", "pub mod api;\npub mod infra;\n"),
            (
                "api.rs",
                "use cfg_if::cfg_if;\n\
                 cfg_if! {\n\
                 if #[cfg(unix)] {\n\
                 pub fn leak() -> crate::infra::Secret { loop {} }\n\
                 }\n\
                 }\n",
            ),
            ("infra.rs", "pub struct Secret;\n"),
        ],
        "crate::api",
        &["crate::infra"],
    )
    .unwrap();
    assert_eq!(
        out,
        vec!["crate::infra::Secret exposed by fn crate::api::leak"],
        "the unqualified cfg_if! spelling must be transparent too: {out:?}"
    );
}

/// Two arms declaring the SAME module name — the per-platform shim spelled with one file. Both
/// declarations resolve to the identical file, so the walkers' dedup must collapse them: flattening
/// must not inflate one real violation into two findings (the false-positive direction of this
/// change).
#[test]
pub(super) fn two_cfg_if_arms_declaring_one_module_name_do_not_double_report() {
    let out = findings(
        "cfg-if-arm-twin-mod",
        &[
            (
                "lib.rs",
                "pub mod infra;\n\
                 cfg_if::cfg_if! {\n\
                 if #[cfg(unix)] {\n\
                 pub mod api;\n\
                 } else {\n\
                 pub mod api;\n\
                 }\n\
                 }\n",
            ),
            (
                "api.rs",
                "pub fn leak() -> crate::infra::Secret { loop {} }\n",
            ),
            ("infra.rs", "pub struct Secret;\n"),
        ],
        "crate::api",
        &["crate::infra"],
    )
    .unwrap();
    assert_eq!(
        out,
        vec!["crate::infra::Secret exposed by fn crate::api::leak"],
        "two arms declaring the same module must dedup to one finding: {out:?}"
    );
}

/// A `cfg_if!` inside an `impl` block: transparency is for **item position**, where `syn` gives an
/// `Item::Macro` whose arms parse as items. Inside an `impl` (or `trait`) body the invocation is an
/// `ImplItem::Macro` whose arms are impl items, a different flattening across a different set of
/// walkers — its own change, not silently smuggled into this one. Pinned as a **stated bound** so
/// the hole is discoverable rather than latent; when it closes, this test is the one that fails.
#[test]
pub(super) fn a_cfg_if_inside_an_impl_body_is_a_stated_bound() {
    let out = findings(
        "cfg-if-impl-body",
        &[
            ("lib.rs", "pub mod api;\npub mod infra;\n"),
            (
                "api.rs",
                "pub struct Api;\n\
                 impl Api {\n\
                 cfg_if::cfg_if! {\n\
                 if #[cfg(unix)] {\n\
                 pub fn leak() -> crate::infra::Secret { loop {} }\n\
                 }\n\
                 }\n\
                 }\n",
            ),
            ("infra.rs", "pub struct Secret;\n"),
        ],
        "crate::api",
        &["crate::infra"],
    )
    .unwrap();
    assert!(
        out.is_empty(),
        "an impl-body cfg_if is a stated bound, not a claimed reaction: {out:?}"
    );
}

/// The other absence site reads the same gate: an **unconditional** `#[path = "…"]` inside an arm
/// whose target file does not exist is tolerated, because the arm's predicate removes the whole item
/// — `#[path]` included — exactly as a bare `#[cfg]` beside it would. Without this the two absence
/// outcomes would drift apart, which is the divergence the shared gate exists to prevent (and 圭表
/// states the same rule for its own walker).
#[test]
pub(super) fn an_absent_path_remap_target_inside_a_cfg_if_arm_is_tolerated() {
    let out = findings(
        "cfg-if-arm-absent-path",
        &[
            (
                "lib.rs",
                "pub mod api;\npub mod infra;\n\
                 cfg_if::cfg_if! {\n\
                 if #[cfg(windows)] {\n\
                 #[path = \"windows_impl.rs\"]\n\
                 pub mod imp;\n\
                 }\n\
                 }\n",
            ),
            (
                "api.rs",
                "pub fn leak() -> crate::infra::Secret { loop {} }\n",
            ),
            ("infra.rs", "pub struct Secret;\n"),
        ],
        "crate::api",
        &["crate::infra"],
    )
    .unwrap();
    assert_eq!(
        out,
        vec!["crate::infra::Secret exposed by fn crate::api::leak"],
        "an absent unconditional #[path] target inside an arm must be tolerated: {out:?}"
    );
}

/// Two-crate reproduction of the audit-sweep finding at hunyi's own composed dedup
/// (`driver::outcome_from`, the analogue of guibiao's `evaluate`) — not just the per-fact catalog
/// tests. Identical governed module path + rule declared against two different workspace members
/// must survive as two distinct violations, mirroring the guibiao-side regression in
/// `crates/guibiao/src/tests/`.
#[test]
pub(super) fn two_crates_with_the_identical_async_exposure_boundary_stay_distinct_violations() {
    fn tree_and_metadata(label: &str, package: &str) -> (TempSrcTree, Value) {
        let tree = TempSrcTree::new(label);
        tree.write_all(&[("lib.rs", "pub mod registry;\npub async fn register() {}\n")]);
        let metadata = serde_json::json!({
            "packages": [{
                "name": package,
                "dependencies": [],
                "targets": [{ "kind": ["lib"], "src_path": tree.root().to_string_lossy().into_owned() }],
            }],
        });
        (tree, metadata)
    }

    let (_alpha_tree, alpha_metadata) = tree_and_metadata("async-identity-alpha", "alpha");
    let (_beta_tree, beta_metadata) = tree_and_metadata("async-identity-beta", "beta");
    let combined_metadata = serde_json::json!({
        "packages": [
            alpha_metadata["packages"][0].clone(),
            beta_metadata["packages"][0].clone(),
        ],
    });

    fn boundary_for(package: &str) -> AsyncExposureBoundary {
        AsyncExposureBoundary::in_crate(package)
            .module("crate")
            .must_not_expose_async_fn()
            .because("no async seam here")
    }

    let mut violations = Vec::new();
    eval_into(
        &combined_metadata,
        &[boundary_for("alpha"), boundary_for("beta")],
        check_async_exposure_boundary,
        &mut violations,
    )
    .expect("both boundaries resolve");
    let outcome = outcome_from(violations);
    let report = match outcome {
        Outcome::Violations(report) => report,
        other => panic!("expected two violations, got {other:?}"),
    };
    assert_eq!(
        report.violations.len(),
        2,
        "each crate's async-exposure violation must survive dedup: {:?}",
        report.violations
    );
    let ids: std::collections::BTreeSet<_> = report.violations.iter().map(Violation::id).collect();
    assert_eq!(
        ids.len(),
        2,
        "identity must differ by crate, not collapse to one"
    );
}

/// A `pub fn` inside an `extern` block is a real item in the module's own namespace — as public as
/// a same-shaped ordinary `fn` — so a forbidden type named only in its signature must react exactly
/// like an ordinary function's would, not escape the query because the declaration has no body.
#[test]
pub(super) fn a_forbidden_type_in_an_extern_block_pub_fn_signature_is_observed() {
    let out = semantic_findings(
        "extern-block-fn-exposure",
        &[
            ("lib.rs", "pub mod infra;\npub mod api;\n"),
            ("infra.rs", "pub struct Secret;\n"),
            (
                "api.rs",
                "extern \"C\" {\n    pub fn handle() -> crate::infra::Secret;\n}\n",
            ),
        ],
        "crate::api",
        &["crate::infra"],
        false,
        &[],
    )
    .unwrap();
    assert_eq!(
        out,
        vec!["crate::infra::Secret exposed by fn crate::api::handle"]
    );
}

/// The identical shape for `pub static` inside an `extern` block.
#[test]
pub(super) fn a_forbidden_type_in_an_extern_block_pub_static_is_observed() {
    let out = semantic_findings(
        "extern-block-static-exposure",
        &[
            ("lib.rs", "pub mod infra;\npub mod api;\n"),
            ("infra.rs", "pub struct Secret;\n"),
            (
                "api.rs",
                "extern \"C\" {\n    pub static S: crate::infra::Secret;\n}\n",
            ),
        ],
        "crate::api",
        &["crate::infra"],
        false,
        &[],
    )
    .unwrap();
    assert_eq!(
        out,
        vec!["crate::infra::Secret exposed by static crate::api::S"]
    );
}

/// A private (no `vis`) `fn`/`static` inside an `extern` block must NOT react — extern-block items
/// default to the enclosing block's own item visibility exactly like a module-level item does, and
/// only `pub` ones are on the module's public surface.
#[test]
pub(super) fn a_non_pub_extern_block_item_is_not_observed() {
    let out = semantic_findings(
        "extern-block-private-not-observed",
        &[
            ("lib.rs", "pub mod infra;\npub mod api;\n"),
            ("infra.rs", "pub struct Secret;\n"),
            (
                "api.rs",
                "extern \"C\" {\n    fn handle() -> crate::infra::Secret;\n    static S: crate::infra::Secret;\n}\n",
            ),
        ],
        "crate::api",
        &["crate::infra"],
        false,
        &[],
    )
    .unwrap();
    assert_eq!(out, Vec::<String>::new());
}

/// A forbidden-marker impl's self type reached through a `type X = Y;` alias whose target `Y` is
/// itself a mutually-exclusive `#[cfg]`-gated `use` alias: the self-type landing must react
/// regardless of which cfg branch is declared first. Before the fix, `scan.alias_targets` was a
/// single-valued `HashMap<String, String>` populated via the resolver's then-single-candidate lookup,
/// so only one landing candidate for `X` was ever recorded (found on adversarial review of
/// `hunyi-cfg-branch-use-reexport-merging`).
#[test]
pub(super) fn forbidden_marker_self_type_landing_reacts_when_the_forbidden_alias_is_declared_first()
{
    let out = marker_findings(
        "self-type-landing-cfg-forbidden-first",
        &[
            ("lib.rs", "pub mod domain;\npub mod wire;\n"),
            ("domain.rs", "pub struct Order;\n"),
            (
                "wire.rs",
                "#[cfg(unix)]\nuse crate::domain::Order as Y;\n#[cfg(not(unix))]\nuse crate::domain::NotOrder as Y;\ntype X = Y;\nimpl serde::Serialize for X {}\n",
            ),
        ],
        "crate::domain",
        &["serde::Serialize"],
    )
    .unwrap();
    assert_eq!(
        out,
        ["impl serde::Serialize for crate::wire::X in crate::wire"],
        "the marker-acquisition REACTION now checks every landing candidate (X resolves to the \
         governed crate::domain::Order under one cfg branch), even though the finding's OWNER \
         label renders the self type as written (`X`), a separate, deliberate identity concern \
         from the landing/gating check: {out:?}"
    );
}

/// The identical shape with the resolvable (defined) alias target declared SECOND. Before the fix
/// this silently passed (`Ok([])`): only the first-declared (undefined `NotOrder`) landing was
/// recorded, which fails the `defined` check, so the genuinely governed self type was never seen.
#[test]
pub(super) fn forbidden_marker_self_type_landing_reacts_when_the_forbidden_alias_is_declared_second()
 {
    let out = marker_findings(
        "self-type-landing-cfg-forbidden-second",
        &[
            ("lib.rs", "pub mod domain;\npub mod wire;\n"),
            ("domain.rs", "pub struct Order;\n"),
            (
                "wire.rs",
                "#[cfg(not(unix))]\nuse crate::domain::NotOrder as Y;\n#[cfg(unix)]\nuse crate::domain::Order as Y;\ntype X = Y;\nimpl serde::Serialize for X {}\n",
            ),
        ],
        "crate::domain",
        &["serde::Serialize"],
    )
    .unwrap();
    assert_eq!(
        out,
        ["impl serde::Serialize for crate::wire::X in crate::wire"],
        "the marker-acquisition REACTION now checks every landing candidate (X resolves to the \
         governed crate::domain::Order under one cfg branch), even though the finding's OWNER \
         label renders the self type as written (`X`), a separate, deliberate identity concern \
         from the landing/gating check: {out:?}"
    );
}

/// A forbidden exposure reached through a `type X = Y;` alias whose target `Y` is itself a
/// mutually-exclusive `#[cfg]`-gated `use` alias: the exposure must react regardless of which cfg
/// branch is declared first. Before the fix, `scan.aliases` (the exposure-pipeline `AliasMap`) was
/// populated via the resolver's then-single-candidate lookup even though the map itself was already
/// multi-valued, so only one landing candidate for `X` was ever pushed (found on adversarial
/// review of `hunyi-cfg-branch-use-reexport-merging`).
#[test]
pub(super) fn type_alias_exposure_reacts_when_the_forbidden_alias_is_declared_first() {
    let out = semantic_findings(
        "type-alias-cfg-forbidden-first",
        &[
            ("lib.rs", "pub mod safe;\npub mod infra;\npub mod api;\n"),
            ("safe.rs", "pub struct Handle;\n"),
            ("infra.rs", "pub struct Secret;\n"),
            (
                "api.rs",
                "#[cfg(unix)]\nuse crate::infra::Secret as Y;\n#[cfg(not(unix))]\nuse crate::safe::Handle as Y;\ntype X = Y;\npub fn leak() -> X { loop {} }\n",
            ),
        ],
        "crate::api",
        &["crate::infra"],
        false,
        &[],
    )
    .unwrap();
    assert_eq!(
        out,
        vec!["crate::infra::Secret exposed by fn crate::api::leak"]
    );
}

/// The identical shape with the forbidden alias declared SECOND. Before the fix this silently
/// passed (`Ok([])`).
#[test]
pub(super) fn type_alias_exposure_reacts_when_the_forbidden_alias_is_declared_second() {
    let out = semantic_findings(
        "type-alias-cfg-forbidden-second",
        &[
            ("lib.rs", "pub mod safe;\npub mod infra;\npub mod api;\n"),
            ("safe.rs", "pub struct Handle;\n"),
            ("infra.rs", "pub struct Secret;\n"),
            (
                "api.rs",
                "#[cfg(not(unix))]\nuse crate::safe::Handle as Y;\n#[cfg(unix)]\nuse crate::infra::Secret as Y;\ntype X = Y;\npub fn leak() -> X { loop {} }\n",
            ),
        ],
        "crate::api",
        &["crate::infra"],
        false,
        &[],
    )
    .unwrap();
    assert_eq!(
        out,
        vec!["crate::infra::Secret exposed by fn crate::api::leak"]
    );
}

/// A cfg_attr(path)-hidden module's own `pub use` re-export must still be folded into the
/// crate-wide re-export closure `exposure.rs`'s own `scan_crate` call builds — the same
/// `resolve_child_modules` mechanism fixed by `hunyi-cfg-attr-path-module-loss`. Before the fix,
/// `facade.rs` never entered the crate-wide scan (only reachable via the `cfg_attr`-wrapped
/// `#[path]`'s conventional form), so its re-export was missing from `scan.reexports` and
/// `crate::facade::Secret` never canonicalized to the real, forbidden `crate::infra::Secret`.
#[test]
pub(super) fn signature_coupling_reacts_through_a_cfg_attr_path_hidden_reexport() {
    let out = semantic_findings(
        "cfg-attr-exposure-reexport",
        &[
            (
                "lib.rs",
                "pub mod infra;\n#[cfg_attr(windows, path = \"weird.rs\")]\npub mod facade;\npub mod api;\n",
            ),
            ("infra.rs", "pub struct Secret;\n"),
            ("facade.rs", "pub use crate::infra::Secret;\n"),
            (
                "api.rs",
                "pub fn leak() -> crate::facade::Secret { loop {} }\n",
            ),
        ],
        "crate::api",
        &["crate::infra"],
        false,
        &[],
    )
    .unwrap();
    assert_eq!(
        out,
        vec!["crate::infra::Secret exposed by fn crate::api::leak"]
    );
}

/// `module_resolve.rs::descend`'s targeted, single-module-anchored resolution now follows a
/// `cfg_attr`-wrapped `#[path]` target exactly like `scan::resolve_child_modules`'s crate-wide walk —
/// found on adversarial review of `hunyi-cfg-attr-path-module-loss`, whose own commit claimed this
/// function was "already correct, fails loud on this shape," a claim that did not survive scrutiny:
/// even a LONE such declaration (no resolving sibling at all) previously never followed its target,
/// no matter whether the file existed. Now it does, closing the same false-negative class this
/// change already closed for the crate-wide walk, at this second, previously-unfixed entry point.
#[test]
pub(super) fn cfg_attr_wrapped_path_resolves_through_its_own_target_with_no_sibling_at_all() {
    let out = semantic_findings(
        "cfg-attr-lone-target-resolves",
        &[
            (
                "lib.rs",
                "pub mod infra;\n#[cfg_attr(windows, path = \"win.rs\")]\nmod foo;\n",
            ),
            ("infra.rs", "pub struct Secret;\n"),
            (
                "win.rs",
                "pub fn leak() -> crate::infra::Secret { loop {} }\n",
            ),
        ],
        "crate::foo",
        &["crate::infra"],
        false,
        &[],
    )
    .unwrap();
    assert_eq!(
        out,
        vec!["crate::infra::Secret exposed by fn crate::foo::leak"]
    );
}

/// A `cfg_attr`-wrapped `#[path]` sibling (a mutually-exclusive `#[cfg]`/`cfg_if!` co-declaration of
/// the identical module name) must still react through its own file, not be silently absorbed by a
/// sibling's successful resolution. Before this fix, `has_path_attr`'s `continue` only skipped that
/// one declaration; when ANY sibling for the same name resolved, the empty-branch check that would
/// otherwise trigger a fail-loud error never fired, so the `cfg_attr` target's own file — and
/// everything it exposes — silently vanished with exit 0 instead. Fixed by extending the same union
/// `scan::resolve_child_modules` already applies: the `cfg_attr` target is read alongside the
/// sibling's own resolution, not skipped.
#[test]
pub(super) fn cfg_attr_wrapped_path_sibling_reacts_through_its_own_file_not_absorbed_by_a_sibling()
{
    let out = semantic_findings(
        "cfg-attr-sibling-anchor",
        &[
            ("lib.rs", "pub mod infra;\n#[cfg(windows)]\n#[cfg_attr(target_arch = \"x86\", path = \"foo_x86.rs\")]\nmod foo;\n#[cfg(not(windows))]\nmod foo;\n"),
            ("infra.rs", "pub struct Secret;\n"),
            ("foo.rs", "pub fn safe() {}\n"),
            (
                "foo_x86.rs",
                "pub fn leak() -> crate::infra::Secret { loop {} }\n",
            ),
        ],
        "crate::foo",
        &["crate::infra"],
        false,
        &[],
    )
    .unwrap();
    assert_eq!(
        out,
        vec!["crate::infra::Secret exposed by fn crate::foo::leak"]
    );
}

/// The identical shape via `cfg_if!` arms rather than bare `#[cfg]` siblings.
#[test]
pub(super) fn cfg_attr_wrapped_path_sibling_reacts_through_a_cfg_if_arm() {
    let out = semantic_findings(
        "cfg-attr-sibling-anchor-cfg-if",
        &[
            (
                "lib.rs",
                "pub mod infra;\ncfg_if::cfg_if! {\n    if #[cfg(windows)] {\n        #[cfg_attr(target_arch = \"x86\", path = \"foo_x86.rs\")]\n        mod foo;\n    } else {\n        mod foo;\n    }\n}\n",
            ),
            ("infra.rs", "pub struct Secret;\n"),
            ("foo.rs", "pub fn safe() {}\n"),
            (
                "foo_x86.rs",
                "pub fn leak() -> crate::infra::Secret { loop {} }\n",
            ),
        ],
        "crate::foo",
        &["crate::infra"],
        false,
        &[],
    )
    .unwrap();
    assert_eq!(
        out,
        vec!["crate::infra::Secret exposed by fn crate::foo::leak"]
    );
}

/// A LONE `cfg_attr`-wrapped `#[path]` declaration (no resolving sibling) still fails loud when
/// neither the conventional file nor the `cfg_attr` target exists — the genuinely fail-loud case
/// that survives from the original bound, now reached only when `has_backing_source` is false.
#[test]
pub(super) fn cfg_attr_wrapped_path_with_no_sibling_and_no_backing_file_still_fails_loud() {
    let err = semantic_findings(
        "cfg-attr-lone-fails-loud",
        &[(
            "lib.rs",
            "#[cfg_attr(windows, path = \"win.rs\")]\nmod foo;\n",
        )],
        "crate::foo",
        &[],
        false,
        &[],
    )
    .unwrap_err();
    assert!(
        err.contains("not found") || err.contains("could not"),
        "a lone unbacked cfg_attr(path) declaration must still fail loud: {err}"
    );
}

/// A module carrying TWO SEPARATE (not nested) `cfg_attr`-wrapped `#[path]` attributes — one per
/// platform predicate, the natural 3+-way per-platform shim shape — must have EVERY candidate's
/// target read, not only the first-declared one. Found on a fourth adversarial review of
/// `hunyi-cfg-attr-path-module-loss`: `cfg_attr_path_value`'s `find_map` silently returned only the
/// first matching attribute's target, dropping every other stacked candidate — the identical
/// cfg-blind-union false negative this whole change closes, one level deeper (a module can stack
/// attributes, not just nest them). Exercises both `descend` (`module_resolve.rs`, this test) and
/// the crate-wide `resolve_child_modules` (`scan.rs`, shares the identical fixed helper).
#[test]
pub(super) fn stacked_cfg_attr_wrapped_path_attributes_are_all_read_not_only_the_first() {
    let out = semantic_findings(
        "stacked-cfg-attr-path",
        &[
            ("lib.rs", "pub mod infra;\n#[cfg_attr(windows, path = \"win.rs\")]\n#[cfg_attr(target_os = \"macos\", path = \"mac.rs\")]\nmod foo;\n"),
            ("infra.rs", "pub struct Secret;\n"),
            (
                "mac.rs",
                "pub fn leak() -> crate::infra::Secret { loop {} }\n",
            ),
        ],
        "crate::foo",
        &["crate::infra"],
        false,
        &[],
    )
    .unwrap();
    assert_eq!(
        out,
        vec!["crate::infra::Secret exposed by fn crate::foo::leak"]
    );
}

/// Two mutually-exclusive `#[cfg]`-gated `use ... as Name;` declarations for the identical local
/// name, in the same file, must both react (cfg-blind: observation cannot know which is live).
/// Before the fix, the second `use` silently overwrote the first in `UseMap` — a single
/// `HashMap<String, String>` — so the verdict depended on source order rather than on whether
/// either branch's binding was genuinely forbidden.
#[test]
pub(super) fn mutually_exclusive_cfg_gated_use_aliases_both_react() {
    let out = semantic_findings(
        "cfg-use-alias-merge-forbidden-first",
        &[
            ("lib.rs", "pub mod safe;\npub mod infra;\npub mod api;\n"),
            ("safe.rs", "pub struct Handle;\n"),
            ("infra.rs", "pub struct Secret;\n"),
            (
                "api.rs",
                "#[cfg(unix)]\nuse crate::infra::Secret as Handle;\n#[cfg(not(unix))]\nuse crate::safe::Handle;\npub fn leak() -> Handle { loop {} }\n",
            ),
        ],
        "crate::api",
        &["crate::infra"],
        false,
        &[],
    )
    .unwrap();
    assert_eq!(
        out,
        vec!["crate::infra::Secret exposed by fn crate::api::leak"]
    );
}

/// The identical shape with the two `use` declarations in the OPPOSITE order — the forbidden
/// binding declared second. Before the fix this silently passed (`Ok([])`): the resolver took
/// only the first `use`-map candidate, and here that candidate was the non-forbidden one.
#[test]
pub(super) fn mutually_exclusive_cfg_gated_use_aliases_react_regardless_of_declaration_order() {
    let out = semantic_findings(
        "cfg-use-alias-merge-forbidden-second",
        &[
            ("lib.rs", "pub mod safe;\npub mod infra;\npub mod api;\n"),
            ("safe.rs", "pub struct Handle;\n"),
            ("infra.rs", "pub struct Secret;\n"),
            (
                "api.rs",
                "#[cfg(not(unix))]\nuse crate::safe::Handle;\n#[cfg(unix)]\nuse crate::infra::Secret as Handle;\npub fn leak() -> Handle { loop {} }\n",
            ),
        ],
        "crate::api",
        &["crate::infra"],
        false,
        &[],
    )
    .unwrap();
    assert_eq!(
        out,
        vec!["crate::infra::Secret exposed by fn crate::api::leak"]
    );
}

/// The identical `use`-alias collision expressed as a `cfg_if!` macro invocation rather than bare
/// `#[cfg]` attributes — the audit's cited "identical shape via the 0.3.1 transparency" form, now
/// that `cfg_if!` arm bodies are read as real code (a separate, already-closed finding).
#[test]
pub(super) fn mutually_exclusive_cfg_if_use_aliases_both_react() {
    let out = semantic_findings(
        "cfg-if-use-alias-merge",
        &[
            ("lib.rs", "pub mod safe;\npub mod infra;\npub mod api;\n"),
            ("safe.rs", "pub struct Handle;\n"),
            ("infra.rs", "pub struct Secret;\n"),
            (
                "api.rs",
                "cfg_if::cfg_if! { if #[cfg(unix)] { use crate::infra::Secret as Handle; } else { use crate::safe::Handle; } }\npub fn leak() -> Handle { loop {} }\n",
            ),
        ],
        "crate::api",
        &["crate::infra"],
        false,
        &[],
    )
    .unwrap();
    assert_eq!(
        out,
        vec!["crate::infra::Secret exposed by fn crate::api::leak"]
    );
}

/// Two mutually-exclusive `pub use ... as X;` re-export targets for the identical local name, in
/// the same file, must both canonicalize correctly through the crate-wide `ReexportMap` — a facade
/// path reached through either branch's binding must resolve to ITS OWN target, not collapse to
/// whichever branch was declared second. Before the fix, `ReexportMap` was a single
/// `HashMap<String, String>`.
#[test]
pub(super) fn mutually_exclusive_reexport_targets_both_canonicalize_correctly() {
    let out = semantic_findings(
        "reexport-map-merge-forbidden-first",
        &[
            (
                "lib.rs",
                "pub mod safe;\npub mod infra;\npub mod api;\npub mod facade;\n",
            ),
            ("safe.rs", "pub struct Thing;\n"),
            ("infra.rs", "pub struct Secret;\n"),
            (
                "api.rs",
                "#[cfg(unix)]\npub use crate::infra::Secret as Handle;\n#[cfg(not(unix))]\npub use crate::safe::Thing as Handle;\n",
            ),
            ("facade.rs", "pub fn f() -> crate::api::Handle { loop {} }\n"),
        ],
        "crate::facade",
        &["crate::infra"],
        false,
        &[],
    )
    .unwrap();
    assert_eq!(
        out,
        vec!["crate::infra::Secret exposed by fn crate::facade::f"]
    );
}

/// The identical re-export collision with the two `pub use` declarations in the OPPOSITE order —
/// the forbidden target declared second.
#[test]
pub(super) fn mutually_exclusive_reexport_targets_react_regardless_of_declaration_order() {
    let out = semantic_findings(
        "reexport-map-merge-forbidden-second",
        &[
            (
                "lib.rs",
                "pub mod safe;\npub mod infra;\npub mod api;\npub mod facade;\n",
            ),
            ("safe.rs", "pub struct Thing;\n"),
            ("infra.rs", "pub struct Secret;\n"),
            (
                "api.rs",
                "#[cfg(not(unix))]\npub use crate::safe::Thing as Handle;\n#[cfg(unix)]\npub use crate::infra::Secret as Handle;\n",
            ),
            ("facade.rs", "pub fn f() -> crate::api::Handle { loop {} }\n"),
        ],
        "crate::facade",
        &["crate::infra"],
        false,
        &[],
    )
    .unwrap();
    assert_eq!(
        out,
        vec!["crate::infra::Secret exposed by fn crate::facade::f"]
    );
}

/// The identical `use`-alias collision, this time exercising `resolve_principal`
/// (`crates/hunyi/src/crate_scope.rs`) — the shared principal-trait resolver dyn-trait and
/// impl-trait's *operand-scoped* boundaries both use (per `matches_forbidden_principal`'s own doc:
/// "Both the single-module and subtree operand reactions use this leaf so their resolution
/// semantics cannot drift"). Discovered while fixing the signature-coupling instance of this bug —
/// not named in the original audit findings, but the identical mechanism, independently reproduced
/// here before being fixed, per this project's own reproduce-before-fixing discipline.
/// A cfg_attr(path)-hidden module's own `pub use` re-export must still be observed and folded into
/// the crate-wide re-export closure `resolve_principal` (`crate_scope.rs::extern_resolution`)
/// consumes — the identical `scan_crate` mechanism `dyn_operand_module_findings` and
/// `impl_trait_operand_module_findings` share with signature-coupling, forbidden-marker, and
/// trait-impl-locality, all fixed by the same `resolve_child_modules` change
/// (`hunyi-cfg-attr-path-module-loss`). Independently reproduced here before being folded into
/// that change's verification, per this project's reproduce-before-fixing discipline.
#[test]
pub(super) fn dyn_trait_operand_resolution_reacts_through_a_cfg_attr_path_hidden_reexport() {
    let tree = TempSrcTree::new("dyn-trait-cfg-attr-path-reexport");
    tree.write_all(&[
        (
            "lib.rs",
            "pub mod infra;\n#[cfg_attr(windows, path = \"weird.rs\")]\npub mod facade;\npub mod api;\n",
        ),
        ("infra.rs", "pub trait Port {}\n"),
        ("facade.rs", "pub use crate::infra::Port;\n"),
        (
            "api.rs",
            "pub fn f() -> Box<dyn crate::facade::Port> { loop {} }\n",
        ),
    ]);
    let out = crate::dyn_trait::dyn_operand_module_findings(
        tree.src(),
        &tree.root(),
        "crate::api",
        &["crate::infra::Port".to_string()],
        "x",
        &[],
    )
    .unwrap();
    assert_eq!(out.len(), 1, "{out:?}");
    assert_eq!(
        out[0].0,
        crate::finding::SemanticFact::Exposed {
            kind: crate::finding::ExposureKind::DynTrait,
            subject: "dyn crate::facade::Port".to_string(),
            seam: crate::finding::PublicSeam::FreeFn {
                module: "crate::api".to_string(),
                name: "f".to_string(),
            },
        }
    );
}

#[test]
pub(super) fn dyn_trait_operand_resolution_reacts_regardless_of_cfg_gated_use_alias_order() {
    let tree = TempSrcTree::new("dyn-trait-principal-cfg-use-merge");
    tree.write_all(&[
        ("lib.rs", "pub mod safe;\npub mod infra;\npub mod api;\n"),
        ("safe.rs", "pub trait SafePort {}\n"),
        ("infra.rs", "pub trait Port {}\n"),
        (
            "api.rs",
            "#[cfg(not(unix))]\nuse crate::safe::SafePort as P;\n#[cfg(unix)]\nuse crate::infra::Port as P;\npub fn f() -> Box<dyn P> { loop {} }\n",
        ),
    ]);
    let out = crate::dyn_trait::dyn_operand_module_findings(
        tree.src(),
        &tree.root(),
        "crate::api",
        &["crate::infra::Port".to_string()],
        "x",
        &[],
    )
    .unwrap();
    assert_eq!(out.len(), 1, "{out:?}");
    assert_eq!(
        out[0].0,
        crate::finding::SemanticFact::Exposed {
            kind: crate::finding::ExposureKind::DynTrait,
            subject: "dyn P".to_string(),
            seam: crate::finding::PublicSeam::FreeFn {
                module: "crate::api".to_string(),
                name: "f".to_string(),
            },
        }
    );
}

/// The identical shape at impl-trait's own operand-scoped boundary — same shared
/// `resolve_principal`/`matches_forbidden_principal` leaf as the dyn-trait test above.
#[test]
pub(super) fn impl_trait_operand_resolution_reacts_regardless_of_cfg_gated_use_alias_order() {
    let tree = TempSrcTree::new("impl-trait-principal-cfg-use-merge");
    tree.write_all(&[
        ("lib.rs", "pub mod safe;\npub mod infra;\npub mod api;\n"),
        ("safe.rs", "pub trait SafePort {}\n"),
        ("infra.rs", "pub trait Port {}\n"),
        (
            "api.rs",
            "#[cfg(not(unix))]\nuse crate::safe::SafePort as P;\n#[cfg(unix)]\nuse crate::infra::Port as P;\npub fn f() -> impl P { loop {} }\n",
        ),
    ]);
    let out = crate::impl_trait::impl_trait_operand_module_findings(
        tree.src(),
        &tree.root(),
        "crate::api",
        &["crate::infra::Port".to_string()],
        "x",
        &[],
    )
    .unwrap();
    assert_eq!(out.len(), 1, "{out:?}");
    assert_eq!(
        out[0].0,
        crate::finding::SemanticFact::Exposed {
            kind: crate::finding::ExposureKind::ImplTrait,
            subject: "impl P".to_string(),
            seam: crate::finding::PublicSeam::FreeFn {
                module: "crate::api".to_string(),
                name: "f".to_string(),
            },
        }
    );
}

/// The reverse cfg declaration order of `dyn_trait_operand_resolution_reacts_regardless_of_cfg_gated_use_alias_order`
/// above: `#[cfg(unix)]` written first, `#[cfg(not(unix))]` second. Both orderings must resolve
/// the alias identically — a merge that silently favored "whichever `use` was declared first"
/// would pass the original test and still be wrong.
#[test]
pub(super) fn dyn_trait_operand_resolution_reacts_regardless_of_cfg_gated_use_alias_order_reversed()
{
    let tree = TempSrcTree::new("dyn-trait-principal-cfg-use-merge-reversed");
    tree.write_all(&[
        ("lib.rs", "pub mod safe;\npub mod infra;\npub mod api;\n"),
        ("safe.rs", "pub trait SafePort {}\n"),
        ("infra.rs", "pub trait Port {}\n"),
        (
            "api.rs",
            "#[cfg(unix)]\nuse crate::infra::Port as P;\n#[cfg(not(unix))]\nuse crate::safe::SafePort as P;\npub fn f() -> Box<dyn P> { loop {} }\n",
        ),
    ]);
    let out = crate::dyn_trait::dyn_operand_module_findings(
        tree.src(),
        &tree.root(),
        "crate::api",
        &["crate::infra::Port".to_string()],
        "x",
        &[],
    )
    .unwrap();
    assert_eq!(out.len(), 1, "{out:?}");
    assert_eq!(
        out[0].0,
        crate::finding::SemanticFact::Exposed {
            kind: crate::finding::ExposureKind::DynTrait,
            subject: "dyn P".to_string(),
            seam: crate::finding::PublicSeam::FreeFn {
                module: "crate::api".to_string(),
                name: "f".to_string(),
            },
        }
    );
}

/// The reverse cfg declaration order of `impl_trait_operand_resolution_reacts_regardless_of_cfg_gated_use_alias_order`
/// above — see the dyn-trait twin's doc comment for why both orders must be pinned separately.
#[test]
pub(super) fn impl_trait_operand_resolution_reacts_regardless_of_cfg_gated_use_alias_order_reversed()
 {
    let tree = TempSrcTree::new("impl-trait-principal-cfg-use-merge-reversed");
    tree.write_all(&[
        ("lib.rs", "pub mod safe;\npub mod infra;\npub mod api;\n"),
        ("safe.rs", "pub trait SafePort {}\n"),
        ("infra.rs", "pub trait Port {}\n"),
        (
            "api.rs",
            "#[cfg(unix)]\nuse crate::infra::Port as P;\n#[cfg(not(unix))]\nuse crate::safe::SafePort as P;\npub fn f() -> impl P { loop {} }\n",
        ),
    ]);
    let out = crate::impl_trait::impl_trait_operand_module_findings(
        tree.src(),
        &tree.root(),
        "crate::api",
        &["crate::infra::Port".to_string()],
        "x",
        &[],
    )
    .unwrap();
    assert_eq!(out.len(), 1, "{out:?}");
    assert_eq!(
        out[0].0,
        crate::finding::SemanticFact::Exposed {
            kind: crate::finding::ExposureKind::ImplTrait,
            subject: "impl P".to_string(),
            seam: crate::finding::PublicSeam::FreeFn {
                module: "crate::api".to_string(),
                name: "f".to_string(),
            },
        }
    );
}

// --- body-nested impl observation (const-eval trick / fn-body sibling) ----
//
// `const _: () = { impl Foo { … } };` is a common "const-eval trick" idiom (forcing a
// compile-time trait assertion or a doctest/dogfooding scratch impl); `fn _also() { impl Foo {
// … } }` is its fn-body-nested sibling. Both wrap a real `impl` block — inherent or trait —
// inside a body that every capability below previously treated as opaque, the same way it
// correctly treats a body-nested `mod` (see `async_subtree_does_not_observe_a_body_nested_module`
// above) as unreachable. Unlike a `mod`, an `impl` is not scoped by where it is lexically
// written — Rust binds it to its self type's own coherence set regardless of nesting — so
// `Svc::leak`/`Svc::run` below are real, externally callable public API the instant `Svc` itself
// is module-level, and every capability had a genuine false negative here. See
// `syn_util::body_nested_impls` for the extraction and its stated one-level/`const`-or-`fn`-only
// bound; the tests after the reaction cases below pin that bound so it does not silently widen.

#[test]
pub(super) fn signature_coupling_reacts_on_a_const_wrapped_inherent_impl() {
    assert_eq!(
        findings(
            "body-nested-const-signature",
            &[
                ("lib.rs", "pub mod infra;\npub mod api;\n"),
                ("infra.rs", "pub struct Db;\n"),
                (
                    "api.rs",
                    "pub struct Svc;\nconst _: () = {\n    impl Svc {\n        pub fn leak(&self) -> crate::infra::Db { unimplemented!() }\n    }\n};\n",
                ),
            ],
            "crate::api",
            &["crate::infra"],
        )
        .unwrap(),
        ["crate::infra::Db exposed by fn <crate::api::Svc>::leak"],
    );
}

#[test]
pub(super) fn signature_coupling_reacts_on_a_fn_body_wrapped_inherent_impl() {
    assert_eq!(
        findings(
            "body-nested-fnbody-signature",
            &[
                ("lib.rs", "pub mod infra;\npub mod api;\n"),
                ("infra.rs", "pub struct Db;\n"),
                (
                    "api.rs",
                    "pub struct Svc;\nfn _also() {\n    impl Svc {\n        pub fn leak(&self) -> crate::infra::Db { unimplemented!() }\n    }\n}\n",
                ),
            ],
            "crate::api",
            &["crate::infra"],
        )
        .unwrap(),
        ["crate::infra::Db exposed by fn <crate::api::Svc>::leak"],
    );
}

#[test]
pub(super) fn async_exposure_reacts_on_a_const_wrapped_inherent_impl() {
    assert_eq!(
        async_mod(
            "body-nested-const-async",
            "pub struct Svc;\nconst _: () = {\n    impl Svc {\n        pub async fn run(&self) {}\n    }\n};\n",
        )
        .unwrap(),
        ["async fn <crate::m::Svc>::run(&self)"],
    );
}

#[test]
pub(super) fn async_exposure_reacts_on_a_fn_body_wrapped_inherent_impl() {
    assert_eq!(
        async_mod(
            "body-nested-fnbody-async",
            "pub struct Svc;\nfn _also() {\n    impl Svc {\n        pub async fn run(&self) {}\n    }\n}\n",
        )
        .unwrap(),
        ["async fn <crate::m::Svc>::run(&self)"],
    );
}

#[test]
pub(super) fn dyn_trait_reacts_on_a_const_wrapped_inherent_impl() {
    assert_eq!(
        dyn_mod(
            "body-nested-const-dyn",
            "pub struct Svc;\nconst _: () = {\n    impl Svc {\n        pub fn dynamic(&self) -> Box<dyn crate::Port> { unimplemented!() }\n    }\n};\n",
        )
        .unwrap(),
        ["dyn crate::Port exposed by fn <crate::m::Svc>::dynamic"],
    );
}

#[test]
pub(super) fn dyn_trait_reacts_on_a_fn_body_wrapped_inherent_impl() {
    assert_eq!(
        dyn_mod(
            "body-nested-fnbody-dyn",
            "pub struct Svc;\nfn _also() {\n    impl Svc {\n        pub fn dynamic(&self) -> Box<dyn crate::Port> { unimplemented!() }\n    }\n}\n",
        )
        .unwrap(),
        ["dyn crate::Port exposed by fn <crate::m::Svc>::dynamic"],
    );
}

#[test]
pub(super) fn impl_trait_reacts_on_a_const_wrapped_inherent_impl() {
    assert_eq!(
        impl_trait_mod(
            "body-nested-const-impltrait",
            "pub struct Svc;\nconst _: () = {\n    impl Svc {\n        pub fn existential(&self) -> impl crate::Port { unimplemented!() }\n    }\n};\n",
        )
        .unwrap(),
        ["impl crate::Port exposed by fn <crate::m::Svc>::existential"],
    );
}

#[test]
pub(super) fn impl_trait_reacts_on_a_fn_body_wrapped_inherent_impl() {
    assert_eq!(
        impl_trait_mod(
            "body-nested-fnbody-impltrait",
            "pub struct Svc;\nfn _also() {\n    impl Svc {\n        pub fn existential(&self) -> impl crate::Port { unimplemented!() }\n    }\n}\n",
        )
        .unwrap(),
        ["impl crate::Port exposed by fn <crate::m::Svc>::existential"],
    );
}

#[test]
pub(super) fn trait_impl_locality_reacts_on_a_const_wrapped_trait_impl() {
    let out = locality_findings(
        "body-nested-const-locality",
        &[
            (
                "lib.rs",
                "pub mod command;\npub mod commands;\npub mod rogue;\n",
            ),
            ("command.rs", "pub trait Command { fn run(&self); }\n"),
            (
                "commands.rs",
                "pub struct Ok1;\nimpl crate::command::Command for Ok1 { fn run(&self) {} }\n",
            ),
            (
                "rogue.rs",
                "pub struct Rogue;\nconst _: () = {\n    impl crate::command::Command for Rogue {\n        fn run(&self) {}\n    }\n};\n",
            ),
        ],
        "crate::command::Command",
        &["crate::commands"],
    )
    .unwrap();
    assert_eq!(
        out,
        ["crate::rogue (impl crate::command::Command for crate::rogue::Rogue)"],
    );
}

#[test]
pub(super) fn trait_impl_locality_reacts_on_a_fn_body_wrapped_trait_impl() {
    let out = locality_findings(
        "body-nested-fnbody-locality",
        &[
            (
                "lib.rs",
                "pub mod command;\npub mod commands;\npub mod rogue;\n",
            ),
            ("command.rs", "pub trait Command { fn run(&self); }\n"),
            (
                "commands.rs",
                "pub struct Ok1;\nimpl crate::command::Command for Ok1 { fn run(&self) {} }\n",
            ),
            (
                "rogue.rs",
                "pub struct Rogue2;\nfn _also() {\n    impl crate::command::Command for Rogue2 {\n        fn run(&self) {}\n    }\n}\n",
            ),
        ],
        "crate::command::Command",
        &["crate::commands"],
    )
    .unwrap();
    assert_eq!(
        out,
        ["crate::rogue (impl crate::command::Command for crate::rogue::Rogue2)"],
    );
}

#[test]
pub(super) fn forbidden_marker_reacts_on_a_const_wrapped_hand_impl() {
    // The impl form shares `scan.impls` with trait-impl-locality, so it closes the identical
    // gap for `ForbiddenMarkerBoundary`'s hand-impl acquisition form.
    let out = marker_findings(
        "body-nested-const-marker",
        &[
            ("lib.rs", "pub mod domain;\npub mod wire;\n"),
            ("domain.rs", "pub struct Order;\n"),
            (
                "wire.rs",
                "const _: () = {\n    impl serde::Serialize for crate::domain::Order {}\n};\n",
            ),
        ],
        "crate::domain",
        &["serde::Serialize"],
    )
    .unwrap();
    assert_eq!(
        out,
        ["impl serde::Serialize for crate::domain::Order in crate::wire"],
    );
}

#[test]
pub(super) fn forbidden_marker_reacts_on_a_fn_body_wrapped_hand_impl() {
    let out = marker_findings(
        "body-nested-fnbody-marker",
        &[
            ("lib.rs", "pub mod domain;\npub mod wire;\n"),
            ("domain.rs", "pub struct Order;\n"),
            (
                "wire.rs",
                "fn _also() {\n    impl serde::Serialize for crate::domain::Order {}\n}\n",
            ),
        ],
        "crate::domain",
        &["serde::Serialize"],
    )
    .unwrap();
    assert_eq!(
        out,
        ["impl serde::Serialize for crate::domain::Order in crate::wire"],
    );
}

// --- body-nested impl observation: the control and the stated scope bounds ---

#[test]
pub(super) fn signature_coupling_control_the_identical_unwrapped_impl_also_reacts() {
    // Control: proves the fixture shape is sound on its own (not a false pass from an unrelated
    // fixture error) — the identical `leak()` written as an ordinary top-level inherent impl.
    assert_eq!(
        findings(
            "body-nested-control-unwrapped",
            &[
                ("lib.rs", "pub mod infra;\npub mod api;\n"),
                ("infra.rs", "pub struct Db;\n"),
                (
                    "api.rs",
                    "pub struct Svc;\nimpl Svc {\n    pub fn leak(&self) -> crate::infra::Db { unimplemented!() }\n}\n",
                ),
            ],
            "crate::api",
            &["crate::infra"],
        )
        .unwrap(),
        ["crate::infra::Db exposed by fn <crate::api::Svc>::leak"],
    );
}

#[test]
pub(super) fn a_plain_fn_directly_in_a_const_body_stays_a_stated_bound() {
    // Scope bound: `body_nested_impls` extracts ONLY `impl` blocks. A plain `pub fn` written
    // directly in a const/fn body (no enclosing `impl`) is genuinely scoped to that body and
    // unreachable as `crate::…` — exactly like the existing body-nested-`mod` bound — so it must
    // stay unobserved; recovering it would be a NEW, unaudited claim, not the fix this change
    // makes.
    assert_eq!(
        findings(
            "body-nested-bound-plain-fn",
            &[
                ("lib.rs", "pub mod infra;\npub mod api;\n"),
                ("infra.rs", "pub struct Db;\n"),
                (
                    "api.rs",
                    "const _: () = {\n    pub fn also_hidden() -> crate::infra::Db { unimplemented!() }\n};\n",
                ),
            ],
            "crate::api",
            &["crate::infra"],
        )
        .unwrap(),
        Vec::<String>::new(),
    );
}

#[test]
pub(super) fn an_impl_nested_one_level_further_stays_a_stated_bound() {
    // Scope bound: only an `impl` that is a DIRECT statement of the const/fn's own outermost
    // block is recovered. One level further in (here, inside an `if` block within the fn) is
    // out of scope — the audited trigger shapes are both exactly one level deep, and recursing
    // into arbitrary expression trees would invent tolerance for a shape nobody has shown.
    assert_eq!(
        findings(
            "body-nested-bound-two-levels",
            &[
                ("lib.rs", "pub mod infra;\npub mod api;\n"),
                ("infra.rs", "pub struct Db;\n"),
                (
                    "api.rs",
                    "pub struct Svc;\nfn _also() {\n    if true {\n        impl Svc {\n            pub fn leak(&self) -> crate::infra::Db { unimplemented!() }\n        }\n    }\n}\n",
                ),
            ],
            "crate::api",
            &["crate::infra"],
        )
        .unwrap(),
        Vec::<String>::new(),
    );
}

#[test]
pub(super) fn a_static_wrapped_impl_stays_a_stated_bound() {
    // Scope bound: only `const`/`fn` bodies are inspected, not `static`. The const-eval trick is
    // specifically about `const` (compile-time evaluation even when never read); no audited
    // idiom uses `static` for it, so widening to `static` would be unaudited tolerance.
    assert_eq!(
        findings(
            "body-nested-bound-static",
            &[
                ("lib.rs", "pub mod infra;\npub mod api;\n"),
                ("infra.rs", "pub struct Db;\n"),
                (
                    "api.rs",
                    "pub struct Svc;\nstatic S: () = {\n    impl Svc {\n        pub fn leak(&self) -> crate::infra::Db { unimplemented!() }\n    }\n};\n",
                ),
            ],
            "crate::api",
            &["crate::infra"],
        )
        .unwrap(),
        Vec::<String>::new(),
    );
}

#[test]
pub(super) fn a_struct_directly_in_a_const_body_stays_a_stated_bound() {
    // Scope bound: `body_nested_impls` extracts ONLY `impl` blocks. A struct DEFINITION written
    // directly in a const/fn body has no enclosing `impl` to recover and is genuinely scoped to
    // that body -- exactly like the plain-fn bound above -- so it must stay unobserved;
    // recovering it would be a NEW, unaudited claim, not the fix this change makes.
    assert_eq!(
        findings(
            "body-nested-bound-struct",
            &[
                ("lib.rs", "pub mod infra;\npub mod api;\n"),
                ("infra.rs", "pub struct Db;\n"),
                (
                    "api.rs",
                    "const _: () = {\n    pub struct AlsoHidden { pub field: crate::infra::Db }\n};\n",
                ),
            ],
            "crate::api",
            &["crate::infra"],
        )
        .unwrap(),
        Vec::<String>::new(),
    );
}

#[test]
pub(super) fn a_mod_directly_in_a_const_body_stays_a_stated_bound() {
    // Scope bound: `body_nested_impls` extracts ONLY `impl` blocks. A `mod` declared directly in
    // a const/fn body is unreachable as `crate::...` for the same reason a body-nested struct is
    // (no enclosing `impl` to recover) -- this recovery's own bound, distinct from
    // `async_subtree_does_not_observe_a_body_nested_module`'s unrelated async-subtree coverage,
    // which was the only pre-existing test to even incidentally touch a body-nested `mod`.
    assert_eq!(
        findings(
            "body-nested-bound-mod",
            &[
                ("lib.rs", "pub mod infra;\npub mod api;\n"),
                ("infra.rs", "pub struct Db;\n"),
                (
                    "api.rs",
                    "const _: () = {\n    pub mod inner {\n        pub struct AlsoHidden { pub field: crate::infra::Db }\n    }\n};\n",
                ),
            ],
            "crate::api",
            &["crate::infra"],
        )
        .unwrap(),
        Vec::<String>::new(),
    );
}
