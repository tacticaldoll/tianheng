//! The lexical scope table an inline path head is resolved from: which bindings a first identifier
//! can name at the place it is written.
//!
//! Two kinds of brace open a scope, and only two. A **module body** — the file, or an inline
//! `mod name { … }` — and a **block**: a fn body (a method body inside an `impl` or `trait` included),
//! a nested `{ … }`, an `unsafe`/`async`/`const` block, a closure or match-arm body, an initializer
//! block. Every other brace is transparent to name binding. An `impl`, `trait`, `enum`, `struct` or
//! `union` body holds members reached only through a path (`Self::`, the type), so they bind no bare
//! head and are not recorded. An `extern` block and a `cfg_if!` body with its arms hold items that
//! belong to the enclosing scope, so those items are recorded there.
//!
//! A `use` or an item written directly in a scope binds for that whole scope, text before it
//! included, and ends at its closing brace; a lookup walks the occurrence's chain of blocks up to the
//! nearest module scope. A glob is an edge to the module it names, followed at lookup time through
//! [`CrateScopes`] with a visited set, so a chain of globs across modules reaches a fixed point and
//! `use super::*` carries a parent's private imports to its child. Where one scope binds a name more
//! than once — cfg-exclusive imports, which the scanner reads cfg-blind — every binding is a
//! candidate, never the last one written. The table is built from comment- and string-stripped text,
//! the text the call scan reads; declarations are read from that text with macro bodies stripped,
//! placed by position, so a `use` written inside an unexpanded macro body stays unobserved, the
//! existing stated bound.
//!
//! Pure string processing over [`super::lexer`] and [`super::path_vocab`]; it imports no consumer.

use std::collections::{HashMap, HashSet};

use super::lexer::{
    UseStatementScan, is_ident_byte, keyword_starts_at, scan_use_statement,
    strip_macro_bodies_tracked, transparent_macro_body_at,
};
use super::path_vocab::{
    brace_content, canonical_module_path, canonical_segment, fold_canonical_segments,
    inline_mod_at, is_crate_root_shadow, path_within, resolve_self_super, split_top_commas,
};

/// The inline-path scan's brace-nesting cap for [`use_tree_leaves`], so a pathologically nested `use`
/// cannot overflow the stack — a DoS backstop set far beyond any real or lint-clean source, refused
/// past as a scan error. The module scan's import reports carry their own cap,
/// `use_scan::MAX_USE_NEST_DEPTH`, through the same parser.
pub(super) const MAX_SYMBOL_NEST_DEPTH: usize = 64;

/// What a written `use` path, or a path beginning with `::`, is resolved against: the crate-root
/// modules, and whether the package is edition 2015, where both resolve from the crate root rather
/// than from the extern prelude.
#[derive(Clone, Copy)]
pub(super) struct PathRoots<'a> {
    pub root_modules: &'a [String],
    pub edition_2015: bool,
}

impl PathRoots<'_> {
    /// Whether a bare or `::`-rooted head of a `use` path, written in `module`, names a crate-root
    /// module: at the crate root in any edition (a sibling `mod` shadows the extern prelude there), and
    /// from any module in edition 2015, where such a path starts at the crate root.
    pub(super) fn names_root_module(&self, module: &str, head: &str) -> bool {
        if self.edition_2015 {
            self.root_modules.iter().any(|m| m == head)
        } else {
            is_crate_root_shadow(module, head, self.root_modules)
        }
    }
}

/// Which Rust namespace a path head is looked up in: a head followed by `::` names a module or type,
/// a bare call names a value.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Namespace {
    Type,
    Value,
}

/// What a head resolves to from one scope.
pub(super) enum Head {
    /// Every path the head can name there — each `use` binding of it in the nearest scope that binds
    /// it, a block-local `type` alias's target, or what the scope's globs bring — each to be chased
    /// through the crate-wide alias and re-export closure like any other.
    Paths(Vec<String>),
    /// A block-local item. It is named by no path outside its block, so no prefix reaches it.
    Local,
    /// No scope between the occurrence and its module binds the head.
    Unbound,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ScopeKind {
    Module,
    Block,
}

/// Who may name an item or an import from outside the module that declares it, as its `pub` qualifier says.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Visibility {
    /// No qualifier, or `pub(self)`: the declaring module's own subtree.
    Private,
    /// `pub`, or `pub(crate)`: the whole compilation unit, which is all a glob in it can be written from.
    Public,
    /// `pub(super)`: the subtree of the declaring module's parent.
    Super,
    /// `pub(in path)`, with the path as written.
    In(String),
}

impl Visibility {
    /// Whether code in module `from` can name something `owner` declares with this visibility.
    fn visible_from(&self, owner: &str, from: &str) -> bool {
        match self {
            Visibility::Public => true,
            Visibility::Private => path_within(from, owner),
            Visibility::Super => path_within(
                from,
                owner
                    .rsplit_once("::")
                    .map_or("crate", |(parent, _)| parent),
            ),
            Visibility::In(written) => {
                let parts: Vec<&str> = written.split("::").map(str::trim).collect();
                match resolve_self_super(owner, &parts).or_else(|| fold_canonical_segments(&parts))
                {
                    Some(region) => path_within(from, &region),
                    None => true,
                }
            }
        }
    }
}

/// The visibility qualifier written before the item keyword or `use` at `i`, skipping the other
/// qualifiers an item head may carry.
fn visibility_before(bytes: &[u8], i: usize) -> Visibility {
    let mut j = i;
    loop {
        while j > 0 && bytes[j - 1].is_ascii_whitespace() {
            j -= 1;
        }
        let mut start = j;
        while start > 0 && is_ident_byte(bytes[start - 1]) {
            start -= 1;
        }
        if start < j
            && matches!(
                &bytes[start..j],
                b"unsafe" | b"async" | b"const" | b"extern" | b"default" | b"auto"
            )
        {
            j = start;
            continue;
        }
        break;
    }
    if j > 0 && bytes[j - 1] == b')' {
        let close = j - 1;
        let mut open = close;
        while open > 0 && bytes[open] != b'(' {
            open -= 1;
        }
        let mut w = open;
        while w > 0 && bytes[w - 1].is_ascii_whitespace() {
            w -= 1;
        }
        if w >= 3 && keyword_starts_at(bytes, w - 3, b"pub") {
            let inner = String::from_utf8_lossy(&bytes[open + 1..close])
                .trim()
                .to_string();
            return match inner.as_str() {
                "crate" => Visibility::Public,
                "self" => Visibility::Private,
                "super" => Visibility::Super,
                other => match other.strip_prefix("in") {
                    Some(path) if path.starts_with(char::is_whitespace) => {
                        Visibility::In(path.trim().to_string())
                    }
                    _ => Visibility::Public,
                },
            };
        }
        return Visibility::Private;
    }
    if j >= 3 && keyword_starts_at(bytes, j - 3, b"pub") {
        Visibility::Public
    } else {
        Visibility::Private
    }
}

enum Binding {
    /// A `use` leaf, resolved from its scope's module, and who may name it through a glob.
    Import {
        target: String,
        visibility: Visibility,
    },
    /// A block-local `type Name = Target;`, resolved from its own scope when looked up.
    Alias(String),
}

#[derive(Default, Clone, Copy)]
struct ItemNamespaces {
    type_ns: bool,
    value_ns: bool,
}

struct Scope {
    parent: Option<u32>,
    kind: ScopeKind,
    module: String,
    bindings: HashMap<String, Vec<Binding>>,
    items: HashMap<String, ItemNamespaces>,
    /// Each glob written in this scope: the module it names, and who may reach through it.
    globs: Vec<(String, Visibility)>,
}

/// A `type Name = Target;` written directly in a module body, not in a block and not as an
/// associated type: `(name, written target, module)`.
pub(super) type ModuleAlias = (String, String, String);

/// One file's scopes, with the innermost scope of every byte of the text it was built from.
pub(super) struct ScopeTable {
    scopes: Vec<Scope>,
    scope_at: Vec<u32>,
    module_aliases: Vec<ModuleAlias>,
    /// `({module}::{name}, visibility)` for every item declared directly in a module body.
    module_items: Vec<(String, Visibility)>,
}

/// Where a declaration at a byte is recorded: in a scope, or nowhere because it is a member of an
/// `impl`, `trait`, `enum`, `struct` or `union` body.
const MEMBER: u32 = u32::MAX;

/// What a `{` opens.
#[derive(Clone, PartialEq, Eq)]
enum Brace {
    Module(String),
    Block,
    /// A body whose members bind no bare head.
    Members,
    /// A body whose items belong to the enclosing scope: an `extern` block.
    PassThrough,
    /// A `cfg_if!` body or one of its arms; its items belong to the enclosing scope.
    CfgArms,
}

/// What every module of a compilation unit binds at module scope, read from each file's
/// [`ScopeTable`]: what a glob of that module can bring into another.
pub(super) struct CrateScopes {
    bindings: HashMap<String, HashMap<String, Vec<(String, Visibility)>>>,
    globs: HashMap<String, Vec<(String, Visibility)>>,
    /// `{module}::{name}` for every item a module declares at its own top level, with its visibility.
    items: HashMap<String, Visibility>,
}

impl CrateScopes {
    /// Whether the crate root declares an item named `name` — what a 2015 `use name…` or `::name…` path
    /// starts from when no crate-root module is named that.
    pub(super) fn is_root_item(&self, name: &str) -> bool {
        self.items.contains_key(&format!("crate::{name}"))
    }

    pub(super) fn new<'a>(tables: impl IntoIterator<Item = &'a ScopeTable>) -> Self {
        let mut bindings: HashMap<String, HashMap<String, Vec<(String, Visibility)>>> =
            HashMap::new();
        let mut globs: HashMap<String, Vec<(String, Visibility)>> = HashMap::new();
        let mut items: HashMap<String, Visibility> = HashMap::new();
        for table in tables {
            for (item, visibility) in &table.module_items {
                items.insert(item.clone(), visibility.clone());
            }
            for scope in table.scopes.iter().filter(|s| s.kind == ScopeKind::Module) {
                let names = bindings.entry(scope.module.clone()).or_default();
                for (name, all) in &scope.bindings {
                    for binding in all {
                        if let Binding::Import { target, visibility } = binding {
                            names
                                .entry(name.clone())
                                .or_default()
                                .push((target.clone(), visibility.clone()));
                        }
                    }
                }
                globs
                    .entry(scope.module.clone())
                    .or_default()
                    .extend(scope.globs.iter().cloned());
            }
        }
        CrateScopes {
            bindings,
            globs,
            items,
        }
    }

    /// Every path `name` can name in `module` as a glob of it, written in `from`, brings it: `module`'s
    /// own imports and items, or else what its own globs bring, to a fixed point. Each counts only where
    /// its visibility reaches `from` — a private one from `module`'s own subtree, as `use super::*` sees
    /// it, a `pub(super)` one from its parent's.
    fn glob_candidates(
        &self,
        module: &str,
        name: &str,
        from: &str,
        visited: &mut HashSet<String>,
    ) -> Vec<String> {
        if !module.starts_with("crate") || !visited.insert(module.to_string()) {
            return Vec::new();
        }
        let visible = |visibility: &Visibility| visibility.visible_from(module, from);
        let mut found: Vec<String> = self
            .bindings
            .get(module)
            .and_then(|names| names.get(name))
            .into_iter()
            .flatten()
            .filter(|(_, visibility)| visible(visibility))
            .map(|(target, _)| target.clone())
            .collect();
        let item = format!("{module}::{name}");
        if self.items.get(&item).is_some_and(visible) {
            found.push(item);
        }
        if !found.is_empty() {
            return found;
        }
        for (glob, visibility) in self.globs.get(module).into_iter().flatten() {
            if visible(visibility) {
                found.extend(self.glob_candidates(glob, name, from, visited));
            }
        }
        found
    }
}

fn with_rest(mut path: String, rest: &[String]) -> String {
    for segment in rest {
        path.push_str("::");
        path.push_str(segment);
    }
    path
}

impl ScopeTable {
    /// Build the table for one file from its comment- and string-stripped `text`, whose module is
    /// `file_module`.
    pub(super) fn build(
        text: &str,
        file_module: &str,
        roots: PathRoots<'_>,
    ) -> Result<Self, String> {
        let bytes = text.as_bytes();
        let braces = classify_braces(bytes);
        let mut scopes = Vec::new();
        push_scope(
            &mut scopes,
            None,
            ScopeKind::Module,
            file_module.to_string(),
        );
        let mut scope_at = vec![0u32; bytes.len() + 1];
        let mut record_at = vec![0u32; bytes.len() + 1];
        let mut frames: Vec<(u32, u32, bool)> = Vec::new();
        let (mut scope, mut record, mut in_cfg) = (0u32, 0u32, false);
        for (i, &byte) in bytes.iter().enumerate() {
            scope_at[i] = scope;
            record_at[i] = record;
            match byte {
                b'{' => {
                    let brace = match braces.get(&i) {
                        Some(kind) => kind.clone(),
                        None if in_cfg && follows_cfg_arm_head(bytes, i) => Brace::CfgArms,
                        None => Brace::Block,
                    };
                    let (next_scope, next_record, next_cfg) = match brace {
                        Brace::Module(name) => {
                            let module = format!("{}::{name}", scopes[scope as usize].module);
                            let id =
                                push_scope(&mut scopes, Some(scope), ScopeKind::Module, module);
                            (id, id, false)
                        }
                        Brace::Block => {
                            let module = scopes[scope as usize].module.clone();
                            let id = push_scope(&mut scopes, Some(scope), ScopeKind::Block, module);
                            (id, id, false)
                        }
                        Brace::Members => (scope, MEMBER, false),
                        Brace::PassThrough => (scope, record, false),
                        Brace::CfgArms => (scope, record, true),
                    };
                    frames.push((scope, record, in_cfg));
                    (scope, record, in_cfg) = (next_scope, next_record, next_cfg);
                }
                b'}' => {
                    if let Some(outer) = frames.pop() {
                        (scope, record, in_cfg) = outer;
                    }
                }
                _ => {}
            }
        }
        scope_at[bytes.len()] = scope;
        record_at[bytes.len()] = record;

        let identity: Vec<usize> = (0..bytes.len()).collect();
        let (declarations, positions) = strip_macro_bodies_tracked(text, &identity);
        let mut table = ScopeTable {
            scopes,
            scope_at,
            module_aliases: Vec::new(),
            module_items: Vec::new(),
        };
        table.record_uses(&declarations, &positions, &record_at, roots)?;
        table.record_items(&declarations, &positions, &record_at);
        Ok(table)
    }

    /// The innermost scope enclosing byte `at` of the text the table was built from.
    pub(super) fn scope_at(&self, at: usize) -> u32 {
        self.scope_at[at.min(self.scope_at.len() - 1)]
    }

    /// Resolve `head`, followed by `rest`, from `scope`: the bindings of the nearest scope on the chain
    /// of blocks up to the nearest module scope that binds it, a block-local item, or what that chain's
    /// globs bring.
    pub(super) fn resolve(
        &self,
        scope: u32,
        head: &str,
        rest: &[String],
        ns: Namespace,
        crate_scopes: &CrateScopes,
    ) -> Head {
        self.resolve_at_depth(scope, head, rest, ns, crate_scopes, 0)
    }

    fn resolve_at_depth(
        &self,
        scope: u32,
        head: &str,
        rest: &[String],
        ns: Namespace,
        crate_scopes: &CrateScopes,
        depth: usize,
    ) -> Head {
        let mut current = Some(scope);
        while let Some(id) = current {
            let entry = &self.scopes[id as usize];
            if let Some(all) = entry.bindings.get(head).filter(|all| !all.is_empty()) {
                let mut paths = Vec::new();
                let mut names_a_local_item = false;
                for binding in all {
                    match binding {
                        Binding::Import { target, .. } => paths.push(target.clone()),
                        Binding::Alias(written) => {
                            match self.resolve_alias(id, written, crate_scopes, depth) {
                                Head::Paths(found) => paths.extend(found),
                                Head::Local => names_a_local_item = true,
                                Head::Unbound => {}
                            }
                        }
                    }
                }
                if paths.is_empty() && names_a_local_item {
                    return Head::Local;
                }
                return Head::Paths(paths.into_iter().map(|p| with_rest(p, rest)).collect());
            }
            if entry.kind == ScopeKind::Block
                && entry.items.get(head).is_some_and(|item| match ns {
                    Namespace::Type => item.type_ns,
                    Namespace::Value => item.value_ns,
                })
            {
                return Head::Local;
            }
            if entry.kind == ScopeKind::Module
                && crate_scopes
                    .items
                    .contains_key(&format!("{}::{head}", entry.module))
            {
                return Head::Unbound;
            }
            let mut brought = Vec::new();
            for (glob, _) in &entry.globs {
                brought.extend(crate_scopes.glob_candidates(
                    glob,
                    head,
                    &entry.module,
                    &mut HashSet::new(),
                ));
            }
            if !brought.is_empty() {
                return Head::Paths(brought.into_iter().map(|p| with_rest(p, rest)).collect());
            }
            if entry.kind == ScopeKind::Module {
                return Head::Unbound;
            }
            current = entry.parent;
        }
        Head::Unbound
    }

    /// A block-local alias's target, read from the alias's own scope as its written path says.
    fn resolve_alias(
        &self,
        scope: u32,
        written: &str,
        crate_scopes: &CrateScopes,
        depth: usize,
    ) -> Head {
        let module = &self.scopes[scope as usize].module;
        if written.starts_with("::") || depth > MAX_SYMBOL_NEST_DEPTH {
            return Head::Unbound;
        }
        let parts: Vec<String> = written
            .split("::")
            .map(|s| canonical_module_path(s.trim()))
            .filter(|s| !s.is_empty())
            .collect();
        let Some((head, rest)) = parts.split_first() else {
            return Head::Unbound;
        };
        let parts_str: Vec<&str> = parts.iter().map(String::as_str).collect();
        let single = |path: Option<String>| path.map_or(Head::Unbound, |p| Head::Paths(vec![p]));
        match head.as_str() {
            "std" | "core" | "alloc" => Head::Paths(vec![parts.join("::")]),
            "crate" => single(fold_canonical_segments(&parts_str)),
            "self" | "super" => single(resolve_self_super(module, &parts_str)),
            _ => match self.resolve_at_depth(
                scope,
                head,
                rest,
                Namespace::Type,
                crate_scopes,
                depth + 1,
            ) {
                Head::Unbound => Head::Paths(vec![format!("{module}::{}", parts.join("::"))]),
                resolved => resolved,
            },
        }
    }

    /// Each module scope's `use` bindings, keyed `(module, name)`, the last written winning — the view
    /// the crate-wide alias and re-export closure resolves module-level targets through.
    pub(super) fn module_bindings(&self) -> HashMap<(String, String), String> {
        let mut map = HashMap::new();
        for scope in self.scopes.iter().filter(|s| s.kind == ScopeKind::Module) {
            for (name, bindings) in &scope.bindings {
                if let Some(Binding::Import { target, .. }) = bindings
                    .iter()
                    .rev()
                    .find(|b| matches!(b, Binding::Import { .. }))
                {
                    map.insert((scope.module.clone(), name.clone()), target.clone());
                }
            }
        }
        map
    }

    /// Every `type` alias written directly in a module body — the ones a path outside a block can name.
    pub(super) fn module_aliases(&self) -> &[ModuleAlias] {
        &self.module_aliases
    }

    fn record_uses(
        &mut self,
        declarations: &str,
        positions: &[usize],
        record_at: &[u32],
        roots: PathRoots<'_>,
    ) -> Result<(), String> {
        for statement in use_statements(declarations) {
            let scope = record_at[positions[statement.at]];
            if scope == MEMBER {
                continue;
            }
            let module = self.scopes[scope as usize].module.clone();
            for leaf in use_tree_leaves(&statement.body, MAX_SYMBOL_NEST_DEPTH)? {
                match leaf {
                    UseLeaf::Name { path, binds } => {
                        if let Some(target) = resolve_written_path(&path, &module, roots) {
                            self.scopes[scope as usize]
                                .bindings
                                .entry(binds)
                                .or_default()
                                .push(Binding::Import {
                                    target,
                                    visibility: statement.visibility.clone(),
                                });
                        }
                    }
                    UseLeaf::Glob(base) => {
                        if let Some(target) = resolve_written_path(&base, &module, roots) {
                            self.scopes[scope as usize]
                                .globs
                                .push((target, statement.visibility.clone()));
                        }
                    }
                    UseLeaf::SelfLeaf(_) => {}
                }
            }
        }
        Ok(())
    }

    /// Record every item declared directly in a block — its name and namespaces, or, for a `type`
    /// alias, a binding to its written target — and every `type` alias written directly in a module
    /// body. A module's other items are read crate-wide into [`CrateScopes`]; a member of a body that
    /// opens no scope is not recorded at all.
    fn record_items(&mut self, declarations: &str, positions: &[usize], record_at: &[u32]) {
        const KEYWORDS: [&[u8]; 9] = [
            b"struct", b"enum", b"union", b"trait", b"type", b"mod", b"fn", b"const", b"static",
        ];
        let bytes = declarations.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            let Some(keyword) = KEYWORDS
                .iter()
                .find(|kw| keyword_starts_at(bytes, i, kw) && at_item_start(bytes, i))
            else {
                i += 1;
                continue;
            };
            let scope = record_at[positions[i]];
            let after = i + keyword.len();
            i = after;
            if scope == MEMBER {
                continue;
            }
            let mut name_at = skip_ws(bytes, after);
            if *keyword == b"static" && keyword_starts_at(bytes, name_at, b"mut") {
                name_at = skip_ws(bytes, name_at + 3);
            }
            let Some((name, name_end)) = identifier_at(bytes, name_at) else {
                continue;
            };
            if name == "_" {
                continue;
            }
            let entry = &mut self.scopes[scope as usize];
            if entry.kind == ScopeKind::Module {
                self.module_items.push((
                    format!("{}::{name}", entry.module),
                    visibility_before(bytes, i - keyword.len()),
                ));
                if *keyword == b"type" {
                    if let Some(target) = alias_target(declarations, name_end) {
                        self.module_aliases
                            .push((name, target, entry.module.clone()));
                    }
                }
                continue;
            }
            let (type_ns, value_ns) = match *keyword {
                b"type" => {
                    if let Some(target) = alias_target(declarations, name_end) {
                        entry
                            .bindings
                            .entry(name)
                            .or_default()
                            .push(Binding::Alias(target));
                    }
                    continue;
                }
                b"struct" => (
                    true,
                    item_delimiter(bytes, name_end, false).is_some_and(|(_, d)| d != b'{'),
                ),
                b"enum" | b"union" | b"trait" | b"mod" => (true, false),
                _ => (false, true),
            };
            let item = entry.items.entry(name).or_default();
            item.type_ns |= type_ns;
            item.value_ns |= value_ns;
        }
    }
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
        bindings: HashMap::new(),
        items: HashMap::new(),
        globs: Vec::new(),
    });
    id
}

/// Every `{` that opens something other than a block: an inline module body, the body of an item
/// whose members bind no bare head, an `extern` block, or a `cfg_if!` body. Arms inside a `cfg_if!`
/// body are recognized during the walk, by what precedes them.
fn classify_braces(bytes: &[u8]) -> HashMap<usize, Brace> {
    let mut braces = HashMap::new();
    for i in 0..bytes.len() {
        if let Some((name_start, name_end, brace)) = inline_mod_at(bytes, i) {
            let name = canonical_segment(&String::from_utf8_lossy(&bytes[name_start..name_end]))
                .to_string();
            braces.insert(brace, Brace::Module(name));
        } else if let Some(brace) = extern_block_brace_at(bytes, i) {
            braces.insert(brace, Brace::PassThrough);
        } else if let Some(brace) = member_body_brace_at(bytes, i) {
            braces.insert(brace, Brace::Members);
        } else if let Some((open, _)) = transparent_macro_body_at(bytes, i) {
            if bytes[open] == b'{' {
                braces.insert(open, Brace::CfgArms);
            }
        }
    }
    braces
}

/// The body `{` of an `impl`, `trait`, `enum`, `struct` or `union` item whose keyword starts at `i`.
/// `impl` and `union` are read as items only at an item position, since `impl Trait` is also a type
/// and `union` is a weak keyword; the other three are strict keywords that only ever head an item.
fn member_body_brace_at(bytes: &[u8], i: usize) -> Option<usize> {
    for (keyword, needs_item_position) in [
        (&b"impl"[..], true),
        (b"union", true),
        (b"trait", false),
        (b"enum", false),
        (b"struct", false),
    ] {
        if keyword_starts_at(bytes, i, keyword) && (!needs_item_position || at_item_start(bytes, i))
        {
            let first_only = keyword == b"struct";
            return item_delimiter(bytes, i + keyword.len(), !first_only)
                .and_then(|(at, delimiter)| (delimiter == b'{').then_some(at));
        }
    }
    None
}

/// The first `{` or `;` after `from` outside any `(…)`, `[…]` or `<…>` group — or, when
/// `skip_parens` is false, the first `{`, `(` or `;` outside `[…]` and `<…>`, which is how a tuple
/// struct is told from a braced one. The `>` of `->` and `=>` closes no group.
fn item_delimiter(bytes: &[u8], from: usize, skip_parens: bool) -> Option<(usize, u8)> {
    let (mut parens, mut brackets, mut angles) = (0usize, 0usize, 0usize);
    for (k, &byte) in bytes.iter().enumerate().skip(from) {
        let outside = parens == 0 && brackets == 0 && angles == 0;
        match byte {
            b'{' | b';' if outside => return Some((k, byte)),
            b'(' if outside && !skip_parens => return Some((k, byte)),
            b'(' => parens += 1,
            b')' => parens = parens.saturating_sub(1),
            b'[' => brackets += 1,
            b']' => brackets = brackets.saturating_sub(1),
            b'<' => angles += 1,
            b'>' if k > 0 && matches!(bytes[k - 1], b'-' | b'=') => {}
            b'>' => angles = angles.saturating_sub(1),
            _ => {}
        }
    }
    None
}

/// Whether the `{` at `i`, inside a `cfg_if!` body, opens one of its arms: it follows the arm's
/// `#[cfg(…)]` or an `else`.
fn follows_cfg_arm_head(bytes: &[u8], i: usize) -> bool {
    let mut j = i;
    while j > 0 && bytes[j - 1].is_ascii_whitespace() {
        j -= 1;
    }
    j > 0 && (bytes[j - 1] == b']' || (j >= 4 && keyword_starts_at(bytes, j - 4, b"else")))
}

/// Whether the keyword at `i` stands at an item position: at the start of the text, after `;`, `{`,
/// `}` or an attribute's `]`, or after a qualifier (`pub`, `pub(…)`, `unsafe`, `async`, `const`,
/// `extern`, `default`, `auto`) that itself stands at one. A keyword in a type or expression — the
/// `impl` of `-> impl Trait`, the `static` of `&'static str` — is not.
fn at_item_start(bytes: &[u8], i: usize) -> bool {
    let mut j = i;
    loop {
        while j > 0 && bytes[j - 1].is_ascii_whitespace() {
            j -= 1;
        }
        if j == 0 {
            return true;
        }
        match bytes[j - 1] {
            b';' | b'{' | b'}' | b']' => return true,
            b')' => {
                let mut depth = 0usize;
                let mut k = j;
                while k > 0 {
                    k -= 1;
                    match bytes[k] {
                        b')' => depth += 1,
                        b'(' => {
                            depth -= 1;
                            if depth == 0 {
                                break;
                            }
                        }
                        _ => {}
                    }
                }
                let mut w = k;
                while w > 0 && bytes[w - 1].is_ascii_whitespace() {
                    w -= 1;
                }
                if w >= 3 && keyword_starts_at(bytes, w - 3, b"pub") {
                    j = w - 3;
                } else {
                    return false;
                }
            }
            byte if is_ident_byte(byte) => {
                let mut start = j;
                while start > 0 && is_ident_byte(bytes[start - 1]) {
                    start -= 1;
                }
                if matches!(
                    &bytes[start..j],
                    b"pub" | b"unsafe" | b"async" | b"const" | b"extern" | b"default" | b"auto"
                ) {
                    j = start;
                } else {
                    return false;
                }
            }
            _ => return false,
        }
    }
}

/// The identifier starting at `at`, raw prefix removed, and the index just past it.
fn identifier_at(bytes: &[u8], at: usize) -> Option<(String, usize)> {
    let start = if bytes.get(at) == Some(&b'r') && bytes.get(at + 1) == Some(&b'#') {
        at + 2
    } else {
        at
    };
    let mut end = start;
    while end < bytes.len() && is_ident_byte(bytes[end]) {
        end += 1;
    }
    if end == start || bytes[start].is_ascii_digit() {
        return None;
    }
    Some((
        String::from_utf8_lossy(&bytes[start..end]).into_owned(),
        end,
    ))
}

/// The written target path of a `type Name<…> = Target;` whose name ends at `from`.
fn alias_target(source: &str, from: usize) -> Option<String> {
    let bytes = source.as_bytes();
    let mut j = from;
    while j < bytes.len() && bytes[j] != b'=' && bytes[j] != b';' {
        if bytes[j] == b'<' {
            j = skip_angles(bytes, j);
        } else {
            j += 1;
        }
    }
    if bytes.get(j) != Some(&b'=') {
        return None;
    }
    let end = alias_target_end(bytes, j + 1)?;
    let target = leading_path(source[j + 1..end].trim());
    (!target.is_empty()).then_some(target)
}

/// Index of the next non-whitespace byte at or after `i`.
pub(super) fn skip_ws(bytes: &[u8], i: usize) -> usize {
    let mut j = i;
    while j < bytes.len() && bytes[j].is_ascii_whitespace() {
        j += 1;
    }
    j
}

/// Index just past the balanced `<…>` group opening at `start` (`bytes[start] == '<'`); the end of
/// input if unbalanced (never panics).
pub(super) fn skip_angles(bytes: &[u8], start: usize) -> usize {
    let mut depth = 0usize;
    let mut k = start;
    while k < bytes.len() {
        match bytes[k] {
            b'<' => depth += 1,
            b'>' => {
                depth -= 1;
                if depth == 0 {
                    return k + 1;
                }
            }
            _ => {}
        }
        k += 1;
    }
    bytes.len()
}

/// Index of the top-level `;` that terminates a `type … = <target>;`, starting at `from` (just past
/// the aliasing `=`); the end of input if none. Tracks `[]`/`()`/`{}` nesting and skips `<…>` groups
/// whole (via [`skip_angles`], so a `->` return arrow's `>` is never miscounted), so a `;` inside an
/// array/tuple type (`[T; N]`) does not prematurely end the target.
pub(super) fn alias_target_end(bytes: &[u8], from: usize) -> Option<usize> {
    let mut depth = 0i32;
    let mut k = from;
    while k < bytes.len() {
        match bytes[k] {
            b'<' => {
                k = skip_angles(bytes, k);
                continue;
            }
            b'[' | b'(' | b'{' => depth += 1,
            b']' | b')' | b'}' => depth -= 1,
            b';' if depth <= 0 => return Some(k),
            _ => {}
        }
        k += 1;
    }
    None
}

/// The leading `::`-path of a type expression (`std::time::SystemTime<T>` → `std::time::SystemTime`,
/// `&Foo` → `Foo`). Stops at the first byte that is neither an identifier byte nor `:`.
pub(super) fn leading_path(expr: &str) -> String {
    let expr = expr.trim_start_matches(['&', ' ', '*']);
    let bytes = expr.as_bytes();
    let mut j = 0;
    while j < bytes.len() && (is_ident_byte(bytes[j]) || bytes[j] == b':') {
        j += 1;
    }
    expr[..j].trim_end_matches(':').to_string()
}

/// The `{` of an `extern` block starting at `i`, if one starts there: `extern {`, `extern "C" {`, and the
/// `unsafe extern "C" {` form Rust 2024 requires (the `unsafe` sits before the keyword, so matching on
/// `extern` alone reaches all three).
///
/// An extern block's brace opens no naming scope — its `fn`/`static` items are declared in the module
/// that CONTAINS the block, and can legally coexist with a `mod` of the same name because the two live in
/// different namespaces. `extern crate foo;` is deliberately not matched: it has no brace, so the `{`
/// requirement excludes it without a special case.
pub(super) fn extern_block_brace_at(bytes: &[u8], i: usize) -> Option<usize> {
    if !keyword_starts_at(bytes, i, b"extern") {
        return None;
    }
    let mut cursor = skip_ws(bytes, i + b"extern".len());
    if bytes.get(cursor) == Some(&b'"') {
        cursor += 1;
        while cursor < bytes.len() && bytes[cursor] != b'"' {
            cursor += 1;
        }
        cursor = skip_ws(bytes, cursor.saturating_add(1));
    }
    (bytes.get(cursor) == Some(&b'{')).then_some(cursor)
}

/// One `use … ;` statement in declaration-cleaned text: where its `use` keyword stands, its tree, and
/// whether a `pub` qualifier makes it a re-export.
pub(super) struct UseStatement {
    pub at: usize,
    pub body: String,
    visibility: Visibility,
}

impl UseStatement {
    /// Whether the statement carries any `pub` qualifier — what makes it part of the re-export closure.
    pub(super) fn is_pub(&self) -> bool {
        self.visibility != Visibility::Private
    }
}

/// Every `use … ;` statement in declaration-cleaned `source`, in order: the one enumeration every reader of
/// imports in this scanner starts from. A precise-capturing `use<…>` bound is not a statement, and an
/// unterminated statement ends the enumeration, as [`scan_use_statement`] decides.
pub(super) fn use_statements(source: &str) -> Vec<UseStatement> {
    let bytes = source.as_bytes();
    let mut statements = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if keyword_starts_at(bytes, i, b"use") {
            match scan_use_statement(bytes, source, i) {
                UseStatementScan::Statement { body, next } => {
                    statements.push(UseStatement {
                        at: i,
                        body,
                        visibility: visibility_before(bytes, i),
                    });
                    i = next;
                    continue;
                }
                UseStatementScan::NotAStatement { resume_at } => {
                    i = resume_at;
                    continue;
                }
                UseStatementScan::Unterminated => break,
            }
        }
        i += 1;
    }
    statements
}

/// One leaf of a use tree.
#[derive(Debug, PartialEq, Eq)]
pub(super) enum UseLeaf {
    /// A path the tree imports, and the name it binds: its ` as ` alias, or else its last segment.
    Name { path: String, binds: String },
    /// A glob, by its base path: `a::b::*` and the `*` of `a::b::{*}` are both `a::b`.
    Glob(String),
    /// A `{self}` leaf, `{self as x}` included: the group's prefix module itself.
    SelfLeaf(String),
}

/// Every leaf of a use tree — the one parser every reader of imports in this scanner shares. Groups are
/// expanded, a ` as ` alias is read as the name bound, and brace handling goes through [`brace_content`] /
/// [`split_top_commas`] (char-based, never a byte slice), so a malformed `}`-before-`{` cannot panic.
///
/// `cap` is the caller's measured stack-safety bound on group nesting: past it the tree is refused as a scan
/// error rather than read partially, since a real, compilable `use` nested that deep would otherwise vanish
/// from observation with no report — the false negative PROJECT.md's core contract forbids.
pub(super) fn use_tree_leaves(tree: &str, cap: usize) -> Result<Vec<UseLeaf>, String> {
    fn go(tree: &str, out: &mut Vec<UseLeaf>, depth: usize, cap: usize) -> Result<(), String> {
        if depth > cap {
            return Err(format!(
                "cannot judge a `use` tree nested past {cap} brace levels: '{tree}'"
            ));
        }
        let tree = tree.trim();
        match tree.find('{') {
            Some(open) => {
                let prefix = tree[..open].trim();
                let base = prefix.trim_end_matches(':').trim();
                for part in split_top_commas(&brace_content(&tree[open..])) {
                    let part = part.trim();
                    let head = part.find(" as ").map_or(part, |idx| part[..idx].trim());
                    if part.is_empty() {
                        continue;
                    } else if part == "*" {
                        if !base.is_empty() {
                            out.push(UseLeaf::Glob(base.to_string()));
                        }
                    } else if head == "self" {
                        if !base.is_empty() {
                            out.push(UseLeaf::SelfLeaf(base.to_string()));
                        }
                    } else {
                        go(&format!("{prefix}{part}"), out, depth + 1, cap)?;
                    }
                }
            }
            None => {
                let (path, alias) = match tree.split_once(" as ") {
                    Some((path, alias)) => (path.trim(), Some(alias.trim())),
                    None => (tree, None),
                };
                if let Some(base) = path.strip_suffix("::*") {
                    let base = base.trim();
                    if !base.is_empty() {
                        out.push(UseLeaf::Glob(base.to_string()));
                    }
                    return Ok(());
                }
                let path = path.trim_end_matches(':');
                if path.is_empty() {
                    return Ok(());
                }
                let named = alias.unwrap_or_else(|| {
                    path.rsplit_once("::").map_or(path, |(_, leaf)| leaf).trim()
                });
                let binds = canonical_module_path(named);
                if !binds.is_empty() {
                    out.push(UseLeaf::Name {
                        path: path.to_string(),
                        binds,
                    });
                }
            }
        }
        Ok(())
    }
    let mut out = Vec::new();
    go(tree, &mut out, 0, cap)?;
    Ok(out)
}

/// Resolve a *written* module path (from a `use` / `type` / `pub use`) to a canonical absolute
/// form, for the def closure and the scope table. A `std`/`core`/`alloc` head or any external head
/// stays as written (canonicalized); `crate`/`self`/`super` resolve against `current_module`. A head
/// naming a crate-root module resolves to `crate::…` where [`PathRoots::names_root_module`] says the
/// path starts at the crate root: bare at the crate root in any edition, and bare or `::`-rooted
/// anywhere in edition 2015.
pub(super) fn resolve_written_path(
    path: &str,
    current_module: &str,
    roots: PathRoots<'_>,
) -> Option<String> {
    let raw = path.trim();
    let global = raw.starts_with("::");
    let parts: Vec<String> = raw
        .trim_start_matches("::")
        .split("::")
        .map(|s| canonical_module_path(s.trim()))
        .filter(|s| !s.is_empty())
        .collect();
    let (head, _rest) = parts.split_first()?;
    let parts_str: Vec<&str> = parts.iter().map(String::as_str).collect();
    match head.as_str() {
        "std" | "core" | "alloc" => Some(parts.join("::")),
        "crate" => fold_canonical_segments(&parts_str),
        _ if global && !roots.edition_2015 => Some(parts.join("::")),
        "self" | "super" if !global => resolve_self_super(current_module, &parts_str),
        other => {
            if roots.names_root_module(current_module, other) {
                let mut out = vec!["crate".to_string()];
                out.extend(parts.iter().cloned());
                Some(out.join("::"))
            } else {
                Some(parts.join("::"))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn resolved(source: &str, at: &str, head: &str, ns: Namespace) -> Option<String> {
        let roots = PathRoots {
            root_modules: &[],
            edition_2015: false,
        };
        let table = ScopeTable::build(source, "crate::core", roots).unwrap();
        let crate_scopes = CrateScopes::new([&table]);
        let offset = source.find(at).unwrap();
        match table.resolve(table.scope_at(offset), head, &[], ns, &crate_scopes) {
            Head::Paths(paths) => Some(paths.join(" | ")),
            Head::Local => Some("<local>".to_string()),
            Head::Unbound => None,
        }
    }

    #[test]
    fn a_block_use_binds_its_whole_block_and_nothing_outside() {
        let source = "use crate::a::X;\nfn h() { X::fa(); }\nfn g() { X::early(); use crate::b::X; X::fb(); }\nfn k() { X::late(); }\n";
        for (at, expected) in [
            ("X::fa", "crate::a::X"),
            ("X::early", "crate::b::X"),
            ("X::fb", "crate::b::X"),
            ("X::late", "crate::a::X"),
        ] {
            assert_eq!(
                resolved(source, at, "X", Namespace::Type).as_deref(),
                Some(expected),
                "{at}"
            );
        }
    }

    #[test]
    fn a_block_item_shadows_in_its_namespace_only() {
        let source = "use std::process::Command;\nfn g() { struct Command {} let _ = Command::new(); let _ = Command(); }\n";
        assert_eq!(
            resolved(source, "Command::new", "Command", Namespace::Type).as_deref(),
            Some("<local>")
        );
        assert_eq!(
            resolved(source, "Command()", "Command", Namespace::Value).as_deref(),
            Some("std::process::Command"),
            "a braced struct names no value"
        );
    }

    #[test]
    fn member_bodies_bind_nothing_and_pass_through_bodies_bind_outward() {
        let source = "use std::process::Command;\nimpl S { const Command: u8 = 0; fn f() { Command::a(); } }\nenum E { Command }\nfn g() { extern \"C\" { fn Command(); } Command(); }\ncfg_if! { if #[cfg(unix)] { use crate::u::Y; } else { use crate::w::Y; } }\nfn y() { Y::b(); }\n";
        assert_eq!(
            resolved(source, "Command::a", "Command", Namespace::Type).as_deref(),
            Some("std::process::Command")
        );
        assert_eq!(
            resolved(source, "Command();", "Command", Namespace::Value).as_deref(),
            Some("<local>"),
            "an extern block's item belongs to the enclosing block"
        );
        assert_eq!(
            resolved(source, "Y::b", "Y", Namespace::Type).as_deref(),
            Some("crate::u::Y | crate::w::Y"),
            "each cfg_if arm's use binds at module scope, and both are candidates"
        );
    }

    #[test]
    fn a_keyword_in_a_type_heads_no_item() {
        let source = "use std::process::Command;\nfn g() -> impl Sized { let _: &'static str = \"\"; Command::new() }\n";
        assert_eq!(
            resolved(source, "Command::new", "Command", Namespace::Type).as_deref(),
            Some("std::process::Command")
        );
    }
}
