//! The 渾儀 finding vocabulary and seam labels — how a semantic finding and the public **seam**
//! it sits at are rendered and identified, in one place. A typed semantic fact owns the stable
//! named values used by `(target, rule key, structured fact)` and renders its human text separately, so
//! presentation can change without silently changing baseline identity.

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum ExposureKind {
    Signature,
    DynTrait,
    ImplTrait,
}

impl ExposureKind {
    pub(super) fn fact_type(self) -> &'static str {
        match self {
            Self::Signature => "tianheng.fact/hunyi/signature-exposure",
            Self::DynTrait => "tianheng.fact/hunyi/dyn-trait-exposure",
            Self::ImplTrait => "tianheng.fact/hunyi/impl-trait-exposure",
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
    pub(super) fn as_str(&self) -> &'static str {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum MemberKind {
    Field,
    Variant,
}

impl MemberKind {
    pub(super) fn as_str(&self) -> &'static str {
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
    pub(super) fn as_str(&self) -> &'static str {
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
    pub(super) fn key_fields(&self) -> Vec<(&'static str, &str)> {
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
        /// The module the impl **block** is written in — distinct from `owner`, the self type's
        /// own canonical path. Rust's coherence rules let an inherent `impl` for a type declared
        /// in one module be written in ANY module of the same crate, so two impl blocks for the
        /// SAME owner in DIFFERENT modules (a platform-conditional split) must not collapse to
        /// one seam merely because they share an owner and a method name. Identity-only: `Display`
        /// ignores it, matching `SemanticFact::AsyncInherentMethod`'s own already-shipped
        /// precedent of carrying `module` distinct from `owner` without rendering it.
        module: String,
        owner: String,
        name: String,
    },
    InherentAssoc {
        kind: AssocKind,
        /// See `InherentMethod::module` — the impl block's own declaring module, not the owner's.
        module: String,
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
        /// See `InherentMethod::module` — the impl block's own declaring module. Carried for the
        /// identical reason and by the identical coherence argument: an owner alone does NOT make
        /// this seam distinct, because Rust permits two inherent `impl` blocks for the SAME self
        /// type in two different modules, and two such blocks can carry the same forbidden bound
        /// in their own generics. Without the module the two collapse to one fact, so a baseline
        /// accepting the first would suppress the second's never-accepted violation.
        module: String,
        owner: String,
    },
    Reexport {
        module: String,
        exported: String,
    },
    ExternCrate {
        /// The module the `pub extern crate` item is written in. A crate may republish the same
        /// external crate root from more than one module (`pub extern crate serde;` is legal in
        /// each), so the crate name alone does not make the seam distinct — the same
        /// per-declaration-site identity every other module-scoped seam here carries.
        module: String,
        name: String,
    },
    /// A trait `impl` block's own impl-site position. Deliberately NOT module-qualified, and that
    /// is a coherence argument rather than an omission: `trait_ref` and `owner` both carry their
    /// rendered generic arguments (`canonical_path_str` / `canonical_self_owner`), and Rust's
    /// coherence rules reject two impl blocks of the same trait — same arguments — for the same
    /// self type anywhere in one crate (E0119), wherever they are written. Two blocks that DO
    /// coexist therefore differ in `trait_ref` or `owner` already. `InherentGenerics` above needs
    /// the module precisely because inherent impls carry no such exclusion.
    TraitImpl {
        trait_ref: String,
        owner: String,
        position: TraitImplPosition,
    },
}

impl PublicSeam {
    pub(super) fn key_fields(&self) -> Vec<(&'static str, &str)> {
        match self {
            Self::FreeFn { module, name } => vec![
                ("seam_kind", "free_fn"),
                ("seam_module", module),
                ("seam_name", name),
            ],
            Self::InherentMethod {
                module,
                owner,
                name,
            } => vec![
                ("seam_kind", "inherent_method"),
                ("seam_module", module),
                ("seam_owner", owner),
                ("seam_name", name),
            ],
            Self::InherentAssoc {
                kind,
                module,
                owner,
                name,
            } => vec![
                ("seam_kind", "inherent_assoc"),
                ("seam_item_kind", kind.as_str()),
                ("seam_module", module),
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
            Self::InherentGenerics { module, owner } => vec![
                ("seam_kind", "inherent_generics"),
                ("seam_module", module),
                ("seam_owner", owner),
            ],
            Self::Reexport { module, exported } => vec![
                ("seam_kind", "reexport"),
                ("seam_module", module),
                ("seam_name", exported),
            ],
            Self::ExternCrate { module, name } => vec![
                ("seam_kind", "extern_crate"),
                ("seam_module", module),
                ("seam_name", name),
            ],
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
            Self::InherentMethod { owner, name, .. } => write!(f, "fn <{owner}>::{name}"),
            Self::InherentAssoc {
                kind, owner, name, ..
            } => {
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
            // `module` is identity-only on both of these, exactly as it is on `InherentMethod` /
            // `InherentAssoc`: the rendered sentence is unchanged by qualifying the identity, and
            // two same-text violations stay separable by the `file` each one carries.
            Self::InherentGenerics { owner, .. } => write!(f, "impl <{owner}> (generics)"),
            Self::Reexport { module, exported } => write!(f, "pub use {module}::{exported}"),
            Self::ExternCrate { name, .. } => write!(f, "pub extern crate {name}"),
            Self::TraitImpl {
                trait_ref,
                owner,
                position,
            } => write!(f, "impl {trait_ref} for {owner} ({position})"),
        }
    }
}
