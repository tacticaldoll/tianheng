//! An input a wrapper hands a gate across the process boundary, in each state it can be in.
//!
//! A gate that stands in front of an irreversible act is run by a wrapper, and its inputs arrive as
//! environment. `std::env::var` answers **not set** and **set but not UTF-8** with one `Err`, which is the
//! absent-versus-unreadable collapse this crate draws everywhere else: `Quoted`, `WorkspaceVersion`,
//! `PackageName`, `Declared`, `Package`, `Tracked`, `Failure` and `Site` all carry it inside `src`.
//!
//! **It was drawn for three of one gate's four inputs and not the fourth**, and the fourth was the one whose
//! absence means *no merge is being made*. So a subject the wrapper supplied as bytes the gate could not read
//! took the arm that returns clean, the run exited `0`, `require_one_pass` saw `1 passed`, and
//! `exec gh pr merge` recorded a subject no judgement had read — the one outcome the Core Contract forbids,
//! in front of a record that cannot be amended. Two spellings of one rule is what let the repair that closed
//! the other three stop one line short of it.
//!
//! One reader, one typed result, and consumers that match exhaustively: a fourth input cannot now be added
//! under a different rule, because there is no second rule to add it under.

use std::ffi::OsString;

/// What a wrapper handed a gate for one judged input.
///
/// Typed apart rather than an `Option`, because a caller reading `None` as *the wrapper never supplied this*
/// says so to an operator — over a value the wrapper did supply, and that the operator can see they passed.
#[derive(Debug, PartialEq, Eq)]
pub enum Supplied {
    /// The value, as the wrapper supplied it.
    Value(String),
    /// The wrapper set nothing. For an input that decides whether the act is happening at all, this is the
    /// ordinary answer rather than a fault.
    Absent,
    /// The wrapper set bytes this gate cannot read, which is **not** the same fact as setting nothing.
    Unreadable,
}

/// Which state `value` is in, where `value` is what [`std::env::var_os`] answered.
///
/// Taking the `Option<OsString>` rather than reading the variable itself is what makes both non-`Value`
/// answers reachable from a direction: the inputs are process environment, a parallel test run shares one,
/// and `set_var` would mutate it for every sibling. The same reason `shengmo::workspace::locate` takes its
/// marker as an argument.
pub fn supplied(value: Option<OsString>) -> Supplied {
    match value {
        None => Supplied::Absent,
        Some(value) => match value.into_string() {
            Ok(value) => Supplied::Value(value),
            Err(_) => Supplied::Unreadable,
        },
    }
}

/// [`supplied`] over the named environment variable — the form every gate harness actually uses.
pub fn from_env(name: &str) -> Supplied {
    supplied(std::env::var_os(name))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// All three answers, over values a direction constructs rather than over the environment it shares.
    #[test]
    fn absent_readable_and_unreadable_are_three_answers() {
        use std::os::unix::ffi::OsStringExt;

        assert_eq!(supplied(None), Supplied::Absent);
        assert_eq!(
            supplied(Some(OsString::from("a value"))),
            Supplied::Value("a value".to_string())
        );
        // A value that was set to nothing keeps its own meaning: it is a value, not an absence.
        assert_eq!(
            supplied(Some(OsString::from(""))),
            Supplied::Value(String::new())
        );
        assert_eq!(
            supplied(Some(OsString::from_vec(vec![b'a', 0xff, 0xfe]))),
            Supplied::Unreadable
        );
    }
}
