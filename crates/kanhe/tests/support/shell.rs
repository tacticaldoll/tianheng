//! One shell command line as argv under a supplied environment, so two spellings of one command compare as one.
//!
//! **A command is compared by its argv, not by its text.** `cargo "+$MSRV" test` with `MSRV=1.85` supplied
//! tokenizes to the same words as `cargo +1.85 test`, and a text comparison can only be made to agree by
//! rewriting one side — which is how a join ended up deleting every quote on a line and expanding `$MSRVX` as
//! `$MSRV`. Reading both sides through one tokenizer makes the question the shell's grammar rather than the
//! text's.
//!
//! **The environment is the one supplied, and the guarantee stops there.** A caller supplies the values a
//! workflow declares; what a variable holds when the line actually runs — after an earlier step writes
//! `$GITHUB_ENV`, or an action exports one — is not something this reads, so the argv is the one the supplied
//! values give.
//!
//! **What this does not read, it declines.** A command substitution, a backtick, a glob, a brace expansion, a
//! tilde, an operator, a positional or special parameter, a `${…}` beyond a bare name, and a variable the
//! supplied environment does not define are each either decided when the line runs or expanded by rules this
//! does not implement, so [`words`] returns `None` for the line rather than a guess. A caller reading a witness therefore treats `None` as *not a
//! witness*, and a caller reading a declaration treats it as unreadable.

use std::collections::BTreeMap;

/// The shell's own words — bash's builtins and reserved words — as `compgen -b` and `compgen -k` print them.
///
/// **A command word the shell owns decides what the shell does next; a program's cannot.** `exit`, `return`,
/// `exec`, `set -n`, `source`, `eval` and every loop or conditional can end the script or skip what follows,
/// and `builtin` and `command` reach any of them indirectly, while an external program cannot end the shell
/// that runs it. So a script is read as a list of commands only when none of its lines is the shell's own,
/// rather than by naming which of these alter control flow. Held against bash itself by
/// `the_shell_words_are_the_ones_bash_owns`, both ways.
pub const SHELL_OWN_WORDS: [&str; 83] = [
    "!",
    ".",
    ":",
    "[",
    "[[",
    "]]",
    "{",
    "}",
    "alias",
    "bg",
    "bind",
    "break",
    "builtin",
    "caller",
    "case",
    "cd",
    "command",
    "compgen",
    "complete",
    "compopt",
    "continue",
    "coproc",
    "declare",
    "dirs",
    "disown",
    "do",
    "done",
    "echo",
    "elif",
    "else",
    "enable",
    "esac",
    "eval",
    "exec",
    "exit",
    "export",
    "false",
    "fc",
    "fg",
    "fi",
    "for",
    "function",
    "getopts",
    "hash",
    "help",
    "history",
    "if",
    "in",
    "jobs",
    "kill",
    "let",
    "local",
    "logout",
    "mapfile",
    "popd",
    "printf",
    "pushd",
    "pwd",
    "read",
    "readarray",
    "readonly",
    "return",
    "select",
    "set",
    "shift",
    "shopt",
    "source",
    "suspend",
    "test",
    "then",
    "time",
    "times",
    "trap",
    "true",
    "type",
    "typeset",
    "ulimit",
    "umask",
    "unalias",
    "unset",
    "until",
    "wait",
    "while",
];

/// The words `line` runs, with `env` expanded — or `None` for a line whose words depend on its running.
///
/// An unquoted `#` at the start of a word ends the line, as the shell's comment does.
pub fn words(line: &str, env: &BTreeMap<String, String>) -> Option<Vec<String>> {
    let mut words = Vec::new();
    let mut current = String::new();
    let mut in_word = false;
    // Whether the word being read has a quoted part: an unquoted expansion to nothing removes its word, and a
    // quoted one is an empty word.
    let mut quoted = false;
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            ' ' | '\t' => {
                if in_word && (quoted || !current.is_empty()) {
                    words.push(std::mem::take(&mut current));
                }
                in_word = false;
                quoted = false;
            }
            '#' if !in_word => break,
            '\'' => {
                in_word = true;
                quoted = true;
                loop {
                    match chars.next()? {
                        '\'' => break,
                        other => current.push(other),
                    }
                }
            }
            '"' => {
                in_word = true;
                quoted = true;
                loop {
                    match chars.next()? {
                        '"' => break,
                        '\\' => match chars.next()? {
                            escaped @ ('$' | '`' | '"' | '\\') => current.push(escaped),
                            other => {
                                current.push('\\');
                                current.push(other);
                            }
                        },
                        '$' => current.push_str(&expansion(&mut chars, env)?),
                        '`' => return None,
                        other => current.push(other),
                    }
                }
            }
            '\\' => {
                in_word = true;
                current.push(chars.next()?);
            }
            '$' => {
                in_word = true;
                let value = expansion(&mut chars, env)?;
                // Unquoted, an expansion is split and globbed by the shell; a value that would be is declined.
                if value
                    .chars()
                    .any(|v| v.is_whitespace() || "*?[".contains(v))
                {
                    return None;
                }
                current.push_str(&value);
            }
            '`' | '|' | '&' | ';' | '<' | '>' | '(' | ')' | '*' | '?' | '[' | '{' | '}' | '~' => {
                return None;
            }
            other => {
                in_word = true;
                current.push(other);
            }
        }
    }
    if in_word && (quoted || !current.is_empty()) {
        words.push(current);
    }
    Some(words)
}

/// The value of the parameter a `$` opens: a bare `$NAME` or a braced `${NAME}`, defined in `env`.
fn expansion(
    chars: &mut std::iter::Peekable<std::str::Chars<'_>>,
    env: &BTreeMap<String, String>,
) -> Option<String> {
    let braced = chars.peek() == Some(&'{');
    if braced {
        chars.next();
    }
    let mut name = String::new();
    while let Some(&c) = chars.peek() {
        let admitted = if name.is_empty() {
            c.is_ascii_alphabetic() || c == '_'
        } else {
            c.is_ascii_alphanumeric() || c == '_'
        };
        if !admitted {
            break;
        }
        name.push(c);
        chars.next();
    }
    if name.is_empty() {
        return None;
    }
    if braced && chars.next()? != '}' {
        return None;
    }
    env.get(&name).cloned()
}

/// The commands a script runs, one argv per logical line — or `None` when any line is one [`words`] declines,
/// or one whose command word is the shell's own.
///
/// **The script is the unit, not the line.** A line can make the lines after it data rather than commands — a
/// here-document's opener, a string left open at the line's end, a function's opening brace — and no reading
/// of one line on its own can see that. Declining any construct beyond simple commands already covers every
/// such opener, so a script carrying one contributes nothing at all, rather than contributing whatever its
/// remaining lines happen to spell. Continuations are joined by the repository's one reading of the shell's
/// rule, `kanhe::gate_identity::logical_lines`.
pub fn script_commands(script: &str, env: &BTreeMap<String, String>) -> Option<Vec<Vec<String>>> {
    let mut commands = Vec::new();
    for (_, line) in kanhe::gate_identity::logical_lines(script) {
        let read = words(&line, env)?;
        // The command word is the first word that is not an assignment prefix.
        let command = read.iter().find(|word| !is_assignment(word));
        // A line of assignments alone is the shell's own act too: it changes what later lines expand to, and
        // this reads every line in the environment the workflow declares.
        let assignment_only = !read.is_empty() && command.is_none();
        if assignment_only || command.is_some_and(|word| SHELL_OWN_WORDS.contains(&word.as_str())) {
            return None;
        }
        if !read.is_empty() {
            commands.push(read);
        }
    }
    Some(commands)
}

/// Whether `word` is an assignment the shell reads before a command word.
///
/// Bash's assignment words are `NAME=value`, `NAME+=value` and the array forms `NAME[i]=value` and
/// `NAME[i]+=value`. The array forms carry an unquoted `[`, which [`words`] declines before a word reaches this,
/// so the two scalar forms are the whole of what arrives here — and both are read, since an append changes what
/// a later expansion gives exactly as an assignment does.
fn is_assignment(word: &str) -> bool {
    word.split_once('=').is_some_and(|(name, _)| {
        let name = name.strip_suffix('+').unwrap_or(name);
        name.chars()
            .next()
            .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
            && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
    })
}
