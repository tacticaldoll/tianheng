//! The fail-closed glob hazard: whether a glob can bring a name resolving under a confined prefix into scope, read
//! over the [`CrateScopes`] resolver.

use std::collections::BTreeSet;
use std::ops::Bound;

use super::item_head::Visibility;
use super::path_vocab::{names_a_block_item, path_within};
use super::resolve::{
    BindingNames, CrateScopes, Namespace, ReadingSize, Walk, extern_crate_names, least,
};
use super::scope_tree::{Binding, DeclKind};

/// Whether a glob naming `glob`, written in module `viewer`, can bring a name resolving under `prefix` into scope. It
/// reacts when the glob's module is the prefix, beneath it or an ancestor of it; and, for a local module, when any
/// `pub` import, any `pub extern crate` or any module-level `type` alias in it or beneath it names a path under the
/// prefix, or a `pub` glob there, or a private glob in the glob's target module itself that `viewer` can see,
/// reaches the prefix by this same test, to a fixed point. A module declared in a block is not beneath any module a
/// glob can name.
///
/// A glob's names are read only here, since nothing enumerates what an external glob brings; a private glob of an
/// ancestor reaches `viewer` through `use super::*`, so its visibility is judged from `viewer`, and only in the
/// module the glob names, which is the only one whose private globs it carries — each module a `pub` glob chain
/// reaches is a target in turn. A concrete binding is enumerated: a call through one resolves and reports as a
/// call, so a private import needs no hazard of its own.
///
/// Every module is read before the answer is given, and a refusal of any chain it reads is the answer — the least of
/// them — even where another module already reaches the prefix. So whether this refuses depends on what the unit
/// holds, never on the order its modules or names are met in. The modules beneath a glob are read from the ordered
/// module map as the one run of paths it starts, rather than by testing every module of the unit against it for each
/// glob a chain reaches.
pub(super) fn glob_reaches_prefix(
    scopes: &CrateScopes,
    glob: &str,
    prefix: &str,
    viewer: &str,
) -> Result<bool, String> {
    let mut hazard = Hazard::default();
    let mut visited: BTreeSet<String> = BTreeSet::new();
    let (mut work, terminal_reaches) = scopes.hazard_paths(glob, Namespace::Type, prefix, true)?;
    hazard.reaches |= terminal_reaches;
    for path in &work {
        hazard.work_size.add(path)?;
    }
    while let Some(glob) = work.pop() {
        if !visited.insert(glob.clone()) {
            continue;
        }
        if path_within(&glob, prefix) || path_within(prefix, &glob) {
            hazard.reaches = true;
            continue;
        }
        let beneath = scopes
            .modules
            .range::<str, _>((Bound::Included(glob.as_str()), Bound::Unbounded))
            .take_while(|(module, _)| module.starts_with(glob.as_str()));
        for (module, module_scopes) in beneath {
            if !path_within(module, &glob) || names_a_block_item(module) {
                continue;
            }
            for &(t, s) in module_scopes {
                read_scope(
                    scopes,
                    t,
                    s,
                    &Reading {
                        prefix,
                        target: &glob,
                        viewer,
                    },
                    &mut hazard,
                    &mut work,
                );
            }
        }
    }
    match hazard.refused {
        Some(refusal) => Err(refusal),
        None => Ok(hazard.reaches),
    }
}

/// What one read of a scope asks about: the confined `prefix`, the module `target` the glob being chased names, and
/// the `viewer` module the reacting glob is written in.
struct Reading<'a> {
    prefix: &'a str,
    target: &'a str,
    viewer: &'a str,
}

/// What the hazard walk has found: whether any module reaches the prefix, and the least refusal of a chain it read.
#[derive(Default)]
struct Hazard {
    reaches: bool,
    refused: Option<String>,
    work_size: ReadingSize,
}

impl Hazard {
    fn refuse(&mut self, refusal: String) {
        self.refused = least(self.refused.take(), refusal);
    }

    /// Fold what a path through a re-export names: it reaches the prefix, or it is refused.
    fn read(&mut self, denoted: Result<(Vec<String>, bool), String>, prefix: &str) {
        match denoted {
            Ok((found, terminal_reaches)) => {
                self.reaches |= terminal_reaches || found.iter().any(|p| path_within(p, prefix))
            }
            Err(refusal) => self.refuse(refusal),
        }
    }
}

/// Read one scope of a module beneath the glob: every re-exporting binding, `pub extern crate` and module-level
/// `type` alias for a path under `prefix`, and every `pub` glob, or private glob of `target` visible from `viewer`,
/// onto `work`, to be read by the same test.
fn read_scope(
    scopes: &CrateScopes,
    t: usize,
    s: u32,
    reading: &Reading<'_>,
    hazard: &mut Hazard,
    work: &mut Vec<String>,
) {
    read_reexports(scopes, t, s, reading, hazard);
    read_extern_crates(scopes, t, s, reading, hazard);
    queue_globs(scopes, t, s, reading, hazard, work);
}

/// Every binding of the scope another module can name through it — a non-private import or a `type` alias — read
/// for a path under the prefix.
fn read_reexports(
    scopes: &CrateScopes,
    t: usize,
    s: u32,
    reading: &Reading<'_>,
    hazard: &mut Hazard,
) {
    let scope = &scopes.tables[t].scopes[s as usize];
    for (name, bindings) in &scope.bindings {
        for binding in bindings {
            let reexported = match binding {
                Binding::Import { visibility, .. } => *visibility != Visibility::Private,
                Binding::Alias { .. } => true,
            };
            if !reexported {
                continue;
            }
            match scopes.binding_names(t, s, name, binding, Namespace::Either, &mut Walk::default())
            {
                BindingNames::Paths(paths, _) => {
                    for path in paths {
                        hazard.read(
                            scopes.hazard_paths(&path, Namespace::Either, reading.prefix, false),
                            reading.prefix,
                        );
                    }
                }
                BindingNames::Local => {}
                BindingNames::PastCap(refusal) => hazard.refuse(refusal),
            }
        }
    }
}

/// Every non-private `extern crate` of the scope, read for a path under the prefix.
fn read_extern_crates(
    scopes: &CrateScopes,
    t: usize,
    s: u32,
    reading: &Reading<'_>,
    hazard: &mut Hazard,
) {
    let scope = &scopes.tables[t].scopes[s as usize];
    for declaration in scope.declarations.values().flatten() {
        if let DeclKind::ExternCrate { target, .. } = &declaration.kind {
            if declaration.visibility != Visibility::Private {
                hazard.read(
                    scopes.hazard_paths(
                        extern_crate_names(target).path(),
                        Namespace::Either,
                        reading.prefix,
                        false,
                    ),
                    reading.prefix,
                );
            }
        }
    }
}

/// Every module a `pub` glob of the scope, or a private glob of the target visible from the viewer, names, pushed
/// onto `work` and charged to the walk's width budget as it is pushed.
fn queue_globs(
    scopes: &CrateScopes,
    t: usize,
    s: u32,
    reading: &Reading<'_>,
    hazard: &mut Hazard,
    work: &mut Vec<String>,
) {
    let scope = &scopes.tables[t].scopes[s as usize];
    for (i, inner) in scope.globs.iter().enumerate() {
        if inner.visibility != Visibility::Private
            || (scope.module == reading.target
                && inner.visibility.visible_from(&scope.module, reading.viewer))
        {
            match scopes.glob_targets(t, s, i) {
                Ok(targets) => {
                    for target in targets {
                        if let Err(refusal) = hazard.work_size.add(&target) {
                            hazard.refuse(refusal);
                            break;
                        }
                        work.push(target);
                    }
                }
                Err(refusal) => hazard.refuse(refusal),
            }
        }
    }
}
