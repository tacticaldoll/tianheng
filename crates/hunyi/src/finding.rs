//! The 渾儀 finding vocabulary and seam labels — how a semantic finding and the public **seam**
//! it sits at are rendered and identified, in one place. A typed semantic fact owns the stable
//! named values used by `(target, rule, finding_key)` and renders its human text separately, so
//! presentation can change without silently changing baseline identity.

use crate::resolve::{ShapeExposure, strip_raw, type_to_string};
use crate::syn_util::VisibleItemKind;
use xuanji::{Finding, FindingKey, StructuredFactIdentity};

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
    Visibility {
        visibility: String,
        item_kind: VisibleItemKind,
        item_name: String,
    },
    UnsafeSite {
        module: String,
        site: UnsafeSiteFact,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum UnsafeSiteFact {
    Block,
    FreeFn {
        name: String,
    },
    InherentMethod {
        owner: String,
        name: String,
    },
    TraitMethod {
        owner: String,
        name: String,
    },
    TraitImplMethod {
        trait_ref: String,
        owner: String,
        name: String,
    },
    InherentImpl {
        owner: String,
    },
    TraitImpl {
        trait_ref: String,
        owner: String,
    },
    Trait {
        name: String,
    },
    ExternBlock,
}

impl UnsafeSiteFact {
    fn shape(&self) -> &'static str {
        match self {
            Self::Block => "unsafe-block",
            Self::FreeFn { .. } => "unsafe-free-function",
            Self::InherentMethod { .. } => "unsafe-inherent-method",
            Self::TraitMethod { .. } => "unsafe-trait-method",
            Self::TraitImplMethod { .. } => "unsafe-trait-impl-method",
            Self::InherentImpl { .. } => "unsafe-inherent-impl",
            Self::TraitImpl { .. } => "unsafe-trait-impl",
            Self::Trait { .. } => "unsafe-trait",
            Self::ExternBlock => "unsafe-extern-block",
        }
    }

    fn key_fields<'a>(&'a self, module: &'a str) -> Vec<(&'static str, &'a str)> {
        let mut fields = vec![("module", module)];
        match self {
            Self::Block | Self::ExternBlock => {}
            Self::FreeFn { name } | Self::Trait { name } => fields.push(("name", name)),
            Self::InherentMethod { owner, name } => {
                fields.push(("name", name.as_str()));
                fields.push(("owner", owner.as_str()));
                fields.push(("owner_kind", "inherent"));
            }
            Self::TraitMethod { owner, name } => {
                fields.push(("name", name.as_str()));
                fields.push(("owner", owner.as_str()));
                fields.push(("owner_kind", "trait"));
            }
            Self::TraitImplMethod {
                trait_ref,
                owner,
                name,
            } => {
                fields.push(("name", name.as_str()));
                fields.push(("owner", owner.as_str()));
                fields.push(("owner_kind", "trait_impl"));
                fields.push(("trait", trait_ref.as_str()));
            }
            Self::InherentImpl { owner } => fields.push(("owner", owner)),
            Self::TraitImpl { trait_ref, owner } => {
                fields.push(("owner", owner.as_str()));
                fields.push(("trait", trait_ref.as_str()));
            }
        }
        fields
    }
}

fn display_unsafe_owner<'a>(module: &str, owner: &'a str) -> &'a str {
    owner
        .strip_prefix(module)
        .and_then(|suffix| suffix.strip_prefix("::"))
        .unwrap_or(owner)
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
            } => match item_kind {
                VisibleItemKind::TraitAlias => {
                    write!(f, "{visibility} trait {item_name} (alias)")
                }
                VisibleItemKind::ExternCrate => {
                    write!(f, "{visibility} extern crate {item_name}")
                }
                kind => write!(f, "{visibility} {} {item_name}", kind.as_str()),
            },
            Self::UnsafeSite { module, site } => match site {
                UnsafeSiteFact::Block => write!(f, "unsafe block in {module}"),
                UnsafeSiteFact::FreeFn { name } => write!(f, "unsafe fn {name} in {module}"),
                UnsafeSiteFact::InherentMethod { owner, name } => {
                    write!(
                        f,
                        "unsafe fn {}::{name} in {module}",
                        display_unsafe_owner(module, owner)
                    )
                }
                UnsafeSiteFact::TraitMethod { owner, name } => {
                    write!(
                        f,
                        "unsafe fn {}::{name} in {module}",
                        display_unsafe_owner(module, owner)
                    )
                }
                UnsafeSiteFact::TraitImplMethod {
                    trait_ref,
                    owner,
                    name,
                } => write!(
                    f,
                    "unsafe fn <{trait_ref} for {}>::{name} in {module}",
                    display_unsafe_owner(module, owner)
                ),
                UnsafeSiteFact::InherentImpl { owner } => {
                    write!(
                        f,
                        "unsafe impl {} in {module}",
                        display_unsafe_owner(module, owner)
                    )
                }
                UnsafeSiteFact::TraitImpl { trait_ref, owner } => {
                    write!(
                        f,
                        "unsafe impl {trait_ref} for {} in {module}",
                        display_unsafe_owner(module, owner)
                    )
                }
                UnsafeSiteFact::Trait { name } => write!(f, "unsafe trait {name} in {module}"),
                UnsafeSiteFact::ExternBlock => write!(f, "unsafe extern block in {module}"),
            },
        }
    }
}

impl SemanticFact {
    pub(crate) fn into_finding(self) -> Finding {
        let text = self.to_string();
        self.into_finding_with_text(text)
    }

    fn into_finding_with_text(self, text: String) -> Finding {
        if let SemanticFact::UnsafeSite { module, site } = &self {
            return Finding::new(
                text,
                StructuredFactIdentity::of(
                    "tianheng.fact/hunyi/unsafe-site",
                    site.shape(),
                    site.key_fields(module),
                ),
            );
        }
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
                    ("item_kind", item_kind.as_str()),
                    ("item_name", item_name),
                    ("visibility", visibility),
                ],
            ),
            SemanticFact::UnsafeSite { .. } => unreachable!("handled above"),
        };
        let key = FindingKey::of("hunyi", code, fields);
        Finding::new(text, key)
    }
}

/// Single-module counterpart: each fact rides beside the real file its own item was resolved from
/// (never a single first-branch file for the whole module — see
/// [`crate::module_resolve::resolve_module_items_with_files`]). Dedup stays fact-identity-only, as
/// it was when findings carried no file; the first-appearing file for a given fact wins.
pub(crate) fn sort_faceted_facts(findings: &mut Vec<(SemanticFact, std::path::PathBuf)>) {
    findings.sort_by(|a, b| a.0.cmp(&b.0));
    findings.dedup_by(|a, b| a.0 == b.0);
    findings.sort_by_cached_key(|(finding, _)| finding.to_string());
}

/// Multi-module counterpart: each fact rides beside the module it sits in (kept for the
/// violation's stable metadata) AND the real file that module's own branch was resolved from
/// (never a re-resolution keyed by the module string alone, which misattributes a finding when
/// two `#[cfg]`-split branches share one module path — see `PROJECT.md`'s Decisions). Dedup
/// remains fact-identity-only, as it was when findings carried only a module string.
pub(crate) fn sort_attributed_facts(
    findings: &mut Vec<(SemanticFact, String, std::path::PathBuf)>,
) {
    findings.sort_by(|a, b| a.0.cmp(&b.0));
    findings.dedup_by(|a, b| a.0 == b.0);
    findings.sort_by_cached_key(|(finding, module, _)| (finding.to_string(), module.clone()));
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

/// Canonicalize a signature's `(params) -> ret` tail for an owner-qualified async fact.
///
/// The tail improves the human finding's readability and collision margin, but it is also stored in
/// the version-2 `signature` key field. Its exact byte form is therefore published baseline wire;
/// whitespace or type-rendering polish is an identity change even though this does NOT represent the
/// compiler's implicit future. Params render each input's type via [`type_to_string`] (a receiver as
/// `self`/`&self`/`&mut self`); the return renders `sig.output`'s written type (empty for `-> ()`);
/// an unrenderable type contributes `_`.
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

    fn published_exposure_code(kind: ExposureKind) -> &'static str {
        match kind {
            ExposureKind::Signature => "signature_exposure",
            ExposureKind::DynTrait => "dyn_trait_exposure",
            ExposureKind::ImplTrait => "impl_trait_exposure",
        }
    }

    fn published_item_kind(kind: &ItemKind) -> &'static str {
        match kind {
            ItemKind::Struct => "struct",
            ItemKind::Enum => "enum",
            ItemKind::Union => "union",
            ItemKind::Type => "type",
            ItemKind::Const => "const",
            ItemKind::Static => "static",
            ItemKind::Trait => "trait",
        }
    }

    fn published_member_kind(kind: &MemberKind) -> &'static str {
        match kind {
            MemberKind::Field => "field",
            MemberKind::Variant => "variant",
        }
    }

    fn published_assoc_kind(kind: &AssocKind) -> &'static str {
        match kind {
            AssocKind::Const => "const",
            AssocKind::Type => "type",
        }
    }

    fn published_visibility_item_kind(kind: VisibleItemKind) -> &'static str {
        match kind {
            VisibleItemKind::Fn => "fn",
            VisibleItemKind::Struct => "struct",
            VisibleItemKind::Enum => "enum",
            VisibleItemKind::Union => "union",
            VisibleItemKind::Type => "type",
            VisibleItemKind::Const => "const",
            VisibleItemKind::Static => "static",
            VisibleItemKind::Trait => "trait",
            VisibleItemKind::TraitAlias => "trait_alias",
            VisibleItemKind::Mod => "mod",
            VisibleItemKind::ExternCrate => "extern_crate",
            VisibleItemKind::Use => "use",
        }
    }

    fn published_position_fields(position: &TraitImplPosition) -> Vec<(&'static str, &str)> {
        match position {
            TraitImplPosition::TraitArg => vec![("seam_position", "trait_arg")],
            TraitImplPosition::SelfType => vec![("seam_position", "self")],
            TraitImplPosition::Where(subject) => vec![
                ("seam_position", "where"),
                ("seam_position_subject", subject),
            ],
            TraitImplPosition::Assoc(name) => {
                vec![("seam_position", "assoc"), ("seam_position_name", name)]
            }
            TraitImplPosition::MethodReturn(name) => vec![
                ("seam_position", "method_return"),
                ("seam_position_name", name),
            ],
        }
    }

    fn published_seam_fields(seam: &PublicSeam) -> Vec<(&'static str, &str)> {
        match seam {
            PublicSeam::FreeFn { module, name } => vec![
                ("seam_kind", "free_fn"),
                ("seam_module", module),
                ("seam_name", name),
            ],
            PublicSeam::InherentMethod { owner, name } => vec![
                ("seam_kind", "inherent_method"),
                ("seam_owner", owner),
                ("seam_name", name),
            ],
            PublicSeam::InherentAssoc { kind, owner, name } => vec![
                ("seam_kind", "inherent_assoc"),
                ("seam_item_kind", published_assoc_kind(kind)),
                ("seam_owner", owner),
                ("seam_name", name),
            ],
            PublicSeam::TraitMethod {
                module,
                trait_name,
                name,
            } => vec![
                ("seam_kind", "trait_method"),
                ("seam_module", module),
                ("seam_trait", trait_name),
                ("seam_name", name),
            ],
            PublicSeam::Item { kind, module, name } => vec![
                ("seam_kind", "item"),
                ("seam_item_kind", published_item_kind(kind)),
                ("seam_module", module),
                ("seam_name", name),
            ],
            PublicSeam::Member {
                kind,
                module,
                owner,
                member,
            } => vec![
                ("seam_kind", "member"),
                ("seam_item_kind", published_member_kind(kind)),
                ("seam_module", module),
                ("seam_owner", owner),
                ("seam_member", member),
            ],
            PublicSeam::TraitAssoc {
                kind,
                module,
                trait_name,
                name,
            } => vec![
                ("seam_kind", "trait_assoc"),
                ("seam_item_kind", published_assoc_kind(kind)),
                ("seam_module", module),
                ("seam_trait", trait_name),
                ("seam_name", name),
            ],
            PublicSeam::InherentGenerics { owner } => {
                vec![("seam_kind", "inherent_generics"), ("seam_owner", owner)]
            }
            PublicSeam::Reexport { module, exported } => vec![
                ("seam_kind", "reexport"),
                ("seam_module", module),
                ("seam_name", exported),
            ],
            PublicSeam::ExternCrate { name } => {
                vec![("seam_kind", "extern_crate"), ("seam_name", name)]
            }
            PublicSeam::TraitImpl {
                trait_ref,
                owner,
                position,
            } => {
                let mut fields = vec![
                    ("seam_kind", "trait_impl"),
                    ("seam_trait", trait_ref.as_str()),
                    ("seam_owner", owner.as_str()),
                ];
                fields.extend(published_position_fields(position));
                fields
            }
        }
    }

    fn assert_semantic_fact_is_cataloged(fact: &SemanticFact) {
        match fact {
            SemanticFact::Exposed {
                kind,
                subject: _,
                seam,
            } => {
                published_exposure_code(*kind);
                published_seam_fields(seam);
            }
            SemanticFact::MisplacedImpl {
                module: _,
                trait_ref: _,
                owner: _,
            }
            | SemanticFact::ForbiddenImpl {
                marker: _,
                owner: _,
                module: _,
            } => {}
            SemanticFact::ForbiddenDerive {
                marker: _,
                canonical: _,
            } => {}
            SemanticFact::AsyncFreeFn {
                module: _,
                name: _,
                tail: _,
            }
            | SemanticFact::AsyncTraitMethod {
                module: _,
                trait_name: _,
                name: _,
                tail: _,
            } => {}
            SemanticFact::AsyncInherentMethod {
                owner: _,
                name: _,
                tail: _,
            } => {}
            SemanticFact::Visibility {
                visibility: _,
                item_kind,
                item_name: _,
            } => {
                published_visibility_item_kind(*item_kind);
            }
            SemanticFact::UnsafeSite { module: _, site } => assert_unsafe_site_is_cataloged(site),
        }
    }

    fn assert_unsafe_site_is_cataloged(site: &UnsafeSiteFact) {
        match site {
            UnsafeSiteFact::Block
            | UnsafeSiteFact::FreeFn { name: _ }
            | UnsafeSiteFact::InherentMethod { owner: _, name: _ }
            | UnsafeSiteFact::TraitMethod { owner: _, name: _ }
            | UnsafeSiteFact::TraitImplMethod {
                trait_ref: _,
                owner: _,
                name: _,
            }
            | UnsafeSiteFact::InherentImpl { owner: _ }
            | UnsafeSiteFact::TraitImpl {
                trait_ref: _,
                owner: _,
            }
            | UnsafeSiteFact::Trait { name: _ }
            | UnsafeSiteFact::ExternBlock => {}
        }
    }

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
            .map(|seam| {
                let mut expected_fields = published_seam_fields(seam);
                expected_fields.push(("subject", "Port"));
                expected_fields.sort_by_key(|(name, _)| *name);
                let fact = SemanticFact::Exposed {
                    kind: ExposureKind::Signature,
                    subject: "Port".into(),
                    seam: seam.clone(),
                };
                assert_semantic_fact_is_cataloged(&fact);
                let finding = fact.into_finding();
                assert_eq!(finding.key().namespace(), "hunyi");
                assert_eq!(finding.key().code(), "signature_exposure");
                assert_eq!(finding.key().fields().collect::<Vec<_>>(), expected_fields);
                finding.key().clone()
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
                    item_kind: VisibleItemKind::Fn,
                    item_name: "run".into(),
                },
                "visibility_exposure",
                vec![
                    ("item_kind", "fn"),
                    ("item_name", "run"),
                    ("visibility", "pub"),
                ],
            ),
        ];
        for (fact, code, expected_fields) in cases {
            assert_semantic_fact_is_cataloged(&fact);
            let finding = fact.into_finding();
            let fields: Vec<_> = finding.key().fields().collect();
            assert_eq!(finding.key().namespace(), "hunyi");
            assert_eq!(finding.key().code(), code, "{}", finding.text());
            assert_eq!(fields, expected_fields, "{}", finding.text());
        }

        let unsafe_fact = SemanticFact::UnsafeSite {
            module: "crate::m".into(),
            site: UnsafeSiteFact::FreeFn { name: "run".into() },
        };
        assert_semantic_fact_is_cataloged(&unsafe_fact);
        let finding = unsafe_fact.into_finding();
        assert_eq!(finding.key().fact_type(), "tianheng.fact/hunyi/unsafe-site");
        assert_eq!(finding.key().shape(), "unsafe-free-function");
        assert_eq!(
            finding.key().fields().collect::<Vec<_>>(),
            vec![("module", "crate::m"), ("name", "run")]
        );
    }

    #[test]
    fn every_exposure_kind_has_its_exact_published_code() {
        for kind in [
            ExposureKind::Signature,
            ExposureKind::DynTrait,
            ExposureKind::ImplTrait,
        ] {
            let expected_code = published_exposure_code(kind);
            let fact = exposure(kind, "crate::api", "run");
            assert_semantic_fact_is_cataloged(&fact);
            let finding = fact.into_finding();
            assert_eq!(finding.key().namespace(), "hunyi");
            assert_eq!(finding.key().code(), expected_code);
            assert_eq!(
                finding.key().fields().collect::<Vec<_>>(),
                vec![
                    ("seam_kind", "free_fn"),
                    ("seam_module", "crate::api"),
                    ("seam_name", "run"),
                    ("subject", "Port"),
                ]
            );
        }
    }

    #[test]
    fn every_unsafe_site_form_has_exact_structured_identity() {
        let cases = vec![
            (
                UnsafeSiteFact::Block,
                "unsafe-block",
                vec![("module", "crate::m")],
            ),
            (
                UnsafeSiteFact::FreeFn { name: "run".into() },
                "unsafe-free-function",
                vec![("module", "crate::m"), ("name", "run")],
            ),
            (
                UnsafeSiteFact::InherentMethod {
                    owner: "crate::m::Api".into(),
                    name: "run".into(),
                },
                "unsafe-inherent-method",
                vec![
                    ("module", "crate::m"),
                    ("name", "run"),
                    ("owner", "crate::m::Api"),
                    ("owner_kind", "inherent"),
                ],
            ),
            (
                UnsafeSiteFact::TraitMethod {
                    owner: "crate::m::Port".into(),
                    name: "run".into(),
                },
                "unsafe-trait-method",
                vec![
                    ("module", "crate::m"),
                    ("name", "run"),
                    ("owner", "crate::m::Port"),
                    ("owner_kind", "trait"),
                ],
            ),
            (
                UnsafeSiteFact::TraitImplMethod {
                    trait_ref: "Port".into(),
                    owner: "crate::m::Api".into(),
                    name: "run".into(),
                },
                "unsafe-trait-impl-method",
                vec![
                    ("module", "crate::m"),
                    ("name", "run"),
                    ("owner", "crate::m::Api"),
                    ("owner_kind", "trait_impl"),
                    ("trait", "Port"),
                ],
            ),
            (
                UnsafeSiteFact::InherentImpl {
                    owner: "crate::m::Api".into(),
                },
                "unsafe-inherent-impl",
                vec![("module", "crate::m"), ("owner", "crate::m::Api")],
            ),
            (
                UnsafeSiteFact::TraitImpl {
                    trait_ref: "Send".into(),
                    owner: "crate::m::Api".into(),
                },
                "unsafe-trait-impl",
                vec![
                    ("module", "crate::m"),
                    ("owner", "crate::m::Api"),
                    ("trait", "Send"),
                ],
            ),
            (
                UnsafeSiteFact::Trait {
                    name: "Port".into(),
                },
                "unsafe-trait",
                vec![("module", "crate::m"), ("name", "Port")],
            ),
            (
                UnsafeSiteFact::ExternBlock,
                "unsafe-extern-block",
                vec![("module", "crate::m")],
            ),
        ];

        for (site, shape, fields) in cases {
            let fact = SemanticFact::UnsafeSite {
                module: "crate::m".into(),
                site,
            };
            assert_semantic_fact_is_cataloged(&fact);
            let finding = fact.into_finding();
            assert_eq!(finding.key().fact_type(), "tianheng.fact/hunyi/unsafe-site");
            assert_eq!(finding.key().shape(), shape);
            assert_eq!(finding.key().fields().collect::<Vec<_>>(), fields);
        }
    }

    #[test]
    fn every_visibility_item_kind_has_its_exact_published_value() {
        for kind in [
            VisibleItemKind::Fn,
            VisibleItemKind::Struct,
            VisibleItemKind::Enum,
            VisibleItemKind::Union,
            VisibleItemKind::Type,
            VisibleItemKind::Const,
            VisibleItemKind::Static,
            VisibleItemKind::Trait,
            VisibleItemKind::TraitAlias,
            VisibleItemKind::Mod,
            VisibleItemKind::ExternCrate,
            VisibleItemKind::Use,
        ] {
            assert_eq!(kind.as_str(), published_visibility_item_kind(kind));
        }
    }

    #[test]
    fn key_renderers_are_pinned_as_version_two_wire() {
        let signature: syn::Signature =
            syn::parse_str("async fn run(&self, value: crate::Port) -> crate::Reply").unwrap();
        assert_eq!(
            render_sig_tail(&signature),
            "(&self, crate::Port) -> crate::Reply"
        );

        let ty: syn::Type = syn::parse_str("Vec<crate::Port>").unwrap();
        assert_eq!(type_to_string(&ty).as_deref(), Some("Vec<crate::Port>"));

        // canonical_self_owner + render_last_segment_args also produce the byte content of the
        // version-2 owner / seam_owner / trait key fields (trait_impl / forbidden_marker /
        // inherent-seam facts). Their exact form — including the positional `_#{ordinal}` fallback
        // a maintainer might mistake for free presentation — is baseline wire too, so pin it.
        let uses: std::collections::HashMap<String, String> = std::collections::HashMap::new();
        let no_params: std::collections::HashSet<String> = std::collections::HashSet::new();
        let owner: syn::Type = syn::parse_str("Repo<crate::Id>").unwrap();
        assert_eq!(
            crate::resolve::canonical_self_owner(&owner, &uses, "app::infra", 0, &no_params),
            "app::infra::Repo<crate::Id>"
        );
        // Base resolves but the generic arg is an unrenderable const expression: the readable base
        // is kept and the arg disambiguated by the block's ordinal (injective, not a collapse).
        let const_owner: syn::Type = syn::parse_str("Arr<{ N + 1 }>").unwrap();
        assert_eq!(
            crate::resolve::canonical_self_owner(&const_owner, &uses, "app::infra", 7, &no_params),
            "app::infra::Arr<_#7>"
        );

        let bare: syn::Path = syn::parse_str("Foo").unwrap();
        assert_eq!(
            crate::resolve::render_last_segment_args(&bare).as_deref(),
            Some("")
        );
        let generics: syn::Path = syn::parse_str("Foo<u8, T>").unwrap();
        assert_eq!(
            crate::resolve::render_last_segment_args(&generics).as_deref(),
            Some("<u8, T>")
        );
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
