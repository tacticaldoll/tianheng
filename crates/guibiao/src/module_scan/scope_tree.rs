//! One file's lexical scopes, read from its [`TokenTree`]: which bindings and declarations each scope holds, and the
//! innermost scope of every token.
//!
//! Two kinds of group open a scope, and only two: a **module body** — the file, or an inline `mod name { … }` — and a
//! **block**. [`classify_group`] decides which a group is, and what the others are: the body of an `impl`, `trait`,
//! `enum`, `struct` or `union` holds members reached only through a path, so nothing written directly in it is
//! recorded; an `extern` block and a `cfg_if!` with its arms hold items of the enclosing scope; a macro's group holds
//! nothing the scanner reads as a declaration.
//!
//! A `use`, a `type` alias or an item written directly in a scope binds for that whole scope, text before it
//! included, and ends at its closing brace. Every binding is recorded as written, with its scope, and read by the
//! resolver only when a lookup reaches it.

use std::collections::BTreeMap;

use super::item_head::{
    GroupKind, ItemKeyword, Visibility, classify_group, heads_a_path, item_at_keyword,
    macro_group_kind, macro_stands_as_an_item,
};
use super::occurrence::path_run;
use super::path_vocab::block_segment;
use super::token_tree::{Kind, Node, TokenTree};
use super::use_tree::{UseLeaf, use_statements};

/// Which Rust namespace a name is looked up in: a segment followed by `::` names a module or type, a
/// call names a value, and a `use` leaf or a path mentioned without being called may name either — a
/// lookup in [`Namespace::Either`] is the lookup in each, joined, so a declaration in one namespace
/// never hides what a glob brings in the other.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(super) enum Namespace {
    Type,
    Value,
    Either,
}

impl Namespace {
    /// The other of the two namespaces a name is declared in; [`Namespace::Either`] is its own.
    pub(super) fn other(self) -> Namespace {
        match self {
            Namespace::Type => Namespace::Value,
            Namespace::Value => Namespace::Type,
            Namespace::Either => Namespace::Either,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ScopeKind {
    Module,
    Block,
}

/// A binding, by the path it is written as: the resolver reads it from the scope it is written in when a lookup
/// reaches it, so a `use` path and a `type` alias's target resolve the same way.
pub(super) enum Binding {
    /// A `use` leaf or `pub use` as written, and who may name it through a glob.
    Import {
        written: String,
        visibility: Visibility,
    },
    /// A `type Name = Target;` written directly in a module body or a block, by its written target, read from its
    /// own scope when looked up.
    Alias {
        written: String,
        visibility: Visibility,
    },
}

impl Binding {
    pub(super) fn visibility(&self) -> &Visibility {
        match self {
            Binding::Import { visibility, .. } | Binding::Alias { visibility, .. } => visibility,
        }
    }
}

/// A glob written in a scope: its base path as written, read from that scope when a lookup reaches it, and who may
/// reach through it.
pub(super) struct Glob {
    pub written: String,
    pub visibility: Visibility,
}

/// What a declaration declares, beyond its name: what a path through it reads next.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum DeclKind {
    /// A module, by its path — for one declared in a block, a path through [`block_segment`] that no path written in
    /// source can spell.
    Module(String),
    /// An `extern crate`, by the crate it names: `crate` for `extern crate self`.
    ExternCrate(String),
    /// Any other item, by its keyword: it names itself, and a path through it reads what the item holds.
    Item(ItemKeyword),
}

/// One declaration of a name in a scope. A scope may declare a name more than once — under exclusive cfgs, which the
/// scanner reads cfg-blind, or once in the type namespace and once in the value namespace — and each declaration is
/// recorded, with its own namespaces and its own visibility.
#[derive(Clone, Debug)]
pub(super) struct Declaration {
    pub type_ns: bool,
    pub value_ns: bool,
    pub visibility: Visibility,
    pub kind: DeclKind,
}

impl Declaration {
    pub(super) fn in_namespace(&self, ns: Namespace) -> bool {
        match ns {
            Namespace::Type => self.type_ns,
            Namespace::Value => self.value_ns,
            Namespace::Either => self.type_ns || self.value_ns,
        }
    }
}

pub(super) struct Scope {
    pub parent: Option<u32>,
    pub kind: ScopeKind,
    pub module: String,
    pub bindings: BTreeMap<String, Vec<Binding>>,
    pub declarations: BTreeMap<String, Vec<Declaration>>,
    pub globs: Vec<Glob>,
    /// Whether a macro invocation stands directly in this scope, where it may expand to items the scanner does not
    /// read.
    pub macro_items: bool,
}

/// One file's scopes, with the innermost scope of every token of the tree it was built from, and the refusal of a
/// `use` tree the scanner could not read, which the file's judgement reports.
pub(super) struct ScopeTable {
    pub scopes: Vec<Scope>,
    /// The table's place among its unit's tables, which names its blocks.
    pub table: usize,
    scope_of: Vec<u32>,
    refusal: Option<String>,
}

/// What stands open while the tree is read forward: the group's kind, and the scope and recording scope outside it.
struct Frame {
    close: usize,
    kind: GroupKind,
    outer: (u32, Option<u32>, bool),
}

fn push_scope(
    scopes: &mut Vec<Scope>,
    parent: Option<u32>,
    kind: ScopeKind,
    module: String,
) -> u32 {
    let id = u32::try_from(scopes.len()).expect("scope table exceeds u32");
    scopes.push(Scope {
        parent,
        kind,
        module,
        bindings: BTreeMap::new(),
        declarations: BTreeMap::new(),
        globs: Vec::new(),
        macro_items: false,
    });
    id
}

impl ScopeTable {
    /// Build the table for one file, whose module is `file_module` and whose place among its unit's tables is
    /// `table`, from its token tree.
    pub(super) fn build(tree: &TokenTree, file_module: &str, table: usize) -> Self {
        let mut scopes = Vec::new();
        push_scope(
            &mut scopes,
            None,
            ScopeKind::Module,
            file_module.to_string(),
        );
        let n = tree.len();
        let mut scope_of = vec![0u32; n + 1];
        let mut record_of: Vec<Option<u32>> = vec![Some(0); n + 1];
        let mut frames: Vec<Frame> = Vec::new();
        let (mut scope, mut record, mut in_macro) = (0u32, Some(0u32), false);
        for i in 0..n {
            if frames.last().is_some_and(|frame| frame.close == i) {
                if let Some(frame) = frames.pop() {
                    (scope, record, in_macro) = frame.outer;
                }
            }
            scope_of[i] = scope;
            record_of[i] = record.filter(|_| !in_macro);
            let Kind::Open(_) = tree.kind(i) else {
                continue;
            };
            let kind = classify_group(tree, i, frames.last().map(|frame| &frame.kind));
            let outer = (scope, record, in_macro);
            match &kind {
                GroupKind::ModuleBody(name) => {
                    let enclosing = &scopes[scope as usize];
                    let module = match enclosing.kind {
                        ScopeKind::Module => format!("{}::{name}", enclosing.module),
                        ScopeKind::Block => {
                            format!(
                                "{}::{}::{name}",
                                enclosing.module,
                                block_segment(table, scope)
                            )
                        }
                    };
                    scope = push_scope(&mut scopes, Some(scope), ScopeKind::Module, module);
                    record = Some(scope);
                }
                GroupKind::Block => {
                    let module = scopes[scope as usize].module.clone();
                    scope = push_scope(&mut scopes, Some(scope), ScopeKind::Block, module);
                    record = Some(scope);
                }
                GroupKind::MemberBody | GroupKind::UseGroup => record = None,
                GroupKind::MacroBody => in_macro = true,
                GroupKind::ExternBlock
                | GroupKind::CfgIf
                | GroupKind::CfgArm
                | GroupKind::Other => {}
            }
            frames.push(Frame {
                close: tree.partner(i),
                kind,
                outer,
            });
        }
        scope_of[n] = scope;
        record_of[n] = record.filter(|_| !in_macro);

        let mut built = ScopeTable {
            scopes,
            table,
            scope_of,
            refusal: None,
        };
        built.record_uses(tree, &record_of);
        built.record_items(tree, &record_of);
        built
    }

    /// The innermost scope enclosing token `at` of the tree the table was built from.
    pub(super) fn scope_at(&self, at: usize) -> u32 {
        self.scope_of[at.min(self.scope_of.len() - 1)]
    }

    /// The refusal of a `use` tree in the file the scanner could not read, if any.
    pub(super) fn refusal(&self) -> Option<&str> {
        self.refusal.as_deref()
    }

    /// Every glob written in the file, with the scope it is written in, as `(scope, written base)`.
    pub(super) fn globs(&self) -> impl Iterator<Item = (u32, &str)> {
        self.scopes.iter().enumerate().flat_map(|(id, scope)| {
            let id = u32::try_from(id).expect("scope table exceeds u32");
            scope
                .globs
                .iter()
                .map(move |glob| (id, glob.written.as_str()))
        })
    }

    fn record_uses(&mut self, tree: &TokenTree, record_of: &[Option<u32>]) {
        for statement in use_statements(tree) {
            let Some(scope) = record_of[statement.at] else {
                continue;
            };
            let leaves = match statement.leaves {
                Ok(leaves) => leaves,
                Err(refusal) => {
                    self.refusal.get_or_insert(refusal);
                    continue;
                }
            };
            let entry = &mut self.scopes[scope as usize];
            for leaf in leaves {
                match leaf {
                    UseLeaf::Name { path, binds }
                    | UseLeaf::SelfLeaf {
                        module: path,
                        binds,
                    } => {
                        entry
                            .bindings
                            .entry(binds)
                            .or_default()
                            .push(Binding::Import {
                                written: path,
                                visibility: statement.visibility.clone(),
                            });
                    }
                    UseLeaf::Glob(base) => entry.globs.push(Glob {
                        written: base,
                        visibility: statement.visibility.clone(),
                    }),
                }
            }
        }
    }

    /// Record every item declared directly in a module body or a block as a [`Declaration`] of its scope — an
    /// `extern crate` among them, under its `as` alias where it has one — and every `type` alias as a binding to its
    /// written target. A block's `type` alias is only that binding; a module's is also an item of the module. A
    /// member of a body that opens no scope, and anything inside a macro's group, is not recorded at all.
    ///
    /// A file-form module a block declares is its own file's, and the reachability walk names that file's module by the
    /// block's readable form, numbered among the modules of that name the blocks of its module declare in source order;
    /// its declaration is named the same way, which is what lets a path through it reach that file's scope. An inline
    /// one keeps its block's scope-numbered name.
    fn record_items(&mut self, tree: &TokenTree, record_of: &[Option<u32>]) {
        let mut block_modules: BTreeMap<(String, String), usize> = BTreeMap::new();
        for (i, &record) in record_of.iter().enumerate().take(tree.len()) {
            if let Node::Macro { name, open, .. } = tree.node_at(i) {
                if macro_group_kind(tree, open) == Some(GroupKind::MacroBody)
                    && tree.text(name) != "macro_rules"
                    && macro_stands_as_an_item(tree, name)
                {
                    if let Some(scope) = record {
                        self.scopes[scope as usize].macro_items = true;
                    }
                }
                continue;
            }
            if !matches!(tree.kind(i), Kind::Keyword | Kind::Ident) {
                continue;
            }
            let Some(scope) = record else {
                continue;
            };
            let Some(head) = item_at_keyword(tree, i) else {
                continue;
            };
            if head.keyword == ItemKeyword::ExternCrate {
                self.record_extern_crate(tree, i, scope, head.visibility);
                continue;
            }
            let Some(name_at) = head.name else {
                continue;
            };
            let (type_ns, value_ns) = match head.keyword {
                ItemKeyword::Struct => (true, head.value_struct),
                ItemKeyword::Enum
                | ItemKeyword::Union
                | ItemKeyword::Trait
                | ItemKeyword::Mod
                | ItemKeyword::Type => (true, false),
                ItemKeyword::Fn | ItemKeyword::Const | ItemKeyword::Static => (false, true),
                ItemKeyword::Impl
                | ItemKeyword::Use
                | ItemKeyword::ExternCrate
                | ItemKeyword::ExternBlock
                | ItemKeyword::MacroRules => continue,
            };
            let name = tree.text(name_at).to_string();
            let entry = &mut self.scopes[scope as usize];
            if head.keyword == ItemKeyword::Type {
                if let Some(target) = alias_target(tree, name_at + 1) {
                    entry
                        .bindings
                        .entry(name.clone())
                        .or_default()
                        .push(Binding::Alias {
                            written: target,
                            visibility: head.visibility.clone(),
                        });
                }
                if entry.kind == ScopeKind::Block {
                    continue;
                }
            }
            let kind = match (head.keyword, entry.kind) {
                (ItemKeyword::Mod, ScopeKind::Module) => {
                    DeclKind::Module(format!("{}::{name}", entry.module))
                }
                (ItemKeyword::Mod, ScopeKind::Block) => {
                    let count = block_modules
                        .entry((entry.module.clone(), name.clone()))
                        .or_default();
                    *count += 1;
                    if head.body.is_some() {
                        DeclKind::Module(format!(
                            "{}::{}::{name}",
                            entry.module,
                            block_segment(self.table, scope)
                        ))
                    } else {
                        let block = if *count == 1 {
                            "{block}".to_string()
                        } else {
                            format!("{{block {count}}}")
                        };
                        DeclKind::Module(format!("{}::{block}::{name}", entry.module))
                    }
                }
                (keyword, _) => DeclKind::Item(keyword),
            };
            entry
                .declarations
                .entry(name)
                .or_default()
                .push(Declaration {
                    type_ns,
                    value_ns,
                    visibility: head.visibility,
                    kind,
                });
        }
    }

    /// Record the `extern crate` whose `extern` is at `at`: the name it binds — its `as` alias, or else the crate's
    /// own name — and the crate it names, `crate` for `self`. An `as _` alias binds no name.
    fn record_extern_crate(
        &mut self,
        tree: &TokenTree,
        at: usize,
        scope: u32,
        visibility: Visibility,
    ) {
        let named = at + 2;
        if named >= tree.len() {
            return;
        }
        let target = if tree.is(named, "self") {
            "crate".to_string()
        } else {
            tree.text(named).to_string()
        };
        let binds = if tree.is(named + 1, "as") {
            if !tree.is_word(named + 2) {
                return;
            }
            tree.text(named + 2).to_string()
        } else {
            tree.text(named).to_string()
        };
        self.scopes[scope as usize]
            .declarations
            .entry(binds)
            .or_default()
            .push(Declaration {
                type_ns: true,
                value_ns: false,
                visibility,
                kind: DeclKind::ExternCrate(target),
            });
    }
}

/// The written target of a `type Name<…> = Target;` whose header continues at `from`: the first path after its `=`,
/// past each leading reference (`&` or `&&`, a lifetime and `mut`) and raw pointer (`*const` or `*mut`). Where no
/// space stands before the `=`, the parameter list closes at a `>=` or `>>=` that holds it, as rustc splits the
/// token. A target no path begins — a qualified path, a tuple, a slice, a `fn` type — binds nothing.
fn alias_target(tree: &TokenTree, from: usize) -> Option<String> {
    let mut k = from;
    let mut past_eq = false;
    if tree.is(k, "<") {
        let close = super::item_head::angle_group_end(tree, k)?;
        past_eq = tree.is(close, ">=") || tree.is(close, ">>=");
        k = close + 1;
    }
    if !past_eq {
        while k < tree.len() && !tree.is(k, "=") {
            if tree.is(k, ";") || matches!(tree.kind(k), Kind::Open(_) | Kind::Close(_)) {
                return None;
            }
            k += 1;
        }
        k += 1;
    }
    loop {
        if tree.is(k, "&") || tree.is(k, "&&") {
            k += 1;
            if tree.kind(k) == Kind::Lifetime {
                k += 1;
            }
            if tree.is(k, "mut") {
                k += 1;
            }
        } else if tree.is(k, "*") && (tree.is(k + 1, "const") || tree.is(k + 1, "mut")) {
            k += 2;
        } else {
            break;
        }
    }
    let rooted = tree.is(k, "::");
    let head = if rooted { k + 1 } else { k };
    if !heads_a_path(tree, head) {
        return None;
    }
    let segments = path_run(tree, head).segments;
    let joined = segments.join("::");
    Some(if rooted {
        format!("::{joined}")
    } else {
        joined
    })
}
