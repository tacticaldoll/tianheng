//! Small `syn`-level predicates and renderers shared across capabilities and sibling modules:
//! the `#[path]` attribute probe, the bare-`pub` visibility test, and the public-item /
//! `use`-tree descriptions the visibility capability reports. Pure `syn` reading; the only
//! non-`syn` dependency is [`crate::resolve::strip_raw`] for raw-identifier canonicalization.

use crate::resolve::strip_raw;

pub(crate) fn has_path_attr(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|attr| attr.path().is_ident("path"))
}

pub(crate) fn is_public(vis: &syn::Visibility) -> bool {
    matches!(vis, syn::Visibility::Public(_))
}

/// Describe a direct item declared bare-`pub` (`Visibility::Public`) by kind and name, or
/// `None` for a non-`pub` item or one with no governed visibility. `pub use` (including a
/// glob) is observed as a raw `Item::Use`; attribute-derived public surface
/// (`#[macro_export]`, `#[no_mangle]`) carries no `pub` keyword and is out of scope.
pub(crate) fn pub_item_description(item: &syn::Item) -> Option<String> {
    match item {
        syn::Item::Fn(i) if is_public(&i.vis) => Some(format!("pub fn {}", i.sig.ident)),
        syn::Item::Struct(i) if is_public(&i.vis) => Some(format!("pub struct {}", i.ident)),
        syn::Item::Enum(i) if is_public(&i.vis) => Some(format!("pub enum {}", i.ident)),
        syn::Item::Union(i) if is_public(&i.vis) => Some(format!("pub union {}", i.ident)),
        syn::Item::Type(i) if is_public(&i.vis) => Some(format!("pub type {}", i.ident)),
        syn::Item::Const(i) if is_public(&i.vis) => Some(format!("pub const {}", i.ident)),
        syn::Item::Static(i) if is_public(&i.vis) => Some(format!("pub static {}", i.ident)),
        syn::Item::Trait(i) if is_public(&i.vis) => Some(format!("pub trait {}", i.ident)),
        syn::Item::TraitAlias(i) if is_public(&i.vis) => {
            Some(format!("pub trait {} (alias)", i.ident))
        }
        syn::Item::Mod(i) if is_public(&i.vis) => Some(format!("pub mod {}", i.ident)),
        syn::Item::ExternCrate(i) if is_public(&i.vis) => {
            Some(format!("pub extern crate {}", i.ident))
        }
        syn::Item::Use(i) if is_public(&i.vis) => Some(format!(
            "pub use {}{}",
            if i.leading_colon.is_some() { "::" } else { "" },
            use_tree_desc(&i.tree)
        )),
        // A `pub macro` (declarative macros 2.0) parses as `Item::Verbatim` with no readable
        // visibility field, and a `#[macro_export] macro_rules!` / `#[no_mangle]` symbol
        // carries no `pub` keyword — all out of this capability's syntactic scope (stated
        // bounds; the deferred attribute capability's domain).
        _ => None,
    }
}

/// Render a `use` tree to a stable description for a finding (`crate::db::Handle`,
/// `crate::db::*`, `a as b`, `{x, y}`), reusing path-segment joining — no `quote`.
fn use_tree_desc(tree: &syn::UseTree) -> String {
    match tree {
        syn::UseTree::Path(p) => {
            format!(
                "{}::{}",
                strip_raw(&p.ident.to_string()),
                use_tree_desc(&p.tree)
            )
        }
        syn::UseTree::Name(n) => strip_raw(&n.ident.to_string()),
        syn::UseTree::Rename(r) => format!(
            "{} as {}",
            strip_raw(&r.ident.to_string()),
            strip_raw(&r.rename.to_string())
        ),
        syn::UseTree::Glob(_) => "*".to_string(),
        syn::UseTree::Group(g) => {
            let inner: Vec<String> = g.items.iter().map(use_tree_desc).collect();
            format!("{{{}}}", inner.join(", "))
        }
    }
}
