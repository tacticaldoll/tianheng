//! The statement split's failure matrix: [`statements`] ends a statement where bash ends the command.

use crate::shell::{Statement, statements};

fn values(statement: &Statement) -> Vec<&str> {
    statement
        .words
        .iter()
        .filter(|word| !word.operator)
        .map(|word| word.value.as_str())
        .collect()
}

/// A backslash-newline is removed where bash removes it, so the two physical lines are one statement and the
/// invocation's flags all belong to it.
#[test]
fn a_backslash_newline_joins_two_lines_into_one_statement() {
    let split = statements("cargo test -p kanhe \\\n    -- --exact the_gate\nnext\n")
        .expect("the fixture is placed");
    assert_eq!(split.len(), 2);
    assert_eq!(split[0].line, 1);
    assert_eq!(
        values(&split[0]),
        ["cargo", "test", "-p", "kanhe", "--", "--exact", "the_gate"]
    );
    assert_eq!(split[1].line, 3);
    assert_eq!(values(&split[1]), ["next"]);
}

/// An escaped backslash is a literal character, so the newline after it ends the statement — and a guard
/// written on the next line is another command, which bash refuses to parse there.
///
/// Measured rather than reasoned about: `printf 'x=1 \\\\\n|| echo hi\n' > s.sh; bash s.sh` runs the first
/// line alone (`\: command not found`) and refuses the second with `syntax error near unexpected token` —
/// two commands, never one. The join this row forbids is the one that read such a guard as the
/// acquisition's own.
#[test]
fn an_escaped_backslash_ends_the_statement_where_bash_ends_it() {
    let split = statements("x=$(tool) \\\\\n|| cannot_judge\n").expect("the fixture is placed");
    assert_eq!(split.len(), 2, "bash never runs these as one command");
    assert_eq!(values(&split[0]), ["x=$(…)", "tool", "\\"]);
    assert_eq!(split[1].line, 2);
    assert_eq!(values(&split[1]), ["cannot_judge"]);
}

/// A backslash followed by whitespace escapes the **space**, not the newline, so the statement ends there.
///
/// Measured rather than reasoned about: `printf 'echo A \\ \necho B\n' > s.sh; bash s.sh` prints `A  ` and
/// then `B` — two commands.
#[test]
fn a_backslash_before_whitespace_escapes_the_space_not_the_newline() {
    let split = statements("echo A \\ \necho B\n").expect("the fixture is placed");
    assert_eq!(
        split.len(),
        2,
        "bash runs these as two commands, so joining them is a statement this script never had"
    );
    assert_eq!(split[1].line, 2);
    assert_eq!(values(&split[1]), ["echo", "B"]);
}

/// Inside single quotes a backslash is literal text: it joins nothing, and the string runs across the
/// newline exactly as bash passes it.
#[test]
fn a_backslash_inside_single_quotes_is_literal_and_joins_nothing() {
    let split = statements("printf '%s' 'a\\\nb'\nprintf done\n").expect("the fixture is placed");
    assert_eq!(split.len(), 2);
    assert_eq!(values(&split[0]), ["printf", "%s", "a\\\nb"]);
    assert_eq!(values(&split[1]), ["printf", "done"]);
}

/// A continuation pulls the next physical line up before the comment rule runs, as bash does: the `#` then
/// ends the command, and the line after the comment stands alone.
///
/// Measured: for `echo START \` / `# comment` / `--exact ghost`, bash prints `START` and reports
/// `--exact: command not found`. A split that compacted the comment away first would bind the third line
/// into the first command — an invocation bash never runs.
#[test]
fn a_comment_under_a_continuation_ends_the_command_where_bash_does() {
    let split =
        statements("echo START \\\n# comment\n--exact ghost\n").expect("the fixture is placed");
    assert_eq!(split.len(), 2);
    assert_eq!(values(&split[0]), ["echo", "START"]);
    assert_eq!(split[1].line, 3);
    assert_eq!(values(&split[1]), ["--exact", "ghost"]);
}

/// A newline inside a command substitution separates the substitution's own commands, never the statement
/// the substitution stands in.
#[test]
fn a_newline_inside_a_command_substitution_does_not_end_the_statement() {
    let split = statements("x=$(printf a\nprintf b) || cannot_judge\ny=1\n")
        .expect("the fixture is placed");
    assert_eq!(split.len(), 2);
    assert!(
        split[0].words.windows(2).any(|pair| {
            pair[0].operator
                && pair[0].value == "|"
                && pair[1].operator
                && pair[1].value == "|"
                && pair[0].start + 1 == pair[1].start
        }),
        "the guard belongs to the acquisition's statement, not to a statement of its own"
    );
}

/// What the lexer cannot place refuses the script, so no reader answers about words it never judged.
#[test]
fn what_the_lexer_cannot_place_refuses_the_script() {
    let Err((line, what)) = statements("cat <<EOF\n--exact ghost\nEOF\n") else {
        panic!("a here-document is not placed, so the script must be refused rather than read past")
    };
    assert_eq!(line, 1);
    assert!(what.contains("here-document"), "{what}");
}
