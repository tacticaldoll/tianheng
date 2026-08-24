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

use std::collections::BTreeMap;

use crate::region::Source;

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
/// `None` where no arm calls anything in that shape, which is itself a finding for a wrapper that takes
/// values: it means no arm hands its guard a value to judge.
pub fn value_guard(script: &str) -> Option<String> {
    let source = Source::of(script);
    source
        .shell()
        .lines()
        .filter_map(|line| line.trim().strip_suffix(VALUE_GUARD_CALL))
        .map(str::to_string)
        .next()
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
