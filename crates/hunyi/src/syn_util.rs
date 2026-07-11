//! Small `syn`-level predicates and renderers shared across capabilities and sibling modules:
//! the `#[path]` attribute probe, the bare-`pub` visibility test, and the public-item /
//! `use`-tree descriptions the visibility capability reports. Pure `syn` reading; the only
//! non-`syn` dependency is [`crate::resolve::strip_raw`] for raw-identifier canonicalization.

use crate::resolve::strip_raw;

/// Whether a module's attributes remap its source file off the conventional path — the stated
/// `#[path]` coverage bound (such a module is not conventionally file-backed, so the whole-crate
/// walks skip it rather than govern the wrong file). Recognizes **both** the direct
/// `#[path = "…"]` and the combined `#[cfg_attr(<pred>, …, path = "…")]` spelling (equivalent to
/// `#[cfg(<pred>)] #[path = "…"]`), including arbitrarily **nested** `cfg_attr`
/// (`#[cfg_attr(a, cfg_attr(b, path = "…"))]`). Cfg-blind, like the rest of the scan: a
/// `cfg_attr(path)` is treated as a remap whether or not its predicate holds — the conservative
/// choice, since the alternative (governing a same-named conventional file rustc may not compile)
/// is the false-negative class. It matches only a genuine `path = "…"` **name-value** meta (the only
/// valid `#[path]` form), so a `#[cfg_attr(<pred>, deprecated)]` on a normal file module is **not**
/// mistaken for a remap (which would drop a governed module — the inverse false negative).
pub(crate) fn has_path_attr(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(is_path_remap)
}

fn is_path_remap(attr: &syn::Attribute) -> bool {
    if attr.path().is_ident("path") {
        return true;
    }
    // `cfg_attr(<predicate>, attr, …)`: the first meta is the predicate, the rest are attributes
    // applied when it holds. A `path` among them (or nested in a further `cfg_attr`) is a remap.
    if attr.path().is_ident("cfg_attr") {
        if let Ok(metas) = attr.parse_args_with(cfg_attr_metas) {
            return applied_metas_remap(&metas);
        }
    }
    false
}

type MetaList = syn::punctuated::Punctuated<syn::Meta, syn::Token![,]>;

fn cfg_attr_metas(input: syn::parse::ParseStream) -> syn::Result<MetaList> {
    MetaList::parse_terminated(input)
}

/// Whether the **applied** metas of a `cfg_attr` (all but the first, which is the predicate) carry a
/// `path` remap — a `path = "…"` name-value, or one nested inside a further `cfg_attr`.
fn applied_metas_remap(metas: &MetaList) -> bool {
    metas.iter().skip(1).any(meta_is_path_remap)
}

fn meta_is_path_remap(meta: &syn::Meta) -> bool {
    match meta {
        // The only valid `#[path]` form is `path = "…"` (a name-value). A bare `path` or `path(…)`
        // is not a remap — matching guibiao's byte scanner, which requires `path =`.
        syn::Meta::NameValue(nv) => nv.path.is_ident("path"),
        // A nested `cfg_attr(<pred>, …)`: recurse into ITS applied metas.
        syn::Meta::List(list) if list.path.is_ident("cfg_attr") => list
            .parse_args_with(cfg_attr_metas)
            .map(|metas| applied_metas_remap(&metas))
            .unwrap_or(false),
        _ => false,
    }
}

pub(crate) fn is_public(vis: &syn::Visibility) -> bool {
    matches!(vis, syn::Visibility::Public(_))
}

/// The declared-visibility **rank** of an item, most (3) to least (0) visible:
/// `pub`=3 · `pub(crate)`=2 · `pub(super)`=1 · private / `pub(self)`=0. A visibility boundary
/// reacts when an item's rank is strictly above its ceiling.
///
/// A `pub(in P)` form is ranked by its path **matched whole and single-segment**: exactly
/// `crate`→2, `super`→1, `self`→0. **Every other restricted form** — a multi-segment path
/// (e.g. `pub(in super::super)`, which reaches the grandparent's whole subtree, *broader* than
/// `pub(super)`), a leading-`::` path, or an unrecognized single segment — falls to the `_`
/// catch-all and ranks **2 (Crate), a conservative upper bound**: a `pub(in P)` path is always
/// an ancestor module *within the crate*, so the item is at most crate-visible. This upper bound
/// never under-reacts (no false negative); it may over-react under a Super/Module ceiling when
/// the real path is narrow (a stated bound). The catch-all is why we never index `segments[0]`.
pub(crate) fn visibility_rank(vis: &syn::Visibility) -> u8 {
    match vis {
        syn::Visibility::Public(_) => 3,
        syn::Visibility::Restricted(r) => {
            let single = if r.path.leading_colon.is_none() && r.path.segments.len() == 1 {
                r.path.segments.first().map(|s| s.ident.to_string())
            } else {
                None
            };
            match single.as_deref() {
                Some("crate") => 2,
                Some("super") => 1,
                Some("self") => 0,
                _ => 2,
            }
        }
        syn::Visibility::Inherited => 0,
    }
}

/// Render an item's declared-visibility keyword for a finding: `pub`, `pub(crate)`,
/// `pub(super)`, `pub(self)`, or `pub(in a::b)`. `Inherited` (private) never reaches a finding
/// (rank 0 passes every ceiling), so its empty rendering is unreachable.
fn vis_prefix(vis: &syn::Visibility) -> String {
    match vis {
        syn::Visibility::Public(_) => "pub".to_string(),
        syn::Visibility::Restricted(r) => {
            let path: Vec<String> = r
                .path
                .segments
                .iter()
                .map(|s| strip_raw(&s.ident.to_string()))
                .collect();
            let joined = path.join("::");
            // `pub(in crate|super|self)` is equivalent to the keyword form; render it as such.
            if r.in_token.is_some() && !matches!(joined.as_str(), "crate" | "super" | "self") {
                format!("pub(in {joined})")
            } else {
                format!("pub({joined})")
            }
        }
        syn::Visibility::Inherited => String::new(),
    }
}

/// The `(visibility, "kind name")` of a direct item whose visibility this capability governs, or
/// `None` for an item with no governed visibility. The description carries **no** visibility
/// prefix (the caller prepends it, so a bare-`pub` item under the Crate ceiling renders exactly
/// `pub fn foo` as before). `pub use` (including a glob) is observed as a raw `Item::Use`;
/// attribute-derived public surface (`#[macro_export]`, `#[no_mangle]`, `pub macro`) carries no
/// readable visibility keyword and is out of scope (stated bounds; the deferred attribute
/// capability's domain).
fn item_vis_and_desc(item: &syn::Item) -> Option<(&syn::Visibility, String)> {
    match item {
        syn::Item::Fn(i) => Some((&i.vis, format!("fn {}", i.sig.ident))),
        syn::Item::Struct(i) => Some((&i.vis, format!("struct {}", i.ident))),
        syn::Item::Enum(i) => Some((&i.vis, format!("enum {}", i.ident))),
        syn::Item::Union(i) => Some((&i.vis, format!("union {}", i.ident))),
        syn::Item::Type(i) => Some((&i.vis, format!("type {}", i.ident))),
        syn::Item::Const(i) => Some((&i.vis, format!("const {}", i.ident))),
        syn::Item::Static(i) => Some((&i.vis, format!("static {}", i.ident))),
        syn::Item::Trait(i) => Some((&i.vis, format!("trait {}", i.ident))),
        syn::Item::TraitAlias(i) => Some((&i.vis, format!("trait {} (alias)", i.ident))),
        syn::Item::Mod(i) => Some((&i.vis, format!("mod {}", i.ident))),
        syn::Item::ExternCrate(i) => Some((&i.vis, format!("extern crate {}", i.ident))),
        syn::Item::Use(i) => Some((
            &i.vis,
            format!(
                "use {}{}",
                if i.leading_colon.is_some() { "::" } else { "" },
                use_tree_desc(&i.tree)
            ),
        )),
        _ => None,
    }
}

/// Describe a direct item whose declared-visibility rank is **strictly above** `ceiling_rank`
/// (the boundary's ceiling), rendered `{visibility} {kind} {name}`; `None` when the item is at or
/// below the ceiling or has no governed visibility. Under the Crate ceiling (rank 2) only bare
/// `pub` (rank 3) reacts and renders `pub {kind} {name}`, byte-identical to the prior rule.
pub(crate) fn item_finding(item: &syn::Item, ceiling_rank: u8) -> Option<String> {
    let (vis, desc) = item_vis_and_desc(item)?;
    (visibility_rank(vis) > ceiling_rank).then(|| format!("{} {}", vis_prefix(vis), desc))
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
