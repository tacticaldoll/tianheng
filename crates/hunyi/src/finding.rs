//! The 渾儀 finding vocabulary and seam labels — how a semantic finding and the public **seam**
//! it sits at are rendered and identified, in one place. A finding string is the third component
//! of a violation's `(target, rule, finding)` baseline identity, so every literal that can become
//! one lives here; the collectors and the reaction hearts build findings through these.

use crate::resolve::{ShapeExposure, strip_raw, type_to_string};

/// One exposed type path (signature-coupling), tagged with the public **seam** it was exposed at
/// — the `syn::Path` counterpart of [`ShapeExposure`]'s `seam`. The seam becomes part of the
/// finding so two distinct seams exposing the *same* forbidden type never collapse to one
/// `(target, rule, finding)` baseline entry and mask a new leak (the one forbidden bug).
pub(crate) struct PathExposure {
    pub(crate) seam: String,
    pub(crate) path: syn::Path,
    /// A named public re-export (`pub use`) position vs. a signature/field/type position.
    /// A bare `pub use` head is an external crate by edition-2018+ grammar, so it is resolved
    /// against the raw external-crate set; a bare **type-position** head, by contrast, may be
    /// a local child module of the governed module, so it is resolved against the set with the
    /// module's own child modules excluded (the shadow) — the two need different oracle inputs.
    pub(crate) is_reexport: bool,
}

/// The finding vocabulary of the semantic dimension, rendered in one place.
///
/// A semantic violation's `finding` is the third component of its `(target, rule, finding)`
/// baseline identity, so every format literal that can become a `finding` lives here and only
/// here: a reviewer sees the whole vocabulary at once, and a new finding shape must add a variant
/// rather than sprout an inline `format!`. Behavior-preserving — each variant's `Display` renders
/// byte-identically to the inline format it replaced, and the `*_findings` functions still return
/// `Vec<String>` / `Vec<(String, String)>`, so baseline identity and the injectivity tests are
/// unchanged. Visibility findings are deliberately *not* here: they are a heterogeneous
/// `pub {kind} {name}` item descriptor, already cohesive in `pub_item_description`, not one
/// canonical relation line.
pub(crate) enum SemanticFinding {
    /// `{subject} exposed by {seam}` — signature-coupling and its re-export / trait-impl depths,
    /// plus the dyn-/impl-trait shapes (`subject` is a canonical type path or a `dyn …`/`impl …`
    /// shape; both render identically). The one exposure literal, formerly written twice
    /// (path pipeline + shape pipeline).
    Exposed { subject: String, seam: String },
    /// `{module} (impl {trait} for {owner})` — trait-impl-locality: a trait impl outside its
    /// allowed site. `trait` is the impl's written trait path **with generic arguments** so two
    /// distinct instantiations for the same self type (`impl Convert<u8> for Foo` /
    /// `impl Convert<u16> for Foo`) stay distinct findings and a baseline cannot mask a new one.
    MisplacedImpl {
        module: String,
        trait_ref: String,
        owner: String,
    },
    /// `derive {marker} on {canonical}` — forbidden-marker: a forbidden `#[derive]` on a type.
    ForbiddenDerive { marker: String, canonical: String },
    /// `impl {marker} for {owner} in {module}` — forbidden-marker: a forbidden trait acquired via a
    /// hand-written `impl`. `marker` is the written trait path (with generic args), `owner` the self
    /// type (with generic args), and `module` the impl site — together injective, so two distinct
    /// acquisitions (`impl Marker<u8>`/`impl Marker<u16>`, or the same leaf from different modules)
    /// never collapse to one `(target, rule, finding)` and mask a new one.
    ForbiddenImpl {
        marker: String,
        owner: String,
        module: String,
    },
    /// `async fn {module}::{name}{tail}` — a public free `async fn` (implicit-existential exposure).
    AsyncFreeFn {
        module: String,
        name: String,
        tail: String,
    },
    /// `async fn trait {module}::{trait_name}::{name}{tail}` — a public trait's `async fn` method.
    AsyncTraitMethod {
        module: String,
        trait_name: String,
        name: String,
        tail: String,
    },
    /// `async fn <{owner}>::{name}{tail}` — a public inherent `async fn` method, owner-qualified.
    AsyncInherentMethod {
        owner: String,
        name: String,
        tail: String,
    },
}

impl std::fmt::Display for SemanticFinding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Exposed { subject, seam } => write!(f, "{subject} exposed by {seam}"),
            Self::MisplacedImpl {
                module,
                trait_ref,
                owner,
            } => write!(f, "{module} (impl {trait_ref} for {owner})"),
            Self::ForbiddenDerive { marker, canonical } => {
                write!(f, "derive {marker} on {canonical}")
            }
            Self::ForbiddenImpl {
                marker,
                owner,
                module,
            } => write!(f, "impl {marker} for {owner} in {module}"),
            Self::AsyncFreeFn { module, name, tail } => {
                write!(f, "async fn {module}::{name}{tail}")
            }
            Self::AsyncTraitMethod {
                module,
                trait_name,
                name,
                tail,
            } => write!(f, "async fn trait {module}::{trait_name}::{name}{tail}"),
            Self::AsyncInherentMethod { owner, name, tail } => {
                write!(f, "async fn <{owner}>::{name}{tail}")
            }
        }
    }
}

/// Render a shape exposure (`dyn …` / `impl …`) as its seam-qualified finding string — the
/// shape/existential analogue of signature-coupling's `{type} exposed by {seam}`. Two distinct
/// seams exposing the same shape stay distinct findings (the one forbidden bug), so a baselined
/// exposure never masks a new one at another seam.
pub(crate) fn shape_finding(exposure: ShapeExposure) -> String {
    SemanticFinding::Exposed {
        subject: exposure.shape,
        seam: exposure.seam,
    }
    .to_string()
}

/// Attach `seam` to every path a position-walker produced (the signature-coupling analogue of
/// [`crate::resolve::stamp_seam`]).
pub(crate) fn tag_paths(paths: Vec<syn::Path>, seam: &str) -> Vec<PathExposure> {
    paths
        .into_iter()
        .map(|path| PathExposure {
            seam: seam.to_string(),
            path,
            is_reexport: false,
        })
        .collect()
}

// Seam labels — the public element an exposure lives at, in one vocabulary shared by all three
// 渾儀 exposure collectors (signature-coupling, dyn, impl-trait) and disjoint-by-prefix with
// async-exposure's `async fn …` identities, so no two element kinds ever render the same seam.
// A free fn is `fn {module}::name`; an inherent method `fn <{SelfTy}>::name` (owner-qualified
// like async, so `impl A`/`impl B` methods stay distinct); a trait method `fn trait
// {module}::Trait::name`. A named item (struct/enum/union/trait/type/const/static) is `{kind}
// {module}::name`; a field/variant is `{field|variant} {module}::Owner::name`; a trait associated
// item `{type|const} trait {module}::Trait::name`.

pub(crate) fn fn_seam(module: &str, name: &syn::Ident) -> String {
    format!("fn {module}::{}", strip_raw(&name.to_string()))
}

pub(crate) fn inherent_method_seam(owner: &str, name: &syn::Ident) -> String {
    format!("fn <{owner}>::{}", strip_raw(&name.to_string()))
}

/// The seam for an inherent `impl` block's public associated `const`/`type` — `{kind} <{owner}>::
/// {name}`, parallel to [`inherent_method_seam`]'s `fn <{owner}>::{name}`. Owner-qualified so
/// `impl Foo`/`impl Bar` assoc items of the same name never collide, and `kind`-tagged so a `const`
/// and a `type` (and a method's `fn`) stay distinct findings under the baseline.
pub(crate) fn inherent_assoc_seam(kind: &str, owner: &str, name: &syn::Ident) -> String {
    format!("{kind} <{owner}>::{}", strip_raw(&name.to_string()))
}

pub(crate) fn trait_method_seam(module: &str, trait_name: &str, name: &syn::Ident) -> String {
    format!(
        "fn trait {module}::{trait_name}::{}",
        strip_raw(&name.to_string())
    )
}

pub(crate) fn item_seam(kind: &str, module: &str, name: &syn::Ident) -> String {
    format!("{kind} {module}::{}", strip_raw(&name.to_string()))
}

pub(crate) fn field_seam(kind: &str, module: &str, owner: &str, member: &str) -> String {
    format!("{kind} {module}::{owner}::{member}")
}

pub(crate) fn trait_assoc_seam(
    kind: &str,
    module: &str,
    trait_name: &str,
    name: &syn::Ident,
) -> String {
    format!(
        "{kind} trait {module}::{trait_name}::{}",
        strip_raw(&name.to_string())
    )
}

/// Render a field's member name — a named field's ident, or a tuple field's positional index.
pub(crate) fn member_label(index: usize, field: &syn::Field) -> String {
    match &field.ident {
        Some(ident) => strip_raw(&ident.to_string()),
        None => index.to_string(),
    }
}

/// Render a signature's `(params) -> ret` tail for an owner-qualified finding — for readability and
/// extra collision-margin, NOT to represent the implicit future. Params render each input's type
/// via [`type_to_string`] (a receiver as `self`/`&self`/`&mut self`); the return renders
/// `sig.output`'s written type (empty for `-> ()`); an unrenderable type contributes `_`.
pub(crate) fn render_sig_tail(sig: &syn::Signature) -> String {
    let params: Vec<String> = sig
        .inputs
        .iter()
        .map(|arg| match arg {
            syn::FnArg::Receiver(receiver) => {
                let reference = if receiver.reference.is_some() {
                    "&"
                } else {
                    ""
                };
                let mutability = if receiver.mutability.is_some() {
                    "mut "
                } else {
                    ""
                };
                format!("{reference}{mutability}self")
            }
            syn::FnArg::Typed(pat_type) => {
                type_to_string(&pat_type.ty).unwrap_or_else(|| "_".to_string())
            }
        })
        .collect();
    let ret = match &sig.output {
        syn::ReturnType::Type(_, ty) => {
            format!(
                " -> {}",
                type_to_string(ty).unwrap_or_else(|| "_".to_string())
            )
        }
        syn::ReturnType::Default => String::new(),
    };
    format!("({}){ret}", params.join(", "))
}
