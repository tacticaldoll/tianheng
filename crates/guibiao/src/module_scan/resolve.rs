//! The one resolver: what a written path names from the scope it stands in, and what a crate-rooted path names once
//! every module of a compilation unit is read.
//!
//! A lookup walks the occurrence's chain of blocks up to its module; where one scope binds a name more than once —
//! cfg-exclusive imports, which the scanner reads cfg-blind — every binding is a candidate. A glob is an edge to the
//! module it names: every glob's targets are read once for the unit, as a fixed point, and a lookup through a glob the
//! [`Walk`] is already looking the same name up through ends as a cycle. Each segment a
//! module binds rather than declares is replaced by every path that binding names, and every path a resolution passes
//! is a path it names. Every written path's first segment is classified by one dispatch, [`written_root`], and every
//! answer is a typed [`Named`] each reader matches exhaustively.
//!
//! [`MAX_RESOLUTION_CHAIN`] caps two counts, each on its own: the bindings and glob paths one [`Walk`] is nested
//! inside, and the segments one branch of [`CrateScopes::denote`] replaces, each branch counted from where it began. A
//! binding or glob a walk already holds, or a segment a branch already replaced, ends it as a cycle. What a lookup
//! through globs reaches is no walk: [`CrateScopes::through_globs`] reads every scope it reaches once into one graph
//! and settles it, and [`CrateScopes::glob_targets`] reads what the unit's globs name in passes, each bounded by the
//! size of what it reads rather than by the cap. Scopes, modules, blocks
//! and names are held in ordered maps, so every lookup meets them in one order; a fold across several answers —
//! [`joined`], a crate-rooted path's candidates, the glob hazard — takes the least refusal and reads every answer,
//! while a refusal met inside one scope's lookup ends that lookup with it.

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet, HashMap};

use super::item_head::Visibility;
use super::path_vocab::path_within;
use super::path_vocab::{PathSite, WrittenRoot, written_root};
use super::path_vocab::{Sysroot, sysroot_crate};
use super::path_vocab::{block_segment, is_block_segment, names_a_block_item, readable_module};
pub(super) use super::scope_tree::Namespace;
use super::scope_tree::{Binding, DeclKind, Declaration, ScopeKind, ScopeTable};
use super::token_tree::Edition;

/// The cap on the links of a chain of `type` aliases, imports, globs and re-exports a resolution walks, which bounds
/// how deep a pathological source can walk — set far beyond any real source. A resolution that walks past it answers
/// [`Named::PastCap`] with [`chain_refusal`], which the reader refuses as a scan error. How wide a walk reads is
/// bounded apart from it, by the memo [`CrateScopes::scope_lookup`] keeps: a scope's answer for a name is read once per
/// depth rather than once per path, so a lattice of globs is read in time its size bounds rather than its path count.
const MAX_RESOLUTION_CHAIN: usize = 64;

/// The refusal of a chain longer than [`MAX_RESOLUTION_CHAIN`] links, quoting the binding it was measured
/// from and the module that binding is written in: the one wording every walk that meets the cap uses.
fn chain_refusal(quote: &str, module: &str) -> String {
    format!(
        "cannot judge a chain of more than {MAX_RESOLUTION_CHAIN} imports, globs, re-exports and `type` \
         aliases from `{quote};` in {}",
        readable_module(module)
    )
}

/// What a written path names from the scope it stands in: the resolver's typed answer, which each
/// reader matches exhaustively.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum Named {
    /// Every path it names — through the bindings of the nearest scope that binds its head, an item a
    /// module declares, a sysroot crate or a crate-rooted path — each carried through what a
    /// crate-rooted path names.
    Paths(Vec<String>),
    /// A crate the path names by its root (`::dep::…`, and in edition 2015 a root path no crate-root
    /// name claims), without the `::`.
    External(String),
    /// A bare head no scope between the path and its module binds, and that names no sysroot crate.
    Unbound { head: String, rest: Vec<String> },
    /// A block-local item. It is named by no path outside its block, so no prefix reaches it.
    Local,
    /// Nothing a path can start from.
    Invalid,
    /// A refusal of the walk rather than an answer, carrying its message: a chain longer than
    /// [`MAX_RESOLUTION_CHAIN`] links, quoting where it was measured from; the answers of the globs reaching a
    /// head not settling within the graph's readings; or a compilation unit's globs not settling on what they
    /// name within the fixed point's passes.
    PastCap(String),
}

/// A declaration a lookup reached, by what a path through it reads next.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum Declared {
    /// A module — a declared one, or the crate an `extern crate self` names — whose own scope the rest of a
    /// path is read in.
    Module(String),
    /// An item, or the root of a crate whose contents are not read: the path names it, followed by the rest.
    Item(String),
}

impl Declared {
    pub(super) fn path(&self) -> &str {
        match self {
            Declared::Module(path) | Declared::Item(path) => path,
        }
    }
}

/// A path a binding names, still to be read, with how that binding is written — what a refusal of a chain through
/// it quotes, recorded where the binding is met rather than looked up again.
type Bound = (String, String);

/// What a name is looked up as in one scope, before the rest of its path is read.
#[derive(Clone, PartialEq, Eq)]
enum Head {
    /// Every candidate the scope binds or declares it as, or else what its globs bring: the paths its
    /// bindings name, still to be read — a name a glob brings is a binding of the scope the glob is written
    /// in — and the items it declares, by crate-rooted path, which name themselves. Neither removes the
    /// other — cfg-exclusive, both can be the live one. `through` holds the paths a name a glob brings passes
    /// on its way — the glob's module followed by the name — which a call reports under and which are not read
    /// again, since what they name is already among the candidates, read where the glob stands.
    ///
    /// `heads` holds what the name alone names where the lookup found it, before anything it names is read further:
    /// the paths its bindings name and the paths it declares, and for a name a glob brings the glob's module followed
    /// by the name. A fold of several answers joins them, so a name bound in one namespace and brought by a glob in
    /// the other keeps both.
    ///
    /// `foreign` holds what a glob of a crate whose contents are not read brings the name as, where a fold joined such
    /// an answer with this one — cfg-exclusive globs or files, or the two namespaces of one lookup — since either can
    /// be the live one; it is read as [`Head::Foreign`] is, by a path through the scope's module alone.
    Candidates {
        bound: Vec<Bound>,
        declared: Vec<Declared>,
        through: Vec<String>,
        heads: Vec<String>,
        foreign: Vec<String>,
    },
    Local,
    /// Nothing the scope binds or declares, and nothing its globs of this compilation unit bring, but a glob of
    /// a crate whose contents are not read can: that glob's path with the name appended, for each such glob.
    /// A path through the scope's module names what one of them names, since nothing else can; a bare head
    /// written in the scope may instead name a prelude or a local binding, so the lexical lookup reads it as
    /// [`Head::Unbound`] and the glob's own finding is what reacts.
    Foreign(Vec<String>),
    Unbound,
    /// A refusal of the walk, as [`Named::PastCap`] carries one.
    PastCap(String),
}

/// One scope a lookup through globs reaches, seen from a module: its table, its scope and that module.
type GlobNode = (usize, u32, String);

/// What [`CrateScopes::through_globs`] read of one scope: its answer, where the scope binds or declares the name for
/// certain or its answer is already kept, or the globs it reads the name through — beside `own`, what the scope binds
/// or declares it as where that may not hold on every build ([`CrateScopes::may_not_hold_here`]), joined with what the
/// globs bring.
enum GlobRead {
    Answered(Head),
    Globs {
        own: Option<Head>,
        edges: Vec<GlobEdge>,
    },
}

/// One glob of a scope a lookup through globs reads: into the scopes of a module of the unit, by their positions in
/// the graph, or a name of a crate whose contents are not read.
enum GlobEdge {
    Into {
        quote: String,
        target: String,
        into: Vec<usize>,
    },
    Foreign(String),
}

/// The scopes a lookup through globs reaches, each once, in the order they were met, and what each read.
#[derive(Default)]
struct GlobGraph {
    nodes: Vec<GlobNode>,
    index: HashMap<GlobNode, usize>,
    reads: Vec<GlobRead>,
}

impl GlobGraph {
    /// The position of a scope seen from `from`, added where it is met first.
    fn node(&mut self, t: usize, id: u32, from: &str) -> usize {
        let key = (t, id, from.to_string());
        if let Some(&n) = self.index.get(&key) {
            return n;
        }
        self.nodes.push(key.clone());
        self.index.insert(key, self.nodes.len() - 1);
        self.nodes.len() - 1
    }

    /// Every scope's answer for `head`, read to a fixed point: a scope that reads its globs answers what they bring
    /// joined, starting from none, each read after the scopes its globs reach where no cycle forbids it, and every
    /// such scope is read again until a reading changes nothing. The readings are bounded by the graph's size plus
    /// one; past it every answer is refused, since a graph whose answers do not settle is one the scanner cannot judge.
    fn settle(&self, head: &str) -> Result<Vec<Head>, String> {
        let order = self.children_first();
        let mut answers: Vec<Head> = self
            .reads
            .iter()
            .map(|read| match read {
                GlobRead::Answered(answer) => answer.clone(),
                GlobRead::Globs { own, .. } => own.clone().unwrap_or(Head::Unbound),
            })
            .collect();
        for _ in 0..=self.nodes.len() {
            let mut changed = false;
            for &n in &order {
                let GlobRead::Globs { own, edges } = &self.reads[n] else {
                    continue;
                };
                let globbed = match joined(edges.iter().map(|edge| match edge {
                    GlobEdge::Foreign(path) => Head::Foreign(vec![path.clone()]),
                    GlobEdge::Into {
                        quote,
                        target,
                        into,
                    } => brought_through(
                        joined(into.iter().map(|&m| answers[m].clone())),
                        quote,
                        target,
                        head,
                    ),
                })) {
                    Head::Local => Head::Unbound,
                    other => other,
                };
                let answer = match (own, globbed) {
                    (None, globbed) => globbed,
                    (Some(own), Head::Unbound) => own.clone(),
                    (Some(own), globbed) => joined([own.clone(), globbed]),
                };
                if answer != answers[n] {
                    answers[n] = answer;
                    changed = true;
                }
            }
            if !changed {
                return Ok(answers);
            }
        }
        Err(format!(
            "cannot judge what the globs reaching `{head}` bring: their answers do not settle within {} readings",
            self.nodes.len() + 1
        ))
    }

    /// Every node, each after the nodes its globs reach where no cycle forbids it: a depth-first walk from each node
    /// not yet met, held on an explicit stack so a long chain of globs is no recursion.
    fn children_first(&self) -> Vec<usize> {
        let children = |n: usize| -> Vec<usize> {
            match &self.reads[n] {
                GlobRead::Globs { edges, .. } => edges
                    .iter()
                    .flat_map(|edge| match edge {
                        GlobEdge::Into { into, .. } => into.clone(),
                        GlobEdge::Foreign(_) => Vec::new(),
                    })
                    .collect(),
                GlobRead::Answered(_) => Vec::new(),
            }
        };
        let mut order = Vec::with_capacity(self.nodes.len());
        let mut seen = vec![false; self.nodes.len()];
        for root in 0..self.nodes.len() {
            if seen[root] {
                continue;
            }
            seen[root] = true;
            let mut stack = vec![(root, children(root), 0usize)];
            while let Some((n, kids, next)) = stack.last_mut() {
                if let Some(&kid) = kids.get(*next) {
                    *next += 1;
                    if !seen[kid] {
                        seen[kid] = true;
                        stack.push((kid, children(kid), 0));
                    }
                } else {
                    order.push(*n);
                    stack.pop();
                }
            }
        }
        order
    }
}

/// What a glob of `target`, written `quote`, brings `head` as from what `target`'s scopes answer: a name `target`
/// binds or declares is bound where the glob stands, through the glob's own quote for a declaration, the path
/// `target::head` is passed on the way, and what the name alone names there is that path.
fn brought_through(found: Head, quote: &str, target: &str, head: &str) -> Head {
    match found {
        Head::Candidates {
            bound,
            declared,
            through,
            foreign,
            ..
        } => Head::Candidates {
            bound: bound
                .into_iter()
                .chain(
                    declared
                        .iter()
                        .map(|d| (d.path().to_string(), quote.to_string())),
                )
                .collect(),
            declared: Vec::new(),
            through: through
                .into_iter()
                .chain([format!("{target}::{head}")])
                .collect(),
            heads: vec![format!("{target}::{head}")],
            foreign,
        },
        other => other,
    }
}

/// Whether a path names something in a namespace, as [`CrateScopes::denote_in`] ends its reading of it: it reaches a
/// declaration or a binding there; its last segment names nothing its module binds or declares in that namespace; or
/// what it reaches is not read — a crate whose contents are not read, a segment that names nothing before the last,
/// a cycle. Ordered from what says least to what says most, so the answer over several readings is their maximum.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
enum Presence {
    #[default]
    Absent,
    Unknown,
    Known,
}

/// What [`CrateScopes::denote_in`] reads a path as: every path the reading passes through, and whether the path
/// names something in the namespace its last segment was read in.
#[derive(Clone, Debug, Default)]
struct Denoted {
    paths: Vec<String>,
    presence: Presence,
}

/// `path` with each segment of `rest` appended after a `::`.
pub(super) fn with_rest(mut path: String, rest: &[String]) -> String {
    for segment in rest {
        path.push_str("::");
        path.push_str(segment);
    }
    path
}

/// What an `extern crate` naming `target` declares: the crate root's module for `crate`, and otherwise the
/// root of a crate whose contents are not read.
pub(super) fn extern_crate_names(target: &str) -> Declared {
    if target == "crate" {
        Declared::Module(target.to_string())
    } else {
        Declared::Item(target.to_string())
    }
}

/// The bindings and globs one resolution is inside, outermost first, each with what it was written
/// as: how a cycle ends a resolution, and how a chain longer than the cap is refused, quoting the
/// binding it was measured from. `cuts` counts every cycle ended while reading, conservatively: a cycle
/// the walk ends makes the answer depend on the walk it was read in, and a cycle a path's own chain ends
/// is counted beside it, so an answer is remembered only where no cycle was cut at all.
#[derive(Default)]
pub(super) struct Walk {
    resolving: Vec<Resolving>,
    cuts: usize,
}

struct Resolving {
    key: (usize, u32, String),
    quote: String,
    module: String,
}

impl Walk {
    /// Whether the walk is already inside the binding or glob `key` — a cycle, which the walk ends there and
    /// counts.
    fn meets_cycle(&mut self, key: &(usize, u32, String)) -> bool {
        let cycle = self.resolving.iter().any(|entry| &entry.key == key);
        self.cuts += usize::from(cycle);
        cycle
    }

    /// Whether `key` is the binding the walk is reading now: the head of `use md5x::md5x;` looked up in its own scope.
    /// rustc resolves an import's path without that import, so the scope is read without it, which is the scope's
    /// answer for that lookup rather than a cycle the walk cut — and so it is kept, under its own memo key.
    fn reads_itself(&self, key: &(usize, u32, String)) -> bool {
        self.resolving.last().is_some_and(|entry| &entry.key == key)
    }

    /// Enter a binding, unless the chain already holds [`MAX_RESOLUTION_CHAIN`] links.
    fn enter(
        &mut self,
        key: (usize, u32, String),
        quote: String,
        module: &str,
    ) -> Result<(), String> {
        if self.resolving.len() >= MAX_RESOLUTION_CHAIN {
            let first = &self.resolving[0];
            return Err(chain_refusal(&first.quote, &first.module));
        }
        self.resolving.push(Resolving {
            key,
            quote,
            module: module.to_string(),
        });
        Ok(())
    }

    fn leave(&mut self) {
        self.resolving.pop();
    }
}

/// What one binding names, read from its own scope: the paths still to be read, and the paths passed on the way
/// that are not.
pub(super) enum BindingNames {
    Paths(Vec<String>, Vec<String>),
    Local,
    PastCap(String),
}

/// A memo key: a written path, the table and scope it stands in, where it stands, and the namespace its
/// last segment is read in.
type NameKey = (usize, u32, String, PathSite, Namespace);

/// A memo key for an import's presence: its table, scope, name, written path and namespace.
type PresenceKey = (usize, u32, String, String, Namespace);

/// A memo key for [`CrateScopes::denote`]: a crate-rooted path, and the namespace its last segment is read in.
type DenoteKey = (String, Namespace);

/// A glob, by its table, its scope and its position among that scope's globs.
type GlobKey = (usize, u32, usize);

/// What each glob of the unit names, by [`GlobKey`]: the paths it names, or the refusal met reading it.
type GlobTargets = HashMap<GlobKey, Result<Vec<String>, String>>;

/// A memo key for [`CrateScopes::scope_lookup`]: the table and scope, the name, its namespace, the module it is seen
/// from, how many links deep the walk stands, since whether a lookup meets the chain cap depends on that depth, and
/// whether the lookup is the head of the binding the walk is reading, which reads the scope without that binding.
type LookupKey = (usize, u32, String, Namespace, String, usize, bool);

/// Every table of one compilation unit: what each module binds at module scope, what each declares,
/// and the edition a written path's root is read in. It resolves every binding — a `use` leaf, a
/// `pub use`, a glob and a `type` alias alike — from the scope it is written in when it is looked up,
/// never when it is recorded, and answers what a head names from a scope and what a crate-rooted path
/// names.
pub(super) struct CrateScopes {
    pub(super) tables: Vec<ScopeTable>,
    edition: Edition,
    /// Whether the unit is a proc-macro crate, whose extern prelude holds `proc_macro` beside the sysroot crates.
    proc_macro: bool,
    /// Every module scope of the unit, `(table, scope)`, by module path — a module declared twice
    /// under exclusive cfgs has one scope per declaration.
    pub(super) modules: BTreeMap<String, Vec<(usize, u32)>>,
    /// Every block, by its module's path followed by its [`block_segment`], so a path through a module
    /// declared in a block is read in the block that declares it.
    blocks: BTreeMap<String, (usize, u32)>,
    /// What each `extern crate` of the crate root binds: in scope in every module of the unit, after
    /// what the module's own scopes bind.
    extern_prelude: BTreeMap<String, Vec<Declared>>,
    named: RefCell<HashMap<NameKey, Named>>,
    denoted: RefCell<HashMap<DenoteKey, Result<Vec<String>, String>>>,
    /// What [`CrateScopes::scope_lookup`] answered, by [`LookupKey`], where no cycle was cut while answering.
    looked: RefCell<HashMap<LookupKey, Head>>,
    /// Whether each import names something in a namespace, by the import's table, scope, name, written path and
    /// namespace — a scope may import one name more than once, under exclusive cfgs — read once, since a chain of
    /// re-exports asks it of every link as each lookup through the chain passes.
    presence: RefCell<HashMap<PresenceKey, Presence>>,
    /// Every glob's targets once [`CrateScopes::glob_targets`] has read them to a fixed point.
    globs: RefCell<Option<GlobTargets>>,
    /// The reading of every glob's targets in progress while that fixed point is iterated, which a glob read during
    /// an iteration answers from.
    reading_globs: RefCell<Option<GlobTargets>>,
    /// The glob whose own path is being read while the fixed point is iterated, which names nothing meanwhile.
    reading_glob: RefCell<Option<GlobKey>>,
    /// Whether the glob being read was asked for while it named nothing.
    read_itself: std::cell::Cell<bool>,
}

impl CrateScopes {
    /// Whether the unit's extern prelude holds the sysroot crate `head` names.
    fn extern_prelude_holds(&self, head: &str) -> bool {
        match sysroot_crate(head) {
            Some(Sysroot::Prelude) => true,
            Some(Sysroot::ProcMacro) => self.proc_macro,
            Some(Sysroot::Test) | None => false,
        }
    }

    /// These scopes, read as a proc-macro crate's, whose extern prelude holds `proc_macro`.
    pub(super) fn in_a_proc_macro_crate(self) -> Self {
        CrateScopes {
            proc_macro: true,
            ..self
        }
    }

    /// Hold `tables`, the unit's files, in their edition. Nothing is resolved here: a binding no
    /// resolution walks through is never read.
    pub(super) fn new(tables: Vec<ScopeTable>, edition: Edition) -> Self {
        let mut modules: BTreeMap<String, Vec<(usize, u32)>> = BTreeMap::new();
        let mut blocks: BTreeMap<String, (usize, u32)> = BTreeMap::new();
        let mut extern_prelude: BTreeMap<String, Vec<Declared>> = BTreeMap::new();
        for (t, table) in tables.iter().enumerate() {
            for (id, scope) in table.scopes.iter().enumerate() {
                let id = u32::try_from(id).expect("scope table exceeds u32");
                match scope.kind {
                    ScopeKind::Module => modules
                        .entry(scope.module.clone())
                        .or_default()
                        .push((t, id)),
                    ScopeKind::Block => {
                        blocks.insert(
                            format!("{}::{}", scope.module, block_segment(table.table, id)),
                            (t, id),
                        );
                    }
                }
                if scope.kind == ScopeKind::Module && scope.module == "crate" {
                    for (name, all) in &scope.declarations {
                        for declaration in all {
                            if let DeclKind::ExternCrate(target) = &declaration.kind {
                                extern_prelude
                                    .entry(name.clone())
                                    .or_default()
                                    .push(extern_crate_names(target));
                            }
                        }
                    }
                }
            }
        }
        CrateScopes {
            tables,
            edition,
            proc_macro: false,
            modules,
            blocks,
            extern_prelude,
            named: RefCell::default(),
            denoted: RefCell::default(),
            looked: RefCell::default(),
            presence: RefCell::default(),
            globs: RefCell::default(),
            reading_globs: RefCell::default(),
            reading_glob: RefCell::default(),
            read_itself: std::cell::Cell::default(),
        }
    }

    pub(super) fn table(&self, t: usize) -> &ScopeTable {
        &self.tables[t]
    }

    /// What `written`, standing in `scope` of table `t`, names through its head alone: the paths its head's bindings
    /// and declarations name, each with the rest of `written` after it, and no path carried further through what
    /// those name. What an import names, where a re-export the rest of its path runs through is not followed. A head a
    /// glob brings is bound as the glob's module followed by the head, whatever that module binds the head as.
    pub(super) fn head_names(
        &self,
        t: usize,
        scope: u32,
        written: &str,
        site: PathSite,
        ns: Namespace,
    ) -> Named {
        self.name_raw(
            t,
            scope,
            written,
            site,
            ns,
            &mut Vec::new(),
            &mut Walk::default(),
            true,
        )
    }

    /// What `written`, standing in `scope` of table `t`, names — its last segment read in `ns` — with
    /// every path carried through what a crate-rooted path names. Memoized by the scope and the path.
    pub(super) fn name(
        &self,
        t: usize,
        scope: u32,
        written: &str,
        site: PathSite,
        ns: Namespace,
    ) -> Named {
        let key = (t, scope, written.to_string(), site, ns);
        if let Some(named) = self.named.borrow().get(&key) {
            return named.clone();
        }
        let mut through = Vec::new();
        let named = match self.name_raw(
            t,
            scope,
            written,
            site,
            ns,
            &mut through,
            &mut Walk::default(),
            false,
        ) {
            Named::Paths(paths) => {
                let mut denoted = through;
                let mut past_cap = None;
                for path in &paths {
                    match self.denote(path, ns) {
                        Ok(found) => denoted.extend(found),
                        Err(refusal) => past_cap = least(past_cap, refusal),
                    }
                }
                let reached = !denoted.is_empty();
                denoted.retain(|path| !names_a_block_item(path));
                match past_cap {
                    Some(refusal) => Named::PastCap(refusal),
                    None if reached && denoted.is_empty() => Named::Local,
                    None => Named::Paths(sorted_unique(denoted)),
                }
            }
            other => other,
        };
        self.named.borrow_mut().insert(key, named.clone());
        named
    }

    /// Every path the crate-rooted `path`, its last segment read in `ns`, names: each segment a module binds
    /// rather than declares is replaced by every path its bindings name there, to a fixed point, and a path
    /// through no such segment names itself. A path rooted anywhere else names itself. Memoized by the path
    /// and the namespace.
    pub(super) fn denote(&self, path: &str, ns: Namespace) -> Result<Vec<String>, String> {
        let key = (path.to_string(), ns);
        if let Some(found) = self.denoted.borrow().get(&key) {
            return found.clone();
        }
        let found = self
            .denote_in(path, ns, &mut Walk::default())
            .map(|denoted| denoted.paths);
        self.denoted.borrow_mut().insert(key, found.clone());
        found
    }

    /// What `written` names from `scope`, before a crate-rooted path is carried through what it names:
    /// the one resolution every binding, glob and occurrence goes through.
    #[allow(clippy::too_many_arguments)]
    fn name_raw(
        &self,
        t: usize,
        scope: u32,
        written: &str,
        site: PathSite,
        ns: Namespace,
        through: &mut Vec<String>,
        walk: &mut Walk,
        head_alone: bool,
    ) -> Named {
        let module = &self.tables[t].scopes[scope as usize].module;
        let mut named_by =
            |head: Head, head_name: String, rest: Vec<String>, from_root: bool| match head {
                Head::Candidates { heads, foreign, .. } if head_alone => Named::Paths(
                    heads
                        .into_iter()
                        .chain(foreign.into_iter().filter(|_| from_root))
                        .map(|p| with_rest(p, &rest))
                        .collect(),
                ),
                Head::Candidates {
                    bound,
                    declared,
                    through: passed,
                    foreign,
                    ..
                } => {
                    through.extend(passed.into_iter().map(|p| with_rest(p, &rest)));
                    Named::Paths(
                        bound
                            .into_iter()
                            .map(|(path, _)| path)
                            .chain(declared.iter().map(|d| d.path().to_string()))
                            .chain(foreign.into_iter().filter(|_| from_root))
                            .map(|p| with_rest(p, &rest))
                            .collect(),
                    )
                }
                Head::Foreign(paths) if from_root => {
                    Named::Paths(paths.into_iter().map(|p| with_rest(p, &rest)).collect())
                }
                Head::Local => Named::Local,
                Head::PastCap(refusal) => Named::PastCap(refusal),
                Head::Unbound | Head::Foreign(_) if self.extern_prelude_holds(&head_name) => {
                    Named::Paths(vec![with_rest(head_name, &rest)])
                }
                Head::Unbound if from_root => Named::External(with_rest(head_name, &rest)),
                Head::Unbound | Head::Foreign(_) => Named::Unbound {
                    head: head_name,
                    rest,
                },
            };
        let last_ns = |rest: &[String]| if rest.is_empty() { ns } else { Namespace::Type };
        match written_root(written, module, site, self.edition) {
            WrittenRoot::Crate(path) => Named::Paths(vec![path]),
            WrittenRoot::Extern { head, rest } => {
                let found = self.in_extern_prelude(&head, last_ns(&rest));
                named_by(found, head, rest, true)
            }
            WrittenRoot::FromCrateRoot { head, rest } => {
                let found = self.lookup_in_module("crate", &head, last_ns(&rest), "crate", walk);
                named_by(found, head, rest, true)
            }
            WrittenRoot::Bare { head, rest } => {
                let found = self.lookup(t, scope, &head, last_ns(&rest), walk);
                named_by(found, head, rest, false)
            }
            WrittenRoot::Invalid => Named::Invalid,
        }
    }

    /// What `head` names from `scope`: the answer of the nearest scope on the chain of blocks up to the
    /// nearest module scope that answers it, each read by [`CrateScopes::scope_lookup`], and else what an
    /// `extern crate` of the crate root binds it as. What only a glob of a crate whose contents are not read
    /// could bring is no answer here — a bare head may name a prelude or a local binding instead — so the
    /// chain walks on past it.
    ///
    /// A block whose only answer is an import of what is not read — `use std::fmt;` — may hold the name in the other
    /// namespace alone, as `std::fmt` names a module and no value, so rustc reads `fmt()` past it from the scope around
    /// the block. Its answer is kept and the chain walks on, the answers joined, so the call reports under what the
    /// outer scope binds it as: `use crate::forbidden::fmt; fn g() { use std::fmt; fmt(); }` calls
    /// `crate::forbidden::fmt`, measured against rustc 1.96.0, edition 2021. A block holding the name only by a
    /// `cfg`-gated item — an import, or an item of the block — walks on the same way, and a module scope of either kind
    /// joins the extern prelude's answer: both are [`CrateScopes::may_not_hold_here`], the one judgement every lookup
    /// asks of a scope, asked of every answer a scope binds or declares the name as. A name read in both namespaces is
    /// looked up in each to the end of its chain before the two are joined, so a block's `struct now {}`, a type alone,
    /// leaves a value `now` to the scope around the block.
    fn lookup(&self, t: usize, scope: u32, head: &str, ns: Namespace, walk: &mut Walk) -> Head {
        if ns == Namespace::Either {
            return joined(
                [Namespace::Type, Namespace::Value].map(|ns| self.lookup(t, scope, head, ns, walk)),
            );
        }
        let mut current = Some(scope);
        let mut unsettled = Vec::new();
        while let Some(id) = current {
            let entry = &self.tables[t].scopes[id as usize];
            match self.scope_lookup(t, id, head, ns, &entry.module, walk) {
                Head::Unbound | Head::Foreign(_) if entry.kind == ScopeKind::Block => {
                    current = entry.parent;
                }
                Head::Unbound | Head::Foreign(_) => break,
                answer if !self.answer_ends_lookup(&answer, t, id, head, ns, walk) => {
                    unsettled.push(answer);
                    current = match entry.kind {
                        ScopeKind::Block => entry.parent,
                        ScopeKind::Module => None,
                    };
                }
                answer if unsettled.is_empty() => return answer,
                answer => return joined(unsettled.into_iter().chain([answer])),
            }
        }
        let prelude = self.in_extern_prelude(head, ns);
        if unsettled.is_empty() {
            prelude
        } else {
            joined(unsettled.into_iter().chain([prelude]))
        }
    }

    /// Whether `answer`, what scope `id` gives for `head` in `ns`, ends a lookup there: every answer does except one
    /// naming what the scope binds or declares — its candidates, or an item of a block — where that may not hold on
    /// every build ([`CrateScopes::may_not_hold_here`]). The one place an answer's kind is asked, so where the lookup
    /// walks out of a block, where it reads the scope's own globs, and where a lookup through globs reaches the scope,
    /// one answer is read one way.
    fn answer_ends_lookup(
        &self,
        answer: &Head,
        t: usize,
        id: u32,
        head: &str,
        ns: Namespace,
        walk: &mut Walk,
    ) -> bool {
        !(matches!(answer, Head::Candidates { .. } | Head::Local)
            && self.may_not_hold_here(t, id, head, ns, walk))
    }

    /// Whether what scope `id` binds or declares `head` as in `ns` may not hold on every build, so the lookup reads on
    /// past it and joins what it finds: the scope holds the name only by items a `cfg` gates
    /// ([`CrateScopes::held_only_where_gated`]), or only through imports of what is not read
    /// ([`CrateScopes::binds_only_what_is_not_read`]). The one answer to whether a scope's answer ends a lookup, read
    /// alike where the lookup walks out of a block, where it reads a scope's own globs, and where a lookup through globs
    /// reaches the scope.
    fn may_not_hold_here(
        &self,
        t: usize,
        id: u32,
        head: &str,
        ns: Namespace,
        walk: &mut Walk,
    ) -> bool {
        self.held_only_where_gated(t, id, head, ns)
            || self.binds_only_what_is_not_read(t, id, head, ns, walk)
    }

    /// Whether scope `id` binds or declares `head` in `ns` only by items a `cfg` gates, so no binding or declaration
    /// of it there is compiled on every build.
    fn held_only_where_gated(&self, t: usize, id: u32, head: &str, ns: Namespace) -> bool {
        let entry = &self.tables[t].scopes[id as usize];
        let held = entry.ungated.get(head);
        (entry.bindings.contains_key(head) || entry.declarations.contains_key(head))
            && !held.is_some_and(|&(type_ns, value_ns)| match ns {
                Namespace::Type => type_ns,
                Namespace::Value => value_ns,
                Namespace::Either => type_ns || value_ns,
            })
    }

    /// Whether block `id` holds `head` in `ns` only through imports whose target is not read in `ns` — neither
    /// declared nor bound there, nor aliased, and every import of it of [`Presence::Unknown`] — so the block may not
    /// hold it in `ns` at all. [`Namespace::Either`] asks each namespace, and one that may not be held is enough.
    fn binds_only_what_is_not_read(
        &self,
        t: usize,
        id: u32,
        head: &str,
        ns: Namespace,
        walk: &mut Walk,
    ) -> bool {
        if ns == Namespace::Either {
            return [Namespace::Type, Namespace::Value]
                .into_iter()
                .any(|ns| self.binds_only_what_is_not_read(t, id, head, ns, walk));
        }
        let entry = &self.tables[t].scopes[id as usize];
        if entry
            .declarations
            .get(head)
            .is_some_and(|all| all.iter().any(|declaration| declaration.in_namespace(ns)))
        {
            return false;
        }
        let bindings = entry.bindings.get(head).map_or(&[][..], Vec::as_slice);
        !bindings.is_empty()
            && bindings.iter().all(|binding| match binding {
                Binding::Import { written, .. } => matches!(
                    self.import_presence(t, id, head, written, ns, walk),
                    Ok(Presence::Unknown)
                ),
                Binding::Alias { .. } => false,
            })
    }

    /// What the crate root's `extern crate` items bind `head` as — the names they add to every module's
    /// scope, a crate's root being a module or type — or [`Head::Unbound`].
    fn in_extern_prelude(&self, head: &str, ns: Namespace) -> Head {
        match self.extern_prelude.get(head) {
            Some(declared) if ns != Namespace::Value => Head::Candidates {
                bound: Vec::new(),
                heads: declared.iter().map(|d| d.path().to_string()).collect(),
                declared: declared.clone(),
                through: Vec::new(),
                foreign: Vec::new(),
            },
            _ => Head::Unbound,
        }
    }

    /// What `head` names in one scope, seen from module `from` — the one module-level lookup a bare head, a `use`
    /// path's first segment, every segment of a crate-rooted path and every glob go through. Every binding of it and
    /// every declaration of it in `ns` that `from` can see are its candidates, each declaration by what it
    /// declares — a module, the crate an `extern crate` names, or an item — and a block's item other than a module
    /// or an `extern crate` is [`Head::Local`], named by no path outside its block. Only where the scope binds and
    /// declares nothing are its globs read, each asking the module it names this same question seen from the
    /// module the glob is written in, so a glob brings each declaration and each binding its own visibility lets
    /// the glob's module see; [`CrateScopes::through_globs`] reads every scope that chain of globs reaches once, so a
    /// lookup that comes back to a scope — two modules globbing each other, neither binding the name — ends there. A
    /// binding holds its name in every namespace, and where there is none a lookup in
    /// [`Namespace::Either`] is the lookup in each namespace, joined. A binding
    /// the walk is already inside is passed over, so a cycle ends; the one the walk is reading now is passed over as
    /// the import resolved without itself, which is no cut, so a `use md5x::md5x;` names the crate and is kept.
    ///
    /// An answer is kept, by [`LookupKey`], only where no cycle was cut while answering, since a cut answer holds what
    /// one walk could read rather than what the scope names, and never past the chain cap, which is a refusal of the
    /// walk rather than an answer; that is what keeps a lattice of globs from being read once per path through it.
    fn scope_lookup(
        &self,
        t: usize,
        id: u32,
        head: &str,
        ns: Namespace,
        from: &str,
        walk: &mut Walk,
    ) -> Head {
        if ns == Namespace::Either {
            return joined(
                [Namespace::Type, Namespace::Value]
                    .map(|ns| self.scope_lookup(t, id, head, ns, from, walk)),
            );
        }
        let memo = (
            t,
            id,
            head.to_string(),
            ns,
            from.to_string(),
            walk.resolving.len(),
            walk.reads_itself(&(t, id, head.to_string())),
        );
        if let Some(head) = self.looked.borrow().get(&memo) {
            return head.clone();
        }
        let cuts = walk.cuts;
        let answer = self.scope_lookup_once(t, id, head, ns, from, walk);
        if walk.cuts == cuts && !matches!(answer, Head::PastCap(_)) {
            self.looked.borrow_mut().insert(memo, answer.clone());
        }
        answer
    }

    /// [`CrateScopes::scope_lookup`] for one namespace, read without the memo: what the scope binds or declares, and
    /// where it binds and declares nothing, what its globs bring, by [`CrateScopes::through_globs`]. A scope binding the
    /// name only through imports of what is not read — `use std::fmt;` — may not hold it in `ns`, as `std::fmt` names a
    /// module and no value, so rustc reads `fmt()` from the scope's globs; its globs are read too and the answers
    /// joined, so `use crate::forbidden::*; use std::fmt;` then `fmt()` calls `crate::forbidden::fmt`, measured against
    /// rustc 1.96.0, edition 2021. Where such an import does hold the name, the glob's answer is an over-reaction,
    /// the declared bound
    /// `inline-symbol-path-confinement/an-import-of-what-is-not-read-beside-a-glob-is-read-with-the-glob-a-stated-bound`.
    /// A scope binding or declaring the name only by what a `cfg` gates has its globs read too and the answers joined,
    /// since on a build that compiles the gated item out the glob names it: `use super::*;` beside
    /// `#[cfg(test)] use crate::mock::Command;` calls the `Command` the glob brings outside tests, measured against
    /// rustc 1.96.0, edition 2021. On a build that compiles it in, the glob's answer is an over-reaction, the declared
    /// bound `inline-symbol-path-confinement/a-cfg-gated-name-beside-a-glob-is-read-with-the-glob-a-stated-bound`.
    ///
    /// A scope of a file holding a `use` tree the scanner could not read is refused rather than read, since the
    /// bindings that tree makes are missing from it, and an answer read without them could name less than rustc does:
    /// so a lookup that passes through such a file refuses, whichever query asked it, while one that never reads it is
    /// judged.
    fn scope_lookup_once(
        &self,
        t: usize,
        id: u32,
        head: &str,
        ns: Namespace,
        from: &str,
        walk: &mut Walk,
    ) -> Head {
        if let Some(refusal) = self.tables[t].refusal() {
            return Head::PastCap(format!(
                "it reads names through `{}`, whose file holds a `use` tree the scanner cannot read: {refusal}",
                self.tables[t].scopes[id as usize].module
            ));
        }
        match self.bound_here(t, id, head, ns, from, walk) {
            Some(answer)
                if !self.tables[t].scopes[id as usize].globs.is_empty()
                    && !self.answer_ends_lookup(&answer, t, id, head, ns, walk) =>
            {
                match self.through_globs(t, id, head, ns, from, walk) {
                    Head::Unbound => answer,
                    globbed => joined([answer, globbed]),
                }
            }
            Some(answer) => answer,
            None => self.through_globs(t, id, head, ns, from, walk),
        }
    }

    /// What one scope binds or declares `head` as in `ns`, seen from `from`, or `None` where it binds and declares
    /// nothing `from` can see, which leaves the name to the scope's globs.
    fn bound_here(
        &self,
        t: usize,
        id: u32,
        head: &str,
        ns: Namespace,
        from: &str,
        walk: &mut Walk,
    ) -> Option<Head> {
        let entry = &self.tables[t].scopes[id as usize];
        let visible = |visibility: &Visibility| visibility.visible_from(&entry.module, from);
        let key = (t, id, head.to_string());
        let mut bindings: Vec<&Binding> = Vec::new();
        if !walk.reads_itself(&key) && !walk.meets_cycle(&key) {
            for binding in entry.bindings.get(head).into_iter().flatten() {
                if !visible(binding.visibility()) {
                    continue;
                }
                match self.binding_holds(t, id, head, binding, ns, walk) {
                    Ok(true) => bindings.push(binding),
                    Ok(false) => {}
                    Err(refusal) => return Some(Head::PastCap(refusal)),
                }
            }
        }
        let declarations: Vec<&Declaration> = entry
            .declarations
            .get(head)
            .into_iter()
            .flatten()
            .filter(|declaration| declaration.in_namespace(ns) && visible(&declaration.visibility))
            .collect();
        if !bindings.is_empty() || !declarations.is_empty() {
            let (mut bound, mut declared, mut through): (Vec<Bound>, Vec<Declared>, Vec<String>) =
                (Vec::new(), Vec::new(), Vec::new());
            let mut names_a_local_item = false;
            for binding in bindings {
                match self.binding_names(t, id, head, binding, ns, walk) {
                    BindingNames::Paths(found, passed) => {
                        let quote = binding_quote(head, binding);
                        bound.extend(found.into_iter().map(|path| (path, quote.clone())));
                        through.extend(passed);
                    }
                    BindingNames::Local => names_a_local_item = true,
                    BindingNames::PastCap(refusal) => return Some(Head::PastCap(refusal)),
                }
            }
            for declaration in declarations {
                match (&declaration.kind, entry.kind) {
                    (DeclKind::Module(path), _) => declared.push(Declared::Module(path.clone())),
                    (DeclKind::ExternCrate(target), _) => declared.push(extern_crate_names(target)),
                    (DeclKind::Item(_), ScopeKind::Module) => {
                        declared.push(Declared::Item(format!("{}::{head}", entry.module)));
                    }
                    (DeclKind::Item(_), ScopeKind::Block) => names_a_local_item = true,
                }
            }
            if bound.is_empty() && declared.is_empty() && names_a_local_item {
                return Some(Head::Local);
            }
            declared.sort();
            declared.dedup();
            let heads = bound
                .iter()
                .map(|(path, _)| path.clone())
                .chain(declared.iter().map(|d| d.path().to_string()))
                .collect();
            return Some(Head::Candidates {
                bound,
                declared,
                through,
                heads,
                foreign: Vec::new(),
            });
        }
        None
    }

    /// What the globs of a scope that binds and declares nothing under `head` bring it as, seen from `from`: a glob's
    /// module is read seen from the module the glob is written in, which binds or declares the name or reads its own
    /// globs in turn. Every scope those globs reach, and what each binds, is read once into one graph, and the
    /// answers of the scopes that read their globs are then read to a fixed point, starting from none and in the order
    /// a scope's answer is needed by the one that reaches it, until a reading changes nothing. So two modules globbing
    /// each other end, and a graph of globs is read in time its size bounds rather than once per path through it.
    ///
    /// A glob of an item of this crate rather than a module — `use crate::a::E::*;` bringing an enum's variants — has
    /// no scope to read, so its names are those of a crate whose contents are not read. A scope's answers are kept,
    /// where no cycle was cut while reading what the scopes bind, since each is then the scope's own answer.
    fn through_globs(
        &self,
        t: usize,
        id: u32,
        head: &str,
        ns: Namespace,
        from: &str,
        walk: &mut Walk,
    ) -> Head {
        let cuts = walk.cuts;
        let depth = walk.resolving.len();
        let mut graph = GlobGraph::default();
        let start = graph.node(t, id, from);
        let mut next = 0;
        while next < graph.nodes.len() {
            let (t, id, from) = graph.nodes[next].clone();
            next += 1;
            let reading = if next - 1 == start {
                self.glob_edges(&mut graph, t, id, head, &from)
            } else {
                let itself = walk.reads_itself(&(t, id, head.to_string()));
                let memo = (t, id, head.to_string(), ns, from.clone(), depth, itself);
                let held = self.looked.borrow().get(&memo).cloned();
                match held.or_else(|| self.bound_here(t, id, head, ns, &from, walk)) {
                    Some(answer) if !self.answer_ends_lookup(&answer, t, id, head, ns, walk) => {
                        match self.glob_edges(&mut graph, t, id, head, &from) {
                            GlobRead::Globs { edges, .. } => GlobRead::Globs {
                                own: Some(answer),
                                edges,
                            },
                            refused => refused,
                        }
                    }
                    Some(answer) => GlobRead::Answered(answer),
                    None => self.glob_edges(&mut graph, t, id, head, &from),
                }
            };
            graph.reads.push(reading);
        }
        let answers = match graph.settle(head) {
            Ok(answers) => answers,
            Err(refusal) => return Head::PastCap(refusal),
        };
        if walk.cuts == cuts {
            let mut looked = self.looked.borrow_mut();
            for (n, (t, id, from)) in graph.nodes.iter().enumerate() {
                if n != start
                    && matches!(graph.reads[n], GlobRead::Globs { .. })
                    && !matches!(answers[n], Head::PastCap(_))
                {
                    let itself = walk.reads_itself(&(*t, *id, head.to_string()));
                    looked.insert(
                        (*t, *id, head.to_string(), ns, from.clone(), depth, itself),
                        answers[n].clone(),
                    );
                }
            }
        }
        answers[start].clone()
    }

    /// The globs of one scope that binds and declares nothing under `head`, seen from `from`, as edges of `graph`: a
    /// glob of a module of the unit is an edge to each of that module's scopes, seen from this scope's module, and any
    /// other glob names what a crate whose contents are not read brings.
    fn glob_edges(
        &self,
        graph: &mut GlobGraph,
        t: usize,
        id: u32,
        head: &str,
        from: &str,
    ) -> GlobRead {
        let entry = &self.tables[t].scopes[id as usize];
        let mut edges = Vec::new();
        for (i, glob) in entry.globs.iter().enumerate() {
            if !glob.visibility.visible_from(&entry.module, from) {
                continue;
            }
            let targets = match self.glob_targets(t, id, i) {
                Ok(targets) => targets,
                Err(refusal) => return GlobRead::Answered(Head::PastCap(refusal)),
            };
            for target in targets {
                let scopes = path_within(&target, "crate")
                    .then(|| self.modules.get(&target))
                    .flatten();
                if let Some(scopes) = scopes {
                    let into = scopes
                        .iter()
                        .map(|&(t, s)| graph.node(t, s, &entry.module))
                        .collect();
                    edges.push(GlobEdge::Into {
                        quote: format!("use {}::*", glob.written),
                        target,
                        into,
                    });
                } else {
                    edges.push(GlobEdge::Foreign(format!("{target}::{head}")));
                }
            }
        }
        GlobRead::Globs { own: None, edges }
    }

    /// [`CrateScopes::scope_lookup`] in every scope of `module`, seen from `from`, their answers joined.
    fn lookup_in_module(
        &self,
        module: &str,
        head: &str,
        ns: Namespace,
        from: &str,
        walk: &mut Walk,
    ) -> Head {
        let scopes = match self.blocks.get(module) {
            Some(block) => std::slice::from_ref(block),
            None => self.modules.get(module).map_or(&[][..], Vec::as_slice),
        };
        joined(
            scopes
                .iter()
                .map(|&(t, s)| self.scope_lookup(t, s, head, ns, from, walk))
                .collect::<Vec<_>>(),
        )
    }

    /// Whether `binding` holds `name` in `ns`: a `type` alias names a type, and so does a `{self}` leaf, which imports
    /// its group's module alone; any other import holds each namespace its target
    /// names something in, and every namespace where its target names nothing in either — an item a macro generates,
    /// whose namespaces are not read — so only a target known in the other namespace and absent in this one leaves
    /// this one to the scope's globs. The one namespace filter a declaration also answers, [`Namespace`], read for a
    /// binding from what its target is.
    fn binding_holds(
        &self,
        t: usize,
        scope: u32,
        name: &str,
        binding: &Binding,
        ns: Namespace,
        walk: &mut Walk,
    ) -> Result<bool, String> {
        let Binding::Import {
            written,
            module_only,
            ..
        } = binding
        else {
            return Ok(ns != Namespace::Value);
        };
        if *module_only {
            return Ok(ns != Namespace::Value);
        }
        if self.import_presence(t, scope, name, written, ns, walk)? != Presence::Absent {
            return Ok(true);
        }
        Ok(self.import_presence(t, scope, name, written, ns.other(), walk)? == Presence::Absent)
    }

    /// Whether the import `written`, binding `name` in `scope`, names something in `ns`, read inside the walk so a
    /// cycle of imports ends as the resolution it is part of does. It is remembered only where no cycle was ended while
    /// it was read, since what a read past a cut cycle finds depends on the walk it was read in.
    fn import_presence(
        &self,
        t: usize,
        scope: u32,
        name: &str,
        written: &str,
        ns: Namespace,
        walk: &mut Walk,
    ) -> Result<Presence, String> {
        let key = (t, scope, name.to_string(), written.to_string(), ns);
        if let Some(presence) = self.presence.borrow().get(&key) {
            return Ok(*presence);
        }
        let module = &self.tables[t].scopes[scope as usize].module;
        let cuts = walk.cuts;
        walk.enter(
            (t, scope, name.to_string()),
            format!("use {written}"),
            module,
        )?;
        let presence = match self.name_raw(
            t,
            scope,
            written,
            PathSite::Use,
            ns,
            &mut Vec::new(),
            walk,
            false,
        ) {
            Named::Paths(paths) => paths.iter().try_fold(Presence::Absent, |most, path| {
                let presence = if path_within(path, "crate") {
                    self.denote_in(path, ns, walk)
                        .map(|denoted| denoted.presence)
                } else {
                    Ok(Presence::Unknown)
                };
                presence.map(|presence| most.max(presence))
            }),
            Named::External(_) | Named::Unbound { .. } => Ok(Presence::Unknown),
            Named::Local => Ok(Presence::Known),
            Named::Invalid => Ok(Presence::Absent),
            Named::PastCap(refusal) => Err(refusal),
        };
        walk.leave();
        if let (Ok(presence), true) = (&presence, walk.cuts == cuts) {
            self.presence.borrow_mut().insert(key, *presence);
        }
        presence
    }

    /// The paths one binding names in `ns`, read from its own scope as a `use` path or as a `type` alias's
    /// target. A head no scope binds names the extern-prelude crate of that name, for either.
    pub(super) fn binding_names(
        &self,
        t: usize,
        scope: u32,
        name: &str,
        binding: &Binding,
        ns: Namespace,
        walk: &mut Walk,
    ) -> BindingNames {
        let (written, site, ns, quote) = match binding {
            Binding::Import { written, .. } => {
                (written, PathSite::Use, ns, format!("use {written}"))
            }
            Binding::Alias { written, .. } => (
                written,
                PathSite::Expr,
                Namespace::Type,
                format!("type {name} = {written}"),
            ),
        };
        let module = &self.tables[t].scopes[scope as usize].module;
        if let Err(refusal) = walk.enter((t, scope, name.to_string()), quote, module) {
            return BindingNames::PastCap(refusal);
        }
        let mut through = Vec::new();
        let named = self.name_raw(t, scope, written, site, ns, &mut through, walk, false);
        walk.leave();
        match named {
            Named::Paths(paths) => BindingNames::Paths(paths, through),
            Named::External(path) => BindingNames::Paths(vec![path], through),
            Named::Unbound { head, rest } => {
                BindingNames::Paths(vec![with_rest(head, &rest)], through)
            }
            Named::Local => BindingNames::Local,
            Named::Invalid => BindingNames::Paths(Vec::new(), through),
            Named::PastCap(refusal) => BindingNames::PastCap(refusal),
        }
    }

    /// Every path glob `i` of `scope` names, read from that scope as a `use` path, each crate-rooted one carried
    /// through what it names.
    ///
    /// A glob's own path may be read through another glob of the unit, and that one through the first, so every glob
    /// of the unit is read together, to a fixed point, in passes starting from none. A glob's own path is never read
    /// through the glob itself, which names nothing while it is read, as rustc never resolves a glob through what it
    /// imports. Each glob is read with every remembered answer forgotten first, so it is read from the current readings of
    /// the others — those the globs before it in the same pass have just been read as — and never through itself; the
    /// passes read the unit's globs in turn forwards and backwards, so a chain of globs each read through one written
    /// before or after it settles within a few passes. A pass that changes nothing ends the reading, and every answer
    /// it read is from the settled readings.
    /// Passes are bounded by [`MAX_RESOLUTION_CHAIN`] or twice the unit's globs plus two, whichever is more, past which
    /// every glob is refused, since a source whose globs name each other's names without settling is one the scanner
    /// cannot judge.
    pub(super) fn glob_targets(
        &self,
        t: usize,
        scope: u32,
        i: usize,
    ) -> Result<Vec<String>, String> {
        let key = (t, scope, i);
        if let Some(all) = self.globs.borrow().as_ref() {
            return all.get(&key).cloned().unwrap_or_else(|| Ok(Vec::new()));
        }
        if self.reading_globs.borrow().is_some() {
            if *self.reading_glob.borrow() == Some(key) {
                self.read_itself.set(true);
                return Ok(Vec::new());
            }
            return self.last_read(key);
        }
        let every: Vec<GlobKey> = self
            .tables
            .iter()
            .enumerate()
            .flat_map(|(t, table)| {
                table
                    .scopes
                    .iter()
                    .enumerate()
                    .flat_map(move |(id, entry)| {
                        let id = u32::try_from(id).expect("scope table exceeds u32");
                        (0..entry.globs.len()).map(move |i| (t, id, i))
                    })
            })
            .collect();
        *self.reading_globs.borrow_mut() = Some(GlobTargets::new());
        let passes = MAX_RESOLUTION_CHAIN.max(2 * (every.len() + 1));
        let mut settled = false;
        for pass in 0..passes {
            let mut changed = false;
            let order: Box<dyn Iterator<Item = &GlobKey>> = if pass % 2 == 0 {
                Box::new(every.iter())
            } else {
                Box::new(every.iter().rev())
            };
            for &glob in order {
                changed |= self.read_in_pass(glob);
            }
            if !changed {
                settled = true;
                break;
            }
        }
        let read = self
            .reading_globs
            .borrow_mut()
            .take()
            .expect("the glob readings are set before the passes");
        let all: GlobTargets = if settled {
            read
        } else {
            let refusal = format!(
                "cannot judge a compilation unit whose globs do not settle on what they name within {passes} passes"
            );
            every
                .iter()
                .map(|&key| (key, Err(refusal.clone())))
                .collect()
        };
        self.forget_readings();
        let found = all.get(&key).cloned().unwrap_or_else(|| Ok(Vec::new()));
        *self.globs.borrow_mut() = Some(all);
        found
    }

    /// What glob `key` was last read as while the fixed point is iterated: nothing before its first reading.
    fn last_read(&self, key: GlobKey) -> Result<Vec<String>, String> {
        self.reading_globs
            .borrow()
            .as_ref()
            .and_then(|all| all.get(&key).cloned())
            .unwrap_or_else(|| Ok(Vec::new()))
    }

    /// Read glob `key` once in the current pass, with the glob itself naming nothing while its own path is read, and
    /// keep what it is read as; whether that changed. Every answer remembered before the read is forgotten first, since
    /// one remembered while this glob still answered its last reading would read the glob through itself; and answers
    /// read while the glob named nothing are forgotten after it, since they hold less than the glob brings to any
    /// other reader.
    fn read_in_pass(&self, key: GlobKey) -> bool {
        self.forget_readings();
        *self.reading_glob.borrow_mut() = Some(key);
        self.read_itself.set(false);
        let reading = self.read_glob(key.0, key.1, key.2);
        *self.reading_glob.borrow_mut() = None;
        if self.read_itself.get() {
            self.forget_readings();
        }
        if reading == self.last_read(key) {
            return false;
        }
        if let Some(all) = self.reading_globs.borrow_mut().as_mut() {
            all.insert(key, reading);
        }
        true
    }

    /// Drop every answer read while the globs' targets were still being read, since each may hold a reading of them
    /// the fixed point has since changed.
    fn forget_readings(&self) {
        self.named.borrow_mut().clear();
        self.denoted.borrow_mut().clear();
        self.looked.borrow_mut().clear();
        self.presence.borrow_mut().clear();
    }

    /// One reading of glob `i` of `scope`, each glob its path passes answered from the reading in progress.
    fn read_glob(&self, t: usize, scope: u32, i: usize) -> Result<Vec<String>, String> {
        let entry = &self.tables[t].scopes[scope as usize];
        let written = &entry.globs[i].written;
        let mut walk = Walk::default();
        walk.enter(
            (t, scope, format!("*{i}")),
            format!("use {written}::*"),
            &entry.module,
        )?;
        let named = self.name_raw(
            t,
            scope,
            written,
            PathSite::Use,
            Namespace::Type,
            &mut Vec::new(),
            &mut walk,
            false,
        );
        match named {
            Named::Paths(paths) => paths.iter().try_fold(Vec::new(), |mut found, path| {
                found.extend(self.denote_in(path, Namespace::Type, &mut walk)?.paths);
                Ok(found)
            }),
            Named::External(path) => Ok(vec![path]),
            Named::Unbound { head, rest } => Ok(vec![with_rest(head, &rest)]),
            Named::Local | Named::Invalid => Ok(Vec::new()),
            Named::PastCap(refusal) => Err(refusal),
        }
    }

    /// [`CrateScopes::denote`] inside a walk. The path is read segment by segment from the crate root, each
    /// segment looked up — in the type namespace, the last in `ns` — in the module the segments before it
    /// reached, by [`CrateScopes::lookup_in_module`], the lookup a head written in that module gets. A declared
    /// module, or the crate an `extern crate self` names, is walked into; a declared item, or the root of a crate
    /// whose contents are not read, names itself with the rest of the path after it; a segment the module binds —
    /// through an import, a `type` alias or a glob — is replaced by every path the binding names and read again
    /// from the root; and a segment only a glob of a crate whose contents are not read can bring names that
    /// glob's path. A segment the module neither binds nor declares names itself. A branch that replaces more
    /// than [`MAX_RESOLUTION_CHAIN`] segments is refused, quoting the binding it started from.
    ///
    /// A file-form module a block declares is its own file's module, named through the block's readable form rather
    /// than a block scope, so a readable block segment is read together with the module name after it.
    fn denote_in(&self, path: &str, ns: Namespace, walk: &mut Walk) -> Result<Denoted, String> {
        let mut found = Vec::new();
        let mut presence = Presence::Absent;
        let mut seen: BTreeSet<(String, Vec<String>)> = BTreeSet::new();
        let mut work: Vec<(String, Vec<String>, Vec<Link>)> = Vec::new();
        match path
            .split("::")
            .map(str::to_string)
            .collect::<Vec<_>>()
            .split_first()
        {
            Some((root, rest)) if root == "crate" => {
                work.push((root.clone(), rest.to_vec(), Vec::new()));
            }
            _ => {
                return Ok(Denoted {
                    paths: vec![path.to_string()],
                    presence: Presence::Unknown,
                });
            }
        }
        while let Some((module, rest, chain)) = work.pop() {
            if !seen.insert((module.clone(), rest.clone())) {
                continue;
            }
            let Some((segment, after)) = rest.split_first() else {
                found.push(module);
                presence = presence.max(Presence::Known);
                continue;
            };
            let here = format!("{module}::{segment}");
            if is_block_segment(segment) {
                if let Some((name, beyond)) = after.split_first() {
                    let file_module = format!("{here}::{name}");
                    if !self.blocks.contains_key(&here) && self.modules.contains_key(&file_module) {
                        work.push((file_module, beyond.to_vec(), chain));
                        continue;
                    }
                }
                work.push((here, after.to_vec(), chain));
                continue;
            }
            let segment_ns = if after.is_empty() {
                ns
            } else {
                Namespace::Type
            };
            match self.lookup_in_module(&module, segment, segment_ns, &module, walk) {
                Head::Candidates {
                    bound,
                    declared,
                    through,
                    foreign,
                    ..
                } => {
                    found.extend(through.into_iter().map(|path| with_rest(path, after)));
                    if !foreign.is_empty() {
                        found.extend(foreign.into_iter().map(|path| with_rest(path, after)));
                        presence = presence.max(Presence::Unknown);
                    }
                    for declaration in declared {
                        match declaration {
                            Declared::Module(inner) if !after.is_empty() => {
                                work.push((inner, after.to_vec(), chain.clone()));
                            }
                            Declared::Module(path) | Declared::Item(path) => {
                                found.push(with_rest(path, after));
                                presence = presence.max(Presence::Known);
                            }
                        }
                    }
                    if bound.is_empty() {
                        continue;
                    }
                    found.push(with_rest(here.clone(), after));
                    if chain
                        .iter()
                        .any(|(m, name, _)| *m == module && name == segment)
                    {
                        walk.cuts += 1;
                        presence = presence.max(Presence::Unknown);
                        continue;
                    }
                    if chain.len() >= MAX_RESOLUTION_CHAIN {
                        return Err(chain_refusal(&chain[0].2, &chain[0].0));
                    }
                    for (target, quote) in bound {
                        let link = (module.clone(), segment.clone(), quote);
                        let target: Vec<String> = with_rest(target, after)
                            .split("::")
                            .map(str::to_string)
                            .collect();
                        match target.split_first() {
                            Some((root, rest)) if root == "crate" => {
                                let mut chain = chain.clone();
                                chain.push(link);
                                work.push((root.clone(), rest.to_vec(), chain));
                            }
                            _ => {
                                found.push(target.join("::"));
                                presence = presence.max(Presence::Unknown);
                            }
                        }
                    }
                }
                Head::Foreign(paths) => {
                    found.push(with_rest(here, after));
                    found.extend(paths.into_iter().map(|path| with_rest(path, after)));
                    presence = presence.max(Presence::Unknown);
                }
                Head::PastCap(refusal) => return Err(refusal),
                Head::Local => {
                    found.push(with_rest(here, after));
                    presence = presence.max(Presence::Known);
                }
                Head::Unbound => {
                    found.push(with_rest(here, after));
                    if !after.is_empty() || self.may_generate_items(&module) {
                        presence = presence.max(Presence::Unknown);
                    }
                }
            }
        }
        Ok(Denoted {
            paths: sorted_unique(found),
            presence,
        })
    }

    /// Whether a scope of `module` holds a macro invocation where an item can stand, which may generate an item the
    /// scanner does not read: a name its module declares in neither namespace is then unknown there, not absent.
    fn may_generate_items(&self, module: &str) -> bool {
        self.modules
            .get(module)
            .into_iter()
            .flatten()
            .any(|&(t, s)| self.tables[t].scopes[s as usize].macro_items)
    }
}

/// How `binding`, binding `name`, is written, for a refusal of a chain through it to quote.
fn binding_quote(name: &str, binding: &Binding) -> String {
    match binding {
        Binding::Import { written, .. } => format!("use {written}"),
        Binding::Alias { written, .. } => format!("type {name} = {written}"),
    }
}

/// One link of a chain [`CrateScopes::denote`] follows: a module, the name in it whose binding replaced a segment,
/// and how that binding is written. A branch that meets the same module and name twice is a cycle.
type Link = (String, String, String);

/// Several answers to one lookup, joined: the least refusal if any refuses, whatever order the answers come in; else
/// every candidate any gives; else a block-local item if any names one; else every path a foreign glob could bring;
/// else unbound.
fn joined(heads: impl IntoIterator<Item = Head>) -> Head {
    let (mut bound, mut declared, mut through, mut alone, mut foreign, mut local) = (
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        false,
    );
    let mut refused: Option<String> = None;
    for head in heads {
        match head {
            Head::PastCap(refusal) => refused = least(refused, refusal),
            Head::Candidates {
                bound: b,
                declared: d,
                through: p,
                heads: g,
                foreign: f,
            } => {
                bound.extend(b);
                declared.extend(d);
                through.extend(p);
                alone.extend(g);
                foreign.extend(f);
            }
            Head::Foreign(paths) => foreign.extend(paths),
            Head::Local => local = true,
            Head::Unbound => {}
        }
    }
    if let Some(refusal) = refused {
        Head::PastCap(refusal)
    } else if !bound.is_empty() || !declared.is_empty() {
        declared.sort();
        declared.dedup();
        Head::Candidates {
            bound: sorted_unique(bound),
            declared,
            through: sorted_unique(through),
            heads: sorted_unique(alone),
            foreign: sorted_unique(foreign),
        }
    } else if local {
        Head::Local
    } else if !foreign.is_empty() {
        Head::Foreign(sorted_unique(foreign))
    } else {
        Head::Unbound
    }
}

/// The lesser of a refusal already held and another: the one refusal a fold over several reports, independent of the
/// order they were met in.
pub(super) fn least(held: Option<String>, refusal: String) -> Option<String> {
    Some(match held {
        Some(held) if held <= refusal => held,
        _ => refusal,
    })
}

fn sorted_unique<T: Ord>(paths: impl IntoIterator<Item = T>) -> Vec<T> {
    let mut paths: Vec<T> = paths.into_iter().collect();
    paths.sort();
    paths.dedup();
    paths
}

#[cfg(test)]
mod tests {
    use super::super::token_tree::{Edition, TokenTree};
    use super::*;

    /// What `head` at `at` in `source`, read as `crate::core` beneath a crate root declaring it, names.
    fn resolved(source: &str, at: &str, head: &str, ns: Namespace) -> Option<String> {
        let tree = TokenTree::lex(source, Edition::Rust2018);
        let offset = source.find(at).unwrap();
        let token = (0..tree.len()).find(|&i| tree.start(i) >= offset).unwrap();
        let table = ScopeTable::build(&tree, "crate::core", 0);
        let root = TokenTree::lex("pub mod core;\n", Edition::Rust2018);
        let root = ScopeTable::build(&root, "crate", 1);
        let scopes = CrateScopes::new(vec![table, root], Edition::Rust2018);
        match scopes.name(0, scopes.table(0).scope_at(token), head, PathSite::Expr, ns) {
            Named::Paths(paths) => Some(paths.join(" | ")),
            Named::Local => Some("<local>".to_string()),
            _ => None,
        }
    }

    #[test]
    fn written_root_classifies_each_root_once() {
        let bare = |head: &str, rest: &[&str]| WrittenRoot::Bare {
            head: head.to_string(),
            rest: rest.iter().map(|s| s.to_string()).collect(),
        };
        let from_root = |head: &str, rest: &[&str]| WrittenRoot::FromCrateRoot {
            head: head.to_string(),
            rest: rest.iter().map(|s| s.to_string()).collect(),
        };
        let extern_ = |head: &str, rest: &[&str]| WrittenRoot::Extern {
            head: head.to_string(),
            rest: rest.iter().map(|s| s.to_string()).collect(),
        };
        let crate_ = |path: &str| WrittenRoot::Crate(path.to_string());
        for (written, module, site, edition, expected) in [
            (
                "std::time",
                "crate::core",
                PathSite::Expr,
                Edition::Rust2018,
                bare("std", &["time"]),
            ),
            (
                "::core::time",
                "crate::core",
                PathSite::Expr,
                Edition::Rust2018,
                extern_("core", &["time"]),
            ),
            (
                "::alloc::vec",
                "crate::core",
                PathSite::Use,
                Edition::Rust2015,
                from_root("alloc", &["vec"]),
            ),
            (
                "crate::a::X",
                "crate::core",
                PathSite::Expr,
                Edition::Rust2018,
                crate_("crate::a::X"),
            ),
            (
                "super::X",
                "crate::core::t",
                PathSite::Use,
                Edition::Rust2018,
                crate_("crate::core::X"),
            ),
            (
                "self::X",
                "crate::core",
                PathSite::Expr,
                Edition::Rust2018,
                crate_("crate::core::X"),
            ),
            (
                "super::super::X",
                "crate",
                PathSite::Expr,
                Edition::Rust2018,
                WrittenRoot::Invalid,
            ),
            (
                "::self::X",
                "crate::core",
                PathSite::Expr,
                Edition::Rust2018,
                WrittenRoot::Invalid,
            ),
            (
                "::super::X",
                "crate::core",
                PathSite::Use,
                Edition::Rust2015,
                WrittenRoot::Invalid,
            ),
            (
                "::md5x::f",
                "crate::core",
                PathSite::Expr,
                Edition::Rust2018,
                extern_("md5x", &["f"]),
            ),
            (
                "::md5x::f",
                "crate::core",
                PathSite::Expr,
                Edition::Rust2015,
                from_root("md5x", &["f"]),
            ),
            (
                "clock::now",
                "crate::core",
                PathSite::Use,
                Edition::Rust2015,
                from_root("clock", &["now"]),
            ),
            (
                "clock::now",
                "crate::core",
                PathSite::Expr,
                Edition::Rust2015,
                bare("clock", &["now"]),
            ),
            (
                "clock::now",
                "crate::core",
                PathSite::Use,
                Edition::Rust2018,
                bare("clock", &["now"]),
            ),
            (
                "r#try::f",
                "crate::core",
                PathSite::Expr,
                Edition::Rust2018,
                bare("try", &["f"]),
            ),
            (
                "",
                "crate::core",
                PathSite::Expr,
                Edition::Rust2018,
                WrittenRoot::Invalid,
            ),
        ] {
            assert_eq!(
                written_root(written, module, site, edition),
                expected,
                "`{written}` at {module}, {edition:?}"
            );
        }
    }

    #[test]
    fn a_block_use_binds_its_whole_block_and_nothing_outside() {
        let source = "mod a { pub struct X; }\nmod b { pub struct X; }\nuse crate::core::a::X;\nfn h() { X::fa(); }\nfn g() { X::early(); use crate::core::b::X; X::fb(); }\nfn k() { X::late(); }\n";
        for (at, expected) in [
            ("X::fa", "crate::core::a::X"),
            ("X::early", "crate::core::b::X"),
            ("X::fb", "crate::core::b::X"),
            ("X::late", "crate::core::a::X"),
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
