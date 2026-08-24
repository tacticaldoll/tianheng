//! Reading a shell wrapper's own argument parser, so a claim about its arms is a claim about the file.
//!
//! Two wrappers stand in front of this repository's irreversible acts, and the rules their parsers must obey
//! are the same rules. Stating those rules twice is how one of them ended up with a value guard that never
//! judged the value's shape while the other argued the point at length — so the reading lives here, once,
//! and both directions over it are the same reading.
//!
//! **What a reader over this must not do is shrink its set.** An arm it cannot attribute is a hole in the
//! claim, not an absence from it, so an unattributable guard call stops the read rather than vanishing from
//! it. That failure is measured: an arm spelled `-j)` was dropped by a form requiring a leading `--`, was
//! absent from the literal it was compared against too, and the both-ways equality then held over two sets
//! agreeing by both missing it.

use std::collections::{BTreeMap, BTreeSet};

use crate::refusal::Refusal;
use crate::region::Source;

/// The line opening the parser both wrappers write: `case $1 in`, over the positional arguments.
///
/// A wrapper whose parser is spelled otherwise yields no arms, and the both-ways equality over an
/// empty set is what refuses — the same floor `value_guard` states one function up.
const PARSER_CASE: &str = "case $1 in";

/// The call shape a value guard is recognised by: the count, the flag, and the value.
///
/// **The shape rather than the name, because the two wrappers spell the name differently.** A literal pair of
/// names would be a third thing to keep in step, which is what this module exists to remove. What is common
/// is that a guard able to judge a value must be *given* one.
pub const VALUE_GUARD_CALL: &str = r#" "$#" "$1" "${2-}""#;

/// What one `case` arm of a wrapper's parser does with the argument after it.
///
/// Three properties rather than two, because two could not tell *asks for nothing* from *asks and hands over
/// nothing to judge*. Measured: with one arm's call shortened to two arguments, a check comparing only
/// `guards` against `consumes` stayed green — the arm still opened with the guard's name, so it still read as
/// guarded while the value it took went unjudged.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Arm {
    /// The arm calls the value guard at all.
    pub guards: bool,
    /// The arm calls it with a value to judge — [`VALUE_GUARD_CALL`].
    pub guards_with_value: bool,
    /// The arm consumes the following argument, which is what taking a value *is*.
    pub consumes: bool,
}

/// The value guard's name, derived from the shape of its call.
///
/// **The candidates are a value first, and *how many* is answered explicitly.** The first form here took
/// `.next()` over the call lines — the habit [`crate::selection`] exists to end, and the one this module's
/// own header forbids in so many words. A wrapper with two differently-named guards would have had one
/// silently picked, the other's arms reading as unguarded.
///
/// [`crate::selection::the_only`] over the **distinct** names rather than the lines: one guard called from
/// several arms is the ordinary shape, and several guards in one wrapper is the finding. A refusal rather than
/// `None`, so a wrapper taking values with no guard at all says which way the count was wrong.
pub fn value_guard(script: &str, wrapper: &str) -> Result<String, Refusal> {
    let source = Source::of(script);
    let names: BTreeSet<String> = source
        .shell()
        .lines()
        .filter_map(|line| line.trim().strip_suffix(VALUE_GUARD_CALL))
        .map(str::to_string)
        .collect();
    crate::selection::the_only(&format!("value guard in {wrapper}"), names)
}

/// Every arm of the wrapper's `case`, by flag, read against the named guard.
///
/// Executed text, because both wrappers discuss these flags at length in prose and a reader over the whole
/// file would collect the commentary as parser arms.
///
/// # Panics
///
/// When a guard call belongs to no arm this reader could identify — the read stops rather than dropping it,
/// for the reason the module header gives.
pub fn parser_arms(script: &str, guard: &str) -> BTreeMap<String, Arm> {
    let source = Source::of(script);
    let executed = source.shell();
    // The trailing space excludes the guard's own definition line, which would otherwise be attributed to
    // whichever arm the reader had open — an order dependency between a function and the parser below it.
    let call = format!("{guard} ");
    // **Bounded to the parser's own `case`, not every `case` in the file.** This read every
    // `)`-terminated line in the script, and was correct only because `scripts/merge-pr.sh`'s inner
    // `case $conclusion in` writes each body on its pattern line (`SUCCESS) ;;`), so no line there ends in
    // `)`. Reformatting that inner case onto separate lines would have its `*)` open an arm and collide with
    // the parser's own `*` in the map below — `BTreeMap::insert` would drop one silently, and a dropped
    // catch-all is the arm every refusal rests on. The dependency was on someone else's formatting.
    let mut in_parser = false;
    let mut arms: BTreeMap<String, Arm> = BTreeMap::new();
    let mut open: Option<(Vec<String>, Arm)> = None;
    let close = |open: &mut Option<(Vec<String>, Arm)>, arms: &mut BTreeMap<String, Arm>| {
        if let Some((flags, arm)) = open.take() {
            for flag in flags {
                arms.insert(flag, arm);
            }
        }
    };
    for line in executed.lines() {
        let trimmed = line.trim();
        if trimmed == PARSER_CASE {
            in_parser = true;
            continue;
        }
        if in_parser && trimmed == "esac" {
            in_parser = false;
            continue;
        }
        if !in_parser {
            continue;
        }
        // A `case` pattern: every alternative is a flag or the catch-all. Read this way rather than by a
        // leading `--`, so the short and glued spellings the wrappers also carry are arms here too.
        let flags: Vec<String> = trimmed
            .trim_end_matches(')')
            .split('|')
            .map(|flag| flag.trim().to_string())
            .collect();
        if trimmed.ends_with(')')
            && flags
                .iter()
                .all(|flag| flag.starts_with('-') || flag == "*")
        {
            close(&mut open, &mut arms);
            open = Some((
                flags,
                Arm {
                    guards: false,
                    guards_with_value: false,
                    consumes: false,
                },
            ));
            continue;
        }
        if trimmed == ";;" {
            close(&mut open, &mut arms);
            continue;
        }
        let guards = trimmed.starts_with(&call);
        let guards_with_value = guards && trimmed.ends_with(VALUE_GUARD_CALL);
        // **The guard line is not evidence of consumption**, because the guard's own call carries the token
        // the consumption scan looks for: the three-argument form satisfies both tests at once, so for any
        // arm using it the two properties would agree by construction and the *asks but never reads*
        // direction would be dead. The scan surfaces are disjoint, so neither property testifies for the
        // other.
        let consumes = !guards
            && (trimmed.contains("$2") || trimmed.contains("${2") || trimmed.contains("shift 2"));
        match open.as_mut() {
            Some((_, arm)) => {
                arm.guards |= guards;
                arm.guards_with_value |= guards_with_value;
                arm.consumes |= consumes;
            }
            None => assert!(
                !guards,
                "a `{guard}` call at `{trimmed}` belongs to no arm this reader could identify, so a claim \
                 over these arms would run over a set that does not describe the wrapper. Either the arm is \
                 spelled in a shape this reader does not recognise, or the call sits outside the parser"
            ),
        }
    }
    close(&mut open, &mut arms);
    arms
}
