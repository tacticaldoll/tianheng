//! Data types produced by the crate scan.

use crate::finding::UnsafeSiteFact;
use crate::resolve::{AliasMap, ExternRenameMap, ReexportMap, UseMap};
use std::collections::HashSet;
use std::path::PathBuf;
pub(crate) struct ImplSite {
    pub(crate) module: String,
    pub(crate) file: PathBuf,
    pub(crate) trait_path: syn::Path,
    pub(crate) self_ty: syn::Type,
    pub(crate) uses: UseMap,
    pub(crate) type_params: HashSet<String>,
}

/// One type definition observed in the crate: its canonical path (`module::Name`), the module
/// it is defined in and the real file it was read from (for a forbidden-`derive` finding's source
/// file — its own branch's file, same provenance guarantee as [`ImplSite::file`]), the paths in
/// its `#[derive(...)]`/`#[cfg_attr(_, derive(...))]`, and that module's `use`-map (so a renamed
/// derive macro, `use serde::Serialize as Ser; #[derive(Ser)]`, resolves to its true leaf).
pub(crate) struct TypeDef {
    pub(crate) canonical: String,
    pub(crate) module: String,
    pub(crate) file: PathBuf,
    pub(crate) derives: Vec<syn::Path>,
    pub(crate) uses: UseMap,
}

/// One crate-wide scan: the `pub use` re-export closure, the set of locally-defined trait
/// paths (for anchor verification), every trait-impl site, and every type definition.
pub(crate) struct CrateScan {
    pub(crate) reexports: ReexportMap,
    pub(crate) aliases: AliasMap,
    pub(crate) extern_renames: ExternRenameMap,
    pub(crate) trait_defs: HashSet<String>,
    pub(crate) impls: Vec<ImplSite>,
    pub(crate) type_defs: Vec<TypeDef>,
    /// For each non-generic `type X = <path>;` whose target is a nominal path, the alias's canonical
    /// key (`{module}::X`) mapped to the **landing type** its target resolves to under the same
    /// bare-head `CurrentModule` fallback the impl-self check uses (`type Bar = Real` in `crate::dom`
    /// → `crate::dom::Real`; `type Baz = Vec<u8>` / `= String` → `crate::dom::Vec` / `crate::dom::String`,
    /// neither crate-defined). A `type` alias defines no new type — coherence sees through it — so a
    /// marker impl'd on `Bar` governs a subtree type IFF this landing type is itself a crate-defined
    /// subtree type. The forbidden-marker check consults this to react on `type Bar = Real` while NOT
    /// firing on an alias to a foreign/prelude type (whose marker lands off the governed subtree).
    /// (This is distinct from `aliases`, the exposure closure's resolvable-target map, which does not
    /// record a bare-local-struct target.) Multi-valued for the identical cfg-blind reason `UseMap`
    /// and `ReexportMap` are: `type X = Y;` where `Y` itself is a mutually-exclusive `#[cfg]`-gated
    /// `use ... as Y;` name must keep every landing candidate, not just the first found.
    pub(crate) alias_targets: AliasMap,
}

/// One `unsafe` site observed in the crate: its enclosing (file) module, the real file it was
/// read from (its own branch's file — the same provenance guarantee as [`ImplSite::file`]), and a
/// stable label (`unsafe block`, `unsafe fn decode`, `unsafe impl Send`, `unsafe trait Zeroable`,
/// `unsafe extern block`). The label is module-qualified at the finding layer for injectivity.
pub(crate) struct UnsafeSite {
    pub(crate) module: String,
    pub(crate) file: PathBuf,
    pub(crate) site: UnsafeSiteFact,
}
