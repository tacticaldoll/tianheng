//! The 渾儀 finding vocabulary and seam labels — how a semantic finding and the public **seam**
//! it sits at are rendered and identified, in one place. A typed semantic fact owns the stable
//! descriptor used by `(target, rule, finding_key)`; rendering that descriptor as human text is a
//! separate step, so presentation can change without silently changing baseline identity.

use crate::resolve::{ShapeExposure, strip_raw, type_to_string};
use xuanji::{Finding, FindingKey};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SemanticFactKind {
    SignatureExposure,
    DynTrait,
    ImplTrait,
    AsyncExposure,
    Visibility,
    TraitImpl,
    ForbiddenMarker,
    UnsafeSite,
}

impl SemanticFactKind {
    fn code(self) -> &'static str {
        match self {
            Self::SignatureExposure => "signature_exposure",
            Self::DynTrait => "dyn_trait_exposure",
            Self::ImplTrait => "impl_trait_exposure",
            Self::AsyncExposure => "async_exposure",
            Self::Visibility => "visibility_exposure",
            Self::TraitImpl => "trait_impl_site",
            Self::ForbiddenMarker => "forbidden_marker_acquisition",
            Self::UnsafeSite => "unsafe_site",
        }
    }
}

pub(crate) struct SemanticFact {
    kind: SemanticFactKind,
    /// A canonical, dimension-owned description of the observed semantic fact. This is machine
    /// identity even though its current spelling is also a useful default presentation.
    descriptor: String,
}

impl SemanticFact {
    pub(crate) fn new(kind: SemanticFactKind, descriptor: String) -> Self {
        Self { kind, descriptor }
    }

    pub(crate) fn into_finding(self) -> Finding {
        let text = self.descriptor.clone();
        self.into_finding_with_text(text)
    }

    fn into_finding_with_text(self, text: String) -> Finding {
        let key = FindingKey::new(
            "hunyi",
            self.kind.code(),
            [("descriptor", self.descriptor.as_str())],
        )
        .expect("hunyi fact schemas use non-empty, unique static field names");
        Finding::new(text, key)
    }
}

/// One exposed type path (signature-coupling), tagged with the public **seam** it was exposed at
/// — the `syn::Path` counterpart of [`ShapeExposure`]'s `seam`. The seam becomes part of the
/// fact descriptor so two distinct seams exposing the *same* forbidden type never collapse to one
/// `(target, rule, finding_key)` baseline entry and mask a new leak (the one forbidden bug).
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
/// Each variant produces the canonical descriptor carried by a typed semantic fact. Every format
/// literal lives here and only here: a reviewer sees the whole vocabulary at once, and a new shape
/// must add a variant rather than sprout an inline `format!`. Its current `Display` is also the
/// default human presentation, but the fact key is built before presentation. Visibility findings
/// are deliberately *not* here: they are a heterogeneous
/// `{visibility} {kind} {name}` item descriptor, already cohesive in `item_finding`, not one
/// canonical relation line.
pub(crate) enum SemanticFinding {
    /// `{subject} exposed by {seam}` — signature-coupling and its re-export / trait-impl depths,
    /// plus the dyn-/impl-trait shapes (`subject` is a canonical type path or a `dyn …`/`impl …`
    /// shape; both render identically). The one exposure literal, shared by the path pipeline
    /// and the shape pipeline.
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
    /// never collapse to one `(target, rule, finding_key)` and mask a new one.
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

#[cfg(test)]
mod fact_tests {
    use super::*;

    #[test]
    fn semantic_fact_shape_and_descriptor_are_both_identity() {
        let signature = SemanticFact::new(
            SemanticFactKind::SignatureExposure,
            "Port exposed by fn crate::api::run".to_string(),
        )
        .into_finding()
        .key()
        .clone();
        let dyn_trait = SemanticFact::new(
            SemanticFactKind::DynTrait,
            "Port exposed by fn crate::api::run".to_string(),
        )
        .into_finding()
        .key()
        .clone();
        let other = SemanticFact::new(
            SemanticFactKind::SignatureExposure,
            "Port exposed by fn crate::api::other".to_string(),
        )
        .into_finding()
        .key()
        .clone();
        assert_ne!(signature, dyn_trait);
        assert_ne!(signature, other);
    }

    #[test]
    fn semantic_fact_presentation_is_not_identity() {
        let original = SemanticFact::new(
            SemanticFactKind::SignatureExposure,
            "Port exposed by fn crate::api::run".to_string(),
        )
        .into_finding_with_text("Port exposed by fn crate::api::run".to_string());
        let polished = SemanticFact::new(
            SemanticFactKind::SignatureExposure,
            "Port exposed by fn crate::api::run".to_string(),
        )
        .into_finding_with_text("fn crate::api::run exposes Port".to_string());
        assert_eq!(original.key(), polished.key());
        assert_ne!(original.text(), polished.text());
    }
}
