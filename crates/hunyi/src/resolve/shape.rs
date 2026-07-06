//! 渾儀's type-**shape** layer — the `syn` Visit collectors that observe `dyn`/`impl Trait`
//! and type-path nodes, the [`ShapeExposure`] they yield, and the hand-rolled renderers that
//! turn a `syn` type/path node into a **stable finding string** (never `quote`/`syn`'s
//! `printing` feature, which would breach 渾儀's dependency allowlist). It sits atop the
//! name-resolution layer (its parent [`mod@super`]): it renders and collects shapes, then leans on
//! [`resolve_path`]/[`strip_raw`] to canonicalize the paths it observes.

use syn::visit::Visit;

use super::{BareFallback, UseMap, resolve_path, strip_raw};

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

/// One observed shape node — a `dyn Trait` or a returned `impl Trait` — as its rendered shape
/// (the stable finding string) plus its **principal trait** path as written (for operand-scoped
/// matching). Shape-only governance reads `shape`; operand-scoped governance resolves `principal`
/// (via [`resolve_path`], which reads the path's segment idents and ignores any generic/
/// parenthesized args) against the forbidden set. Shared by the `dyn` and `impl Trait` collectors.
pub(crate) struct ShapeExposure {
    pub(crate) shape: String,
    pub(crate) principal: Option<syn::Path>,
    /// The public **seam** (the owning item / sub-element) this shape is exposed at, e.g.
    /// `fn crate::api::make` or `field crate::api::Cfg::sink`. Empty as pushed by the visitor
    /// (which sees only the shape node, not its owner); the `collect_item_*` walker stamps it
    /// via [`stamp_seam`] once the owning element is known. It becomes part of the finding so two
    /// distinct seams exposing the *same* shape never collapse to one `(target, rule, finding)`
    /// baseline entry and mask a new leak (the one forbidden bug) — the shape/existential
    /// analogue of async-exposure's owner-qualified identity.
    pub(crate) seam: String,
}

/// Stamp `seam` onto every exposure a position-walker produced — called by `collect_item_*` once
/// the owning item / sub-element (a `fn`, `field`, `variant`, …) is known, since the [`Visit`]
/// collectors observe only the shape node and cannot name its owner.
pub(crate) fn stamp_seam(mut exposures: Vec<ShapeExposure>, seam: &str) -> Vec<ShapeExposure> {
    for exposure in &mut exposures {
        exposure.seam = seam.to_string();
    }
    exposures
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
    pub(crate) exposures: Vec<ShapeExposure>,
}

impl<'ast> Visit<'ast> for DynCollector {
    fn visit_type_trait_object(&mut self, node: &'ast syn::TypeTraitObject) {
        self.exposures.push(ShapeExposure {
            shape: trait_object_to_string(node),
            principal: principal_trait_path(&node.bounds),
            seam: String::new(),
        });
        syn::visit::visit_type_trait_object(self, node);
    }
}

/// The **principal (base) trait** path of a shape node's bounds: the path of the **first**
/// `TypeParamBound::Trait`. Rust's grammar guarantees the base trait is syntactically first, so
/// any auto-trait (`Send`, `Sync`) or lifetime bound can only follow it and is never the
/// principal — hence "first trait bound", not a name-skip (`dyn Send` / `impl Send` correctly
/// yields `Send`, its own principal). `None` if there is no trait bound at all (only lifetimes).
/// Returned as the `syn::Path` as written; the caller resolves and canonicalizes it exactly as an
/// exposed type path (segment idents only; generic/parenthesized args on `Iterator<…>` / `Fn(…)`
/// are ignored by [`resolve_path`]). A `dyn` and an `impl Trait` share this — their `bounds` are
/// the same `Punctuated<TypeParamBound>`.
fn principal_trait_path(
    bounds: &syn::punctuated::Punctuated<syn::TypeParamBound, syn::token::Plus>,
) -> Option<syn::Path> {
    bounds.iter().find_map(|bound| match bound {
        syn::TypeParamBound::Trait(trait_bound) => Some(trait_bound.path.clone()),
        _ => None,
    })
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
fn trait_object_to_string(node: &syn::TypeTraitObject) -> String {
    let parts: Vec<String> = node.bounds.iter().map(bound_to_string).collect();
    format!("dyn {}", parts.join(" + "))
}

/// Render an `impl Trait` node (`syn::TypeImplTrait`) to a stable finding string
/// (`impl crate::Port`, `impl Iterator<Item = u8>`, `impl Fn(i32) -> i32`) — its `bounds` are the
/// same `Punctuated<TypeParamBound>` a trait object carries, so it renders through the same
/// [`bound_to_string`], sharing the `dyn` renderer's injectivity and rendering bound.
fn impl_trait_to_string(node: &syn::TypeImplTrait) -> String {
    let parts: Vec<String> = node.bounds.iter().map(bound_to_string).collect();
    format!("impl {}", parts.join(" + "))
}

/// A Visitor recording every **`impl Trait` node** within a syntax node, at any depth — the leaf
/// observation for `semantic-impl-trait-boundary`. Fed only a function/method **return type** by
/// the caller (existential positions), so argument-position `impl Trait` (APIT) is never collected.
#[derive(Default)]
pub(crate) struct ImplTraitCollector {
    pub(crate) exposures: Vec<ShapeExposure>,
}

impl<'ast> Visit<'ast> for ImplTraitCollector {
    fn visit_type_impl_trait(&mut self, node: &'ast syn::TypeImplTrait) {
        self.exposures.push(ShapeExposure {
            shape: impl_trait_to_string(node),
            principal: principal_trait_path(&node.bounds),
            seam: String::new(),
        });
        syn::visit::visit_type_impl_trait(self, node);
    }
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
        syn::GenericArgument::Lifetime(lt) => {
            Some(format!("'{}", strip_raw(&lt.ident.to_string())))
        }
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
            Some(format!(
                "{}: {}",
                strip_raw(&c.ident.to_string()),
                bounds.join(" + ")
            ))
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
            let inputs: Option<Vec<String>> =
                f.inputs.iter().map(|a| type_to_string(&a.ty)).collect();
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

/// The canonical, injective owner label for an `impl` block's self type — used to seam-qualify
/// the block's methods (`fn <{owner}>::name`) and its trait-impl-locality finding (`impl for
/// {owner}`), so two distinct impl blocks never collapse to one `(target, rule, finding)` and
/// mask a leak (the one forbidden bug).
///
/// The self type's **base path** is resolved and canonicalized against `uses`/`module`, so
/// `Foo`, `self::Foo`, and `crate::m::Foo` all render to the same identity (no baseline churn from
/// the written token form); its generic arguments are appended as rendered, so `Foo<u8>` and
/// `Foo<u16>` stay distinct. When a part cannot render — a **complex const-generic expression**
/// argument (`Arr<{ N + 1 }>`), or a non-path / `verbatim` / impl-Trait self type — the label
/// falls back to a positional marker `_#{ordinal}` (the impl block's index among the module's
/// items / the scanned impl sites), which stays injective for two otherwise-indistinguishable
/// blocks. `ordinal` is reached only by these rare unrenderable self types and is stable unless
/// the items are reordered — this is the render-granularity bound, now injective rather than a
/// silent collapse (previously the self type wrongly rendered `_`, masking a distinct block).
pub(crate) fn canonical_self_owner(
    self_ty: &syn::Type,
    uses: &UseMap,
    module: &str,
    ordinal: usize,
) -> String {
    if let syn::Type::Path(tp) = self_ty {
        if tp.qself.is_none() {
            if let Some(base) = resolve_path(&tp.path, uses, module, BareFallback::CurrentModule) {
                return match render_last_segment_args(&tp.path) {
                    Some(args) => format!("{base}{args}"),
                    // Base resolved but a generic arg is unrenderable: keep the readable base,
                    // disambiguate the arg by the block's position so two blocks stay distinct.
                    None => format!("{base}<_#{ordinal}>"),
                };
            }
        }
    }
    // A non-path self type: render it if the hand-rolled renderer can, else a positional marker.
    type_to_string(self_ty).unwrap_or_else(|| format!("_#{ordinal}"))
}

/// Render a path's **last** segment's angle-bracketed generic arguments (`<u8, T>`), `""` when it
/// has none, or `None` when any argument is unrenderable (a complex const-generic expression) or
/// the segment is parenthesized (`Fn(..)`). Used to append a self type's generics to its resolved
/// base path in [`canonical_self_owner`].
fn render_last_segment_args(path: &syn::Path) -> Option<String> {
    match &path.segments.last()?.arguments {
        syn::PathArguments::None => Some(String::new()),
        syn::PathArguments::AngleBracketed(args) => {
            let rendered: Option<Vec<String>> =
                args.args.iter().map(generic_argument_to_string).collect();
            Some(format!("<{}>", rendered?.join(", ")))
        }
        syn::PathArguments::Parenthesized(_) => None,
    }
}

/// Render a `syn::Path` (idents joined by `::`, with angle-bracketed type arguments) for
/// a finding string. `None` for a shape it cannot render (e.g. parenthesized `Fn` args).
pub(crate) fn path_to_string(path: &syn::Path) -> Option<String> {
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
