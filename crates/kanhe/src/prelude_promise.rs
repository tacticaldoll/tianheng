//! Repository check: the composed prelude's promise is named by the external compilation contract.
//!
//! `tianheng::prelude` is the adopter's entrypoint, and `crates/tianheng/tests/adopter_surface.rs` is a
//! separate crate compiled against it — so every name it mentions is reachable the way an adopter reaches
//! it. That file's own header says it "deliberately names the whole promised surface", and until this check
//! nothing held it: the promise grew and the contract did not, which is how a window's worth of additions
//! reached the prelude without one of them entering the file whose job is to enumerate them. Not the stronger
//! claim that nothing outside had compiled them — `examples/observer-participant` is its own workspace and
//! reaches part of the protocol through a source patch. What went unheld is the correspondence, not the
//! reachability.
//!
//! **The relation is containment, not equality, and the asymmetry is the point.** Every promised member must
//! be mentioned; the contract may mention more, because it legitimately names things that are not prelude
//! members — the root-import `check_semantic`, `std::path::Path`, its own helpers. Requiring equality would
//! refuse the contract for being a test.
//!
//! **Mentioned, not asserted through one form.** `assert_public_type::<T>()` takes a type, and the promise is
//! not all types: a trait is `E0782: expected a type, found a trait`, and a function item is named by a
//! turbofish or a closure instead. Demanding one form would mean either a per-kind list — the several hand-kept lists
//! this repository's own drift law refuses — or a contract that cannot name its trait at all. So the check
//! asks the question it can answer honestly: does the name appear where the compiler will see it.

use crate::selection::{all_of, the_only};
use std::collections::BTreeSet;

/// What the check could not judge, kept distinct from what it found.
///
/// A disagreement is a list of promised names the contract never mentions. An input it cannot read is not an
/// empty disagreement: a prelude block that parses to nothing, or a contract file with no identifiers at all,
/// means the corpus never arrived, and reporting that as clean is the vacuity direction every check here owes
/// a refusal to.
#[derive(Debug, PartialEq, Eq)]
pub enum Promise {
    /// Every promised member is mentioned by the contract.
    Kept,
    /// These promised members appear nowhere in the contract, in the order they must be added.
    Unnamed(Vec<String>),
    /// An input could not be judged, with the reason an operator acts on.
    CannotJudge(String),
}

/// Why this reader could not read the promise, as opposed to a promise it read and disagreed with.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Unreadable {
    /// A promised member written in a form this reader does not read — a path, a rename, a nested group or a
    /// glob — quoted as written.
    Member(String),
    /// Several `pub mod prelude {` markers, so which block carries the promise is decided by position.
    SeveralPreludes(usize),
    /// The block's opener was found and its closing brace was not, so where the promise ends is unknown.
    ///
    /// Refused rather than read to end of file. Reading on is what made a re-export *after* the module a
    /// promised member — a promise that widened by exactly the amount the reader failed to bound.
    UnclosedPrelude,
    /// A `pub use super::{` statement whose `};` this reader never reached, quoted from its opener.
    ///
    /// The block-level twin of [`Self::UnclosedPrelude`], one level in. It returned `Ok` of an empty set
    /// before, which `judge` then reported as *the promise parsed to no member* — an input this reader could
    /// not read, wearing the diagnostic of a promise that genuinely holds nothing, and discarding whatever
    /// earlier statements in the same block had already contributed. `pub use super::{A, B} ;` is legal Rust
    /// and carries no `};`, so the shape is reachable rather than hypothetical.
    UnterminatedStatement(String),
}

/// The names `pub mod prelude` re-exports, recognized by **position** rather than by a bare marker.
///
/// The block is found by entering `pub mod prelude {`, **leaving at its matching close**, and reading the
/// `pub use super::{ … };` between — so a re-export anywhere else in the file is not the promise. A reader
/// keyed on the marker alone agrees exactly while no sibling re-export of that form exists, and the first one
/// added would widen the promise without changing a line here; a correctness that depends on the absence of
/// something is the shape this family declines. Measured when this was written: no sibling existed, which is
/// precisely why the looser reader could not have been caught by running it.
///
/// **Leaving is half of that, and this doc claimed the whole of it while the code held one half.** Entering
/// alone excludes a sibling written *before* the module and absorbs one written after it, because the read
/// then ran to end of file. Every fixture placed the sibling before — the arrangement the unbounded reader
/// answers correctly — so the property was asserted in the one position that could not fail. The two
/// directions are one requirement and are now held in both.
///
/// **A member this reader cannot understand is refused, never dropped.** The forms it reads are plain
/// identifiers; a path (`runner::Format`), a rename (`Foo as Bar`), a nested group (`a::{B, C}`) or a glob
/// (`runner::*`) is not one, and dropping such an entry would narrow the promise by exactly the amount the
/// reader failed to parse — silently, in the check whose subject is a promise narrowing unobserved. Measured
/// on a mixed list before this was written: three of five members vanished. Refusing costs no new extraction
/// rule and cannot narrow anything; if the prelude grows one of these forms, its author meets a refusal
/// naming the member.
///
/// **The glob was the one form this rule carved an exception for**, and it was the worst one to carve.
/// `trim_end_matches("::*")` turned `runner::*` into the identifier `runner`, which passes the test below and
/// enters the promise as a single member — so every name the glob re-exports went unchecked while the set
/// read as complete. One re-export could have silently emptied this check of most of its subject, in the file
/// whose entire purpose is catching a promise that narrowed without anyone noticing. The special case is
/// gone: a glob is a form this reader does not read, and it is refused like the rest.
pub fn promised_members(lib_rs: &str) -> Result<BTreeSet<String>, Unreadable> {
    // **Exactly one prelude block, or this reader does not know which one holds the promise.** The marker was
    // taken with `split_once`, so a second occurrence — a nested `pub mod prelude`, or the literal inside a
    // doc comment — silently decided the question by position. Unlike the re-export statements below, these
    // are not unioned: two blocks are two different promises, and merging them would invent a third.
    let Ok(module) = the_only(
        "`pub mod prelude {` block",
        all_of(lib_rs.split("pub mod prelude {").skip(1)),
    ) else {
        let count = lib_rs.split("pub mod prelude {").count() - 1;
        if count == 0 {
            return Ok(BTreeSet::new());
        }
        return Err(Unreadable::SeveralPreludes(count));
    };

    // **And the block ends at its own closing brace.** Entering the module was only half of what the
    // paragraph above claims: `split(…).skip(1)` runs from the opener to end of file, so it excluded a
    // sibling re-export written *before* the module and absorbed one written after it. Every fixture put the
    // sibling before, which is the arrangement the unbounded reader answers correctly, so the distinction the
    // doc asserted had never been run in the direction that fails.
    let Some(module) = block_body(module) else {
        return Err(Unreadable::UnclosedPrelude);
    };

    // **Every re-export statement in the block, not the first.** `split_once("pub use super::{")` read one,
    // so a second statement's members were dropped from the promise and no external contract had to name
    // them — the hole this file exists to close, in the same function whose own doc rejects the identical
    // dependence-on-absence for the outside-the-module case.
    //
    // Read rather than refused, because a second statement is a form this reader **can** read: splitting the
    // prelude across two `pub use super::{…}` is legal and ordinary, and refusing it would be a false
    // refusal over a promise that is perfectly well stated.
    let statements = all_of(module.split("pub use super::{").skip(1));
    if statements.is_empty() {
        return Ok(BTreeSet::new());
    }

    let mut members = BTreeSet::new();
    for statement in statements {
        // **A statement this reader cannot terminate is refused, never returned as an empty promise.** The
        // arm returned `Ok(BTreeSet::new())`, which discarded every member already read from earlier
        // statements and reached `judge` as *the promise parsed to no member* — the one diagnostic that names
        // a different fact. Refusing here keeps this reader's own rule, stated in this module's header: an
        // input it cannot read is not an empty disagreement.
        let Some((list, _)) = statement.split_once("};") else {
            return Err(Unreadable::UnterminatedStatement(
                statement.chars().take(60).collect(),
            ));
        };
        for entry in list.split(',') {
            let entry = entry.trim();
            if entry.is_empty() {
                continue;
            }
            if !entry.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
                return Err(Unreadable::Member(entry.to_string()));
            }
            members.insert(entry.to_string());
        }
    }
    Ok(members)
}

/// The prelude block's own text: from just inside its opener to its matching close, or `None` if the close is
/// never reached.
///
/// **Walked over executed Rust, and the two brace directions are not symmetric.** A stray `}` inside a comment
/// would end the block early and drop every member after it — the promise narrowing silently, which is the
/// one failure this file exists to catch. A stray `{` leaves the walk unbalanced, and that is refused rather
/// than guessed. Cutting line comments removes the dangerous direction; `region`'s declared block-comment
/// residue remains and errs toward the safe one, since an unclosed walk refuses.
///
/// A brace inside a **string literal** is the same residue and needs the same lexing, which is why this reads
/// a region rather than rolling its own cut. Valid Rust in this position is re-export statements, so no
/// instance exists here — and the reason that is not an argument for reading raw text is the finding this
/// function comes from: the loose reader was also correct on every input anyone had written.
fn block_body(tail: &str) -> Option<String> {
    let source = crate::region::Source::of(tail);
    let mut depth = 1usize;
    let mut body = String::new();
    for line in source.rust().lines() {
        for ch in line.chars() {
            if ch == '}' {
                depth -= 1;
                if depth == 0 {
                    return Some(body);
                }
            }
            body.push(ch);
            if ch == '{' {
                depth += 1;
            }
        }
        body.push('\n');
    }
    None
}

/// Every identifier the contract mentions, wherever it mentions it.
///
/// Comments are **not** stripped, deliberately. A contract naming a promised member only in a comment is
/// weaker than one naming it in code, but this check's question is whether the promise was noticed at all;
/// deciding that a mention is load-bearing is a judgement over text, which this repository has measured and
/// rejected. The compiler is what makes a mention bite, and a mention that compiles nothing still fails the
/// reviewer reading the diff — which is the layer that owns it.
pub fn mentioned_identifiers(contract_rs: &str) -> BTreeSet<String> {
    let mut found = BTreeSet::new();
    let mut current = String::new();
    for ch in contract_rs.chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' {
            current.push(ch);
        } else if !current.is_empty() {
            found.insert(std::mem::take(&mut current));
        }
    }
    if !current.is_empty() {
        found.insert(current);
    }
    found
}

/// Judge the promise against the contract.
pub fn judge(lib_rs: &str, contract_rs: &str) -> Promise {
    let promised = match promised_members(lib_rs) {
        Ok(promised) => promised,
        Err(Unreadable::Member(entry)) => {
            return Promise::CannotJudge(format!(
                "the prelude promises `{entry}`, which this check cannot read as a member name. A promised \
                 member written as a path, a rename, a nested group or a glob is not a member this check may \
                 drop: \
                 dropping it narrows the promise silently, and a promise that shrinks without saying so is \
                 the failure this whole check exists to catch"
            ));
        }
        Err(Unreadable::UnclosedPrelude) => {
            return Promise::CannotJudge(
                "`pub mod prelude {` was found and its closing brace was not, so where the promise ends is \
                 unknown. Read to end of file instead, every `pub use super::{ … };` after the module would \
                 join the promise — which is the widening this check would then have to report against the \
                 contract"
                    .to_string(),
            );
        }
        Err(Unreadable::SeveralPreludes(count)) => {
            return Promise::CannotJudge(format!(
                "{count} `pub mod prelude {{` markers are present, so which block carries the promise is \
                 decided by whichever comes first in the file. Two blocks are two promises and merging them \
                 would invent a third, so this is reported rather than resolved"
            ));
        }
        Err(Unreadable::UnterminatedStatement(opener)) => {
            return Promise::CannotJudge(format!(
                "a `pub use super::{{` statement in the prelude block reaches no `}};` — read from `{opener}`. \
                 The members of every statement before it were read, so this is an input this check cannot \
                 read rather than a promise of nothing, and the two demand different repairs: one is a \
                 malformed statement to fix, the other a prelude with no members"
            ));
        }
    };
    if promised.is_empty() {
        return Promise::CannotJudge(
            "the prelude promise parsed to no member — `pub mod prelude {` with its `pub use super::{ … };` \
             is what this check reads, and a promise of nothing would make every direction below hold \
             vacuously"
                .to_string(),
        );
    }
    let mentioned = mentioned_identifiers(contract_rs);
    if mentioned.is_empty() {
        return Promise::CannotJudge(
            "the external compilation contract yielded no identifier, so it was not read; an unread \
             contract mentions nothing and would report every promised member unnamed"
                .to_string(),
        );
    }
    let unnamed: Vec<String> = promised
        .into_iter()
        .filter(|name| !mentioned.contains(name))
        .collect();
    if unnamed.is_empty() {
        Promise::Kept
    } else {
        Promise::Unnamed(unnamed)
    }
}
