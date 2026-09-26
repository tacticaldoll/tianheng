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
//! does not implement, so [`words`] returns `None` for the line rather than a guess. A caller reading a witness
//! therefore treats `None` as *not a witness*, and a caller reading a declaration treats it as unreadable.
//!
//! **Where one statement ends is bash's answer, taken from the lexer's own work.** [`statements`] cuts at a
//! newline the lexer left as an operator of the text itself, and nowhere else: a backslash-newline is already
//! gone by then, removed where bash removes it, so a line ending in an escaped backslash or in a backslash
//! inside single quotes ends its statement exactly where bash ends the command. A reader whose claim is about
//! which words belong to one command takes this split rather than walking continuations itself.

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

/// bash's reserved words, as `compgen -k` prints them — held against bash itself by
/// `the_reserved_words_are_the_ones_bash_owns`, both ways.
pub const RESERVED_WORDS: [&str; 22] = [
    "if", "then", "else", "elif", "fi", "case", "esac", "for", "select", "while", "until", "do",
    "done", "in", "function", "time", "{", "}", "!", "[[", "]]", "coproc",
];

/// The reserved words after which this reader does not treat the next word as a command: a `case` subject, a
/// loop's name and list, a function's name, a conditional expression. Other reserved words open a command
/// position here, as in `then cmd`, `! cmd`, and `{ cmd`. This reader does not model the optional name after
/// `coproc` or the `-p` option after `time`.
const RESERVED_BEFORE_A_NON_COMMAND: [&str; 7] =
    ["case", "for", "select", "in", "function", "[[", "]]"];

/// The indices of `words` that stand where a command's name does: the first word, the first after an operator
/// that ends or opens a command, and the first after a reserved word that begins one — past any assignment words,
/// which precede a command's name rather than being it.
///
/// A redirection's operator is followed by its target, not a command, so `<` and `>` open nothing. A reserved word
/// is one only where it stands in this position, unquoted, which is the one place bash reads it as syntax.
pub fn command_positions(words: &[Word]) -> Vec<usize> {
    let mut positions = Vec::new();
    // One reading per substitution depth: a substitution's words are a command list of their own, and the word
    // after it continues the list it stands in.
    let mut expecting: Vec<bool> = vec![true];
    let mut conditional: Vec<bool> = vec![false];
    let mut previous: Option<&Word> = None;
    for (index, word) in words.iter().enumerate() {
        let depth = word.depth;
        if expecting.len() <= depth {
            expecting.resize(depth + 1, true);
            conditional.resize(depth + 1, false);
        }
        // A substitution opened since the last word at this depth begins its own list.
        if previous.is_some_and(|prior| prior.depth < depth) {
            expecting[depth] = true;
            conditional[depth] = false;
        }
        previous = Some(word);
        if word.operator {
            // `>&` and `<&` are one redirection: its `&` opens no command.
            let redirects = index
                .checked_sub(1)
                .and_then(|before| words.get(before))
                .is_some_and(|before| {
                    before.operator
                        && matches!(before.value.as_str(), "<" | ">")
                        && before.start + 1 == word.start
                });
            if !conditional[depth] {
                expecting[depth] = !matches!(word.value.as_str(), "<" | ">") && !redirects;
            }
            continue;
        }
        // Inside `[[ … ]]` the words are a conditional expression, and `&&` or `||` there joins no commands.
        if conditional[depth] {
            if word.written == "]]" {
                conditional[depth] = false;
            }
            continue;
        }
        if !expecting[depth] || word.assigns().is_some() {
            continue;
        }
        positions.push(index);
        let reserved = word.written == word.value && RESERVED_WORDS.contains(&word.value.as_str());
        if reserved && word.value == "[[" {
            conditional[depth] = true;
        }
        expecting[depth] =
            reserved && !RESERVED_BEFORE_A_NON_COMMAND.contains(&word.value.as_str());
    }
    positions
}

/// The argv a command line runs, and how many of its leading words are assignments rather than the command.
///
/// **Which words are assignments is decided before expansion, as the shell decides it.** bash marks an assignment
/// word while parsing — unquoted `NAME=` or `NAME+=` opening a word that precedes the command name — and expands
/// afterwards, so `"$PREFIX" cargo` with `PREFIX=X=1` runs a command named `X=1`, while `X=1 cargo` runs `cargo`.
/// The two expand to the same words; this keeps them apart.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Argv {
    /// How many of [`words`](Argv::words) are the assignments preceding the command name.
    pub assignments: usize,
    /// Every word, expanded, the assignments first.
    pub words: Vec<String>,
}

impl Argv {
    /// The command name, where the line has one beyond its assignments.
    pub fn command(&self) -> Option<&str> {
        self.words.get(self.assignments).map(String::as_str)
    }
}

/// The argv `line` runs, with `env` expanded — or `None` for a line whose words depend on its running.
///
/// A projection of [`lex`]: the line is split exactly as the exit-class check splits a script, and this declines
/// any word carrying a part it does not expand — an operator, a substitution, ANSI-C quoting, a special or
/// compound parameter, an unterminated quote, or an unquoted glob, brace or tilde. An unquoted `#` at the start of
/// a word ends the line, as the shell's comment does; [`lex`] reads it so. The leading assignments are the words
/// [`Word::assigns`] names, up to the first that it does not, which is where bash's parser stops marking them.
pub fn words(line: &str, env: &BTreeMap<String, String>) -> Option<Argv> {
    expand(lex(line), env)
}

/// The argv one simple command's words run, or `None` for one [`words`] declines.
fn expand(lexed: Vec<Word>, env: &BTreeMap<String, String>) -> Option<Argv> {
    let mut read = Vec::new();
    let mut assignments = 0;
    let mut prefix = true;
    for word in lexed {
        if word.operator {
            return None;
        }
        prefix &= word.assigns().is_some();
        let mut text = String::new();
        let mut quoted = false;
        for part in &word.parts {
            match part {
                Part::Literal {
                    text: literal,
                    quoted: was,
                } => {
                    quoted |= *was;
                    text.push_str(literal);
                }
                Part::Unquoted(literal) => {
                    if literal.chars().any(|c| "*?[]{}~\\".contains(c)) {
                        return None;
                    }
                    text.push_str(literal);
                }
                Part::Param { name, quoted: was } => {
                    let value = env.get(name)?;
                    // Unquoted, an expansion is split and globbed by the shell; a value that would be is declined.
                    if !was
                        && value
                            .chars()
                            .any(|v| v.is_whitespace() || "*?[".contains(v))
                    {
                        return None;
                    }
                    quoted |= *was;
                    text.push_str(value);
                }
                Part::Special(_)
                | Part::Compound(_)
                | Part::Substitution
                | Part::AnsiC(_)
                | Part::Arithmetic(_)
                | Part::Unterminated
                | Part::Unread(_) => return None,
            }
        }
        if prefix {
            assignments += 1;
        }
        // An unquoted expansion to nothing removes its word; a quoted one is an empty word.
        if quoted || !text.is_empty() {
            read.push(text);
        }
    }
    Some(Argv {
        assignments,
        words: read,
    })
}

/// The commands a script runs, one argv per command — or `None` when any command is one [`words`] declines,
/// or one whose command word is the shell's own.
///
/// **The script is the unit, not the line.** A line can make the lines after it data rather than commands — a
/// here-document's opener, a string left open at the line's end, a function's opening brace — and no reading
/// of one line on its own can see that. Declining any construct beyond simple commands already covers every
/// such opener, so a script carrying one contributes nothing at all, rather than contributing whatever its
/// remaining lines happen to spell.
///
/// The split is [`statements`]': lexed once, whole, and cut where bash cuts it — at a newline of the text's
/// own, never across a substitution that spans lines — so the continuation rule is the lexer's rather than a
/// second one run first.
pub fn script_commands(script: &str, env: &BTreeMap<String, String>) -> Option<Vec<Argv>> {
    let mut commands = Vec::new();
    for statement in statements(script).ok()? {
        let read = expand(statement.words, env)?;
        // A line of assignments alone is the shell's own act too: it changes what later lines expand to, and
        // this reads every line in the environment the workflow declares.
        let assignment_only = !read.words.is_empty() && read.command().is_none();
        if assignment_only
            || read
                .command()
                .is_some_and(|word| SHELL_OWN_WORDS.contains(&word))
        {
            return None;
        }
        if !read.words.is_empty() {
            commands.push(read);
        }
    }
    Some(commands)
}

/// One statement of a script: the words between two of the text's own newlines, with the line it begins on.
///
/// A command substitution's interior words stay in the statement, at their own [`depth`](Word::depth): the
/// substitution may span lines without splitting the command it stands in, and a reader scanning a
/// statement's words sees them in the order they begin.
pub struct Statement {
    /// The one-based line the statement's first word begins on.
    pub line: usize,
    /// The statement's words and operators, in the order each begins.
    pub words: Vec<Word>,
}

/// The statements of `script`, or the line and name of the first thing in it [`lex`] could not place.
///
/// The boundary is read off the lexer's work rather than walked again: a newline the lexer emitted as an
/// operator of the text itself ends a statement, and nothing else does. A backslash-newline is already
/// removed by then, as bash removes it; an escaped backslash or one inside single quotes is literal text, so
/// the statement ends at the newline exactly as bash ends the command there. Where a comment opens is the
/// lexer's own reading too — an unquoted `#` beginning a word — so a reader over statements takes no region
/// pass first, and a comment's text can no more carry a token into a statement than it can carry one into
/// bash.
pub fn statements(script: &str) -> Result<Vec<Statement>, (usize, &'static str)> {
    let mut out = Vec::new();
    let mut current: Vec<Word> = Vec::new();
    for word in lex_placed(script)? {
        if word.operator && word.depth == 0 && word.value == "\n" {
            if !current.is_empty() {
                let line = current[0].line;
                out.push(Statement {
                    line,
                    words: std::mem::take(&mut current),
                });
            }
        } else {
            current.push(word);
        }
    }
    if !current.is_empty() {
        let line = current[0].line;
        out.push(Statement {
            line,
            words: current,
        });
    }
    Ok(out)
}

/// One word of shell text, or one operator, where bash's own grammar puts the boundary.
///
/// **The boundaries are bash's definition, not a list grown to fit.** `bash(1)`, *DEFINITIONS*: a metacharacter
/// is space, tab, newline, `|`, `&`, `;`, `(`, `)`, `<` or `>`, and outside quotes each one ends a word; every one
/// but a blank also stands as an operator. So `(exit 3)`, `exit;` and `*) exit 3 ;;` put `exit` in a word of its
/// own, while `"(exit"` quotes its parenthesis into the word's text. A command substitution's contents are shell
/// text the shell runs, so they are read as words too, in order, wherever the substitution stands — unquoted,
/// inside double quotes, or between backquotes — while the substitution itself stays part of the word it stands
/// in, so `$(printf x)exit` is one word. A backslash before a newline joins the two lines, and an unquoted `#`
/// beginning a word is a comment to the end of its line.
///
/// **What a word is made of is kept, because its two readers need different things of it.** The exit-class check
/// reads a word's [`value`](Word::value), what quote removal leaves; [`words`] expands its parameters under a
/// supplied environment and declines what it cannot. One reading of the quoting serves both, so a quoting rule
/// corrected here is corrected for each.
pub struct Word {
    /// Where the word begins in the text, which orders words read out of a substitution after the one holding it.
    pub start: usize,
    /// The line the word begins on, counted from one.
    pub line: usize,
    /// The word as written, quotes included — what an `exit` argument is compared as.
    pub written: String,
    /// What quote removal leaves of it, a parameter kept as `$NAME` and a substitution as `$(…)` — what the shell
    /// runs as a command name when nothing is expanded.
    pub value: String,
    /// The word's parts, in order.
    pub parts: Vec<Part>,
    /// Whether it is a metacharacter standing as an operator rather than a word.
    pub operator: bool,
    /// How many command substitutions the word stands inside — `0` for the text's own words. Each substitution is a
    /// command list of its own, so where a command begins is asked of one depth at a time.
    pub depth: usize,
}

/// One part of a [`Word`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Part {
    /// Text the shell takes as written: quoted, or escaped by a backslash.
    Literal {
        /// The text itself, escapes resolved.
        text: String,
        /// Whether it stood inside quotes — which is what keeps an empty expansion a word.
        quoted: bool,
    },
    /// Unquoted text, whose glob, brace and tilde characters the shell would still expand.
    Unquoted(String),
    /// `$NAME` or `${NAME}`, inside double quotes or not.
    Param {
        /// The parameter's name, sigil and braces excluded.
        name: String,
        /// Whether it stood inside double quotes, which is what keeps its value one word.
        quoted: bool,
    },
    /// A special parameter — `$1`, `$@`, `$?` and the rest.
    Special(char),
    /// A `${…}` holding more than a name — a default, a substring, an indirection.
    Compound(String),
    /// A command substitution, `$(…)` or backquoted, whose own words are read beside this one.
    Substitution,
    /// ANSI-C quoting, `$'…'`, as bash decodes it.
    AnsiC(String),
    /// An arithmetic expansion `$((…))` or command `((…))`, as written between its parentheses. Its `<` and `>`
    /// are operators of arithmetic, not redirections, and it runs no command of its own.
    Arithmetic(String),
    /// A quote the text ends inside.
    Unterminated,
    /// A construct this reader does not place, and what it is — a here-document, whose body the lines after the
    /// operator hold, or a locale-translated `$"…"`, whose text is decided when the line runs. A reader seeing one
    /// declines the text rather than reading past it.
    Unread(&'static str),
}

/// Every word of `text`, or the line and name of the first thing in it [`lex`] could not place — for a reader
/// whose claim is about every word, where a word read past a misplaced one would be a word it never judged.
pub fn lex_placed(text: &str) -> Result<Vec<Word>, (usize, &'static str)> {
    let words = lex(text);
    match words
        .iter()
        .find_map(|word| word.unread().map(|what| (word.line, what)))
    {
        Some(unplaced) => Err(unplaced),
        None => Ok(words),
    }
}

/// bash's metacharacters, from `bash(1)`, *DEFINITIONS*.
const METACHARACTERS: [char; 10] = [' ', '\t', '\n', '|', '&', ';', '(', ')', '<', '>'];

/// Every word and operator of `text`, in order of where each begins.
pub fn lex(text: &str) -> Vec<Word> {
    let mut lexer = Lexer {
        chars: text.chars().collect(),
        at: 0,
        line: 1,
        out: Vec::new(),
        depth: 0,
    };
    lexer.run(None);
    let mut words = lexer.out;
    words.sort_by_key(|word| word.start);
    words
}

struct Lexer {
    chars: Vec<char>,
    at: usize,
    line: usize,
    out: Vec<Word>,
    /// The command substitutions the lexer is inside, which each word it emits records.
    depth: usize,
}

impl Word {
    /// The name this word assigns, where it is an assignment word — `NAME=value` or `NAME+=value`, the name and
    /// its `=` unquoted, as bash's parser marks one before anything is expanded.
    ///
    /// Bash's array forms, `NAME[i]=value`, carry a `[` that [`words`] declines before a word reaches its
    /// reader, so the two scalar forms are the whole of what is named here. Where the word stands is not asked:
    /// an argument spelled `NAME=value` is named too, and a reader that needs the position asks it separately.
    pub fn assigns(&self) -> Option<&str> {
        let Some(Part::Unquoted(text)) = self.parts.first() else {
            return None;
        };
        let (name, _) = text.split_once('=')?;
        let name = name.strip_suffix('+').unwrap_or(name);
        is_name(name).then_some(name)
    }

    /// The first thing in this word the lexer could not place, where there is one: a quote the text ends inside,
    /// or a construct it does not read.
    pub fn unread(&self) -> Option<&'static str> {
        self.parts.iter().find_map(|part| match part {
            Part::Unterminated => Some("a quote the text ends inside"),
            Part::Unread(what) => Some(*what),
            _ => None,
        })
    }

    fn new(start: usize, line: usize) -> Self {
        Self {
            start,
            line,
            written: String::new(),
            value: String::new(),
            parts: Vec::new(),
            operator: false,
            depth: 0,
        }
    }

    fn push(&mut self, part: Part) {
        self.value.push_str(&match &part {
            Part::Literal { text, .. } | Part::Unquoted(text) | Part::AnsiC(text) => text.clone(),
            Part::Param { name, .. } => format!("${name}"),
            Part::Special(c) => format!("${c}"),
            Part::Compound(inner) => format!("${{{inner}}}"),
            Part::Substitution => "$(…)".to_string(),
            Part::Arithmetic(inner) => format!("(({inner}))"),
            Part::Unterminated | Part::Unread(_) => String::new(),
        });
        // Adjacent text of one kind is one part, so a reader sees `Unquoted("abc")` and not three.
        match (self.parts.last_mut(), &part) {
            (Some(Part::Unquoted(prior)), Part::Unquoted(next)) => prior.push_str(next),
            (
                Some(Part::Literal {
                    text: prior,
                    quoted: a,
                }),
                Part::Literal {
                    text: next,
                    quoted: b,
                },
            ) if a == b => prior.push_str(next),
            _ => self.parts.push(part),
        }
    }
}

impl Lexer {
    fn emit(&mut self, mut word: Word) {
        word.depth = self.depth;
        self.out.push(word);
    }

    fn take(&mut self) -> Option<char> {
        let ch = *self.chars.get(self.at)?;
        self.at += 1;
        if ch == '\n' {
            self.line += 1;
        }
        Some(ch)
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.at).copied()
    }

    fn push(&mut self, word: Option<Word>) {
        if let Some(word) = word {
            self.emit(word);
        }
    }

    /// Reads words and operators until `close` stands unquoted — the end of the substitution this run reads —
    /// or the text ends. A `)` closes only at the depth it opened, so a subshell inside a substitution is its own.
    fn run(&mut self, close: Option<char>) {
        let mut word: Option<Word> = None;
        let mut depth = 0usize;
        loop {
            let (start, line) = (self.at, self.line);
            let Some(ch) = self.take() else { break };
            // A backslash-newline between or inside words is removed here, before the word it stands in is read, as
            // bash removes it: a `#` after one begins a word exactly as it would had the two lines been one. Inside
            // double quotes `double_quoted` removes it, and a `$` before one is left unplaced by `dollar`.
            if ch == '\\' && self.peek() == Some('\n') {
                self.take();
                continue;
            }
            // `((` opening a command is arithmetic, whose `<<` is a shift and whose parentheses are its own.
            if ch == '(' && self.peek() == Some('(') && word.is_none() {
                self.take();
                let mut arithmetic = Word::new(start, line);
                arithmetic.written.push_str("((");
                self.arithmetic(&mut arithmetic);
                self.emit(arithmetic);
                continue;
            }
            let closes = Some(ch) == close && (ch == '`' || depth == 0);
            if closes || METACHARACTERS.contains(&ch) {
                self.push(word.take());
                match ch {
                    '(' => depth += 1,
                    ')' => depth = depth.saturating_sub(1),
                    _ => {}
                }
                if ch != ' ' && ch != '\t' {
                    let mut operator = Word::new(start, line);
                    operator.written.push(ch);
                    operator.value.push(ch);
                    operator.operator = true;
                    // `<<` opens a here-document, whose body is the lines that follow rather than words; `<<<` is
                    // a here-string, which is an ordinary word after it.
                    let at = |offset: isize| {
                        self.at
                            .checked_add_signed(offset)
                            .and_then(|index| self.chars.get(index).copied())
                    };
                    if ch == '<' && at(-2) != Some('<') && at(0) == Some('<') && at(1) != Some('<')
                    {
                        operator.parts.push(Part::Unread(
                            "a here-document, whose body the lines after its operator hold",
                        ));
                    }
                    self.emit(operator);
                }
                if closes {
                    return;
                }
                continue;
            }
            // A `#` beginning a word opens a comment that runs to the end of its line.
            if ch == '#' && word.is_none() {
                while self.peek().is_some_and(|next| next != '\n') {
                    self.take();
                }
                continue;
            }
            let mut current = word.take().unwrap_or_else(|| Word::new(start, line));
            current.written.push(ch);
            match ch {
                '\\' => match self.take() {
                    Some(escaped) => {
                        current.written.push(escaped);
                        current.push(Part::Literal {
                            text: escaped.to_string(),
                            quoted: false,
                        });
                    }
                    None => current.push(Part::Unquoted("\\".to_string())),
                },
                '\'' => {
                    let mut text = String::new();
                    let mut closed = false;
                    while let Some(inner) = self.take() {
                        current.written.push(inner);
                        if inner == '\'' {
                            closed = true;
                            break;
                        }
                        text.push(inner);
                    }
                    current.push(Part::Literal { text, quoted: true });
                    if !closed {
                        current.push(Part::Unterminated);
                    }
                }
                '$' => self.dollar(&mut current, false),
                '`' => self.substitution(&mut current, '`'),
                '"' => self.double_quoted(&mut current),
                _ => current.push(Part::Unquoted(ch.to_string())),
            }
            word = Some(current);
        }
        self.push(word);
    }

    /// What a `$` opens, the `$` already read: a parameter, a special one, a compound `${…}`, a substitution, or
    /// ANSI-C quoting — or, followed by none of those, the `$` itself.
    fn dollar(&mut self, word: &mut Word, quoted: bool) {
        match self.peek() {
            Some('\'') if !quoted => {
                self.take();
                word.written.push('\'');
                let mut decoded = String::new();
                let mut closed = false;
                while let Some(inner) = self.take() {
                    word.written.push(inner);
                    match inner {
                        '\'' => {
                            closed = true;
                            break;
                        }
                        '\\' => {
                            let from = self.at;
                            let escape = ansi_c_escape(&self.chars, &mut self.at);
                            let consumed: String = self.chars[from..self.at].iter().collect();
                            self.line += consumed.matches('\n').count();
                            word.written.push_str(&consumed);
                            decoded.extend(escape);
                        }
                        _ => decoded.push(inner),
                    }
                }
                word.push(Part::AnsiC(decoded));
                if !closed {
                    word.push(Part::Unterminated);
                }
            }
            Some('"') if !quoted => {
                self.take();
                word.written.push('"');
                self.double_quoted(word);
                word.push(Part::Unread(
                    "a locale-translated `$\"…\"`, whose text is decided when the line runs",
                ));
            }
            Some('(') if self.chars.get(self.at + 1) == Some(&'(') => {
                self.take();
                self.take();
                word.written.push_str("((");
                self.arithmetic(word);
            }
            Some('(') => self.substitution(word, ')'),
            Some('{') => {
                self.take();
                word.written.push('{');
                let from = self.at;
                let closed = self.braced(word, quoted);
                let end = if closed { self.at - 1 } else { self.at };
                let inner: String = self.chars[from..end].iter().collect();
                word.written.push_str(&inner);
                if closed {
                    word.written.push('}');
                }
                if !closed {
                    word.push(Part::Unterminated);
                } else if is_name(&inner) {
                    word.push(Part::Param {
                        name: inner,
                        quoted,
                    });
                } else {
                    word.push(Part::Compound(inner));
                }
            }
            Some(c) if c.is_ascii_alphabetic() || c == '_' => {
                let mut name = String::new();
                while let Some(c) = self
                    .peek()
                    .filter(|c| c.is_ascii_alphanumeric() || *c == '_')
                {
                    self.take();
                    word.written.push(c);
                    name.push(c);
                }
                word.push(Part::Param { name, quoted });
            }
            Some(c) if c.is_ascii_digit() || "@*#?$!-".contains(c) => {
                self.take();
                word.written.push(c);
                word.push(Part::Special(c));
            }
            // A `$` standing before nothing it opens is text — at the end, before a blank or a metacharacter, or
            // before a double quote's close. Before anything else it is a form this reader does not place, and a
            // `$` before a backslash-newline opens whatever follows the newline, so each is left unplaced.
            next if next.is_none_or(|c| {
                c.is_whitespace() || METACHARACTERS.contains(&c) || (quoted && c == '"')
            }) =>
            {
                if quoted {
                    word.push(Part::Literal {
                        text: "$".to_string(),
                        quoted: true,
                    });
                } else {
                    word.push(Part::Unquoted("$".to_string()));
                }
            }
            _ => word.push(Part::Unread(
                "a `$` before a character this reader does not read an expansion from",
            )),
        }
    }

    /// Reads a `${…}` to its closing brace, its `${` already read, and says whether it closed.
    ///
    /// **Where it ends is bash's rule, not the first `}`.** `bash(1)`, *Parameter Expansion*: the closing brace
    /// is the first one *not escaped by a backslash or within a quoted string, and not within an embedded
    /// arithmetic expansion, command substitution, or parameter expansion*. So a quote, a backslash, a `$` and a
    /// backquote inside are read as the rest of this lexer reads them, into a word that is discarded — the
    /// words of an embedded substitution are shell text the shell runs, and are kept like any other.
    fn braced(&mut self, outer: &mut Word, quoted: bool) -> bool {
        let mut inner = Word::new(self.at, self.line);
        while let Some(ch) = self.take() {
            match ch {
                // What the lexer could not place inside is the outer word's, or the reader would read past it.
                '}' => {
                    if let Some(what) = inner.unread() {
                        outer.push(Part::Unread(what));
                    }
                    return true;
                }
                '\\' => {
                    self.take();
                }
                // Measured on bash 5.2: `"${x:-'}'}"` expands to `'}'`, so a single quote protects a brace
                // inside a double-quoted expansion too.
                '\'' => while self.take().is_some_and(|inner| inner != '\'') {},
                '"' => self.double_quoted(&mut inner),
                '$' => self.dollar(&mut inner, quoted),
                '`' => self.substitution(&mut inner, '`'),
                _ => {}
            }
        }
        false
    }

    /// The rest of an arithmetic expansion or command, its `((` already read, to the `))` that closes it at depth.
    ///
    /// Arithmetic runs no command, so its text is kept whole rather than split into words — except that a command
    /// substitution inside one runs, and this does not read it, so one is a construct left unplaced.
    fn arithmetic(&mut self, word: &mut Word) {
        let mut inner = String::new();
        let mut depth = 0usize;
        let mut closed = false;
        let mut unplaced: Option<&'static str> = None;
        while let Some(ch) = self.take() {
            word.written.push(ch);
            match ch {
                '(' => depth += 1,
                ')' if depth == 0 && self.peek() == Some(')') => {
                    self.take();
                    word.written.push(')');
                    closed = true;
                    break;
                }
                // bash reads a `((` whose first close at depth is a single `)` as two nested subshells instead.
                ')' if depth == 0 => {
                    unplaced.get_or_insert(
                        "a `((` bash reads as nested subshells, its first close being single",
                    );
                }
                ')' => depth -= 1,
                // `$(` not doubled is a command substitution, which runs; `$((` is arithmetic nested in arithmetic.
                '$' if self.peek() == Some('(') && self.chars.get(self.at + 1) != Some(&'(') => {
                    unplaced.get_or_insert("a command substitution inside arithmetic");
                }
                // The alphabet arithmetic is written in: names, numbers, operators, brackets, blanks and `$`.
                c if c.is_ascii_alphanumeric()
                    || c.is_whitespace()
                    || "_+-*/%<>=!&|^~?:,[]{}$#@".contains(c) => {}
                _ => {
                    unplaced.get_or_insert(
                        "a character inside arithmetic this reader does not read — a quote, a backslash or a \
                         backquote",
                    );
                }
            }
            inner.push(ch);
        }
        word.push(Part::Arithmetic(inner));
        if !closed {
            word.push(Part::Unterminated);
        } else if let Some(what) = unplaced {
            word.push(Part::Unread(what));
        }
    }

    /// A command substitution, its `$` or opening backquote already read: **one** reading, quoted or not.
    ///
    /// Its contents are shell text the shell runs, read as words of their own. The substitution itself is part of
    /// the word it stands in, so text beside it joins that word — `$(printf x)exit` is one word, as it is to the
    /// shell — and the word's value is never `exit`.
    fn substitution(&mut self, word: &mut Word, close: char) {
        let from = self.at;
        if close == ')' {
            self.take();
        }
        let read_from = self.out.len();
        self.depth += 1;
        self.run(Some(close));
        self.depth -= 1;
        // A `case` pattern's `)` stands where the substitution's close would, and which one closes is the grammar
        // of `case` rather than of words — so a `case` inside `$(…)` is a construct left unplaced.
        let inner = &self.out[read_from..];
        if close == ')'
            && command_positions(inner)
                .into_iter()
                .any(|index| inner[index].value == "case" && inner[index].written == "case")
        {
            word.push(Part::Unread(
                "a `case` inside `$(…)`, whose `)` patterns this reader does not tell from the substitution's close",
            ));
        }
        word.written.extend(&self.chars[from..self.at]);
        word.push(Part::Substitution);
    }

    /// The rest of a double-quoted span, its opening quote already read.
    fn double_quoted(&mut self, word: &mut Word) {
        while let Some(ch) = self.take() {
            word.written.push(ch);
            match ch {
                '"' => return,
                '\\' => {
                    if let Some(escaped) = self.take() {
                        word.written.push(escaped);
                        // Inside double quotes a backslash escapes only these; before anything else it stays.
                        let text = match escaped {
                            '\n' => String::new(),
                            '$' | '`' | '"' | '\\' => escaped.to_string(),
                            other => format!("\\{other}"),
                        };
                        word.push(Part::Literal { text, quoted: true });
                    }
                }
                '$' => self.dollar(word, true),
                '`' => self.substitution(word, '`'),
                _ => word.push(Part::Literal {
                    text: ch.to_string(),
                    quoted: true,
                }),
            }
        }
        word.push(Part::Unterminated);
    }
}

fn is_name(text: &str) -> bool {
    text.chars()
        .next()
        .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
        && text.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// The character one ANSI-C escape names, read from just after its backslash; `at` is left past the escape.
///
/// **The whole of bash's list, because it is finite.** `bash(1)`, *QUOTING*, names every form: the single
/// letters, `\\`, `\'`, `\"`, `\?`, octal `\nnn`, hex `\xHH`, `\uHHHH`, `\UHHHHHHHH` and control `\cx`. A
/// reader decoding some of them is one spelling short of `exit` for as long as the rest exist, so none is left
/// out. Measured: `$'\x65xit'`, `$'\145xit'`, `$'\u0065xit'` and `$'e\x78it'` each run `exit` under bash 5.
/// An escape bash leaves as written — a backslash before any other character — is both characters.
fn ansi_c_escape(chars: &[char], at: &mut usize) -> Vec<char> {
    let Some(&first) = chars.get(*at) else {
        return vec!['\\'];
    };
    *at += 1;
    let digits = |at: &mut usize, radix: u32, most: usize| {
        let start = *at;
        while *at < chars.len() && *at - start < most && chars[*at].is_digit(radix) {
            *at += 1;
        }
        let text: String = chars[start..*at].iter().collect();
        let decoded = u32::from_str_radix(&text, radix)
            .ok()
            .and_then(char::from_u32);
        // A value that names no character leaves its digits to be read as the text they are.
        if decoded.is_none() {
            *at = start;
        }
        decoded
    };
    let named = match first {
        'a' => Some('\u{7}'),
        'b' => Some('\u{8}'),
        'e' | 'E' => Some('\u{1b}'),
        'f' => Some('\u{c}'),
        'n' => Some('\n'),
        'r' => Some('\r'),
        't' => Some('\t'),
        'v' => Some('\u{b}'),
        '\\' | '\'' | '"' | '?' => Some(first),
        _ => None,
    };
    if let Some(named) = named {
        return vec![named];
    }
    let decoded = match first {
        '0'..='7' => {
            // The first octal digit is one of the up to three `digits` reads, so it is given back to it.
            *at -= 1;
            digits(at, 8, 3)
        }
        'x' => digits(at, 16, 2),
        'u' => digits(at, 16, 4),
        'U' => digits(at, 16, 8),
        'c' => chars.get(*at).map(|&control| {
            *at += 1;
            char::from(control as u8 & 0x1f)
        }),
        _ => None,
    };
    decoded.map_or_else(|| vec!['\\', first], |decoded| vec![decoded])
}
