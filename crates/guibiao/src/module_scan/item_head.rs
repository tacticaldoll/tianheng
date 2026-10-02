//! The item-header grammar, read once over the [`TokenTree`]: where an item starts, the qualifiers before its
//! keyword, its visibility, its name and its body — and, read backward from a `{` to the header that owns it, what
//! kind of group the brace opens. One qualifier list and one header walk answer every reader that asks where an item
//! starts, what a brace opens, or who may name what an item declares.

use super::path_vocab::{
    block_label, canonical_segment, fold_canonical_segments, path_within, resolve_self_super,
    strip_block_segments,
};
use super::token_tree::{AngleSlot, Delimiter, Kind, Node, TokenTree};

/// The words that may stand before an item's keyword without being one: `pub` (with its optional `(…)`) is read
/// separately, and `extern` takes an optional ABI literal. Each qualifies only where an item keyword follows it.
const ITEM_QUALIFIERS: [&str; 7] = [
    "unsafe", "safe", "async", "const", "extern", "default", "auto",
];

/// Who may name an item or an import from outside the module that declares it, as its `pub` qualifier says.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum Visibility {
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
    /// Whether code in module `from` can name something `owner` declares with this visibility. A module declared in
    /// a block has the block's module as its parent, which is what `super` names from it.
    pub(super) fn visible_from(&self, owner: &str, from: &str) -> bool {
        match self {
            Visibility::Public => true,
            Visibility::Private => path_within(from, owner),
            Visibility::Super => path_within(
                from,
                strip_block_segments(
                    owner
                        .rsplit_once("::")
                        .map_or("crate", |(parent, _)| parent),
                ),
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

/// The keyword an item header is named by.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ItemKeyword {
    /// `mod`: a named one with a brace body opens an inline module body.
    Mod,
    /// `struct`: the one keyword whose header records whether it also declares a value.
    Struct,
    /// `enum`: a type with a body of variants.
    Enum,
    /// `union`, a contextual word read as an item keyword only when a word follows it.
    Union,
    /// `trait`: an item whose body holds associated items.
    Trait,
    /// `type`: an alias, whose header has no brace body.
    Type,
    /// `fn`: its brace body is a block, not a member body.
    Fn,
    /// `const` followed by a name or `_` and a `:` — an item, not a `const fn` qualifier or a `const`
    /// block.
    Const,
    /// `static`, whose name follows a `mut` where one is written.
    Static,
    /// `impl`: unnamed, with a member body.
    Impl,
    /// `use`: unnamed, and every brace in its tree is a use group.
    Use,
    /// `extern crate`, named by the token after `crate`, which may be `self`.
    ExternCrate,
    /// `extern`, optionally with an ABI literal, directly before a `{`: unnamed.
    ExternBlock,
    /// `macro_rules!`, named by the word after the `!`.
    MacroRules,
}

/// One item header: where it starts (its first qualifier), its keyword, the token naming it, its visibility, its
/// brace body and — for a `struct` — whether it is written as a tuple or a unit, which declares a value as well as a
/// type.
#[derive(Clone, Debug)]
pub(super) struct ItemHead {
    /// The header's first token: its first qualifier, or its keyword.
    pub start: usize,
    /// The kind of item the header declares.
    pub keyword: ItemKeyword,
    /// The token of the keyword itself, after every qualifier.
    pub keyword_at: usize,
    /// The token naming the item: `None` for an `impl`, a `use` or an extern block, and wherever the
    /// token in the name's position is not a word.
    pub name: Option<usize>,
    /// What the header's `pub` qualifier gives, or `Visibility::Private` when it has none.
    pub visibility: Visibility,
    /// The `{` of the item's brace body: read only for `mod`, `struct`, `enum`, `union`, `trait`,
    /// `impl`, `fn` and extern-block headers, and `None` where the header has no brace body.
    pub body: Option<usize>,
    /// Whether a `struct` is a tuple or unit struct, declaring a value as well as a type; `false` for
    /// every other keyword.
    pub value_struct: bool,
}

/// What a group opens, read from the header before it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum GroupKind {
    /// An inline `mod name { … }`, by its canonical name.
    ModuleBody(String),
    /// A `fn` body and every other brace group no other kind claims: blocks, `unsafe`/`async`/`const` blocks,
    /// closure and arm bodies, initializers, struct literals.
    Block,
    /// The body of an `impl`, `trait`, `struct`, `enum` or `union`: its members bind no bare head.
    MemberBody,
    /// An `extern` block: its items belong to the enclosing scope.
    ExternBlock,
    /// A `cfg_if!` invocation's group: the arms inside it hold items of the enclosing scope.
    CfgIf,
    /// One arm of a `cfg_if!`: a brace group after an attribute or `else`.
    CfgArm,
    /// A brace group inside a `use` statement's tree.
    UseGroup,
    /// Any other macro invocation's or definition's group: declarations inside are not read, calls are.
    MacroBody,
    /// A parenthesized or bracketed group that is no macro's.
    Other,
}

/// Whether the attribute node is an outer attribute, `#[…]`, which applies to the item after it — not an inner one,
/// `#![…]`, which applies to the item it is written in.
pub(super) fn is_outer_attribute(node: Node) -> bool {
    matches!(node, Node::Attribute { hash, open, .. } if open == hash + 1)
}

/// The path an attribute or an applied meta written at tokens `start..end` opens with: its name where the path is
/// one segment — a built-in attribute, `r#` included, is always one — and the index just past the path. A path of
/// several segments names no built-in: measured on rustc 1.96.0, a proc-macro attribute named `cfg` is refused with
/// "name `cfg` is reserved in attribute namespace", so `foo::cfg` is no `cfg` and its arguments are read.
pub(super) fn attribute_path<'t>(
    tree: &'t TokenTree<'_>,
    start: usize,
    end: usize,
) -> (Option<&'t str>, usize) {
    let mut k = start;
    let mut segments = Vec::new();
    while k < end && (tree.is_word(k) || tree.kind(k) == Kind::Keyword || tree.is(k, "::")) {
        if !tree.is(k, "::") {
            segments.push(tree.text(k));
        }
        k += 1;
    }
    let single = (segments.len() == 1 && !tree.is(start, "::")).then(|| segments[0]);
    (single, k)
}

/// Whether a `cfg` is written among the outer attributes standing before the item whose header starts at `start`,
/// directly or applied through a `cfg_attr`, nested ones included — so a build may compile the item out. The
/// predicate is never evaluated.
pub(super) fn cfg_written_before(tree: &TokenTree, start: usize) -> bool {
    let mut metas = Vec::new();
    let mut k = start;
    while let Some(node @ Node::Attribute { open, close, .. }) = tree.node_before(k) {
        if !is_outer_attribute(node) {
            break;
        }
        metas.push((open + 1, close));
        k = node.first();
    }
    while let Some((start, end)) = metas.pop() {
        match attribute_path(tree, start, end) {
            (Some("cfg"), _) => return true,
            (Some("cfg_attr"), after) if tree.kind(after) == Kind::Open(Delimiter::Parenthesis) => {
                metas.extend(cfg_attr_metas(tree, after));
            }
            _ => {}
        }
    }
    false
}

/// Whether the item whose header starts at `start` is one a configuration may leave out where another is compiled in:
/// a `cfg` is written on it ([`cfg_written_before`]), or it stands directly in an arm of a `cfg_if!`, whose arms are
/// exclusive. What encloses its scope is not asked, since a scope's lookups are made only where the scope is compiled.
pub(super) fn may_be_cfg_gated(tree: &TokenTree, start: usize) -> bool {
    cfg_written_before(tree, start)
        || tree.enclosing(start).is_some_and(|open| {
            tree.kind(open) == Kind::Open(Delimiter::Brace)
                && tree
                    .enclosing(open)
                    .is_some_and(|outer| macro_group_kind(tree, outer) == Some(GroupKind::CfgIf))
                && (matches!(tree.node_before(open), Some(Node::Attribute { .. }))
                    || open.checked_sub(1).is_some_and(|e| tree.is(e, "else")))
        })
}

/// The applied metas of the `cfg_attr(…)` whose `(` is at `open`: every comma-separated meta after its predicate, as
/// token ranges. Each is an attribute in its own right, read by [`attribute_path`].
pub(super) fn cfg_attr_metas(tree: &TokenTree, open: usize) -> Vec<(usize, usize)> {
    let close = tree.partner(open);
    let mut metas = Vec::new();
    let mut k = open + 1;
    let mut meta = k;
    let mut past_predicate = false;
    while k <= close {
        if k == close || tree.is(k, ",") {
            if past_predicate && meta < k {
                metas.push((meta, k));
            }
            past_predicate = true;
            meta = k + 1;
            k += 1;
            continue;
        }
        k = tree.node_at(k).last() + 1;
    }
    metas
}

/// The name of the macro whose invocation or definition group opens at `open`, if one does.
fn macro_name_of(tree: &TokenTree, open: usize) -> Option<usize> {
    if let Some(name) = open
        .checked_sub(2)
        .filter(|&n| tree.is(n + 1, "!") && tree.is_word(n))
    {
        return Some(name);
    }
    let m = open.checked_sub(3)?;
    tree.opens_a_macro_rules(m).then_some(m)
}

/// The kind of the macro group opening at `open` — a `cfg_if!`'s, whose arms hold items, or any other macro's —
/// or `None` where no macro opens it. The one place a `cfg_if!` is told apart by its name.
pub(super) fn macro_group_kind(tree: &TokenTree, open: usize) -> Option<GroupKind> {
    macro_name_of(tree, open).map(|name| {
        if tree.text(name) == "cfg_if" {
            GroupKind::CfgIf
        } else {
            GroupKind::MacroBody
        }
    })
}

/// Whether `open` is a macro's group other than `cfg_if!`'s, whose contents declare nothing the scanner reads.
fn opens_an_opaque_macro_body(tree: &TokenTree, open: usize) -> bool {
    macro_group_kind(tree, open) == Some(GroupKind::MacroBody)
}

/// Whether token `i` stands inside a macro's group other than `cfg_if!`'s, at any depth.
pub(super) fn in_macro_body(tree: &TokenTree, i: usize) -> bool {
    let mut at = tree.enclosing(i);
    while let Some(open) = at {
        if opens_an_opaque_macro_body(tree, open) {
            return true;
        }
        at = tree.enclosing(open);
    }
    false
}

/// Whether token `i` stands inside an attribute's brackets, at any depth: `#[…]` or `#![…]`. What an attribute holds
/// is its macro's input, not source the item it is written on declares.
pub(super) fn in_attribute(tree: &TokenTree, i: usize) -> bool {
    let mut at = tree.enclosing(i);
    while let Some(open) = at {
        if tree.kind(open) == Kind::Open(Delimiter::Bracket)
            && open.checked_sub(1).is_some_and(|before| {
                tree.is(before, "#")
                    || (tree.is(before, "!")
                        && before.checked_sub(1).is_some_and(|hash| tree.is(hash, "#")))
            })
        {
            return true;
        }
        at = tree.enclosing(open);
    }
    false
}

/// One `mod` a block declares, and the segment its module's path carries through that block.
pub(super) struct BlockModule {
    /// The `mod` item's header.
    pub head: ItemHead,
    /// The token naming the module; a `mod` header without a name is never a block module.
    pub name_at: usize,
    /// The `{` of the module body the block stands in, or `None` at a file's top level: the module whose blocks
    /// number this one among the modules of its name.
    pub owner: Option<usize>,
    /// The block segment its module's path carries: `{block}` for the first module of its name under
    /// `owner`, `{block N}` for the Nth.
    pub label: String,
}

/// Every `mod` a block declares, inline or file-form, in source order, each with its [`block_label`]: the one reading
/// of which `mod` stands in a block and what its module is named, which the reachability walk and the scope table both
/// take, so a file a block's module reads and the scope a path through it names carry one path. A `mod` stands in a
/// block where a group other than a `cfg_if!` or its arm stands between it and the first module body enclosing it, and
/// is numbered among the modules of its name that module's blocks declare. A `mod` inside a macro's group is none.
pub(super) fn block_modules(tree: &TokenTree) -> Vec<BlockModule> {
    let mut counts: std::collections::BTreeMap<(Option<usize>, String), usize> = Default::default();
    let mut found = Vec::new();
    for i in 0..tree.len() {
        if tree.kind(i) != Kind::Keyword || tree.text(i) != "mod" || in_macro_body(tree, i) {
            continue;
        }
        let mut in_block = false;
        let mut owner = None;
        let mut at = tree.enclosing(i);
        while let Some(open) = at {
            match classify_group(tree, open, None) {
                GroupKind::ModuleBody(_) => {
                    owner = Some(open);
                    break;
                }
                GroupKind::CfgIf | GroupKind::CfgArm => {}
                _ => in_block = true,
            }
            at = tree.enclosing(open);
        }
        if !in_block {
            continue;
        }
        let Some((head, name_at)) =
            item_at_keyword(tree, i).and_then(|head| head.name.map(|name| (head, name)))
        else {
            continue;
        };
        let count = counts
            .entry((owner, canonical_segment(tree.text(name_at)).to_string()))
            .or_default();
        *count += 1;
        found.push(BlockModule {
            head,
            name_at,
            owner,
            label: block_label(*count),
        });
    }
    found
}

/// Whether the brace group at `open` is an arm of the `cfg_if!` enclosing it: a group directly under that macro's
/// group with an attribute or an `else` before it. A group there with neither is a block.
pub(super) fn is_cfg_if_arm(tree: &TokenTree, open: usize) -> bool {
    tree.kind(open) == Kind::Open(Delimiter::Brace)
        && macro_group_kind(tree, open).is_none()
        && tree
            .enclosing(open)
            .is_some_and(|outer| macro_group_kind(tree, outer) == Some(GroupKind::CfgIf))
        && (matches!(tree.node_before(open), Some(Node::Attribute { .. }))
            || open.checked_sub(1).is_some_and(|e| tree.is(e, "else")))
}

/// Classify the group opening at `open`, given the kind of the group enclosing it (`None` at a file's top level).
pub(super) fn classify_group(
    tree: &TokenTree,
    open: usize,
    parent: Option<&GroupKind>,
) -> GroupKind {
    if let Some(kind) = macro_group_kind(tree, open) {
        return kind;
    }
    if tree.kind(open) != Kind::Open(Delimiter::Brace) {
        return GroupKind::Other;
    }
    if is_cfg_if_arm(tree, open) {
        return GroupKind::CfgArm;
    }
    if parent == Some(&GroupKind::UseGroup) {
        return GroupKind::UseGroup;
    }
    match item_owning(tree, open) {
        Some(head) => match head.keyword {
            ItemKeyword::Mod => match head.name {
                Some(name) => GroupKind::ModuleBody(tree.text(name).to_string()),
                None => GroupKind::Block,
            },
            ItemKeyword::Struct
            | ItemKeyword::Enum
            | ItemKeyword::Union
            | ItemKeyword::Trait
            | ItemKeyword::Impl => GroupKind::MemberBody,
            ItemKeyword::ExternBlock => GroupKind::ExternBlock,
            ItemKeyword::Use => GroupKind::UseGroup,
            _ => GroupKind::Block,
        },
        None => GroupKind::Block,
    }
}

/// The item header whose body the `{` at `open` is — or, for a `use`, any brace in its tree — read by walking back
/// over the nodes before it to the previous item boundary and parsing the header there. A brace group inside a
/// `<…>` group — `S<{ 1 }>`, `<const N: usize = { 1 }>` — is a const argument or default and no boundary, which
/// [`inside_an_angle_group`] asks of the forward reading rather than counting the `>`s the walk passes.
fn item_owning(tree: &TokenTree, open: usize) -> Option<ItemHead> {
    let head = item_from(tree, start_after_boundary(tree, open, |_| false))?;
    (head.keyword == ItemKeyword::Use || head.body == Some(open)).then_some(head)
}

/// Where what the group at `open` belongs to starts, so the outer attributes written on it stand directly before
/// the answer: the header of the item whose body it is, and otherwise the first node after the previous `;`, `,`,
/// brace group or attribute — the start of the statement, match arm or field it is written in. A `,` ends a match arm
/// or a field without ending an item, so it is a boundary only where no item owns the group: `fn f<A, B>() { … }`
/// is one header, while in `match v { #[cfg(x)] 1 => (), _ => { … } }` the `#[cfg(x)]` is the first arm's.
pub(super) fn owner_start(tree: &TokenTree, open: usize) -> usize {
    match item_owning(tree, open) {
        Some(head) => head.start,
        None => start_after_boundary(
            tree,
            open,
            |node| matches!(node, Node::Token(t) if tree.is(t, ",")),
        ),
    }
}

/// The first node after the previous item boundary before `open` — a `;`, a brace group or an attribute — or after
/// the previous node `also_ends` accepts.
fn start_after_boundary(tree: &TokenTree, open: usize, also_ends: impl Fn(Node) -> bool) -> usize {
    let mut start = open;
    while let Some(node) = tree.node_before(start) {
        if (is_boundary_node(tree, node) && !inside_an_angle_group(tree, node)) || also_ends(node) {
            break;
        }
        start = node.first();
    }
    start
}

/// Whether `node`, a brace group, stands inside a `<…>` group the one forward reading, [`angles`], pairs: a `<`
/// of its level before it opening a group that closes after it. A comparison's `>` pairs with no `<` that reading
/// accepts, so it moves nothing. A brace group inside a generic list follows the `<`, `<<` or `,` a const argument
/// does or the `=` a const default does, so a group after anything else — a loop's or an `if`'s body — is no const
/// argument; the rest is read from the pairing, once per tree, so a long list of groups is not read back once per
/// group in it.
fn inside_an_angle_group(tree: &TokenTree, node: Node) -> bool {
    let Node::Group { open, .. } = node else {
        return false;
    };
    let before = open.wrapping_sub(1);
    (tree.is(before, "<") || tree.is(before, "<<") || tree.is(before, ",") || tree.is(before, "="))
        && angles(tree)[open].enclosed
}

/// Whether a node ends what stands before an item: a `;`, a brace group, a braced macro call standing as an item, or
/// an attribute. A macro call with a parenthesized or bracketed group stands as an item only with a `;` after it, so
/// without one it is a type or an expression — the `t!()` of `impl Tr for t!() { … }` — and the header goes on.
fn is_boundary_node(tree: &TokenTree, node: Node) -> bool {
    match node {
        Node::Token(t) => tree.is(t, ";"),
        Node::Group { open, .. } => tree.kind(open) == Kind::Open(Delimiter::Brace),
        Node::Macro { name, open, .. } => {
            tree.kind(open) == Kind::Open(Delimiter::Brace) && macro_stands_as_an_item(tree, name)
        }
        Node::Attribute { .. } => true,
    }
}

/// Whether token `i` can head a path: an identifier, or `self`, `Self`, `super` or `crate`. The macro-path walk in
/// [`macro_stands_as_an_item`] takes it for every segment, so it also admits `a::crate::m!`, which rustc refuses; no
/// answer depends on that, since such a path does not compile.
pub(super) fn heads_a_path(tree: &TokenTree, i: usize) -> bool {
    i < tree.len()
        && match tree.kind(i) {
            Kind::Ident | Kind::RawIdent => true,
            Kind::Keyword => matches!(tree.text(i), "self" | "Self" | "super" | "crate"),
            _ => false,
        }
}

/// Whether the macro invocation whose name is the token at `name` stands where an item can. Its path is read back
/// over `segment ::` pairs, each segment one [`heads_a_path`] accepts, and past a leading `::`; the path's first token
/// must stand in a list of items, with nothing before it there or a `{`, a `}`, a `;` or an attribute's closing `]`
/// right before it. Only there can it expand to items. The token before the path is read alone, and a `]` asks only
/// whether the node it closes is an attribute, so the answer takes no recursion.
pub(super) fn macro_stands_as_an_item(tree: &TokenTree, name: usize) -> bool {
    let mut start = name;
    while start >= 2 && tree.is(start - 1, "::") && heads_a_path(tree, start - 2) {
        start -= 2;
    }
    if start >= 1 && tree.is(start - 1, "::") {
        start -= 1;
    }
    if !in_an_item_list(tree, start) {
        return false;
    }
    let Some(before) = start.checked_sub(1) else {
        return true;
    };
    match tree.kind(before) {
        Kind::Open(Delimiter::Brace) | Kind::Close(Delimiter::Brace) => true,
        Kind::Close(Delimiter::Bracket) => {
            matches!(tree.node_before(start), Some(Node::Attribute { .. }))
        }
        _ => tree.is(before, ";"),
    }
}

/// Whether token `i` stands where a list of items can: at a file's top level, or directly inside a brace group. No
/// item stands directly inside a parenthesized or bracketed group.
fn in_an_item_list(tree: &TokenTree, i: usize) -> bool {
    tree.enclosing(i)
        .is_none_or(|open| tree.kind(open) == Kind::Open(Delimiter::Brace))
}

/// Whether an item may start at token `i`: it stands in a list of items — a file's top level or a brace group — and
/// nothing stands before it there, or a `;`, a brace group, a braced macro call standing as an item, or an attribute
/// does. A keyword in a type or an expression — the `impl` of `-> impl Trait`, the `fn` of `x: fn()` — has an
/// operator or a delimiter before it instead.
fn is_item_start(tree: &TokenTree, i: usize) -> bool {
    in_an_item_list(tree, i)
        && tree
            .node_before(i)
            .is_none_or(|node| is_boundary_node(tree, node))
}

/// The item keyword at token `j`, as the tokens after it confirm.
fn item_keyword(tree: &TokenTree, j: usize) -> Option<ItemKeyword> {
    if j >= tree.len() {
        return None;
    }
    let keyword = match (tree.kind(j), tree.text(j)) {
        (Kind::Keyword, "mod") => ItemKeyword::Mod,
        (Kind::Keyword, "struct") => ItemKeyword::Struct,
        (Kind::Keyword, "enum") => ItemKeyword::Enum,
        (Kind::Keyword, "trait") => ItemKeyword::Trait,
        (Kind::Keyword, "type") => ItemKeyword::Type,
        (Kind::Keyword, "fn") => ItemKeyword::Fn,
        (Kind::Keyword, "const")
            if (tree.is_word(j + 1) || tree.is(j + 1, "_")) && tree.is(j + 2, ":") =>
        {
            ItemKeyword::Const
        }
        (Kind::Keyword, "static") => ItemKeyword::Static,
        (Kind::Keyword, "impl") => ItemKeyword::Impl,
        (Kind::Keyword, "use") => ItemKeyword::Use,
        (Kind::Keyword, "extern") if tree.is(j + 1, "crate") => ItemKeyword::ExternCrate,
        (Kind::Keyword, "extern") => {
            let after = if tree.kind(j + 1) == Kind::Literal {
                j + 2
            } else {
                j + 1
            };
            (tree.kind(after) == Kind::Open(Delimiter::Brace))
                .then_some(ItemKeyword::ExternBlock)?
        }
        (Kind::Ident, "union") if tree.is_word(j + 1) => ItemKeyword::Union,
        (Kind::Ident, "macro_rules") if tree.is(j + 1, "!") => ItemKeyword::MacroRules,
        _ => return None,
    };
    Some(keyword)
}

/// The visibility a `pub` at `pub_at` gives, and the index past it and its `(…)` if it has one.
fn visibility_at(tree: &TokenTree, pub_at: usize) -> (Visibility, usize) {
    let open = pub_at + 1;
    if tree.kind(open) != Kind::Open(Delimiter::Parenthesis) {
        return (Visibility::Public, pub_at + 1);
    }
    let close = tree.partner(open);
    let words: Vec<&str> = (open + 1..close).map(|k| tree.text(k)).collect();
    let visibility = match words.as_slice() {
        ["crate"] => Visibility::Public,
        ["self"] => Visibility::Private,
        ["super"] => Visibility::Super,
        ["in", rest @ ..] => Visibility::In(rest.concat()),
        _ => Visibility::Public,
    };
    (visibility, close + 1)
}

/// The item header starting at token `start`, if one does: qualifiers, then an item keyword, at an item start.
fn item_from(tree: &TokenTree, start: usize) -> Option<ItemHead> {
    if start >= tree.len() || !is_item_start(tree, start) {
        return None;
    }
    let mut j = start;
    let mut visibility = Visibility::Private;
    loop {
        if tree.is(j, "pub") {
            let (v, next) = visibility_at(tree, j);
            visibility = v;
            j = next;
            continue;
        }
        if item_keyword(tree, j).is_some() {
            break;
        }
        if j < tree.len()
            && ITEM_QUALIFIERS.contains(&tree.text(j))
            && matches!(tree.kind(j), Kind::Keyword | Kind::Ident)
        {
            j += 1;
            if tree.is(j - 1, "extern") && tree.kind(j) == Kind::Literal {
                j += 1;
            }
            continue;
        }
        return None;
    }
    let keyword = item_keyword(tree, j)?;
    let keyword_at = j;
    let name = match keyword {
        ItemKeyword::Impl | ItemKeyword::Use | ItemKeyword::ExternBlock => None,
        ItemKeyword::MacroRules => Some(j + 2).filter(|&n| tree.is_word(n)),
        ItemKeyword::ExternCrate => Some(j + 2).filter(|&n| tree.is_word(n) || tree.is(n, "self")),
        ItemKeyword::Static if tree.is(j + 1, "mut") => Some(j + 2).filter(|&n| tree.is_word(n)),
        _ => Some(j + 1).filter(|&n| tree.is_word(n)),
    };
    let (body, value_struct) = match keyword {
        ItemKeyword::Mod
        | ItemKeyword::Struct
        | ItemKeyword::Enum
        | ItemKeyword::Union
        | ItemKeyword::Trait
        | ItemKeyword::Impl
        | ItemKeyword::Fn
        | ItemKeyword::ExternBlock => item_body(tree, name.map_or(j + 1, |n| n + 1), keyword),
        _ => (None, false),
    };
    Some(ItemHead {
        start,
        keyword,
        keyword_at,
        name,
        visibility,
        body,
        value_struct,
    })
}

/// The item header whose keyword is the token at `keyword_at`, reading back over the qualifiers before it.
pub(super) fn item_at_keyword(tree: &TokenTree, keyword_at: usize) -> Option<ItemHead> {
    item_keyword(tree, keyword_at)?;
    let mut start = keyword_at;
    while let Some(node) = tree.node_before(start) {
        let first = node.first();
        let qualifies = match node {
            Node::Token(t) => {
                (matches!(tree.kind(t), Kind::Keyword | Kind::Ident)
                    && (ITEM_QUALIFIERS.contains(&tree.text(t)) || tree.is(t, "pub")))
                    || (tree.kind(t) == Kind::Literal
                        && t.checked_sub(1).is_some_and(|e| tree.is(e, "extern")))
            }
            Node::Group { open, .. } => {
                tree.kind(open) == Kind::Open(Delimiter::Parenthesis)
                    && open.checked_sub(1).is_some_and(|p| tree.is(p, "pub"))
            }
            _ => false,
        };
        if !qualifies {
            break;
        }
        start = first;
    }
    item_from(tree, start).filter(|head| head.keyword_at == keyword_at)
}

/// The visibility the qualifiers before the keyword at `keyword_at` give.
pub(super) fn visibility_before(tree: &TokenTree, keyword_at: usize) -> Visibility {
    item_at_keyword(tree, keyword_at).map_or(Visibility::Private, |head| head.visibility)
}

/// The brace body of the item whose header continues at `from`, and — for a `struct` — whether a parenthesized
/// field list or a `;` comes first, making it a tuple or unit struct. A `<…>` generic list is passed over whole, so
/// a brace in a const generic argument is not the body, a macro invocation is passed over whole, so the braces of
/// `impl Tr for t!{} { … }`'s self type are not its body, and a `where` clause's parentheses do not make a tuple.
fn item_body(tree: &TokenTree, from: usize, keyword: ItemKeyword) -> (Option<usize>, bool) {
    let mut k = from;
    let mut after_where = false;
    while k < tree.len() {
        if let Node::Macro { close, .. } = tree.node_at(k) {
            k = close + 1;
            continue;
        }
        match tree.kind(k) {
            Kind::Open(Delimiter::Brace) => return (Some(k), false),
            Kind::Open(Delimiter::Parenthesis)
                if keyword == ItemKeyword::Struct && !after_where =>
            {
                return (None, true);
            }
            Kind::Open(_) => {
                k = tree.partner(k) + 1;
                continue;
            }
            Kind::Close(_) => return (None, false),
            _ if tree.is(k, ";") => return (None, keyword == ItemKeyword::Struct),
            _ if opens_an_angle_group(tree, k) => {
                k = angle_group_end(tree, k).map_or(k + 1, |close| close + 1);
                continue;
            }
            _ if tree.is(k, "where") => after_where = true,
            _ => {}
        }
        k += 1;
    }
    (None, false)
}

/// The `<…>` group a `<` or `<<` opens: the token closing it, and for a `<<` the token closing the inner group
/// its second `<` opens — the same token where one `>>` closes both.
pub(super) struct AngleGroup {
    /// The token closing the group the opener opens.
    pub close: usize,
    /// For a `<<`, the token closing the group its second `<` opens, which is `close` itself where one
    /// `>>` closes both; `None` for a `<` or `<-`.
    pub inner_close: Option<usize>,
}

/// The `<…>` group the `<` or `<<` at `open` opens, reading sibling nodes — so a `>` inside a `(…)`, `[…]` or
/// `{…}` never counts — with `<` and `<-` opening one level and `<<` two, `>` and `>=` closing one and `>>` and `>>=` two;
/// `->`, `=>`, `<=` and `<<=` are other tokens and neither. `None` at a `;` or at the end of the level: what
/// opened is not a generic list. The one reading of a `<…>` group every reader here shares, read from [`angles`].
pub(super) fn angle_group(tree: &TokenTree, open: usize) -> Option<AngleGroup> {
    let slot = angles(tree).get(open)?;
    slot.close.map(|close| AngleGroup {
        close,
        inner_close: slot.inner_close,
    })
}

/// Every `<…>` group of `tree`, paired in one forward pass and kept on the tree. Each level of sibling nodes keeps a
/// running depth — `<` and `<-` raising it by one, `<<` by two, `>` and `>=` lowering it by one, `>>` and `>>=` by
/// two — and a stack of the openers still open: an opener closes at the first token where the depth falls to what it
/// was before the opener, and a `<<`'s inner group where it falls to one above that. A `;` or the level's closer ends
/// every opener still open with no group. The depths an open stack waits for rise from its bottom to its top, so each
/// token closes the openers it closes from the top, and a level is read in time its length bounds. A second pass marks
/// each `(`, `[` or `{` a group of its level spans.
fn angles<'t>(tree: &'t TokenTree<'_>) -> &'t [AngleSlot] {
    tree.angle_slots(|| {
        let mut slots = vec![AngleSlot::default(); tree.len()];
        let level = |k: usize| tree.enclosing(k).map_or(0, |open| open + 1);
        let mut open: Vec<Vec<(usize, isize, bool)>> = vec![Vec::new(); tree.len() + 1];
        let mut depth = vec![0isize; tree.len() + 1];
        for k in 0..tree.len() {
            match tree.kind(k) {
                Kind::Open(_) => continue,
                Kind::Close(_) => {
                    open[tree.partner(k) + 1].clear();
                    continue;
                }
                _ => {}
            }
            let at = level(k);
            if tree.is(k, ";") {
                open[at].clear();
                continue;
            }
            let delta = match tree.text(k) {
                "<" | "<-" => 1,
                "<<" => 2,
                ">" | ">=" => -1,
                ">>" | ">>=" => -2,
                _ => 0,
            };
            let before = depth[at];
            depth[at] += delta;
            while let Some(&(opener, waits_for, inner)) = open[at].last() {
                if depth[at] > waits_for {
                    break;
                }
                open[at].pop();
                if inner {
                    slots[opener].inner_close = Some(k);
                } else {
                    slots[opener].close = Some(k);
                }
            }
            if opens_an_angle_group(tree, k) {
                open[at].push((k, before, false));
                if tree.is(k, "<<") {
                    open[at].push((k, before + 1, true));
                }
            }
        }
        let mut reach = vec![0usize; tree.len() + 1];
        for (k, slot) in slots.iter_mut().enumerate() {
            let at = level(k);
            if matches!(tree.kind(k), Kind::Open(_)) {
                slot.enclosed = reach[at] > k;
            } else if let Some(close) = slot.close {
                reach[at] = reach[at].max(close);
            }
        }
        slots
    })
}

/// Whether token `k` opens a `<…>` group where one may open: `<`, `<<` opening two, or `<-`, which rustc reads as a
/// `<` before a `-` there, so `f::<-1>()` is a turbofish holding `-1`.
pub(super) fn opens_an_angle_group(tree: &TokenTree, k: usize) -> bool {
    tree.is(k, "<") || tree.is(k, "<<") || tree.is(k, "<-")
}

/// The token closing the `<…>` group the `<` or `<<` at `open` opens, as [`angle_group`] reads it.
pub(super) fn angle_group_end(tree: &TokenTree, open: usize) -> Option<usize> {
    angle_group(tree, open).map(|group| group.close)
}

/// The keywords that end an operand, which no expression follows: a path expression's `self`, `Self`, `super` and
/// `crate` segments (*Path expressions*), the boolean literals `true` and `false` (*Literal expressions*), and the
/// `await` of a postfix `.await` (*Await expressions*).
const OPERAND_KEYWORDS: [&str; 7] = ["self", "Self", "super", "crate", "true", "false", "await"];

/// Whether the token before `i` ends an operand, so a `<` at `i` compares: an identifier, a raw identifier, a
/// literal, a closing delimiter, `?`, or one of [`OPERAND_KEYWORDS`]. A `}` ends a block-like operand as well as a
/// statement, and is read as the operand's end, so a comparison after a block never opens a qualified path; a
/// qualified path opening a statement after a `}` is read as a comparison — the declared bound
/// `inline-symbol-path-confinement/a-qualified-path-after-a-closing-brace-is-read-as-a-rooted-path-a-stated-bound`.
fn ends_an_operand(tree: &TokenTree, i: usize) -> bool {
    let Some(prev) = i.checked_sub(1) else {
        return false;
    };
    match tree.kind(prev) {
        Kind::Ident | Kind::RawIdent | Kind::Literal | Kind::Close(_) => true,
        Kind::Keyword => OPERAND_KEYWORDS.contains(&tree.text(prev)),
        Kind::Punct => tree.text(prev) == "?",
        Kind::Lifetime | Kind::Open(_) | Kind::End => false,
    }
}

/// Whether the `<` at `i` opens a generic parameter list rather than a path: after `impl` it always does
/// (*Implementations*), and after `for` it opens a higher-ranked binder where a lifetime follows it, since a binder
/// holds lifetimes alone (*Higher-ranked trait bounds*) — so `impl<T> ::m::Tr for W<T>` and `F: for<'a> ::m::Fn(&'a u8)`
/// read `::m::…` as a rooted path, while `for <T as Tr>::C in …` still opens a qualified path.
fn opens_a_generic_list(tree: &TokenTree, i: usize) -> bool {
    let Some(prev) = i.checked_sub(1) else {
        return false;
    };
    tree.kind(prev) == Kind::Keyword
        && match tree.text(prev) {
            "impl" => true,
            "for" => tree.kind(i + 1) == Kind::Lifetime,
            _ => false,
        }
}

/// The `::` after the `>` closing the qualified path's `<…>` the `<` or `<<` at `i` opens, if it opens one: the
/// token before it does not end an operand — a comparison needs a left operand — nor opens a generic parameter list
/// ([`opens_a_generic_list`]), its `<…>` closes at its level, and a `::` follows. The one judgement of a `<` at a
/// path's start.
pub(super) fn opens_a_qualified_path(tree: &TokenTree, i: usize) -> Option<usize> {
    if ends_an_operand(tree, i) || opens_a_generic_list(tree, i) {
        return None;
    }
    let close = angle_group_end(tree, i)?;
    tree.is(close + 1, "::").then_some(close + 1)
}

/// The token index of every variant name an `enum` body declares: the first word of each comma-separated member,
/// past attributes. A discriminant is an expression, where a `<` opens a `<…>` group only as a turbofish, after `::`,
/// or as a qualified path, which [`opens_a_qualified_path`] decides as it decides one for every path reader — no
/// operand ends before it and a `::` follows its group; such a group is passed over whole, so a comma inside it
/// separates no members, and every other `<` is a comparison or a shift: `enum E { A = f::<u8, m::K>(), B = 1 << 3,
/// C = 64 >> 1, D = 1 + <u8 as Tr<u8, m::K>>::X, F }` declares all five, and `m::K` is a path, measured against rustc
/// 1.96.0, edition 2021, with a generic `const fn f`.
pub(super) fn variant_positions(tree: &TokenTree) -> std::collections::BTreeSet<usize> {
    let mut names = std::collections::BTreeSet::new();
    for open in 0..tree.len() {
        if tree.kind(open) != Kind::Open(Delimiter::Brace)
            || !item_owning(tree, open).is_some_and(|head| head.keyword == ItemKeyword::Enum)
        {
            continue;
        }
        let close = tree.partner(open);
        let mut k = open + 1;
        let mut expect_name = true;
        while k < close {
            let node = tree.node_at(k);
            if let Node::Attribute { close, .. } = node {
                k = close + 1;
                continue;
            }
            let turbofish = opens_an_angle_group(tree, k)
                && k.checked_sub(1).is_some_and(|before| tree.is(before, "::"));
            let close = if turbofish {
                angle_group_end(tree, k)
            } else {
                opens_a_qualified_path(tree, k).map(|tail| tail - 1)
            };
            if let Some(close) = close {
                expect_name = false;
                k = close + 1;
                continue;
            }
            if expect_name && tree.is_word(k) {
                names.insert(k);
            }
            expect_name = tree.is(k, ",");
            k = node.last() + 1;
        }
    }
    names
}

#[cfg(test)]
mod tests {
    use super::super::token_tree::Edition;
    use super::*;

    /// A discriminant's turbofish holds commas that separate no members, while a shift is no `<…>` group, so the
    /// members after both are still named: rustc 1.96.0, edition 2021, compiles
    /// `pub enum E { A = f::<u8, m::K>(), B = 1 << 3, C = 64 >> 1, D = 1 + <u8 as Tr<u8, m::K>>::X, F }` with a generic
    /// `const fn f` and a trait `Tr` declaring `X`, so a qualified path after an operator is passed over too.
    #[test]
    fn a_discriminants_turbofish_separates_no_variants() {
        let source = "pub enum E { A = f::<u8, m::K>(), B = 1 << 3, C = 64 >> 1, D = 1 + <u8 as Tr<u8, m::K>>::X, F }";
        let tree = TokenTree::lex(source, Edition::Rust2018);
        let names: Vec<&str> = variant_positions(&tree)
            .into_iter()
            .map(|k| tree.text(k))
            .collect();
        assert_eq!(names, ["A", "B", "C", "D", "F"]);
    }

    fn kind_of(source: &str, brace: usize) -> GroupKind {
        let tree = TokenTree::lex(source, Edition::Rust2018);
        let opens: Vec<usize> = (0..tree.len())
            .filter(|&i| tree.kind(i) == Kind::Open(Delimiter::Brace))
            .collect();
        classify_group(&tree, opens[brace], None)
    }

    #[test]
    fn a_group_is_classified_by_the_header_before_it() {
        assert_eq!(
            kind_of("pub(crate) mod inner { }", 0),
            GroupKind::ModuleBody("inner".into())
        );
        assert_eq!(
            kind_of("#[cfg(x)] mod r#type { }", 0),
            GroupKind::ModuleBody("type".into())
        );
        assert_eq!(kind_of("pub fn f() -> Vec<u8> { }", 0), GroupKind::Block);
        assert_eq!(
            kind_of("impl<T> Tr for X<T> where T: Fn() -> u8 { }", 0),
            GroupKind::MemberBody
        );
        assert_eq!(kind_of("enum E { A }", 0), GroupKind::MemberBody);
        assert_eq!(
            kind_of("unsafe extern \"C\" { }", 0),
            GroupKind::ExternBlock
        );
        assert_eq!(kind_of("fn g() { unsafe { } }", 1), GroupKind::Block);
        assert_eq!(
            kind_of("thread_local! { } mod m { }", 1),
            GroupKind::ModuleBody("m".into())
        );
        assert_eq!(kind_of("fn g() -> impl Tr { }", 0), GroupKind::Block);
    }

    #[test]
    fn an_item_starts_only_after_a_boundary() {
        let tree = TokenTree::lex(
            "fn f(x: impl Tr) -> fn() { } pub(super) unsafe fn g() {}",
            Edition::Rust2018,
        );
        let impl_at = (0..tree.len()).find(|&i| tree.is(i, "impl")).unwrap();
        assert!(item_at_keyword(&tree, impl_at).is_none());
        let fns: Vec<usize> = (0..tree.len()).filter(|&i| tree.is(i, "fn")).collect();
        assert!(item_at_keyword(&tree, fns[0]).is_some());
        assert!(
            item_at_keyword(&tree, fns[1]).is_none(),
            "a fn pointer type is no item"
        );
        let g = item_at_keyword(&tree, fns[2]).unwrap();
        assert_eq!(g.visibility, Visibility::Super);
        assert_eq!(tree.text(g.name.unwrap()), "g");
    }

    #[test]
    fn a_const_fn_declares_no_const() {
        let tree = TokenTree::lex(
            "const fn f() {} const _: u8 = 0; const { }",
            Edition::Rust2018,
        );
        let consts: Vec<usize> = (0..tree.len()).filter(|&i| tree.is(i, "const")).collect();
        assert!(item_at_keyword(&tree, consts[0]).is_none());
        assert!(item_at_keyword(&tree, consts[1]).is_some_and(|h| h.name.is_none()));
        assert!(item_at_keyword(&tree, consts[2]).is_none());
    }

    #[test]
    fn a_tuple_or_unit_struct_declares_a_value() {
        for (source, value) in [
            ("struct P(u8);", true),
            ("struct U;", true),
            ("struct S { a: u8 }", false),
            ("struct S<F> where F: Fn() { f: F }", false),
            ("struct P<const N: usize = { 3 }>(u8);", true),
        ] {
            let tree = TokenTree::lex(source, Edition::Rust2018);
            let head = item_at_keyword(&tree, 0).unwrap();
            assert_eq!(head.value_struct, value, "{source}");
        }
    }

    #[test]
    fn an_angle_group_closes_as_rustc_splits_it() {
        let tree = TokenTree::lex("< Vec < fn ( ) -> u8 >> ; x", Edition::Rust2018);
        assert_eq!(angle_group_end(&tree, 0), Some(8));
        let tree = TokenTree::lex("< a ; >", Edition::Rust2018);
        assert_eq!(angle_group_end(&tree, 0), None);
        let tree = TokenTree::lex("< T : Into < u8 >>= Y", Edition::Rust2018);
        assert_eq!(angle_group_end(&tree, 0), Some(6));
        let tree = TokenTree::lex("< T >= Y", Edition::Rust2018);
        assert_eq!(angle_group_end(&tree, 0), Some(2));
    }

    #[test]
    fn a_shift_opening_two_groups_closes_the_inner_one_first() {
        for (source, inner, close) in [
            ("<< u8 as T > :: x >", 4, 7),
            ("<< A < B >> :: x >", 4, 7),
            ("<< u8 >>", 2, 2),
        ] {
            let tree = TokenTree::lex(source, Edition::Rust2018);
            let group = angle_group(&tree, 0).unwrap();
            assert_eq!(
                (group.inner_close, group.close),
                (Some(inner), close),
                "{source}"
            );
        }
        let tree = TokenTree::lex("< u8 >", Edition::Rust2018);
        assert_eq!(angle_group(&tree, 0).unwrap().inner_close, None);
    }
}
