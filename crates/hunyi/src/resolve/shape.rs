//! 渾儀's type-**shape** layer — the `syn` Visit collectors that observe `dyn`/`impl Trait`
//! and type-path nodes, the [`ShapeExposure`] they yield, and the hand-rolled renderers that
//! turn a `syn` type/path node into a **stable finding string** (never `quote`/`syn`'s
//! `printing` feature, which would breach 渾儀's dependency allowlist). It sits atop the
//! name-resolution layer (its parent [`mod@super`]): it renders and collects shapes, then leans on
//! [`resolve_path`]/[`strip_raw`] to canonicalize the paths it observes.

use syn::visit::Visit;

use crate::finding::PublicSeam;

use super::{BareFallback, UseMap, resolve_path, strip_raw};

/// A Visitor collecting every type path and trait-bound path within a syntax node, so a
/// forbidden type nested in a generic argument (`Vec<crate::infra::Pool>`) or named in a
/// bound (`T: crate::infra::Pooled`) is observed too.
///
/// `shadowed` names the generic **type parameters** in scope. A path whose **leading** segment
/// (non-`::`-rooted, no generic args) equals one of them is a *parameter use* — the bare form
/// (`x: T`) or an associated-type projection off it (`T::Item`) — not a nominal type, so it is NOT
/// collected: were it collected, a parameter named identically to a same-module `type` alias
/// (`fn f<Secret>(x: Secret)` beside `type Secret = crate::infra::Real;`) or a `use … as Secret`
/// import would be misresolved through that alias to its forbidden target — a false positive. A
/// param's *bounds* (`T: crate::infra::X`) name the forbidden type in a segment that is not the
/// param head, so they are multi-segment paths whose head is not a param and stay collected; only a
/// path headed by the param itself is skipped.
#[derive(Default)]
pub(crate) struct PathCollector {
    pub(crate) paths: Vec<syn::Path>,
    shadowed: std::collections::HashSet<String>,
}

impl PathCollector {
    /// A collector that skips uses (bare or associated-type projections) of the given in-scope
    /// generic type parameters (see the type-level doc).
    pub(crate) fn shadowing(shadowed: std::collections::HashSet<String>) -> Self {
        Self {
            paths: Vec::new(),
            shadowed,
        }
    }

    fn is_shadowed_param(&self, path: &syn::Path) -> bool {
        is_shadowed_param_path(path, &self.shadowed)
    }
}

/// A path is a *use* of one of `shadowed`'s generic type parameters when its leading
/// (non-`::`-rooted) segment names one and carries no generic args — true for the bare form (`T`)
/// AND an associated-type projection off it (`T::Item`, `T::Item::Sub`). A projection off a type
/// parameter can never denote a nominal forbidden type, so it must not be collected/resolved as
/// one: were the module to also declare a same-named import alias (`use crate::infra::Secret as
/// T;`, legal since an enclosing `<T>` only lexically shadows it), resolving `T::Item` would
/// misresolve it through the alias to `crate::infra::Secret::Item` — a false positive on code
/// exposing nothing. A genuine multi-segment leak (`crate::infra::X`) has a non-param head and
/// stays a resolvable path. Shared by every self-type/path check that must not resolve a param use
/// through a same-named alias/type — [`PathCollector`]'s own shadowing, and
/// `containment.rs::resolve_self_type`'s impl-self-type check — kept as one function so the two
/// cannot drift on which self-type SHAPE counts as a param use: `resolve_self_type` used to carry
/// its own, narrower, single-segment-only copy of this check (`Path::get_ident()`, which returns
/// `None` for anything but a bare single segment), so a MULTI-segment self type whose leading
/// segment named the impl's own param (`impl<T> Marker for T::Assoc {}`) was never shadowed and
/// still resolved through a same-named alias — the identical false positive this function exists
/// to prevent, one segment deeper (found on a round-10 adversarial review; see `PROJECT.md`'s
/// Decisions).
pub(crate) fn is_shadowed_param_path(
    path: &syn::Path,
    shadowed: &std::collections::HashSet<String>,
) -> bool {
    let Some(head) = path.segments.first() else {
        return false;
    };
    path.leading_colon.is_none()
        && matches!(head.arguments, syn::PathArguments::None)
        && shadowed.contains(&strip_raw(&head.ident.to_string()))
}

impl<'ast> Visit<'ast> for PathCollector {
    fn visit_type_path(&mut self, node: &'ast syn::TypePath) {
        if node.qself.is_some() || !self.is_shadowed_param(&node.path) {
            self.paths.push(node.path.clone());
        }
        syn::visit::visit_type_path(self, node);
    }

    fn visit_trait_bound(&mut self, node: &'ast syn::TraitBound) {
        self.paths.push(node.path.clone());
        syn::visit::visit_trait_bound(self, node);
    }
}

/// One observed shape node — a `dyn Trait` or a returned `impl Trait` — as its rendered shape
/// (the stable finding string) plus its **non-auto trait** paths as written (for operand-scoped
/// matching). Shape-only governance reads `shape`; operand-scoped governance resolves each entry of
/// `principals` (via [`resolve_path`], which reads the path's segment idents and ignores any
/// generic/parenthesized args) against the forbidden set, matching if **any** resolves into it. A
/// `dyn` object has exactly one non-auto trait; a returned `impl Trait` may name several
/// (`impl Foo + Bar`), so this is a list. Shared by the `dyn` and `impl Trait` collectors.
pub(crate) struct ShapeExposure {
    pub(crate) shape: String,
    pub(crate) principals: Vec<syn::Path>,
    /// The public **seam** (the owning item / sub-element) this shape is exposed at, e.g.
    /// `fn crate::api::make` or `field crate::api::Cfg::sink`. `None` as pushed by the visitor
    /// (which sees only the shape node, not its owner); the `collect_item_*` walker stamps it
    /// via [`stamp_seam`] once the owning element is known. It becomes part of the finding so two
    /// distinct seams exposing the *same* shape never collapse to one `(target, rule, finding)`
    /// baseline entry and mask a new leak (the one forbidden bug) — the shape/existential
    /// analogue of async-exposure's owner-qualified identity.
    pub(crate) seam: Option<PublicSeam>,
}

/// Stamp `seam` onto every exposure a position-walker produced — called by `collect_item_*` once
/// the owning item / sub-element (a `fn`, `field`, `variant`, …) is known, since the [`Visit`]
/// collectors observe only the shape node and cannot name its owner.
pub(crate) fn stamp_seam(
    mut exposures: Vec<ShapeExposure>,
    seam: &PublicSeam,
) -> Vec<ShapeExposure> {
    for exposure in &mut exposures {
        exposure.seam = Some(seam.clone());
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
            principals: principal_trait_paths(&node.bounds),
            seam: None,
        });
        syn::visit::visit_type_trait_object(self, node);
    }
}

/// The **non-auto trait** paths among a shape node's bounds — the operands an operand-scoped rule
/// matches against. A `dyn` object has exactly one non-auto (principal) trait; a returned
/// `impl Trait` may name several (`impl Foo + Bar`), so every non-auto trait is returned, not just
/// the first. Auto traits and lifetime bounds are excluded: they are never a forbiddable operand,
/// and — contrary to a "first trait bound" assumption — **an auto trait may be written *before* the
/// principal** (`dyn Send + crate::Port`, `impl Send + Foo`; both valid Rust, only lifetimes are
/// order-constrained), so taking the first trait bound would resolve `Send` and silently pass a
/// forbidden operand (a false negative). Empty when the bounds carry no non-auto trait
/// (`dyn Send`, or lifetimes only) — correctly matching no operand.
///
/// Stated bound: auto traits are recognized by their std leaf name
/// (`Send`/`Sync`/`Unpin`/`UnwindSafe`/`RefUnwindSafe`); a user-defined `auto trait` (unstable) or a
/// local trait shadowing one of those names is out of scope. Each path is returned as written; the
/// caller resolves and canonicalizes it exactly as an exposed type path (segment idents only;
/// generic/parenthesized args on `Iterator<…>` / `Fn(…)` are ignored by [`resolve_path`]). A `dyn`
/// and an `impl Trait` share this — their `bounds` are the same `Punctuated<TypeParamBound>`.
fn principal_trait_paths(
    bounds: &syn::punctuated::Punctuated<syn::TypeParamBound, syn::token::Plus>,
) -> Vec<syn::Path> {
    const AUTO_TRAITS: [&str; 5] = ["Send", "Sync", "Unpin", "UnwindSafe", "RefUnwindSafe"];
    bounds
        .iter()
        .filter_map(|bound| match bound {
            syn::TypeParamBound::Trait(trait_bound) => {
                let leaf = strip_raw(&trait_bound.path.segments.last()?.ident.to_string());
                (!AUTO_TRAITS.contains(&leaf.as_str())).then(|| trait_bound.path.clone())
            }
            _ => None,
        })
        .collect()
}

/// Render a `+`-joined bound list to a stable finding string behind `keyword` — `dyn`
/// ([`trait_object_to_string`]) and `impl` ([`impl_trait_to_string`]) share this single renderer,
/// since a trait object's and an `impl Trait`'s `bounds` are the same `Punctuated<TypeParamBound>`.
/// Never `quote`/`syn`'s `printing` feature. The render is **injective for every realistic exposed
/// shape** (the closure family, associated-type bindings, lifetimes, simple const generics,
/// macro-named and fn-pointer arguments all render their distinguishing payload), so two
/// structurally-different shapes never collide into one finding and mask a new exposure under the
/// `(target, rule, finding)` baseline identity (the one forbidden bug). A genuinely unrenderable
/// sub-node — a complex const-generic *expression*, a same-named macro with different arguments, a
/// `verbatim` type — is a **stated rendering bound** (it contributes a `_` and may share a finding
/// with another equally-exotic shape); this is the same `(target, rule, finding)` render-granularity
/// bound `semantic-trait-impl-locality`'s `(impl <trait> for <self_ty>)` finding already carries,
/// never a silent claim of cleanliness.
fn render_bounds<'a>(
    bounds: impl Iterator<Item = &'a syn::TypeParamBound>,
    keyword: &str,
) -> String {
    let parts: Vec<String> = bounds.map(bound_to_string).collect();
    format!("{keyword} {}", parts.join(" + "))
}

/// Render a `dyn` trait-object to a stable finding string (`dyn crate::Port`, `dyn Port + Send`,
/// `dyn Fn(i32) -> i32`, `dyn Iterator<Item = u8>`) via the shared [`render_bounds`].
fn trait_object_to_string(node: &syn::TypeTraitObject) -> String {
    render_bounds(node.bounds.iter(), "dyn")
}

/// Render an `impl Trait` node to a stable finding string (`impl crate::Port`,
/// `impl Iterator<Item = u8>`, `impl Fn(i32) -> i32`) via the shared [`render_bounds`].
fn impl_trait_to_string(node: &syn::TypeImplTrait) -> String {
    render_bounds(node.bounds.iter(), "impl")
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
            principals: principal_trait_paths(&node.bounds),
            seam: None,
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

/// Canonicalize a `syn::Type` for semantic finding text and identity, reusing path-segment joining.
///
/// Callers store this output in version-2 subject, owner, trait, signature, and label fields, so its
/// exact byte form is published baseline wire rather than presentation-only rendering. It **never**
/// uses `quote`/`syn`'s `printing` feature, which would breach 渾儀's dependency allowlist. Covers
/// the common shapes; a shape it cannot render returns `None`, and the caller falls back to a
/// location-only finding identity.
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
        // Render the length too (`[u8; 4]` vs `[u8; 8]`) so literal-length-differing arrays stay
        // distinct findings. A complex const length (`N + 1`) is unrenderable: keep the ELEMENT type
        // and mark only the length `_` — never propagate `None` for the whole array. Propagating
        // `None` would route the array into the caller's single shared `_` seam bucket, colliding
        // `[u8; N+1]` with `[u16; N*2]` (losing even the element-type distinction) so a baseline
        // masks a second forbidden exposure. `[elem; _]` keeps distinct element types distinct; two
        // complex-length arrays of the SAME element type still share it — the documented
        // render-granularity bound (one finding, never zero), matching the `dyn` `_` bound.
        syn::Type::Array(a) => {
            let elem = type_to_string(&a.elem)?;
            match expr_to_string(&a.len) {
                Some(len) => Some(format!("[{elem}; {len}]")),
                None => Some(format!("[{elem}; _]")),
            }
        }
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
/// produces an internal positional sentinel `_#{ordinal}`. Every public observation path routes
/// that sentinel through `reject_positional_identity`, so unsupported syntax fails loud instead of
/// publishing traversal position. The sentinel exists only to carry renderer failure to that
/// shared reaction without silently collapsing two sites.
pub(crate) fn canonical_self_owner(
    self_ty: &syn::Type,
    uses: &UseMap,
    module: &str,
    ordinal: usize,
    impl_type_params: &std::collections::HashSet<String>,
) -> String {
    if let syn::Type::Path(tp) = self_ty {
        // A self type naming the impl's own generic type parameter (`impl<T> Trait for T {}` /
        // `for T::Assoc {}`) is a parameter use, never a nominal type — it must not resolve through
        // a same-named alias/type in scope, exactly like `containment.rs::resolve_self_type`'s
        // identical shadow (`is_shadowed_param_path`). This label previously carried NO such
        // shadow at all, unconditionally resolving any bare self type via `resolve_path` — not
        // merely a cosmetic mislabel: this owner is part of `SemanticFact::MisplacedImpl`'s finding
        // IDENTITY in `trait_impl.rs`, so two impls that happen to canonicalize to the same owner
        // string dedup together, and a param resolved through an unrelated alias to the SAME
        // target a genuine direct impl also names collapses two distinct trait-impl-locality
        // violations into one reported finding — a real false negative, not just a wrong display
        // string (found on a round-10 adversarial review; see `PROJECT.md`'s Decisions). Falling
        // through to the plain-render/positional-marker path below (skipping resolution entirely)
        // gives the parameter its own stable, alias-independent label instead.
        if tp.qself.is_none() && !is_shadowed_param_path(&tp.path, impl_type_params) {
            if let Some(base) = resolve_path(&tp.path, uses, module, BareFallback::CurrentModule) {
                return match render_last_segment_args(&tp.path) {
                    Some(args) => format!("{base}{args}"),
                    // Base resolved but a generic arg is unrenderable: preserve the readable base
                    // beside the internal sentinel that the observation path rejects.
                    None => format!("{base}<_#{ordinal}>"),
                };
            }
        }
    }
    // A non-path self type: render it if possible, else return the rejected internal sentinel.
    type_to_string(self_ty).unwrap_or_else(|| format!("_#{ordinal}"))
}

/// Canonicalize an impl self type without inventing traversal-position identity.
///
/// An unsupported self type returns `None`; a caller observing a seam that needs the owner can
/// then fail loud instead of publishing `_#ordinal`.
pub(crate) fn canonical_self_owner_without_fallback(
    self_ty: &syn::Type,
    uses: &UseMap,
    module: &str,
    impl_type_params: &std::collections::HashSet<String>,
) -> Option<String> {
    if let syn::Type::Path(tp) = self_ty {
        if tp.qself.is_none() && !is_shadowed_param_path(&tp.path, impl_type_params) {
            if let Some(base) = resolve_path(&tp.path, uses, module, BareFallback::CurrentModule) {
                return Some(format!("{base}{}", render_last_segment_args(&tp.path)?));
            }
        }
    }
    type_to_string(self_ty)
}

/// Canonicalize a path's **last** segment's angle-bracketed generic arguments (`<u8, T>`), `""` when
/// it has none, or `None` when any argument is unrenderable (a complex const-generic expression) or
/// the segment is parenthesized (`Fn(..)`). Used to append a self type's generics to its resolved
/// base path in [`canonical_self_owner`]; the result enters owner key fields and is version-2 wire.
pub(crate) fn render_last_segment_args(path: &syn::Path) -> Option<String> {
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

/// Canonicalize a `syn::Path` (idents joined by `::`, with angle-bracketed type arguments) for
/// semantic finding text and identity. The result enters version-2 subject, trait, marker, and owner
/// fields, so its byte form is baseline wire. `None` for a shape it cannot render (e.g.
/// parenthesized `Fn` args).
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
