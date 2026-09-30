//! The inline-symbol-path scan: the observation source for `ConfineInlineSymbolPath` (`must_not_call_inline`,
//! `confine_inline_call`). It reads each file of a compilation unit once into a [`TokenTree`], builds its
//! [`ScopeTable`] and its path [`Occurrence`]s from that one tree, and judges **call expressions** — and, under
//! strict, any path mention — anywhere in a governed file, macro-invocation bodies included, each resolved through
//! the one [`CrateScopes`] resolver. A glob that can bring a prefix-resolving name into scope reacts fail-closed.
//! What the scan does not observe, or over-reacts to, is declared by [`crate::observation_bounds`], never a silent
//! pass.
//!
//! This module is I/O and assembly: every judgement of the text is made by the layers below it, and every refusal
//! it returns names the file it was met in.

use std::collections::{BTreeSet, HashSet};
use std::path::{Path, PathBuf};

use crate::finding::ModuleFact;

use super::glob_hazard::glob_reaches_prefix;
use super::item_head::ItemKeyword;
use super::occurrence::{Occurrence, occurrences};
use super::path_vocab::{
    PathSite, SymbolPrefix, canonical_module_path, names_a_block_item, path_within,
};
use super::resolve::{CrateScopes, Named, Namespace, with_rest};
use super::scope_tree::{DeclKind, ScopeKind, ScopeTable};
use super::token_tree::{Edition, TokenTree};
use super::use_scan::{
    FileUse, ImportedPath, classify_uses, external_imports, file_uses, internal_imports,
};

/// One inline offence: the `finding` string (per the identity requirement) and the source file.
pub(crate) struct InlineFinding {
    pub fact: ModuleFact,
    pub file: String,
}

/// One file of a compilation unit, read once: its path, its module, and the occurrences its tree holds.
struct FileScan {
    file: PathBuf,
    module: String,
    occurrences: Vec<Occurrence>,
    uses: Vec<FileUse>,
}

/// Every file of one compilation unit, each read once into its scope table and its occurrences, and the resolver
/// over all of them: what the prefix existence check and the inline findings both read.
pub(crate) struct UnitScan {
    files: Vec<FileScan>,
    scopes: CrateScopes,
}

impl UnitScan {
    /// Read every reachable `(file, module)` pair of the unit, in a package of the given edition. An unreadable file
    /// is refused here; a file whose text the scanner cannot judge is refused when a judgement reads it.
    pub(crate) fn read(all_files: &[(PathBuf, String)], edition: Edition) -> Result<Self, String> {
        let mut files = Vec::new();
        let mut tables = Vec::new();
        for (file, module) in all_files {
            let raw = std::fs::read_to_string(file).map_err(|err| {
                crate::errors::unreadable_governed_file_error(file, &err.to_string())
            })?;
            let tree = TokenTree::lex(&raw, edition);
            let table = ScopeTable::build(&tree, module, tables.len());
            files.push(FileScan {
                file: file.clone(),
                module: module.clone(),
                occurrences: occurrences(&tree),
                uses: file_uses(&tree, &table),
            });
            tables.push(table);
        }
        Ok(UnitScan {
            files,
            scopes: CrateScopes::new(tables, edition),
        })
    }

    /// Every internal import the file read as `module` makes, paired with its importer, every head read through the
    /// unit's resolver: what an import rule judges. Sorted and deduplicated by `(importer, import)`.
    pub(crate) fn imports(
        &self,
        file: &Path,
        module: &str,
    ) -> Result<Vec<(String, ImportedPath)>, String> {
        let t = self.file_index(file, module)?;
        Ok(internal_imports(classify_uses(
            &self.scopes,
            t,
            &self.files[t].uses,
        )?))
    }

    /// Every external crate the file read as `module` imports, paired with its importer: what
    /// `confine_external_crate` judges, read through the same resolver as [`UnitScan::imports`], so one head is
    /// never both. Sorted and deduplicated by `(importer, crate)`.
    pub(crate) fn external_imports(
        &self,
        file: &Path,
        module: &str,
    ) -> Result<Vec<(String, String)>, String> {
        let t = self.file_index(file, module)?;
        Ok(external_imports(classify_uses(
            &self.scopes,
            t,
            &self.files[t].uses,
        )?))
    }

    /// The table of the file read as `module`, or a refusal naming a pair the unit did not read.
    fn file_index(&self, file: &Path, module: &str) -> Result<usize, String> {
        self.files
            .iter()
            .position(|scan| scan.file == file && scan.module == module)
            .ok_or_else(|| {
                crate::errors::scan_refusal_in_file(
                    file,
                    &format!(
                        "it is governed as `{module}`, which the unit scan did not read it as"
                    ),
                )
            })
    }

    /// Every item the unit's modules declare at their own level, keyed `{true_module}::{name}` — the set an
    /// inline-call prefix naming an item of the crate is held to. An item written inside a macro's group, or
    /// generated by one, is not in it; nor is an item of a module declared in a block, which no path names.
    pub(crate) fn item_definitions(&self) -> BTreeSet<String> {
        let mut items = BTreeSet::new();
        for t in 0..self.files.len() {
            for scope in &self.scopes.table(t).scopes {
                if scope.kind != ScopeKind::Module || names_a_block_item(&scope.module) {
                    continue;
                }
                for name in scope.declarations.keys() {
                    items.insert(format!("{}::{name}", scope.module));
                }
            }
        }
        items
    }

    /// The inline findings against one confinement. `governed` is the subset of the unit's files whose module is
    /// judged; `prefix` is the confined prefix in its one canonical form; `ending_with` narrows to read verbs;
    /// `strict` reacts on any mention, not only calls; `external` opts in strict-external observation (a
    /// fully-qualified un-`use`d head no scope binds, matching a declared dependency, is that external crate);
    /// `dependency_names` are the rename-aware declared-dependency import identifiers it matches against. Returns
    /// findings sorted and deduped by fact. Any file of the unit whose text the scanner cannot judge is refused naming
    /// that file, governed or not, since resolving a governed file reads every file's scope table; so is a governed
    /// file whose judged resolution walks a chain past the nesting cap.
    pub(crate) fn findings(
        &self,
        governed: &[(PathBuf, String)],
        prefix: &SymbolPrefix,
        ending_with: Option<&[String]>,
        strict: bool,
        external: bool,
        dependency_names: &[String],
    ) -> Result<Vec<InlineFinding>, String> {
        let judge = Judgement {
            scopes: &self.scopes,
            prefix: prefix.path.as_str(),
            verbs: ending_with.map(|vs| vs.iter().map(|v| canonical_module_path(v)).collect()),
            strict,
            external_dependencies: external.then(|| dependency_names.iter().cloned().collect()),
        };
        for (t, file) in self.files.iter().enumerate() {
            if let Some(refusal) = self.scopes.table(t).refusal() {
                return Err(crate::errors::scan_refusal_in_file(&file.file, refusal));
            }
        }
        let mut findings: Vec<InlineFinding> = Vec::new();
        for (governed_file, governed_module) in governed {
            let t = self.file_index(governed_file, governed_module)?;
            let scan = &self.files[t];
            judge
                .file(t, scan, &mut findings)
                .map_err(|refusal| crate::errors::scan_refusal_in_file(&scan.file, &refusal))?;
        }
        findings.sort_by(|a, b| a.fact.cmp(&b.fact).then(a.file.cmp(&b.file)));
        findings.dedup_by(|a, b| a.fact == b.fact);
        Ok(findings)
    }
}

/// One confinement's judgement over the unit.
struct Judgement<'a> {
    scopes: &'a CrateScopes,
    prefix: &'a str,
    verbs: Option<Vec<String>>,
    strict: bool,
    external_dependencies: Option<BTreeSet<String>>,
}

impl Judgement<'_> {
    /// Judge every glob and every occurrence of one governed file, table `t`. A refusal is returned bare, and the caller
    /// names the file.
    fn file(
        &self,
        t: usize,
        scan: &FileScan,
        findings: &mut Vec<InlineFinding>,
    ) -> Result<(), String> {
        let table = self.scopes.table(t);
        let file = scan.file.display().to_string();
        let dependencies = self.external_dependencies.as_ref();
        for (scope, written) in table.globs() {
            for glob in resolve_written(
                written,
                PathSite::Use,
                Namespace::Type,
                self.scopes,
                t,
                scope,
                dependencies,
            )? {
                if glob_reaches_prefix(
                    self.scopes,
                    &glob,
                    self.prefix,
                    &table.scopes[scope as usize].module,
                )? {
                    findings.push(InlineFinding {
                        fact: ModuleFact::InlineGlob {
                            path: glob,
                            module: scan.module.clone(),
                        },
                        file: file.clone(),
                    });
                }
            }
        }
        for occurrence in &scan.occurrences {
            if !is_judged(self.strict, occurrence.is_call) {
                continue;
            }
            let scope = table.scope_at(occurrence.at);
            let ns = if occurrence.is_call {
                Namespace::Value
            } else {
                Namespace::Either
            };
            for resolved in resolve_written(
                &occurrence.segments,
                PathSite::Expr,
                ns,
                self.scopes,
                t,
                scope,
                dependencies,
            )? {
                if path_within(&resolved, self.prefix)
                    && should_react_on_occurrence(self.strict, self.verbs.as_deref(), &resolved)
                {
                    findings.push(InlineFinding {
                        fact: ModuleFact::InlinePath {
                            path: resolved,
                            module: scan.module.clone(),
                        },
                        file: file.clone(),
                    });
                }
            }
        }
        Ok(())
    }
}

/// The **value-namespace** names `module` declares at its own level, read from one file's `source`: `fn`, `const`,
/// `static`.
///
/// Rust resolves a `mod` in the TYPE namespace, so the only names that can legally collide with `mod foo` are these —
/// `struct foo` beside `mod foo` would be a duplicate type-namespace definition and does not compile. One
/// `use m::foo;` then binds **both**, which is why an inbound module boundary anchored at `m` must consult this: the
/// module reading alone resolves the import to the descendant `m::foo` and misses that it also reaches `m` itself
/// (see `module_check::resolve_import_module`).
///
/// Read from the same declaration inventory as every other reader of items, so names are keyed by their true
/// (inline-`mod`-qualified) module, an associated or block-local `fn` of the same name does not count, and an item
/// inside an `extern` block opened at that level is the enclosing module's: `unsafe extern "C" { pub fn foo(); }`
/// declares `foo` there, and it coexists with `mod foo` for exactly the namespace reason this function exists to
/// observe. An item declared inside a macro's group is not observed, a stated bound.
pub(crate) fn value_namespace_item_names(
    module: &str,
    source: &str,
    edition: Edition,
) -> HashSet<String> {
    let tree = TokenTree::lex(source, edition);
    let table = ScopeTable::build(&tree, module, 0);
    let mut out = HashSet::new();
    for scope in &table.scopes {
        if scope.kind != ScopeKind::Module {
            continue;
        }
        for (name, declarations) in &scope.declarations {
            let value_item = declarations.iter().any(|declaration| {
                matches!(
                    declaration.kind,
                    DeclKind::Item(ItemKeyword::Fn | ItemKeyword::Const | ItemKeyword::Static)
                )
            });
            if value_item {
                out.insert(format!("{}::{name}", scope.module));
            }
        }
    }
    out
}

/// Resolve a written path — an occurrence's `::`-joined segments, or a glob's own path — standing in
/// `scope` of table `t`, its last segment read in `ns`, to every canonical path it can name: empty when it names none a prefix can
/// reach, and a scan error when the resolution walks a chain past the nesting cap.
///
/// [`CrateScopes::name`] answers, and this reader's policy is its match. Every path it names through a
/// binding, a declared item, a sysroot crate or a crate-rooted path is observed. A crate named by its root
/// (`::dep::…`) is observed under `.strict_external()` only, the same answer its bare spelling gets, so
/// `dep::f()` and `::dep::f()` are reported in the same mode. A bare head no scope binds — so no local
/// module, item or import claims it, at any depth, in the scope the path stands in — is, under
/// `.strict_external()`, the external crate where it names a declared dependency, and otherwise names
/// nothing a prefix can reach: the scope table records every item a module declares, so a head it does not
/// find is a prelude name, a local binding, an attribute's name or an item a macro generates — never an
/// item of the path's module. A block-local item is named by no path a prefix can reach either. Leaf-only
/// matching of an unresolved head is deliberately NOT done.
///
/// The path's module is the true (inline) module of its scope, so a file-top item cannot mask an
/// external call in an inline submodule, and a submodule-local item shadows only its own module.
fn resolve_written(
    written: &str,
    site: PathSite,
    ns: Namespace,
    scopes: &CrateScopes,
    t: usize,
    scope: u32,
    external_dependencies: Option<&BTreeSet<String>>,
) -> Result<Vec<String>, String> {
    match scopes.name(t, scope, written, site, ns) {
        Named::Paths(paths) => Ok(paths),
        Named::External(path) => Ok(external_dependencies.map(|_| path).into_iter().collect()),
        Named::Unbound { head, rest } => {
            let dependency =
                external_dependencies.is_some_and(|dependencies| dependencies.contains(&head));
            let path = with_rest(head, &rest);
            Ok(dependency.then_some(path).into_iter().collect())
        }
        Named::Local | Named::Invalid => Ok(Vec::new()),
        Named::PastCap(refusal) => Err(refusal),
    }
}

/// Whether an occurrence can react at all: any mention under `strict`, and otherwise only a call. An
/// occurrence that cannot react is not resolved, so a chain only it would walk is never read; this is the one
/// place that decides it.
fn is_judged(strict: bool, is_call: bool) -> bool {
    strict || is_call
}

/// Decision helper for whether a judged occurrence — one [`is_judged`] admitted before it was resolved —
/// triggers a boundary reaction, once resolved under the prefix.
///
/// Order of evaluation:
/// 1. `strict`: any path under the prefix reacts (whether call or mention).
/// 2. If narrowed by read verbs (`verbs`), the terminal segment must match a declared verb leaf-exact.
/// 3. Otherwise, every call under the prefix reacts.
fn should_react_on_occurrence(strict: bool, verbs: Option<&[String]>, resolved: &str) -> bool {
    if strict {
        true
    } else if let Some(verbs) = verbs {
        let leaf = resolved
            .rsplit_once("::")
            .map_or(resolved, |(_, leaf)| leaf);
        verbs.iter().any(|v| v == leaf)
    } else {
        true
    }
}
