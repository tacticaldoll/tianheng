use super::constitution::*;
use crate::module_scan::{canonical_module_path, package_name_to_import_ident};
use serde_json::Value;
use xuanji::{Polarity, RuleKey, ScanDepth, Severity};

/// A boundary over the intra-crate module import graph — the layering Cargo cannot
/// see. Observed from the target crate's source `use` declarations (PROJECT.md).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleBoundary {
    pub(crate) crate_package: String,
    pub(crate) module: String,
    pub(crate) rule: ModuleRule,
    pub(crate) reason: String,
    pub(crate) severity: Severity,
    pub(crate) anchor: Option<String>,
    pub(crate) depth: ScanDepth,
}

impl ModuleBoundary {
    /// Begin a module boundary within the crate named `package`.
    pub fn in_crate(package: &str) -> ModuleBoundaryBuilder {
        ModuleBoundaryBuilder {
            crate_package: package.to_string(),
        }
    }

    /// The scan depth / granularity recorded with the boundary.
    pub fn scan_depth(&self) -> ScanDepth {
        self.depth
    }

    /// Stable semantic identity for this declared module boundary.
    pub fn rule_key(&self) -> RuleKey {
        let key = self.rule.key();
        if self.depth == ScanDepth::Shallow {
            RuleKey::of(
                key.rule_type(),
                key.fields()
                    .chain(std::iter::once(("scan_depth", self.depth.as_str()))),
            )
        } else {
            key
        }
    }

    /// The rule this boundary declares, exposed read-only for projection and model inspection.
    pub fn rule(&self) -> &ModuleRule {
        &self.rule
    }
}

/// What a module boundary forbids.
///
/// Rules are constructed through [`ModuleBoundary::in_crate`], not variant struct expressions. A
/// consumer can inspect a builder-produced rule without closing over its complete representation:
///
/// ```
/// use guibiao::{ModuleBoundary, ModuleRule};
///
/// let boundary = ModuleBoundary::in_crate("app")
///     .module("crate::core")
///     .must_not_import("crate::adapter")
///     .because("core depends inward only");
/// match boundary.rule() {
///     ModuleRule::MustNotImport { module, .. } => assert_eq!(module, "crate::adapter"),
///     _ => unreachable!(),
/// }
/// ```
///
/// ```compile_fail
/// use guibiao::ModuleRule;
///
/// let _ = ModuleRule::MustNotImport { module: "crate::adapter".to_string() };
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ModuleRule {
    /// The governed module must not import this module (or anything beneath it).
    #[non_exhaustive]
    MustNotImport {
        /// The forbidden module path (e.g. `"crate::projection"`).
        module: String,
    },
    /// The governed module may import only these modules (each "or beneath"), plus its
    /// own subtree; any other internal import is a violation. An empty allowlist permits
    /// only the module's own subtree.
    #[non_exhaustive]
    RestrictImportsTo {
        /// The closed allowlist of importable module paths (e.g. `["crate::types"]`).
        allowed: Vec<String>,
    },
    /// The governed (protected) module must not be imported by this module (or anything
    /// beneath it) — an inbound encapsulation rule, the mirror of `MustNotImport`. A
    /// module within the protected module's own subtree is never an importer.
    #[non_exhaustive]
    MustNotBeImportedBy {
        /// The forbidden importer module path (e.g. `"crate::http"`).
        importer: String,
    },
    /// The governed (protected) module may be imported only by these importers (each "or
    /// beneath") or by its own subtree; any other module that imports it (or anything beneath
    /// it) is a violation — the inbound dual of `RestrictImportsTo`. An empty allowlist permits
    /// only the protected module's own subtree.
    #[non_exhaustive]
    MustOnlyBeImportedBy {
        /// The closed allowlist of importer module paths (e.g. `["crate::facade"]`).
        allowed: Vec<String>,
    },
    /// An **external** crate may be imported only within the governed module's own subtree
    /// (the permitted subtree, or beneath it); any `use <crate_name>::…` from a module
    /// outside that subtree is a violation. The first module rule that observes external
    /// imports — every other rule ignores them. The confined crate is the violation target,
    /// so identity stays injective across different confined crates on the same subtree.
    #[non_exhaustive]
    ConfineExternalCrate {
        /// The confined external crate name (e.g. `"libc"`).
        crate_name: String,
    },
    /// Within the governed module's subtree, forbid inline symbol-path **calls** resolving under
    /// a declared module-path prefix — the inline-symbol-path (layer b) sibling of
    /// [`ConfineExternalCrate`](ModuleRule::ConfineExternalCrate), observing *calls* rather than
    /// `use` imports. The "core reads no ambient clock; time is injected" pattern. The confined
    /// prefix is the violation target, so identity stays injective across nested prefixes on the
    /// same subtree.
    #[non_exhaustive]
    ConfineInlineSymbolPath {
        /// The confined module-path prefix (e.g. `"std::time"`).
        prefix: String,
        /// If `Some`, react only on calls whose terminal segment (leaf-exact) is one of these
        /// verbs (e.g. `["now"]`); `None` reacts on every call under the prefix. Adopter-owned:
        /// a read reachable only through an undeclared verb is a false negative the adopter
        /// accepts by narrowing.
        ending_with: Option<Vec<String>>,
        /// If `true`, react on **any** path under the prefix (mentions included — type
        /// annotations, constants, value captures), not only calls. Mutually exclusive with
        /// `ending_with` (both set is a constitution error).
        strict: bool,
        /// If `true`, resolve a bare path head matching a declared dependency as external after
        /// local precedence checks. Projection metadata and scan breadth only; never identity.
        strict_external: bool,
    },
}

/// The inline-confinement text projection. Neither it nor [`ModuleRule::label`] is identity;
/// [`ModuleRule::key`] carries the semantic rule identity.
pub(crate) fn inline_confinement_text(
    prefix: &str,
    ending_with: &Option<Vec<String>>,
    strict: bool,
) -> String {
    match (ending_with, strict) {
        (_, true) => format!("must not name inline under {prefix} (strict: mentions too)"),
        (Some(verbs), false) => format!(
            "must not call inline under {prefix} ending with: {}",
            verbs.join(", ")
        ),
        (None, false) => format!("must not call inline under {prefix}"),
    }
}

/// The inline-confinement JSON parameters. `strict_external` is emitted only when set, matching the emit-when-set
/// discipline of `ending_with`/`strict` — a strict boundary must not project byte-identically to a
/// default one. This is projection metadata only; it never leaks into [`ModuleRule::label`].
pub(crate) fn inline_confinement_json(
    prefix: &str,
    ending_with: &Option<Vec<String>>,
    strict: bool,
    external: bool,
) -> Vec<(&'static str, Value)> {
    let mut params = vec![("confined_prefix", serde_json::json!(prefix))];
    if let Some(verbs) = ending_with {
        params.push(("ending_with", serde_json::json!(verbs)));
    }
    if strict {
        params.push(("strict", serde_json::json!(true)));
    }
    if external {
        params.push(("strict_external", serde_json::json!(true)));
    }
    params
}

impl ModuleRule {
    /// Stable semantic identity for this declared module rule.
    pub fn key(&self) -> RuleKey {
        match self {
            ModuleRule::MustNotImport { module } => RuleKey::of(
                "tianheng.rule/guibiao/must-not-import",
                [("module", canonical_module_path(module))],
            ),
            ModuleRule::RestrictImportsTo { allowed } => RuleKey::of(
                "tianheng.rule/guibiao/restrict-imports-to",
                [("allowed", canonical_module_set(allowed))],
            ),
            ModuleRule::MustNotBeImportedBy { importer } => RuleKey::of(
                "tianheng.rule/guibiao/must-not-be-imported-by",
                [("importer", canonical_module_path(importer))],
            ),
            ModuleRule::MustOnlyBeImportedBy { allowed } => RuleKey::of(
                "tianheng.rule/guibiao/must-only-be-imported-by",
                [("allowed", canonical_module_set(allowed))],
            ),
            ModuleRule::ConfineExternalCrate { crate_name } => RuleKey::of(
                "tianheng.rule/guibiao/confine-external-crate",
                [("crate", package_name_to_import_ident(crate_name))],
            ),
            ModuleRule::ConfineInlineSymbolPath {
                prefix,
                ending_with,
                strict,
                strict_external: _,
            } => RuleKey::of(
                "tianheng.rule/guibiao/confine-inline-symbol-path",
                [
                    (
                        "ending_with",
                        canonical_set(
                            ending_with
                                .iter()
                                .flat_map(|values| values.iter())
                                .map(|verb| canonical_module_path(verb)),
                        ),
                    ),
                    ("prefix", canonical_module_path(prefix)),
                    ("strict", strict.to_string()),
                ],
            ),
        }
    }

    /// The label feeding the violation `rule` string and the projection — one source.
    pub(crate) fn label(&self) -> &'static str {
        match self {
            ModuleRule::MustNotImport { .. } => "module must not import",
            ModuleRule::RestrictImportsTo { .. } => "restrict imports to",
            ModuleRule::MustNotBeImportedBy { .. } => "module must not be imported by",
            ModuleRule::MustOnlyBeImportedBy { .. } => "module may only be imported by",
            ModuleRule::ConfineExternalCrate { .. } => "external crate confined to module",
            // Presentation parity: the modifier remains a projection detail of the same
            // inline-rule family; `key()` separately preserves the established no-rekey contract.
            ModuleRule::ConfineInlineSymbolPath { .. } => "inline symbol path confined to module",
        }
    }

    /// The inline-confinement payload — `(prefix, ending_with, strict, external)` — or `None` for a
    /// non-inline rule. Dispatch and the exit-2 constitution checks route through this accessor; the only
    /// `external`-conditional behavior lives in the scan (`inline_symbol_findings` / `resolve_head`).
    pub(crate) fn inline_payload(&self) -> Option<(&str, Option<&[String]>, bool, bool)> {
        match self {
            ModuleRule::ConfineInlineSymbolPath {
                prefix,
                ending_with,
                strict,
                strict_external,
            } => Some((prefix, ending_with.as_deref(), *strict, *strict_external)),
            _ => None,
        }
    }

    /// The repair-direction [`Polarity`] of a violation of this rule. The two `MustNot*` rules
    /// forbid a specific module edge (repair: remove the import) → `DenyBreach`; `RestrictImportsTo`,
    /// `MustOnlyBeImportedBy`, and `ConfineExternalCrate` permit a region and react to an edge
    /// outside it (repair: move the import into the permitted subtree, or widen) → `AllowlistGap`.
    pub(crate) fn polarity(&self) -> Polarity {
        match self {
            ModuleRule::MustNotImport { .. } | ModuleRule::MustNotBeImportedBy { .. } => {
                Polarity::DenyBreach
            }
            ModuleRule::RestrictImportsTo { .. }
            | ModuleRule::MustOnlyBeImportedBy { .. }
            | ModuleRule::ConfineExternalCrate { .. } => Polarity::AllowlistGap,
            // A forbidden inline call under the prefix is a breach to remove (or replace with
            // injected time) — the same repair shape as `MustNotImport`, not an allowlist gap.
            // Identity parity: the strict-external modifier shares the polarity.
            ModuleRule::ConfineInlineSymbolPath { .. } => Polarity::DenyBreach,
        }
    }

    /// The human-readable rule text with its parameter, for the text projection.
    pub(crate) fn text(&self) -> String {
        match self {
            ModuleRule::MustNotImport { module } => format!("must not import {module}"),
            ModuleRule::RestrictImportsTo { allowed } if allowed.is_empty() => {
                "restrict imports to nothing".to_string()
            }
            ModuleRule::RestrictImportsTo { allowed } => {
                format!("restrict imports to: {}", allowed.join(", "))
            }
            ModuleRule::MustNotBeImportedBy { importer } => {
                format!("must not be imported by {importer}")
            }
            ModuleRule::MustOnlyBeImportedBy { allowed } if allowed.is_empty() => {
                "may only be imported by nothing".to_string()
            }
            ModuleRule::MustOnlyBeImportedBy { allowed } => {
                format!("may only be imported by: {}", allowed.join(", "))
            }
            ModuleRule::ConfineExternalCrate { crate_name } => {
                format!("confines external crate {crate_name} to this module's subtree")
            }
            ModuleRule::ConfineInlineSymbolPath {
                prefix,
                ending_with,
                strict,
                strict_external,
            } => {
                let text = inline_confinement_text(prefix, ending_with, *strict);
                if *strict_external {
                    format!("{text} (strict-external)")
                } else {
                    text
                }
            }
        }
    }

    /// The JSON parameter fields for the projection. `must_not_import` names its single
    /// `forbidden` path; `restrict_imports_to` emits its closed set as `only` (always,
    /// as `[]` when empty), matching the crate-level restrict-to vocabulary;
    /// `must_not_be_imported_by` names its declared forbidden `importer`;
    /// `confine_external_crate` names the confined `external_crate`.
    pub(crate) fn json_params(&self) -> Vec<(&'static str, Value)> {
        match self {
            ModuleRule::MustNotImport { module } => {
                vec![("forbidden", serde_json::json!(module))]
            }
            ModuleRule::RestrictImportsTo { allowed } => {
                vec![("only", serde_json::json!(allowed))]
            }
            ModuleRule::MustNotBeImportedBy { importer } => {
                vec![("importer", serde_json::json!(importer))]
            }
            // `only_importers` (not bare `only`): this rule governs the inbound *importer*
            // surface, distinct from `restrict_imports_to`'s outbound `only` — the same
            // surface-qualified-key precedent `only_workspace` sets, so the projection is
            // self-describing without reading the `rule` label.
            ModuleRule::MustOnlyBeImportedBy { allowed } => {
                vec![("only_importers", serde_json::json!(allowed))]
            }
            // `external_crate` (self-describing): this rule confines a named external crate to
            // the governed module's subtree, a surface distinct from every internal-edge rule.
            ModuleRule::ConfineExternalCrate { crate_name } => {
                vec![("external_crate", serde_json::json!(crate_name))]
            }
            // `confined_prefix` (self-describing): the module-path prefix whose inline calls are
            // forbidden in the subtree. `ending_with` / `strict` are emitted only when set, so a
            // bare confinement keeps byte-identical JSON (the same discipline as the anchor).
            ModuleRule::ConfineInlineSymbolPath {
                prefix,
                ending_with,
                strict,
                strict_external,
            } => inline_confinement_json(prefix, ending_with, *strict, *strict_external),
        }
    }
}

/// Fluent builder for a [`ModuleBoundary`].
pub struct ModuleBoundaryBuilder {
    crate_package: String,
}

impl ModuleBoundaryBuilder {
    /// The module whose imports are governed (e.g. `"crate::kernel"`).
    pub fn module(self, module: &str) -> ModuleTargetDraft {
        ModuleTargetDraft {
            crate_package: self.crate_package,
            module: module.to_string(),
        }
    }
}

/// A module boundary awaiting its module rule.
pub struct ModuleTargetDraft {
    crate_package: String,
    module: String,
}

impl ModuleTargetDraft {
    /// Forbid the governed module from importing `module` (or anything beneath it).
    ///
    /// A `use`-glob is observed at its base module and retains its glob shape. A glob whose base is
    /// equal to or an ancestor of the forbidden module (`use crate::*;` while forbidding
    /// `crate::secret`) therefore reacts fail-closed, as do the narrow forms
    /// (`use crate::secret;`, `use crate::secret::*;`). A plain non-glob ancestor import
    /// (`use crate;`) remains clean because it does not bring the descendant into scope.
    pub fn must_not_import(self, module: &str) -> ModuleBoundaryDraft {
        self.with_rule(ModuleRule::MustNotImport {
            module: module.to_string(),
        })
    }

    /// Restrict the governed module's internal imports to a closed allowlist: any
    /// internal `use` reaching a module that is neither within the governed module's
    /// own subtree nor within an allowlist entry (each matched "or beneath") is a
    /// violation. An empty allowlist permits only the module's own subtree. Governs
    /// new internal modules by default, the module-level mirror of the crate-level
    /// [`restrict_dependencies_to`](crate::CrateBoundaryBuilder::restrict_dependencies_to).
    pub fn restrict_imports_to<I, S>(self, allowed: I) -> ModuleBoundaryDraft
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.with_rule(ModuleRule::RestrictImportsTo {
            allowed: allowed.into_iter().map(Into::into).collect(),
        })
    }

    /// Forbid the governed (protected) module from being imported by `importer` (or
    /// anything beneath it) — an inbound encapsulation rule, the mirror of
    /// [`must_not_import`](Self::must_not_import). A module within the protected module's
    /// own subtree is never treated as an importer.
    pub fn must_not_be_imported_by(self, importer: &str) -> ModuleBoundaryDraft {
        self.with_rule(ModuleRule::MustNotBeImportedBy {
            importer: importer.to_string(),
        })
    }

    /// Restrict who may import the governed (protected) module to a closed allowlist: only a
    /// listed importer (each "or beneath") or the protected module's own subtree may import it;
    /// any other importer is a violation — the inbound dual of
    /// [`restrict_imports_to`](Self::restrict_imports_to). An empty allowlist permits only the
    /// module's own subtree.
    pub fn must_only_be_imported_by<I, S>(self, allowed: I) -> ModuleBoundaryDraft
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.with_rule(ModuleRule::MustOnlyBeImportedBy {
            allowed: allowed.into_iter().map(Into::into).collect(),
        })
    }

    /// Confine an **external** crate's imports to the governed module's own subtree: any
    /// `use <crate_name>::…` written from a module outside this module (or beneath it) is a
    /// violation. This is the first module rule that observes external-crate imports — every
    /// other rule ignores them. Only the named crate is observed (a `cargo metadata`
    /// cross-check is deliberately *not* performed: confining a crate the target never imports
    /// is simply clean). The confined crate is the violation target, so declaring two
    /// confinements of different crates on the same module stays injective. Confining on
    /// `crate` (the root) is a constitution error, since it would permit the crate everywhere.
    ///
    /// The crate name may be written in either **package** form (`"windows-sys"`) or **import
    /// identifier** form (`"windows_sys"`): the rule observes the source `use` identifier, in
    /// which Cargo maps a package's `-` to `_`, so the confined name is matched with that same
    /// fold. A raw identifier (`r#name`) is canonicalized as elsewhere.
    pub fn confine_external_crate(self, crate_name: &str) -> ModuleBoundaryDraft {
        self.with_rule(ModuleRule::ConfineExternalCrate {
            crate_name: crate_name.to_string(),
        })
    }

    /// Within the governed subtree, forbid inline symbol-path **calls** resolving under the
    /// module-path `prefix` (e.g. `"std::time"`) — the inline-symbol-path (layer b) sibling of
    /// [`confine_external_crate`](Self::confine_external_crate), for the "core reads no ambient
    /// clock; time is injected" pattern. By default only a **call** (`prefix::…::verb(...)`)
    /// reacts; a type annotation, a bare constant reference, and any non-call mention pass (so the
    /// core may *receive* injected time), keeping 圭表 free of a built-in read-verb heuristic. The
    /// returned [`InlineConfinementDraft`] is a dedicated draft — its `.ending_with` /
    /// `.strict_prefix_only` modifiers cannot be applied to the other module rules.
    ///
    /// Resolution follows the alias-carrying use-map, local `type` aliases, and the local
    /// `pub use` re-export closure to a fixpoint, and reacts fail-closed on a glob that can bring
    /// a prefix-resolving name into scope. The stated bounds (receiver-method reads, in-macro-body
    /// aliases, fragment/proc-macro construction, external-crate re-exports, value-position
    /// captures under the default, and the inherited file-scope scanner bounds) are declared
    /// non-observations, never silent passes.
    pub fn must_not_call_inline(self, prefix: &str) -> InlineConfinementDraft {
        InlineConfinementDraft {
            crate_package: self.crate_package,
            module: self.module,
            prefix: prefix.to_string(),
            ending_with: None,
            strict: false,
            external: false,
            severity: Severity::Enforce,
            depth: ScanDepth::Subtree,
        }
    }

    fn with_rule(self, rule: ModuleRule) -> ModuleBoundaryDraft {
        ModuleBoundaryDraft {
            crate_package: self.crate_package,
            module: self.module,
            rule,
            severity: Severity::Enforce,
            depth: ScanDepth::Subtree,
        }
    }
}

/// A module boundary awaiting its severity and reason.
pub struct ModuleBoundaryDraft {
    crate_package: String,
    module: String,
    rule: ModuleRule,
    severity: Severity,
    depth: ScanDepth,
}

impl ModuleBoundaryDraft {
    /// Configure the observation scan depth / granularity level.
    pub fn depth(mut self, depth: ScanDepth) -> Self {
        self.depth = depth;
        self
    }

    /// Convenience modifier setting the scan depth to [`ScanDepth::Subtree`].
    ///
    /// Module rules retain their legacy subtree default, so this is a source-compatible
    /// ergonomic spelling for existing declarations and an explicit override when a draft was
    /// previously changed to [`ScanDepth::Shallow`].
    pub fn including_submodules(self) -> Self {
        self.depth(ScanDepth::Subtree)
    }

    /// Finish the boundary, recording the human-readable `reason` (the repair hint).
    pub fn because(self, reason: &str) -> ModuleBoundary {
        ModuleBoundary {
            crate_package: self.crate_package,
            module: self.module,
            rule: self.rule,
            reason: reason.to_string(),
            severity: self.severity,
            anchor: None,
            depth: self.depth,
        }
    }
}

/// A dedicated draft for an inline-symbol-path confinement (from
/// [`must_not_call_inline`](ModuleTargetDraft::must_not_call_inline)). Distinct from
/// [`ModuleBoundaryDraft`] so its narrowing / escalation modifiers cannot be applied to the other
/// module rules (no modifier pollution). Chain [`ending_with`](Self::ending_with) **or**
/// [`strict_prefix_only`](Self::strict_prefix_only) (they are mutually exclusive), and
/// [`warn`](Self::warn), before [`because`](Self::because).
pub struct InlineConfinementDraft {
    crate_package: String,
    module: String,
    prefix: String,
    ending_with: Option<Vec<String>>,
    strict: bool,
    external: bool,
    severity: Severity,
    depth: ScanDepth,
}

impl InlineConfinementDraft {
    /// Configure the observation scan depth / granularity level.
    pub fn depth(mut self, depth: ScanDepth) -> Self {
        self.depth = depth;
        self
    }

    /// Narrow the confinement to react only on calls whose **terminal segment** (leaf-exact) is
    /// one of `verbs` (e.g. `["now"]`) — the adopter's declared read verbs. The adopter owns any
    /// false negative from omitting a verb (a future `::current()` passes); the engine bakes in no
    /// default verb set. Mutually exclusive with [`strict_prefix_only`](Self::strict_prefix_only):
    /// declaring both is a constitution error (exit 2).
    pub fn ending_with<I, S>(mut self, verbs: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.ending_with = Some(verbs.into_iter().map(Into::into).collect());
        self
    }

    /// Escalate the confinement to react on **any** path under the prefix — mentions included
    /// (type annotations, constants, value-position captures), not only calls. The whole-surface
    /// isolation posture for a subtree that may not even name the module. Mutually exclusive with
    /// [`ending_with`](Self::ending_with): declaring both is a constitution error (exit 2).
    pub fn strict_prefix_only(mut self) -> Self {
        self.strict = true;
        self
    }

    /// **Opt-in.** Resolve a written path's bare head that matches a **declared dependency name**
    /// (rename-aware, `-`→`_`-normalized to its import identifier) as that external crate — so a
    /// **fully-qualified, un-`use`d external call** (`chrono::Utc::now()` with no `use chrono`)
    /// resolving under the confined prefix reacts. This closes the asymmetry whereby a sysroot
    /// head (`std`/`core`/`alloc`) was caught while a fully-qualified external head resolved as a
    /// fake local path and was silently missed (a false negative).
    ///
    /// The flag has a second effect: the existing glob-hazard reaction **extends** to external-crate
    /// globs. A `use chrono::*;` under a `chrono::…` confinement now resolves its glob head as
    /// external `chrono` (an ancestor of the prefix) and reacts fail-closed; under the default it
    /// stays `{module}::chrono` and does not react.
    ///
    /// The reclassification honors **local precedence, first match wins**, checked against the
    /// call's TRUE inline module (`{module}::inner…`, following any `mod name { … }` around it): the
    /// enclosing module's `use`-map, a crate-root module shadow, any local module `{module}::head`,
    /// then any top-level item definition (mod/struct/enum/union/trait/type/fn/const/static) of that
    /// name **in the calling module** — only if none claim the head does the dependency match fire.
    /// A local item shadows a same-named external call only within its OWN module: a file-top
    /// `fn rand` does not mask a `rand::random()` call inside an inline `mod tests { … }`, and a
    /// submodule-local `fn rand` masks only calls in that submodule.
    ///
    /// It catches fully-qualified external calls **by the crate's real name**. It does NOT close:
    /// an `extern crate dep as alias;` rename (a call through the local `alias` head is a stated
    /// bound — the use-map observes `use` only), glob-brought names beyond the glob-hazard
    /// reaction, and macro-constructed names. Do not read this as "all external calls caught."
    ///
    /// One further stated bound, strict-external only: a `mod name {` token or unbalanced braces
    /// **inside a macro-invocation body** can perturb the call scan's inline-module tracking (the
    /// call scan keeps macro bodies — real reads hide there — while the item collector strips them),
    /// so a call's true module may be mis-attributed. Rare and declared, never a silent pass.
    ///
    /// One stated **over-**reaction bound, only under a **single-segment** bare crate prefix
    /// (`must_not_call_inline("rand")`) — a multi-segment prefix (`chrono::Utc`) is immune: 圭表's
    /// text scan cannot tell a local binding or a definition site from a call, so a local
    /// `let rand = …; rand()`, or the definition site of an associated / nested `fn rand(…)` (whose
    /// `rand(` reads as a call), may false-positive. Module-top-level definitions are exempt (they
    /// resolve to the local item). Declared, not silent.
    ///
    /// Orthogonal to [`ending_with`](Self::ending_with) / [`strict_prefix_only`](Self::strict_prefix_only):
    /// it changes head *resolution*, not call-vs-mention breadth, and composes with either — it is
    /// **not** itself part of their mutual exclusion (but the contradictory
    /// `.ending_with(…).strict_prefix_only()` pair is still a constitution error under the flag).
    /// When not set, the fully-qualified external call remains a stated non-observation and
    /// behavior is byte-identical to a confinement without the flag.
    pub fn strict_external(mut self) -> Self {
        self.external = true;
        self
    }

    /// Finish the boundary, recording the human-readable `reason` (the repair hint).
    pub fn because(self, reason: &str) -> ModuleBoundary {
        let rule = ModuleRule::ConfineInlineSymbolPath {
            prefix: self.prefix,
            ending_with: self.ending_with,
            strict: self.strict,
            strict_external: self.external,
        };
        ModuleBoundary {
            crate_package: self.crate_package,
            module: self.module,
            rule,
            reason: reason.to_string(),
            severity: self.severity,
            anchor: None,
            depth: self.depth,
        }
    }
}

boundary_common!(ModuleBoundary);
draft_common!(ModuleBoundaryDraft);
draft_common!(InlineConfinementDraft);
