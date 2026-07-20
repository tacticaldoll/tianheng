//! The 渾儀 finding vocabulary and seam labels — how a semantic finding and the public **seam**
//! it sits at are rendered and identified, in one place. A typed semantic fact owns the stable
//! named values used by `(target, rule, finding_key)` and renders its human text separately, so
//! presentation can change without silently changing baseline identity.

use crate::resolve::{ShapeExposure, strip_raw, type_to_string};
use xuanji::{Finding, FindingKey};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum ExposureKind {
    Signature,
    DynTrait,
    ImplTrait,
}

impl ExposureKind {
    fn code(self) -> &'static str {
        match self {
            Self::Signature => "signature_exposure",
            Self::DynTrait => "dyn_trait_exposure",
            Self::ImplTrait => "impl_trait_exposure",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum ItemKind {
    Struct,
    Enum,
    Union,
    Type,
    Const,
    Static,
    Trait,
}

impl ItemKind {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Struct => "struct",
            Self::Enum => "enum",
            Self::Union => "union",
            Self::Type => "type",
            Self::Const => "const",
            Self::Static => "static",
            Self::Trait => "trait",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum MemberKind {
    Field,
    Variant,
}

impl MemberKind {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Field => "field",
            Self::Variant => "variant",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum AssocKind {
    Const,
    Type,
}

impl AssocKind {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Const => "const",
            Self::Type => "type",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum TraitImplPosition {
    TraitArg,
    SelfType,
    Where(String),
    Assoc(String),
    MethodReturn(String),
}

impl TraitImplPosition {
    fn key_fields(&self) -> Vec<(&'static str, &str)> {
        match self {
            Self::TraitArg => vec![("seam_position", "trait_arg")],
            Self::SelfType => vec![("seam_position", "self")],
            Self::Where(subject) => {
                vec![
                    ("seam_position", "where"),
                    ("seam_position_subject", subject),
                ]
            }
            Self::Assoc(name) => {
                vec![("seam_position", "assoc"), ("seam_position_name", name)]
            }
            Self::MethodReturn(name) => vec![
                ("seam_position", "method_return"),
                ("seam_position_name", name),
            ],
        }
    }
}

impl std::fmt::Display for TraitImplPosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TraitArg => f.write_str("trait-arg"),
            Self::SelfType => f.write_str("self"),
            Self::Where(subject) => write!(f, "where {subject}"),
            Self::Assoc(name) => write!(f, "assoc {name}"),
            Self::MethodReturn(name) => write!(f, "method {name} return"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum PublicSeam {
    FreeFn {
        module: String,
        name: String,
    },
    InherentMethod {
        owner: String,
        name: String,
    },
    InherentAssoc {
        kind: AssocKind,
        owner: String,
        name: String,
    },
    TraitMethod {
        module: String,
        trait_name: String,
        name: String,
    },
    Item {
        kind: ItemKind,
        module: String,
        name: String,
    },
    Member {
        kind: MemberKind,
        module: String,
        owner: String,
        member: String,
    },
    TraitAssoc {
        kind: AssocKind,
        module: String,
        trait_name: String,
        name: String,
    },
    InherentGenerics {
        owner: String,
    },
    Reexport {
        module: String,
        exported: String,
    },
    ExternCrate {
        name: String,
    },
    TraitImpl {
        trait_ref: String,
        owner: String,
        position: TraitImplPosition,
    },
}

impl PublicSeam {
    fn key_fields(&self) -> Vec<(&'static str, &str)> {
        match self {
            Self::FreeFn { module, name } => vec![
                ("seam_kind", "free_fn"),
                ("seam_module", module),
                ("seam_name", name),
            ],
            Self::InherentMethod { owner, name } => vec![
                ("seam_kind", "inherent_method"),
                ("seam_owner", owner),
                ("seam_name", name),
            ],
            Self::InherentAssoc { kind, owner, name } => vec![
                ("seam_kind", "inherent_assoc"),
                ("seam_item_kind", kind.as_str()),
                ("seam_owner", owner),
                ("seam_name", name),
            ],
            Self::TraitMethod {
                module,
                trait_name,
                name,
            } => vec![
                ("seam_kind", "trait_method"),
                ("seam_module", module),
                ("seam_trait", trait_name),
                ("seam_name", name),
            ],
            Self::Item { kind, module, name } => vec![
                ("seam_kind", "item"),
                ("seam_item_kind", kind.as_str()),
                ("seam_module", module),
                ("seam_name", name),
            ],
            Self::Member {
                kind,
                module,
                owner,
                member,
            } => vec![
                ("seam_kind", "member"),
                ("seam_item_kind", kind.as_str()),
                ("seam_module", module),
                ("seam_owner", owner),
                ("seam_member", member),
            ],
            Self::TraitAssoc {
                kind,
                module,
                trait_name,
                name,
            } => vec![
                ("seam_kind", "trait_assoc"),
                ("seam_item_kind", kind.as_str()),
                ("seam_module", module),
                ("seam_trait", trait_name),
                ("seam_name", name),
            ],
            Self::InherentGenerics { owner } => {
                vec![("seam_kind", "inherent_generics"), ("seam_owner", owner)]
            }
            Self::Reexport { module, exported } => vec![
                ("seam_kind", "reexport"),
                ("seam_module", module),
                ("seam_name", exported),
            ],
            Self::ExternCrate { name } => vec![("seam_kind", "extern_crate"), ("seam_name", name)],
            Self::TraitImpl {
                trait_ref,
                owner,
                position,
            } => {
                let mut fields = vec![
                    ("seam_kind", "trait_impl"),
                    ("seam_trait", trait_ref),
                    ("seam_owner", owner),
                ];
                fields.extend(position.key_fields());
                fields
            }
        }
    }
}

impl std::fmt::Display for PublicSeam {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FreeFn { module, name } => write!(f, "fn {module}::{name}"),
            Self::InherentMethod { owner, name } => write!(f, "fn <{owner}>::{name}"),
            Self::InherentAssoc { kind, owner, name } => {
                write!(f, "{} <{owner}>::{name}", kind.as_str())
            }
            Self::TraitMethod {
                module,
                trait_name,
                name,
            } => write!(f, "fn trait {module}::{trait_name}::{name}"),
            Self::Item { kind, module, name } => write!(f, "{} {module}::{name}", kind.as_str()),
            Self::Member {
                kind,
                module,
                owner,
                member,
            } => write!(f, "{} {module}::{owner}::{member}", kind.as_str()),
            Self::TraitAssoc {
                kind,
                module,
                trait_name,
                name,
            } => write!(f, "{} trait {module}::{trait_name}::{name}", kind.as_str()),
            Self::InherentGenerics { owner } => write!(f, "impl <{owner}> (generics)"),
            Self::Reexport { module, exported } => write!(f, "pub use {module}::{exported}"),
            Self::ExternCrate { name } => write!(f, "pub extern crate {name}"),
            Self::TraitImpl {
                trait_ref,
                owner,
                position,
            } => write!(f, "impl {trait_ref} for {owner} ({position})"),
        }
    }
}

/// One exposed type path (signature-coupling), tagged with the public **seam** it was exposed at
/// — the `syn::Path` counterpart of [`ShapeExposure`]'s `seam`. The seam becomes part of the
/// fact key so two distinct seams exposing the *same* forbidden type never collapse to one
/// `(target, rule, finding_key)` baseline entry and mask a new leak (the one forbidden bug).
pub(crate) struct PathExposure {
    pub(crate) seam: PublicSeam,
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
/// Each variant owns both its named identity-bearing values and its human rendering. Every format
/// literal lives here and only here: a reviewer sees the whole vocabulary at once, and a new shape
/// must add a variant rather than sprout an inline `format!`. `Display` is presentation only; the
/// key conversion below reads the variant's fields directly.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum SemanticFact {
    /// `{subject} exposed by {seam}` — signature-coupling and its re-export / trait-impl depths,
    /// plus the dyn-/impl-trait shapes (`subject` is a canonical type path or a `dyn …`/`impl …`
    /// shape; both render identically). The one exposure literal, shared by the path pipeline
    /// and the shape pipeline.
    Exposed {
        kind: ExposureKind,
        subject: String,
        seam: PublicSeam,
    },
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
    ForbiddenDerive {
        marker: String,
        canonical: String,
    },
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
    Visibility {
        visibility: String,
        item_kind: &'static str,
        item_name: String,
    },
    UnsafeSite {
        label: String,
        module: String,
    },
}

impl std::fmt::Display for SemanticFact {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Exposed { subject, seam, .. } => write!(f, "{subject} exposed by {seam}"),
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
            Self::Visibility {
                visibility,
                item_kind,
                item_name,
            } => match *item_kind {
                "trait_alias" => write!(f, "{visibility} trait {item_name} (alias)"),
                "extern_crate" => write!(f, "{visibility} extern crate {item_name}"),
                kind => write!(f, "{visibility} {kind} {item_name}"),
            },
            Self::UnsafeSite { label, module } => write!(f, "{label} in {module}"),
        }
    }
}

impl SemanticFact {
    pub(crate) fn into_finding(self) -> Finding {
        let text = self.to_string();
        self.into_finding_with_text(text)
    }

    fn into_finding_with_text(self, text: String) -> Finding {
        let (code, fields): (&str, Vec<(&str, &str)>) = match &self {
            SemanticFact::Exposed {
                kind,
                subject,
                seam,
            } => {
                let mut fields = seam.key_fields();
                fields.push(("subject", subject));
                (kind.code(), fields)
            }
            SemanticFact::MisplacedImpl {
                module,
                trait_ref,
                owner,
            } => (
                "trait_impl_site",
                vec![("module", module), ("owner", owner), ("trait", trait_ref)],
            ),
            SemanticFact::ForbiddenDerive { marker, canonical } => (
                "forbidden_marker_acquisition",
                vec![("form", "derive"), ("marker", marker), ("owner", canonical)],
            ),
            SemanticFact::ForbiddenImpl {
                marker,
                owner,
                module,
            } => (
                "forbidden_marker_acquisition",
                vec![
                    ("form", "impl"),
                    ("marker", marker),
                    ("module", module),
                    ("owner", owner),
                ],
            ),
            SemanticFact::AsyncFreeFn { module, name, tail } => (
                "async_exposure",
                vec![
                    ("form", "free_fn"),
                    ("module", module),
                    ("name", name),
                    ("signature", tail),
                ],
            ),
            SemanticFact::AsyncTraitMethod {
                module,
                trait_name,
                name,
                tail,
            } => (
                "async_exposure",
                vec![
                    ("form", "trait_method"),
                    ("module", module),
                    ("name", name),
                    ("signature", tail),
                    ("trait", trait_name),
                ],
            ),
            SemanticFact::AsyncInherentMethod { owner, name, tail } => (
                "async_exposure",
                vec![
                    ("form", "inherent_method"),
                    ("name", name),
                    ("owner", owner),
                    ("signature", tail),
                ],
            ),
            SemanticFact::Visibility {
                visibility,
                item_kind,
                item_name,
            } => (
                "visibility_exposure",
                vec![
                    ("item_kind", item_kind),
                    ("item_name", item_name),
                    ("visibility", visibility),
                ],
            ),
            SemanticFact::UnsafeSite { label, module } => {
                ("unsafe_site", vec![("label", label), ("module", module)])
            }
        };
        let key = FindingKey::of("hunyi", code, fields);
        Finding::new(text, key)
    }
}

/// Deduplicate by typed identity, then restore the historical presentation order. Identity and
/// display are deliberately separate: two distinct facts may one day share polished wording, and
/// one fact must still collapse even if another display-equivalent fact was observed between its
/// occurrences.
pub(crate) fn sort_facts(findings: &mut Vec<SemanticFact>) {
    findings.sort();
    findings.dedup();
    findings.sort_by_cached_key(ToString::to_string);
}

/// The attributed counterpart of [`sort_facts`]. The enclosing module rides beside a fact only for
/// file lookup, so dedup remains fact-identity-only as it was when findings were strings.
pub(crate) fn sort_attributed_facts(findings: &mut Vec<(SemanticFact, String)>) {
    findings.sort_by(|a, b| a.0.cmp(&b.0));
    findings.dedup_by(|a, b| a.0 == b.0);
    findings.sort_by_cached_key(|(finding, module)| (finding.to_string(), module.clone()));
}

/// Render a shape exposure (`dyn …` / `impl …`) as its seam-qualified finding string — the
/// shape/existential analogue of signature-coupling's `{type} exposed by {seam}`. Two distinct
/// seams exposing the same shape stay distinct findings (the one forbidden bug), so a baselined
/// exposure never masks a new one at another seam.
pub(crate) fn shape_finding(exposure: ShapeExposure, kind: ExposureKind) -> SemanticFact {
    SemanticFact::Exposed {
        kind,
        subject: exposure.shape,
        seam: exposure
            .seam
            .expect("a collected shape exposure must have a public seam"),
    }
}

/// Attach `seam` to every path a position-walker produced (the signature-coupling analogue of
/// [`crate::resolve::stamp_seam`]).
pub(crate) fn tag_paths(paths: Vec<syn::Path>, seam: &PublicSeam) -> Vec<PathExposure> {
    paths
        .into_iter()
        .map(|path| PathExposure {
            seam: seam.clone(),
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

pub(crate) fn fn_seam(module: &str, name: &syn::Ident) -> PublicSeam {
    PublicSeam::FreeFn {
        module: module.to_string(),
        name: strip_raw(&name.to_string()),
    }
}

pub(crate) fn inherent_method_seam(owner: &str, name: &syn::Ident) -> PublicSeam {
    PublicSeam::InherentMethod {
        owner: owner.to_string(),
        name: strip_raw(&name.to_string()),
    }
}

/// The seam for an inherent `impl` block's public associated `const`/`type` — `{kind} <{owner}>::
/// {name}`, parallel to [`inherent_method_seam`]'s `fn <{owner}>::{name}`. Owner-qualified so
/// `impl Foo`/`impl Bar` assoc items of the same name never collide, and `kind`-tagged so a `const`
/// and a `type` (and a method's `fn`) stay distinct findings under the baseline.
pub(crate) fn inherent_assoc_seam(kind: AssocKind, owner: &str, name: &syn::Ident) -> PublicSeam {
    PublicSeam::InherentAssoc {
        kind,
        owner: owner.to_string(),
        name: strip_raw(&name.to_string()),
    }
}

pub(crate) fn trait_method_seam(module: &str, trait_name: &str, name: &syn::Ident) -> PublicSeam {
    PublicSeam::TraitMethod {
        module: module.to_string(),
        trait_name: trait_name.to_string(),
        name: strip_raw(&name.to_string()),
    }
}

pub(crate) fn item_seam(kind: ItemKind, module: &str, name: &syn::Ident) -> PublicSeam {
    PublicSeam::Item {
        kind,
        module: module.to_string(),
        name: strip_raw(&name.to_string()),
    }
}

pub(crate) fn field_seam(kind: MemberKind, module: &str, owner: &str, member: &str) -> PublicSeam {
    PublicSeam::Member {
        kind,
        module: module.to_string(),
        owner: owner.to_string(),
        member: member.to_string(),
    }
}

pub(crate) fn trait_assoc_seam(
    kind: AssocKind,
    module: &str,
    trait_name: &str,
    name: &syn::Ident,
) -> PublicSeam {
    PublicSeam::TraitAssoc {
        kind,
        module: module.to_string(),
        trait_name: trait_name.to_string(),
        name: strip_raw(&name.to_string()),
    }
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

    fn exposure(kind: ExposureKind, module: &str, name: &str) -> SemanticFact {
        SemanticFact::Exposed {
            kind,
            subject: "Port".to_string(),
            seam: PublicSeam::FreeFn {
                module: module.to_string(),
                name: name.to_string(),
            },
        }
    }

    #[test]
    fn semantic_fact_shape_and_observed_values_are_both_identity() {
        let facts = [
            exposure(ExposureKind::Signature, "crate::api", "run"),
            exposure(ExposureKind::DynTrait, "crate::api", "run"),
            SemanticFact::Exposed {
                kind: ExposureKind::Signature,
                subject: "OtherPort".into(),
                seam: PublicSeam::FreeFn {
                    module: "crate::api".into(),
                    name: "run".into(),
                },
            },
            exposure(ExposureKind::Signature, "crate::other", "run"),
            exposure(ExposureKind::Signature, "crate::api", "other"),
        ];
        let keys: std::collections::BTreeSet<_> = facts
            .into_iter()
            .map(|fact| fact.into_finding().key().clone())
            .collect();
        assert_eq!(keys.len(), 5);
    }

    #[test]
    fn semantic_fact_presentation_is_not_identity() {
        let original = exposure(ExposureKind::Signature, "crate::api", "run")
            .into_finding_with_text("Port exposed by fn crate::api::run".to_string());
        let polished = exposure(ExposureKind::Signature, "crate::api", "run")
            .into_finding_with_text("fn crate::api::run exposes Port".to_string());
        assert_eq!(original.key(), polished.key());
        assert_ne!(original.text(), polished.text());
    }

    #[test]
    fn every_public_seam_shape_is_named_and_identity_injective() {
        let seams = vec![
            PublicSeam::FreeFn {
                module: "crate::api".into(),
                name: "run".into(),
            },
            PublicSeam::InherentMethod {
                owner: "crate::Api".into(),
                name: "run".into(),
            },
            PublicSeam::InherentAssoc {
                kind: AssocKind::Const,
                owner: "crate::Api".into(),
                name: "VALUE".into(),
            },
            PublicSeam::InherentAssoc {
                kind: AssocKind::Type,
                owner: "crate::Api".into(),
                name: "Value".into(),
            },
            PublicSeam::TraitMethod {
                module: "crate::api".into(),
                trait_name: "Port".into(),
                name: "run".into(),
            },
            PublicSeam::Item {
                kind: ItemKind::Struct,
                module: "crate::api".into(),
                name: "Api".into(),
            },
            PublicSeam::Item {
                kind: ItemKind::Enum,
                module: "crate::api".into(),
                name: "Api".into(),
            },
            PublicSeam::Item {
                kind: ItemKind::Union,
                module: "crate::api".into(),
                name: "Api".into(),
            },
            PublicSeam::Item {
                kind: ItemKind::Type,
                module: "crate::api".into(),
                name: "Api".into(),
            },
            PublicSeam::Item {
                kind: ItemKind::Const,
                module: "crate::api".into(),
                name: "API".into(),
            },
            PublicSeam::Item {
                kind: ItemKind::Static,
                module: "crate::api".into(),
                name: "API".into(),
            },
            PublicSeam::Item {
                kind: ItemKind::Trait,
                module: "crate::api".into(),
                name: "Api".into(),
            },
            PublicSeam::Member {
                kind: MemberKind::Field,
                module: "crate::api".into(),
                owner: "Api".into(),
                member: "port".into(),
            },
            PublicSeam::Member {
                kind: MemberKind::Variant,
                module: "crate::api".into(),
                owner: "Api::Port".into(),
                member: "0".into(),
            },
            PublicSeam::TraitAssoc {
                kind: AssocKind::Const,
                module: "crate::api".into(),
                trait_name: "Port".into(),
                name: "VALUE".into(),
            },
            PublicSeam::TraitAssoc {
                kind: AssocKind::Type,
                module: "crate::api".into(),
                trait_name: "Port".into(),
                name: "Value".into(),
            },
            PublicSeam::InherentGenerics {
                owner: "crate::Api".into(),
            },
            PublicSeam::Reexport {
                module: "crate::api".into(),
                exported: "Port".into(),
            },
            PublicSeam::ExternCrate {
                name: "port".into(),
            },
            PublicSeam::TraitImpl {
                trait_ref: "crate::Port".into(),
                owner: "crate::Api".into(),
                position: TraitImplPosition::TraitArg,
            },
            PublicSeam::TraitImpl {
                trait_ref: "crate::Port".into(),
                owner: "crate::Api".into(),
                position: TraitImplPosition::SelfType,
            },
            PublicSeam::TraitImpl {
                trait_ref: "crate::Port".into(),
                owner: "crate::Api".into(),
                position: TraitImplPosition::Where("T".into()),
            },
            PublicSeam::TraitImpl {
                trait_ref: "crate::Port".into(),
                owner: "crate::Api".into(),
                position: TraitImplPosition::Assoc("Value".into()),
            },
            PublicSeam::TraitImpl {
                trait_ref: "crate::Port".into(),
                owner: "crate::Api".into(),
                position: TraitImplPosition::MethodReturn("run".into()),
            },
        ];
        let keys: std::collections::BTreeSet<_> = seams
            .iter()
            .cloned()
            .map(|seam| {
                SemanticFact::Exposed {
                    kind: ExposureKind::Signature,
                    subject: "Port".into(),
                    seam,
                }
                .into_finding()
                .key()
                .clone()
            })
            .collect();
        assert_eq!(keys.len(), seams.len());
        for key in keys {
            let fields: Vec<_> = key.fields().collect();
            assert!(fields.iter().any(|(name, _)| *name == "seam_kind"));
            assert!(fields.iter().any(|(name, _)| *name == "subject"));
            assert!(!fields.iter().any(|(name, _)| *name == "descriptor"));
        }
    }

    #[test]
    fn every_semantic_fact_family_has_its_exact_named_identity_schema() {
        let cases = vec![
            (
                SemanticFact::MisplacedImpl {
                    module: "crate::m".into(),
                    trait_ref: "crate::Port".into(),
                    owner: "crate::Api".into(),
                },
                "trait_impl_site",
                vec![
                    ("module", "crate::m"),
                    ("owner", "crate::Api"),
                    ("trait", "crate::Port"),
                ],
            ),
            (
                SemanticFact::ForbiddenDerive {
                    marker: "Marker".into(),
                    canonical: "crate::Api".into(),
                },
                "forbidden_marker_acquisition",
                vec![
                    ("form", "derive"),
                    ("marker", "Marker"),
                    ("owner", "crate::Api"),
                ],
            ),
            (
                SemanticFact::ForbiddenImpl {
                    marker: "Marker".into(),
                    owner: "crate::Api".into(),
                    module: "crate::m".into(),
                },
                "forbidden_marker_acquisition",
                vec![
                    ("form", "impl"),
                    ("marker", "Marker"),
                    ("module", "crate::m"),
                    ("owner", "crate::Api"),
                ],
            ),
            (
                SemanticFact::AsyncFreeFn {
                    module: "crate::m".into(),
                    name: "run".into(),
                    tail: "()".into(),
                },
                "async_exposure",
                vec![
                    ("form", "free_fn"),
                    ("module", "crate::m"),
                    ("name", "run"),
                    ("signature", "()"),
                ],
            ),
            (
                SemanticFact::AsyncTraitMethod {
                    module: "crate::m".into(),
                    trait_name: "Port".into(),
                    name: "run".into(),
                    tail: "()".into(),
                },
                "async_exposure",
                vec![
                    ("form", "trait_method"),
                    ("module", "crate::m"),
                    ("name", "run"),
                    ("signature", "()"),
                    ("trait", "Port"),
                ],
            ),
            (
                SemanticFact::AsyncInherentMethod {
                    owner: "crate::Api".into(),
                    name: "run".into(),
                    tail: "()".into(),
                },
                "async_exposure",
                vec![
                    ("form", "inherent_method"),
                    ("name", "run"),
                    ("owner", "crate::Api"),
                    ("signature", "()"),
                ],
            ),
            (
                SemanticFact::Visibility {
                    visibility: "pub".into(),
                    item_kind: "fn",
                    item_name: "run".into(),
                },
                "visibility_exposure",
                vec![
                    ("item_kind", "fn"),
                    ("item_name", "run"),
                    ("visibility", "pub"),
                ],
            ),
            (
                SemanticFact::UnsafeSite {
                    label: "unsafe fn run".into(),
                    module: "crate::m".into(),
                },
                "unsafe_site",
                vec![("label", "unsafe fn run"), ("module", "crate::m")],
            ),
        ];
        for (fact, code, expected_fields) in cases {
            let finding = fact.into_finding();
            let fields: Vec<_> = finding.key().fields().collect();
            assert_eq!(finding.key().code(), code, "{}", finding.text());
            assert_eq!(fields, expected_fields, "{}", finding.text());
        }
    }

    #[test]
    #[should_panic(expected = "must have a public seam")]
    fn an_unstamped_shape_exposure_fails_loudly() {
        shape_finding(
            ShapeExposure {
                shape: "dyn Port".into(),
                principals: Vec::new(),
                seam: None,
            },
            ExposureKind::DynTrait,
        );
    }
}
