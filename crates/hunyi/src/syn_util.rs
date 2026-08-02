//! Small `syn`-level predicates and renderers shared across capabilities and sibling modules:
//! the `#[path]` attribute probe, the bare-`pub` visibility test, and the public-item /
//! `use`-tree descriptions the visibility capability reports. Pure `syn` reading; the only
//! non-`syn` dependency is [`crate::resolve::strip_raw`] for raw-identifier canonicalization.

use crate::resolve::strip_raw;

/// The file path of an **unconditional** `#[path = "…"]` remap (the direct name-value form only),
/// or `None`. This is the value both `crate::scan`'s whole-crate walks and
/// `crate::module_resolve`'s targeted resolver *follow* to observe a relocated module's source
/// (closing the coverage false negative where its `unsafe` sites / items were silently dropped).
/// A `cfg_attr`-wrapped `path` is deliberately **excluded** here: both walkers instead extract it
/// separately via [`cfg_attr_path_value`] and union it with the conventional file. A module has at
/// most one applied unconditional `#[path]`, so the first match is the value.
pub(crate) fn direct_path_value(attrs: &[syn::Attribute]) -> Option<String> {
    attrs.iter().find_map(|attr| {
        if !attr.path().is_ident("path") {
            return None;
        }
        match &attr.meta {
            syn::Meta::NameValue(syn::MetaNameValue {
                value:
                    syn::Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Str(s),
                        ..
                    }),
                ..
            }) => Some(s.value()),
            _ => None,
        }
    })
}

/// Whether any attribute is a BARE `#[cfg(...)]` — the conservative, predicate-blind "might
/// legitimately be absent on this build" signal a missing conventional file is checked against: a
/// `#[cfg]`-gated `mod x;` whose file genuinely doesn't exist on this platform/feature set is
/// expected, not broken, so a walker tolerates it; an **unconditional** `mod x;` with no backing
/// file is a real, unrecoverable compile error. Shared by both of this crate's module walkers
/// ([`crate::scan::resolve_child_modules`] and [`crate::module_resolve::descend`]) so they agree
/// on this policy rather than silently drifting — the 0.2.2 lesson (found once as an unnoticed
/// divergence between the two).
///
/// Deliberately does **not** match `cfg_attr` (verified against a real `rustc` build): unlike a
/// bare `#[cfg(pred)]`, which removes the whole item when `pred` is false, `#[cfg_attr(pred, …)]`
/// only conditionally applies its wrapped attribute(s) — the `mod` item itself is never removed,
/// so a `#[cfg_attr(pred, allow(dead_code))] mod x;` with no `x.rs` is a genuine compile error
/// (E0583) on every platform, not a legitimate absence. A `cfg_attr` wrapping `path` specifically
/// is a different, already-handled case ([`cfg_attr_path_value`], consulted separately from this
/// absence test). 漏刻's CI-audit scanner independently hand-rolls the identical bare-`cfg`-only
/// distinction for the same reason (`louke::audit::scan::mod_preamble_attrs`).
pub(crate) fn has_cfg_attr(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|attr| attr.path().is_ident("cfg"))
}

/// The one macro whose body this dimension reads as ordinary code: `cfg_if!`. Its arms wrap
/// human-authored items without transforming their identities, so an item written inside an arm is
/// a real declaration of the enclosing module — which is why 圭表 already observes them
/// (`guibiao::module_scan::…::is_transparent_macro_name`) and why 渾儀 must too: the same source
/// otherwise reacts in the static dimension and passes in the semantic one (a measured exposure
/// false negative, the one bug class the core contract forbids).
///
/// Gating on the macro **name** is load-bearing, not conservatism. [`transparent_macro_arm_items`]
/// reads every top-level brace group of the body as an arm, and for an arbitrary macro that is
/// wrong: in `wrap! { impl Foo { pub fn hidden() -> Forbidden { … } } }` the `impl` body's braces
/// ARE a top-level brace group, so the same walk would recover a `fn hidden` the macro may never
/// emit verbatim — a false positive (measured; see the change's `design.md`). Restricting the
/// mechanism to the one macro whose grammar puts items **directly** in braces is what keeps it
/// sound. Another body-wrapping macro is therefore not observed: a stated bound, shared with 圭表
/// and written into the spec rather than left implicit.
fn is_transparent_macro(item: &syn::ItemMacro) -> bool {
    // `ident.is_none()` excludes a definition (`macro_rules! cfg_if { … }`, whose invocation path
    // is `macro_rules`) from ever being read as an invocation of it. Matched on the LAST segment,
    // so the qualified `cfg_if::cfg_if! { … }` spelling counts — the same test 圭表 applies.
    item.ident.is_none()
        && item
            .mac
            .path
            .segments
            .last()
            .is_some_and(|seg| seg.ident == "cfg_if")
}

/// The items of every arm of a transparent macro invocation, in source order.
///
/// `cfg_if!`'s grammar is `if #[cfg(a)] { items } else if #[cfg(b)] { items } else { items }`, so
/// the body's top-level **brace** groups are exactly the arms: a `#[cfg(…)]` predicate is a `#`
/// followed by a *bracket* group, and `if` / `else` are bare identifiers. Each arm is parsed as a
/// [`syn::File`] — the same parse the crate applies to a real source file, so an arm's items are
/// observed identically to top-level ones.
///
/// A body that does not parse as items yields **nothing** rather than failing the scan: a
/// same-named macro that is not `cfg_if!` at all (or a `cfg_if!` invocation whose arm holds
/// statements) is then invisible, which is the pre-existing state for every macro — never a hard
/// error on source rustc accepts. Arms are independent, so one unparseable arm does not cost the
/// others their items.
///
/// **Item position only** — a stated bound, measured. Inside an `impl` or `trait` body `syn` gives an
/// `ImplItem::Macro` / `TraitItem::Macro`, whose arms parse as impl/trait items rather than items and
/// are reached through a different set of walkers, so such an invocation is not flattened and its
/// contents stay unobserved (pinned by `a_cfg_if_inside_an_impl_body_is_a_stated_bound`, declared in
/// the spec, and owned by its own change). A `cfg_if!` in a **function body** never reaches here at
/// all: `syn` places it as a statement, not an item.
fn transparent_macro_arm_items(mac: &syn::Macro) -> Vec<syn::Item> {
    mac.parse_body_with(parse_transparent_arms)
        .unwrap_or_default()
}

fn parse_transparent_arms(input: syn::parse::ParseStream) -> syn::Result<Vec<syn::Item>> {
    let mut items = Vec::new();
    while !input.is_empty() {
        if input.peek(syn::token::Brace) {
            let arm;
            syn::braced!(arm in input);
            match arm.parse::<syn::File>() {
                Ok(file) => items.extend(file.items),
                // Drain the arm buffer: syn reports a partially-consumed nested buffer as an
                // "unexpected token" error against the ENCLOSING parse, which would discard the
                // arms that did parse.
                Err(_) => drain(&arm)?,
            }
        } else {
            skip_token(input)?;
        }
    }
    Ok(items)
}

fn drain(input: syn::parse::ParseStream) -> syn::Result<()> {
    while !input.is_empty() {
        skip_token(input)?;
    }
    Ok(())
}

/// Advance one token tree, whatever it is — the predicate attributes and `if` / `else` keywords
/// between arms. Never names a `proc_macro2` type: 渾儀's dependency surface is `syn` only
/// (`self_governance.rs`'s own crate boundary), so the cursor's token tree is stepped over rather
/// than matched on.
fn skip_token(input: syn::parse::ParseStream) -> syn::Result<()> {
    input.step(|cursor| match cursor.token_tree() {
        Some((_, rest)) => Ok(((), rest)),
        None => Err(cursor.error("unexpected end of macro body")),
    })
}

/// An item observed after transparent-macro flattening, paired with how it was reached.
pub(crate) struct FlatItem {
    pub(crate) item: syn::Item,
    /// Reached through a transparent macro arm, hence **conditionally compiled by construction**:
    /// every `cfg_if!` arm is gated by a predicate in the macro header (the trailing `else` by the
    /// negation of all the others). The module walkers treat this exactly like a bare `#[cfg]` on
    /// the item itself — 圭表's settled rule, adopted rather than re-derived so the two dimensions
    /// cannot disagree on one shape (the 0.2.2 lesson, found once as a silent divergence).
    pub(crate) in_transparent_arm: bool,
}

impl FlatItem {
    fn plain(item: syn::Item) -> Self {
        Self {
            item,
            in_transparent_arm: false,
        }
    }

    fn in_arm(mut self) -> Self {
        self.in_transparent_arm = true;
        self
    }
}

/// `items` with every transparent-macro invocation replaced by its arms' items, recursively (a
/// nested `cfg_if!` inside an arm is flattened too). The invocation itself is dropped: no
/// capability observes `syn::Item::Macro`, so keeping it would only be a duplicate the arms
/// already cover. Idempotent — a flattened list holds no transparent invocation left to expand.
///
/// Flattening is **shallow** with respect to module bodies: an inline `mod x { … }` inside an arm
/// is returned as one item, and the arm tag does not propagate into its body (that body is
/// flattened on its own when the walk descends into it, with `in_transparent_arm` false). This is
/// deliberate — it is exactly how a bare `#[cfg] mod x { … }` already behaves here, where the
/// gate on the outer `mod` does not tolerate an absent file for an inner `mod y;` — so arm
/// membership introduces no divergence from the existing rule.
pub(crate) fn flatten_transparent_macros(items: &[syn::Item]) -> Vec<FlatItem> {
    let mut out = Vec::new();
    for item in items {
        match item {
            syn::Item::Macro(mac) if is_transparent_macro(mac) => {
                let arm_items = transparent_macro_arm_items(&mac.mac);
                out.extend(
                    flatten_transparent_macros(&arm_items)
                        .into_iter()
                        .map(FlatItem::in_arm),
                );
            }
            other => out.push(FlatItem::plain(other.clone())),
        }
    }
    out
}

/// [`flatten_transparent_macros`] for the observers that only read items and never judge a
/// module's absent source file, so arm membership is not theirs to consult.
pub(crate) fn flatten_transparent_macro_items(items: &[syn::Item]) -> Vec<syn::Item> {
    flatten_transparent_macros(items)
        .into_iter()
        .map(|flat| flat.item)
        .collect()
}

type MetaList = syn::punctuated::Punctuated<syn::Meta, syn::Token![,]>;

fn cfg_attr_metas(input: syn::parse::ParseStream) -> syn::Result<MetaList> {
    MetaList::parse_terminated(input)
}

/// Every file path named by a `path = "…"` remap wrapped in `#[cfg_attr(<pred>, …, path = "…")]`
/// (including arbitrarily nested `cfg_attr`) — one module may carry more than one SEPARATE (not
/// nested) `cfg_attr`-wrapped `#[path]` attribute, each gated by its own predicate for a different
/// platform/feature (`#[cfg_attr(windows, path = "win.rs")] #[cfg_attr(target_os = "macos", path =
/// "mac.rs")] mod foo;`), and every one is a candidate a cfg-blind walker must union — taking only
/// the first (found on adversarial review: a `find_map` silently dropped every candidate but the
/// first-declared) would silently drop whichever platform's file wasn't first. Unlike
/// [`direct_path_value`] (the unconditional `#[path = "…"]` form, followed as the sole source), the
/// module declaration itself is never removed by `cfg_attr` (unlike a bare `#[cfg]`) — so these are
/// candidates among several a cfg-blind walker must union: the conventional file may equally be the
/// one a given build actually compiles.
pub(crate) fn cfg_attr_path_values(attrs: &[syn::Attribute]) -> Vec<String> {
    attrs
        .iter()
        .filter(|attr| attr.path().is_ident("cfg_attr"))
        .filter_map(|attr| {
            attr.parse_args_with(cfg_attr_metas)
                .ok()
                .and_then(|metas| applied_metas_path_value(&metas))
        })
        .collect()
}

/// The **applied** metas of a `cfg_attr` (all but the first, which is the predicate): the value of
/// a `path = "…"` name-value among them, or one nested inside a further `cfg_attr`.
fn applied_metas_path_value(metas: &MetaList) -> Option<String> {
    metas.iter().skip(1).find_map(meta_path_value)
}

fn meta_path_value(meta: &syn::Meta) -> Option<String> {
    match meta {
        syn::Meta::NameValue(syn::MetaNameValue {
            path,
            value:
                syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(s),
                    ..
                }),
            ..
        }) if path.is_ident("path") => Some(s.value()),
        syn::Meta::List(list) if list.path.is_ident("cfg_attr") => list
            .parse_args_with(cfg_attr_metas)
            .ok()
            .and_then(|metas| applied_metas_path_value(&metas)),
        _ => None,
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
pub(crate) struct VisibleItem<'a> {
    pub(crate) visibility: &'a syn::Visibility,
    pub(crate) kind: VisibleItemKind,
    pub(crate) name: String,
}

/// The finite visibility-fact vocabulary. Its labels are published `item_kind` wire;
/// keeping the variants typed makes a new governed item kind an explicit compatibility decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum VisibleItemKind {
    Fn,
    Struct,
    Enum,
    Union,
    Type,
    Const,
    Static,
    Trait,
    TraitAlias,
    Mod,
    ExternCrate,
    Use,
}

impl VisibleItemKind {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Fn => "fn",
            Self::Struct => "struct",
            Self::Enum => "enum",
            Self::Union => "union",
            Self::Type => "type",
            Self::Const => "const",
            Self::Static => "static",
            Self::Trait => "trait",
            Self::TraitAlias => "trait_alias",
            Self::Mod => "mod",
            Self::ExternCrate => "extern_crate",
            Self::Use => "use",
        }
    }
}

fn item_observation_parts(item: &syn::Item) -> Option<VisibleItem<'_>> {
    let observed = |visibility, kind, name| VisibleItem {
        visibility,
        kind,
        name,
    };
    match item {
        syn::Item::Fn(i) => Some(observed(
            &i.vis,
            VisibleItemKind::Fn,
            i.sig.ident.to_string(),
        )),
        syn::Item::Struct(i) => Some(observed(
            &i.vis,
            VisibleItemKind::Struct,
            i.ident.to_string(),
        )),
        syn::Item::Enum(i) => Some(observed(&i.vis, VisibleItemKind::Enum, i.ident.to_string())),
        syn::Item::Union(i) => Some(observed(
            &i.vis,
            VisibleItemKind::Union,
            i.ident.to_string(),
        )),
        syn::Item::Type(i) => Some(observed(&i.vis, VisibleItemKind::Type, i.ident.to_string())),
        syn::Item::Const(i) => Some(observed(
            &i.vis,
            VisibleItemKind::Const,
            i.ident.to_string(),
        )),
        syn::Item::Static(i) => Some(observed(
            &i.vis,
            VisibleItemKind::Static,
            i.ident.to_string(),
        )),
        syn::Item::Trait(i) => Some(observed(
            &i.vis,
            VisibleItemKind::Trait,
            i.ident.to_string(),
        )),
        syn::Item::TraitAlias(i) => Some(observed(
            &i.vis,
            VisibleItemKind::TraitAlias,
            i.ident.to_string(),
        )),
        syn::Item::Mod(i) => Some(observed(&i.vis, VisibleItemKind::Mod, i.ident.to_string())),
        syn::Item::ExternCrate(i) => Some(observed(
            &i.vis,
            VisibleItemKind::ExternCrate,
            i.ident.to_string(),
        )),
        syn::Item::Use(i) => Some(observed(
            &i.vis,
            VisibleItemKind::Use,
            format!(
                "{}{}",
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
pub(crate) fn item_observation(
    item: &syn::Item,
    ceiling_rank: u8,
) -> Option<(String, VisibleItemKind, String)> {
    let observed = item_observation_parts(item)?;
    (visibility_rank(observed.visibility) > ceiling_rank).then(|| {
        (
            vis_prefix(observed.visibility),
            observed.kind,
            observed.name,
        )
    })
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
