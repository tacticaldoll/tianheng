//! 渾儀's shared **name-resolution** layer — the dimension-internal facility both
//! semantic capabilities turn on.
//!
//! Resolution is *observation*, not reaction: it reads source structure to map a path
//! as written into a canonical `crate::…` path. It therefore lives **here, in the
//! semantic dimension** — not in 璇璣 (`xuanji`), which is the dimension-agnostic reaction
//! model and holds no observation engine; and not shared with 圭表 (`guibiao`), whose
//! token scanner must stay `syn`-free to keep the dependency-light core. The two
//! resolvers are intentionally separate (a PROJECT.md decision); this one is `syn`-based.
//!
//! It resolves a name three ways and bounds the rest honestly:
//! - an in-scope `use` (including renamed and path-qualified), and `crate::`/`self`/`super`;
//! - a **bare or relative name against the current module** (a same-module item needs no
//!   `use`) — opt-in via [`BareFallback`], because exposure-governance wants a bare local
//!   name *ignored* while impl-locality must resolve it (the bare name *is* the anchor);
//! - following **local `pub use` re-export chains**, so a path reached through a facade
//!   matches the item it denotes.
//!
//! Out of scope (stated bounds, never a silent claim): glob imports, macro-generated
//! names, and cross-crate re-exports.

use std::collections::HashMap;

use syn::visit::Visit;

/// Each name a `use` brings into a module's scope mapped to its written full path.
pub(crate) type UseMap = HashMap<String, String>;

/// A `pub use` re-export closure: an alias's canonical path → the canonical path it
/// re-exports. Following it to a fixpoint canonicalizes a facade path to the item it
/// denotes.
pub(crate) type ReexportMap = HashMap<String, String>;

/// Whether a bare/relative name (not in the `use`-map, not `crate`/`self`/`super`)
/// resolves against the current module, or is left unresolved.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum BareFallback {
    /// Leave a bare name unresolved (`None`) — exposure-governance's choice: a bare
    /// local name is not the cross-module forbidden type, and resolving it risks a
    /// same-module false positive.
    Ignore,
    /// Resolve a bare name against the current module (`crate::module::Name`) — impl-
    /// locality's choice: the bare name *is* the anchored trait, so leaving it
    /// unresolved would be a false negative.
    CurrentModule,
}

/// Strip a raw-identifier prefix so `r#type` compares as `type`.
pub(crate) fn strip_raw(ident: &str) -> String {
    ident.strip_prefix("r#").unwrap_or(ident).to_string()
}

/// Canonicalize a `::`-delimited path so each raw-identifier segment compares as its
/// plain form.
pub(crate) fn canonical_path_str(path: &str) -> String {
    path.split("::")
        .map(strip_raw)
        .collect::<Vec<_>>()
        .join("::")
}

/// Map each name a `use` brings into the module's scope to its full written path
/// (`use a::b::C` → `C → a::b::C`; `use a::b::C as D` → `D → a::b::C`; `use a::b` →
/// `b → a::b`). Glob imports bring no nameable leaf (a stated bound). Only the module's
/// own `use`s are collected — Rust modules do not inherit ancestor `use`s.
pub(crate) fn collect_uses(items: &[syn::Item]) -> UseMap {
    let mut map = UseMap::new();
    for item in items {
        if let syn::Item::Use(use_item) = item {
            collect_use_tree(&use_item.tree, String::new(), &mut map);
        }
    }
    map
}

fn collect_use_tree(tree: &syn::UseTree, prefix: String, map: &mut UseMap) {
    let join = |prefix: &str, ident: &str| {
        if prefix.is_empty() {
            ident.to_string()
        } else {
            format!("{prefix}::{ident}")
        }
    };
    match tree {
        syn::UseTree::Path(path) => {
            let ident = strip_raw(&path.ident.to_string());
            collect_use_tree(&path.tree, join(&prefix, &ident), map);
        }
        syn::UseTree::Name(name) => {
            let ident = strip_raw(&name.ident.to_string());
            map.insert(ident.clone(), join(&prefix, &ident));
        }
        syn::UseTree::Rename(rename) => {
            let ident = strip_raw(&rename.ident.to_string());
            let alias = strip_raw(&rename.rename.to_string());
            map.insert(alias, join(&prefix, &ident));
        }
        // A glob brings no nameable leaf into the map — a documented out-of-scope bound.
        syn::UseTree::Glob(_) => {}
        syn::UseTree::Group(group) => {
            for item in &group.items {
                collect_use_tree(item, prefix.clone(), map);
            }
        }
    }
}

/// Resolve `crate::`/`self`/`super`-rooted segments to an absolute `crate::…` path,
/// relative to `module` (e.g. `crate::domain`). `None` when the head is not one of those
/// (a `use`-head or bare name — resolved elsewhere). Over-popping past the crate root is
/// unresolvable.
fn resolve_crate_relative(segs: &[String], module: &str) -> Option<String> {
    let head = segs.first()?;
    match head.as_str() {
        "crate" => Some(segs.join("::")),
        "self" | "super" => {
            let mut parts: Vec<&str> = module.split("::").collect();
            let mut i = 0;
            while i < segs.len() {
                match segs[i].as_str() {
                    "self" => i += 1,
                    "super" => {
                        if parts.len() <= 1 {
                            return None;
                        }
                        parts.pop();
                        i += 1;
                    }
                    _ => break,
                }
            }
            let rest = &segs[i..];
            if rest.is_empty() {
                Some(parts.join("::"))
            } else {
                Some(format!("{}::{}", parts.join("::"), rest.join("::")))
            }
        }
        _ => None,
    }
}

/// Resolve a path as written (in a signature or an `impl` header) to a canonical crate
/// path, using the module's in-scope `use`s, `crate::`/`self`/`super` relative to
/// `module`, and — per `bare` — a bare/relative name against the current module. `None`
/// when not resolvable (a glob/external/primitive name under [`BareFallback::Ignore`]) —
/// a stated bound, never a silent claim.
pub(crate) fn resolve_path(
    path: &syn::Path,
    uses: &UseMap,
    module: &str,
    bare: BareFallback,
) -> Option<String> {
    let segs: Vec<String> = path
        .segments
        .iter()
        .map(|s| strip_raw(&s.ident.to_string()))
        .collect();
    let head = segs.first()?;

    if let Some(canonical) = resolve_crate_relative(&segs, module) {
        return Some(canonical);
    }
    match uses.get(head) {
        Some(full) => {
            let rest = &segs[1..];
            let combined = if rest.is_empty() {
                full.clone()
            } else {
                format!("{full}::{}", rest.join("::"))
            };
            // The use-target may itself be `crate`/`self`/`super`-relative (e.g.
            // `use super::x::Y`); canonicalize it against the module so it compares as an
            // absolute path. A bare-headed target (an external crate, edition 2018+) is
            // left as written — it cannot match a local anchor/forbidden path anyway.
            let combined_segs: Vec<String> = combined.split("::").map(strip_raw).collect();
            Some(resolve_crate_relative(&combined_segs, module).unwrap_or(combined))
        }
        None => match bare {
            BareFallback::Ignore => None,
            // A name needs no `use` in its own module: resolve against `module`.
            BareFallback::CurrentModule => {
                if module.is_empty() {
                    Some(format!("crate::{}", segs.join("::")))
                } else {
                    Some(format!("{module}::{}", segs.join("::")))
                }
            }
        },
    }
}

/// Collect the **local** `pub use` (and `pub(crate)`/`pub(in …)`) re-exports declared in
/// `items` (which live in `module`) into `out`, keyed by the alias's canonical path. A
/// re-export of an external crate item, or a glob, contributes no local hop. A private
/// `use` is not collected — it is invisible from other modules, so it can only be a
/// same-module name already in that module's [`UseMap`].
pub(crate) fn collect_reexports(items: &[syn::Item], module: &str, out: &mut ReexportMap) {
    for item in items {
        if let syn::Item::Use(use_item) = item {
            if matches!(use_item.vis, syn::Visibility::Inherited) {
                continue;
            }
            let mut local = UseMap::new();
            collect_use_tree(&use_item.tree, String::new(), &mut local);
            for (name, written) in local {
                let alias = format!("{module}::{name}");
                if let Some(target) = canonicalize_use_target(&written, module) {
                    if target != alias {
                        out.insert(alias, target);
                    }
                }
            }
        }
    }
}

/// Canonicalize a `pub use` target written as `crate::`/`self`/`super`-rooted to an
/// absolute path; a bare-headed target re-exports an external crate (edition 2018+) and
/// is out of scope for the local closure.
fn canonicalize_use_target(written: &str, module: &str) -> Option<String> {
    let segs: Vec<String> = written.split("::").map(strip_raw).collect();
    resolve_crate_relative(&segs, module)
}

/// Follow the re-export closure from `path` to a fixpoint, so a facade path becomes the
/// canonical path of the item it denotes. Cycle-guarded.
pub(crate) fn canonicalize_through_reexports(path: &str, reexports: &ReexportMap) -> String {
    let mut current = path.to_string();
    let mut seen = std::collections::HashSet::new();
    while seen.insert(current.clone()) {
        match reexports.get(&current) {
            Some(next) => current = next.clone(),
            None => break,
        }
    }
    current
}

/// A Visitor collecting every type path and trait-bound path within a syntax node, so a
/// forbidden type nested in a generic argument (`Vec<crate::infra::Pool>`) or named in a
/// bound (`T: crate::infra::Pooled`) is observed too.
#[derive(Default)]
pub(crate) struct PathCollector {
    pub(crate) paths: Vec<syn::Path>,
}

impl<'ast> Visit<'ast> for PathCollector {
    fn visit_type_path(&mut self, node: &'ast syn::TypePath) {
        self.paths.push(node.path.clone());
        syn::visit::visit_type_path(self, node);
    }

    fn visit_trait_bound(&mut self, node: &'ast syn::TraitBound) {
        self.paths.push(node.path.clone());
        syn::visit::visit_trait_bound(self, node);
    }
}

/// A Visitor recording every **trait-object (`dyn`) node** within a syntax node, at any
/// depth — the leaf observation for `dyn-trait-boundary`. Distinct from [`PathCollector`]:
/// that one accumulates resolvable *paths* and **erases the `dyn` wrapper** (for
/// `Box<dyn crate::Port>` it keeps `Box<…>` and `crate::Port`, not the `dyn`-ness), so
/// dyn-shape governance needs its own collector that records the wrapper node itself,
/// rendered as a stable finding string. Overriding `visit_type_trait_object` fires for a
/// `dyn` nested anywhere — `Box<dyn …>`, `&dyn …`, `Vec<Box<dyn …>>`, an `impl Trait`'s
/// type arguments — so detection is any-depth by construction.
#[derive(Default)]
pub(crate) struct DynCollector {
    pub(crate) dyns: Vec<String>,
}

impl<'ast> Visit<'ast> for DynCollector {
    fn visit_type_trait_object(&mut self, node: &'ast syn::TypeTraitObject) {
        self.dyns.push(trait_object_to_string(node));
        syn::visit::visit_type_trait_object(self, node);
    }
}

/// Render a `dyn` trait-object to a stable finding string (`dyn crate::Port`,
/// `dyn Port + Send`, `dyn Fn(i32) -> i32`, `dyn Iterator<Item = u8>`) — never `quote`/`syn`'s
/// `printing` feature. The render is **injective for every realistic exposed `dyn`** (the
/// closure family, associated-type bindings, lifetimes, simple const generics, macro-named and
/// fn-pointer arguments all render their distinguishing payload), so two structurally-different
/// trait objects never collide into one finding and mask a new exposure under the
/// `(target, rule, finding)` baseline identity (the one forbidden bug). A genuinely
/// unrenderable sub-node — a complex const-generic *expression*, a same-named macro with
/// different arguments, a `verbatim` type — is a **stated rendering bound** (it contributes a
/// `_` and may share a finding with another equally-exotic dyn); this is the same
/// `(target, rule, finding)` render-granularity bound `semantic-trait-impl-locality`'s
/// `(impl for <self_ty>)` finding already carries, never a silent claim of cleanliness.
pub(crate) fn trait_object_to_string(node: &syn::TypeTraitObject) -> String {
    let parts: Vec<String> = node.bounds.iter().map(bound_to_string).collect();
    format!("dyn {}", parts.join(" + "))
}

/// Render one `TypeParamBound` (a trait bound with its `?`/path, or a lifetime) for a finding.
/// Shared by [`trait_object_to_string`] and the `Constraint` generic-argument arm so a `dyn`
/// and an `Iterator<Item: Bound>` render the same way. An unrenderable trait path yields `_`
/// (the stated rendering bound).
fn bound_to_string(bound: &syn::TypeParamBound) -> String {
    match bound {
        syn::TypeParamBound::Trait(tb) => {
            let modifier = match tb.modifier {
                syn::TraitBoundModifier::Maybe(_) => "?",
                _ => "",
            };
            let path = path_to_string(&tb.path).unwrap_or_else(|| "_".to_string());
            format!("{modifier}{path}")
        }
        syn::TypeParamBound::Lifetime(lt) => format!("'{}", strip_raw(&lt.ident.to_string())),
        _ => "_".to_string(),
    }
}

/// Render a const-generic expression argument (`Foo<3>`, `Foo<N>`) for a finding — the common
/// literal and path forms; a complex const expression is a stated bound (`None`).
fn expr_to_string(expr: &syn::Expr) -> Option<String> {
    match expr {
        syn::Expr::Lit(lit) => match &lit.lit {
            syn::Lit::Int(i) => Some(i.base10_digits().to_string()),
            syn::Lit::Bool(b) => Some(b.value.to_string()),
            syn::Lit::Char(c) => Some(format!("{:?}", c.value())),
            syn::Lit::Str(s) => Some(format!("{:?}", s.value())),
            _ => None,
        },
        syn::Expr::Path(p) => path_to_string(&p.path),
        _ => None,
    }
}

/// Render one angle-bracketed generic argument, keeping each kind's distinguishing payload so
/// `Iterator<Item = u8>` and `Iterator<Item = u16>` do not collide. A kind it cannot stably
/// render (a complex const expression) returns `None`, which propagates so the whole path is a
/// stated rendering bound rather than a silently-distinct collapse.
fn generic_argument_to_string(arg: &syn::GenericArgument) -> Option<String> {
    match arg {
        syn::GenericArgument::Type(ty) => type_to_string(ty),
        syn::GenericArgument::Lifetime(lt) => Some(format!("'{}", strip_raw(&lt.ident.to_string()))),
        syn::GenericArgument::AssocType(binding) => Some(format!(
            "{} = {}",
            strip_raw(&binding.ident.to_string()),
            type_to_string(&binding.ty)?
        )),
        syn::GenericArgument::AssocConst(binding) => Some(format!(
            "{} = {}",
            strip_raw(&binding.ident.to_string()),
            expr_to_string(&binding.value)?
        )),
        syn::GenericArgument::Const(expr) => expr_to_string(expr),
        syn::GenericArgument::Constraint(c) => {
            let bounds: Vec<String> = c.bounds.iter().map(bound_to_string).collect();
            Some(format!("{}: {}", strip_raw(&c.ident.to_string()), bounds.join(" + ")))
        }
        _ => None,
    }
}

/// Render a `syn::Type` to a stable string for a finding, reusing path-segment joining —
/// **never** `quote`/`syn`'s `printing` feature, which would breach 渾儀's dependency
/// allowlist. Covers the common shapes; a shape it cannot render returns `None`, and the
/// caller falls back to a location-only finding identity.
pub(crate) fn type_to_string(ty: &syn::Type) -> Option<String> {
    match ty {
        syn::Type::Path(tp) => {
            let mut out = String::new();
            if let Some(qself) = &tp.qself {
                out.push('<');
                out.push_str(&type_to_string(&qself.ty)?);
                out.push('>');
                out.push_str("::");
            }
            out.push_str(&path_to_string(&tp.path)?);
            Some(out)
        }
        syn::Type::Reference(r) => {
            let inner = type_to_string(&r.elem)?;
            if r.mutability.is_some() {
                Some(format!("&mut {inner}"))
            } else {
                Some(format!("&{inner}"))
            }
        }
        syn::Type::Tuple(t) => {
            let parts: Option<Vec<String>> = t.elems.iter().map(type_to_string).collect();
            Some(format!("({})", parts?.join(", ")))
        }
        syn::Type::Slice(s) => Some(format!("[{}]", type_to_string(&s.elem)?)),
        syn::Type::Array(a) => Some(format!("[{}; _]", type_to_string(&a.elem)?)),
        syn::Type::Group(g) => type_to_string(&g.elem),
        syn::Type::Paren(p) => type_to_string(&p.elem),
        // A nested trait-object renders through the same `dyn …` form, so a `dyn` hidden inside
        // another type (`Box<dyn crate::Foo<Box<dyn crate::Bar>>>`) keeps a distinct, stable
        // finding rather than collapsing to a degenerate placeholder.
        syn::Type::TraitObject(t) => Some(trait_object_to_string(t)),
        syn::Type::Ptr(p) => {
            let inner = type_to_string(&p.elem)?;
            if p.mutability.is_some() {
                Some(format!("*mut {inner}"))
            } else {
                Some(format!("*const {inner}"))
            }
        }
        syn::Type::Never(_) => Some("!".to_string()),
        // A macro-typed argument renders by its macro *name* (`bar!`), so `dyn Foo<bar!()>` and
        // `dyn Foo<baz!()>` stay distinct; the macro's *arguments* are unobservable without
        // expansion (the stated macro bound), so two `bar!(…)` with different args share a render.
        syn::Type::Macro(m) => Some(format!("{}!", path_to_string(&m.mac.path)?)),
        syn::Type::BareFn(f) => {
            let inputs: Option<Vec<String>> = f.inputs.iter().map(|a| type_to_string(&a.ty)).collect();
            let output = match &f.output {
                syn::ReturnType::Default => String::new(),
                syn::ReturnType::Type(_, ty) => format!(" -> {}", type_to_string(ty)?),
            };
            Some(format!("fn({}){output}", inputs?.join(", ")))
        }
        // An impl-Trait, `verbatim`, or other exotic self-type is not rendered; the caller falls
        // back to a location-only (trait-impl) or `_`-bound (dyn) finding — a stated bound.
        _ => None,
    }
}

/// Render a `syn::Path` (idents joined by `::`, with angle-bracketed type arguments) for
/// a finding string. `None` for a shape it cannot render (e.g. parenthesized `Fn` args).
fn path_to_string(path: &syn::Path) -> Option<String> {
    let mut segs = Vec::with_capacity(path.segments.len());
    if path.leading_colon.is_some() {
        segs.push(String::new());
    }
    for seg in &path.segments {
        let ident = strip_raw(&seg.ident.to_string());
        match &seg.arguments {
            syn::PathArguments::None => segs.push(ident),
            syn::PathArguments::AngleBracketed(args) => {
                let rendered: Option<Vec<String>> =
                    args.args.iter().map(generic_argument_to_string).collect();
                segs.push(format!("{ident}<{}>", rendered?.join(", ")));
            }
            // A parenthesized `Fn(…) -> …` argument list (the boxed-closure family — the most
            // common exposed trait object) renders to its full shape, so `dyn Fn(i32) -> i32`
            // and `dyn FnMut(String) -> bool` stay **distinct** findings instead of both
            // collapsing to a degenerate placeholder that would collide under the baseline.
            syn::PathArguments::Parenthesized(args) => {
                let inputs: Option<Vec<String>> = args.inputs.iter().map(type_to_string).collect();
                let output = match &args.output {
                    syn::ReturnType::Default => String::new(),
                    syn::ReturnType::Type(_, ty) => format!(" -> {}", type_to_string(ty)?),
                };
                segs.push(format!("{ident}({}){output}", inputs?.join(", ")));
            }
        }
    }
    Some(segs.join("::"))
}
