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

/// The names `pub mod prelude` re-exports, recognized by **position** rather than by a bare marker.
///
/// The block is found by entering `pub mod prelude {` and reading its `pub use super::{ … };`, so a re-export
/// anywhere else in the file is not the promise. **Entering the module is what makes that true by construction
/// rather than by circumstance.** A reader keyed on the marker alone agrees exactly while no sibling re-export
/// of that form exists, and the first one added would widen the promise without changing a line here — a
/// correctness that depends on the absence of something is the shape this family declines. Measured when this
/// was written: no sibling existed, which is precisely why the looser reader could not have been caught by
/// running it.
///
/// **A member this reader cannot understand is refused, never dropped.** The forms it reads are plain
/// identifiers; a path (`runner::Format`), a rename (`Foo as Bar`) or a nested group (`a::{B, C}`) is not one,
/// and dropping such an entry would narrow the promise by exactly the amount the reader failed to parse —
/// silently, in the check whose subject is a promise narrowing unobserved. Measured on a mixed list before this
/// was written: three of five members vanished. Refusing costs no new extraction rule and cannot narrow
/// anything; if the prelude grows one of these forms, its author meets a refusal naming the member.
pub fn promised_members(lib_rs: &str) -> Result<BTreeSet<String>, String> {
    let Some(module) = lib_rs.split_once("pub mod prelude {") else {
        return Ok(BTreeSet::new());
    };
    let Some(block) = module.1.split_once("pub use super::{") else {
        return Ok(BTreeSet::new());
    };
    let Some((list, _)) = block.1.split_once("};") else {
        return Ok(BTreeSet::new());
    };
    let mut members = BTreeSet::new();
    for entry in list.split(',') {
        let entry = entry.trim().trim_end_matches("::*");
        if entry.is_empty() {
            continue;
        }
        if !entry.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
            return Err(entry.to_string());
        }
        members.insert(entry.to_string());
    }
    Ok(members)
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
        Err(entry) => {
            return Promise::CannotJudge(format!(
                "the prelude promises `{entry}`, which this check cannot read as a member name. A promised \
                 member written as a path, a rename, or a nested group is not a member this check may drop: \
                 dropping it narrows the promise silently, and a promise that shrinks without saying so is \
                 the failure this whole check exists to catch"
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
