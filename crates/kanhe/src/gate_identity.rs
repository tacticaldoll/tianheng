//! Whether the test a wrapper asks for by name is a test that exists.
//!
//! A gate reached through `cargo test … -- --exact <ident>` is asked for by a string, and `libtest` exits `0`
//! when that string selects nothing. The wrapper's own assertion covers the moment; this covers the interval,
//! by holding the identifier to the target it is cited against — a test identifier is a reference into this
//! repository exactly as a path is, and the reference gate matches paths only.

use crate::refusal::{Refusal, cannot_judge, violation};

/// One `--exact` citation found in a script: the identifier, and the invocation it belongs to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Citation {
    pub script: String,
    pub identifier: String,
    /// The `--test <target>` of the same invocation, absent where the line names none.
    pub target: Option<String>,
    /// The `-p <package>` of the same invocation, absent where the line names none.
    pub package: Option<String>,
}

/// Physical lines joined where one ends in a backslash — the shell's own continuation rule.
///
/// A gate invocation spans several lines. Asking about `--exact` and `--test` per physical line can find them
/// in different units and bind neither, so the shell's continuation rule is applied before either is read.
pub fn logical_lines(script: &str) -> Vec<String> {
    let mut joined = Vec::new();
    let mut current = String::new();
    for line in script.lines() {
        if let Some(head) = line.strip_suffix('\\') {
            current.push_str(head);
            current.push(' ');
            continue;
        }
        current.push_str(line);
        joined.push(std::mem::take(&mut current));
    }
    if !current.is_empty() {
        joined.push(current);
    }
    joined
}

/// The value following `flag` in one logical line's whitespace-separated words.
fn value_after(words: &[&str], flag: &str) -> Option<String> {
    words
        .iter()
        .position(|word| *word == flag)
        .and_then(|at| words.get(at + 1))
        .map(|value| (*value).to_string())
}

/// Every `--exact <ident>` a script cites, with the invocation each belongs to.
pub fn citations(script_path: &str, script: &str) -> Vec<Citation> {
    let mut found = Vec::new();
    for line in logical_lines(script) {
        // A comment names no invocation; the `#` is the shell's own rule for that.
        if line.trim_start().starts_with('#') {
            continue;
        }
        let words: Vec<&str> = line.split_whitespace().collect();
        for (at, word) in words.iter().enumerate() {
            if *word != "--exact" {
                continue;
            }
            let Some(identifier) = words.get(at + 1) else {
                continue;
            };
            found.push(Citation {
                script: script_path.to_string(),
                identifier: (*identifier).to_string(),
                target: value_after(&words, "--test"),
                package: value_after(&words, "-p"),
            });
        }
    }
    found
}

/// Each name a target's harness registers, **exactly as `--list` prints it**.
///
/// The full path, module qualification and all, because that is the string `--exact` compares against. Taking
/// the last `::` segment was the first shape and it is inexact in **both** directions: a test moved into a
/// module lists as `inner::the_gate`, truncates to `the_gate`, matches a citation of `the_gate` and reports the
/// gate registered — while `--exact the_gate` selects nothing, which is the condition this whole check exists
/// to catch. And a leaf shared by two modules truncates to one name twice, so a citation that `--exact` resolves
/// to exactly one test is refused for naming a set.
///
/// It read clean because both live citations sit at file scope, where the truncation is the identity function —
/// the comparison was `f() == f()`.
pub fn registered_names(listing: &str) -> Vec<String> {
    listing
        .lines()
        .filter_map(|line| line.split_once(": "))
        .filter(|(_, kind)| kind.trim() == "test")
        .map(|(name, _)| name.to_string())
        .collect()
}

/// Whether each citation names a test its target registers exactly once.
///
/// The identifier is resolved through the **harness** — `cargo test -p <pkg> --test <target> -- --list` — not
/// by mapping the target to a source path. That mapping would reimplement cargo's target resolution in string
/// form, and this repository has already shipped a false negative from mimicking a compiler's resolution by
/// reasoning instead of measuring. `--list` *is* the set `--exact` filters against, so the join is exact only
/// while both sides carry the **same** name — which is why the listed name is compared whole rather than by its
/// last segment, and why a name registered twice is settled for free.
pub fn offences(
    citations: &[Citation],
    list: impl Fn(&str, &str) -> Result<String, String>,
) -> Vec<Refusal> {
    let mut offences = Vec::new();
    for citation in citations {
        let (Some(package), Some(target)) = (&citation.package, &citation.target) else {
            offences.push(cannot_judge(format!(
                "{}: `--exact {}` names no `--test <target>` in its invocation, so the identifier cannot be \
                 bound to the harness that would register it — an identifier this check cannot resolve is \
                 not one it resolved as fine",
                citation.script, citation.identifier
            )));
            continue;
        };
        let listing = match list(package, target) {
            Ok(listing) => listing,
            Err(err) => {
                offences.push(cannot_judge(format!(
                    "{}: could not list the tests `{target}` registers in `{package}`, so whether it carries \
                     `{}` is unread rather than answered: {err}",
                    citation.script, citation.identifier
                )));
                continue;
            }
        };
        let matches = registered_names(&listing)
            .into_iter()
            .filter(|name| *name == citation.identifier)
            .count();
        match matches {
            1 => {}
            0 => offences.push(violation(format!(
                "{}: `--exact {}` names a test `{target}` does not register, so the gate this wrapper asks \
                 for selects nothing — and `libtest` exits 0 for a filter that matches nothing",
                citation.script, citation.identifier
            ))),
            many => offences.push(violation(format!(
                "{}: `--exact {}` names a test `{target}` registers {many} times, so the wrapper's citation \
                 names a set rather than the one gate it stands in front of",
                citation.script, citation.identifier
            ))),
        }
    }
    offences
}
