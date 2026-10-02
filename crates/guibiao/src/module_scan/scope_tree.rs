//! One file's lexical scopes, read from its [`TokenTree`]: which bindings and declarations each scope holds, and the
//! innermost scope of every token.
//!
//! Two kinds of group open a scope, and only two: a **module body** — the file, or an inline `mod name { … }` — and a
//! **block**. [`classify_group`] decides which a group is, and what the others are: the body of an `impl`, `trait`,
//! `enum`, `struct` or `union` holds members reached only through a path, so nothing written directly in it is
//! recorded; an `extern` block and a `cfg_if!` with its arms hold items of the enclosing scope; a macro's group holds
//! nothing the scanner reads as a declaration, and a block or a module body inside it holds the `use` statements
//! written there, read as its bindings for the paths beside them — `macro_rules! m { () => { use crate::clock::{self};
//! clock::now(); } }` names `crate::clock::now`, as the expansion does.
//!
//! A `use`, a `type` alias or an item written directly in a scope binds for that whole scope, text before it
//! included, and ends at its closing brace. Every binding is recorded as written, with its scope, and read by the
//! resolver only when a lookup reaches it.

use std::collections::BTreeMap;

use super::item_head::block_modules;
use super::item_head::{
    GroupKind, ItemKeyword, Visibility, classify_group, heads_a_path, item_at_keyword,
    macro_group_kind, macro_stands_as_an_item, may_be_cfg_gated,
};
use super::occurrence::path_run;
use super::path_vocab::block_segment;
use super::token_tree::{Delimiter, Kind, Node, TokenTree};
use super::use_tree::{UseLeaf, macro_use_statements, use_statements};

/// Which Rust namespace a name is looked up in: a segment followed by `::` names a module or type, a
/// call names a value, and a `use` leaf or a path mentioned without being called may name either — a
/// lookup in [`Namespace::Either`] is the lookup in each, joined, so a declaration in one namespace
/// never hides what a glob brings in the other.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(super) enum Namespace {
    /// Modules, types, traits and crates.
    Type,
    /// Functions, constants, statics and struct constructors.
    Value,
    /// Both, looked up in each and the answers joined.
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

/// What a scope is to a lookup and to the paths its items are named by.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ScopeKind {
    /// The file or an inline `mod name { … }`: a lookup that misses here ends rather than read the scopes around it,
    /// and an item declared here is named by a path through the module.
    Module,
    /// A `{ … }` block: a lookup that misses here reads on into the enclosing scope, and an item declared here is
    /// local to it.
    Block,
}

/// A binding, by the path it is written as: the resolver reads it from the scope it is written in when a lookup
/// reaches it, so a `use` path and a `type` alias's target resolve the same way.
pub(super) enum Binding {
    /// A `use` leaf or `pub use` as written, and who may name it through a glob. `module_only` is a `{self}` leaf's,
    /// which imports the module its group names and nothing else of that name: rustc 1.96.0, edition 2021, resolves
    /// `both()` past `use crate::local::both::{self};` to the scope around it though `local` declares a `fn both`.
    Import {
        /// The imported path, the group's prefix included, led by `::` where the tree is; for a `{self}` leaf, the
        /// module the group names.
        written: String,
        /// The `use` statement's visibility.
        visibility: Visibility,
        /// Whether this is a `{self}` leaf, which holds its name in the type namespace alone.
        module_only: bool,
    },
    /// A `type Name = Target;` written directly in a module body or a block, by its written target, read from its
    /// own scope when looked up.
    Alias {
        /// The target's path, its segments joined by `::` with any generic arguments left off, and led by `::` where
        /// the target is rooted.
        written: String,
        /// The alias item's visibility.
        visibility: Visibility,
    },
}

impl Binding {
    /// The binding's visibility, of either kind: a lookup from a module it does not reach passes the binding over.
    pub(super) fn visibility(&self) -> &Visibility {
        match self {
            Binding::Import { visibility, .. } | Binding::Alias { visibility, .. } => visibility,
        }
    }
}

/// A glob written in a scope: its base path as written, read from that scope when a lookup reaches it, and who may
/// reach through it.
pub(super) struct Glob {
    /// The base path the glob reads from — `a::b` for `a::b::*` and `a::b::{*}` — and `::` for a glob with no path
    /// before it.
    pub written: String,
    /// The `use` statement's visibility, which decides which modules a lookup may reach through the glob from.
    pub visibility: Visibility,
}

/// What a declaration declares, beyond its name: what a path through it reads next.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum DeclKind {
    /// A module, by its path — for one declared in a block, a path through [`block_segment`] that no path written in
    /// source can spell.
    Module(String),
    /// An `extern crate`, by the crate it names — `crate` for `extern crate self` — and whether a `cfg` gates it,
    /// which a crate root's ungated declaration of the same name does not make certain.
    ExternCrate {
        /// The crate the name stands for, `crate` where it names `self`.
        target: String,
        /// Whether a `cfg` may gate the declaration; only an ungated one at the crate root makes the name certain in
        /// the extern prelude.
        gated: bool,
    },
    /// Any other item, by its keyword: it names itself, and a path through it reads what the item holds.
    Item(ItemKeyword),
}

/// One declaration of a name in a scope. A scope may declare a name more than once — under exclusive cfgs, which the
/// scanner reads cfg-blind, or once in the type namespace and once in the value namespace — and each declaration is
/// recorded, with its own namespaces and its own visibility.
#[derive(Clone, Debug)]
pub(super) struct Declaration {
    /// Whether it declares the name in the type namespace: a module, a type, a trait or an `extern crate`.
    pub type_ns: bool,
    /// Whether it declares the name in the value namespace: a function, a constant, a static, or the constructor of
    /// a tuple or unit struct.
    pub value_ns: bool,
    /// The item's visibility, which decides which modules a lookup may find it from.
    pub visibility: Visibility,
    /// What a path through the declared name reads next.
    pub kind: DeclKind,
}

impl Declaration {
    /// Whether the declaration declares its name in `ns`; in [`Namespace::Either`], whether it does in either.
    pub(super) fn in_namespace(&self, ns: Namespace) -> bool {
        match ns {
            Namespace::Type => self.type_ns,
            Namespace::Value => self.value_ns,
            Namespace::Either => self.type_ns || self.value_ns,
        }
    }
}

/// One module body or block of a file, with what is written directly in it.
pub(super) struct Scope {
    /// The id of the scope enclosing this one; `None` only for the file's own module body.
    pub parent: Option<u32>,
    /// Whether a lookup that misses here ends here or reads on into `parent`.
    pub kind: ScopeKind,
    /// The module path the scope stands in: a module body's own, a block's enclosing module's.
    pub module: String,
    /// Each name a `use` leaf or a `type` alias written here binds, with every binding of it.
    pub bindings: BTreeMap<String, Vec<Binding>>,
    /// Each name an item written here declares, with every declaration of it.
    pub declarations: BTreeMap<String, Vec<Declaration>>,
    /// Every glob `use` written here.
    pub globs: Vec<Glob>,
    /// Every name with a binding or a declaration here that no configuration leaves out, by the namespaces it holds,
    /// `(type, value)`: a binding counts in both, its target not read here. A name bound or declared only by what a
    /// `cfg` gates ([`super::item_head::may_be_cfg_gated`]) is absent, so a glob of the scope may be what names it on
    /// a build that compiles the gated one out.
    pub ungated: BTreeMap<String, (bool, bool)>,
    /// Whether a macro invocation stands directly in this scope, where it may expand to items the scanner does not
    /// read.
    pub macro_items: bool,
}

impl Scope {
    /// Record that `name` is bound or declared here, in the namespaces given, by an item no configuration leaves out.
    fn mark_ungated(&mut self, name: &str, type_ns: bool, value_ns: bool) {
        let held = self.ungated.entry(name.to_string()).or_default();
        held.0 |= type_ns;
        held.1 |= value_ns;
    }
}

/// One file's scopes, with the innermost scope of every token of the tree it was built from, and the refusal of what
/// in it the scanner could not read — a `use` tree, or a module a block declares that the reading naming a block's
/// modules gave no name — which the file's judgement reports.
pub(super) struct ScopeTable {
    /// Every scope of the file, by its id as the index: id `0` is the file's own module body, and a scope's parent
    /// precedes it.
    pub scopes: Vec<Scope>,
    /// The table's place among its unit's tables, which names its blocks.
    pub table: usize,
    /// The innermost scope of each token by its index, with one entry more for the end of the text, where
    /// [`ScopeTable::scope_at`] answers for any index past it.
    scope_of: Vec<u32>,
    /// The first refusal met while recording, kept over any met after it.
    refusal: Option<String>,
}

#[cfg(test)]
thread_local! {
    /// Every table this thread has built, counted by the module and the table number it was built as: the work a
    /// direction holds an evaluation to building once per file and module, counted where a table is built.
    static TABLE_BUILDS: std::cell::RefCell<std::collections::HashMap<(String, usize), usize>> =
        std::cell::RefCell::new(std::collections::HashMap::new());
}

/// The tables this thread has built since it last asked, by module and table number, emptying the count.
#[cfg(test)]
pub(crate) fn take_table_builds() -> std::collections::HashMap<(String, usize), usize> {
    TABLE_BUILDS.with(|builds| std::mem::take(&mut *builds.borrow_mut()))
}

/// What stands open while the tree is read forward: the group's kind, and outside it the scope, the recording scope,
/// whether a macro's group encloses it, and the block inside one that records its `use` statements.
struct Frame {
    /// The token closing the group, where the frame is popped and `outer` restored.
    close: usize,
    /// How the group was classified, which [`classify_group`] reads as the enclosing kind of a group opened in it.
    kind: GroupKind,
    /// The scope, the recording scope, whether a macro's group encloses, and the macro block, as they stood before
    /// the group opened.
    outer: (u32, Option<u32>, bool, Option<u32>),
}

/// Append to `scopes` a scope holding nothing yet, and answer its id: its index in `scopes`.
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
        ungated: BTreeMap::new(),
        macro_items: false,
    });
    id
}

impl ScopeTable {
    /// Build the table for one file, whose module is `file_module` and whose place among its unit's tables is
    /// `table`, from its token tree. A module a block declares is named by the [`super::item_head::BlockModule`] label the reachability
    /// walk names its file by, so a path through it and the file it reads carry one path —
    /// `fn g() { mod k { #[path = "y.rs"] pub mod m; } k::m::s(); }` reaches the `s` that `y.rs` binds; one inside a
    /// macro's group has no label, since nothing there is recorded, and takes its block's own segment.
    pub(super) fn build(tree: &TokenTree, file_module: &str, table: usize) -> Self {
        Self::build_naming(tree, file_module, table, &block_modules(tree))
    }

    /// [`ScopeTable::build`], with the block modules `blocks` names: a `mod` a block declares that `blocks` leaves
    /// unnamed is refused rather than recorded.
    fn build_naming(
        tree: &TokenTree,
        file_module: &str,
        table: usize,
        blocks: &[super::item_head::BlockModule],
    ) -> Self {
        #[cfg(test)]
        TABLE_BUILDS.with(|builds| {
            *builds
                .borrow_mut()
                .entry((file_module.to_string(), table))
                .or_default() += 1;
        });
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
        let mut macro_use_of: Vec<Option<u32>> = vec![None; n + 1];
        let mut frames: Vec<Frame> = Vec::new();
        let (mut scope, mut record, mut in_macro, mut macro_block) =
            (0u32, Some(0u32), false, None::<u32>);
        let labels: BTreeMap<usize, &str> = blocks
            .iter()
            .flat_map(|block| {
                std::iter::once(block.head.keyword_at)
                    .chain(block.head.body)
                    .map(|at| (at, block.label.as_str()))
            })
            .collect();
        for i in 0..n {
            if frames.last().is_some_and(|frame| frame.close == i) {
                if let Some(frame) = frames.pop() {
                    (scope, record, in_macro, macro_block) = frame.outer;
                }
            }
            scope_of[i] = scope;
            record_of[i] = record.filter(|_| !in_macro);
            macro_use_of[i] = macro_block;
            let Kind::Open(_) = tree.kind(i) else {
                continue;
            };
            let kind = classify_group(tree, i, frames.last().map(|frame| &frame.kind));
            let outer = (scope, record, in_macro, macro_block);
            match &kind {
                GroupKind::ModuleBody(name) => {
                    let enclosing = &scopes[scope as usize];
                    let module = match enclosing.kind {
                        ScopeKind::Module => format!("{}::{name}", enclosing.module),
                        ScopeKind::Block => match labels.get(&i) {
                            Some(label) => format!("{}::{label}::{name}", enclosing.module),
                            None => format!(
                                "{}::{}::{name}",
                                enclosing.module,
                                block_segment(table, scope)
                            ),
                        },
                    };
                    scope = push_scope(&mut scopes, Some(scope), ScopeKind::Module, module);
                    record = Some(scope);
                    macro_block = in_macro.then_some(scope);
                }
                GroupKind::Block => {
                    let module = scopes[scope as usize].module.clone();
                    scope = push_scope(&mut scopes, Some(scope), ScopeKind::Block, module);
                    record = Some(scope);
                    macro_block = in_macro.then_some(scope);
                }
                GroupKind::MemberBody | GroupKind::UseGroup => {
                    record = None;
                    macro_block = None;
                }
                GroupKind::MacroBody => {
                    in_macro = true;
                    macro_block = None;
                }
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
        macro_use_of[n] = macro_block;

        let mut built = ScopeTable {
            scopes,
            table,
            scope_of,
            refusal: None,
        };
        built.record_uses(tree, &record_of, &macro_use_of);
        built.record_items(tree, &record_of, &labels);
        built
    }

    /// The innermost scope enclosing token `at` of the tree the table was built from.
    pub(super) fn scope_at(&self, at: usize) -> u32 {
        self.scope_of[at.min(self.scope_of.len() - 1)]
    }

    /// The refusal of what in the file the scanner could not read, if any.
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

    /// Record every `use` written directly in a scope as its bindings and globs, and every one a block inside a macro's
    /// group holds as that block's. A tree in a macro's group that does not read as a `use` tree is the macro's input,
    /// and is passed over here as the strict judgement passes it over.
    fn record_uses(
        &mut self,
        tree: &TokenTree,
        record_of: &[Option<u32>],
        macro_use_of: &[Option<u32>],
    ) {
        let in_scopes = use_statements(tree)
            .into_iter()
            .map(|statement| (record_of[statement.at], statement));
        let in_macro_blocks = macro_use_statements(tree)
            .into_iter()
            .filter(|statement| statement.leaves.is_ok())
            .map(|statement| (macro_use_of[statement.at], statement));
        for (record, statement) in in_scopes.chain(in_macro_blocks) {
            let Some(scope) = record else {
                continue;
            };
            let leaves = match statement.leaves {
                Ok(leaves) => leaves,
                Err(refusal) => {
                    self.refusal.get_or_insert(refusal);
                    continue;
                }
            };
            let gated = may_be_cfg_gated(
                tree,
                item_at_keyword(tree, statement.at).map_or(statement.at, |head| head.start),
            );
            let entry = &mut self.scopes[scope as usize];
            for leaf in leaves {
                let (path, binds, module_only) = match leaf {
                    UseLeaf::Name { path, binds } => (path, binds, false),
                    UseLeaf::SelfLeaf { module, binds } => (module, binds, true),
                    UseLeaf::Glob(base) => {
                        entry.globs.push(Glob {
                            written: base,
                            visibility: statement.visibility.clone(),
                        });
                        continue;
                    }
                    UseLeaf::Empty(_) => continue,
                };
                if !gated {
                    entry.mark_ungated(&binds, true, true);
                }
                entry
                    .bindings
                    .entry(binds)
                    .or_default()
                    .push(Binding::Import {
                        written: path,
                        visibility: statement.visibility.clone(),
                        module_only,
                    });
            }
        }
    }

    /// Record every item declared directly in a module body or a block as a [`Declaration`] of its scope — an
    /// `extern crate` among them, under its `as` alias where it has one — and every `type` alias as a binding to its
    /// written target. A block's `type` alias of a path, parenthesized or not, is only that binding; one of anything
    /// else — a tuple, an array, a `fn` pointer — has no path to bind to, so it is a block-local item, as it is for rustc, which resolves
    /// `Command::default()` in `fn g() { type Command = (u8, u8); … }` to the alias whatever the module imports as
    /// `Command`, measured against rustc 1.96.0, edition 2021. A module's `type` alias is also an item of the module. A
    /// member of a body that opens no scope, and anything inside a macro's group, is not recorded at all.
    ///
    /// A file-form module a block declares is its own file's, and the reachability walk names that file's module by the
    /// block's readable form, numbered among the modules of that name the blocks of its module declare in source order;
    /// its declaration is named the same way, which is what lets a path through it reach that file's scope. An inline
    /// one keeps its block's scope-numbered name.
    fn record_items(
        &mut self,
        tree: &TokenTree,
        record_of: &[Option<u32>],
        labels: &BTreeMap<usize, &str>,
    ) {
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
            let gated = may_be_cfg_gated(tree, head.start);
            if head.keyword == ItemKeyword::ExternCrate {
                self.record_extern_crate(tree, i, scope, head.visibility, gated);
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
            let label = match (head.keyword, self.scopes[scope as usize].kind) {
                (ItemKeyword::Mod, ScopeKind::Block) => match labels.get(&head.keyword_at) {
                    Some(label) => Some(*label),
                    None => {
                        self.refusal.get_or_insert_with(|| {
                            format!(
                                "cannot judge the module `{name}` a block declares: the reading that names a \
                                 block's modules gave it no name"
                            )
                        });
                        continue;
                    }
                },
                _ => None,
            };
            let entry = &mut self.scopes[scope as usize];
            if head.keyword == ItemKeyword::Type {
                let target = alias_target(tree, name_at + 1);
                let aliased = target.is_some();
                if let Some(written) = target {
                    if !gated {
                        entry.mark_ungated(&name, true, true);
                    }
                    entry
                        .bindings
                        .entry(name.clone())
                        .or_default()
                        .push(Binding::Alias {
                            written,
                            visibility: head.visibility.clone(),
                        });
                }
                if aliased && entry.kind == ScopeKind::Block {
                    continue;
                }
            }
            let kind = match (head.keyword, label) {
                (ItemKeyword::Mod, None) => DeclKind::Module(format!("{}::{name}", entry.module)),
                (ItemKeyword::Mod, Some(label)) => {
                    DeclKind::Module(format!("{}::{label}::{name}", entry.module))
                }
                (keyword, _) => DeclKind::Item(keyword),
            };
            if !gated {
                entry.mark_ungated(&name, type_ns, value_ns);
            }
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
        gated: bool,
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
        if !gated {
            self.scopes[scope as usize].mark_ungated(&binds, true, false);
        }
        self.scopes[scope as usize]
            .declarations
            .entry(binds)
            .or_default()
            .push(Declaration {
                type_ns: true,
                value_ns: false,
                visibility,
                kind: DeclKind::ExternCrate { target, gated },
            });
    }
}

/// The written target of a `type Name<…> = Target;` whose header continues at `from`: the first path after its `=`,
/// past each leading reference (`&` or `&&`, a lifetime and `mut`) and raw pointer (`*const` or `*mut`). Where no
/// space stands before the `=`, the parameter list closes at a `>=` or `>>=` that holds it, as rustc splits the
/// token. A parenthesized target is the target it holds, where its parentheses hold that path alone, with the
/// generic arguments a type path takes — `type C = (crate::a::T);` and `type C = (crate::a::G<u8>);` name
/// `crate::a::T` and `crate::a::G` to rustc 1.96.0, edition 2021 — and a target no path begins — a
/// qualified path, a tuple, a slice, a `fn` type — binds nothing.
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
    let mut closers = Vec::new();
    loop {
        if tree.kind(k) == Kind::Open(Delimiter::Parenthesis) {
            closers.push(tree.partner(k));
            k += 1;
        } else if tree.is(k, "&") || tree.is(k, "&&") {
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
    let run = path_run(tree, head);
    let end = if super::item_head::opens_an_angle_group(tree, run.end) {
        super::item_head::angle_group_end(tree, run.end)? + 1
    } else {
        run.end
    };
    if !closers
        .iter()
        .rev()
        .enumerate()
        .all(|(depth, close)| end + depth == *close)
    {
        return None;
    }
    let joined = run.segments.join("::");
    Some(if rooted {
        format!("::{joined}")
    } else {
        joined
    })
}

#[cfg(test)]
mod tests {
    use super::super::token_tree::Edition;
    use super::*;

    /// A `mod` a block declares that the block reading leaves unnamed is a refusal of its file, never an index into a
    /// missing label.
    #[test]
    fn a_block_module_left_unnamed_is_refused() {
        let tree = TokenTree::lex("fn f() { mod m; }\n", Edition::Rust2021);
        let table = ScopeTable::build_naming(&tree, "crate", 0, &[]);
        let refusal = table
            .refusal()
            .expect("an unnamed block module must refuse its file");
        assert!(
            refusal.contains("cannot judge the module `m` a block declares"),
            "{refusal}"
        );
        assert!(ScopeTable::build(&tree, "crate", 0).refusal().is_none());
    }
}
